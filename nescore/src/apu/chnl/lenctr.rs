//
// apu/chnl/lenctr.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 02 2020
//
use crate::common::Clockable;

pub trait LengthCounterUnit {
    fn enable_length(&mut self, e: bool);
    fn clock_length(&mut self);
    fn length_status(&self) -> bool;
}

#[macro_export]
macro_rules! impl_length_counter {
    ($t:ident, $f:ident) => {
        impl LengthCounterUnit for $t {
            fn enable_length(&mut self, e: bool) {
                self.$f.set_enable(e);
            }
            fn clock_length(&mut self) {
                self.$f.tick();
            }
            fn length_status(&self) -> bool {
                !self.$f.mute()
            }
        }
    };
}

const LOAD_LOOKUP_TABLE: [usize; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];


#[derive(Default)]
pub struct LengthCounter {
    enabled: bool,  // Length Counter is enabled
    halted: bool,   // Length Counter is halted
    counter: usize, // The actual counter
}

impl Clockable for LengthCounter {
    fn tick(&mut self) {
        if self.enabled && !self.halted {
            if self.counter > 0 {
                self.counter -= 1;
            }
        }
    }
}

impl LengthCounter {
    pub fn set_enable(&mut self, e: bool) {
        self.enabled = e;
        if !self.enabled {
            self.counter = 0;
        }
    }

    pub fn load(&mut self, len: usize) {
        if self.enabled {
            self.counter = LOAD_LOOKUP_TABLE[len];
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halted = halt;
    }

    pub fn mute(&self) -> bool {
        self.counter == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        let mut lenctr = LengthCounter::default();
        lenctr.set_enable(true);

        let load = 0;
        let count = LOAD_LOOKUP_TABLE[load] - 1;

        lenctr.load(load);

        for _ in 0..count {
            lenctr.tick();
        }

        assert!(!lenctr.mute());
        lenctr.tick();
        assert!(lenctr.mute());
    }

    #[test]
    fn halt() {
        let mut lenctr = LengthCounter::default();
        lenctr.set_enable(true);
        lenctr.load(0);

        for _ in 0..9 {
            lenctr.tick();
        }

        lenctr.set_halt(true);
        lenctr.tick();
        assert!(!lenctr.mute());

        lenctr.set_halt(false);
        lenctr.tick();
        assert!(lenctr.mute());
    }

    #[test]
    fn no_load_when_disabled() {
        let mut lenctr = LengthCounter::default();
        lenctr.set_enable(false);
        lenctr.load(0);

        assert!(lenctr.mute());
    }
}
