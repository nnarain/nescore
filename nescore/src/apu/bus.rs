//
// apu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jun 21 2020
//

use crate::common::{IoAccess, IoAccessRef, Interrupt};
use crate::mapper::Mapper;

pub struct ApuIoBus {
    cpu: IoAccessRef,
    mapper: Mapper,
}

impl ApuIoBus {
    pub fn new(cpu: IoAccessRef, mapper: Mapper) -> Self {
        ApuIoBus {
            cpu,
            mapper,
        }
    }
}

impl IoAccess for ApuIoBus {
    fn read_byte(&self, addr: u16) -> u8 {
        self.mapper.borrow().read(addr)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.mapper.borrow_mut().write(addr, value);
    }

    fn raise_interrupt(&mut self, interrupt_type: Interrupt) {
        self.cpu.borrow_mut().raise_interrupt(interrupt_type);
    }
}
