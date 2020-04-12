//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//
pub mod bus;
mod regs;
mod hw;
mod sprite;

use regs::*;
use hw::*;
use sprite::Sprite;
use crate::common::{IoAccess, Clockable, Register, Interrupt};

use std::cell::RefCell;

const NUM_SCANLINES: usize = 262;
const CYCLES_PER_SCANLINE: usize = 341;
const TILES_PER_ROW: usize = 32;

pub type Pixel = (u8, u8, u8);
pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;
pub const CYCLES_PER_FRAME: usize = NUM_SCANLINES * CYCLES_PER_SCANLINE;


// Palette from http://wiki.nesdev.com/w/index.php/PPU_palettes
const COLOR_INDEX_TO_RGB: [u32; 64] = [
    0x656565, 0x002D69, 0x131F7F, 0x3C137C, 0x600B62, 0x730A37, 0x710F07, 0x5A1A00,
    0x342800, 0x0B3400, 0x003C00, 0x003D10, 0x003840, 0x000000, 0x000000, 0x000000,

    0xAEAEAE, 0x0F63B3, 0x4051D0, 0x7841CC, 0xA736A9, 0xC03470, 0xBD3C30, 0x9F4A00,
    0x6D5C00, 0x366D00, 0x077704, 0x00793D, 0x00727D, 0x000000, 0x000000, 0x000000,

    0xFEFEFF, 0x5DB3FF, 0x8FA1FF, 0xC890FF, 0xF785FA, 0xFF83C0, 0xFF8B7F, 0xEF9A49,
    0xBDAC2C, 0x85BC2F, 0x55C753, 0x3CC98C, 0x3EC2CD, 0x4E4E4E, 0x000000, 0x000000,

    0xFEFEFF, 0xBCDFFF, 0xD1D8FF, 0xE8D1FF, 0xFBCDFD, 0xFFCCE5, 0xFFCFCA, 0xF8D5B4,
    0xF8D5B4, 0x85BC2F, 0xB9E8B8, 0xAEE8D0, 0xAFE5EA, 0xB6B6B6, 0x000000, 0x000000
];


#[derive(Debug, Copy, Clone, PartialEq)]
enum Scanline {
    PreRender,
    Visible,
    PostRender,
    VBlank,
}

impl Scanline {
    pub fn from(scanline: usize) -> Self {
        match scanline {
            261 => Scanline::PreRender,
            0..=239 => Scanline::Visible,
            240 => Scanline::PostRender,
            241..=260 => Scanline::VBlank,

            _ => panic!("Invalid scanline!"),
        }
    }
}

/// NES Picture Processing Unit
pub struct Ppu<Io: IoAccess> {
    oam: [u8; 256],            // Object Attribute Memory (Sprites)
    sprite_cache: [Option<Sprite>; 8],   // Up to 8 sprites per scanline

    ctrl: PpuCtrl,              // PPUCTRL   - Control Register
    status: RefCell<PpuStatus>, // PPUSTATUS - Status Register
    mask: PpuMask,              // PPUMASK   - Mask Register (Render controls)
    oam_addr: RefCell<u16>,     // OAMADDR   - OAM Address

    v: RefCell<PpuAddr>,     // VRAM Address
    t: RefCell<PpuAddr>,     // Temporary VRAM Address
    x: u8,                   // Fine X scroll
    w: RefCell<bool>,        // Write toggle

    // Render pipeline hardware
    tile_reg: TileRegister,    // PPU tile shift registers
    pal_reg: PaletteRegister,  // PPU palette shift registers
    sprite_regs: [SpriteRegister; 8],

    cycle: usize,              // Cycle count per scanline
    scanline: usize,           // Current scanline

    bus: Option<Io>,
}

impl<Io: IoAccess> Default for Ppu<Io> {
    fn default() -> Self {
        Ppu{
            oam: [0; 256],
            sprite_cache: [None; 8],

            ctrl: PpuCtrl::default(),
            status: RefCell::new(PpuStatus::default()),
            mask: PpuMask::default(),
            oam_addr: RefCell::new(0),

            v: RefCell::new(PpuAddr::default()),
            t: RefCell::new(PpuAddr::default()),
            x: 0,
            w: RefCell::new(false),

            tile_reg: TileRegister::default(),
            pal_reg: PaletteRegister::default(),
            sprite_regs: [SpriteRegister::default(); 8],

            cycle: 0,
            scanline: NUM_SCANLINES - 1, // Initialize to the Pre-render scanline

            bus: None,
        }
    }
}

