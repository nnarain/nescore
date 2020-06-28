//
// apu/chnl/pulse.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
use crate::common::{IoAccess, Clockable};
use super::{SoundChannel, LengthCounter, LengthCounterUnit, Envelope, EnvelopeUnit, Divider, Timer};

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
        self.counter = (self.counter + 1) % self.waveform[0].len();
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

    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

#[derive(Clone, Copy)]
pub enum NegateAddMode {
    OnesComplement, TwosComplement,
}

/// APU Pulse Channel
pub struct Pulse {
    timer: Timer,

    // Sweep Unit
    sweep_divider: Divider,        // Divider used to apply sweep to channel period
    sweep_enabled: bool,           // Whether sweep is enabled
    sweep_negate: bool,            // Whether sweep is negated
    sweep_shift: u8,               // Value for the sweep unit barrel shifter
    sweep_reload: bool,            // Whether the sweep unit was written to since the last sweep clock
    sweep_add_mode: NegateAddMode, // Used to change the negate add between the two pulse channels

    lenctr: LengthCounter,         // Length counter unit
    envelope: Envelope,            // Envelope unit
    waveform: WaveformSequencer,   // Used to produce a square wave
}

impl Default for Pulse {
    fn default() -> Self {
        Pulse {
            timer: Timer::default(),

            sweep_divider: Divider::default(),
            sweep_enabled: false,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_reload: false,
            sweep_add_mode: NegateAddMode::OnesComplement,

            lenctr: LengthCounter::default(),
            envelope: Envelope::default(),
            waveform: WaveformSequencer::default(),
        }
    }
}

impl_length_counter!(Pulse, lenctr);
impl_envelope!(Pulse, envelope);

impl Clockable for Pulse {
    fn tick(&mut self) {
        if self.timer.tick() {
            // Clock the waveform sequencer
            self.waveform.tick();
        }
    }
}

impl Pulse {
    pub fn add_mode(mut self, mode: NegateAddMode) -> Self {
        self.sweep_add_mode = mode;
        self
    }

    pub fn clock_sweep(&mut self) {
        let event = self.sweep_divider.tick();

        if self.sweep_reload {
            self.sweep_divider.reset();
            self.sweep_reload = false;
        }

        if event {
            let channel_period = self.timer.period();

            if self.sweep_enabled && channel_period >= 8 {
                let target_period = util::sweep(channel_period,
                                                self.sweep_shift as u16,
                                                self.sweep_negate,
                                                self.sweep_add_mode);
                self.timer.set_period(target_period);
            }
        }
    }

    fn should_output(&self) -> bool {
        // The timer is not less than 8
        let period = self.timer.period();
        let gate0 = period >= 8 && period < 0x7FF;
        // At the high portion of the waveform
        let gate1 = self.waveform.output() != 0;
        // The length counter is not silencing the channel
        let gate2 = !self.lenctr.mute();

        gate0 && gate1 && gate2
    }
}

impl SoundChannel for Pulse {
    fn output(&self) -> u8 {
        if self.should_output() {
            self.envelope.output()
        }
        else {
            0
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
                self.waveform.set_duty(bit_group!(data, 0x03, 6) as usize);

                let l = bit_is_set!(data, 5);
                self.lenctr.set_halt(l);
                self.envelope.set_loop(l);

                self.envelope.set_constant(bit_is_set!(data, 4));
                self.envelope.set_volume(data & 0x0F);
            },
            1 => {
                self.sweep_divider.set_period(bit_group!(data, 0x07, 4) as u32);
                self.sweep_enabled = bit_is_set!(data, 7);
                self.sweep_negate = bit_is_set!(data, 3);
                self.sweep_shift = data & 0x07;
                self.sweep_reload = true;
            },
            2 => {
                self.timer.set_period_low(data);
            },
            3 => {
                self.timer.set_period_high(data & 0x07);

                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);
                self.envelope.start();
                self.waveform.reset();
            }
            _ => panic!("Invalid register for Pulse {}", reg),
        }
    }
}

mod util {
    use super::NegateAddMode;

    pub fn sweep(value: u16, shift: u16, negate: bool, add_mode: NegateAddMode) -> u16 {
        let change = value >> shift;

        let value = value as i16;

        let target = if negate {
            match add_mode {
                NegateAddMode::OnesComplement => value - (change as i16 + 1),
                NegateAddMode::TwosComplement => value - (change as i16),
            }
        }
        else {
            value + change as i16
        };

        target as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_negate_twos() {
        let target = util::sweep(0x03, 1, true, NegateAddMode::TwosComplement);
        assert_eq!(target, 0x02);
    }

    #[test]
    fn sweep_negate_ones() {
        let target = util::sweep(0x03, 1, true, NegateAddMode::OnesComplement);
        assert_eq!(target, 0x01);
    }

    #[test]
    fn sweep_no_negate() {
        let target_period = util::sweep(10, 0, false, NegateAddMode::OnesComplement);
        assert_eq!(target_period, 20);
    }

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

    #[test]
    fn waveform_reset_phase() {
        let mut waveform = WaveformSequencer::default().duty(0);
        waveform.tick();

        assert_eq!(waveform.output(), 1);

        waveform.reset();
        waveform.tick();

        assert_eq!(waveform.output(), 1);
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
