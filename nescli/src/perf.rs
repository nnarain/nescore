//
// perf.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sept 16 2020
//
use clap::Clap;

use nescore::{Nes, Cartridge};

#[derive(Clap)]
pub struct Options {
    /// ROM file
    rom: String,
}

pub fn dispatch(opts: Options) {
    let mut nes: Nes = Cartridge::from_path(&opts.rom).unwrap().into();

    for _ in 0..100000 {
        let _ = nes.emulate_frame();
    }

    println!("done");
}
