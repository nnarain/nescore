//
// common.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//
use std::rc::Rc;
use std::cell::RefCell;

/// Access a memory mapped component
pub trait IoAccess {
    #[allow(unused)]
    fn read_byte(&self, addr: u16) -> u8 { 0 }
    #[allow(unused)]
    fn write_byte(&mut self, addr: u16, data: u8) {}
    fn raise_interrupt(&mut self){}
}

pub type IoAccessRef = Rc<RefCell<dyn IoAccess>>;

/// A clockable component. Optionally, returns a value for every tick
pub trait Clockable<T=()> {
    fn tick(&mut self) -> T;
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
