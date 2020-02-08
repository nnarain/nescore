//
// ppu/regs.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 07 2020
//

// TODO: Do this with From<Register> and From<u8>... I think...

/// PPU Control Register
#[derive(Default)]
pub struct PpuCtrl {
    pub base_nametable_address: u8,     // Base nametable address (0=$2000, 1=$2400, 2=$2800, 3=$2C00)
    pub inc_mode: bool,                 // VRAM address increment mode
    pub sprite_pattern_table: bool,     // Sprite pattern table address
    pub background_pattern_table: bool, // Background pattern table address
    pub sprite_size: bool,              // 0: 8x8, 1: 8x16
    pub master_slave_select: bool,      // Master slave, select
    pub nmi_enable: bool,               // Generate NMI on Vblank
}

impl PpuCtrl {
    pub fn from(value: u8) -> Self {
        PpuCtrl {
            base_nametable_address: value & 0x03,
            inc_mode: bit_is_set!(value, 2),
            sprite_pattern_table: bit_is_set!(value, 3),
            background_pattern_table: bit_is_set!(value, 4),
            sprite_size: bit_is_set!(value, 5),
            master_slave_select: bit_is_set!(value, 6),
            nmi_enable: bit_is_set!(value, 7),
        }
    }

    pub fn value(&self) -> u8 {
        self.base_nametable_address
        | (self.inc_mode as u8) << 2
        | (self.sprite_pattern_table as u8) << 3
        | (self.background_pattern_table as u8) << 4
        | (self.sprite_size as u8) << 5
        | (self.master_slave_select as u8) << 6
        | (self.nmi_enable as u8) << 7
    }

    pub fn nametable(&self) -> u16 {
        0x2000u16 + (0x400u16 * self.base_nametable_address as u16)
    }
}

/// PPU Status
#[derive(Default)]
pub struct PpuStatus {
    pub lsb: u8,               // Least significant bits of the previous write to a PPU register
    pub sprite_overflow: bool, //
    pub sprite0_hit: bool,     //
    pub vblank: bool,          // Set when PPU enters vertical blanking period
}

impl PpuStatus {
    pub fn value(&self) -> u8 {
        self.lsb
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

impl PpuMask {
    pub fn from(value: u8) -> Self {
        PpuMask {
            greyscale: bit_is_set!(value, 0),
            show_background_left: bit_is_set!(value, 1),
            show_sprites_left: bit_is_set!(value, 2),
            background_enabled: bit_is_set!(value, 3),
            sprites_enabled: bit_is_set!(value, 4),
            emphasize_red: bit_is_set!(value, 5),
            emphasize_green: bit_is_set!(value, 6),
            emphasize_blue: bit_is_set!(value, 7)
        }
    }

    pub fn value(&self) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppuctrl() {
        let ctrl = PpuCtrl::from(0xFF);

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
    fn nametable_address() {
        let ctrl = PpuCtrl::from(0x00);
        assert_eq!(ctrl.nametable(), 0x2000);
        let ctrl = PpuCtrl::from(0x01);
        assert_eq!(ctrl.nametable(), 0x2400);
        let ctrl = PpuCtrl::from(0x02);
        assert_eq!(ctrl.nametable(), 0x2800);
        let ctrl = PpuCtrl::from(0x03);
        assert_eq!(ctrl.nametable(), 0x2C00);
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
    fn ppumask() {
        let mask = PpuMask::from(0xFF);
        assert_eq!(mask.greyscale, true);
        assert_eq!(mask.show_background_left, true);
        assert_eq!(mask.show_sprites_left, true);
        assert_eq!(mask.background_enabled, true);
        assert_eq!(mask.sprites_enabled, true);
        assert_eq!(mask.emphasize_red, true);
        assert_eq!(mask.emphasize_blue, true);
        assert_eq!(mask.emphasize_green, true);
    }
}