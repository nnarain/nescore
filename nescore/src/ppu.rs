//
// ppu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 10 2019
//

use crate::io::IoAccess;
use crate::clk::Clockable;

const NUM_SCANLINES: usize = 262;
const CYCLES_PER_SCANLINE: usize = 341;

#[derive(Clone, Copy)]
enum IncMode {
    Add1 = 1,
    Add32 = 32,
}

impl IncMode {
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

enum SpriteSize {
    Size8x8, Size8x16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Scanline {
    PreRender,
    Visible,
    PostRender,
    VBlank,
}

impl Scanline {
    pub fn from(scanline: isize) -> Scanline {
        match scanline {
            -1 => Scanline::PreRender,
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

    // PPUCTRL
    vram_addr: u16,
    inc_mode: IncMode,

    cycle: usize,
    scanline: isize,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            vram: [0; 0x4000],
            oam: [0; 256],

            vram_addr: 0,
            inc_mode: IncMode::Add1,

            cycle: 0,
            scanline: -1,
        }
    }
    
    fn run_cycle(&mut self, io: &mut dyn IoAccess) {
        let state = Scanline::from(self.scanline);
        match state {
            Scanline::PreRender => {},
            Scanline::Visible => {},
            Scanline::PostRender => {},
            Scanline::VBlank => {}
        }
    }

    pub fn read_direct(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
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
            0x2000 => {
                self.inc_mode = if bit_is_set!(value, 2) { IncMode::Add32 } else { IncMode::Add1 };
            },
            0x2006 => {
                self.vram_addr = (self.vram_addr << 8) | (value as u16);
            },
            0x2007 => {
                self.vram[self.vram_addr as usize] = value;
                self.vram_addr += self.inc_mode.to_u16();
            }
            _ => {}
        }
    }
}

impl Clockable for Ppu {
    fn tick(&mut self, io: &mut dyn IoAccess) {
        self.run_cycle(io);

        // Increment cycle and scanline counters
        // TODO: Cleaner way to do this?
        self.cycle += 1;

        if self.cycle == CYCLES_PER_SCANLINE {
            self.scanline += 1;

            if self.scanline == 262 {
                self.scanline = -1;
            }
        }

        self.cycle %= CYCLES_PER_SCANLINE;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanline_state() {
        assert_eq!(Scanline::from(-1), Scanline::PreRender);
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

        assert_eq!(ppu.scanline, -1);

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