impl<Io: IoAccess> Ppu<Io> {
    fn run_cycle(&mut self) -> Option<Pixel> {
        let scanline = Scanline::from(self.scanline);
        match scanline {
            Scanline::PreRender => {
                if self.cycle == 1 {
                    self.clear_sprite_data();
                    self.status.borrow_mut().sprite0_hit = false;
                    self.status.borrow_mut().vblank = false;
                }

                if self.cycle >= 280 && self.cycle <= 304 {
                    // At dots 280 to 304, the vertical bits of t are copied to v (if rendering)
                    if self.mask.rendering_enabled() {
                        self.v.borrow_mut().reload_y(self.t.borrow().value());
                    }
                }

                // Same as a normal scanline but no output to the screen
                if self.mask.rendering_enabled() {
                    self.process_scanline(self.cycle);
                }

                None
            },
            Scanline::Visible => {
                let pixel = if self.cycle <= 255 {
                    // Generate a pixel
                    let pixel = Some(self.apply_mux());

                    // Clock sprite registers
                    self.tick_sprite_registers();

                    pixel
                }
                else {
                    None
                };

                if self.mask.rendering_enabled() {
                    self.process_scanline(self.cycle);
                }

                pixel
            },
            Scanline::PostRender => {
                // PPU is idle
                None
            },
            Scanline::VBlank => {
                if self.cycle == 1 && self.scanline == 241 {
                    self.status.borrow_mut().vblank = true;

                    // Signal NMI interrupt
                    if self.ctrl.nmi_enable {
                        self.raise_interrupt();
                    }
                }

                None
            }
        }
    }

    // Processing a single scanline per cycle
    fn process_scanline(&mut self, dot: usize) {
        match dot {
            0 => {
                // Cycle 0 is idle
            },
            1..=256 => {
                // 4 memory accesses each taking 2 cycles
                // In addition to all that, the sprite evaluation happens independently
                if dot % 8 == 0 {
                    if dot <= 240 {
                        self.load_shift_registers();
                    }
                }

                if dot == 256 {
                    // At dot 256 the increment part of v is incremented (if rendering)
                    if self.mask.rendering_enabled() {
                        self.v.borrow_mut().increment_v();
                    }
                }
            },
            257..=320 => {
                // Cycles 257 - 320: Get tile data for sprites on next scanline
                // Sprite eval is complete by cycle 257
                if dot == 257 {
                    let scanline = ((self.scanline + 1) % NUM_SCANLINES) as u16;
                    self.evaluate_sprites(scanline);

                    // At dot 257, the horizontal bits of t are copied to v (if rendering)
                    if self.mask.rendering_enabled() {
                        self.v.borrow_mut().reload_x(self.t.borrow().value());
                    }
                }
            },
            321..=336 => {
                // Cycles 321-336: Fetch first two tiles of the next scanline
                // accesses: 2 nametable bytes, attribute, pattern table low, pattern table high
                let scanline = (self.scanline + 1) % NUM_SCANLINES;

                if dot % 8 == 0 {
                    self.load_shift_registers();
                }

                // Sprite data loading has completed by cycle 321
                if dot == 321 {
                    self.load_sprite_data(scanline as u16);
                }
            },
            337..=340 => {
                // Cycles 337 - 340
                // Two nametable bytes are fetch, unknown purpose
            },
            _ => panic!("Invalid cycle for scanline! {}", dot),
        }

        // Tick shift register
        match dot {
            0..=256 | 322..=335 => self.tick_shifters(),
            _ => {}
        }
    }

    fn load_shift_registers(&mut self) {
        let mut addr = self.v.borrow_mut();

        let tile_no = self.read_vram(addr.tile());

        // Read pattern from pattern table memory
        let pattern = self.read_pattern(self.ctrl.background_pattern_table(), tile_no, addr.fine_y());
        // Fetch tile attributes
        let attribute = self.read_attribute(addr.nametable(), addr.coarse_y() as usize, addr.coarse_x() as usize);

        // Load shift registers
        self.tile_reg.load(pattern);
        self.pal_reg.latch(attribute);

        // Increment VRAM to the next tile
        if self.mask.rendering_enabled() {
            addr.increment_h();
        }
    }

