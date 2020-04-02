//
// apu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
mod chnl;
mod seq;

use crate::common::{IoAccess, Clockable};

use seq::FrameSequencer;

#[derive(Default)]
pub struct Apu {
    pulse1: chnl::Pulse,
}

impl IoAccess for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x4000..=0x4003 => self.pulse1.read_byte(addr - 0x4000),
            _ => panic!("Invalid address for APU: ${:04X}", addr),
        }
     }
    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000..=0x4003 => self.pulse1.write_byte(addr - 0x4000, data),
            _ => panic!("Invalid address for APU: ${:04X}", addr),
        }
    }
}

impl Clockable for Apu {
    fn tick(&mut self) {

    }
}

#[cfg(test)]
mod tests {

}
