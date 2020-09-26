//
// nes.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 17 2020
//
use crate::cart::Cartridge;
use crate::cpu::{Cpu, bus::CpuIoBus};
use crate::ppu::{Ppu, bus::PpuIoBus};
use crate::apu::{Apu, bus::ApuIoBus};
use crate::joy::Joy;
use crate::mapper::Mapper;
use crate::common::Clockable;

use crate::ppu::Pixel;
use crate::apu::Sample;
use crate::joy::{Controller, Button};

// use crate::utils::sampler::DownSampler;

use crate::ppu::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

/// Size of the display frame buffer: display size * RGB (3 bytes)
const FRAME_BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 3;

/// Buffer for video data
pub type FrameBuffer = [u8; FRAME_BUFFER_SIZE];
/// Buffer for audio data
pub type SampleBuffer = Vec<crate::apu::Sample>;


use std::rc::Rc;
use std::cell::RefCell;

#[cfg(feature="events")]
use std::sync::mpsc::{channel, Receiver};

/// Sequencer event
enum Event {
    CPU, PPU, APU, None,
}

type SequencerEvents = [Event; 3];

/// Component frame sequencer
#[derive(Default)]
struct FrameSequencer {
    counter: u32,
}

impl Clockable<SequencerEvents> for FrameSequencer {
    fn tick(&mut self) -> SequencerEvents {
        let events = match self.counter {
            0 => [Event::PPU, Event::CPU, Event::APU],
            1 => [Event::PPU, Event::None, Event::None],
            2 => [Event::PPU, Event::None, Event::None],
            3 => [Event::PPU, Event::CPU, Event::None],
            4 => [Event::PPU, Event::None, Event::None],
            5 => [Event::PPU, Event::None, Event::None],
            _ => panic!("Invalid clock for sequencer"),
        };

        self.counter = (self.counter + 1) % 6;

        events
    }
}

/// Representation of the NES system
#[derive(Default)]
pub struct Nes {
    cpu: Rc<RefCell<Cpu<CpuIoBus>>>, // NES Central Processing Unit
    ppu: Rc<RefCell<Ppu<PpuIoBus>>>, // NES Picture Processing Unit
    apu: Rc<RefCell<Apu>>,           // NES Audio Processing Unit
    joy: Rc<RefCell<Joy>>,           // NES Joystick
    mapper: Option<Mapper>,          // Cartridge Mapper

    sequencer: FrameSequencer,
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
    /// ```
    /// # use nescore::Nes;
    /// let nes = Nes::default().debug_mode(true);
    /// ```
    pub fn debug_mode(self, debug: bool) -> Self {
        self.cpu.borrow_mut().set_debug(debug);
        self
    }

    /// Run the emulator for a single frame
    /// ```no_run
    /// # use nescore::{Nes, Cartridge};
    /// let mut nes: Nes = Cartridge::from_path("/path/to/rom").unwrap().into();
    /// let (videobuffer, audiobuffer) = nes.emulate_frame();
    /// ```
    ///
    /// * `videobuffer` - A RGB8 frame buffer
    /// * `audiobuffer` - Raw APU output (This must be down sampled to host playback rate)
    pub fn emulate_frame(&mut self) -> (FrameBuffer, SampleBuffer) {
        let mut framebuffer = [0x00u8; FRAME_BUFFER_SIZE];
        let mut framebuffer_idx = 0usize;

        let mut samplebuffer: Vec<Sample> = Vec::new();

        if self.mapper.is_some() {
            for _ in 0..crate::ppu::CYCLES_PER_FRAME {
                // Clock the CPU, PPU and APU
                let (pixel, sample) = self.clock_components();

                if let Some((r, g, b)) = pixel {
                    // Insert RGB data into the frame buffer
                    framebuffer[framebuffer_idx] = r;
                    framebuffer[framebuffer_idx + 1] = g;
                    framebuffer[framebuffer_idx + 2] = b;
                    framebuffer_idx = (framebuffer_idx + 3) % FRAME_BUFFER_SIZE;
                }

                if let Some(sample) = sample {
                    samplebuffer.push(sample);
                }
            }
        }

        (framebuffer, samplebuffer)
    }

    pub fn run_audio(&mut self, buffer_size: usize) -> Vec<f32> {
        let mut buffer = vec![0f32; 0];

        while buffer.len() < buffer_size {
            let sample = self.clock_components().1;
            if let Some(sample) = sample {
                buffer.push(sample);
            }
        }

        buffer
    }

