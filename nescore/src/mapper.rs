//
// mapper.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 11 2019
//
use std::boxed::Box;
use crate::io::IoAccess;
use crate::cart::Cartridge;

pub type Mapper = Box<dyn IoAccess>;

struct DummyMapper;

impl IoAccess for DummyMapper {
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, addr: u16, data: u8) {

    }
}

pub fn from_cartridge(cart: Cartridge) -> Mapper {
    Box::new(DummyMapper{})
}
