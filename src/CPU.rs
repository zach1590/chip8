use super::opcode::Opcode;
use std::fs;
use rand::Rng;
use std::time::Instant;

pub struct Cpu {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],        // v[] in the wiki
    pub address_register: u16,      // I in the wiki
    pub program_counter: u16,       // pc in the wiki
    pub stack: [u16; 24],           // For subroutine calls
    pub stack_counter: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keyboard: u16,              // Each bit will represent a key (16 keys)
    pub waiting_for_key_flag: bool,
    pub display: [u8; 64 * 32],      // Each byte represent a pixel (Supposed to be 1 bit = 1 pixel)       
    pub draw_flag: u8,              // Do we need to draw on this interation
    pub last_time: Instant,         // Keep track of time for the timers
}

impl Cpu {
    pub fn new() -> Cpu {
        return Cpu {
            memory: [0; 4096],
            registers: [0; 16],
            address_register: 0,
            program_counter: 0x200,         // 512 in decimal             
            stack: [0; 24],                 // Write a stack class? YES
            stack_counter: 0,               // Current stack level
            delay_timer: 0,
            sound_timer: 0,
            keyboard: 0,
            waiting_for_key_flag: false,
            display: [0; 64*32],
            draw_flag: 0,
            last_time: Instant::now(),
        };
    }

