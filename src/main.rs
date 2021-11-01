use std::fs;
use rand::Rng;

fn main() {
    let filename = "programs/Pong.ch8";
    let mut ch8: Cpu = Cpu::new();

    let program_instructions = ch8.get_program_opcodes(filename);
    ch8.display_program_opcodes(&program_instructions);

    //println!("num of opcodes {}", program_instructions.len());
}

struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],        // v[] in the wiki
    address_register: u16,      // I in the wiki
    program_counter: u16,       // pc in the wiki
    stack: [u16; 24],
    stack_counter: usize,
    delay_timer: u8,
    sound_timer: u8,
    keyboard: u16,              // Each bit will represent a key (16 keys)
    waiting_for_key: bool,
    display: [u8; 64 * 32]      // Each byte represent a pixel (Supposed to be 1 bit = 1 pixel)
}

impl Cpu {
    fn new() -> Cpu {
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
            waiting_for_key: false,
            display: [0; 64*32],
        };
    }

    fn get_program_opcodes(self: &Cpu, file: &str) -> Vec<Opcode> {

        let program_bytes = fs::read(file).unwrap();
        let mut program_opcodes: Vec<Opcode> = Vec::new(); 
        let mut i = 0;

        while i < program_bytes.len() {
            program_opcodes.push(Opcode::new(&program_bytes[i..=i+1]));
            i += 2;
        }
        return program_opcodes;
    }

    fn display_program_opcodes(self: &mut Self, program: &Vec<Opcode>) {
        for op in program {
            self.execute(&op);
        }
    }

    fn execute(self: &mut Self, op: &Opcode) {
        
        // Do I need to increment the instruction/program counter here ???
        


        match op.digits[0] {
            0x0 => {
                match op.digits[1..=3] {
                    [0x0, 0xE, 0x0] => {
                        print!("Clears the Screen: ");
                        for i in 0..self.display.len() {
                            self.display[i] = 0;
                        }
                        op.display();
                    },
                    [0x0, 0xE, 0xE] => {
                        print!("Return from Subroutine: ");
                        //self.stack_counter -= 1;
                        self.program_counter = self.stack[self.stack_counter];
                        self.stack[self.stack_counter] = 0;
                        op.display();
                    },
                    _ => {
                        print!("Not neccessary for most ROMs: ");
                        op.display();
                    }
                }
            },
            0x1 => {
                // Jump to address NNN from Opcode 1NNN
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
                print!("program_counter: {:04X}", self.program_counter);
                print!(" - Jump to address 1NNN: ");               
                op.display();
            },
            0x2 => {
                // Calls subroutine at NNN from Opcode 2NNN
                // Push current program_counter so we know where the subroutine was called from
                self.stack[self.stack_counter] = self.program_counter;  
                self.stack_counter += 1;

                // Set the program_counter to where we need to go for the subroutine
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];

                print!("program_counter: {:04X}", self.program_counter);
                print!(" - Call subroutine at 2NNN: ");               
                op.display();
            },
            0x3 => {
                // Opcode represents 3XNN
                // Skips next instruction if registers[X] == NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] == nn {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
                op.display();
            },
            0x4 => {
                // Opcode represents 4XNN
                // Skips next instruction if registers[X] != NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] != nn {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
                op.display();
            },
            0x5 => {
                // Opcode represents 5XY0
                // Skips next instruction if registers[X] == registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] == self.registers[y] {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
                op.display();
            },
            0x6 => {
                // 6XNN where we set registers[X] = NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[x] = nn;
                print!("Setting register[{}] to {:02X} - ", x, nn);
                op.display();
            },
            0x7 => {
                // 7XNN where we set registers[X] = registers[X] + NN, do not set carry flag
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u16;
                let xnn = u16::from(self.registers[x]) + nn;            // Rust panics due to overflow
                self.registers[x] = xnn as u8;                          // So we just truncate afterwards instead
                print!("Adding {:02X} to register[{}] - ", nn, x);
                op.display();
            },
            0x8 => {
                match op.digits[1..=3] {
                    [x, y, 0x0] => {
                        // Set register[x] to register[y]
                        self.registers[usize::from(x)] = self.registers[usize::from(y)];
                        print!("Setting register[{}] to registers[{}] - ", x, y);
                    },
                    [x, y, 0x1] => {
                        // Set register[x] to register[x] OR register[y]
                        self.registers[usize::from(x)] |= self.registers[usize::from(y)];
                        print!("OR operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x2] => {
                        // Set register[x] to register[x] AND register[y]
                        self.registers[usize::from(x)] &= self.registers[usize::from(y)];
                        print!("AND operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x3] => {
                        // Set register[x] to register[x] XOR register[y]
                        self.registers[usize::from(x)] ^= self.registers[usize::from(y)];
                        print!("XOR operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x4] => {
                        // Set register[x] to register[x] + register[y], set carry if needed
                        let regx = u16::from(self.registers[usize::from(x)]);
                        let regy = u16::from(self.registers[usize::from(y)]);
                        let result = regx + regy;
                        self.registers[0xF] = u8::from(result > 255);
                        self.registers[usize::from(x)] = result as u8;  // When it overflows, the result would be useless?                   
                        
                        print!("register[{}] + registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    [x, y, 0x5] => {
                        // Set register[x] to register[x] - register[y], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regx.overflowing_sub(regy);   // May not be correct
                        self.registers[0xF] = u8::from(regx > regy);    // 1 when no borrow  
                        self.registers[usize::from(x)] = result as u8;

                        print!("register[{}] - registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    [x, _y, 0x6] => {
                        // If LSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then divide reg[X] by 2
                        self.registers[0xF] = self.registers[usize::from(x)] & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] >> 1;
                        print!("Set flag to LSB: {} and divided by 2 - ", self.registers[0xF]);
                    },
                    [x, y, 0x7] => {
                        // Set register[x] to register[y] - register[x], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regy.overflowing_sub(regx);   // May not be correct
                        self.registers[0xF] = u8::from(regy > regx);    // 1 when no borrow
                        self.registers[usize::from(x)] = result as u8;

                        print!("register[{}] - registers[{}] flag: {} - ", y, x, self.registers[0xF]);
                    },
                    [x, _y, 0xE] => {
                        // If MSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then multiply reg[X] by 2
                        self.registers[0xF] = (self.registers[usize::from(x)] >> 7) & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] << 1;
                        print!("Set flag to MSB: {} and multiplied by 2 - ", self.registers[0xF]);
                    },
                    _ => {
                        print!("Opcode does not exist in spec - ");
                    }
                }
                op.display();
            },
            0x9 => {
                // Opcode represents 9XY0
                // Skips next instruction if registers[X] != registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] != self.registers[y] {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
                op.display();
            },
            0xA => {
                // Opcode is ANNN, set addr register to NNN
                self.address_register = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
                print!("Set Address Register: {:04X}", self.address_register);
                op.display();
            },
            0xB => {
                // Opcode is BNNN, jump to NNN + reg[0]
                let reg0 = u16::from(self.registers[0]);
                self.program_counter = ((op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3]) + reg0;
                print!("Jump! Program Counter: {:04X}", self.program_counter);
                op.display();
            },
            0xC => {
                // Opcode is CXKK, set reg[X] to random byte AND KK
                let kk = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[usize::from(op.digits[1])] = kk & random_byte();
                print!("Set register[x] based on random byte - ");
                op.display();
            },
            0xD => {
                // Opcode is DXYN, Display N-byte sprite starting at address_register
                // Place the sprite starting from reg[X], reg[Y] and set reg[F]=1 if any bit is erased
                let i = usize::from(self.address_register);
                let n = usize::from(op.digits[3]);
                let x = usize::from(self.registers[usize::from(op.digits[1])]);
                let y = usize::from(self.registers[usize::from(op.digits[2])]);
                let sprite = &self.memory[i..i+n];

                let mut pixel;                                              // Pixel for sprite
                let mut collisions = 0x00;                                  // Initially no pixels have been erased yet
                let mut pos;                                                // For indexing the display array
                for (idy, byte) in sprite.iter().enumerate() {              // Each byte goes on next row of display
                    for idx in 0..8 {                                       
                        pixel = *byte >> ((7 - idx) & 0x1);                  // Get each bit of the byte
                        pos = (64 * (y + idy)) + x + idx;
                        collisions |= u8::from((pixel == 1) && (self.display[pos] == 1));
                        self.display[pos] ^= pixel;                         // We display through XOR
                    }
                }
                self.registers[0xF] = collisions;
                print!("Updated the display, Collision Flag: {} - ", collisions);
                op.display();
            },
            0xE => {
                match op.digits[1..=3] {
                    [x, 0x9, 0xE] => {
                        // If key with value reg[x] is pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) == 1 {
                            self.program_counter += 2;
                        }
                        print!("x: {:02X} Key: {:02X} is pressed so we skip instruction - ", x, regx);
                    },
                    [x, 0xA, 0x1] => {
                        // If key with value reg[x] is NOT pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) != 1 {
                            self.program_counter += 2;
                        }
                        print!("x: {:02X} Key: {:02X} is not pressed so we skip instruction - ", x, regx);
                    },
                    _ => {
                        print!("Doesnt exist in the spec: ");
                    },
                }
                op.display();
            },
            0xF => {
                match op.digits[1..=3] {
                    [x, 0x0, 0x7] => {
                        self.registers[usize::from(x)] = self.delay_timer;
                    },
                    [x, 0x0, 0xA] => {
                        self.waiting_for_key = true;
                        if self.waiting_for_key {
                            let keypress = 0;   // take in a keypress here
                            self.keyboard = self.keyboard | (0x01 << keypress);
                            self.registers[usize::from(x)] = 0x01 << keypress;
                            self.waiting_for_key = false;
                        }
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
                    _ => {
                        print!("Doesnt exist in the spec: ");
                    },
                }
                op.display();
            },
            _ => {
                print!("Not yet supported: ");
                op.display();
            }
        }
    }
}

struct Opcode {
    digits: [u16; 4],
}

impl Opcode {
    fn new(opcode: &[u8]) -> Opcode {
        return Opcode {
            digits: [
                u16::from((opcode[0]) & 0xF0) >> 4,
                u16::from(opcode[0]) & 0x0F,
                u16::from((opcode[1]) & 0xF0) >> 4,
                u16::from(opcode[1]) & 0x0F,
            ]
        };
    }

    fn display(self: &Self){
        println!("{:01X}{:01X}{:01X}{:01X}", 
                    self.digits[0], self.digits[1], self.digits[2], self.digits[3]);
    }
}

fn random_byte() -> u8 {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen();
    return n1;
}