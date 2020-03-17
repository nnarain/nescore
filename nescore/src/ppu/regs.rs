//
// ppu/regs.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 07 2020
//

use crate::common::Register;
use std::ops::AddAssign;
use std::num::Wrapping;

/// PPU Control Register
#[derive(Default)]
pub struct PpuCtrl {
    pub base_nametable_address: u8,     // Base nametable address (0=$2000, 1=$2400, 2=$2800, 3=$2C00)
    pub inc_mode: bool,                 // VRAM address increment mode (0: Add 1, 1: Add 32)
    pub sprite_pattern_table: bool,     // Sprite pattern table address (0: $0000, 1: $1000)
    pub background_pattern_table: bool, // Background pattern table address (0: $0000, 1: $1000)
    pub sprite_size: bool,              // 0: 8x8, 1: 8x16
    pub master_slave_select: bool,      // Master slave, select
    pub nmi_enable: bool,               // Generate NMI on Vblank
}

impl PpuCtrl {
    pub fn vram_increment(&self) -> u16 {
        if self.inc_mode { 32 } else { 1 }
    }

    /// Base nametable address
    pub fn nametable(&self) -> u16 {
        0x2000u16 + (0x400u16 * self.base_nametable_address as u16)
    }

    /// Attribute table for the selected nametable
    pub fn attribute(&self) -> u16 {
        // Nametable + Size of nametable - Size of attribute table
        self.nametable() + 0x400 - 0x40
    }

    pub fn background_pattern_table(&self) -> u16 {
        self.background_pattern_table as u16 * 0x1000
    }

    pub fn sprite_pattern_table(&self) -> u16 {
        self.sprite_pattern_table as u16 * 0x1000
    }

    pub fn sprite_height(&self) -> u8 {
        (self.sprite_size as u8 * 8) + 8
    }
 }

impl Register<u8> for PpuCtrl {
    fn load(&mut self, value: u8) {
        self.base_nametable_address = value & 0x03;
        self.inc_mode = bit_is_set!(value, 2);
        self.sprite_pattern_table = bit_is_set!(value, 3);
        self.background_pattern_table = bit_is_set!(value, 4);
        self.sprite_size = bit_is_set!(value, 5);
        self.master_slave_select = bit_is_set!(value, 6);
        self.nmi_enable = bit_is_set!(value, 7);
    }

    fn value(&self) -> u8 {
        self.base_nametable_address
        | (self.inc_mode as u8) << 2
        | (self.sprite_pattern_table as u8) << 3
        | (self.background_pattern_table as u8) << 4
        | (self.sprite_size as u8) << 5
        | (self.master_slave_select as u8) << 6
        | (self.nmi_enable as u8) << 7
    }
}

/// PPU Status
#[derive(Default)]
pub struct PpuStatus {
    pub lsb: u8,               // Least significant bits of the previous write to a PPU register
    pub sprite_overflow: bool, //
    pub sprite0_hit: bool,     // Set when a non-zero pixel of sprite 0 overlaps a nonzero background pixel.
    pub vblank: bool,          // Set when PPU enters vertical blanking period
}

impl Register<u8> for PpuStatus {
    fn value(&self) -> u8 {
        (self.lsb & 0x1F)
        | (self.sprite_overflow as u8) << 5
        | (self.sprite0_hit as u8) << 6
        | (self.vblank as u8) << 7
    }
}

#[derive(Default)]
pub struct PpuMask {
    pub greyscale: bool,            // Grey scale render mode
    pub show_background_left: bool, // Show background in left most 8 pixels of the screen
    pub show_sprites_left: bool,    // Show sprites in left most 8 pixels of the screen
    pub background_enabled: bool,   // Show background
    pub sprites_enabled: bool,      // Show sprites
    pub emphasize_red: bool,        // Emphasize Red
    pub emphasize_green: bool,      // Emphasize Green
    pub emphasize_blue: bool,       // Emphasize Blue
}

impl Register<u8> for PpuMask {
    fn load(&mut self, value: u8) {
        self.greyscale = bit_is_set!(value, 0);
        self.show_background_left = bit_is_set!(value, 1);
        self.show_sprites_left = bit_is_set!(value, 2);
        self.background_enabled = bit_is_set!(value, 3);
        self.sprites_enabled = bit_is_set!(value, 4);
        self.emphasize_red = bit_is_set!(value, 5);
        self.emphasize_green = bit_is_set!(value, 6);
        self.emphasize_blue = bit_is_set!(value, 7);
    }

    fn value(&self) -> u8 {
        self.greyscale as u8
        | (self.show_background_left as u8) << 1
        | (self.show_sprites_left as u8) << 2
        | (self.background_enabled as u8) << 3
        | (self.sprites_enabled as u8) << 4
        | (self.emphasize_red as u8) << 5
        | (self.emphasize_green as u8) << 6
        | (self.emphasize_blue as u8) << 7
    }
}

