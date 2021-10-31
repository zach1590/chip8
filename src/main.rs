use std::fs;

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
    keyboard: u8,           // Only need 16 keys so this is overkill (only need the lowest four bits)
    display: [u64; 32]      // [[u8; 64]; 32]
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
            display: [0; 32],
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
                        self.address_register = self.stack[self.stack_counter];
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
                self.address_register = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];
                print!("address_register: {:04X}", self.address_register);
                print!(" - Jump to address 1NNN: ");               
                op.display();
            },
            0x2 => {
                // Calls subroutine at NNN from Opcode 2NNN
                // Push current address_register so we know where the subroutine was called from
                self.stack[self.stack_counter] = self.address_register;  
                self.stack_counter += 1;

                // Set the address_register to where we need to go for the subroutine
                self.address_register = (op.digits[1] << 8) | (op.digits[2] << 4) | op.digits[3];

                print!("address_register: {:04X}", self.address_register);
                print!(" - Call subroutine at 2NNN: ");               
                op.display();
            },
            0x3 => {
                // Opcode represents 3XNN
                // Skips next instruction if registers[X] == NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] == nn {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.address_register);
                    self.address_register += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.address_register);
                }
                op.display();
            },
            0x4 => {
                // Opcode represents 4XNN
                // Skips next instruction if registers[X] != NN
                let x = usize::from(op.digits[1]);
                let nn = ((op.digits[2] << 4) | op.digits[3]) as u8;
                if self.registers[x] != nn {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.address_register);
                    self.address_register += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.address_register);
                }
                op.display();
            },
            0x5 => {
                // Opcode represents 5XY0
                // Skips next instruction if registers[X] == registers[Y]
                let x = usize::from(op.digits[1]);
                let y = usize::from(op.digits[2]);
                if self.registers[x] == self.registers[y] {
                    print!("Skipping instruction: {:04X} - Opcode: ", self.address_register);
                    self.address_register += 2;      // instructions are 2 bytes
                }
                else {
                    print!("Not Skipping instruction: {:04X} - Opcode: ", self.address_register);
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
                        self.registers[0xF] = u8::from(regx > regy);    // 1 when no borrow, video did differently    
                        self.registers[usize::from(x)] = result as u8;

                        print!("register[{}] - registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    [x, y, 0x6] => {
                        // If LSB of reg[X] is 1, then reg[F] = 1 otherwise 0, then divide reg[X] by 2
                        let regx = self.registers[usize::from(x)];
                        self.registers[0xF] = regx & 0x01;
                        self.registers[usize::from(x)] = regx >> 1;
                        print!("register[{}] - registers[{}] flag: {} - ", x, y, self.registers[0xF]);
                    },
                    _ => {
                        print!("Opcode does not exist in spec - ");
                    }
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