    fn evaluate_sprites(&mut self, scanline: u16) {
        // Sprite data is delay by one scanline
        let scanline = scanline as i16;
        // Scan primary OAM for sprites that are on the specified scanline
        let mut found_sprites = 0;

        // Clear sprite cache
        self.sprite_cache = [None; 8];

        if scanline < 240 {
            for n in 0..64 {
                if found_sprites < 8 {
                    let offset = n * 4;
                    let sprite = Sprite::from(&self.oam[offset..offset+4], n as u8);

                    let intersect = scanline - sprite.y as i16;
                    let h = self.ctrl.sprite_height() as i16;

                    if intersect >= 0 && intersect < h {
                        // Found a valid sprite
                        // Move to the sprite cache
                        self.sprite_cache[found_sprites] = Some(sprite);
                        found_sprites += 1;
                    }
                }
                else {
                    // TODO: Set sprite overflow flag
                    break;
                }
            }
        }
    }

    fn load_sprite_data(&mut self, scanline: u16) {
        for (i, sprite) in self.sprite_cache.iter().enumerate() {
            if let Some(ref sprite) = sprite {
                let sprite_height = self.ctrl.sprite_height();

                let base = if sprite_height == 16 {
                    0x0000
                }
                else {
                    self.ctrl.sprite_pattern_table()
                };

                // Determine fine y for vertical flipping
                let fine_y = if !sprite.flip_v() {
                    (scanline - sprite.y) as u8
                }
                else {
                    (sprite_height - 1) - (scanline - sprite.y) as u8
                };

                let pattern = self.read_pattern(base, sprite.tile, fine_y);

                // Reverse bit pattern if the sprite is horizontally flipped
                let pattern = if sprite.flip_h() {
                    (reverse_bits!(pattern.0), reverse_bits!(pattern.1))
                }
                else {
                    pattern
                };

                self.sprite_regs[i].load(sprite.x, pattern, sprite.palette(), sprite.priority(), sprite.num);
            }
        }
    }

    fn clear_sprite_data(&mut self) {
        // TODO: Clear OAM data?
        for sprite_reg in &mut self.sprite_regs {
            sprite_reg.load(0, (0, 0), 0, false, 0);
        }
    }

    fn read_nametable(&self, nametable: u16, idx: usize) -> u8 {
        let addr = helpers::calc_nametable_address(nametable, idx);
        self.read_vram(addr)
    }

    fn read_attribute(&self, nametable: u16, tile_row: usize, tile_col: usize) -> u8 {
        let table_addr = nametable + (0x400 - 0x40);
        let addr = helpers::calc_attribute_address(table_addr, tile_row, tile_col);
        let attr = self.read_vram(addr);

        // Attributes are encoded as 2 bit for each quadrant, represented as:
        // (bottomright << 6) | (bottomleft << 4) | (topright << 2) | (topleft << 0)
        // [6, 4, 2, 0] => [3 * 2, 2 * 2, 1 * 2, 0 * 2]

        // Determine a value [0, 3] for the tile in its quadrant

        // Left - Right
        let lr = ((tile_col % 4) >= 2) as u8;
        // Top Bottom
        let tb = ((tile_row % 4) >= 2) as u8;

        // multiply by two to get the number of bits to shift
        let c = ((tb << 1) | lr) * 2;

        // Return the 2 bits for the tiles attribute
        (attr >> c) & 0x03
    }

    fn read_pattern(&self, base: u16, tile_no: u8, fine_y: u8) -> (u8, u8) {
        // TODO: Sprite size 16?
        let tile_no = tile_no as u16;
        // 16 bytes per tile
        let tile_offset = (tile_no * 16) + fine_y as u16;

        let lo = self.read_vram(base + tile_offset);
        let hi = self.read_vram(base + tile_offset + 8);

        (lo, hi)
    }

    fn get_sprite_pixel_data(&self) -> (u8, u8, bool, bool) {
        let mut pixel_data = (0, 0, false, false);

        // Find the first opaque pixel for the active sprites
        for sprite_reg in self.sprite_regs.iter() {
            if sprite_reg.active() {
                let sprite_data = sprite_reg.get_value();
                // Check if not opaque
                if sprite_data.0 != 0 {
                    pixel_data = (sprite_data.0, sprite_data.1, sprite_data.2, sprite_data.3 == 0);
                    break;
                }
            }
        }

        pixel_data
    }

