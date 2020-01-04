///
/// nescore/lib.rs
///
/// @author Natesh Narain <nnaraindev@gmail.com>
///

// nescore submodules

#[macro_use] mod bit;

mod io;
mod clk;
mod cpu;
mod ppu;
mod mapper;

pub mod cart;

// Public re-exports
pub use cart::Cartridge;

use cpu::Cpu;
use cpu::bus::CpuIoBus;

use ppu::Ppu;

use mapper::Mapper;

use clk::Clockable;

// CPU Cycles in a frame: (256x240) - resolution, 1 px per PPU tick. 1 CPU tick for 3 PPU ticks
const CPU_CYCLES_PER_FRAME: usize = (256 * 240) / 3;

/// Representation of the NES system
pub struct Nes {
    cpu: Cpu,              // NES CPU
    ppu: Ppu,              // NES PPU
                           // TODO: APU
    mapper: Option<Mapper> // Catridge Mapper
}

impl Nes {
    pub fn new() -> Self {
        Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            mapper: None
        }
    }

    /// Run the emulator for a single frame
    pub fn emulate_frame(&mut self) {
        if let Some(ref mut mapper) = self.mapper {
            // TODO: Send audio and video data back to host
            let mut cpu_io_bus = CpuIoBus::new(&mut self.ppu, mapper);

            // Clock the CPU
            for _ in 0..CPU_CYCLES_PER_FRAME {
                self.cpu.tick(&mut cpu_io_bus);

                // TODO: Clock PPU
                // self.ppu.tick(&mut ppu_io_bus)
                // self.ppu.tick(&mut ppu_io_bus)
                // self.ppu.tick(&mut ppu_io_bus)
            }
        }
    }

    /// Load a cartridge
    /// TODO: Should the cartridge actually be consumed? (Multiple NES instances)
    pub fn insert(&mut self, cart: Cartridge) {
        // Consume provided cartridge and get the mapper
        self.mapper = Some(mapper::from_cartridge(cart));
    }
}

#[cfg(test)]
mod tests {

}
