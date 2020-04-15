//
// run.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 13 2020
//

use clap::Clap;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use nescore::{Nes, CartridgeLoader, Button};
use nescore::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

use std::io::prelude::*;
use std::fs::File;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

#[derive(Clap, Debug)]
pub struct Options {
    /// Debug mode
    #[clap(short = "d")]
    pub debug: bool,
    /// The ROM file to run
    pub rom: String,
}

fn map_nes_key(keycode: Keycode) -> Option<Button> {
    match keycode {
        Keycode::W => Some(Button::Up),
        Keycode::A => Some(Button::Left),
        Keycode::D => Some(Button::Right),
        Keycode::S => Some(Button::Down),

        Keycode::J => Some(Button::A),
        Keycode::K => Some(Button::B),

        Keycode::Return => Some(Button::Start),
        Keycode::RShift => Some(Button::Select),

        _ => None,
    }
}

pub fn dispatch(opts: Options) {
    let save_file_path = format!("{}.sav", &opts.rom);

    let mut nes = CartridgeLoader::default()
                        .rom_path(&opts.rom)
                        .save_path(&save_file_path)
                        .load()
                        .map(|cart| Nes::default().with_cart(cart).debug_mode(opts.debug))
                        .unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("nescore", WINDOW_WIDTH, WINDOW_HEIGHT)
                                .position_centered()
                                .opengl()
                                .build()
                                .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut display = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24,
                                                               DISPLAY_WIDTH as u32,
                                                               DISPLAY_HEIGHT as u32).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {keycode, ..} => {
                    let btn = keycode.map(map_nes_key).flatten();
                    if let Some(btn) = btn {
                        nes.input(btn, true);
                    }
                }
                Event::KeyUp {keycode, ..} => {
                    let btn = keycode.map(map_nes_key).flatten();
                    if let Some(btn) = btn {
                        nes.input(btn, false);
                    }
                }
                _ => {}
            }
        }

        // Run the nescore for a single frame
        let framebuffer = nes.emulate_frame();

        // Update screen
        canvas.clear();

        // Update the on screen texture
        display.update(None, &framebuffer, DISPLAY_WIDTH * 3).unwrap();
        // Update the canvas
        canvas.copy(&display, None, Some(Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT))).unwrap();

        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    // Write the save file
    match File::create(&save_file_path) {
        Ok(ref mut file) => {
            let save_buffer = nes.eject();
            file.write_all(&save_buffer[..]).unwrap();
        },
        Err(e) => println!("{}", e),
    }
}
