//
// cpu/state.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 03 2019
//
use crate::io::IoAccess;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AddressingMode {
    Accumulator,
    Implied,
    Immediate,
    ZeroPage,
    Absolute,
    ZeroPageX,
    ZeroPageY,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
    Relative,
}

impl AddressingMode {
    pub fn operand_len(&self) -> usize {
        match *self {
            AddressingMode::Accumulator => 0,
            AddressingMode::Implied => 0,
            AddressingMode::Immediate => 1,
            AddressingMode::ZeroPage => 1,
            AddressingMode::Absolute => 2,
            AddressingMode::ZeroPageX => 1,
            AddressingMode::ZeroPageY => 1,
            AddressingMode::AbsoluteX => 2,
            AddressingMode::AbsoluteY => 2,
            AddressingMode::Indirect => 2,
            AddressingMode::IndexedIndirect => 1,
            AddressingMode::IndirectIndexed => 1,
            AddressingMode::Relative => 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AddressingModeResult {
    Byte(u8), Address(u16), Offset(u8), Implied
}

impl AddressingModeResult {
    pub fn to_address(&self) -> u16 {
        match *self {
            AddressingModeResult::Address(a) => a,
            _ => panic!("Not an Address result!"),
        }
    }

    pub fn to_byte(&self, io: &mut dyn IoAccess) -> u8 {
        match *self {
            AddressingModeResult::Byte(b) => b,
            AddressingModeResult::Address(a) => io.read_byte(a),
            _ => panic!("This result cannot be converted to a byte"),
        }
    }

    pub fn to_offset(&self) -> u8 {
        match *self {
            AddressingModeResult::Offset(o) => o,
            _ => panic!("Not an Offset result!"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InstructionCategory {
    Read, Write, ReadModifyWrite, Branch, Implied
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
}

impl Instruction {
    pub fn category(&self) -> InstructionCategory {
        match *self {
              Instruction::LDA
            | Instruction::LDX
            | Instruction::LDY
            | Instruction::EOR
            | Instruction::AND
            | Instruction::ORA
            | Instruction::ADC
            | Instruction::SBC
            | Instruction::CMP
            | Instruction::BIT => InstructionCategory::Read,
            
              Instruction::STA
            | Instruction::STX
            | Instruction::STY => InstructionCategory::Write,

              Instruction::ASL
            | Instruction::LSR
            | Instruction::ROL
            | Instruction::ROR
            | Instruction::INC
            | Instruction::DEC => InstructionCategory::ReadModifyWrite,

              Instruction::BCC
            | Instruction::BCS
            | Instruction::BEQ
            | Instruction::BMI
            | Instruction::BNE
            | Instruction::BPL
            | Instruction::BVC
            | Instruction::BVS => InstructionCategory::Branch,

              Instruction::NOP
            | Instruction::CLD
            | Instruction::CLI
            | Instruction::CLV
            | Instruction::CPX
            | Instruction::CLC
            | Instruction::DEX
            | Instruction::DEY
            | Instruction::BRK
            | Instruction::INX
            | Instruction::INY
            | Instruction::PHA
            | Instruction::PHP
            | Instruction::PLA
            | Instruction::PLP
            | Instruction::CPY => InstructionCategory::Implied,

            _ => InstructionCategory::Implied,
        }
    }
}

#[derive(Copy, Clone)]
pub enum State {
    Reset,
    Fetch,
    Execute(Instruction, AddressingMode, [u8; 3]),
}
