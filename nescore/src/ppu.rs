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
    fn run_cycle(&mut self) {
        match Scanline::from(self.scanline) {
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
                    // TODO: Set NMI
                }
            }
        }

        // TODO: Every cycles produces a pixel
    }

    // Processing a single scanline per cycle
    fn process_scanline(&mut self, cycle: usize) {
        // Cycles 257 - 320: Get tile data for sprites on next scanline
        // accesses: 2 garbage nametable bytes, pattern table low, pattern table high

        // Cycles 321-336: Fetch first two tiles of the next scanline
        // accesses: 2 nametable bytes, attribute, pattern table low, pattern table high

        // Cycles 337 - 340
        // Two nametable bytes are fetch, unknown purpose


        // In addition to all that, the sprite evaluation happens independently

        match cycle {
            0 => {
                // Cycle 0 is idle
            },
            1..=256 => {
                // 4 memory accesses each taking 2 cycles
                if cycle % 8 == 0 {
                    // TODO: Fix types for offsets

                    // Get pixel scroll offset
                    let (scroll_x, scroll_y) = self.ctrl.base_scroll();
                    // Determine tile offset (+3 because the first two tiles have already been loaded)
                    let coarse = (scroll_x / 8 + 3, scroll_y / 8);
                    // Determine tile index for nametable
                    let tile_idx = (coarse.1 * TILES_PER_ROW as u16) + coarse.0;

                    // Read tile number from nametable
                    let tile_no = self.read_nametable(tile_idx as usize);
                    // Read pattern from battern pattern table memory
                    let pattern = self.read_pattern(self.ctrl.background_pattern_table(), tile_no);
                    // Fetch tile attributes
                    let attribute = self.read_attribute(coarse.1 as usize, coarse.0 as usize);

                    // Load shift registers
                    self.tile_reg.load(pattern);
                    self.pal_reg.latch(attribute);
                }
            },
            257..=320 => {

            },
            321..=336 => {

            },
            337..=340 => {

            },
            _ => panic!("Invalid cycle for scanline! {}", cycle),
        }
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

    fn read_pattern(&self, base: u16, tile_no: u8) -> (u8, u8) {
        let tile_no = tile_no as u16;

        let lo = self.read_vram(base + tile_no);
        let hi = self.read_vram(base + tile_no + 8);

        (lo, hi)
    }

    /// Check if the PPU is in vertical blanking mode
    pub fn is_vblank(&self) -> bool {
        self.status.vblank
    }

    /// Read directly from PPU VRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.bus.as_ref().map(|bus| bus.read_byte(addr)).unwrap()
    }

    /// Write directly to PPU VRAM
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        self.bus.as_mut().map(|bus| bus.write_byte(addr, value));
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
            0x2004 => {
                let data = self.oam[*self.oam_addr.borrow() as usize];
                *self.oam_addr.borrow_mut() = self.oam_addr.borrow().wrapping_add(1);

                data
            },
            0x2005 => {
                self.scroll.x
            },
            0x2006 => {
                self.addr.borrow().value() as u8
            },
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
                *self.oam_addr.borrow_mut() = self.oam_addr.borrow().wrapping_add(1);
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

impl<Io: IoAccess> Clockable for Ppu<Io> {
    fn tick(&mut self) {
        self.run_cycle();

        self.cycle += 1;

        if self.cycle == CYCLES_PER_SCANLINE {
            self.scanline = (self.scanline + 1) % NUM_SCANLINES;
        }

        self.cycle %= CYCLES_PER_SCANLINE;
    }
}

mod helpers {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
