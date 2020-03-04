//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//
pub mod bus;
mod regs;
mod hw;

use regs::*;
use hw::*;
use crate::common::{IoAccess, Clockable, Register};

use std::cell::RefCell;

const NUM_SCANLINES: usize = 262;
const CYCLES_PER_SCANLINE: usize = 341;
const TILES_PER_ROW: usize = 32;

pub type Pixel = (u8, u8, u8);
pub const DISPLAY_WIDTH: usize = 256;
pub const DISPLAY_HEIGHT: usize = 240;
pub const CYCLES_PER_FRAME: usize = NUM_SCANLINES * CYCLES_PER_SCANLINE;

const COLOR_INDEX_TO_RGB: [u32; 64] = [
    0x7C7C7C, 0x0000FC, 0x0000BC, 0x4428BC, 0x940084, 0xA80020, 0xA81000, 0x881400,
    0x503000, 0x007800, 0x006800, 0x005800, 0x004058, 0x000000, 0x000000, 0x000000,
    0xBCBCBC, 0x0078F8, 0x0058F8, 0x6844FC, 0xD800CC, 0xE40058, 0xF83800, 0xE45C10,
    0xAC7C00, 0x00B800, 0x00A800, 0x00A844, 0x008888, 0x000000, 0x000000, 0x000000,
    0xF8F8F8, 0x3CBCFC, 0x6888FC, 0x9878F8, 0xF878F8, 0xF85898, 0xF87858, 0xFCA044,
    0xF8B800, 0xB8F818, 0x58D854, 0x58F898, 0x00E8D8, 0x787878, 0x000000, 0x000000,
    0xFCFCFC, 0xA4E4FC, 0xB8B8F8, 0xD8B8F8, 0xF8B8F8, 0xF8A4C0, 0xF0D0B0, 0xFCE0A8,
    0xF8D878, 0xD8F878, 0xB8F8B8, 0xB8F8D8, 0x00FCFC, 0xF8D8F8, 0x000000, 0x000000
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
    oam: [u8; 256],

    ctrl: PpuCtrl,          // PPUCTRL   - Control Register
    status: PpuStatus,      // PPUSTATUS - Status Register
    mask: PpuMask,          // PPUMASK   - Mask Register (Render controls)
    addr: RefCell<PpuAddr>, // PPUADDR   - VRAM Address
    scroll: PpuScroll,      // PPUSCROLL - Scroll register
    oam_addr: RefCell<u16>, // OAMADDR   - OAM Address

    tile_reg: TileRegister,    // PPU tile shift registers
    pal_reg: PaletteRegister, // PPU palette shift registers

    cycle: usize,           // Cycle count per scanline
    scanline: usize,        // Current scanline

    bus: Option<Io>,
}

impl<Io: IoAccess> Default for Ppu<Io> {
    fn default() -> Self {
        Ppu{
            oam: [0; 256],

            ctrl: PpuCtrl::default(),
            status: PpuStatus::default(),
            mask: PpuMask::default(),
            addr: RefCell::new(PpuAddr::default()),
            scroll: PpuScroll::default(),
            oam_addr: RefCell::new(0),

            tile_reg: TileRegister::default(),
            pal_reg: PaletteRegister::default(),

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
                self.status.vblank = false;
                // Same as a normal scanline but no output to the screen
                self.process_scanline(self.cycle);
            },
            Scanline::Visible => {
                self.process_scanline(self.cycle);
            },
            Scanline::PostRender => {
                // PPU is idle
            },
            Scanline::VBlank => {
                self.status.vblank = true;
                // Set NMI during 2nd cycle of VBlank period
                if self.ctrl.nmi_enable && self.cycle == 1 {
                    self.raise_interrupt();
                }
            }
        }

