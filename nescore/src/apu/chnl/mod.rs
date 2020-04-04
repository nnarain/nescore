mod pulse;
mod triangle;
mod lenctr;

pub use pulse::Pulse;
pub use triangle::Triangle;
pub use lenctr::LengthCounter;

// TODO: Should SoundChannel be a generic type?
// i.e. SoundChannel<Pulse>?

/// Common Sound Channel Functionality
pub trait SoundChannel {
    fn clock_length(&mut self) {}
    fn clock_envelope(&mut self) {}
    fn is_enabled(&self) -> bool { false }
    #[allow(unused)]
    fn enable_length(&mut self, e: bool) {}
    fn length_status(&self) -> bool { false }
}
