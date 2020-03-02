//
// Simple GUI for nescore
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 01 2020
//
use clap::clap_app;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use nescore::{Nes, Cartridge};
use nescore::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

const WINDOW_WIDTH: u32 = 256;
const WINDOW_HEIGHT: u32 = 240;

fn main() -> Result<(), String> {
    let matches = clap_app!(nesui =>
        (version: "1.0")
        (author: "Natesh Narain <nnaraindev@gmail.com>")
        (about: "Run a NES ROM file")
        (@arg ROM: -f --file +takes_value +required "The ROM file to run")
        (@arg debug: -d "Enable Debug Mode")
    ).get_matches();

    let enable_debug = matches.occurrences_of("debug") > 0;

    // TODO: Error handling
    let mut nes = matches.value_of("ROM")
                     .ok_or("ROM file not specified")
                     .map(Cartridge::from_path).map(|r| r.unwrap())
                     .map(|cart| Nes::default().with_cart(cart).debug_mode(enable_debug))
                     .unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("nesui", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut display = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24,
                                                               DISPLAY_WIDTH as u32,
                                                               DISPLAY_HEIGHT as u32).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
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

    Ok(())
}