        // TODO: Every cycles produces a pixel
        if scanline == Scanline::Visible && self.cycle <= 255 {
            Some(self.generate_pixel())
        }
        else {
            None
        }
    }

    // Processing a single scanline per cycle
    fn process_scanline(&mut self, cycle: usize) {
        match cycle {
            0 => {
                // Cycle 0 is idle
            },
            1..=256 => {
                // 4 memory accesses each taking 2 cycles
                // In addition to all that, the sprite evaluation happens independently
                if cycle % 8 == 0 {
                    let dot = (cycle - 1) as u8;
                    self.load_shift_registers(dot, 2, self.scanline as u8);
                }
            },
            257..=320 => {
                // Cycles 257 - 320: Get tile data for sprites on next scanline
                // accesses: 2 garbage nametable bytes, pattern table low, pattern table high
            },
            321..=336 => {
                // Cycles 321-336: Fetch first two tiles of the next scanline
                // accesses: 2 nametable bytes, attribute, pattern table low, pattern table high
                if cycle % 8 == 0 {
                    let dot = (cycle - 321) as u8;
                    let scanline = (self.scanline + 1) % NUM_SCANLINES;
                    self.load_shift_registers(dot, 0, scanline as u8);
                }
            },
            337..=340 => {
                // Cycles 337 - 340
                // Two nametable bytes are fetch, unknown purpose
            },
            _ => panic!("Invalid cycle for scanline! {}", cycle),
        }

        // Tick shift register
        match cycle {
            2..=257 | 322..=335 => self.tick_shifters(),
            _ => {}
        }
    }

    fn load_shift_registers(&mut self, dot: u8, tile_x_offset: u8, scanline: u8) {
        // Get pixel scroll offset
        let scroll = self.scroll.offset();
        let (base_x, base_y) = (scroll.0 as usize + dot as usize, scroll.1 as usize + scanline as usize);

        // Determine tile offset
        let coarse = ((base_x / 8) + tile_x_offset as usize, base_y / 8);
        // Determine tile index for nametable
        let tile_idx = (coarse.1 as usize * TILES_PER_ROW) + coarse.0 as usize;

        // Read tile number from nametable
        let tile_no = self.read_nametable(tile_idx);

        // Read pattern from pattern table memory
        let fine_y = (base_y % 8) as u8;
        let pattern = self.read_pattern(self.ctrl.background_pattern_table(), tile_no, fine_y);
        // Fetch tile attributes
        let attribute = self.read_attribute(coarse.1 as usize, coarse.0 as usize);

        // Load shift registers
        self.tile_reg.load(pattern);
        self.pal_reg.latch(attribute);
    }

    fn read_nametable(&self, idx: usize) -> u8 {
        let addr = helpers::calc_nametable_address(self.ctrl.nametable(), idx);
        self.read_vram(addr)
    }

    fn read_attribute(&self, tile_row: usize, tile_col: usize) -> u8 {
        let addr = helpers::calc_attribute_address(self.ctrl.attribute(), tile_row, tile_col);
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
        let tile_no = tile_no as u16;
        // 16 bytes per tile
        let tile_offset = (tile_no * 16) + fine_y as u16;

        let lo = self.read_vram(base + tile_offset);
        let hi = self.read_vram(base + tile_offset + 8);

        (lo, hi)
    }

    fn generate_pixel(&self) -> Pixel {
        // Use fine X to select the pixel to render
        let fine_x = (self.scroll.x % 8) as u8;

        // Fetch pattern and attributes from shifters
        let pattern = self.tile_reg.get_value(fine_x);
        let attribute = self.pal_reg.get_value(fine_x);

        // Determine palette offset: http://wiki.nesdev.com/w/index.php/PPU_palettes
        // TODO: Background vs sprite palette selection
        let palette_offset = 0x00 | (attribute << 2) | pattern;

        // Fetch color from palette
        let color = self.read_vram(0x3F00 + palette_offset as u16) as usize;

        helpers::color_to_pixel(COLOR_INDEX_TO_RGB[color])
    }

    fn tick_shifters(&mut self) {
        self.tile_reg.tick();
        self.pal_reg.tick();
    }

    /// Check if the PPU is in vertical blanking mode
    pub fn is_vblank(&self) -> bool {
        self.status.vblank
    }

    /// Raise NMI interrupt
    fn raise_interrupt(&mut self) {
        if let Some(ref mut bus) = self.bus {
            bus.raise_interrupt();
        }
    }

    /// Read directly from PPU VRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        if let Some(ref bus) = self.bus {
            bus.read_byte(addr)
        }
        else {
            panic!("PPU's bus not initialized");
        }
    }

    /// Write directly to PPU VRAM
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        if let Some(ref mut bus) = self.bus {
            bus.write_byte(addr, value);
        }
    }

    pub fn load_bus(&mut self, bus: Io) {
        self.bus = Some(bus);
    }
}

