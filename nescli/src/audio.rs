//
// audio.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 12 2020
//

use clap::Clap;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;


use nescore::{Nes, CartridgeLoader};

use crate::common::audio::AudioStreamSource;

#[derive(Clap)]
pub struct Options {
    /// ROM file
    rom: String,
}

pub fn dispatch(opts: Options) {
    // Open an SDL window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let _window = video_subsystem.window("play audio", 500, 500)
                                .position_centered()
                                .opengl()
                                .build()
                                .unwrap();

    // Audio
    let audio_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: None,
    };

    let mut audio_device = audio_subsystem.open_playback(None, &audio_spec, |_|{
        AudioStreamSource::default()
    }).unwrap();
    audio_device.resume();

    // Event handling
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Load the ROM into a Nes instance
    let mut nes = CartridgeLoader::default().rom_path(&opts.rom).load().map(Nes::from).unwrap();

    'running: loop {
        // Process events...
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                _ => {}
            }
        }

        // Run the emulator...
        let buffer = nes.run_audio(4096);

        {
            let mut audio_device = audio_device.lock();
            audio_device.update(buffer);
        }
    }

}
