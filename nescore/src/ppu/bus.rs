//
// ppu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 11 2020
//


use crate::common::{IoAccess, IoAccessRef};
use crate::mapper::Mapper;

pub struct PpuIoBus {
    cpu: IoAccessRef,
    mapper: Mapper,
}

impl PpuIoBus {
    pub fn new(cpu_io: IoAccessRef, mapper: Mapper) -> Self {
        PpuIoBus {
            cpu: cpu_io,
            mapper: mapper,
        }
    }
}

impl IoAccess for PpuIoBus {
    fn read_byte(&self, addr: u16) -> u8 {
        self.mapper.borrow().read_chr(addr)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().write_chr(addr, value);
    }

    fn raise_interrupt(&mut self) {
        self.cpu.borrow_mut().raise_interrupt();
    }
}
