//
// apu/chnl/div.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date May 29 2020
//

use crate::common::Clockable;

/// Outputs a clock periodically
pub struct Divider {
    counter: u32,
    period: u32,
}

impl Default for Divider {
    fn default() -> Self {
        Divider {
            counter: 0,
            period: 0,
        }
    }
}

impl Clockable<bool> for Divider {
    fn tick(&mut self) -> bool {
        let event = if self.counter == 0 {
            self.reset();
            true
        }
        else {
            self.counter -= 1;
            false
        };

        event
    }
}

impl Divider {
    pub fn reset(&mut self) {
        self.counter = self.period;
    }

    pub fn set_period(&mut self, period: u32) {
        self.period = period;
    }

    pub fn count(&self) -> u32 {
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn down_counter() {
        // Down counter's period is RELOAD + 1
        let mut divider = Divider::default();
        divider.set_period(3);
        divider.reset();

        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), true);
        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), false);
        assert_eq!(divider.tick(), true);
    }
}