#[derive(Default)]
pub struct PpuScroll {
    pub x: u8,
    pub y: u8,
    flag: bool,
}

impl PpuScroll {
    pub fn offset(&self) -> (u8, u8) {
        (self.x, self.y)
    }
}

impl Register<u8> for PpuScroll {
    fn load(&mut self, value: u8) {
        if !self.flag {
            self.x = value;
        }
        else {
            self.y = value;
        }

        self.flag = !self.flag;
    }
    fn value(&self) -> u8 {
        self.x
    }
}

/// PPU Address Register
#[derive(Default)]
pub struct PpuAddr {
    addr: u16,
}

impl Register<u16> for PpuAddr {
    fn load(&mut self, value: u16) {
        self.addr = (self.addr << 8) | value;
    }

    fn value(&self) -> u16 {
        self.addr
    }
}

impl AddAssign<u16> for PpuAddr {
    fn add_assign(&mut self, rhs: u16) {
        self.addr = (Wrapping(self.addr) + Wrapping(rhs)).0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppuctrl() {
        let ctrl = PpuCtrl::new(0xFF);

        assert_eq!(ctrl.base_nametable_address, 0x03);
        assert_eq!(ctrl.inc_mode, true);
        assert_eq!(ctrl.sprite_pattern_table, true);
        assert_eq!(ctrl.background_pattern_table, true);
        assert_eq!(ctrl.sprite_size, true);
        assert_eq!(ctrl.master_slave_select, true);
        assert_eq!(ctrl.nmi_enable, true);

        let value = ctrl.value();
        assert_eq!(value, 0xFF);
    }

    #[test]
    fn sprite_height() {
        let mut ctrl = PpuCtrl::default();

        ctrl.load(0x00);
        assert_eq!(ctrl.sprite_height(), 8);

        ctrl.load(0x20);
        assert_eq!(ctrl.sprite_height(), 16);
    }

    #[test]
    fn nametable_address() {
        let mut ctrl = PpuCtrl::default();
        ctrl.load(0x00);
        assert_eq!(ctrl.nametable(), 0x2000);
        ctrl.load(0x01);
        assert_eq!(ctrl.nametable(), 0x2400);
        ctrl.load(0x02);
        assert_eq!(ctrl.nametable(), 0x2800);
        ctrl.load(0x03);
        assert_eq!(ctrl.nametable(), 0x2C00);
    }

    #[test]
    fn nametable_attribute_address() {
        let mut ctrl = PpuCtrl::default();
        ctrl.load(0x00);
        assert_eq!(ctrl.attribute(), 0x23C0);
        ctrl.load(0x01);
        assert_eq!(ctrl.attribute(), 0x27C0);
        ctrl.load(0x02);
        assert_eq!(ctrl.attribute(), 0x2BC0);
        ctrl.load(0x03);
        assert_eq!(ctrl.attribute(), 0x2FC0);
    }

    #[test]
    fn ppustatus() {
        let mut status = PpuStatus::default();
        status.sprite0_hit = true;
        status.sprite_overflow = true;
        status.vblank = true;

        let value: u8 = status.value();

        assert_eq!(value, 0xE0);
    }

    #[test]
    fn sprite_zero_hit() {
        let mut status = PpuStatus::default();
        status.sprite0_hit = true;

        assert!(mask_is_set!(status.value(), 0x40));
    }

    #[test]
    fn ppustatus_lsb() {
        let mut status = PpuStatus::default();
        status.lsb = 0xFF;

        assert_eq!(status.value(), 0x1F);
    }

    #[test]
    fn ppumask() {
        let mut mask = PpuMask::default();
        mask.load(0xFF);

        assert_eq!(mask.greyscale, true);
        assert_eq!(mask.show_background_left, true);
        assert_eq!(mask.show_sprites_left, true);
        assert_eq!(mask.background_enabled, true);
        assert_eq!(mask.sprites_enabled, true);
        assert_eq!(mask.emphasize_red, true);
        assert_eq!(mask.emphasize_blue, true);
        assert_eq!(mask.emphasize_green, true);
    }

    #[test]
    fn ppuaddr() {
        let mut addr = PpuAddr::default();

        addr.load(0xDE);
        addr.load(0xAD);
        assert_eq!(addr.value(), 0xDEAD);

        addr.load(0x20);
        addr.load(0x00);
        assert_eq!(addr.value(), 0x2000);
    }

    #[test]
    fn ppuscroll() {
        let mut scroll = PpuScroll::default();
        scroll.load(0xDE);
        scroll.load(0xAD);

        assert_eq!(scroll.x, 0xDE);
        assert_eq!(scroll.y, 0xAD);

        assert_eq!(scroll.offset(), (0xDE, 0xAD));
    }
}