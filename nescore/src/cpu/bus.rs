//
// cpu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

use crate::io::IoAccess;

pub struct CpuIoBus<'a> {
    ppu: &'a mut dyn IoAccess
}

impl<'a> CpuIoBus<'a> {
    pub fn new(ppu_io: &'a mut dyn IoAccess) -> Self {
        CpuIoBus {
            ppu: ppu_io
        }
    }
}

impl<'a> IoAccess for CpuIoBus<'a> {
    fn read_byte(&self, addr: u16) -> u8 {
        // TODO: IO Mapping for CPU
        0
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        // TODO: IO Mapping for CPU
    }
}
