//
// apu/chnl/triangle.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 03 2020
//
use crate::common::{Clockable, IoAccess};
use super::{SoundChannel, LengthCounter, LengthCounterUnit, Timer};

pub struct Triangle {
    timer: Timer,
    lenctr: LengthCounter,

    linear_counter: usize,
    reload_value: usize,
    reload_flag: bool,
    ctrl_flag: bool,

    sequence: [u8; 32],
    sequence_idx: usize,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            timer: Timer::default(),
            lenctr: LengthCounter::default(),

            linear_counter: 0,
            reload_value: 0,
            reload_flag: false,
            ctrl_flag: false,

            sequence: [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
                        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            sequence_idx: 0,
        }
    }
}

impl_length_counter!(Triangle, lenctr);

impl SoundChannel for Triangle {
    fn output(&self) -> u8 {
        self.sequence[self.sequence_idx]
    }
}

impl Clockable for Triangle {
    fn tick(&mut self) {
        if self.timer.tick() && !self.lenctr.mute() && self.linear_counter != 0 {
            self.sequence_idx = (self.sequence_idx + 1) % self.sequence.len();
        }
    }
}

impl Triangle {
    pub fn clock_linear(&mut self) {
        if self.reload_flag {
            self.linear_counter = self.reload_value;
        }
        else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.ctrl_flag {
            self.reload_flag = false;
        }
    }
}

impl IoAccess for Triangle {
    #[allow(unused)]
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0 => {
                let ctrl = bit_is_set!(data, 7);
                self.ctrl_flag = ctrl;
                self.lenctr.set_halt(ctrl);

                self.reload_value = (data & 0xEF) as usize;
            },
            1 => {
                // UNUSED
            },
            2 => {
                self.timer.set_period_low(data);
            },
            3 => {
                self.timer.set_period_high(data & 0x07);
                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);

                self.reload_flag = true;
            },
            _ => panic!("Invalid register for Triangle sound channel"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_triangle() {
        let mut triangle = Triangle::default();
        // Linear counter = 1
        triangle.write_byte(0, 0x01);
        // Timer = 1
        triangle.write_byte(2, 1);
        // Length counter
        triangle.enable_length(true);
        triangle.write_byte(3, 0x08);

        triangle.clock_linear();

        for i in (0..16).rev() {
            assert_eq!(triangle.output(), i);
            clock_sequencer(&mut triangle, 2);
        }

        for i in 0..16 {
            assert_eq!(triangle.output(), i);
            clock_sequencer(&mut triangle, 2);
        }
    }

    fn clock_sequencer(triangle: &mut Triangle, period: u32) {
        for _ in 0..period {
            triangle.tick();
        }
    }

}
