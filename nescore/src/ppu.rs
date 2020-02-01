//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//

use crate::io::IoAccess;
use crate::clk::Clockable;

#[derive(Clone, Copy)]
enum IncMode {
    Add1 = 1,
    Add32 = 32,
}

impl IncMode {
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

/// NES Picture Processing Unit
pub struct Ppu {
    vram: [u8; 0x4000],

    vram_addr: u16,
    inc_mode: IncMode,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            vram: [0; 0x4000],

            vram_addr: 0,
            inc_mode: IncMode::Add1,
        }
    }
    
    pub fn read_direct(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }
}

impl IoAccess for Ppu {
    fn read_byte(&self, _addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x2000 => {
                self.inc_mode = if bit_is_set!(value, 2) { IncMode::Add32 } else { IncMode::Add1 };
            },
            0x2006 => {
                self.vram_addr = (self.vram_addr << 8) | (value as u16);
            },
            0x2007 => {
                self.vram[self.vram_addr as usize] = value;
                self.vram_addr += self.inc_mode.to_u16();
            }
            _ => {}
        }
    }
}

impl Clockable for Ppu {
    fn tick(&mut self, _io: &mut dyn IoAccess) {

    }
}

#[cfg(test)]
mod test {

}
