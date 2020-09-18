//
// apu/mod.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 17 2020
//

// Modules
mod apu;
pub mod bus;
mod chnl;
mod seq;

// Public re-exports
pub use apu::{Apu, Sample, APU_OUTPUT_RATE};

#[cfg(feature="events")]
pub use apu::events;
