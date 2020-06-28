//
// apu/chnl/dmc.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jun 20 2020
//

use crate::common::{Clockable, IoAccess, IoAccessRef};
use super::{SoundChannel, Timer};

// Frequency lookup table in CPU cycles
const FREQ_LOOKUP: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

#[derive(Default)]
pub struct Dmc {
    irq_enabled: bool,
    loop_enabled: bool,
    sample_address: u16,
    sample_length: u16,
    enabled: bool,

    // Output unit
    bits_remaining: u8,
    silence: bool,
    shift: u8,

    output: u8,

    timer: Timer,

    sample_buffer: Option<u8>,
    current_addr: u16,
    remaining_bytes: u16,

    bus: Option<IoAccessRef>,
}

impl Clockable for Dmc {
    fn tick(&mut self) {
        if self.timer.tick() {
            if self.sample_buffer.is_none() && self.remaining_bytes > 0 {
                if let Some(ref bus) = self.bus {
                    self.sample_buffer = Some(bus.borrow().read_byte(self.current_addr));
                }

                // Advance sample address. Wrap around to $8000 if needed
                if self.current_addr == 0xFFFF {
                    self.current_addr = 0x8000;
                }
                else {
                    self.current_addr += 1;
                }

                if self.remaining_bytes > 0 {

                    self.remaining_bytes -= 1;

                    if self.remaining_bytes == 0 {
                        if self.loop_enabled {
                            self.start_cycle();
                        }
                        else {
                            if self.irq_enabled {
                                // TODO: Raise IRQ Interrupt
                            }
                        }
                    }
                }
            }

            // Output unit
            if !self.silence {
                if self.output >= 2 && self.output <= 125 {
                    if bit_is_set!(self.shift, 0) {
                        self.output += 2;
                    }
                    else {
                        self.output -= 2;
                    }
                }
            }

            self.shift >>= 1;

            if self.bits_remaining > 0 {
                self.bits_remaining -= 1;

                if self.bits_remaining == 0 {
                    self.start_cycle();
                }
            }
        }
    }
}

impl SoundChannel for Dmc {
    fn output(&self) -> u8 {
        self.output & 0x7F
    }
}

impl IoAccess for Dmc {
    #[allow(unused)]
    fn read_byte(&self, addr: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, reg: u16, data: u8) {
        match reg {
            0 => {
                self.irq_enabled = bit_is_set!(data, 7);
                self.loop_enabled = bit_is_set!(data, 6);
                self.timer.set_period(FREQ_LOOKUP[(data & 0x0F) as usize] / 2);
            },
            1 => self.output = data & 0xEF,
            2 => self.sample_address = 0xC000 | (data as u16) << 6,
            3 => self.sample_length = (data as u16) << 4 | 0x01,
            _ => panic!("Invalid register for DMC"),
        }
    }
}

impl Dmc {
    pub fn load_bus(&mut self, bus: IoAccessRef) {
        self.bus = Some(bus);
    }

    pub fn set_enable(&mut self, e: bool) {
        self.enabled = e;

        if self.enabled {
            self.start_cycle();
        }
    }

    fn start_cycle(&mut self) {
        self.current_addr = self.sample_address;
        self.remaining_bytes = self.sample_length;

        self.bits_remaining = 8;

        if self.sample_buffer.is_some() {
            self.shift = self.sample_buffer.unwrap();
            self.sample_buffer = None;
            self.silence = false;
        }
        else {
            self.silence = true;
        }
    }

    pub fn status(&self) -> bool {
        self.remaining_bytes > 0
    }
}
