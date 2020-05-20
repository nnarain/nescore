#[macro_use] mod lenctr;
#[macro_use] mod envelope;

mod pulse;
mod triangle;
mod noise;

pub use pulse::Pulse;
pub use triangle::Triangle;
pub use noise::Noise;
pub use lenctr::{LengthCounter, LengthCounterUnit};
pub use envelope::{Envelope, EnvelopeUnit};

/// Common Sound Channel Functionality
pub trait SoundChannel {
    fn is_enabled(&self) -> bool { false }
}
