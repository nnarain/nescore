//
// ppu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 11 2020
//


use crate::common::{IoAccess, IoAccessRef};
use crate::mapper::Mapper;

const INTERNAL_RAM: usize = 0x1000;

pub struct PpuIoBus {
    cpu: IoAccessRef,
    mapper: Mapper,

    nametable_ram: [u8; INTERNAL_RAM],
    palette_ram: [u8; 0xFF],
}

impl PpuIoBus {
    pub fn new(cpu_io: IoAccessRef, mapper: Mapper) -> Self {
        PpuIoBus {
            cpu: cpu_io,
            mapper: mapper,

            nametable_ram: [0x00; INTERNAL_RAM],
            palette_ram: [0x00; 0xFF],
        }
    }
}

impl IoAccess for PpuIoBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.mapper.borrow().read_chr(addr),
            0x2000..=0x2FFF => self.nametable_ram[(addr - 0x2000) as usize],
            0x3000..=0x3EFF => self.nametable_ram[(addr - 0x1000 - 0x2000) as usize],
            0x3F00..=0x3FFF => self.palette_ram[(addr - 0x3F00) as usize],

            _ => panic!("Invalid read {:04X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.mapper.borrow_mut().write_chr(addr, value),
            0x2000..=0x2FFF => self.nametable_ram[(addr - 0x2000) as usize] = value,
            0x3000..=0x3EFF => self.nametable_ram[(addr - 0x1000 - 0x2000) as usize] = value,
            0x3F00..=0x3FFF => self.palette_ram[(addr - 0x3F00) as usize] = value,

            _ => panic!("Invalid write {:04X}={:02X}", addr, value),
        }
    }

    fn raise_interrupt(&mut self) {
        self.cpu.borrow_mut().raise_interrupt();
    }
}