    pub fn load_sprites(self: &mut Cpu) {
        let sprites: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,     // 0
            0x20, 0x60, 0x20, 0x20, 0x70,     // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0,     // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0,     // 3
            0x90, 0x90, 0xF0, 0x10, 0x10,     // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0,     // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0,     // 6
            0xF0, 0x10, 0x20, 0x40, 0x40,     // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0,     // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0,     // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90,     // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0,     // B
            0xF0, 0x80, 0x80, 0x80, 0xF0,     // C
            0xE0, 0x90, 0x90, 0x90, 0xE0,     // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0,     // E
            0xF0, 0x80, 0xF0, 0x80, 0x80,     // F
        ];
        for (i, byte) in sprites.iter().enumerate() {
            self.memory[i] = *byte;
        }
    }

    pub fn load_program(self: &mut Cpu, file: &str) {

        let program_bytes = fs::read(file).unwrap();
        for (i, byte) in (&program_bytes).into_iter().enumerate(){
            self.memory[512 + i] = *byte;
        }
    }

    // For when FX0A was called we should end up in this function to take care if it
    // This function is called once we actually have a keypress to use. The main loop
    // takes care of pasuing execution to wait for a keypress
    pub fn key_pressed(self: &mut Self, key_val: u8) {
        let regx: u8 = self.memory[usize::from(self.program_counter) - 2];  // -2 because we want the fx0a instruction
        self.registers[usize::from(regx & 0x0F)] = key_val;
        self.waiting_for_key_flag = false;
    }

    pub fn execute(self: &mut Self, opcode: &[u8]) {

        let new_time = Instant::now();
        let elasped_time = new_time.duration_since(self.last_time);
        if  elasped_time.as_millis() >= 16 {                      // 1/60hz is 16.6666ms
            self.last_time = new_time;
            if self.delay_timer > 0 { self.delay_timer -= 1; }
            if self.sound_timer > 0 { self.sound_timer -= 1; }
        }

        let op = Opcode::new(&opcode);
        match op.digits[0] {
            0x0 => {
                match op.digits[1..=3] {
                    [0x0, 0xE, 0x0] => {
                        //println!("Clears the Screen: ");
                        for i in 0..self.display.len() {
                            self.display[i] = 0;
                        }
                        self.draw_flag = 1;
                    },
                    [0x0, 0xE, 0xE] => {
                        //println!("Return from Subroutine: ");
                        self.stack_counter -= 1;
                        self.program_counter = self.stack[self.stack_counter];
                        self.stack[self.stack_counter] = 0;
                    },
                    _ => { }
                }
            },
            0x1 => {
                // Jump to address NNN from Opcode 1NNN
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
            },
            0x2 => {
                // Calls subroutine at NNN from Opcode 2NNN
                // Push current program_counter so we know where the subroutine was called from
                self.stack[self.stack_counter] = self.program_counter;  
                self.stack_counter += 1;

                // Set the program_counter to where we need to go for the subroutine
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
            },
            0x3 => {
                // Opcode represents 3XNN
                // Skips next instruction if registers[X] == NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] == nn {
                    self.program_counter += 2; // instructions are 2 bytes
                }
            },
            0x4 => {
                // Opcode represents 4XNN
                // Skips next instruction if registers[X] != NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] != nn {
                    self.program_counter += 2;
                }
            },
            0x5 => {
                // Opcode represents 5XY0
                // Skips next instruction if registers[X] == registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] == self.registers[y] {
                    self.program_counter += 2;      // instructions are 2 bytes
                }
            },
            0x6 => {
                // 6XNN where we set registers[X] = NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[x] = nn;
            },
            0x7 => {
                // 7XNN where we set registers[X] = registers[X] + NN, do not set carry flag
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u16;
                let xnn = u16::from(self.registers[x]) + nn;            // Rust panics due to overflow
                self.registers[x] = (xnn & 0x00FF) as u8;               // So we just truncate afterwards instead
            },
            0x8 => {
                match op.digits[1..=3] {
                    [x, y, 0x0] => {
                        // Set register[x] to register[y]
                        self.registers[usize::from(x)] = self.registers[usize::from(y)];
                    },
                    [x, y, 0x1] => {
                        // Set register[x] to register[x] OR register[y]
                        self.registers[usize::from(x)] |= self.registers[usize::from(y)];
                    },
                    [x, y, 0x2] => {
                        // Set register[x] to register[x] AND register[y]
                        self.registers[usize::from(x)] &= self.registers[usize::from(y)];
                    },
                    [x, y, 0x3] => {
                        // Set register[x] to register[x] XOR register[y]
                        self.registers[usize::from(x)] ^= self.registers[usize::from(y)];
                    },
                    [x, y, 0x4] => {
                        // Set register[x] to register[x] + register[y], set carry if needed
                        let regx = u16::from(self.registers[usize::from(x)]);
                        let regy = u16::from(self.registers[usize::from(y)]);
                        let result = regx + regy;
                        self.registers[0xF] = u8::from(result > 255);
                        self.registers[usize::from(x)] = result as u8;
                    },
                    [x, y, 0x5] => {
                        // Set register[x] to register[x] - register[y], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regx.overflowing_sub(regy);   // May not be correct
                        self.registers[0xF] = u8::from(regx > regy);    // 1 when no borrow  
                        self.registers[usize::from(x)] = result as u8;
                    },
                    [x, _y, 0x6] => {
                        // If LSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then divide reg[X] by 2
                        self.registers[0xF] = self.registers[usize::from(x)] & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] >> 1;
                    },
                    [x, y, 0x7] => {
                        // Set register[x] to register[y] - register[x], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regy.overflowing_sub(regx);   // May not be correct
                        self.registers[0xF] = u8::from(regy > regx);    // 1 when no borrow
                        self.registers[usize::from(x)] = result as u8;
                    },
                    [x, _y, 0xE] => {
                        // If MSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then multiply reg[X] by 2
                        self.registers[0xF] = (self.registers[usize::from(x)] >> 7) & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] << 1;
                    },
                    _ => { }
                }
            },
            0x9 => {
                // Opcode represents 9XY0
                // Skips next instruction if registers[X] != registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;      // instructions are 2 bytes
                }
            },
            0xA => {
                // Opcode is ANNN, set addr register to NNN
                self.address_register = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
            },
            0xB => {
                // Opcode is BNNN, jump to NNN + reg[0]
                let reg0 = u16::from(self.registers[0]);
                self.program_counter = ((op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3]) + reg0;
            },
            0xC => {
                // Opcode is CXKK, set reg[X] to random byte AND KK
                let kk = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[usize::from(op.digits[1])] = kk & random_byte();
            },
            0xD => {
                // Opcode is DXYN, Display N-byte sprite starting at address_register
                // Place the sprite starting from reg[X], reg[Y] and set reg[F]=1 if any bit is erased
                let i = usize::from(self.address_register);
                let n = usize::from(op.digits[3]);
                let x = usize::from(self.registers[usize::from(op.digits[1])]);
                let y = usize::from(self.registers[usize::from(op.digits[2])]);
                
                let mut bit;                                                // Pixel for sprite
                self.registers[15] = 0;                                     // Initially no pixels have been erased yet
                let mut pos;                                                // For indexing the display array
                for idy in 0..n {

                    let byte: u8 = self.memory[i + idy];
                    for idx in 0..8 {

                        bit = (byte >> (7 - idx)) & 0x01;
                        pos = x + idx + (y + idy)*64;
                        if pos > 2047 { 
                            continue;
                        }
                        if (bit == 1) && (self.display[pos] == 0xFF) {
                            self.registers[15] = 1;
                        }
                        if bit == 1 { 
                            self.display[pos] = self.display[pos] ^ 0xFF;
                        }
                        else {
                            self.display[pos] = self.display[pos] ^ 0x00;
                        }
                    }
                }
                self.draw_flag = 1;
            },
            0xE => {
                match op.digits[1..=3] {
                    [x, 0x9, 0xE] => {
                        // If key with value reg[x] is pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) == 1 {
                            self.program_counter += 2;
                        }
                    },
                    [x, 0xA, 0x1] => {
                        // If key with value reg[x] is NOT pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) != 1 {
                            self.program_counter += 2;
                        }
                    },
                    _ => { },
                }
            },
            0xF => {
                match op.digits[1..=3] {
                    [x, 0x0, 0x7] => {
                        self.registers[usize::from(x)] = self.delay_timer;
                    },
                    [_x, 0x0, 0xA] => {
                        self.waiting_for_key_flag = true;
                    },
                    [x, 0x1, 0x5] => {
                        self.delay_timer = self.registers[usize::from(x)];
                    },
                    [x, 0x1, 0x8] => {
                        self.sound_timer = self.registers[usize::from(x)];
                    },
                    [x, 0x1, 0xE] => {
                        self.address_register += u16::from(self.registers[usize::from(x)]);
                    },
                    [x, 0x2, 0x9] => {
                        // Set I to location of sprite for digit reg[x]
                        // if reg[x] = 1, we want the sprite for 1
                        // Each sprite is 5 bytes long and they begin at mem location 0x0000
                        self.address_register = u16::from(self.registers[usize::from(x)]) * 5;
                    },
                    [x, 0x3, 0x3] => {
                        let i = usize::from(self.address_register);
                        let value = self.registers[usize::from(x)];
                        self.memory[i] = (value / 100) as u8;                   // The hundreds digit of reg[x]
                        self.memory[i + 1] = ((value % 100) / 10) as u8;        // The tens digit of reg[x]
                        self.memory[i + 2] = (value % 10) as u8;                // The ones digit of reg[x]
                    },
                    [x, 0x5, 0x5] => {
                        let i = usize::from(self.address_register);
                        let xpos = usize::from(x);
                        for pos in 0..=xpos {
                            self.memory[i + pos] = self.registers[pos];
                        }
                    },
                    [x, 0x6, 0x5] => {
                        let i = usize::from(self.address_register);
                        let xpos = usize::from(x);
                        for pos in 0..=xpos {
                            self.registers[pos] = self.memory[i + pos];
                        } 
                    },
                    _ => { },
                }
            },
            _ => { }
        }
    }
}

fn random_byte() -> u8 {
    let num = rand::thread_rng().gen_range(0..255);
    return num;
}

