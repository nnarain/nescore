//
// apu/seq.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 01 2020
//
use crate::common::{Clockable, Register};
use std::cell::RefCell;

pub enum Event {
    None,
    EnvelopAndLinear,
    LengthAndSweep,
    Irq,
}

type SequencerEvents = [Event; 3];

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Mode {
    Step4,
    Step5,
}

pub enum Step {
    One, Two, Three, Four, Five,
}

/// Provides low frequency clock events
/// https://wiki.nesdev.com/w/index.php/APU#Frame_Counter_.28.244017.29
pub struct FrameSequencer {
    cycles: usize,
    mode: Mode,
    irq_inhibit: bool,
    frame_irq: RefCell<bool>,
}

impl Default for FrameSequencer {
    fn default() -> Self {
        FrameSequencer {
            cycles: 0,
            mode: Mode::Step4,
            irq_inhibit: false,
            frame_irq: RefCell::new(false),
        }
    }
}

impl Register<u8> for FrameSequencer {
    fn load(&mut self, data: u8) {
        self.mode = if bit_is_set!(data, 7) { Mode::Step5 } else { Mode::Step4 };
        self.irq_inhibit = bit_is_set!(data, 6);

        if self.irq_inhibit {
            *self.frame_irq.borrow_mut() = false;
        }

        self.cycles = 0;
    }

    fn value(&self) -> u8 {
        let mode = if self.mode == Mode::Step5 { 1 } else { 0 };
        (mode << 7)
        | ((self.irq_inhibit as u8) << 6)
    }
}

impl Clockable<SequencerEvents> for FrameSequencer {
    fn tick(&mut self) -> SequencerEvents {
        // Covert the tracked cycles to the APU step
        // Map the step to the correct set of clock events
        // Step 4 is the only thing different between step 4 and 5 mode (expect the extra step in 5 mode)
        let events = helpers::cycles_to_step(self.cycles).map(|step| {
            match step {
                Step::One => [Event::EnvelopAndLinear, Event::None, Event::None],
                Step::Two => [Event::EnvelopAndLinear, Event::LengthAndSweep, Event::None],
                Step::Three => [Event::EnvelopAndLinear, Event::None, Event::None],
                Step::Four => {
                    // Set frame IRQ if in mode 4 and the irq is not inhibited
                    if self.mode == Mode::Step4 && !self.irq_inhibit {
                        *self.frame_irq.borrow_mut() = true;
                    }
                    helpers::step4(self.mode, self.irq_inhibit)
                },
                Step::Five => [Event::EnvelopAndLinear, Event::LengthAndSweep, Event::None],
            }
        })
        .unwrap_or([Event::None, Event::None, Event::None]);

        // Advance cycles given the mode
        self.cycles = match self.mode {
            Mode::Step4 => (self.cycles + 1) % 14915,
            Mode::Step5 => (self.cycles + 1) % 18641,
        };

        events
    }
}

impl FrameSequencer {
    pub fn irq_status(&self) -> bool {
        let status = *self.frame_irq.borrow();
        *self.frame_irq.borrow_mut() = false;

        status
    }
}

mod helpers {
    use super::{Step, Mode, Event, SequencerEvents};
    pub fn cycles_to_step(cycles: usize) -> Option<Step> {
        match cycles {
            3728 => Some(Step::One),
            7456 => Some(Step::Two),
            11185 => Some(Step::Three),
            14914 => Some(Step::Four),
            18640 => Some(Step::Five),
            _ => None,
        }
    }

    pub fn step4(mode: Mode, irq_inhibit: bool) -> SequencerEvents {
        match mode {
            Mode::Step4 => [Event::EnvelopAndLinear, Event::LengthAndSweep, if !irq_inhibit {Event::Irq} else {Event::None}],
            Mode::Step5 => [Event::None, Event::None, Event::None],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clocks_for_step4_mode() {
        let mut frame_sequencer = FrameSequencer::default();
        let mut envelope_counter = 0;
        let mut length_counter = 0;
        let mut irq_counter = 0;

        // Set Step 4 mode
        frame_sequencer.load(0x00);

        // Iterate for number of APU cycles in a frame
        for _ in 0..14915 {
            // Tick the frame sequencer and iterate over the events
            for event in frame_sequencer.tick().iter() {
                // Tick each counter for the corresponding event
                match event {
                    Event::EnvelopAndLinear => envelope_counter += 1,
                    Event::LengthAndSweep => length_counter += 1,
                    Event::Irq => irq_counter += 1,
                    Event::None => {}
                }
            }
        }

        assert_eq!(envelope_counter, 4);
        assert_eq!(length_counter, 2);
        assert_eq!(irq_counter, 1);
        // Check IRQ status
        assert_eq!(frame_sequencer.irq_status(), true);
        // Should automatically clear
        assert_eq!(frame_sequencer.irq_status(), false);
    }

    #[test]
    fn clocks_for_step5_mode() {
        let mut frame_sequencer = FrameSequencer::default();
        let mut envelope_counter = 0;
        let mut length_counter = 0;
        let mut irq_counter = 0;

        // Set Step 4 mode
        frame_sequencer.load(0x80);

        // Iterate for number of APU cycles in a frame
        for _ in 0..18641 {
            // Tick the frame sequencer and iterate over the events
            for event in frame_sequencer.tick().iter() {
                // Tick each counter for the corresponding event
                match event {
                    Event::EnvelopAndLinear => envelope_counter += 1,
                    Event::LengthAndSweep => length_counter += 1,
                    Event::Irq => irq_counter += 1,
                    Event::None => {}
                }
            }
        }

        assert_eq!(envelope_counter, 4);
        assert_eq!(length_counter, 2);
        assert_eq!(irq_counter, 0);
    }

    #[test]
    fn set_mode() {
        let mut frame_sequencer = FrameSequencer::default();

        frame_sequencer.load(0x80);
        assert_eq!(frame_sequencer.mode, Mode::Step5);
        assert_eq!(frame_sequencer.value(), 0x80);

        frame_sequencer.load(0x00);
        assert_eq!(frame_sequencer.mode, Mode::Step4);
        assert_eq!(frame_sequencer.value(), 0x00);
    }
}