    fn apply_mux(&self) -> Pixel {
        let dot = self.cycle;

        // Fetch pattern and attributes from shifters
        let bg_pattern = self.tile_reg.get_value(self.x);
        let bg_palette = self.pal_reg.get_value(self.x);

        let (bg_pattern, bg_palette) = if self.mask.background_enabled {
            if !self.mask.show_background_left && dot < 8 {
                (0, 0)
            }
            else {
                (bg_pattern, bg_palette)
            }
        }
        else {
            (0, 0)
        };

        let (sp_pattern, sp_palette, sp_priority, is_sprite0) = if self.mask.sprites_enabled {
            if !self.mask.show_sprites_left && dot < 8 {
                (0, 0, false, false)
            }
            else {
                self.get_sprite_pixel_data()
            }
        }
        else {
            (0, 0, false, false)
        };

        // Determine sprite 0 hit status
        if is_sprite0 && sp_pattern > 0 && bg_pattern > 0 {
            self.status.borrow_mut().sprite0_hit = true;
        }

        // Choose which pattern and palette to use
        // Select the sprite data is the sprite pixel is opaque and has front priority OR the background is transparent
        let (pattern, palette, palette_group) = helpers::pixel_mux((bg_pattern, bg_palette), (sp_pattern, sp_palette), sp_priority);
        // Determine palette offset: http://wiki.nesdev.com/w/index.php/PPU_palettes
        let palette_offset = palette_group | (palette << 2) | pattern;

        // Fetch color from palette
        let color = self.read_vram(0x3F00 + palette_offset as u16) as usize;

        // TODO: Color emphasis
        // TODO: Grey Scale
        helpers::color_to_pixel(COLOR_INDEX_TO_RGB[color])
    }

    fn tick_shifters(&mut self) {
        self.tile_reg.tick();
        self.pal_reg.tick();
    }

    fn tick_sprite_registers(&mut self) {
        for sprite_reg in self.sprite_regs.iter_mut() {
            sprite_reg.tick();
        }
    }

    /// Raise NMI interrupt
    fn raise_interrupt(&mut self) {
        if let Some(ref mut bus) = self.bus {
            bus.raise_interrupt(Interrupt::Nmi);
        }
    }

    /// Read directly from PPU VRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        if let Some(ref bus) = self.bus {
            bus.read_byte(addr & 0x3FFF)
        }
        else {
            panic!("PPU's bus not initialized");
        }
    }

    /// Write directly to PPU VRAM
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        if let Some(ref mut bus) = self.bus {
            bus.write_byte(addr & 0x3FFF, value);
        }
    }

    pub fn write_oam(&mut self, addr: u8, value: u8) {
        self.oam[addr as usize] = value;
    }

    pub fn load_bus(&mut self, bus: Io) {
        self.bus = Some(bus);
    }

    pub fn read_tile(&self, nametable: u16, x: usize, y: usize) -> u8 {
        let idx = (y * TILES_PER_ROW) + x;
        self.read_nametable(nametable, idx)
    }
}

// TODO: Latch behaviour

impl<Io: IoAccess> IoAccess for Ppu<Io> {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x2000 => {
                // Clear the write toggle
                *self.w.borrow_mut() = false;
                self.ctrl.value()
            },
            0x2001 => {
                self.mask.value()
            },
            0x2002 => {
                let status = self.status.borrow().value();
                // VBlank flag is clear on reading the status register
                self.status.borrow_mut().vblank = false;

                status
            },
            0x2003 => {
                0
            },
            0x2004 => {
                let data = self.oam[*self.oam_addr.borrow() as usize];
                // Increment OAM pointer
                let new_oam_addr = self.oam_addr.borrow().wrapping_add(1) % 256;
                *self.oam_addr.borrow_mut() = new_oam_addr;

                data
            },
            0x2005 => {
                self.x
            },
            // PPU Address
            0x2006 => {
                self.v.borrow().value() as u8
            },
            // PPU Data
            0x2007 => {
                let data = self.read_vram(self.v.borrow().value());
                *self.v.borrow_mut() += self.ctrl.vram_increment();

                data
            },

