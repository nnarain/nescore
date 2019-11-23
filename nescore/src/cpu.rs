//
// cpu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 18 2019
//
pub mod bus;

use crate::io::IoAccess;
use crate::clk::Clockable;

const INTERNAL_RAM_SIZE: usize = 0x800;

/// NES Central Processing Unit
pub struct Cpu {
    a: u8,                        // General Purpose Accumulator
    x: u16,                       // Index register X
    y: u16,                       // Index register Y
    pc: u16,                      // Program Counter
    sp: u16,                      // Stack Pointer
    p: u8,                        // Flag register

    ram: [u8; INTERNAL_RAM_SIZE], // CPU RAM
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            p: 0,

            ram: [0; INTERNAL_RAM_SIZE]
        }
    }
}

impl Clockable for Cpu {
    /// Return after one CPU cycle
    fn tick(&mut self, io: &mut dyn IoAccess) {
        // TODO: Implement opcodes
    }
}


#[cfg(test)]
mod tests {

}
