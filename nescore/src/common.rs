//
// common.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

/// Access a memory mapped component
pub trait IoAccess {
    fn read_byte(&mut self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, data: u8);
}

/// A clockable component
pub trait Clockable {
    fn tick(&mut self, io: &mut dyn IoAccess);
}

// TODO: Too generic for a 'Register'
pub trait Register<T> {
    fn new(value: T) -> Self
    where Self: Default {
        let mut r = Self::default();
        r.load(value);
        r
    }
    fn load(&mut self, _value: T){}
    fn value(&self) -> T;
}
