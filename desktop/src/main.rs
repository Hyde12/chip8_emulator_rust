use std::{
    env,
    fs::{self},
    io::Error,
    time::Duration,
};

use chip8_core::{self, Emulator, SCREEN_LENGTH, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};
use rodio::{Source, source::SineWave};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Woah there buddy, add a rom file path.");
    }

    let mut emulator = Emulator::new();
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

    window.set_target_fps(60);

    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    let player = rodio::Player::connect_new(&handle.mixer());

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_LENGTH];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .get_keys()
            .iter()
            .for_each(|key| match key_to_index(*key) {
                0..=15 => {
                    emulator.set_key_state(key_to_index(*key), true);
                }
                _ => {}
            });

        window
            .get_keys_released()
            .iter()
            .for_each(|key| match key_to_index(*key) {
                0..=15 => {
                    emulator.set_key_state(key_to_index(*key), false);
                }
                _ => {}
            });

        if emulator.is_playing_sound() {
            let source = SineWave::new(500.0)
                .take_duration(Duration::from_secs_f32(0.01))
                .amplify(0.20);
            player.append(source);
        }

        for _ in 0..10 {
            emulator.fde_loop();
        }

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

fn key_to_index(key: Key) -> usize {
    match key {
        Key::Key1 => 0x1,
        Key::Key2 => 0x2,
        Key::Key3 => 0x3,
        Key::Key4 => 0xC,
        Key::Q => 0x4,
        Key::W => 0x5,
        Key::E => 0x6,
        Key::R => 0xD,
        Key::A => 0x7,
        Key::S => 0x8,
        Key::D => 0x9,
        Key::F => 0xE,
        Key::Z => 0xA,
        Key::X => 0x0,
        Key::C => 0xB,
        Key::V => 0xF,
        _ => 200,
    }
}
