///
/// nescore/lib.rs
///
/// @author Natesh Narain <nnaraindev@gmail.com>
///

// Modules
#[macro_use] mod bit;
#[macro_use] mod common;

mod nes;
mod cpu;
mod ppu;
mod apu;
mod mapper;
mod joy;

#[cfg(feature = "events")]
pub mod log;
pub mod cart;

// Public re-exports
pub use nes::Nes;
pub use cpu::{Instruction, AddressingMode, format};
pub use cart::{Cartridge, CartridgeLoader};
pub use joy::{Controller, Button};

/// NES system specifications and associated types
pub mod specs {
    pub use super::ppu::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

    pub use super::apu::{Sample, APU_OUTPUT_RATE};
    pub type SampleBuffer = Vec<super::apu::Sample>;
}

#[cfg(feature="events")]
pub mod events {
    pub use super::cpu::events::*;
    pub use super::apu::events::*;
}

// For benchmarking
// #[cfg(test)]
// pub use cpu::*;
// #[cfg(test)]
// pub use ppu::*;
// #[cfg(test)]
// pub use apu::*;
