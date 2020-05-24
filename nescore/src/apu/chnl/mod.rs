//
// apu/chnl/mod.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date April 1 2020
//

#[macro_use] mod lenctr;
#[macro_use] mod envelope;
mod div;
mod timer;

mod pulse;
mod triangle;
mod noise;

pub use pulse::{Pulse, NegateAddMode};
pub use triangle::Triangle;
pub use noise::Noise;

pub use lenctr::{LengthCounter, LengthCounterUnit};
pub use envelope::{Envelope, EnvelopeUnit};
pub use div::Divider;
pub use timer::Timer;

/// Common Sound Channel Functionality
pub trait SoundChannel {
    fn output(&self) -> u8;
}