            _ => panic!("Invalid read from PPU: ${:04X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // PPU Control Register
            0x2000 => {
                self.ctrl.load(value);
                self.t.borrow_mut().set_nametable(value);
            },
            // PPU Mask
            0x2001 => {
                self.mask.load(value);
            },
            // OAM ADDR
            0x2003 => {
                *self.oam_addr.borrow_mut() = value as u16;
            },
            // OAM DATA
            0x2004 => {
                self.oam[*self.oam_addr.borrow() as usize] = value;
                // Increment OAM pointer
                let new_oam_addr = self.oam_addr.borrow().wrapping_add(1) % 256;
                *self.oam_addr.borrow_mut() = new_oam_addr;
            },
            // PPU Scroll
            0x2005 => {
                if *self.w.borrow() {
                    // When the write toggle is set
                    self.t.borrow_mut().set_y(value);
                }
                else {
                    // When the write toggle is clear
                    self.t.borrow_mut().set_coarse_x(value);
                    self.x = value & 0x07;
                }

                // Toggle w
                let w = *self.w.borrow();
                *self.w.borrow_mut() = !w;
            },
            // PPU Address
            0x2006 => {
                if *self.w.borrow() {
                    self.t.borrow_mut().set_low_byte(value);
                    self.v.borrow_mut().load(self.t.borrow().value());
                }
                else {
                    self.t.borrow_mut().set_high_byte(value);
                }

                // Toggle w
                let w = *self.w.borrow();
                *self.w.borrow_mut() = !w;
            },
            // PPU Data
            0x2007 => {
                let addr = self.v.borrow().value();
                self.write_vram(addr, value);

                *self.v.borrow_mut() += self.ctrl.vram_increment();
            }
            _ => {
                // FIXME: OAM DMA
                // FIXME: DMA uses OAM ADDR?
                if mask_is_set!(addr, 0xFF00) {
                    let oam_addr = (addr & 0xFF) as u8;
                    self.write_oam(oam_addr, value);
                }
            }
        }

        self.status.borrow_mut().lsb = value;
    }
}

impl<Io: IoAccess> Clockable<Option<Pixel>> for Ppu<Io> {
    fn tick(&mut self) -> Option<Pixel> {
        let pixel = self.run_cycle();

        self.cycle += 1;

        if self.cycle == CYCLES_PER_SCANLINE {
            self.scanline = (self.scanline + 1) % NUM_SCANLINES;
        }

        self.cycle %= CYCLES_PER_SCANLINE;

        pixel
    }
}

mod helpers {
    use super::Pixel;

    pub fn calc_nametable_address(base: u16, tile_offset: usize) -> u16 {
        base + (tile_offset as u16)
    }

    pub fn calc_attribute_address(base: u16, tile_row: usize, tile_col: usize) -> u16 {
        let row = tile_row as u16;
        let col = tile_col as u16;

        let row_offset = (row / 4) * 8;
        let col_offset  = col / 4;

        base + row_offset + col_offset
    }

    pub fn color_to_pixel(c: u32) -> Pixel {
        (
            ((c >> 16) & 0xFF) as u8,
            ((c >> 8) & 0xFF) as u8,
            (c & 0xFF) as u8,
        )
    }

    // Determine the pixel priority given the background and sprite data
    pub fn pixel_mux(bg_pattern: (u8, u8), sp_pattern: (u8, u8), sp_priority: bool) -> (u8, u8, u8) {
        if bg_pattern.0 == 0 && sp_pattern.0 == 0 {
            (0x00, 0x00, 0x00)
        }
        else if bg_pattern.0 == 0 && sp_pattern.0 > 0 {
            (sp_pattern.0, sp_pattern.1, 0x10)
        }
        else if bg_pattern.0 > 0 && sp_pattern.0 == 0 {
            (bg_pattern.0, bg_pattern.1, 0x00)
        }
        else if bg_pattern.0 > 0 && sp_pattern.0 > 0 && !sp_priority {
            (sp_pattern.0, sp_pattern.1, 0x10)
        }
        else if bg_pattern.0 > 0 && sp_pattern.0 > 0 && sp_priority {
            (bg_pattern.0, bg_pattern.1, 0x00)
        }
        else {
            // Should not get here
            (0, 0, 0)
        }
    }
}

