# Chip 8

Chip 8 Emulator (Interpreter?) programmed in Rust and SDL2 using the rust bindings

SDL2 was for the Graphics, Sound, and User Input. Most of the SDL code was taken from the examples at `https://docs.rs/sdl2/0.35.1/sdl2/index.html`

## Problems

Sound is a little buggy, doesn't play when it should every once in while and the sound itself *sounds* inconsistent

## Execute

This was built on Windows and requires the SDL2.dll file to run

To run, place any programs you have in the `programs/` directory, and type `./chip8.exe <program-name>` into your terminal

## Play

For user input, the following keys were used as a keyboard:
```
-----------------
| 1 | 2 | 3 | 4 |
|---|---|---|---|
| Q | W | E | R |
|---|---|---|---|
| A | S | D | F |
|---|---|---|---|
| Z | X | C | V |
-----------------
```
