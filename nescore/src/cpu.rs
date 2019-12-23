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
    x: u8,                       // Index register X
    y: u8,                       // Index register Y
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
                // Indicate addressing is complete because it is unnecessary in Implied and Accumulator addressing modes
                if !self.addressing_complete {
                    self.addressing_complete = match *mode {
                        AddressingMode::Implied | AddressingMode::Accumulator => true,
                        _ => false,
                    };
                }

                let execute_complete = if !self.addressing_complete {
                    // Apply addressing mode
                    match mode {
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
                        AddressingMode::Relative        => self.relative(io),

                        _ => panic!("Addressing mode not handled!!")
                    }

                    false
                }
                else {
                    match instr {
                        Instruction::NOP => self.nop(*cycle),
                        Instruction::LDA => self.lda(io),
                        Instruction::JMP => self.jmp(),
                        Instruction::ADC => self.adc(io),
                        Instruction::AND => self.and(io),
                        Instruction::ASL => self.asl(io),
                        Instruction::STA => self.sta(io),
                        Instruction::BCC => self.bcc(),
                        Instruction::BCS => self.bcs(),
                        Instruction::BEQ => self.beq(),
                        Instruction::BNE => self.bne(),
                        Instruction::BMI => self.bmi(),
                        Instruction::BPL => self.bpl(),
                        Instruction::BIT => self.bit(io),
                        Instruction::BVC => self.bvc(),
                        Instruction::BVS => self.bvs(),
                        Instruction::CLC => self.clc(),
                        Instruction::CLD => self.cld(),
                        Instruction::CLI => self.cli(),
                        Instruction::CLV => self.clv(),
                        Instruction::CMP => self.cmp(io),
                        Instruction::CPX => self.cpx(io),
                        Instruction::CPY => self.cpy(io),
                        Instruction::DEC => self.dec(io, *cycle),
                        Instruction::DEX => self.dex(),
                        Instruction::DEY => self.dey(),
                        Instruction::INC => self.inc(io),
                        Instruction::INX => self.inx(),
                        Instruction::INY => self.iny(),
                        Instruction::EOR => self.eor(io),
                        Instruction::LDX => self.ldx(io),
                        Instruction::LDY => self.ldy(io),
                        Instruction::PHA => self.pha(io, *cycle),
                        Instruction::PHP => self.php(io, *cycle),
                        Instruction::PLA => self.pla(io, *cycle),
                        Instruction::PLP => self.plp(io, *cycle),
                        Instruction::LSR => self.lsr(io),
                        Instruction::ORA => self.ora(io),

                        _ => panic!("incomplete")
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
            0xBD => (Instruction::LDA, AddressingMode::AbsoluteX),
            0xB9 => (Instruction::LDA, AddressingMode::AbsoluteY),
            0xA1 => (Instruction::LDA, AddressingMode::IndexedIndirect),
            0xB1 => (Instruction::LDA, AddressingMode::IndirectIndexed),
            // JMP
            0x4C => (Instruction::JMP, AddressingMode::Absolute),
            0x6C => (Instruction::JMP, AddressingMode::Indirect),
            // ADC
            0x69 => (Instruction::ADC, AddressingMode::Immediate),
            0x65 => (Instruction::ADC, AddressingMode::ZeroPage),
            0x75 => (Instruction::ADC, AddressingMode::ZeroPageX),
            0x6D => (Instruction::ADC, AddressingMode::Absolute),
            0x7D => (Instruction::ADC, AddressingMode::AbsoluteX),
            0x79 => (Instruction::ADC, AddressingMode::AbsoluteY),
            0x61 => (Instruction::ADC, AddressingMode::IndexedIndirect),
            0x71 => (Instruction::ADC, AddressingMode::IndirectIndexed),
            // AND
            0x29 => (Instruction::AND, AddressingMode::Immediate),
            0x25 => (Instruction::AND, AddressingMode::ZeroPage),
            0x35 => (Instruction::AND, AddressingMode::ZeroPageX),
            0x2D => (Instruction::AND, AddressingMode::Absolute),
            0x3D => (Instruction::AND, AddressingMode::AbsoluteX),
            0x39 => (Instruction::AND, AddressingMode::AbsoluteY),
            0x21 => (Instruction::AND, AddressingMode::IndexedIndirect),
            0x31 => (Instruction::AND, AddressingMode::IndirectIndexed),
            // ASL
            0x0A => (Instruction::ASL, AddressingMode::Accumulator),
            0x06 => (Instruction::ASL, AddressingMode::ZeroPage),
            0x16 => (Instruction::ASL, AddressingMode::ZeroPageX),
            0x0E => (Instruction::ASL, AddressingMode::Absolute),
            0x1E => (Instruction::ASL, AddressingMode::AbsoluteX),
            // STA
            0x85 => (Instruction::STA, AddressingMode::ZeroPage),
            0x95 => (Instruction::STA, AddressingMode::ZeroPageX),
            0x8D => (Instruction::STA, AddressingMode::Absolute),
            0x9D => (Instruction::STA, AddressingMode::AbsoluteX),
            0x99 => (Instruction::STA, AddressingMode::AbsoluteY),
            0x81 => (Instruction::STA, AddressingMode::IndexedIndirect),
            0x91 => (Instruction::STA, AddressingMode::IndirectIndexed),
            // BCC
            0x90 => (Instruction::BCC, AddressingMode::Relative),
            // BCS
            0xB0 => (Instruction::BCS, AddressingMode::Relative),
            // BEQ
            0xF0 => (Instruction::BEQ, AddressingMode::Relative),
            // BNE
            0xD0 => (Instruction::BNE, AddressingMode::Relative),
            // BMI
            0x30 => (Instruction::BMI, AddressingMode::Relative),
            // BPL
            0x10 => (Instruction::BPL, AddressingMode::Relative),
            // BIT
            0x24 => (Instruction::BIT, AddressingMode::ZeroPage),
            0x2C => (Instruction::BIT, AddressingMode::Absolute),
            // BVC
            0x50 => (Instruction::BVC, AddressingMode::Relative),
            // BVS
            0x70 => (Instruction::BVS, AddressingMode::Relative),
            // CLC
            0x18 => (Instruction::CLC, AddressingMode::Implied),
            // CLD
            0xD8 => (Instruction::CLD, AddressingMode::Implied),
            // CLI
            0x58 => (Instruction::CLI, AddressingMode::Implied),
            // CLV
            0xB8 => (Instruction::CLV, AddressingMode::Implied),
            // CMP
            0xC9 => (Instruction::CMP, AddressingMode::Immediate),
            0xC5 => (Instruction::CMP, AddressingMode::ZeroPage),
            0xD5 => (Instruction::CMP, AddressingMode::ZeroPageX),
            0xCD => (Instruction::CMP, AddressingMode::Absolute),
            0xDD => (Instruction::CMP, AddressingMode::AbsoluteX),
            0xD9 => (Instruction::CMP, AddressingMode::AbsoluteY),
            0xC1 => (Instruction::CMP, AddressingMode::IndexedIndirect),
            0xD1 => (Instruction::CMP, AddressingMode::IndirectIndexed),
            // CPX
            0xE0 => (Instruction::CPX, AddressingMode::Immediate),
            0xE4 => (Instruction::CPX, AddressingMode::ZeroPage),
            0xEC => (Instruction::CPX, AddressingMode::Absolute),
            // CPY
            0xC0 => (Instruction::CPX, AddressingMode::Immediate),
            0xC4 => (Instruction::CPY, AddressingMode::ZeroPage),
            0xCC => (Instruction::CPY, AddressingMode::Absolute),
            // DEC
            0xC6 => (Instruction::DEC, AddressingMode::ZeroPage),
            0xD6 => (Instruction::DEC, AddressingMode::ZeroPageX),
            0xCE => (Instruction::DEC, AddressingMode::Absolute),
            0xDE => (Instruction::DEC, AddressingMode::AbsoluteX),
            // DEX
            0xCa => (Instruction::DEX, AddressingMode::Implied),
            // DEY
            0x88 => (Instruction::DEY, AddressingMode::Implied),
            // INC
            0xE6 => (Instruction::INC, AddressingMode::ZeroPage),
            0xF6 => (Instruction::INC, AddressingMode::ZeroPageX),
            0xEE => (Instruction::INC, AddressingMode::Absolute),
            0xFE => (Instruction::INC, AddressingMode::AbsoluteX),
            // INX
            0xE8 => (Instruction::INX, AddressingMode::Implied),
            // INY
            0xC8 => (Instruction::INY, AddressingMode::Implied),
            // EOR
            0x49 => (Instruction::EOR, AddressingMode::Immediate),
            0x45 => (Instruction::EOR, AddressingMode::ZeroPage),
            0x55 => (Instruction::EOR, AddressingMode::ZeroPageX),
            0x4D => (Instruction::EOR, AddressingMode::Absolute),
            0x5D => (Instruction::EOR, AddressingMode::AbsoluteX),
            0x59 => (Instruction::EOR, AddressingMode::AbsoluteY),
            0x41 => (Instruction::EOR, AddressingMode::IndexedIndirect),
            0x51 => (Instruction::EOR, AddressingMode::IndirectIndexed),
            // LDX
            0xA2 => (Instruction::LDX, AddressingMode::Immediate),
            0xA6 => (Instruction::LDX, AddressingMode::ZeroPage),
            0xB6 => (Instruction::LDX, AddressingMode::ZeroPageY),
            0xAE => (Instruction::LDX, AddressingMode::Absolute),
            0xBE => (Instruction::LDX, AddressingMode::AbsoluteY),
            // LDY
            0xA0 => (Instruction::LDY, AddressingMode::Immediate),
            0xA4 => (Instruction::LDY, AddressingMode::ZeroPage),
            0xB4 => (Instruction::LDY, AddressingMode::ZeroPageX),
            0xAC => (Instruction::LDY, AddressingMode::Absolute),
            0xBC => (Instruction::LDY, AddressingMode::AbsoluteX),
            // PHA
            0x48 => (Instruction::PHA, AddressingMode::Implied),
            // PHP
            0x08 => (Instruction::PHP, AddressingMode::Implied),
            // PLA
            0x68 => (Instruction::PLA, AddressingMode::Implied),
            // PLP
            0x28 => (Instruction::PLP, AddressingMode::Implied),
            // LSR
            0x4A => (Instruction::LSR, AddressingMode::Accumulator),
            0x46 => (Instruction::LSR, AddressingMode::ZeroPage),
            0x56 => (Instruction::LSR, AddressingMode::ZeroPageX),
            0x4E => (Instruction::LSR, AddressingMode::Absolute),
            0x5E => (Instruction::LSR, AddressingMode::AbsoluteX),
            // ORA
            0x09 => (Instruction::ORA, AddressingMode::Immediate),
            0x05 => (Instruction::ORA, AddressingMode::ZeroPage),
            0x15 => (Instruction::ORA, AddressingMode::ZeroPageX),
            0x0D => (Instruction::ORA, AddressingMode::Absolute),
            0x1D => (Instruction::ORA, AddressingMode::AbsoluteX),
            0x19 => (Instruction::ORA, AddressingMode::AbsoluteY),
            0x01 => (Instruction::ORA, AddressingMode::IndexedIndirect),
            0x11 => (Instruction::ORA, AddressingMode::IndirectIndexed),

            _ => {
                panic!("Invalid opcode: {opcode}", opcode=opcode);
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

    /// ADC - Add with Carry
    fn adc(&mut self, io: &mut dyn IoAccess) -> bool {
        // A,Z,C,N = A+M+C
        let a = self.a as u16;
        let m = self.read_bus(io) as u16;
        let c = self.get_carry() as u16;

        let r = a + m + c;
        let is_carry = r > 0xFF;
        self.a = (r & 0x0FF) as u8;

        self.update_flags_with_carry(self.a, is_carry);

        true
    }

    /// AND - Logical AND
    fn and(&mut self, io: &mut dyn IoAccess) -> bool {
        // A,Z,N = A&M
        let a = self.a;
        let m = self.read_bus(io);

        self.a = a & m;

        self.update_flags(self.a);

        true
    }

    /// ASL - Arithmetic shift left
    fn asl(&mut self, io: &mut dyn IoAccess) -> bool {
        let m = self.read_bus(io);
        let c = bit_is_set!(m, 7);

        let r = m << 1;

        self.write_bus(io, r);

        self.set_zero_flag(self.a);
        self.set_negative_flag(r);
        self.set_flag_bit(Flags::Carry, c);

        true
    }

    fn sta(&mut self, io: &mut dyn IoAccess) -> bool {
        self.write_bus(io, self.a);
        true
    }

    fn bcc(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Carry) == false);
        true
    }

    fn bcs(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Carry));
        true
    }

    fn beq(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Zero));
        true
    }

    fn bne(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Zero) == false);
        true
    }

    fn bmi(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Negative));
        true
    }

    /// BPL - Branch if Positive
    fn bpl(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Negative) == false);
        true
    }

    fn bvc(&mut self) -> bool {
        self.branch(!self.get_flag_bit(Flags::Overflow));
        true
    }

    fn bvs(&mut self) -> bool {
        self.branch(self.get_flag_bit(Flags::Overflow));
        true
    }

    /// BIT - Bit Test
    fn bit(&mut self, io: &mut dyn IoAccess) -> bool {
        let m = self.read_bus(io);
        let r = self.a & m;

        // Copy bit 6 to V flag, and bit 7 to N flag
        self.set_flag_bit(Flags::Overflow, bit_is_set!(r, 6));
        self.set_flag_bit(Flags::Negative, bit_is_set!(r, 7));

        self.set_flag_bit(Flags::Zero, r == 0);

        true
    }

    fn clc(&mut self) -> bool {
        self.set_flag_bit(Flags::Carry, false);
        true
    }

    fn cld(&mut self) -> bool {
        self.set_flag_bit(Flags::Decimal, false);
        true
    }

    fn cli(&mut self) -> bool {
        self.set_flag_bit(Flags::InterruptDisable, false);
        true
    }

    fn clv(&mut self) -> bool {
        self.set_flag_bit(Flags::Overflow, false);
        true
    }

    fn cmp(&mut self, io: &mut dyn IoAccess) -> bool {
        self.compare(self.a, self.read_bus(io));
        true
    }

    fn cpx(&mut self, io: &mut dyn IoAccess) -> bool {
        self.compare(self.x, self.read_bus(io));
        true
    }

    fn cpy(&mut self, io: &mut dyn IoAccess) -> bool {
        self.compare(self.y, self.read_bus(io));
        true
    }

    fn dec(&mut self, io: &mut dyn IoAccess, _cycle: u8) -> bool {
        let mut m = self.read_bus(io);
        m = self.decrement(m);
        self.write_bus(io, m);

        true
    }

    fn dex(&mut self) -> bool {
        self.x = self.decrement(self.x);
        true
    }

    fn dey(&mut self) -> bool {
        self.y = self.decrement(self.y);
        true
    }

    fn inc(&mut self, io: &mut dyn IoAccess) -> bool {
        let mut m = self.read_bus(io);
        m = self.increment(m);
        self.write_bus(io, m);

        true
    }

    fn inx(&mut self) -> bool {
        self.x = self.increment(self.x);
        true
    }

    fn iny(&mut self) -> bool {
        self.y = self.increment(self.y);
        true
    }

    fn eor(&mut self, io: &mut dyn IoAccess) -> bool {
        let m = self.read_bus(io);
        self.a ^= m;
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
        true
    }

    fn ldx(&mut self, io: &mut dyn IoAccess) -> bool {
        self.x = self.read_bus(io);
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
        true
    }

    fn ldy(&mut self, io: &mut dyn IoAccess) -> bool {
        self.y = self.read_bus(io);
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
        true
    }

    fn pha(&mut self, io: &mut dyn IoAccess, cycle: u8) -> bool {
        match cycle {
            2 => {
                self.push(io, self.a);
                true
            },
            _ => {
                false
            }
        }
    }

    fn php(&mut self, io: &mut dyn IoAccess, cycle: u8) -> bool {
        match cycle {
            2 => {
                self.push(io, self.p);
                true
            },
            _ => {
                false
            }
        }
    }

    fn pla(&mut self, io: &mut dyn IoAccess, cycle: u8) -> bool {
        match cycle {
            3 => {
                self.a = self.pull(io);
                true
            },
            _ => {
                false
            }
        }
    }

    fn plp(&mut self, io: &mut dyn IoAccess, cycle: u8) -> bool {
        match cycle {
            3 => {
                self.p = self.pull(io);
                true
            },
            _ => {
                false
            }
        }
    }

    fn lsr(&mut self, io: &mut dyn IoAccess) -> bool {
        let m = self.read_bus(io);
        let c = bit_is_set!(m, 0);

        let r = m >> 1;
        self.write_bus(io, r);

        self.set_zero_flag(r);
        self.set_negative_flag(r);
        self.set_flag_bit(Flags::Carry, c);

        true
    }

    fn ora(&mut self, io: &mut dyn IoAccess) -> bool {
        let m = self.read_bus(io);
        self.a = self.a | m;

        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);

        true
    }

    //------------------------------------------------------------------------------------------------------------------
    // Addressing Modes
    //------------------------------------------------------------------------------------------------------------------

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
                self.address_bus = (self.address_bus & 0xFF00) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x00FF) | ((self.read_next_u8(io) as u16) << 8);
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

    fn absolute_i(&mut self, io: &mut dyn IoAccess, cycle: u8, i: u8) {
        match cycle {
            0 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xFF00) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x00FF) | ((self.read_next_u8(io) as u16) << 8);
            },
            2 => {
                // Add the index value to the address bus
                self.address_bus += i as u16;
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

    fn zeropage_i(&mut self, io: &mut dyn IoAccess, cycle: u8, i: u8) {
        match cycle {
            0 => {
                self.address_bus = self.read_next_u8(io) as u16;
            },
            1 => {
                // TODO: Wrapping?
                self.address_bus += i as u16;
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
                self.pointer_address += self.x as u16;
            },
            2 => {
                // Fetch lower byte of address
                self.address_bus = (self.address_bus & 0xFF00) | (self.read_u8(io, self.pointer_address) as u16);
            },
            3 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x00FF) | ((self.read_u8(io, self.pointer_address + 1) as u16) << 8);
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
                self.address_bus = (self.address_bus & 0xFF00) | (self.read_u8(io, self.pointer_address) as u16);
            },
            2 => {
                // Fetch the higher byte of address
                self.address_bus = (self.address_bus & 0x00FF) | ((self.read_u8(io, self.pointer_address + 1) as u16) << 8);
            },
            3 => {
                self.address_bus += self.y as u16;
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
                self.pointer_address = (self.pointer_address & 0xFF00) | (self.read_next_u8(io) as u16);
            },
            1 => {
                // Fetch the higher byte of address
                self.pointer_address = (self.pointer_address & 0x00FF) | ((self.read_next_u8(io) as u16) << 8);
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

    fn relative(&mut self, io: &mut dyn IoAccess) {
        self.pointer_address = self.read_next_u8(io) as u16;
        self.addressing_complete = true;
    }

    //------------------------------------------------------------------------------------------------------------------
    // Flags Register
    //------------------------------------------------------------------------------------------------------------------

    fn update_flags_with_carry(&mut self, a: u8, c: bool) {
        self.update_flags(a);
        self.set_flag_bit(Flags::Carry, c);
    }

    fn update_flags(&mut self, a: u8) {
        self.set_zero_flag(a);
        self.set_negative_flag(a);
    }

    fn set_zero_flag(&mut self, a: u8) {
        self.set_flag_bit(Flags::Zero, a == 0);
    }

    fn set_negative_flag(&mut self, a: u8) {
        self.set_flag_bit(Flags::Negative, bit_is_set!(a, 7));
    }

    /// Get carry flag as a u8 for arthimetic operations
    fn get_carry(&self) -> u8 {
        if self.get_flag_bit(Flags::Carry) { 1 } else { 0 }
    }

    /// Get flag bit
    fn get_flag_bit(&self, f: Flags) -> bool {
        mask_is_set!(self.p, f as u8)
    }

    /// Set a flag bit
    fn set_flag_bit(&mut self, f: Flags, v: bool) {
        if v {
            mask_set!(self.p, f as u8);
        }
        else {
            mask_clear!(self.p, f as u8);
        }
    }

    //------------------------------------------------------------------------------------------------------------------
    // CPU Opertions
    //------------------------------------------------------------------------------------------------------------------

    /// Branch
    fn branch(&mut self, cond_met: bool) {
        if cond_met {
            let offset = Wrapping(self.pointer_address as i16);
            let pc = Wrapping(self.pc.0 as i16);
            self.pc = Wrapping((pc + offset).0 as u16);
        }
    }

    /// Do and compare operation on the given arguments and set appropriate flags
    fn compare(&mut self, a: u8, m: u8) {
        let r = (Wrapping(a) - Wrapping(m)).0;

        self.set_flag_bit(Flags::Carry, a >= m);
        self.set_zero_flag(r);
        self.set_negative_flag(r);
    }

    /// Decrement value, setting flags
    fn decrement(&mut self, a: u8) -> u8 {
        let new_a = (Wrapping(a) - Wrapping(1u8)).0;
        self.set_zero_flag(new_a);
        self.set_negative_flag(new_a);

        new_a
    }

    /// Increment value, setting flags
    fn increment(&mut self, a: u8) -> u8 {
        let new_a = (Wrapping(a) + Wrapping(1u8)).0;
        self.set_zero_flag(new_a);
        self.set_negative_flag(new_a);

        new_a
    }

    /// Push a value onto the stack
    fn push(&mut self, io: &mut dyn IoAccess, data: u8) {
        // Read the next instruction byte and throw it away
        self.pc += Wrapping(1u16);

        self.write_u8(io, self.sp, data);
        self.sp -= 1u16;
    }

    /// PUll a value off the stack
    fn pull(&mut self, io: &mut dyn IoAccess) -> u8 {
        // Read the next instruction byte and throw it away
        self.pc += Wrapping(1u16);

        self.sp += 1u16;
        let data = self.read_u8(io, self.sp);

        data
    }

    //------------------------------------------------------------------------------------------------------------------
    // Base CPU Read/Write Operations
    //------------------------------------------------------------------------------------------------------------------

    /// Fetch the next opcode and increment the program counter
    fn fetch(&mut self, io: &mut dyn IoAccess) -> u8 {
        self.read_next_u8(io)
    }

    fn read_bus(&self, io: &mut dyn IoAccess) -> u8 {
        let mode = match self.state {
            State::Execute(_, mode, _) => mode,
            _ => panic!("Must be in execution state!"),
        };

        if mode == AddressingMode::Accumulator {
            self.a
        }
        else {
            self.read_u8(io, self.address_bus)
        }
    }

    fn write_bus(&mut self, io: &mut dyn IoAccess, value: u8) {
        let mode = match self.state {
            State::Execute(_, mode, _) => mode,
            _ => panic!("Must be in execution state!"),
        };

        if mode == AddressingMode::Accumulator {
            self.a = value;
        }
        else {
            self.write_u8(io, self.address_bus, value);
        }
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
        if (addr as usize) < INTERNAL_RAM_SIZE {
            self.ram[(addr as usize) % 0x200]
        }
        else {
            io.read_byte(addr)
        }
    }

    fn write_u8(&mut self, io: &mut dyn IoAccess, addr: u16, value: u8) {
        if (addr as usize) < INTERNAL_RAM_SIZE {
            self.ram[(addr as usize) % 0x200] = value;
        }
        else {
            io.write_byte(addr, value);
        }
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

//----------------------------------------------------------------------------------------------------------------------
// Tests
//----------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use helper::*;

    #[test]
    fn pc_after_reset() {
        let mut cpu = Cpu::new();
        cpu.pc = Wrapping(0x0001);

        simple_test_base(&mut cpu, vec![], 0);

        assert_eq!(cpu.pc.0, 0x4020);
    }

    #[test]
    fn nop() {
        let prg = vec![0xEA];
        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.pc.0, 0x4021);
    }

    #[test]
    fn lda_immediate() {
        let prg = vec![
            0xA9, 0xA5 // LDA $A5
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.a, 0xA5);
    }

    #[test]
    fn lda_absolute() {
        let prg = vec![
            0xAD, 0x23, 0x40, // LDA ($4023)
            0xDE,             // Data: $DE
        ];

        let cpu = simple_test(prg, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_zeropage() {
        let mut cpu = Cpu::new();
        cpu.ram[0x02] = 0xDE;

        let prg = vec![
            0xA5, 0x02, // LDA ($02)
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_zeropage_x() {
        let mut cpu = Cpu::new();
        cpu.ram[0x03] = 0xDE;
        cpu.x = 0x0001;

        let prg = vec![
            0xB5, 0x02, // LDA $02, X
        ];

        simple_test_base(&mut cpu, prg, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_x() {
        let mut cpu = Cpu::new();
        cpu.x = 0x0001;

        let prg = vec![
            0xBD, 0x23, 0x40, // LDA $0003, X
            0x00, 0xDE,       // Data: $DE
        ];

        simple_test_base(&mut cpu, prg, 5);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_y() {
        let mut cpu = Cpu::new();
        cpu.y = 0x0001;

        let prg = vec![
            0xB9, 0x23, 0x40, // LDA $0003, Y
            0x00, 0xDE,       // Data: $DE
        ];

        simple_test_base(&mut cpu, prg, 5);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indexed_indirect() {
        let mut cpu = Cpu::new();
        cpu.ram[0x03] = 0x05;
        cpu.ram[0x04] = 0x00;
        cpu.ram[0x05] = 0xDE;

        cpu.x = 0x0001;

        let prg = vec![
            0xA1, 0x02, // LDA ($0002, X)
        ];

        simple_test_base(&mut cpu, prg, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indirect_indexed() {
        let mut cpu = Cpu::new();
        cpu.ram[0x02] = 0x05;
        cpu.ram[0x03] = 0x00;
        cpu.ram[0x06] = 0xDE;

        cpu.y = 0x0001;

        let prg = vec![
            0xB1, 0x02, // LDA ($0002), Y
        ];

        simple_test_base(&mut cpu, prg, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_flags_zero() {
        let prg = vec![
            0xA9, 0x00 // LDA $00
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.p, Flags::Zero as u8);
    }

    #[test]
    fn lda_flags_negative() {
        let prg = vec![
            0xA9, 0x80 // LDA $00
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.p, Flags::Negative as u8);
    }

    #[test]
    fn jmp_absolute() {
        let prg = vec![
            0x4C, 0x00, 0x10 // LDA JMP $1000
        ];

       // FIXME: JMP with absolute addressing should be 3 cycles
       let cpu = simple_test(prg, 4);

        assert_eq!(cpu.pc.0, 0x1000);
    }

    #[test]
    fn jmp_indirect() {
        let prg = vec![
            0x6C, 0x23, 0x40, // LDA JMP ($0003)
            0x00, 0x10,       // Address: $1000
        ];

        let cpu = simple_test(prg, 5);

        assert_eq!(cpu.pc.0, 0x1000);
    }

    #[test]
    fn adc_immediate_no_carry() {
        let prg = vec![
            0x69, 0x05, // ADC $05
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.a, 0x05);
        assert_eq!(mask_is_clear!(cpu.p, Flags::Carry as u8), true);
    }

    #[test]
    fn adc_immediate_carry() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;

        let prg = vec![
            0x69, 0x01, // ADC $01
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(mask_is_set!(cpu.p, Flags::Carry as u8), true);
    }

    #[test]
    fn adc_immediate_with_carry_set() {
        let mut cpu = Cpu::new();

        let prg = vec![
            0x69, 0xFF, // ADC $FF; a=$0  -> a=$FF
            0x69, 0x01, // ADC $01; a=$FF -> a=00, c=1
            0x69, 0x00, // ADC $00; a=$00 -> a=$01, c=0
        ];

        simple_test_base(&mut cpu, prg, 9);

        assert_eq!(cpu.a, 0x01);
        assert_eq!(mask_is_clear!(cpu.p, Flags::Carry as u8), true);
    }

    #[test]
    fn and_immediate() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;

        let prg = vec![
            0x29, 0xF0, // AND $F0
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0xF0);
        assert_eq!(mask_is_set!(cpu.p, Flags::Negative as u8), true);
    }

    #[test]
    fn and_immediate_zero_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;

        let prg = vec![
            0x29, 0x00, // AND $F0
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(mask_is_set!(cpu.p, Flags::Negative as u8), false);
        assert_eq!(mask_is_set!(cpu.p, Flags::Zero as u8), true);
    }

    #[test]
    fn zero_flag_cleared() {
        let prg = vec![
            0xA9, 0x00, // LDA $00; a=$0,z=1
            0xA9, 0x01, // LDA $01; a=$1,z=0
        ];

        let cpu = simple_test(prg, 6);

        assert_eq!(cpu.a, 0x01);
        assert_eq!(mask_is_clear!(cpu.p, Flags::Zero as u8), true);
    }

    #[test]
    fn asl_accumulator() {
        let mut cpu = Cpu::new();
        cpu.a = 0x01;

        let prg = vec![
            0x0A, // ASL
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.a, 0x02);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), false);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), false);
    }

    #[test]
    fn asl_accumulator_carry_and_negative_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0xC0;

        let prg = vec![
            0x0A, // ASL
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), true);
    }

    #[test]
    fn sta_zeropage() {
        let mut cpu = Cpu::new();
        cpu.a = 0xDE;

        let prg = vec![
            0x85, 0x05, // STA $05
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.ram[0x05], 0xDE);
    }

    #[test]
    fn bcc_carry_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Carry as u8);

        let prg = vec![
            0x90, 0x02, // BCC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bcc_carry_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Carry as u8);

        let prg = vec![
            0x90, 0x02, // BCC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bcs_carry_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Carry as u8);

        let prg = vec![
            0xB0, 0x02, // BSC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bcs_carry_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Carry as u8);

        let prg = vec![
            0xB0, 0x02, // BSC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn beq_zero_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Zero as u8);

        let prg = vec![
            0xF0, 0x02, // BEQ $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn beq_zero_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Zero as u8);

        let prg = vec![
            0xF0, 0x02, // BEQ $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bne_zero_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Zero as u8);

        let prg = vec![
            0xD0, 0x02, // BNE $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bne_zero_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Zero as u8);

        let prg = vec![
            0xD0, 0x02, // BNE $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bmi_negative_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Negative as u8);

        let prg = vec![
            0x30, 0x02, // BMI $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bmi_negative_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Negative as u8);

        let prg = vec![
            0x30, 0x02, // BMI $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bpl_negative_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Negative as u8);

        let prg = vec![
            0x10, 0x02, // BPL $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bpl_negative_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Negative as u8);

        let prg = vec![
            0x10, 0x02, // BPL $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bvc_overflow_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Overflow as u8);

        let prg = vec![
            0x50, 0x02, // BVC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bvc_overflow_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Overflow as u8);

        let prg = vec![
            0x50, 0x02, // BVC $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bvs_overflow_not_set() {
        let mut cpu = Cpu::new();
        mask_clear!(cpu.p, Flags::Overflow as u8);

        let prg = vec![
            0x70, 0x02, // BVS $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4022);
    }

    #[test]
    fn bvs_overflow_set() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Overflow as u8);

        let prg = vec![
            0x70, 0x02, // BVS $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.pc.0, 0x4024);
    }

    #[test]
    fn bit_check_mask() {
        let mut cpu = Cpu::new();
        cpu.a = 0x01;
        cpu.ram[0x02] = 0x01;

        let prg = vec![
            0x24, 0x02, // BIT $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
    }

    #[test]
    fn bit_check_mask_not_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0x01;
        cpu.ram[0x02] = 0x00;

        let prg = vec![
            0x24, 0x02, // BIT $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn bit_check_mask_absolute() {
        let mut cpu = Cpu::new();
        cpu.a = 0x01;
        cpu.ram[0x02] = 0x01;

        let prg = vec![
            0x2C, 0x02, 0x00, // BIT $02
        ];

        simple_test_base(&mut cpu, prg, 4);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
    }

    #[test]
    fn clear_instructions() {
        let mut cpu = Cpu::new();
        mask_set!(cpu.p, Flags::Carry as u8);
        mask_set!(cpu.p, Flags::Decimal as u8);
        mask_set!(cpu.p, Flags::InterruptDisable as u8);
        mask_set!(cpu.p, Flags::Overflow as u8);

        let prg = vec![
            0x18, // CLC
            0xD8, // CLD
            0x58, // CLI
            0xB8, // CLV
        ];

        simple_test_base(&mut cpu, prg, 8);

        assert_eq!(cpu.get_flag_bit(Flags::Carry), false);
        assert_eq!(cpu.get_flag_bit(Flags::Decimal), false);
        assert_eq!(cpu.get_flag_bit(Flags::InterruptDisable), false);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), false);
    }

    #[test]
    fn cmp_zero_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0x02;

        let prg = vec![
            0xC9, 0x02, // CMP $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn cmp_negative_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0x00;

        let prg = vec![
            0xC9, 0x01, // CMP $02
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Negative), true);
    }

    #[test]
    fn dec_mem() {
        let mut cpu = Cpu::new();
        cpu.ram[0x02] = 0x01;

        let prg = vec![
            0xC6, 0x02, // DEC $02
        ];

        // FIXME: Cycle accuracy
        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.ram[0x02], 0x00);
    }

    #[test]
    fn dex() {
        let mut cpu = Cpu::new();
        cpu.x = 0x01;

        let prg = vec![
            0xCA, // DEX
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn dey() {
        let mut cpu = Cpu::new();
        cpu.y = 0x01;

        let prg = vec![
            0x88, // DEY
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn inc_mem() {
        let mut cpu = Cpu::new();
        cpu.ram[0x02] = 0xFF;

        let prg = vec![
            0xE6, 0x02, // INC $02
        ];

        // FIXME: Cycle accuracy
        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.ram[0x02], 0x00);
    }

    #[test]
    fn inx() {
        let mut cpu = Cpu::new();
        cpu.x = 0xFF;

        let prg = vec![
            0xE8, // INX
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn iny() {
        let mut cpu = Cpu::new();
        cpu.y = 0xFF;

        let prg = vec![
            0xC8, // INY
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn eor() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;

        let prg = vec![
            0x49, 0xFF, // EOR $FF
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn ldx_immediate() {
        let prg = vec![
            0xA2, 0xA5 // LDX $A5
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.x, 0xA5);
    }

    #[test]
    fn ldy_immediate() {
        let prg = vec![
            0xA0, 0xA5 // LDY $A5
        ];

        let cpu = simple_test(prg, 3);

        assert_eq!(cpu.y, 0xA5);
    }

    #[test]
    fn pha() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;
        cpu.sp = 0x0A;

        let prg = vec![
            0x48, // PHA
        ];

        simple_test_base(&mut cpu, prg, 4);

        assert_eq!(cpu.sp, 0x09);
        assert_eq!(cpu.ram[0x0A], 0xFF);
    }

    #[test]
    fn pla() {
        let mut cpu = Cpu::new();
        cpu.ram[0x0A] = 0xFF;
        cpu.sp = 0x09;

        let prg = vec![
            0x68,       // PLA
        ];

        simple_test_base(&mut cpu, prg, 5);

        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.sp, 0x0A);
    }


    #[test]
    fn php() {
        let mut cpu = Cpu::new();
        cpu.p = 0xFF;
        cpu.sp = 0x0A;

        let prg = vec![
            0x08, // PHP
        ];

        simple_test_base(&mut cpu, prg, 4);

        assert_eq!(cpu.sp, 0x09);
        assert_eq!(cpu.ram[0x0A], 0xFF);
    }

    #[test]
    fn plp() {
        let mut cpu = Cpu::new();
        cpu.ram[0x0A] = 0xFF;
        cpu.sp = 0x09;

        let prg = vec![
            0x28, // PLP
        ];

        simple_test_base(&mut cpu, prg, 5);

        assert_eq!(cpu.p, 0xFF);
        assert_eq!(cpu.sp, 0x0A);
    }

    #[test]
    fn lsr_zero_set() {
        let mut cpu = Cpu::new();
        cpu.a = 0x01;

        let prg = vec![
            0x4A, // LSR A
        ];

        simple_test_base(&mut cpu, prg, 2);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
    }

    #[test]
    fn ora() {
        let mut cpu = Cpu::new();
        cpu.a = 0xF0;

        let prg = vec![
            0x09, 0x0F, // ORA $0F
        ];

        simple_test_base(&mut cpu, prg, 3);

        assert_eq!(cpu.a, 0xFF);
    }

    ///-----------------------------------------------------------------------------------------------------------------
    /// Helper functions
    ///-----------------------------------------------------------------------------------------------------------------
    mod helper {
        use super::*;

        pub struct CpuIoBus {
            prg_rom: Vec<u8>, // ROM
            rom_offest: usize,
        }

        impl CpuIoBus {
            pub fn from(prg_rom: Vec<u8>) -> Self {
                CpuIoBus {
                    prg_rom: prg_rom,
                    rom_offest: 0x4020,
                }
            }
        }

        impl IoAccess for CpuIoBus {
            fn read_byte(&self, addr: u16) -> u8 {
                if addr == 0xFFFC {
                    0x20
                }
                else if addr == 0xFFFD {
                    0x40
                }
                else {
                    if addr >= 0x4020 {
                        self.prg_rom[(addr as usize) - self.rom_offest]
                    }
                    else {
                        panic!("Address out of supplied program ROM range");
                    }
                }
            }

            fn write_byte(&mut self, _addr: u16, _data: u8) {

            }
        }

        pub fn simple_test(prg: Vec<u8>, ticks: usize) -> Cpu {
            let mut cpu = Cpu::new();
            simple_test_base(&mut cpu, prg, ticks);

            cpu
        }

        pub fn simple_test_base(cpu: &mut Cpu, prg: Vec<u8>, ticks: usize) {
            let mut io = CpuIoBus::from(prg);
            run_cpu(cpu, &mut io, ticks);
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
