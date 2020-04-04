//
// apu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
mod chnl;
mod seq;

use chnl::{SoundChannel, Pulse, Triangle};
use seq::{FrameSequencer, Event};

use crate::common::{IoAccess, Clockable, Register};

#[derive(Default)]
pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,

    sequencer: FrameSequencer,
}

impl Clockable for Apu {
    fn tick(&mut self) {
        for event in self.sequencer.tick().iter() {
            match event {
                Event::Envelop => {}
                Event::Length => {
                    self.pulse1.clock_length();
                    self.pulse2.clock_length();
                    self.triangle.clock_length();
                },
                Event::Irq => {},
                Event::None => {}
            }
        }
    }
}

impl IoAccess for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x4000..=0x4003 => self.pulse1.read_byte(addr - 0x4000),
            0x4004..=0x4007 => self.pulse2.read_byte(addr - 0x4004),
            0x4008..=0x400B => self.triangle.read_byte(addr - 0x4008),
            0x400C..=0x400F => 0, // TODO: Noise
            0x4010..=0x4013 => 0, // TODO: DMC
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
            0x400C..=0x400F => {}, // TODO: Noise
            0x4010..=0x4013 => {}, // TODO: DMC
            0x4015 => {
                self.pulse1.enable_length(bit_is_set!(data, 0));
                self.pulse2.enable_length(bit_is_set!(data, 1));
                self.triangle.enable_length(bit_is_set!(data, 2));
            },
            0x4017 => {
                self.sequencer.load(data);

                if data == 0x80 {
                    // Immediately clock length units
                    self.pulse1.clock_length();
                    self.pulse2.clock_length();
                    self.triangle.clock_length();
                }
            },
            _ => panic!("Invalid address for APU: ${:04X}", addr),
        }
    }
}

impl Apu {
    fn status(&self) -> u8 {
        (self.pulse1.length_status() as u8)
        | (self.pulse2.length_status() as u8) << 1
        | (self.triangle.length_status() as u8) << 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pulse_lenctr() {
        let mut apu = Apu::default();

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

    fn run_for_step4_frame(apu: &mut Apu) {
        for _ in 0..14915 {
            apu.tick();
        }
    }
}
