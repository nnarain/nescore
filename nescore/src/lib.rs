///
/// nescore/lib.rs
///
/// @author Natesh Narain <nnaraindev@gmail.com>
///

#[macro_use]
mod bit;
mod cpu;
mod ppu;
mod mapper;
mod common;

pub mod cart;
pub use cart::Cartridge;
pub use ppu::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

use cpu::Cpu;
use cpu::bus::CpuIoBus;
use ppu::Ppu;
use ppu::bus::PpuIoBus;
use mapper::Mapper;
use common::Clockable;

use std::rc::Rc;
use std::cell::RefCell;

/// Size of the display frame buffer: display size * RGB (3 bytes)
const FRAME_BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 3;
/// CPU Cycles in a frame: (256x240) - resolution, 1 px per PPU tick. 1 CPU tick for 3 PPU ticks
const CPU_CYCLES_PER_FRAME: usize = 113 * 262;//(DISPLAY_WIDTH * DISPLAY_HEIGHT) / 3;

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
    pub fn emulate_frame(&mut self) -> [u8; FRAME_BUFFER_SIZE] {
        let mut framebuffer = [0x00u8; FRAME_BUFFER_SIZE];
        let mut idx = 0usize;

        if self.mapper.is_some() {
            for _ in 0..CPU_CYCLES_PER_FRAME {
                let pixels = self.tick_master_clock();
                // TODO: Idiomatic way to do this
                for p in &pixels {
                    if let Some(p) = p {
                        let r = (p & 0xFF) as u8;
                        let g = ((p >> 8) & 0xFF) as u8;
                        let b = ((p >> 16) & 0xFF) as u8;

                        for v in &[r, g, b] {
                            if idx < FRAME_BUFFER_SIZE {
                                framebuffer[idx] = *v;
                                idx += 1;
                            }
                        }
                    }
                }
            }
        }

        framebuffer
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

    /// Clock the NES components
    fn tick_master_clock(&mut self) -> [Option<ppu::Pixel>; 3] {
        let mut pixels = [None; 3];

        self.cpu.borrow_mut().tick();

        pixels[0] = self.ppu.borrow_mut().tick();
        pixels[1] = self.ppu.borrow_mut().tick();
        pixels[2] = self.ppu.borrow_mut().tick();

        pixels
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