//----------------------------------------------------------------------------------------------------------------------
// Tests
//----------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mux() {
        let bg = (0, 0);
        let sp = (0, 0);
        assert_eq!(helpers::pixel_mux(bg, sp, false), (0, 0, 0));

        let bg = (0, 0);
        let sp = (0, 0);
        assert_eq!(helpers::pixel_mux(bg, sp, true), (0, 0, 0));

        let bg = (0, 0);
        let sp = (1, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, false), (1, 2, 0x10));

        let bg = (0, 0);
        let sp = (1, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, true), (1, 2, 0x10));

        let bg = (1, 3);
        let sp = (0, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, false), (1, 3, 0x00));

        let bg = (1, 3);
        let sp = (0, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, true), (1, 3, 0x00));

        let bg = (1, 3);
        let sp = (1, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, false), (1, 2, 0x10));

        let bg = (1, 3);
        let sp = (1, 2);
        assert_eq!(helpers::pixel_mux(bg, sp, true), (1, 3, 0x00));
    }

    #[test]
    fn visible_pixel_output() {
        let mut ppu = init_ppu();

        let mut pixel_counter = 0;

        for _ in 0..CYCLES_PER_FRAME {
            if ppu.tick().is_some() {
                pixel_counter += 1;
            }
        }

        assert_eq!(pixel_counter, DISPLAY_WIDTH * DISPLAY_HEIGHT);
    }

    #[test]
    fn render_one_sprite_pixel() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = true;
        mask.background_enabled = false;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        // Y position
        ppu.write_byte(0x2003, 0x00);
        ppu.write_byte(0x2004, 0x00); // Sprite data is delayed by one, so subtract 1 from the Y position
        // Tile number
        ppu.write_byte(0x2003, 0x01);
        ppu.write_byte(0x2004, 0x01);
        // Attribute - Front Priority
        ppu.write_byte(0x2003, 0x02);
        ppu.write_byte(0x2004, 0x20);

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Write into nametable
        ppu.write_vram(0x2000, 0x01);
        // Write attribute - Top Left - Background Palette 1
        ppu.write_vram(0x23C0, 0x01);
        // Set first color in Background Palette 1
        ppu.write_vram(0x3F05, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Sprites cannot be displayed on the first scanline
        // Run for one more scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            if let Some(color) = ppu.tick() {
                assert_ne!(color, target_color);
            }
        }

        let pixel = ppu.tick();

        // The color of the pixel should be the index one of the color table
        let color = pixel.unwrap();
        assert_eq!(color, target_color, "Color was: RGB{:?}", color);
    }

    #[test]
    fn render_one_sprite_pixel_dma() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = true;
        mask.background_enabled = false;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        let oam_data: [u8; 4] = [0x00, 0x01, 0x20, 0x00];
        for (i, oam_byte) in oam_data.iter().enumerate() {
            let addr = (0xFF00 | i) as u16;
            ppu.write_byte(addr, *oam_byte);
        }

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Sprites cannot be displayed on the first scanline
        // Run for one more scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            if let Some(color) = ppu.tick() {
                assert_ne!(color, target_color);
            }
        }

        let pixel = ppu.tick();

        // The color of the pixel should be the index one of the color table
        let color = pixel.unwrap();
        assert_eq!(color, target_color, "Color was: RGB{:?}", color);
    }

    #[test]
    fn render_one_sprite_pixel_x1() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = true;
        mask.background_enabled = false;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        let oam_data: [u8; 4] = [0x00, 0x01, 0x20, 0x01];
        for (i, oam_byte) in oam_data.iter().enumerate() {
            let addr = (0xFF00 | i) as u16;
            ppu.write_byte(addr, *oam_byte);
        }

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Sprites cannot be displayed on the first scanline
        // Run for one more scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            if let Some(color) = ppu.tick() {
                assert_ne!(color, target_color);
            }
        }

        // Sprite is at X=1
        assert_ne!(ppu.tick().unwrap(), target_color);
        assert_eq!(ppu.tick().unwrap(), target_color);
    }

    #[test]
    fn render_one_sprite_pixel_y_238() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = true;
        mask.background_enabled = false;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        let oam_data: [u8; 4] = [238, 0x01, 0x20, 0x00];
        for (i, oam_byte) in oam_data.iter().enumerate() {
            let addr = (0xFF00 | i) as u16;
            ppu.write_byte(addr, *oam_byte);
        }

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Run for all but the last scanline
        for _ in 0..239 {
            for _ in 0..CYCLES_PER_SCANLINE {
                if let Some(color) = ppu.tick() {
                    assert_ne!(color, target_color);
                }
            }
        }
        // First pixel of the last scanline, should be set to the target color
        assert_eq!(ppu.tick().unwrap(), target_color);
    }

    #[test]
    fn render_one_sprite_pixel_y_238_sprite_hit() {
        // Screen bottom test - #3

        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = true;
        mask.background_enabled = true;
        mask.show_background_left = true;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        let oam_data: [u8; 4] = [238, 0x01, 0x00, 128];
        for (i, oam_byte) in oam_data.iter().enumerate() {
            let addr = (0xFF00 | i) as u16;
            ppu.write_byte(addr, *oam_byte);
        }

        // Sprite pattern
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Background Pattern
        ppu.write_vram(0x0027, 0xFF);
        ppu.write_vram(0x002F, 0x00);

        let tile_idx = ((29 * TILES_PER_ROW) + 16) as u16;
        ppu.write_vram(0x2000 + tile_idx, 0x02);

        // Set the first color in the Background Palette 1
        ppu.write_vram(0x3F05, 0x01);
        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Run for all but the last scanline
        for _ in 0..239 {
            for _ in 0..CYCLES_PER_SCANLINE {
                if let Some(color) = ppu.tick() {
                    assert_ne!(color, target_color);
                }
            }
        }

        for _ in 0..(16*8) {
            assert_ne!(ppu.tick().unwrap(), target_color);
        }

        // At this point, the sprite zero flag should be clear
        assert!(bit_is_clear!(ppu.read_byte(0x2002), 6));

        assert_eq!(ppu.tick().unwrap(), target_color);

        // The last cycle should have caused the sprite to hit with the background
        assert!(bit_is_set!(ppu.read_byte(0x2002), 6));
    }

    #[test]
    fn render_one_sprite_pixel_hide_left_x_gr_zero() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.sprites_enabled = true;
        mask.show_sprites_left = false;
        mask.background_enabled = false;

        ppu.write_byte(0x2001, mask.value());

        // -- Setup OAM
        let oam_data: [u8; 4] = [0x00, 0x01, 0x20, 0x01];
        for (i, oam_byte) in oam_data.iter().enumerate() {
            let addr = (0xFF00 | i) as u16;
            ppu.write_byte(addr, *oam_byte);
        }

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x01);
        ppu.write_vram(0x0018, 0x00);

        // Set first color in Sprite Palette 1
        ppu.write_vram(0x3F11, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // Sprites cannot be displayed on the first scanline
        // Run for one more scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            if let Some(color) = ppu.tick() {
                assert_ne!(color, target_color);
            }
        }

        // Showing sprites on the left most 8 pixels is disabled
        for _ in 0..8 {
            assert_ne!(ppu.tick().unwrap(), target_color);
        }

        assert_eq!(ppu.tick().unwrap(), target_color);
    }

    #[test]
    fn render_one_pixel() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.background_enabled = true;
        mask.show_background_left = true;

        ppu.write_byte(0x2001, mask.value());

        // Clear scroll
        ppu.write_byte(0x2005, 0);
        ppu.write_byte(0x2005, 0);

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0x80);
        ppu.write_vram(0x0018, 0x00);

        // Write into nametable
        ppu.write_vram(0x2000, 0x01);
        // Write attribute - Top Left - Background Palette 1
        ppu.write_vram(0x23C0, 0x01);
        // Set first color in Background Palette 1
        ppu.write_vram(0x3F05, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        // The first tick of the visible scanline should have a pixel
        let pixel = ppu.tick();
        assert!(pixel.is_some());

        // The color of the pixel should be the index one of the color table
        let color = pixel.unwrap();
        assert_eq!(color, helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]), "Color was: RGB{:?}", color);
    }

    #[test]
    fn render_eight_pixels_tile1() {
        let mut ppu = init_ppu();

        // Enable sprites
        let mut mask = PpuMask::default();
        mask.background_enabled = true;

        ppu.write_byte(0x2001, mask.value());

        // Clear scroll
        ppu.write_byte(0x2005, 0);
        ppu.write_byte(0x2005, 0);

        // Write pattern into pattern table
        ppu.write_vram(0x0010, 0xFF);
        ppu.write_vram(0x0018, 0x00);

        // Write into nametable
        ppu.write_vram(0x2001, 0x01);
        // Write attribute - Top Left - Background Palette 1
        ppu.write_vram(0x23C0, 0x01);
        // Set first color in Background Palette 1
        ppu.write_vram(0x3F05, 0x01);

        // Run the PPU for the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            assert!(ppu.tick().is_none());
        }

        let target_color = helpers::color_to_pixel(COLOR_INDEX_TO_RGB[0x01]);

        // The first tile has no data
        for _ in 0..8 {
            assert_ne!(ppu.tick().unwrap(), target_color);
        }

        // All pixels active for tile 1
        for i in 0..8 {
            assert_eq!(ppu.tick().unwrap(), target_color, "Tile 1: Failed for {}", i);
        }

        assert_ne!(ppu.tick().unwrap(), target_color);
    }

    #[test]
    fn oam_write() {
        let mut ppu = init_ppu();

        // Setup OAM
        ppu.write_byte(0x2003, 0x00);
        ppu.write_byte(0x2003, 0x01);
        ppu.write_byte(0x2004, 0x01);

        assert_eq!(ppu.oam[0x01], 0x01);
    }

    #[test]
    fn determine_nametable_address() {
        assert_eq!(helpers::calc_nametable_address(0x2000, 0x100), 0x2100);
    }

    #[test]
    fn determine_attribute_address() {
        assert_eq!(helpers::calc_attribute_address(0x23C0, 0, 0), 0x23C0);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 0, 3), 0x23C0);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 0, 30), 0x23C7);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 0, 31), 0x23C7);

        assert_eq!(helpers::calc_attribute_address(0x23C0, 4, 0), 0x23C8);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 4, 3), 0x23C8);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 4, 4), 0x23C9);
        assert_eq!(helpers::calc_attribute_address(0x23C0, 4, 7), 0x23C9);
    }

    #[test]
    fn vram_write() {
        let mut ppu = init_ppu();

        // VRAM increment mode to 1
        ppu.write_byte(0x2000, 0x00);
        // Load VRAM addr $150
        ppu.write_byte(0x2006, 0x01);
        ppu.write_byte(0x2006, 0x50);
        // Write to VRAM
        ppu.write_byte(0x2007, 0xDE);
        ppu.write_byte(0x2007, 0xAD);

        assert_eq!(ppu.read_vram(0x0150), 0xDE);
        assert_eq!(ppu.read_vram(0x0151), 0xAD);
    }

    #[test]
    fn vram_write_inc32() {
        let mut ppu = init_ppu();

        // VRAM increment mode to 1
        ppu.write_byte(0x2000, 0x04);
        // Load VRAM addr $150
        ppu.write_byte(0x2006, 0x01);
        ppu.write_byte(0x2006, 0x50);
        // Write to VRAM
        ppu.write_byte(0x2007, 0xDE);
        ppu.write_byte(0x2007, 0xAD);

        assert_eq!(ppu.read_vram(0x0150), 0xDE);
        assert_eq!(ppu.read_vram(0x0150 + 32), 0xAD);
    }

    #[test]
    fn vram_read() {
        let mut ppu = init_ppu();

        // VRAM increment mode to 1
        ppu.write_byte(0x2000, 0x00);
        // Load VRAM addr $150
        ppu.write_byte(0x2006, 0x01);
        ppu.write_byte(0x2006, 0x50);
        // Write to VRAM
        ppu.write_byte(0x2007, 0xDE);
        ppu.write_byte(0x2007, 0xAD);
        // Load VRAM addr $150
        ppu.write_byte(0x2006, 0x01);
        ppu.write_byte(0x2006, 0x50);

        let data = (ppu.read_byte(0x2007), ppu.read_byte(0x2007));

        assert_eq!(data, (0xDE, 0xAD));
    }

    #[test]
    fn vblank() {
        const CYCLES_TO_VBLANK: usize = CYCLES_PER_SCANLINE * 242 + 2;

        let mut ppu = init_ppu();

        for _ in 0..CYCLES_TO_VBLANK-1 {
            ppu.tick();
            assert!(bit_is_clear!(ppu.read_byte(0x2002), 7));
        }

        ppu.tick();
        assert!(bit_is_set!(ppu.read_byte(0x2002), 7));
        // Should be cleared after reading
        assert!(bit_is_clear!(ppu.read_byte(0x2002), 7));
        // Only set during cycle 1
        ppu.tick();
        assert!(bit_is_clear!(ppu.read_byte(0x2002), 7));
    }

    #[test]
    fn scanline_state() {
        assert_eq!(Scanline::from(261), Scanline::PreRender);
        assert_eq!(Scanline::from(0), Scanline::Visible);
        assert_eq!(Scanline::from(239), Scanline::Visible);
        assert_eq!(Scanline::from(240), Scanline::PostRender);
        assert_eq!(Scanline::from(241), Scanline::VBlank);
        assert_eq!(Scanline::from(260), Scanline::VBlank);
    }

    #[test]
    #[should_panic]
    fn scanline_state_invalid() {
        Scanline::from(262);
    }

    #[test]
    fn scanline_transition() {
        let mut ppu = init_ppu();
        assert_eq!(ppu.scanline, NUM_SCANLINES - 1);

        for _ in 0..341 {
            ppu.tick();
        }

        assert_eq!(ppu.scanline, 0);
    }
    struct FakeBus {
        vram: [u8; 0x4000],
    }

    impl Default for FakeBus {
        fn default() -> Self {
            FakeBus {
                vram: [0; 0x4000],
            }
        }
    }

    impl IoAccess for FakeBus {
        fn read_byte(&self, addr: u16) -> u8 {
            self.vram[addr as usize]
        }
        fn write_byte(&mut self, addr: u16, value: u8) {
            self.vram[addr as usize] = value;
        }
    }

    fn init_ppu() -> Ppu<FakeBus> {
        let mut ppu: Ppu<FakeBus> = Ppu::default();
        ppu.load_bus(FakeBus::default());

        ppu
    }
}
