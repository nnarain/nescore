///
/// nescore/lib.rs
///
/// @author Natesh Narain <nnaraindev@gmail.com>
///

// nescore submodules
pub mod cart;
// Public re-exports
pub use cart::Cartridge;

#[macro_use]
mod bit;

mod cpu;
mod ppu;
mod mapper;
mod common;

use cpu::Cpu;
use cpu::bus::CpuIoBus;
use ppu::Ppu;
use ppu::bus::PpuIoBus;
use mapper::Mapper;
use common::Clockable;

use std::rc::Rc;
use std::cell::RefCell;

/// CPU Cycles in a frame: (256x240) - resolution, 1 px per PPU tick. 1 CPU tick for 3 PPU ticks
const CPU_CYCLES_PER_FRAME: usize = (256 * 240) / 3;

/// Representation of the NES system
#[derive(Default)]
pub struct Nes {
    cpu: Rc<RefCell<Cpu<CpuIoBus>>>,    // NES CPU
    ppu: Rc<RefCell<Ppu<PpuIoBus>>>,    // NES PPU
                           // TODO: APU
    mapper: Option<Mapper> // Catridge Mapper
}

impl Nes {
    /// Directly set the CPU entry point
    /// ```
    /// # use nescore::Nes;
    /// let nes = Nes::default().entry(0xC000);
    /// ```
    pub fn entry(self, entry_addr: u16) -> Self {
        self.cpu.borrow_mut().set_pc(entry_addr);
        self
    }

    /// Builder function to allow inserting the cartridge
    pub fn with_cart(mut self, cart: Cartridge) -> Self {
        self.insert(cart);
        self
    }

    /// Builder function to set debug mode
    pub fn debug_mode(self, debug: bool) -> Self {
        self.cpu.borrow_mut().set_debug(debug);
        self
    }

    /// Run the emulator for a single frame
    pub fn emulate_frame(&mut self) {
        if self.mapper.is_some() {
            // TODO: Send audio and video data back to host

            // Clock the CPU
            for _ in 0..CPU_CYCLES_PER_FRAME {
                self.tick_master_clock();
            }
        }
    }

    /// Run until the CPU's PC is at address **addr**
    pub fn run_until(&mut self, addr: u16) {
        // TODO: Time limit
        if self.mapper.is_some() {
            while self.cpu.borrow().get_pc() != addr {
                self.tick_master_clock();
            }
        }
    }

    fn tick_master_clock(&mut self) {
        // One master clock cycle is 1 CPU cycle and 3 PPU cycles
        self.cpu.borrow_mut().tick();
        self.ppu.borrow_mut().tick();
        self.ppu.borrow_mut().tick();
        self.ppu.borrow_mut().tick();
    }

    /// Check if the CPU is in an infinite loop state
    pub fn is_holding(&self) -> bool {
        self.cpu.borrow().is_holding()
    }

    /// Load a cartridge
    pub fn insert(&mut self, cart: Cartridge) {
        // Consume provided cartridge and get the mapper
        let mapper = mapper::from_cartridge(cart);

        // Complete initialization of components
        let cpu_bus = CpuIoBus::new(self.ppu.clone(), mapper.clone());
        self.cpu.borrow_mut().load_bus(cpu_bus);

        let ppu_bus = PpuIoBus::new(mapper.clone());
        self.ppu.borrow_mut().load_bus(ppu_bus);

        self.mapper = Some(mapper);
    }

    //------------------------------------------------------------------------------------------------------------------
    // Inspect the state of the NES system
    //------------------------------------------------------------------------------------------------------------------

    /// Get the CPU's program counter
    pub fn get_program_counter(&self) -> u16 {
        self.cpu.borrow().get_pc()
    }

    /// Read the byte, at the specified address, from CPU's internal RAM
    pub fn read_cpu_ram(&self, addr: u16) -> u8 {
        self.cpu.borrow().read_ram(addr)
    }

    /// Read directly from VRAM
    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        self.ppu.borrow().read_vram(addr)
    }
}

#[cfg(test)]
mod tests {

}
