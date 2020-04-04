//
// apu/chnl/triangle.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 03 2020
//
use crate::common::{Clockable, IoAccess};
use super::{SoundChannel, LengthCounter};

#[derive(Default)]
pub struct Triangle {
    reload: usize,
    timer: u16,
    lenctr: LengthCounter,
}

impl SoundChannel for Triangle {
    fn clock_length(&mut self) {
        self.lenctr.tick();
    }

    fn enable_length(&mut self, e: bool) {
        self.lenctr.set_enable(e);
    }

    fn length_status(&self) -> bool {
        !self.lenctr.mute()
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
                self.lenctr.set_halt(ctrl);

                self.reload = (data & 0xEF) as usize;
            },
            1 => {
                // UNUSED
            },
            2 => {
                self.timer = (self.timer & 0xFF00) | (data as u16);
            },
            3 => {
                self.timer = (self.timer & 0x00FF) | (((data as u16) & 0x07) << 8);
                self.lenctr.load(bit_group!(data, 0x1F, 3) as usize);
            },
            _ => panic!("Invalid register for Triangle sound channel"),
        }
    }
}

#[cfg(test)]
mod tests {

}
