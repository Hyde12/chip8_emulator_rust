use std::{
    env,
    fs::{self},
    io::Error,
};

use chip8_core::{self, SCREEN_LENGTH, SCREEN_WIDTH, emulator};
use minifb::{Key, Window, WindowOptions};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Woah there buddy, add a rom file path.");
    }

    let mut emulator = emulator::new();
    let rom_path = &args[1];
    let rom_data = fs::read(rom_path)?;
    emulator.load(rom_data);

    let mut window_options = WindowOptions::default();
    window_options.scale = minifb::Scale::X16;

    let mut window = Window::new(
        "CHIP-8 EMULATOR",
        SCREEN_WIDTH,
        SCREEN_LENGTH,
        window_options,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(240);

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_LENGTH];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        emulator.fde_loop();
        emulator.timer_tick();

        for (buffer_color, pixel) in buffer.iter_mut().zip(emulator.get_screen().iter()) {
            *buffer_color = if *pixel { 0xFFFFFF } else { 0x0 }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_LENGTH)
            .unwrap();
    }

    Ok(())
}
