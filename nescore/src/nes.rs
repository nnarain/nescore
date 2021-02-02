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

/// Buffer for audio data
pub type SampleBuffer = Vec<crate::apu::Sample>;


use std::{rc::Rc, vec};
use std::cell::RefCell;

#[cfg(feature="events")]
use std::sync::mpsc::{channel, Receiver};

#[derive(Clone, Copy)]
pub enum PixelFormat {
    RGB8,
    RGBA8,
    GBRA8,
    BGRA8,
}

impl PixelFormat {
    pub fn num_bytes(&self) -> usize {
        match *self {
            PixelFormat::RGB8 => 3,
            PixelFormat::RGBA8 => 4,
            PixelFormat::GBRA8 => 4,
            PixelFormat::BGRA8 => 4,
        }
    }
}

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
pub struct Nes {
    cpu: Rc<RefCell<Cpu<CpuIoBus>>>, // NES Central Processing Unit
    ppu: Rc<RefCell<Ppu<PpuIoBus>>>, // NES Picture Processing Unit
    apu: Rc<RefCell<Apu>>,           // NES Audio Processing Unit
    joy: Rc<RefCell<Joy>>,           // NES Joystick
    mapper: Option<Mapper>,          // Cartridge Mapper

    sequencer: FrameSequencer,       // Used to clock components in the right order

    framebuffer: Vec<u8>,
    pixel_format: PixelFormat,       // Pixel format
}

impl Default for Nes {
    fn default() -> Self {
        let pixel_format = PixelFormat::RGB8;
        let framebuffer = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT * pixel_format.num_bytes()];

        Nes {
            cpu: Rc::default(),
            ppu: Rc::default(),
            apu: Rc::default(),
            joy: Rc::default(),
            mapper: None,

            sequencer: FrameSequencer::default(),

            framebuffer,
            pixel_format,
        }
    }
}

impl Nes {
    /// Instantiate a NES emulator instance
    /// ```
    /// # use nescore::Nes;
    /// let nes = Nes::new();
    /// ```
    pub fn new() -> Self {
        Nes::default()
    }

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

    /// Set color output format
    pub fn pixel_format(mut self, pixel_format: PixelFormat) -> Self {
        self.pixel_format = pixel_format;
        self.framebuffer = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT * pixel_format.num_bytes()];

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
    /// # let cart = Cartridge::from_path("/path/to/rom").unwrap();
    /// let mut nes = Nes::from(cart);
    /// let (videobuffer, audiobuffer) = nes.emulate_frame();
    /// ```
    ///
    /// * `videobuffer` - A RGB8 frame buffer
    /// * `audiobuffer` - Raw APU output (This must be down sampled to host playback rate)
    pub fn emulate_frame(&mut self) -> (&[u8], SampleBuffer) {
        // let mut framebuffer = [0x00u8; FRAME_BUFFER_SIZE];
        let mut framebuffer_idx = 0usize;

        let mut samplebuffer: Vec<Sample> = Vec::new();

        if self.mapper.is_some() {
            for _ in 0..crate::ppu::CYCLES_PER_FRAME {
                // Clock the CPU, PPU and APU
                let (pixel, sample) = self.clock_components();

                if let Some(pixel) = pixel {
                    let bytes = self.format_color_output(pixel);
                    // TODO: This produces a clippy warning. However, `i` is not necessarily used to index the entirety
                    // of `bytes`. There's probably a better way to do this...
                    for i in 0..self.pixel_format.num_bytes() {
                        self.framebuffer[framebuffer_idx] = bytes[i];
                        framebuffer_idx = (framebuffer_idx + 1) % self.framebuffer.len();
                    }
                }

                if let Some(sample) = sample {
                    samplebuffer.push(sample);
                }
            }
        }

        (&self.framebuffer, samplebuffer)
    }

    /// Covert a PPU RGB pixel into different color formats
    /// return 4 bytes with the color data and a bool that indicates if the last byte is used
    fn format_color_output(&self, pixel: Pixel) -> [u8; 4] {
        match self.pixel_format {
            PixelFormat::RGB8 =>  [pixel.0, pixel.1, pixel.2, 0],
            PixelFormat::RGBA8 => [pixel.0, pixel.1, pixel.2, 255],
            PixelFormat::GBRA8 => [pixel.1, pixel.2, pixel.0, 255],
            PixelFormat::BGRA8 => [pixel.2, pixel.1, pixel.0, 255],
        }
    }

    /// Run the NES emulator until it fills an audio buffer to the specified size
    /// ```no_run
    /// # use nescore::Nes;
    /// # let mut nes = Nes::default();
    /// let samplebuffer = nes.run_audio(4096);
    /// ```
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
