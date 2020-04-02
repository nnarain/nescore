//
// apu/chnl/pulse.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 31 2020
//
use crate::common::{IoAccess, Clockable};

/// APU Pulse Channel
#[derive(Default)]
pub struct Pulse {
    duty: u8,
    halt: bool,
    constant: bool,
    volume: u8,
    sweep_enabled: bool,
    period: u8,
    negate: bool,
    shift: u8,
    timer: u16,
    load: u8,
}

impl IoAccess for Pulse {
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, reg: u16, data: u8) {
        match reg {
            0 => {
                self.duty = bit_group!(data, 0x03, 6);
                self.halt = bit_is_set!(data, 5);
                self.constant = bit_is_set!(data, 4);
                self.volume = data & 0x0F;
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
                self.load = bit_group!(data, 0x1F, 3);
            }
            _ => panic!("Invalid register for Pulse {}", reg),
        }
    }
}

impl Clockable for Pulse {
    fn tick(&mut self) {

    }
}
