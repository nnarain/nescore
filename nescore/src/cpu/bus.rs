//
// cpu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

use crate::io::IoAccess;
use crate::mapper::{Mapper};

pub struct CpuIoBus<'a> {
    ppu: &'a mut dyn IoAccess,
    mapper: &'a mut Mapper,
}

impl<'a> CpuIoBus<'a> {
    pub fn new(ppu_io: &'a mut dyn IoAccess, mapper: &'a mut Mapper) -> Self {
        CpuIoBus {
            ppu: ppu_io,
            mapper: mapper,
        }
    }
}

impl<'a> IoAccess for CpuIoBus<'a> {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3FFF => self.ppu.read_byte(addr % 8),
            0x4000..=0x401F => {
                // APU and IO
                0
            },
            0x4020..=0xFFFF => self.mapper.read(addr),
            _ => {
                panic!("Invalid address range")
            }
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        // TODO: IO Mapping for CPU
    }
}
