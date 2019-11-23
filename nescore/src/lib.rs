///
/// nescore
/// 
/// @author Natesh Narain <nnaraindev@gmail.com>
/// 

mod io;
mod clk;
mod cpu;
mod ppu;
mod cart;
mod mapper;

use cpu::Cpu;
use cpu::bus::CpuIoBus;

use ppu::Ppu;

use mapper::Mapper;
use cart::Cartridge;

use clk::Clockable;

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
        if let Some(ref mapper) = self.mapper {
            // TODO: Iterate for number of cycles to produce a frame. Simulate component clocks
            // TODO: Mapper as part of the CPU IO interface
            let mut cpu_io_bus = CpuIoBus::new(&mut self.ppu);
            self.cpu.tick(&mut cpu_io_bus);
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
