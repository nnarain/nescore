//
// apu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
pub mod bus;
mod chnl;
mod seq;

use seq::{FrameSequencer, Event};
use chnl::{SoundChannel, Pulse, Triangle, Noise, Dmc, LengthCounterUnit, EnvelopeUnit, NegateAddMode};

pub type Sample = f32;

pub const APU_OUTPUT_RATE: usize = 1_790_000;

use crate::common::{IoAccess, IoAccessRef, Clockable, Register};

#[cfg(feature="events")]
use std::sync::mpsc::Sender;

#[cfg(feature="events")]
pub mod events {
    #[derive(Debug)]
    pub struct ApuEvent {
        pub pulse1: f32,
        pub pulse2: f32,
        pub triangle: f32,
        pub noise: f32,
        pub dmc: f32,
        pub mixer: f32,
    }
}

/// NES APU
pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    sequencer: FrameSequencer,

    bus: Option<IoAccessRef>,

    // Event logging
    #[cfg(feature="events")]
    logger: Option<Sender<events::ApuEvent>>,
}

impl Default for Apu {
    fn default() -> Self {
        Apu {
            pulse1: Pulse::default(),
            pulse2: Pulse::default().add_mode(NegateAddMode::TwosComplement),
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc::default(),

            sequencer: FrameSequencer::default(),

            bus: None,

            #[cfg(feature="events")]
            logger: None,
        }
    }
}

impl Clockable<Sample> for Apu {
    fn tick(&mut self) -> Sample {
        // Clock the frame sequencer to generate low frequency clock events and process them
        for event in self.sequencer.tick().iter() {
            match event {
                Event::EnvelopAndLinear => {
                    self.clock_envelope();
                    self.triangle.clock_linear();
                },
                Event::LengthAndSweep => {
                    self.clock_length();
                    self.clock_sweep();
                },
                Event::Irq => {
                    // FIXME: APU IRQ
                    // if let Some(ref mut bus) = self.bus {
                    //     bus.raise_interrupt(Interrupt::Irq);
                    // }
                },
                Event::None => {}
            }
        }

        // Clock the pulse channels every APU cycle
        self.pulse1.tick();
        self.pulse2.tick();

        // The triangle channel is clocked at twice the rate of the APU
        self.triangle.tick();
        self.triangle.tick();

        // Clock noise channel
        self.noise.tick();

        // Clock DMC
        self.dmc.tick();

        self.mix()
    }
}

impl IoAccess for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x4000..=0x4003 => self.pulse1.read_byte(addr - 0x4000),
            0x4004..=0x4007 => self.pulse2.read_byte(addr - 0x4004),
            0x4008..=0x400B => self.triangle.read_byte(addr - 0x4008),
            0x400C..=0x400F => self.noise.read_byte(addr - 0x400C),
            0x4010..=0x4013 => self.dmc.read_byte(addr - 0x4010),
            0x4015 => self.status(),
            0x4017 => self.sequencer.value(),
            _ => panic!("Invalid address for APU: ${:04X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000..=0x4003 => self.pulse1.write_byte(addr - 0x4000, data),
            0x4004..=0x4007 => self.pulse2.write_byte(addr - 0x4004, data),
            0x4008..=0x400B => self.triangle.write_byte(addr - 0x4008, data),
            0x400C..=0x400F => self.noise.write_byte(addr - 0x400C, data),
            0x4010..=0x4013 => self.dmc.write_byte(addr - 0x4010, data),
            0x4015 => {
                self.pulse1.enable_length(bit_is_set!(data, 0));
                self.pulse2.enable_length(bit_is_set!(data, 1));
                self.triangle.enable_length(bit_is_set!(data, 2));
                self.noise.enable_length(bit_is_set!(data, 3));
                self.dmc.set_enable(bit_is_set!(data, 4));
            },
            0x4017 => {
                self.sequencer.load(data);

                if bit_is_set!(data, 7) {
                    // Immediately clock length units
                    self.pulse1.clock_length();
                    self.pulse2.clock_length();
                    self.triangle.clock_length();
                    self.noise.clock_length();
                }
            },
            _ => panic!("Invalid address for APU: ${:04X}", addr),
        }
    }
}

impl Apu {
    fn mix(&self) -> Sample {
        // TODO: There are other methods such as linear approximation and a look up table

        let pulse1_out = self.pulse1.output() as f32;
        let pulse2_out = self.pulse2.output() as f32;
        let triangle_out = self.triangle.output() as f32;
        let noise_out = self.noise.output() as f32;
        let dmc_out = self.dmc.output() as f32;

        let pulse_out = 95.88 / ((8128.0 / (pulse1_out + pulse2_out) + 100.0));

        let tnd_out = 159.79 / (1.0 / ((triangle_out / 8227.0) + (noise_out / 12241.0) + (dmc_out / 22638.0)) + 100.0);

        let mixed = pulse_out + tnd_out;

        #[cfg(feature="events")]
        {
            let data = events::ApuEvent {
                pulse1: pulse1_out,
                pulse2: pulse2_out,
                triangle: triangle_out,
                noise: noise_out,
                dmc: dmc_out,
                mixer: mixed,
            };

            if let Some(ref logger) = self.logger {
                match logger.send(data) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            }
        }

        mixed
    }

    fn status(&self) -> u8 {
        (self.pulse1.length_status() as u8)
        | (self.pulse2.length_status() as u8) << 1
        | (self.triangle.length_status() as u8) << 2
        | (self.noise.length_status() as u8) << 3
        | (self.dmc.status() as u8) << 4
    }

    fn clock_length(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
    }

    fn clock_envelope(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.noise.clock_envelope();
    }

    fn clock_sweep(&mut self) {
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    pub fn load_bus(&mut self, bus: IoAccessRef) {
        self.dmc.load_bus(bus.clone());
        self.bus = Some(bus);
    }

    #[cfg(feature="events")]
    pub fn set_event_sender(&mut self, sender: Sender<events::ApuEvent>) {
        self.logger = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn pulse_lenctr() {
        let mut apu = init_apu();

        // Mode Step4
        apu.write_byte(0x4017, 0x00);

        // Enable length counters
        apu.write_byte(0x4015, 0x03);
        // Set length counter to ten ticks
        apu.write_byte(0x4003, 0x00);
        apu.write_byte(0x4007, 0x00);

        // Check the status and ensure the length counters report active
        let status = apu.read_byte(0x4015);
        assert!(bit_is_set!(status, 0));
        assert!(bit_is_set!(status, 1));

        // The length counter clock twice per frame
        // Run for 4 frames
        for _ in 0..4 {
            run_for_step4_frame(&mut apu);
        }

        // Ensure the length counters are still active
        let status = apu.read_byte(0x4015);
        assert!(bit_is_set!(status, 0));
        assert!(bit_is_set!(status, 1));

        // Run for another frame
        // The length counter should be reported as inactive
        run_for_step4_frame(&mut apu);

        let status = apu.read_byte(0x4015);
        assert!(bit_is_clear!(status, 0));
        assert!(bit_is_clear!(status, 1));
    }

    fn run_for_step4_frame(apu: &mut dyn Clockable<Sample>) {
        for _ in 0..14915 {
            apu.tick();
        }
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

    fn init_apu() -> Apu {
        let mut apu: Apu = Apu::default();
        apu.load_bus(Rc::new(RefCell::new(FakeBus::default())));

        apu
    }
}
