//
// ppu/regs.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 07 2020
//

use crate::common::Register;
use std::ops::AddAssign;

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

impl PpuMask {
    pub fn rendering_enabled(&self) -> bool {
        self.background_enabled || self.sprites_enabled
    }
}

#[derive(Default)]
pub struct PpuScroll {
    pub x: u8,
    pub y: u8,
    flag: bool,
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

impl PpuAddr {
    /// VRAM Address is composed as the following
    ///
    /// yyy NN YYYYY XXXXX
    /// ||| || ||||| +++++-- coarse X scroll
    /// ||| || +++++-------- coarse Y scroll
    /// ||| ++-------------- nametable select
    /// +++----------------- fine Y scroll
    /// 
    /// https://wiki.nesdev.com/w/index.php/PPU_scrolling

    pub fn set_nametable(&mut self, n: u8) {
        mask_clear!(self.addr, 0x0C00);
        self.addr |= ((n as u16) & 0x03) << 10;
    }

    pub fn nametable(&self) -> u16 {
        0x2000 | (self.addr & 0xC00)
    }

    pub fn tile(&self) -> u16 {
        0x2000 | (self.addr & 0x0FFF)
    }

    pub fn set_coarse_x(&mut self, x: u8) {
        mask_clear!(self.addr, 0x1F);
        self.addr |= (x as u16) >> 3;
    }

    pub fn coarse_x(&self) -> u8 {
        (self.addr as u8) & 0x1F
    }

    pub fn set_y(&mut self, y: u8) {
        mask_clear!(self.addr, 0x73E0);

        let coarse_y = (y as u16) >> 3;
        let fine_y = (y as u16) & 0x07;

        self.addr |= coarse_y << 5;
        self.addr |= fine_y << 12;
    }

    pub fn fine_y(&self) -> u8 {
        (self.addr >> 12) as u8
    }

    pub fn coarse_y(&self) -> u8 {
        ((self.addr >> 5) & 0x1F) as u8
    }

    pub fn set_low_byte(&mut self, lo: u8) {
        mask_clear!(self.addr, 0x00FF);
        self.addr |= lo as u16;
    }

    pub fn set_high_byte(&mut self, hi: u8) {
        mask_clear!(self.addr, 0x3F00);
        self.addr |= ((hi as u16) & 0x3F) << 8;
    }

    pub fn reload_x(&mut self, t: u16) {
        // Horizontal nametable bit and coarse X
        mask_clear!(self.addr, 0x041F);
        self.addr |= t & 0x041F;
    }

    pub fn reload_y(&mut self, t: u16) {
        mask_clear!(self.addr, 0x7BE0);
        self.addr |= t & 0x7BE0;
    }

    pub fn increment_h(&mut self) {
        // 0000, 0001, 0002 ... 001E, 001F -> 0400, 0401 ... 041E, 041F -> 0000, 0001
        // Decompose the VRAM address
        let nametable = (self.addr >> 10) & 0x03;
        let nametable_h = nametable & 0x01;
        let nametable_v = nametable >> 1;
        let coarse_x = self.addr & 0x1F;
        let coarse_y = (self.addr >> 5) & 0x1F;
        let fine_y = (self.addr >> 12) & 0x07;

        // TODO: Probably a better way?

        // Increment coarse x
        let coarse_x = coarse_x + 1;
        // Get coarse x overflow bit
        let coarse_x_overflow = bit_as_value!(coarse_x, 5);
        // Add the overflow to the nametable value
        // This will allow transitioning to the adjacent nametable
        let nametable_h = nametable_h + coarse_x_overflow;

        // Rebuild the nametable portion
        let nametable = (nametable_v & 0x01) << 1 | (nametable_h & 0x01);

        // Re-compose the VRAM address
        self.addr = ((fine_y & 0x07) << 12) | ((nametable & 0x03) << 10) | ((coarse_y & 0x1F) << 5) | (coarse_x & 0x1F);
    }

    pub fn increment_v(&mut self) {
        // Decompose the VRAM address
        let nametable = (self.addr >> 10) & 0x03;
        let nametable_h = nametable & 0x01;
        let nametable_v = nametable >> 1;
        let coarse_x = self.addr & 0x1F;
        let coarse_y = (self.addr >> 5) & 0x1F;
        let fine_y = (self.addr >> 12) & 0x07;

        // Add the nametable overflow to fine y
        let fine_y = fine_y + 1;
        // Get the fine y overflow
        let fine_y_overflow = bit_as_value!(fine_y, 3);
        // Get coarse y overflow
        let coarse_y_overflow = (coarse_y + fine_y_overflow) / 29;
        // Add the fine y overflow to coarse y
        let coarse_y = (coarse_y + fine_y_overflow) % 30;
        // Add the coarse y overflow the the nametable vertical
        let nametable_v = nametable_v + coarse_y_overflow;

        let nametable = (nametable_v & 0x01) << 1 | (nametable_h & 0x01);

        self.addr = ((fine_y & 0x07) << 12) | ((nametable & 0x03) << 10) | ((coarse_y & 0x1F) << 5) | (coarse_x & 0x1F);
    }
}

impl Register<u16> for PpuAddr {
    fn load(&mut self, value: u16) {
        self.addr = value;
    }

