//
// apu/chnl/timer.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date May 29 2020
//

use crate::common::Clockable;
use super::Divider;

/// Generic timer
#[derive(Default)]
pub struct Timer {
    divider: Divider,
    period: u16,
}

impl Clockable<bool> for Timer {
    fn tick(&mut self) -> bool {
        self.divider.tick()
    }
}

impl Timer {
    pub fn set_period_high(&mut self, hi: u8) {
        self.period = (self.period & 0x00FF) | ((hi as u16) << 8);
        self.divider.set_period(self.period as u32);
        self.divider.reset();
    }

    pub fn set_period_low(&mut self, lo: u8) {
        self.period = (self.period & 0xFF00) | lo as u16;
        self.divider.set_period(self.period as u32);
        self.divider.reset();
    }

    pub fn set_period(&mut self, period: u16) {
        self.period = period;
        self.divider.set_period(self.period as u32);
    }

    pub fn count(&self) -> u32 {
        self.divider.count()
    }

    pub fn reset(&mut self) {
        self.divider.reset();
    }

    pub fn period(&self) -> u16 {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_set() {
        let mut timer = Timer::default();
        timer.set_period_high(0x00);
        timer.set_period_low(0x05);
        timer.reset();

        assert_eq!(timer.tick(), false);
        assert_eq!(timer.tick(), false);
        assert_eq!(timer.tick(), false);
        assert_eq!(timer.tick(), false);
        assert_eq!(timer.tick(), false);
        assert_eq!(timer.tick(), true);
    }
}
