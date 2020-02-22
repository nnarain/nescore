//
// ppu/hw.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 21 2020
//

use crate::common::Clockable;

// http://wiki.nesdev.com/w/index.php/PPU_rendering

pub trait Shifter {
    fn get_value(&self, sel: u8) -> u8;
}

/// Represent the two 16 bit registers used to process tile data
pub struct TileRegister {
    plane0: u16,
    plane1: u16,
}

impl Default for TileRegister {
    fn default() -> Self {
        TileRegister {
            plane0: 0x0000,
            plane1: 0x0000,
        }
    }
}

impl Clockable for TileRegister {
    fn tick(&mut self) {
        // Clock shift registers
        self.plane0 >>= 1;
        self.plane1 >>= 1;
    }
}

impl Shifter for TileRegister {
    fn get_value(&self, sel: u8) -> u8 {
        // Get the value of the two registers combined
        let lo = bit_as_value!(self.plane0, sel) as u8;
        let hi = bit_as_value!(self.plane1, sel) as u8;

        (hi << 1) | lo
    }
}

impl TileRegister {
    /// Load tile data into the upper bytes of the two shift registers
    pub fn load(&mut self, value: (u8, u8)) {
        self.plane0 = self.plane0 | ((value.0 as u16) << 8);
        self.plane1 = self.plane1 | ((value.1 as u16) << 8);
    }
}

/// Representation of the two 8 bit shift registers used to hold pallette data for the ppu
pub struct PaletteRegister {
    r0: u8,
    r1: u8,

    latch: u8,
}

impl Default for PaletteRegister {
    fn default() -> Self {
        PaletteRegister {
            r0: 0x00,
            r1: 0x00,

            latch: 0x00,
        }
    }
}

impl Clockable for PaletteRegister {
    fn tick(&mut self) {
        // Append the latch bits to the end of the shift register
        self.r0 = (self.r0 >> 1) | (bit_as_value!(self.latch, 0) << 7);
        self.r1 = (self.r1 >> 1) | (bit_as_value!(self.latch, 1) << 7);
    }
}

impl Shifter for PaletteRegister {
    fn get_value(&self, sel: u8) -> u8 {
        let lo = bit_as_value!(self.r0, sel);
        let hi = bit_as_value!(self.r1, sel);

        (hi << 1) | lo
    }
}

impl PaletteRegister {
    pub fn latch(&mut self, value: u8) {
        self.latch = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_register_load_and_shift() {
        let mut tile_reg = TileRegister::default();

        // Load lower byte with FF and upper with 00
        tile_reg.load((0x03, 0x02));

        // Shift into the lower register
        for _ in 0..8 {
            tile_reg.tick();
        }

        assert_eq!(tile_reg.get_value(0), 0x01);
        assert_eq!(tile_reg.get_value(1), 0x03);
    }

    #[test]
    fn pallette_register_load_and_shift() {
        let mut pallette_reg = PaletteRegister::default();

        // Latch a $1 for both registers
        pallette_reg.latch(0x03);

        for _ in 0..8 {
            pallette_reg.tick();
        }

        assert_eq!(pallette_reg.get_value(0), 0x03);
        assert_eq!(pallette_reg.get_value(7), 0x03);
    }
}