    /// Apply a button input into the emulator
    /// ```
    /// # use nescore::{Nes, Button};
    /// # let mut nes = Nes::default();
    /// nes.input(Button::A, true);
    /// ```
    pub fn input(&mut self, btn: Button, pressed: bool) {
        self.joy.borrow_mut().input(btn, pressed);
    }

    /// Apply a button input to the emulator from the specified controller
    /// ```
    /// # use nescore::{Nes, Button, Controller};
    /// # let mut nes = Nes::default();
    /// // Send an `A` button press to input 1
    /// nes.controller_input(Controller::Input1, Button::A, true);
    /// // Send an `A` button press to input 2
    /// nes.controller_input(Controller::Input2, Button::A, true);
    /// ```
    pub fn controller_input(&mut self, controller: Controller, btn: Button, pressed: bool) {
        self.joy.borrow_mut().controller_input(controller, btn, pressed);
    }

    /// Run until the CPU's PC is at address **addr**
    pub fn run_until(&mut self, addr: u16) {
        // TODO: Time limit
        if self.mapper.is_some() {
            while self.cpu.borrow().get_pc() != addr {
                self.clock_components();
            }
        }
    }

    /// Clock the NES components
    fn clock_components(&mut self) -> (Option<Pixel>, Option<Sample>) {
        let mut pixel: Option<Pixel> = None;
        let mut sample: Option<Sample> = None;

        for event in self.sequencer.tick().iter() {
            match event {
                Event::PPU => {
                    pixel = self.ppu.borrow_mut().tick();
                },
                Event::CPU => {
                    self.cpu.borrow_mut().tick();
                },
                Event::APU => {
                    sample = Some(self.apu.borrow_mut().tick());
                },
                Event::None => {},
            }
        }

        (pixel, sample)
    }

    /// Check if the CPU is in an infinite loop state
    pub fn is_holding(&self) -> bool {
        self.cpu.borrow().is_holding()
    }

    /// Load a cartridge
    pub fn insert(&mut self, cart: Cartridge) {
        // Consume provided cartridge and get the mapper
        let mapper = crate::mapper::from_cartridge(cart);

        // Complete initialization of components
        let cpu_bus = CpuIoBus::new(self.ppu.clone(), self.apu.clone(), self.joy.clone(), mapper.clone());
        self.cpu.borrow_mut().load_bus(cpu_bus);

        let ppu_bus = PpuIoBus::new(self.cpu.clone(), mapper.clone());
        self.ppu.borrow_mut().load_bus(ppu_bus);

        let apu_bus = Rc::new(RefCell::new(ApuIoBus::new(self.cpu.clone(), mapper.clone())));
        self.apu.borrow_mut().load_bus(apu_bus);

        self.mapper = Some(mapper);
    }

    /// Eject the cartridge, returning the save state
    /// ```
    /// # use nescore::Nes;
    /// let nes = Nes::default();
    /// // This consumes the nes instance
    /// let battery_ram = nes.eject();
    /// ```
    pub fn eject(self) -> Vec<u8> {
        self.mapper.map_or(vec![], |mapper| mapper.borrow().get_battery_ram())
    }

    //------------------------------------------------------------------------------------------------------------------
    // Event Logging
    //------------------------------------------------------------------------------------------------------------------
    #[cfg(feature="events")]
    pub fn cpu_event_channel(&mut self) -> Receiver<crate::events::CpuEvent> {
        let (tx, rx) = channel::<crate::events::CpuEvent>();
        self.cpu.borrow_mut().set_event_sender(tx);

        rx
    }

    #[cfg(feature="events")]
    pub fn apu_event_channel(&mut self) -> Receiver<crate::events::ApuEvent> {
        let (tx, rx) = channel::<crate::events::ApuEvent>();
        self.apu.borrow_mut().set_event_sender(tx);

        rx
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

    /// Read a tile from the current nametable
    pub fn read_tile(&self, nametable: u16, x: usize, y: usize) -> u8 {
        self.ppu.borrow().read_tile(nametable, x, y)
    }
}

impl From<Cartridge> for Nes {
    fn from(cart: Cartridge) -> Self {
        Nes::default().with_cart(cart)
    }
}

#[cfg(test)]
mod tests {

}
