extern crate sdl2;
mod cpu;
mod keys;
mod opcode;
mod sound;

use std::env;
use sound::SoundSystem;
use std::time::{Duration, Instant};
use sdl2::rect::Rect;
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Not enough arguments! What game do you want to play!");
    }
    if args.len() > 2 {
        panic!("Too many arguments!");
    }
    let game_name = &args[1];
    let filename = format!("programs/{}", game_name);

    let mut chip8: cpu::Cpu = cpu::Cpu::new();
    chip8.load_program(&filename);
    chip8.load_sprites();

    let sdl_context = sdl2::init().unwrap();                        // SDL for graphics, sound and input
    let video_subsystem = sdl_context.video().unwrap();             // Init Display
    let mut sound_system = SoundSystem::initialize(&sdl_context);   // Init Sound System
    let mut event_pump = sdl_context.event_pump().unwrap();         // Init Event System

    let window = video_subsystem.window("Rust-Chip8-Interpreter", 64*8, 32*8)
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

        oparray = [
                chip8.memory[usize::from(chip8.program_counter)],
                chip8.memory[usize::from(chip8.program_counter) + 1]
        ];

        // Program counter is incremented here because if we are waiting for a keypress 
        // we dont want to incremement it prematurely or we will end eup skipping instructions
        // In addition it must be before execute as certain instruction modify where the PC is and
        // they modify it to where they want it rather than the instruction before what they want
        if !chip8.waiting_for_key_flag {        // If we arent waiting for a keyboard event, 
            chip8.program_counter += 2;
            chip8.execute(&oparray);            // we can just execute the next opcode
        }

        // Handle input events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running      // Specifies which loop to break from
                },
                Event::KeyDown {keycode: Some(x), .. } => { 
                    let (new_key, shift) = keys::handle_key_event(&x);
                    chip8.keyboard |= new_key;

                    // If we were waiting for a key event due to Fx0A instruction
                    // We need to finish off the instruction by placing the input key to register[x]
                    if chip8.waiting_for_key_flag {                      
                        chip8.key_pressed(shift);
                    } 
                },
                Event::KeyUp {keycode: Some(x), .. } => {
                    let (new_key, _shift) = keys::handle_key_event(&x);
                    chip8.keyboard &= !new_key;
                },
                _ => {}
            }
        }

        // Handle sound
        sound_system.handle_timer(&(chip8.sound_timer));

        // Update the display if needed
        if chip8.draw_flag == 1 {
            canvas.clear();                                                             // Clear the buffer
            texture.update(None, &(chip8.display), 64).unwrap();                        // Update texture
            canvas.copy(&texture, None, Some(Rect::new(0, 0, 64*8, 32*8))).unwrap();    // Update canvas
            canvas.present();                                                           // Display canvas
            chip8.draw_flag = 0;
        }
        
        std::thread::sleep(Duration::new(0, 90000)); // previously 500
    }
}