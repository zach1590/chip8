use std::fs;

fn main() {
    let filename = "programs/Pong.ch8";
    let mut ch8: Cpu = Cpu::new();

    let program_instructions = ch8.get_program_opcodes(filename);
    ch8.display_program_opcodes(&program_instructions);
}

struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    program_counter: u16,
    stack: [u16; 24],
    stack_counter: u8,
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
            program_counter: 0,
            stack: [0; 24],
            stack_counter: 0,
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

    fn display_program_opcodes(self: &Self, program: &Vec<Opcode>) {
        for op in program {
            op.display();
            if 0x2 == op.digits[0] { 
                println!("First Digit is 2");
            }
            if 0x2 == op.digits[1] { 
                println!("Second Digit is 2");
            }
        }
    }
}

struct Opcode {
    digits: [u8; 4],
}

impl Opcode {
    fn new(opcode: &[u8]) -> Opcode {
        return Opcode {
            digits: [
                ((opcode[0]) & 0xF0) >> 4,
                (opcode[0]) & 0x0F,
                ((opcode[1]) & 0xF0) >> 4,
                (opcode[1]) & 0x0F,
            ]
        };
    }

    fn display(self: &Self){
        println!("{:01X}{:01X}{:01X}{:01X}", 
                    self.digits[0], self.digits[1], self.digits[2], self.digits[3]);
    }
}