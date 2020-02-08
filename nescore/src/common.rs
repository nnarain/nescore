//
// common.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

/// Access a memory mapped component
pub trait IoAccess {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, data: u8);
}

/// A clockable component
pub trait Clockable {
    fn tick(&mut self, io: &mut dyn IoAccess);
}

// TODO: Too generic for a 'Register'
pub trait Register<T> {
    fn from(t: T) -> Self;
    fn into(self) -> T;
}
