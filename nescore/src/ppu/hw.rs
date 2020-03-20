//
// ppu/hw.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 21 2020
//

use crate::common::Clockable;

// http://wiki.nesdev.com/w/index.php/PPU_rendering

pub trait Shifter<T=u8> {
    fn get_value(&self, sel: u8) -> T;
}

/// Represent the two 16 bit registers used to process tile data
#[derive(Default)]
pub struct TileRegister {
    plane0: u16,
    plane1: u16,
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
        self.plane0 = self.plane0 | ((reverse_bits!(value.0) as u16) << 8);
        self.plane1 = self.plane1 | ((reverse_bits!(value.1) << 8));
    }
}

/// Representation of the two 8 bit shift registers used to hold pallette data for the ppu
#[derive(Default)]
pub struct PaletteRegister {
    r0: u8,
    r1: u8,

    latch: u8,
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

/// Sprite shift registers
#[derive(Default, Clone, Copy)]
pub struct SpriteRegister {
    x_counter: u8,
    is_active: bool,

    palette: u8,
    priority: bool,

    plane0: u8,
    plane1: u8,

    sprite_num: u8,
}

impl Clockable for SpriteRegister {
    fn tick(&mut self) {
        if self.x_counter > 0 {
            // Clock down the x position counter
            self.x_counter -= 1;
            self.is_active = self.x_counter == 0;
        }
        else {
            if self.is_active {
                // Shift pattern data
                self.plane0 >>= 1;
                self.plane1 >>= 1;
            }
        }

        // self.is_active = self.x_counter == 0;
    }
}

impl SpriteRegister {
    pub fn load(&mut self, x_pos: u8, pattern: (u8, u8), palette: u8, front_priority: bool, sprite_num: u8) {
        self.x_counter = x_pos;
        self.plane0 = reverse_bits!(pattern.0);
        self.plane1 = reverse_bits!(pattern.1);

        self.is_active = self.x_counter == 0;

        self.palette = palette;
        self.priority = front_priority;

        self.sprite_num = sprite_num;
    }

    pub fn get_value(&self) -> (u8, u8, bool, u8) {
        // Get the value of the two registers combined
        let lo = bit_as_value!(self.plane0, 0) as u8;
        let hi = bit_as_value!(self.plane1, 0) as u8;

        ((hi << 1) | lo, self.palette, self.priority, self.sprite_num)
    }

    pub fn active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_register_load_and_shift() {
        let mut sprite_reg = SpriteRegister::default();

        // Load x position 10 and the pattern data
        sprite_reg.load(10, (0x80, 0x00), 0, false, 0);

        for _ in 0..9 {
            sprite_reg.tick();
        }

        assert!(!sprite_reg.active());

        sprite_reg.tick();
        assert!(sprite_reg.active());
        assert_eq!(sprite_reg.get_value(), (1, 0, false, 0));

        sprite_reg.tick();
        assert_eq!(sprite_reg.get_value(), (0, 0, false, 0));
    }

    #[test]
    fn sprite_register_load_zero() {
        let mut sprite_reg = SpriteRegister::default();
        // Load X position 1
        sprite_reg.load(0, (0x80, 0x00), 0, false, 0);

        assert!(sprite_reg.active());
    }

    #[test]
    fn tile_register_load_and_shift() {
        let mut tile_reg = TileRegister::default();

        // Load lower byte with FF and upper with 00
        tile_reg.load((0x03, 0x02));

        // Shift into the lower register
        for _ in 0..8 {
            tile_reg.tick();
        }

        assert_eq!(tile_reg.get_value(7), 0x01);
        assert_eq!(tile_reg.get_value(6), 0x03);
    }

    #[test]
    fn pallette_register_load_and_shift() {
        let mut palette_reg = PaletteRegister::default();

        // Latch a $1 for both registers
        palette_reg.latch(0x03);

        for _ in 0..8 {
            palette_reg.tick();
        }

        assert_eq!(palette_reg.get_value(0), 0x03);
        assert_eq!(palette_reg.get_value(7), 0x03);
    }
}
