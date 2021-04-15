//
// mapper.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 11 2019
//


use super::base::MapperBase;
use super::nrom::Nrom;
use super::mmc1::Mmc1;
use super::unrom::Unrom;
use super::cnrom::Cnrom;
use super::axrom::Axrom;

// use std::boxed::Box;
use std::rc::Rc;
use std::cell::RefCell;

use crate::cart::Cartridge;

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    OneScreenLower,
    OneScreenUpper,
    Vertical,
    Horizontal,
}

pub trait MapperControl {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn read_chr(&self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, value: u8);

    fn mirroring(&self) -> Option<Mirroring> { None }

    fn get_battery_ram(&self) -> Vec<u8> {
        (0x6000..0x8000).map(|addr| self.read(addr)).collect()
    }
}

pub type Mapper = Rc<RefCell<dyn MapperControl>>;

/// Create mapper instance from cartridge
pub fn from_cartridge(cart: Cartridge) -> Mapper {
    match cart.info.mapper {
        0 => create_mapper::<Nrom>(cart),
        1 => create_mapper::<Mmc1>(cart),
        2 => create_mapper::<Unrom>(cart),
        3 => create_mapper::<Cnrom>(cart),
        7 => create_mapper::<Axrom>(cart),
        _ => panic!("Invalid or unimplemented mapper: #{mapper}", mapper=cart.info.mapper),
    }
}

/// Instantiate a mapper from a Cartridge
fn create_mapper<T: 'static + MapperControl + From<Cartridge>>(cart: Cartridge) -> Mapper {
    let mapper = MapperBase::<T>::from(cart);
    Rc::new(RefCell::new(mapper))
}
