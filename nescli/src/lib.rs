//
// lib.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 13 2020
//
mod common;

pub mod run;
pub mod info;
pub mod img;
pub mod apu;
pub mod audio;
pub mod perf;

use clap::Clap;

#[derive(Clap)]
pub enum Command {
    /// Run the specified ROM file
    #[clap(name = "run", version = "1.0", author = "Natesh Narain")]
    Run(run::Options),
    /// Dump the header information of the specified ROM file
    #[clap(name = "info", version = "1.0", author = "Natesh Narain")]
    Info(info::Options),
    /// Dump the CHR ROM data to an image file
    #[clap(name = "img", version = "1.0", author = "Natesh Narain")]
    Img(img::Options),
    /// APU plots
    #[clap(name = "apu", version = "1.0", author = "Natesh Narain")]
    Apu(apu::Options),
    /// Just play audio from the ROM file
    #[clap(name = "audio", version = "1.0", author = "Natesh Narain")]
    Audio(audio::Options),
    /// Do nothing but run the emulator
    #[clap(name = "perf", version = "1.0", author = "Natesh Narain")]
    Perf(perf::Options),
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Natesh Narain")]
pub struct Options {
    #[clap(subcommand)]
    pub cmd: Command
}
