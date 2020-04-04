//
// apu/chnl/pulse.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
use crate::common::{IoAccess, Clockable};
// TODO: Is this layer breaking?
use super::{SoundChannel, LengthCounter};

// TODO: Pulse 1 vs Pulse 2, negate

/// APU Pulse Channel
#[derive(Default)]
pub struct Pulse {
    duty: u8,
    constant: bool,
    volume: u8,
    timer: u16,

    lenctr: LengthCounter,

    // Sweep
    sweep_enabled: bool,
    shift: u8,
    period: u8,
    negate: bool,
}

impl SoundChannel for Pulse {
    fn clock_length(&mut self) {
        self.lenctr.tick();
    }

    fn is_enabled(&self) -> bool {
        !self.lenctr.mute()
    }

    fn enable_length(&mut self, e: bool) {
        self.lenctr.set_enable(e);
    }

    fn length_status(&self) -> bool {
        !self.lenctr.mute()
    }
}

impl Clockable for Pulse {
    fn tick(&mut self) {

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
                self.duty = bit_group!(data, 0x03, 6);
                self.constant = bit_is_set!(data, 4);
                self.volume = data & 0x0F;

                self.lenctr.set_halt(bit_is_set!(data, 5))
            },
            1 => {
                self.sweep_enabled = bit_is_set!(data, 7);
                self.period = bit_group!(data, 0x07, 4);
                self.negate = bit_is_set!(data, 3);
                self.shift = data & 0x07;
            },
            2 => {
                self.timer = (self.timer & 0xFF00) | (data as u16);
            },
            3 => {
                self.timer = (self.timer & 0x00FF) | (((data as u16) & 0x07) << 8);

                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);
            }
            _ => panic!("Invalid register for Pulse {}", reg),
        }
    }
}

#[cfg(test)]
mod tests {

}
