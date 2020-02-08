//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//
mod regs;

use regs::*;
use crate::common::{IoAccess, Clockable, Register};

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
pub struct Ppu {
    vram: [u8; 0x4000],
    oam: [u8; 256],

    ctrl: PpuCtrl,     // PPUCTRL - Control Register
    status: PpuStatus, // PPUSTATUS - Status Register
    mask: PpuMask,     // PPUMASK - Mask Register (Render controls)
    addr: PpuAddr,     // PPUADDR - VRAM Address

    cycle: usize,      // Cycle count per scanline
    scanline: usize,   // Current scanline
}

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            vram: [0; 0x4000],
            oam: [0; 256],

            ctrl: PpuCtrl::default(),
            status: PpuStatus::default(),
            mask: PpuMask::default(),
            addr: PpuAddr::default(),

            cycle: 0,
            scanline: NUM_SCANLINES - 1, // Initialize to the Pre-render scanline
        }
    }
    
    fn run_cycle(&mut self, io: &mut dyn IoAccess) {
        match Scanline::from(self.scanline) {
            Scanline::PreRender => {
                self.status.vblank = false;
                // Same as a normal scanline but no output to the screen
                // Fills shift register with data for first two tiles of the next scanline
            },
            Scanline::Visible => {
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

                // Every cycles produces a pixel....
            },
            Scanline::PostRender => {
                // PPU is idle
            },
            Scanline::VBlank => {
                self.status.vblank = false;
                // Set NMI during 2nd cycle of VBlank period
                if self.cycle == 1 {
                    // TODO: Set NMI if NMI is enabled
                }
            }
        }
    }

    /// Check if the PPU is in vertical blanking mode
    pub fn is_vblank(&self) -> bool {
        self.status.vblank
    }

    /// Read directly from PPU VRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }

    /// Write directly to PPU VRAM
    pub fn write_vram(&mut self, addr: u16, value: u8) {
        self.vram[addr as usize] = value;
    }
}

impl IoAccess for Ppu {
    fn read_byte(&self, _addr: u16) -> u8 {
        // TODO: Latch behaviour
        0
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        // TODO: Latch behaviour
        match addr {
            // PPU Control Register
            0x2000 => {
                self.ctrl.load(value);
            },
            // PPU Address
            0x2006 => {
                self.addr.load(value as u16);
            },
            0x2007 => {
                self.write_vram(self.addr.value(), value);
                self.addr += self.ctrl.vram_increment();
            }
            _ => {}
        }
    }
}

impl Clockable for Ppu {
    fn tick(&mut self, io: &mut dyn IoAccess) {
        self.run_cycle(io);

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
        let mut io = PpuIoBus{};
        let mut ppu = Ppu::new();

        assert_eq!(ppu.scanline, NUM_SCANLINES - 1);

        for _ in 0..341 {
            ppu.tick(&mut io);
        }

        assert_eq!(ppu.scanline, 0);
    }

    struct PpuIoBus {}
    impl IoAccess for PpuIoBus {
        fn read_byte(&self, _addr: u16) -> u8 {
            0
        }
        fn write_byte(&mut self, _addr: u16, _value: u8) {

        }
    }
}
