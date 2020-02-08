///
/// nescore/lib.rs
///
/// @author Natesh Narain <nnaraindev@gmail.com>
///

// nescore submodules

#[macro_use] mod bit;

mod common;
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

use common::Clockable;

/// CPU Cycles in a frame: (256x240) - resolution, 1 px per PPU tick. 1 CPU tick for 3 PPU ticks
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

    /// Directly set the CPU entry point
    /// ```
    /// # use nescore::Nes;
    /// let nes = Nes::new().entry(0xC000);
    /// ```
    pub fn entry(mut self, entry_addr: u16) -> Self {
        self.cpu.set_pc(entry_addr);
        self
    }

    /// Builder function to allow inserting the cartridge
    pub fn with_cart(mut self, cart: Cartridge) -> Self {
        self.insert(cart);
        self
    }

    /// Builder function to set debug mode
    pub fn debug_mode(mut self, debug: bool) -> Self {
        self.cpu.set_debug(debug);
        self
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

    /// Run until the CPU's PC is at address **addr**
    pub fn run_until(&mut self, addr: u16) {
        // TODO: Time limit
        if let Some(ref mut mapper) = self.mapper {
            let mut cpu_io_bus = CpuIoBus::new(&mut self.ppu, mapper);

            while self.cpu.get_pc() != addr {
                self.cpu.tick(&mut cpu_io_bus);
            }
        }
    }

    /// Check if the CPU is in an infinite loop state
    pub fn is_holding(&self) -> bool {
        self.cpu.is_holding()
    }

    /// Load a cartridge
    pub fn insert(&mut self, cart: Cartridge) {
        // Consume provided cartridge and get the mapper
        self.mapper = Some(mapper::from_cartridge(cart));
    }

    //------------------------------------------------------------------------------------------------------------------
    // Inspect the state of the NES system
    //------------------------------------------------------------------------------------------------------------------

    /// Get the CPU's program counter
    pub fn get_program_counter(&self) -> u16 {
        self.cpu.get_pc()
    }

    /// Read the byte, at the specified address, from CPU's internal RAM
    pub fn read_cpu_ram(&self, addr: u16) -> u8 {
        self.cpu.read_ram(addr)
    }

    /// Read directly from VRAM
    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        self.ppu.read_direct(addr)
    }
}

#[cfg(test)]
mod tests {

}
