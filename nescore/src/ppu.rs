//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//

use crate::io::IoAccess;

/// NES Picture Processing Unit
pub struct Ppu {

}

impl Ppu {
    pub fn new() -> Self {
        Ppu{}
    }
}

impl IoAccess for Ppu {
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        
    }
}

#[cfg(test)]
mod test {

}
