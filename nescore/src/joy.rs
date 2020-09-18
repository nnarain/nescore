//
// joy.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 14 2020
//

use crate::common::IoAccess;

use std::cell::RefCell;

pub enum Button {
    A      = 0,
    B      = 1,
    Select = 2,
    Start  = 3,
    Up     = 4,
    Down   = 5,
    Left   = 6,
    Right  = 7,
}

pub enum Controller {
    Input1 = 0,
    Input2 = 1,
}

type ControllerState = RefCell<u8>;

/// NES Joystick Control
#[derive(Default)]
pub struct Joy {
    strobe: bool,                       // Flag that indicates controller input is allowed
    ctrls_states: [u8; 2], // States for both standard controllers
    ctrls_shifts: [ControllerState; 2], // Controller shift registers
}

impl Joy {
    pub fn input(&mut self, btn: Button, pressed: bool) {
        self.controller_input(Controller::Input1, btn, pressed)
    }

    pub fn controller_input(&mut self, ctrl: Controller, btn: Button, pressed: bool) {
        let btn = btn as u8;

        if pressed {
            bit_set!(self.ctrls_states[ctrl as usize], btn);
        }
        else {
            bit_clear!(self.ctrls_states[ctrl as usize], btn);
        };

        self.apply_strobe();
    }

    fn apply_strobe(&mut self) {
        if self.strobe {
            for (state, shift) in self.ctrls_states.iter().zip(self.ctrls_shifts.iter()) {
                *shift.borrow_mut() = *state;
            }
        }
    }
}

impl IoAccess for Joy {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x4016 | 0x4017 => {
                let mut controller = self.ctrls_shifts[(addr - 0x4016) as usize].borrow_mut();
                let current_state = *controller & 0x01;

                // Shift down by one bit
                *controller >>= 1;

                current_state
            },
            _ => panic!("Invalid address for read ${:04X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x4016 => {
                self.strobe = bit_is_set!(data, 0);
                self.apply_strobe();
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_input_1() {
        let mut joy = Joy::default();

        // Press button A
        joy.input(Button::A, true);
        // Enable strobe
        joy.write_byte(0x4016, 0x01);
        // Disable strobe
        joy.write_byte(0x4016, 0x00);

        // Get the state of button A
        let btn_a_state = joy.read_byte(0x4016);

        assert_eq!(btn_a_state, 0x01);
    }

    #[test]
    fn basic_input_2() {
        let mut joy = Joy::default();

        // Press button A
        joy.input(Button::A, true);
        // Press button B
        joy.input(Button::B, true);
        // Enable strobe
        joy.write_byte(0x4016, 0x01);
        // Disable strobe
        joy.write_byte(0x4016, 0x00);

        // Get button states
        let btn_a_state = joy.read_byte(0x4016);
        let btn_b_state = joy.read_byte(0x4016);

        assert_eq!(btn_a_state, 0x01);
        assert_eq!(btn_b_state, 0x01);
    }

    #[test]
    fn controller2_basic_input_1() {
        let mut joy = Joy::default();

        // Press button A
        joy.controller_input(Controller::Input2, Button::A, true);
        // Enable strobe
        joy.write_byte(0x4016, 0x01);
        // Disable strobe
        joy.write_byte(0x4016, 0x00);

        // Get button states
        let btn_a_state = joy.read_byte(0x4017);

        assert_eq!(btn_a_state, 0x01);
    }
}