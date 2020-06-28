//
// apu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jun 05 2020
//
use nescore::{Nes, Cartridge};
use nescore::events::ApuEvent;

use piston_window::{EventLoop, PistonWindow, WindowSettings};
use plotters::prelude::*;

use clap::Clap;

const FPS: u64 = 30;

#[derive(Clap)]
pub struct Options {
    /// ROM file
    rom: String,
    /// Enable pulse 1 plot
    #[clap(long = "pulse1")]
    enable_pulse1: bool,
    /// Enable pulse 2 plot
    #[clap(long = "pulse2")]
    enable_pulse2: bool,
    /// Enable triangle
    #[clap(long = "triangle")]
    enable_triangle: bool,
    /// Enable noise
    #[clap(long = "noise")]
    enable_noise: bool,
    /// Enable DMC
    #[clap(long = "dmc")]
    enable_dmc: bool,
    /// Enable mixer plot
    #[clap(long = "mixer")]
    enable_mixer: bool,
}

pub fn dispatch(opts: Options) {
    // Initialize the nes instance
    let cart = Cartridge::from_path(&opts.rom).unwrap();
    let mut nes = Nes::default().with_cart(cart);

    // create the render window
    let mut window: PistonWindow = WindowSettings::new("APU Plot", [800, 600])
        .samples(4)
        .build()
        .unwrap();

    window.set_max_fps(FPS);

    let apu_events = nes.apu_event_channel();

    while let Some(_) = draw_piston_window(&mut window, |b| {
        // Run the NES emulator for a frame
        let _ = nes.emulate_frame();

        // Collect APU events from the last frame
        let data: Vec<ApuEvent> = apu_events.try_iter().collect();

        // Setup chart
        let root = b.into_drawing_area();
        root.fill(&WHITE)?;

        let mut cc = ChartBuilder::on(&root)
            .margin(10)
            .caption("APU Plot", ("san-serif", 30).into_font())
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_ranged(0..data.len(), 0.0..15.0)?;

        cc.configure_mesh()
            .x_label_formatter(&|x| format!("{}", x))
            .y_label_formatter(&|y| format!("{}", y))
            .x_desc("Time")
            .y_desc("Amplitude")
            .axis_desc_style(("sans-serif", 15).into_font())
            .draw()?;

        // Plot the audio data for each channel

        if opts.enable_pulse1 {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.pulse1 as f64)),
                &Palette99::pick(0),
            ))?
            .label("Pulse 1");
        }

        if opts.enable_pulse2 {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.pulse2 as f64)),
                &Palette99::pick(1),
            ))?
            .label("Pulse 2");
        }

        if opts.enable_triangle {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.triangle as f64)),
                &Palette99::pick(2),
            ))?
            .label("Triangle");
        }

        if opts.enable_noise {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.noise as f64)),
                &Palette99::pick(3),
            ))?
            .label("Noise");
        }

        if opts.enable_dmc {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.dmc as f64)),
                &Palette99::pick(4),
            ))?
            .label("DMC");
        }

        if opts.enable_mixer {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(x, data)| (x, data.mixer as f64)),
                &Palette99::pick(5),
            ))?
            .label("Mixer");
        }

        cc.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }) {}
}
