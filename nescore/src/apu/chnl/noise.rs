//
// apu/chnl/noise.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 03 2020
//
use crate::common::{Clockable, IoAccess};
use super::{SoundChannel, LengthCounter, LengthCounterUnit, Envelope, EnvelopeUnit, Timer};

const TIMER_PERIODS: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

pub struct Noise {
    timer: Timer,

    lenctr: LengthCounter,
    envelope: Envelope,

    loop_noise: bool,
    shift_register: u16,
}

impl Default for Noise {
    fn default() -> Self {
        Noise {
            timer: Timer::default(),

            lenctr: LengthCounter::default(),
            envelope: Envelope::default(),

            loop_noise: false,
            shift_register: 1,
        }
    }
}

impl_length_counter!(Noise, lenctr);
impl_envelope!(Noise, envelope);

impl SoundChannel for Noise {
    fn output(&self) -> u8 {
        if bit_is_set!(self.shift_register, 0) || self.lenctr.mute() {
            self.envelope.output()
        }
        else {
            0
        }
    }
}

impl Clockable for Noise {
    fn tick(&mut self) {
        if self.timer.tick() {
            let bit0 = bit_as_value!(self.shift_register, 0);
            let bit1 = if self.loop_noise { bit_as_value!(self.shift_register, 6) } else { bit_as_value!(self.shift_register, 1) };
            let r = (bit0 ^ bit1) as u16;

            self.shift_register >>= 1;

            bit_clear!(self.shift_register, 14);
            self.shift_register |= r << 14;
        }
    }
}

impl IoAccess for Noise {
    #[allow(unused)]
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, reg: u16, data: u8) {
        match reg {
            0 => {
                let l = bit_is_set!(data, 5);
                self.lenctr.set_halt(l);
                self.envelope.set_loop(l);
                self.envelope.set_constant(bit_is_set!(data, 4));
                self.envelope.set_volume(data & 0x0F);
            },
            1 => {}, // Unused
            2 => {
                self.loop_noise = bit_is_set!(data, 7);
                self.timer.set_period(TIMER_PERIODS[(data & 0x0F) as usize]);
            },
            3 => {
                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);
                self.envelope.start();
            },

            _ => panic!("invalid register for Noise channel"),
        }
    }
}
