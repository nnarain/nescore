//
// apu/chnl/pulse.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
use crate::common::{IoAccess, Clockable};
// TODO: Is this layer breaking?
use super::{SoundChannel, LengthCounter, LengthCounterUnit, Envelope, EnvelopeUnit};

struct WaveformSequencer {
    duty: usize,
    counter: usize,
    waveform: [[u8; 8]; 4],
}

impl Default for WaveformSequencer {
    fn default() -> Self {
        WaveformSequencer {
            duty: 0,
            counter: 0,
            waveform: [
                [0, 1, 0, 0, 0, 0, 0, 0],
                [0, 1, 1, 0, 0, 0, 0, 0],
                [0, 1, 1, 1, 1, 0, 0, 0],
                [1, 0, 0, 1, 1, 1, 1, 1]
            ],
        }
    }
}

impl Clockable for WaveformSequencer {
    fn tick(&mut self) {
        self.counter = (self.counter + 1) % 8;
    }
}

impl WaveformSequencer {
    pub fn set_duty(&mut self, duty: usize) {
        self.duty = duty;
    }

    #[cfg(test)]
    pub fn duty(mut self, duty: usize) -> Self {
        self.set_duty(duty);
        self
    }

    pub fn output(&self) -> u8 {
        self.waveform[self.duty][self.counter]
    }
}

// TODO: Pulse 1 vs Pulse 2, negate

/// APU Pulse Channel
#[derive(Default)]
pub struct Pulse {
    constant: bool,
    volume: u8,
    timer_load: u16,
    timer: u16,

    // Sweep
    sweep_enabled: bool,
    shift: u8,
    period: u8,
    negate: bool,

    lenctr: LengthCounter,
    envelope: Envelope,
    waveform: WaveformSequencer,
}

impl_length_counter!(Pulse, lenctr);
impl_envelope!(Pulse, envelope);

impl SoundChannel for Pulse {
    fn is_enabled(&self) -> bool {
        !self.lenctr.mute()
    }
}

impl Clockable for Pulse {
    fn tick(&mut self) {
        // TODO: Down counter?
        if self.timer > 0 {
            self.timer -= 1;

            if self.timer == 0 {
                self.waveform.tick();
            }
        }
    }
}

impl IoAccess for Pulse {
    #[allow(unused)]
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, reg: u16, data: u8) {
        match reg {
            0 => {
                self.constant = bit_is_set!(data, 4);
                self.volume = data & 0x0F;

                self.waveform.set_duty(bit_group!(data, 0x03, 6) as usize);
                self.lenctr.set_halt(bit_is_set!(data, 5))
            },
            1 => {
                self.sweep_enabled = bit_is_set!(data, 7);
                self.period = bit_group!(data, 0x07, 4);
                self.negate = bit_is_set!(data, 3);
                self.shift = data & 0x07;
            },
            2 => {
                self.timer_load = (self.timer_load & 0xFF00) | (data as u16);
                self.timer = self.timer_load;
            },
            3 => {
                self.timer_load = (self.timer_load & 0x00FF) | (((data as u16) & 0x07) << 8);

                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);
            }
            _ => panic!("Invalid register for Pulse {}", reg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveform_seq_duty() {
        let result: Vec<u8> = WaveformSequencer::default().duty(0).take(8).collect();
        assert_eq!(result, vec![0, 1, 0, 0, 0, 0, 0, 0]);

        let result: Vec<u8> = WaveformSequencer::default().duty(1).take(8).collect();
        assert_eq!(result, vec![0, 1, 1, 0, 0, 0, 0, 0]);

        let result: Vec<u8> = WaveformSequencer::default().duty(2).take(8).collect();
        assert_eq!(result, vec![0, 1, 1, 1, 1, 0, 0, 0]);

        let result: Vec<u8> = WaveformSequencer::default().duty(3).take(8).collect();
        assert_eq!(result, vec![1, 0, 0, 1, 1, 1, 1, 1]);
    }

    impl Iterator for WaveformSequencer {
        type Item = u8;

        fn next(&mut self) -> Option<Self::Item> {
            let output = self.output();
            self.tick();

            Some(output)
        }
    }
}