// TODO: Latch behaviour

impl<Io: IoAccess> IoAccess for Ppu<Io> {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x2000 => {
                self.ctrl.value()
            },
            0x2001 => {
                self.mask.value()
            },
            0x2002 => {
                self.status.value()
            },
            0x2003 => {
                0
            },
            0x2004 => {
                let data = self.oam[*self.oam_addr.borrow() as usize];
                // FIXME
                // *self.oam_addr.borrow_mut() = self.oam_addr.borrow().wrapping_add(1);

                data
            },
            0x2005 => {
                self.scroll.x
            },
            // PPU Address
            0x2006 => {
                self.addr.borrow().value() as u8
            },
            // PPU Data
            0x2007 => {
                let data = self.read_vram(self.addr.borrow().value());
                *self.addr.borrow_mut() += self.ctrl.vram_increment();

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
            },
            // PPU Mask
            0x2001 => {
                self.mask.load(value);
            },
            // OAM ADDR
            0x2003 => {
                *self.oam_addr.borrow_mut() = value as u16;
                // FIXME
                // *self.oam_addr.borrow_mut() = self.oam_addr.borrow().wrapping_add(1);
            },
            // OAM DATA
            0x2004 => {
                self.oam[*self.oam_addr.borrow() as usize] = value;
            },
            // PPU Scroll
            0x2005 => {
                self.scroll.load(value);
            },
            // PPU Address
            0x2006 => {
                self.addr.borrow_mut().load(value as u16);
            },
            // PPU Data
            0x2007 => {
                let addr = self.addr.borrow().value();
                self.write_vram(addr, value);

                *self.addr.borrow_mut() += self.ctrl.vram_increment();
            }
            _ => {}
        }

        self.status.lsb = value;
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
            (c & 0xFF) as u8,
            ((c >> 8) & 0xFF) as u8,
            ((c >> 16) & 0xFF) as u8
        )
    }
}

//----------------------------------------------------------------------------------------------------------------------
// Tests
//----------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_pixel_output() {
        let mut ppu = init_ppu();

        // Run the pre-render scanline
        for _ in 0..CYCLES_PER_SCANLINE {
            let pixel = ppu.tick();
            assert!(pixel.is_none());
        }

        let mut pixel_counter = 0;

        // Run for visible scanlines
        for _ in 0..240 {
            for _ in 0..CYCLES_PER_SCANLINE {
                let pixel = ppu.tick();

                if pixel.is_some() {
                    pixel_counter += 1;
                }
            }
        }

        assert_eq!(pixel_counter, DISPLAY_WIDTH * DISPLAY_HEIGHT);
    }

    #[test]
    fn render_one_pixel() {
        let mut ppu = init_ppu();

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
        const CYCLES_TO_VBLANK: usize = CYCLES_PER_SCANLINE * 242 + 1;

        let mut ppu = init_ppu();

        for _ in 0..CYCLES_TO_VBLANK {
            ppu.tick();
        }

        assert_eq!(ppu.is_vblank(), true);
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
