//
// cpu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 18 2019
//
pub mod bus;
mod state;
mod memorymap;

use crate::io::IoAccess;
use crate::clk::Clockable;

use std::num::Wrapping;

use state::{State, Instruction, AddressingMode};

const INTERNAL_RAM_SIZE: usize = 0x800;

/// NES Central Processing Unit
pub struct Cpu {
    a: u8,                        // General Purpose Accumulator
    x: u16,                       // Index register X
    y: u16,                       // Index register Y
    pc: Wrapping<u16>,            // Program Counter
    sp: u16,                      // Stack Pointer
    p: u8,                        // Flag register

    ram: [u8; INTERNAL_RAM_SIZE], // CPU RAM

    state: State,                 // Internal CPU cycle state

    address_bus: u16,             // Value for the address bus
    addressing_complete: bool,    // Indicate addressing is complete
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: Wrapping(0u16),
            sp: 0,
            p: 0,

            ram: [0; INTERNAL_RAM_SIZE],

            state: State::Reset,

            address_bus: 0,
            addressing_complete: false
        }
    }

    /// Execute the current cycle given the internal state
    fn run_cycle(&mut self, io: &mut dyn IoAccess, state: State) -> State {
        match state {
            State::Reset => {
                // Read the PC address from the RESET vector
                self.pc = Wrapping(self.read_u16(io, memorymap::RESET_VECTOR));
                State::Fetch
            },
            State::Fetch => {
                Cpu::get_execute_state(self.fetch(io))
            },
            State::Execute(ref instr, ref mode, ref total_cycles, ref cycle) => {

                // Apply instruction address
                if let Some(mode) = mode {
                    if !self.addressing_complete {
                        match mode {
                            _ => {}
                        }
                    }
                }

                // Once addressing is complete, run the instruction
                if self.addressing_complete {
                    let normalized_cycle = *cycle; // TODO: 
                    match instr {
                        Instruction::LDA => { self.lda(normalized_cycle); }
                    }
                }
                
                // Transition into the next state
                let next_cycle = cycle + 1;

                if *total_cycles != next_cycle {
                    // If not finished this opcode execution, return an execute state with next cycle
                    State::Execute(*instr, *mode, *total_cycles, next_cycle)
                }
                else {
                    // Finished opcode execute, enter fetch state
                    self.addressing_complete = false;
                    State::Fetch
                }
            },
        }
    }

    /// Load Accumulator
    fn lda(&mut self, cycle: u8) {
        
    }

    fn jmp(&mut self) -> State {
        State::Fetch
    }

    // Addressing Modes
    fn absolute(&mut self, cycle: u8) {

    }

    fn read_bus(&self) -> u8 {
        // self.read_u8(io: &mut dyn IoAccess, addr: u16)
        0
    }

    /// Convert opcode into instruction and addressing mode and return an execute state
    fn get_execute_state(opcode: u16) -> State {
        let (instr, mode, total_cycles) = match opcode {
            // LDA
            0xA9 => { (Instruction::LDA, Some(AddressingMode::Absolute), 3) },

            _ => {
                panic!("Invalid opcode");
            }
        };

        State::Execute(instr, mode, total_cycles, 0)
    }

    /// Fetch the next opcode and increment the program counter
    fn fetch(&mut self, io: &mut dyn IoAccess) -> u16 {
        let opcode = self.read_u16(io, self.pc.0);
        self.pc += Wrapping(1u16);

        opcode
    }

    fn read_u16(&self, io: &mut dyn IoAccess, addr: u16) -> u16 {
        let lo = self.read_u8(io, addr) as u16;
        let hi = self.read_u8(io, addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_u8(&self, io: &mut dyn IoAccess, addr: u16) -> u8 {
        // TODO: Read RAM
        io.read_byte(addr)
    }

    fn write_u8(&mut self, io: &mut dyn IoAccess, addr: u16, value: u8) {
        // TODO: Implement Write
        unimplemented!();
    }
}

impl Clockable for Cpu {
    /// Execute one CPU cycle
    fn tick(&mut self, io: &mut dyn IoAccess) {
        // Implement one cycle of the CPU using a state machince
        // Execute the cycle based on the current CPU state and return the next CPU state
        self.state = self.run_cycle
        (io, self.state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use helper::*;

    #[test]
    fn pc_after_reset() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![]);

        run_cpu(&mut cpu, &mut io, 0);

        assert_eq!(cpu.pc.0, 0x0000);
    }

    ///
    /// Helper functions
    ///
    mod helper {
        use super::*;

        pub struct CpuIoBus {
            prg_rom: Vec<u8> // ROM
        }

        impl CpuIoBus {
            pub fn from(prg_rom: Vec<u8>) -> Self {
                CpuIoBus {
                    prg_rom: prg_rom
                }
            }
        }

        impl IoAccess for CpuIoBus {
            fn read_byte(&self, addr: u16) -> u8 {
                if addr == 0xFFFC || addr == 0xFFFD {
                    0x00
                }
                else {
                    if (addr as usize) < self.prg_rom.len() {
                        self.prg_rom[addr as usize]
                    }
                    else {
                        panic!("Address out of supplied program ROM range");
                    }
                }
            }

            fn write_byte(&mut self, addr: u16, data: u8) {

            }
        }

        pub fn run_cpu(cpu: &mut Cpu, io: &mut dyn IoAccess, ticks: usize) {
            // Tick CPU once to exit Reset state
            cpu.tick(io);

            // Tick CPU the expect number of times
            for _ in 0..ticks {
                cpu.tick(io);
            }
        }
    }

}