    fn value(&self) -> u16 {
        self.addr
    }
}

impl AddAssign<u16> for PpuAddr {
    fn add_assign(&mut self, rhs: u16) {
        self.addr = self.addr.wrapping_add(rhs);
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

        assert_eq!(ctrl.background_pattern_table(), 0x1000);

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
    fn ppumask_show_background_left() {
        let mut mask = PpuMask::default();
        mask.load(0x02);

        assert_eq!(mask.show_background_left, true);
        assert_eq!(mask.value(), 0x02);
    }

    #[test]
    fn ppuaddr() {
        let mut addr = PpuAddr::default();

        addr.set_high_byte(0x1E);
        addr.set_low_byte(0xAD);
        assert_eq!(addr.value(), 0x1EAD);

        addr.set_high_byte(0x20);
        addr.set_low_byte(0x00);
        assert_eq!(addr.value(), 0x2000);
    }

    #[test]
    fn ppuaddr_set_nametable() {
        let mut v = PpuAddr::default();
        v.set_nametable(0x03);

        assert_eq!(v.value(), 0x0C00);
    }

    #[test]
    fn ppuaddr_set_coarse_x() {
        let mut v = PpuAddr::default();
        v.set_coarse_x(0x1F);

        assert_eq!(v.value(), 0x0003);
    }

    #[test]
    fn ppuaddr_set_y() {
        let mut v = PpuAddr::default();
        v.set_y(0xFF);

        assert_eq!(v.value(), 0x73E0);
    }

    #[test]
    fn ppuaddr_set_high() {
        let mut v = PpuAddr::default();
        v.set_high_byte(0xFF);

        assert_eq!(v.value(), 0x3F00);
    }

    #[test]
    fn ppuaddr_set_low() {
        let mut v = PpuAddr::default();
        v.set_low_byte(0xFF);

        assert_eq!(v.value(), 0x00FF);
    }

    #[test]
    fn ppuaddr_fine_y() {
        let mut v = PpuAddr::default();
        v.set_y(0x07);

        assert_eq!(v.fine_y(), 0x07);
    }

    #[test]
    fn ppuaddr_increment_h() {
        let mut addr = PpuAddr::default();

        // Simple Increment
        addr.load(0x0000);
        assert_eq!(addr.coarse_x(), 0);
        addr.increment_h();
        assert_eq!(addr.coarse_x(), 1);

        // Wrap nametable - $2000 -> $2400
        addr.load(0x01F);
        assert_eq!(addr.nametable(), 0x2000);
        assert_eq!(addr.coarse_x(), 0x1F);
        addr.increment_h();
        assert_eq!(addr.nametable(), 0x2400);
        assert_eq!(addr.coarse_x(), 0x00);

        // Wrap nametable - $2800 -> $2C00
        addr.load(0x81F);
        assert_eq!(addr.nametable(), 0x2800);
        assert_eq!(addr.coarse_x(), 0x1F);
        addr.increment_h();
        assert_eq!(addr.nametable(), 0x2C00);
        assert_eq!(addr.coarse_x(), 0x00);

        // Wrap nametable - $2400 -> $200
        addr.load(0x41F);
        assert_eq!(addr.nametable(), 0x2400);
        assert_eq!(addr.coarse_x(), 0x1F);
        addr.increment_h();
        assert_eq!(addr.nametable(), 0x2000);
        assert_eq!(addr.coarse_x(), 0x00);
    }

    #[test]
    fn ppuaddr_increment_v() {
        let mut addr = PpuAddr::default();

        // Increment fine y
        addr.load(0x0000);
        assert_eq!(addr.fine_y(), 0x00);
        addr.increment_v();
        assert_eq!(addr.fine_y(), 0x01);
        addr.increment_v();
        assert_eq!(addr.fine_y(), 0x02);

        // Increment fine y, overflow, increment coarse y
        addr.load(0x7000);
        assert_eq!(addr.nametable(), 0x2000);
        assert_eq!(addr.fine_y(), 0x07);
        assert_eq!(addr.coarse_y(), 0x00);
        addr.increment_v();
        assert_eq!(addr.nametable(), 0x2000);
        assert_eq!(addr.fine_y(), 0x00);
        assert_eq!(addr.coarse_y(), 0x01);

        // Increment to vertical nametable
        addr.load(0x73A0);
        assert_eq!(addr.nametable(), 0x2000);
        assert_eq!(addr.fine_y(), 0x07);
        assert_eq!(addr.coarse_y(), 0x1D);
        addr.increment_v();
        assert_eq!(addr.nametable(), 0x2800);
        assert_eq!(addr.fine_y(), 0x00);
        assert_eq!(addr.coarse_y(), 0x00);
    }

    #[test]
    fn ppuaddr_reload_x() {
        let mut v = PpuAddr::default();
        let mut t = PpuAddr::default();

        t.load(0x01F);
        v.load(0x01F);

        assert_eq!(v.nametable(), 0x2000);
        assert_eq!(v.coarse_x(), 0x1F);
        // Increment to next nametable
        v.increment_h();
        assert_eq!(v.nametable(), 0x2400);
        assert_eq!(v.coarse_x(), 0x00);

        v.reload_x(t.value());

        assert_eq!(v.nametable(), 0x2000);
        assert_eq!(v.coarse_x(), 0x1F);
    }

    #[test]
    fn ppuaddr_reload_y() {
        let mut v = PpuAddr::default();
        let mut t = PpuAddr::default();

        t.load(0x73A0);
        v.load(0x73A0);

        assert_eq!(v.nametable(), 0x2000);
        assert_eq!(v.fine_y(), 0x07);
        assert_eq!(v.coarse_y(), 0x1D);
        v.increment_v();
        assert_eq!(v.nametable(), 0x2800);
        assert_eq!(v.fine_y(), 0x00);
        assert_eq!(v.coarse_y(), 0x00);

        v.reload_y(t.value());

        assert_eq!(v.nametable(), 0x2000);
        assert_eq!(v.fine_y(), 0x07);
        assert_eq!(v.coarse_y(), 0x1D);
    }
}