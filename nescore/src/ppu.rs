//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//

use crate::io::IoAccess;
use crate::clk::Clockable;

/// NES Picture Processing Unit
pub struct Ppu {

}

impl Ppu {
    pub fn new() -> Self {
        Ppu{}
    }
}

impl IoAccess for Ppu {
    fn read_byte(&self, _addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, _addr: u16, _value: u8) {
        
    }
}

impl Clockable for Ppu {
    fn tick(&mut self, _io: &mut dyn IoAccess) {

    }
}

#[cfg(test)]
mod test {

}
