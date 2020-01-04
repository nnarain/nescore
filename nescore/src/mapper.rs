//
// mapper.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 11 2019
//
#[macro_use]
mod mem;

mod nrom;
mod mmc1;
mod unrom;

use nrom::Nrom;
use mmc1::Mmc1;
use unrom::Unrom;

use std::boxed::Box;

use crate::cart::Cartridge;

pub trait MapperControl {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub type Mapper = Box<dyn MapperControl>;

/// Create mapper instance from cartridge
pub fn from_cartridge(cart: Cartridge) -> Mapper {
    match cart.info.mapper {
        0 => Box::new(Nrom::from(cart)),
        1 => Box::new(Mmc1::from(cart)),
        2 => Box::new(Unrom::from(cart)),
        _ => panic!("Invalid or unimplemented mapper"),
    }
}
