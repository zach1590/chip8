extern crate sdl2;
mod CPU;
mod keys;
mod opcode;

fn main() {

    let filename = "programs/PONG2";
    let mut ch8: CPU::Cpu = CPU::Cpu::new();

    ch8.load_program(filename);
    ch8.load_sprites();
    ch8.run_program();
    
    //ch8.display_program_opcodes();
    //println!("num of opcodes {}", program_instructions.len());
}


