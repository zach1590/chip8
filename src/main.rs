extern crate sdl2;
mod cpu;
mod keys;
mod opcode;
mod sound;

fn main() {

    let filename = "programs/BRIX";
    let mut ch8: cpu::Cpu = cpu::Cpu::new();

    ch8.load_program(filename);
    ch8.load_sprites();
    ch8.run_program();
    
    
}


