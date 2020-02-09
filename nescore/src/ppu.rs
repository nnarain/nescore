//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//
pub mod bus;
mod regs;

use regs::*;
use crate::common::{IoAccess, Clockable, Register};

use std::cell::RefCell;


const NUM_SCANLINES: usize = 262;
const CYCLES_PER_SCANLINE: usize = 341;


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
        // Cycle 0 is idle

        // Cycles 1 - 256: is accessing data for each tile
        // 2 cycles for each memory access
        // accesses: 2 nametable bytes, attribute, pattern table low, pattern table high
        // Do something every 8 bytes?

        // Cycles 257 - 320: Get tile data for sprites on next scanline
        // accesses: 2 garbage nametable bytes, pattern table low, pattern table high

        // Cycles 321-336: Fetch first two tiles of the next scanline
        // accesses: 2 nametable bytes, attribute, pattern table low, pattern table high

        // Cycles 337 - 340
        // Two nametable bytes are fetch, unknown purpose


        // In addition to all that, the sprite evaluation happens independently

        match cycle {
            0 => {
                // Idle cycle
            },
            1..=256 => {
                // 4 memory accesses each taking 2 cycles
                if cycle % 8 == 0 {

                }
            },
            _ => panic!("Invalid cycle for scanline! {}", cycle),
        }
    }

    fn read_nametable(&self, idx: u8) -> u8 {
        0
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

#[cfg(test)]
mod tests {
    use super::*;

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

        let mut ppu: Ppu<FakeBus> = Ppu::default();

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
        let mut ppu: Ppu<FakeBus> = Ppu::default();

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
