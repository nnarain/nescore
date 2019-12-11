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

// Flags
enum Flags {
    Carry            = 1 << 0,
    Zero             = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal          = 1 << 3,
    Overflow         = 1 << 6,
    Negative         = 1 << 7,
}

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
    pointer_address: u16,         // Pointer address for indirect addressing
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
            pointer_address: 0,
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
            State::Execute(ref instr, ref mode, ref cycle) => {
                // Indicate addressing is complete because it is unnecessary in Implied addressing mode
                if !self.addressing_complete {
                    self.addressing_complete = *mode == AddressingMode::Implied;
                }

                let execute_complete = if !self.addressing_complete {
                    // Apply addressing mode
                    match mode {
                        AddressingMode::Implied         => self.implied(), // TODO: Remove
                        AddressingMode::Immediate       => self.immediate(),
                        AddressingMode::ZeroPage        => self.zeropage(io),
                        AddressingMode::ZeroPageX       => self.zeropage_x(io, *cycle),
                        AddressingMode::ZeroPageY       => self.zeropage_y(io, *cycle),
                        AddressingMode::Absolute        => self.absolute(io, *cycle),
                        AddressingMode::AbsoluteX       => self.absolute_x(io, *cycle),
                        AddressingMode::AbsoluteY       => self.absolute_y(io, *cycle),
                        AddressingMode::IndexedIndirect => self.indexed_indirect(io, *cycle),
                        AddressingMode::IndirectIndexed => self.indirect_indexed(io, *cycle),
                        AddressingMode::Indirect        => self.indirect(io, *cycle),
                    }

                    false
                }
                else {
                    let normalized_cycle = *cycle; // TODO: 

                    match instr {
                        Instruction::NOP => self.nop(*cycle),
                        Instruction::LDA => self.lda(io),
                        Instruction::JMP => self.jmp(),
                    }
                };

                if !execute_complete {
                    // Transition into the next state
                    let next_cycle = cycle + 1;
                    // If not finished this opcode execution, return an execute state with next cycle
                    State::Execute(*instr, *mode, next_cycle)
                }
                else {
                    // Finished opcode execute, enter fetch state
                    self.addressing_complete = false;
                    State::Fetch
                }
            },
        }
    }

    //------------------------------------------------------------------------------------------------------------------
    // Opcode Decoding
    //------------------------------------------------------------------------------------------------------------------

    /// Convert opcode into instruction and addressing mode and return an execute state
    fn get_execute_state(opcode: u8) -> State {
        let (instr, mode) = match opcode {
            // NOP
            0xEA => (Instruction::NOP, AddressingMode::Implied),
            // LDA
            0xA9 => (Instruction::LDA, AddressingMode::Immediate),
            0xA5 => (Instruction::LDA, AddressingMode::ZeroPage),
            0xB5 => (Instruction::LDA, AddressingMode::ZeroPageX),
            0xAD => (Instruction::LDA, AddressingMode::Absolute),
            0xBD => (Instruction::LDA, AddressingMode::AbsoluteX),       // +1 cycle if page crossed
            0xB9 => (Instruction::LDA, AddressingMode::AbsoluteY),       // +1 cycle if page crossed
            0xA1 => (Instruction::LDA, AddressingMode::IndexedIndirect),
            0xB1 => (Instruction::LDA, AddressingMode::IndirectIndexed), // +1 cycles for page crossed
            // JMP
            0x4C => (Instruction::JMP, AddressingMode::Absolute), // TODO: Not cycle accurate!
            0x6C => (Instruction::JMP, AddressingMode::Indirect),

            _ => {
                panic!("Invalid opcode");
            }
        };

        State::Execute(instr, mode, 0)
    }

    //------------------------------------------------------------------------------------------------------------------
    // Instruction Implementation
    //------------------------------------------------------------------------------------------------------------------

    fn nop(&mut self, cycle: u8) -> bool {
        cycle == 1
    }

    /// Load Accumulator
    fn lda(&mut self, io: &mut dyn IoAccess) -> bool {
        self.a = self.read_bus(io);
        self.update_flags(self.a);
        true
    }

    /// Jump
    fn jmp(&mut self) -> bool {
        self.pc = Wrapping(self.address_bus);
        true
    }

    //------------------------------------------------------------------------------------------------------------------
    // Addressing Modes
    //------------------------------------------------------------------------------------------------------------------

    /// Some instruction have implied addressing
    fn implied(&mut self) {
        self.addressing_complete = true;
    }

    /// Immediate Addressing.
    /// Put current PC value on the address bus
    fn immediate(&mut self) {
        self.address_bus = self.pc.0;
        self.pc += Wrapping(1);

        self.addressing_complete = true;
    }

    /// Absolute Addressing.
    /// Fetch the address to read from the next two bytes
    fn absolute(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        match cycle {
            0 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xF0) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x0F) | ((self.read_next_u8(io) as u16) << 8);
                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for absolute addressing mode")
        }
    }

    /// Absolute Addressing Indexed by X
    fn absolute_x(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        self.absolute_i(io, cycle, self.x);
    }

    /// Absolute Addressing Indexed by Y
    fn absolute_y(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        self.absolute_i(io, cycle, self.y);
    }

    fn absolute_i(&mut self, io: &mut dyn IoAccess, cycle: u8, i: u16) {
        match cycle {
            0 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xF0) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x0F) | ((self.read_next_u8(io) as u16) << 8);
            },
            2 => {
                // Add the index value to the address bus
                self.address_bus += i;
                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for absolute addressing mode")
        }
    }

    /// Zero Page Addressing
    /// Fetch the next byte and put it on the address bus
    fn zeropage(&mut self, io: &mut dyn IoAccess) {
        self.address_bus = self.read_next_u8(io) as u16;
        self.addressing_complete = true;
    }

    /// Zero Page Index X Addressing.
    fn zeropage_x(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        self.zeropage_i(io, cycle, self.x);
    }

    /// Zero Page Index Y Addressing
    fn zeropage_y(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        self.zeropage_i(io, cycle, self.y);
    }

    fn zeropage_i(&mut self, io: &mut dyn IoAccess, cycle: u8, i: u16) {
        match cycle {
            0 => {
                self.address_bus = self.read_next_u8(io) as u16;
            },
            1 => {
                // TODO: Wrapping?
                self.address_bus += i;
                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for absolute addressing mode")
        }
    }

    /// Indexed Indirect Addressing
    fn indexed_indirect(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        match cycle {
            0 => {
                self.pointer_address = self.read_next_u8(io) as u16;
            },
            1 => {
                // TODO: Wrapping?
                self.pointer_address += self.x;
            },
            2 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xF0) | (self.read_u8(io, self.pointer_address) as u16);
            },
            3 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x0F) | ((self.read_u8(io, self.pointer_address + 1) as u16) << 8);
                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for absolute addressing mode")
        }
    }

    /// Indirect Indexed Addressing
    fn indirect_indexed(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        match cycle {
            0 => {
                self.pointer_address = self.read_next_u8(io) as u16;
            },
            1 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xF0) | (self.read_u8(io, self.pointer_address) as u16);
            },
            2 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x0F) | ((self.read_u8(io, self.pointer_address + 1) as u16) << 8);
            },
            3 => {
                self.address_bus += self.y;
                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for absolute addressing mode")
        }
    }

    /// Indirect
    /// Only applicable to JMP instruction
    fn indirect(&mut self, io: &mut dyn IoAccess, cycle: u8) {
        match cycle {
            0 => {
                // Fetch lower byte of address
                self.pointer_address = (self.pointer_address & 0xF0) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.pointer_address = (self.pointer_address & 0x0F) | ((self.read_next_u8(io) as u16) << 8);
            },
            2 => {
                let lo = self.read_u8(io, self.pointer_address) as u16;
                let hi = self.read_u8(io, self.pointer_address + 1) as u16;

                self.address_bus = (hi << 8) | lo;

                self.addressing_complete = true;
            }
            _ => panic!("Invalid cycle for indirect addressing"),
        }
    }

    //------------------------------------------------------------------------------------------------------------------
    // Flags Register
    //------------------------------------------------------------------------------------------------------------------

    fn update_flags(&mut self, a: u8) {
        if a == 0 {
            self.p |= Flags::Zero as u8;
        }
        if a & 0x80 != 0 {
            self.p |= Flags::Negative as u8;
        }
    }

    //------------------------------------------------------------------------------------------------------------------
    // Base CPU Read/Write Operations
    //------------------------------------------------------------------------------------------------------------------

    /// Fetch the next opcode and increment the program counter
    fn fetch(&mut self, io: &mut dyn IoAccess) -> u8 {
        self.read_next_u8(io)
    }

    fn read_bus(&self, io: &mut dyn IoAccess) -> u8 {
        self.read_u8(io, self.address_bus)
    }

    fn write_bus(&mut self, io: &mut dyn IoAccess, value: u8) {
        self.write_u8(io, self.address_bus, value);
    }

    fn read_next_u8(&mut self, io: &mut dyn IoAccess) -> u8 {
        let byte = self.read_u8(io, self.pc.0);
        self.pc += Wrapping(1u16);

        byte
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
        self.state = self.run_cycle(io, self.state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use helper::*;

    #[test]
    fn nop() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![0xEA]);

        run_cpu(&mut cpu, &mut io, 2);

        assert_eq!(cpu.pc.0, 0x0001);
    }

    #[test]
    fn pc_after_reset() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![]);

        run_cpu(&mut cpu, &mut io, 0);

        assert_eq!(cpu.pc.0, 0x0000);
    }

    #[test]
    fn lda_immediate() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0xA9, 0xA5 // LDA $A5
        ]);

        run_cpu(&mut cpu, &mut io, 3);

        assert_eq!(cpu.a, 0xA5);
    }

    #[test]
    fn lda_absolute() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0xAD, 0x03, 0x00, // LDA ($0003)
            0xDE,             // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_zeropage() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0xA5, 0x02, // LDA ($02)
            0xDE,       // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 3);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_zeropage_x() {
        let mut cpu = Cpu::new();
        cpu.x = 0x0001;

        let mut io = CpuIoBus::from(vec![
            0xB5, 0x02, // LDA $02, X
            0x00, 0xDE, // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_x() {
        let mut cpu = Cpu::new();
        cpu.x = 0x0001;

        let mut io = CpuIoBus::from(vec![
            0xB5, 0x03, 0x00, // LDA $0003, X
            0x00, 0xDE,       // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_y() {
        let mut cpu = Cpu::new();
        cpu.y = 0x0001;

        let mut io = CpuIoBus::from(vec![
            0xB9, 0x03, 0x00, // LDA $0003, Y
            0x00, 0xDE,       // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 5);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indexed_indirect() {
        let mut cpu = Cpu::new();
        cpu.x = 0x0001;

        let mut io = CpuIoBus::from(vec![
            0xA1, 0x02, // LDA ($0002, X)
            0x00,
            0x05, 0x00, // Address: $0004
            0xDE,       // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indirect_indexed() {
        let mut cpu = Cpu::new();
        cpu.y = 0x0001;

        let mut io = CpuIoBus::from(vec![
            0xB1, 0x02, // LDA ($0002), Y
            0x04, 0x00, // Address: $0004
            0x00, 0xDE, // Data: $DE
        ]);

        run_cpu(&mut cpu, &mut io, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_flags_zero() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0xA9, 0x00 // LDA $00
        ]);

        run_cpu(&mut cpu, &mut io, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.p, Flags::Zero as u8);
    }

    #[test]
    fn lda_flags_negative() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0xA9, 0x80 // LDA $00
        ]);

        run_cpu(&mut cpu, &mut io, 3);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.p, Flags::Negative as u8);
    }

    #[test]
    fn jmp_absolute() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0x4C, 0x00, 0x10 // LDA JMP $1000
        ]);

        run_cpu(&mut cpu, &mut io, 4); // TODO: JMP with absolute addressing should be 3 cycles

        assert_eq!(cpu.pc.0, 0x1000);
    }

    #[test]
    fn jmp_indirect() {
        let mut cpu = Cpu::new();
        let mut io = CpuIoBus::from(vec![
            0x6C, 0x03, 0x00, // LDA JMP ($0003)
            0x00, 0x10,       // Address: $1000
        ]);

        run_cpu(&mut cpu, &mut io, 5);

        assert_eq!(cpu.pc.0, 0x1000);
    }

    ///-----------------------------------------------------------------------------------------------------------------
    /// Helper functions
    ///-----------------------------------------------------------------------------------------------------------------
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
