use sdl2::keyboard::Keycode;

pub fn handle_key_event(key: &sdl2::keyboard::Keycode) -> u16 {
    let shift;
    match key {
        Keycode::Num1 => {
            shift = 0;
        },
        Keycode::Num2 => {
            shift = 1;
        },
        Keycode::Num3 => {
            shift = 2;
        },
        Keycode::Num4 => {
            shift = 3;
        },
        Keycode::Q => {
            shift = 4;
        },
        Keycode::W => {
            shift = 5;
        },
        Keycode::E => {
            shift = 6;
        },
        Keycode::R => {
            shift = 7;
        },
        Keycode::A => {
            shift = 8;
        },
        Keycode::S => {
            shift = 9;
        },
        Keycode::D => {
            shift = 10;
        },
        Keycode::F => {
            shift = 11;
        },
        Keycode::Z => {
            shift = 12;
        },
        Keycode::X => {
            shift = 13;
        },
        Keycode::C => {
            shift = 14;
        },
        Keycode::V => {
            shift = 15;
        },
        _ => { return 0; }
    }
    return (1 << shift) as u16;
}