extern crate sdl2;

use std::fs;
use rand::Rng;
// use console_engine::pixel;
// use console_engine::KeyCode;
use std::time::{Duration, Instant};
use sdl2::rect::Rect;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {

    let filename = "programs/PONG2";
    let mut ch8: Cpu = Cpu::new();

    ch8.load_program(filename);
    ch8.load_sprites();
    ch8.run_program();
    
    //ch8.display_program_opcodes();
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
    display: [u8; 64 * 32],      // Each byte represent a pixel (Supposed to be 1 bit = 1 pixel)
    prog_length: usize,
    draw_flag: u8,
    last_time: Instant,
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
            prog_length: 0,
            draw_flag: 0,
            last_time: Instant::now(),
        };
    }

    fn load_sprites(self: &mut Cpu) {
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

    fn load_program(self: &mut Cpu, file: &str) {

        let program_bytes = fs::read(file).unwrap();
        for (i, byte) in (&program_bytes).into_iter().enumerate(){
            self.memory[512 + i] = *byte;
        }
        self.prog_length = program_bytes.len();
    }

    // fn display_program_opcodes(self: &mut Self) {
        
    //     let mut i = 0;
    //     let mut op;
    //     while i < self.prog_length {
    //         op = Opcode::new(&[self.memory[512 + i], self.memory[512 + i + 1]]);
    //         op.display();
    //         i += 2;
    //     }
    // }

    // fn draw(self: &mut Self, engine: &mut console_engine::ConsoleEngine) {
    //     engine.clear_screen();
    //     let mut pos;
    //     for x in 0..64 as i32 {
    //         for y in 0..32 as i32{
    //             pos = ((64 * y) + x) as usize;
    //             if self.display[pos] == 1 {
    //                 engine.set_pxl(x, y, pixel::pxl('*'));
    //             }
    //             else {
    //                 engine.set_pxl(x, y, pixel::pxl(' '));
    //             }
    //         }
    //     }
    //     engine.draw();
    // }

    fn run_program(self: &mut Self) {

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 64*8, 32*8)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window             // Canvas is the renderer
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();     

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGB332, 64, 32)
            .map_err(|e| e.to_string()).unwrap();

        texture.set_color_mod(255, 255, 255);

        let mut event_pump = sdl_context.event_pump().unwrap();
        
        let mut opcode;
        let mut oparray: [u8; 2];
        self.last_time = Instant::now();

        'running: loop {

            oparray = [
                    self.memory[usize::from(self.program_counter)],
                    self.memory[usize::from(self.program_counter) + 1]
            ];
            opcode = Opcode::new(&oparray);
            self.execute(&opcode);

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running      // Specifies which loop to break from
                    },
                    _ => {}
                }
            }
            // Update the display if needed
            if self.draw_flag == 1 {
                // draw 
                canvas.clear();                                                 // Clear the buffer
                texture.update(None, &(self.display), 64).unwrap();              // Update texture
                canvas.copy(&texture, None, Some(Rect::new(0, 0, 64*8, 32*8))).unwrap();                     // Copy the texture into the canvas
                canvas.present();                                               // Present canvas
                self.draw_flag = 0;
            }
            std::thread::sleep(Duration::new(0, 500));        // had :: at the start originally
        }


        // let mut engine = console_engine::ConsoleEngine::init(64, 32, 60).unwrap();
    
        // while (self.program_counter >= 512) && (self.program_counter <= (512 + self.prog_length as u16)) {
            
        //     oparray = [
        //             self.memory[usize::from(self.program_counter)],
        //             self.memory[usize::from(self.program_counter) + 1]
        //     ];
        //     opcode = Opcode::new(&oparray);
        //     self.execute(&opcode);
            
        //     if self.draw_flag == 1 {
        //         engine.wait_frame();                                 
        //         if engine.is_key_pressed(KeyCode::Char('q')) {       // if the user presses 'q' :
        //             break; // exits app
        //         }
        //         self.draw(&mut engine);
        //         self.draw_flag = 0;
        //     }
        // }
    }

    fn execute(self: &mut Self, op: &Opcode) {

        let new_time = Instant::now();
        let elasped_time = new_time.duration_since(self.last_time);
        if  elasped_time.as_millis() > 16 {                                 // 1/60hz is 16.6666ms
            self.last_time = new_time;
            if self.delay_timer > 0 { self.delay_timer -= 1; }
            if self.sound_timer > 0 { self.sound_timer -= 1; }
        }

        self.program_counter += 2;

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
                    _ => {
                        //print!("Not neccessary for most ROMs: ");
                        //op.display();
                    }
                }
            },
            0x1 => {
                // Jump to address NNN from Opcode 1NNN
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
                //print!("program_counter: {:04X}", self.program_counter);
                //println!(" - Jump to address 1NNN: ");
            },
            0x2 => {
                // Calls subroutine at NNN from Opcode 2NNN
                // Push current program_counter so we know where the subroutine was called from
                self.stack[self.stack_counter] = self.program_counter;  
                self.stack_counter += 1;

                // Set the program_counter to where we need to go for the subroutine
                self.program_counter = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];

                //print!("program_counter: {:04X}", self.program_counter);
                //println!(" - Call subroutine at 2NNN: ");
            },
            0x3 => {
                // Opcode represents 3XNN
                // Skips next instruction if registers[X] == NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] == nn {
                    //println!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2; // instructions are 2 bytes
                }
                else {
                    //println!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
            },
            0x4 => {
                // Opcode represents 4XNN
                // Skips next instruction if registers[X] != NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] != nn {
                    //println!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;
                }
                else {
                    //println!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
            },
            0x5 => {
                // Opcode represents 5XY0
                // Skips next instruction if registers[X] == registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] == self.registers[y] {
                    //println!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    //println!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
            },
            0x6 => {
                // 6XNN where we set registers[X] = NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[x] = nn;
                //println!("Setting register[{}] to {:02X} - ", x, nn);
            },
            0x7 => {
                // 7XNN where we set registers[X] = registers[X] + NN, do not set carry flag
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u16;
                let xnn = u16::from(self.registers[x]) + nn;            // Rust panics due to overflow
                self.registers[x] = (xnn & 0x00FF) as u8;               // So we just truncate afterwards instead
                //println!("Adding {:02X} to register[{}] - ", nn, x);
            },
            0x8 => {
                match op.digits[1..=3] {
                    [x, y, 0x0] => {
                        // Set register[x] to register[y]
                        self.registers[usize::from(x)] = self.registers[usize::from(y)];
                        //println!("Setting register[{}] to registers[{}] - ", x, y);
                    },
                    [x, y, 0x1] => {
                        // Set register[x] to register[x] OR register[y]
                        self.registers[usize::from(x)] |= self.registers[usize::from(y)];
                        //println!("OR operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x2] => {
                        // Set register[x] to register[x] AND register[y]
                        self.registers[usize::from(x)] &= self.registers[usize::from(y)];
                        //println!("AND operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x3] => {
                        // Set register[x] to register[x] XOR register[y]
                        self.registers[usize::from(x)] ^= self.registers[usize::from(y)];
                        //println!("XOR operation on register[{}] with registers[{}] - ", x, y);
                    },
                    [x, y, 0x4] => {
                        // Set register[x] to register[x] + register[y], set carry if needed
                        let regx = u16::from(self.registers[usize::from(x)]);
                        let regy = u16::from(self.registers[usize::from(y)]);
                        let result = regx + regy;
                        self.registers[0xF] = u8::from(result > 255);
                        self.registers[usize::from(x)] = result as u8;  // When it overflows, the result would be useless?                   
                        
                        //println!("register[{}] + registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    [x, y, 0x5] => {
                        // Set register[x] to register[x] - register[y], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regx.overflowing_sub(regy);   // May not be correct
                        self.registers[0xF] = u8::from(regx > regy);    // 1 when no borrow  
                        self.registers[usize::from(x)] = result as u8;

                        //println!("register[{}] - registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    [x, _y, 0x6] => {
                        // If LSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then divide reg[X] by 2
                        self.registers[0xF] = self.registers[usize::from(x)] & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] >> 1;
                        //println!("Set flag to LSB: {} and divided by 2 - ", self.registers[0xF]);
                    },
                    [x, y, 0x7] => {
                        // Set register[x] to register[y] - register[x], set flag if underflow
                        let regx = self.registers[usize::from(x)];
                        let regy = self.registers[usize::from(y)];
                        let (result, _) = regy.overflowing_sub(regx);   // May not be correct
                        self.registers[0xF] = u8::from(regy > regx);    // 1 when no borrow
                        self.registers[usize::from(x)] = result as u8;

                        //println!("register[{}] - registers[{}] flag: {} - ", y, x, self.registers[0xF]);
                    },
                    [x, _y, 0xE] => {
                        // If MSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then multiply reg[X] by 2
                        self.registers[0xF] = (self.registers[usize::from(x)] >> 7) & 0x01;
                        self.registers[usize::from(x)] = self.registers[usize::from(x)] << 1;
                        //println!("Set flag to MSB: {} and multiplied by 2 - ", self.registers[0xF]);
                    },
                    _ => {
                        //print!("Opcode does not exist in spec - ");
                        //op.display();
                    }
                }
            },
            0x9 => {
                // Opcode represents 9XY0
                // Skips next instruction if registers[X] != registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] != self.registers[y] {
                    //println!("Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                    self.program_counter += 2;      // instructions are 2 bytes
                }
                else {
                    //println!("Not Skipping instruction: {:04X} - Opcode: ", self.program_counter);
                }
            },
            0xA => {
                // Opcode is ANNN, set addr register to NNN
                self.address_register = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
                //println!("Set Address Register: {:04X}", self.address_register);
            },
            0xB => {
                // Opcode is BNNN, jump to NNN + reg[0]
                let reg0 = u16::from(self.registers[0]);
                self.program_counter = ((op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3]) + reg0;
                //println!("Jump! Program Counter: {:04X}", self.program_counter);
            },
            0xC => {
                // Opcode is CXKK, set reg[X] to random byte AND KK
                let kk = ((op.digits[2] << 4) | op.digits[3]) as u8;
                self.registers[usize::from(op.digits[1])] = kk & random_byte();
                //println!("Set register[x] based on random byte - ");
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
                        if (bit == 1) && (self.display[pos] == 1){
                            self.registers[15] = 1;
                        }
                        self.display[pos] = self.display[pos] ^ bit;
                    }
                }
                self.draw_flag = 1;
                //println!("Updated the display, Collision Flag: {} - ", collisions);
            },
            0xE => {
                match op.digits[1..=3] {
                    [x, 0x9, 0xE] => {
                        // If key with value reg[x] is pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) == 1 {
                            self.program_counter += 2;
                        }
                        //println!("x: {:02X} Key: {:02X} is pressed so we skip instruction - ", x, regx);
                    },
                    [x, 0xA, 0x1] => {
                        // If key with value reg[x] is NOT pressed, skip next instruction
                        let regx = self.registers[usize::from(x)];
                        if ((self.keyboard >> regx) & 0x01) != 1 {
                            self.program_counter += 2;
                        }
                        //println!("x: {:02X} Key: {:02X} is not pressed so we skip instruction - ", x, regx);
                    },
                    _ => {
                        //print!("Doesnt exist in the spec: ");
                        //op.display();
                    },
                }
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
                        //print!("Doesnt exist in the spec: ");
                        //op.display();
                    },
                }
            },
            _ => {
                //print!("Not yet supported: ");
                //op.display();
            }
        }
    }
}

#[derive(Clone)]
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
    let num = rand::thread_rng().gen_range(0..255);
    return num;
}