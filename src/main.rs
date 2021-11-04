extern crate sdl2;
mod cpu;
mod keys;
mod opcode;
mod sound;

use sound::SoundSystem;
use std::time::{Duration, Instant};
use sdl2::rect::Rect;
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {

    let filename = "programs/BRIX";

    let mut chip8: cpu::Cpu = cpu::Cpu::new();
    chip8.load_program(filename);
    chip8.load_sprites();

    let sdl_context = sdl2::init().unwrap();                        // SDL for graphics, sound and input
    let video_subsystem = sdl_context.video().unwrap();             // Init Display
    let mut sound_system = SoundSystem::initialize(&sdl_context);   // Init Sound System
    let mut event_pump = sdl_context.event_pump().unwrap();         // Init Event System

    let window = video_subsystem.window("Rust-Chip8-Emulator", 64*8, 32*8)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window                                        // Canvas is the renderer
        .into_canvas()
        .accelerated()
        .build()
        .unwrap();     

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::RGB332, 64, 32)
        .map_err(|e| e.to_string()).unwrap();

    sound_system.device.resume();
    chip8.last_time = Instant::now();
    let mut oparray: [u8; 2];

    'running: loop {

        sound_system.handle_timer(&(chip8.sound_timer));
        oparray = [
                chip8.memory[usize::from(chip8.program_counter)],
                chip8.memory[usize::from(chip8.program_counter) + 1]
        ];
        chip8.program_counter += 2;
        chip8.execute(&oparray);

        let mut new_key;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running      // Specifies which loop to break from
                },
                Event::KeyDown {keycode: Some(x), .. } => { 
                    new_key = keys::handle_key_event(&x);
                    chip8.keyboard |= new_key;
                },
                Event::KeyUp {keycode: Some(x), .. } => {
                    new_key = keys::handle_key_event(&x);
                    chip8.keyboard &= !new_key;
                },
                _ => {}
            }
        }
        // Update the display if needed
        if chip8.draw_flag == 1 {
            canvas.clear();                                                             // Clear the buffer
            texture.update(None, &(chip8.display), 64).unwrap();                        // Update texture
            canvas.copy(&texture, None, Some(Rect::new(0, 0, 64*8, 32*8))).unwrap();    // Update canvas
            canvas.present();                                                           // Display canvas
            chip8.draw_flag = 0;
        }
        
        std::thread::sleep(Duration::new(0, 500));
    }
}