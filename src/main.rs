use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut file = File::open("programs/Pong.ch8").unwrap();
    let mut ch8: Cpu = Cpu::new();
    ch8.display_program_opcodes(&mut file);
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

    fn display_program_opcodes(self: &Cpu, file: &mut File){
        // opcodes is of type [u8] (2 bytes represents an opcode)
        let mut opcodes = [0; 2];
        
        while file.read(&mut opcodes[..]).unwrap() > 0 {
            println!("{:02X}{:02X}", &opcodes[0], &opcodes[1]);
            let digit0 = ((opcodes[0]) & 0xF0) >> 4;
            let digit1 = (opcodes[0]) & 0x0F;
            if 0x2 == digit0 { 
                println!("First Digit is 2");
            }
            if 0x2 == digit1 { 
                println!("Second Digit is 2");
            }
        }
    }
}

/*
    Idea for handling opcodes without converting and storing in Hex
    let mut opcodes = [0; 2];
    file.read(&mut opcodes[..]).unwrap();
    
    // Gets the 4 hexadecimal digits on their own
    let digit0 = ((opcodes[0]) & 0xF0) >> 4;
    let digit1 = ((opcodes[0]) & 0x0F);
    let digit2 = ((opcodes[1]) & 0xF0) >> 4;
    let digit3 = ((opcodes[1]) & 0x0F);

    if 0x2 == digit0 && 0x2 == digit1 { }

*/