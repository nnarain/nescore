//
// asm.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 26 2020
//
use std::fmt;

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
            AddressingMode::Relative => 1,
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum InstructionCategory {
    Read, Write, ReadModifyWrite, Branch, Implied
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
    LAX, SAX, DCP, ISB, SLO, RLA, RRA, SRE, ANC, ALR, ARR, AXS, SHY, SHX
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

impl Instruction {
    pub fn category(&self) -> InstructionCategory {
        match *self {
              Instruction::LDA
            | Instruction::LAX
            | Instruction::LDX
            | Instruction::LDY
            | Instruction::EOR
            | Instruction::AND
            | Instruction::ANC
            | Instruction::ALR
            | Instruction::ARR
            | Instruction::AXS
            | Instruction::ORA
            | Instruction::ADC
            | Instruction::SBC
            | Instruction::CMP
            | Instruction::CPY
            | Instruction::CPX
            | Instruction::BIT => InstructionCategory::Read,
            
              Instruction::STA
            | Instruction::STX
            | Instruction::SAX
            | Instruction::STY => InstructionCategory::Write,

              Instruction::ASL
            | Instruction::SHY
            | Instruction::SHX
            | Instruction::LSR
            | Instruction::ROL
            | Instruction::ROR
            | Instruction::DCP
            | Instruction::ISB
            | Instruction::SLO
            | Instruction::RLA
            | Instruction::RRA
            | Instruction::SRE
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
            | Instruction::CLC
            | Instruction::DEX
            | Instruction::DEY
            | Instruction::BRK
            | Instruction::INX
            | Instruction::INY
            | Instruction::PHA
            | Instruction::PHP
            | Instruction::PLA
            | Instruction::PLP => InstructionCategory::Implied,

            _ => InstructionCategory::Implied,
        }
    }
}

pub fn cycle_count(instr: Instruction, mode: AddressingMode) -> usize {
    match mode {
        AddressingMode::Implied => {
            match instr {
                Instruction::BRK => 6,
                Instruction::RTI => 5,
                Instruction::RTS => 5,
                Instruction::PHA | Instruction::PHP => 2,
                Instruction::PLA | Instruction::PLP => 3,
                Instruction::CLD | Instruction::CLI | Instruction::CLV | Instruction::CLC => 1,
                Instruction::SEC | Instruction::SED | Instruction::SEI => 1,
                Instruction::TAX | Instruction::TAY | Instruction::TSX | Instruction::TXA | Instruction::TXS | Instruction::TYA => 1,
                Instruction::DEX | Instruction::DEY => 1,
                Instruction::INX | Instruction::INY => 1,
                Instruction::NOP => 1,

                _ => unreachable!("Matching implied instructions"),
            }
        },
        AddressingMode::Absolute => {
            match instr {
                Instruction::JMP => 2,
                Instruction::JSR => 5,
                _ => {
                    match instr.category() {
                        InstructionCategory::Read => 3,
                        InstructionCategory::ReadModifyWrite => 5,
                        InstructionCategory::Write => 3,
                        InstructionCategory::Implied => 3, // FIXME: What should this be?

                        _ => unreachable!("Matching absolute instructions to categories: {:?}", instr),
                    }
                }
            }
        },
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
            match instr.category() {
                InstructionCategory::Read => 4,
                InstructionCategory::ReadModifyWrite => 6,
                InstructionCategory::Write => 4,
                InstructionCategory::Implied => 3, // FIXME: What should this be?
                _ => unreachable!("Matching Absolute indexed addressing to categories"),
            }
        },
        AddressingMode::ZeroPage => {
            match instr.category() {
                InstructionCategory::Read => 2,
                InstructionCategory::ReadModifyWrite => 4,
                InstructionCategory::Write => 2,
                InstructionCategory::Implied => 3, // FIXME: What should this be?
                _ => unreachable!("Matching ZeroPage to categories {:?}", instr=instr),
            }
        },
        AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
            match instr.category() {
                InstructionCategory::Read => 3,
                InstructionCategory::ReadModifyWrite => 5,
                InstructionCategory::Write => 3,
                InstructionCategory::Implied => 3, // FIXME: What should this be?
                _ => unreachable!("Matching ZeroPage Indexed to categories"),
            }
        },
        AddressingMode::Relative => {
            match instr.category() {
                InstructionCategory::Branch => 2, // FIXME: Not cycle accurate
                _ => unreachable!("Matching branch instructions to categories"),
            }
        },
        AddressingMode::IndexedIndirect => {
            match instr.category() {
                InstructionCategory::Read => 5,
                InstructionCategory::ReadModifyWrite => 7,
                InstructionCategory::Write => 5,
                _ => unreachable!("Matching indexed indirect instructions to categories"),
            }
        },
        AddressingMode::IndirectIndexed => {
            match instr.category() {
                InstructionCategory::Read => 5,
                InstructionCategory::ReadModifyWrite => 7,
                InstructionCategory::Write => 5,
                _ => unreachable!("Matching indirect indexed to categories"),
            }
        },
        AddressingMode::Indirect => {
            // JMP
            4
        }
        AddressingMode::Accumulator | AddressingMode::Immediate => 1,
    }
}

pub fn decode(opcode: u8) -> (Instruction, AddressingMode) {
    match opcode {
        // NOP
        0xEA | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => (Instruction::NOP, AddressingMode::Implied),
        0x04 | 0x44 | 0x64 | 0x82 | 0x89 | 0xC2 | 0xE2 => (Instruction::NOP, AddressingMode::Immediate),
        0x0C => (Instruction::NOP, AddressingMode::Absolute),
        0x80 => (Instruction::NOP, AddressingMode::ZeroPage),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => (Instruction::NOP, AddressingMode::ZeroPageX),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => (Instruction::NOP, AddressingMode::AbsoluteX),
        // LDA
        0xA9 => (Instruction::LDA, AddressingMode::Immediate),
        0xA5 => (Instruction::LDA, AddressingMode::ZeroPage),
        0xB5 => (Instruction::LDA, AddressingMode::ZeroPageX),
        0xAD => (Instruction::LDA, AddressingMode::Absolute),
        0xBD => (Instruction::LDA, AddressingMode::AbsoluteX),
        0xB9 => (Instruction::LDA, AddressingMode::AbsoluteY),
        0xA1 => (Instruction::LDA, AddressingMode::IndexedIndirect),
        0xB1 => (Instruction::LDA, AddressingMode::IndirectIndexed),
        // LAX
        0xAB => (Instruction::LAX, AddressingMode::Immediate),
        0xA7 => (Instruction::LAX, AddressingMode::ZeroPage),
        0xB7 => (Instruction::LAX, AddressingMode::ZeroPageY),
        0xAF => (Instruction::LAX, AddressingMode::Absolute),
        0xBF => (Instruction::LAX, AddressingMode::AbsoluteY),
        0xA3 => (Instruction::LAX, AddressingMode::IndexedIndirect),
        0xB3 => (Instruction::LAX, AddressingMode::IndirectIndexed),
        // SAX
        0x87 => (Instruction::SAX, AddressingMode::ZeroPage),
        0x97 => (Instruction::SAX, AddressingMode::ZeroPageY),
        0x8F => (Instruction::SAX, AddressingMode::Absolute),
        0x83 => (Instruction::SAX, AddressingMode::IndexedIndirect),
        // DCP
        0xC7 => (Instruction::DCP, AddressingMode::ZeroPage),
        0xD7 => (Instruction::DCP, AddressingMode::ZeroPageX),
        0xCF => (Instruction::DCP, AddressingMode::Absolute),
        0xDF => (Instruction::DCP, AddressingMode::AbsoluteX),
        0xDB => (Instruction::DCP, AddressingMode::AbsoluteY),
        0xC3 => (Instruction::DCP, AddressingMode::IndexedIndirect),
        0xD3 => (Instruction::DCP, AddressingMode::IndirectIndexed),
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
        // ANC
        0x0B | 0x2B => (Instruction::ANC, AddressingMode::Immediate),
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
        0xC0 => (Instruction::CPY, AddressingMode::Immediate),
        0xC4 => (Instruction::CPY, AddressingMode::ZeroPage),
        0xCC => (Instruction::CPY, AddressingMode::Absolute),
        // DEC
        0xC6 => (Instruction::DEC, AddressingMode::ZeroPage),
        0xD6 => (Instruction::DEC, AddressingMode::ZeroPageX),
        0xCE => (Instruction::DEC, AddressingMode::Absolute),
        0xDE => (Instruction::DEC, AddressingMode::AbsoluteX),
        // DEX
        0xCA => (Instruction::DEX, AddressingMode::Implied),
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
        // ALR
        0x4B => (Instruction::ALR, AddressingMode::Immediate),
        // ARR
        0x6B => (Instruction::ARR, AddressingMode::Immediate),
        // AXS
        0xCB => (Instruction::AXS, AddressingMode::Immediate),
        // ORA
        0x09 => (Instruction::ORA, AddressingMode::Immediate),
        0x05 => (Instruction::ORA, AddressingMode::ZeroPage),
        0x15 => (Instruction::ORA, AddressingMode::ZeroPageX),
        0x0D => (Instruction::ORA, AddressingMode::Absolute),
        0x1D => (Instruction::ORA, AddressingMode::AbsoluteX),
        0x19 => (Instruction::ORA, AddressingMode::AbsoluteY),
        0x01 => (Instruction::ORA, AddressingMode::IndexedIndirect),
        0x11 => (Instruction::ORA, AddressingMode::IndirectIndexed),
        // ROR
        0x6A => (Instruction::ROR, AddressingMode::Accumulator),
        0x66 => (Instruction::ROR, AddressingMode::ZeroPage),
        0x76 => (Instruction::ROR, AddressingMode::ZeroPageX),
        0x6E => (Instruction::ROR, AddressingMode::Absolute),
        0x7E => (Instruction::ROR, AddressingMode::AbsoluteX),
        // ROL
        0x2A => (Instruction::ROL, AddressingMode::Accumulator),
        0x26 => (Instruction::ROL, AddressingMode::ZeroPage),
        0x36 => (Instruction::ROL, AddressingMode::ZeroPageX),
        0x2E => (Instruction::ROL, AddressingMode::Absolute),
        0x3E => (Instruction::ROL, AddressingMode::AbsoluteX),
        // RLA
        0x27 => (Instruction::RLA, AddressingMode::ZeroPage),
        0x37 => (Instruction::RLA, AddressingMode::ZeroPageX),
        0x2F => (Instruction::RLA, AddressingMode::Absolute),
        0x3F => (Instruction::RLA, AddressingMode::AbsoluteX),
        0x3B => (Instruction::RLA, AddressingMode::AbsoluteY),
        0x23 => (Instruction::RLA, AddressingMode::IndexedIndirect),
        0x33 => (Instruction::RLA, AddressingMode::IndirectIndexed),
        // RRA
        0x67 => (Instruction::RRA, AddressingMode::ZeroPage),
        0x77 => (Instruction::RRA, AddressingMode::ZeroPageX),
        0x6F => (Instruction::RRA, AddressingMode::Absolute),
        0x7F => (Instruction::RRA, AddressingMode::AbsoluteX),
        0x7B => (Instruction::RRA, AddressingMode::AbsoluteY),
        0x63 => (Instruction::RRA, AddressingMode::IndexedIndirect),
        0x73 => (Instruction::RRA, AddressingMode::IndirectIndexed),
        // SRE
        0x47 => (Instruction::SRE, AddressingMode::ZeroPage),
        0x57 => (Instruction::SRE, AddressingMode::ZeroPageX),
        0x4F => (Instruction::SRE, AddressingMode::Absolute),
        0x5F => (Instruction::SRE, AddressingMode::AbsoluteX),
        0x5B => (Instruction::SRE, AddressingMode::AbsoluteY),
        0x43 => (Instruction::SRE, AddressingMode::IndexedIndirect),
        0x53 => (Instruction::SRE, AddressingMode::IndirectIndexed),
        // RTI
        0x40 => (Instruction::RTI, AddressingMode::Implied),
        // JSR
        0x20 => (Instruction::JSR, AddressingMode::Absolute),
        // RTS
        0x60 => (Instruction::RTS, AddressingMode::Implied),
        // SBC
        0xE9 | 0xEB => (Instruction::SBC, AddressingMode::Immediate),
        0xE5 => (Instruction::SBC, AddressingMode::ZeroPage),
        0xF5 => (Instruction::SBC, AddressingMode::ZeroPageX),
        0xED => (Instruction::SBC, AddressingMode::Absolute),
        0xFD => (Instruction::SBC, AddressingMode::AbsoluteX),
        0xF9 => (Instruction::SBC, AddressingMode::AbsoluteY),
        0xE1 => (Instruction::SBC, AddressingMode::IndexedIndirect),
        0xF1 => (Instruction::SBC, AddressingMode::IndirectIndexed),
        // ISB
        0xE7 => (Instruction::ISB, AddressingMode::ZeroPage),
        0xF7 => (Instruction::ISB, AddressingMode::ZeroPageX),
        0xEF => (Instruction::ISB, AddressingMode::Absolute),
        0xFF => (Instruction::ISB, AddressingMode::AbsoluteX),
        0xFB => (Instruction::ISB, AddressingMode::AbsoluteY),
        0xE3 => (Instruction::ISB, AddressingMode::IndexedIndirect),
        0xF3 => (Instruction::ISB, AddressingMode::IndirectIndexed),
        // SLO
        0x07 => (Instruction::SLO, AddressingMode::ZeroPage),
        0x17 => (Instruction::SLO, AddressingMode::ZeroPageX),
        0x0F => (Instruction::SLO, AddressingMode::Absolute),
        0x1F => (Instruction::SLO, AddressingMode::AbsoluteX),
        0x1B => (Instruction::SLO, AddressingMode::AbsoluteY),
        0x03 => (Instruction::SLO, AddressingMode::IndexedIndirect),
        0x13 => (Instruction::SLO, AddressingMode::IndirectIndexed),
        // SEC
        0x38 => (Instruction::SEC, AddressingMode::Implied),
        // SED
        0xF8 => (Instruction::SED, AddressingMode::Implied),
        // SEI
        0x78 => (Instruction::SEI, AddressingMode::Implied),
        // SHY
        0x9C => (Instruction::SHY, AddressingMode::AbsoluteX),
        // SHX
        0x9E => (Instruction::SHX, AddressingMode::AbsoluteY),
        // STX
        0x86 => (Instruction::STX, AddressingMode::ZeroPage),
        0x96 => (Instruction::STX, AddressingMode::ZeroPageY),
        0x8E => (Instruction::STX, AddressingMode::Absolute),
        // STY
        0x84 => (Instruction::STY, AddressingMode::ZeroPage),
        0x94 => (Instruction::STY, AddressingMode::ZeroPageX),
        0x8C => (Instruction::STY, AddressingMode::Absolute),
        // TAX
        0xAA => (Instruction::TAX, AddressingMode::Implied),
        // TAY
        0xA8 => (Instruction::TAY, AddressingMode::Implied),
        // TSX
        0xBA => (Instruction::TSX, AddressingMode::Implied),
        // TXA
        0x8A => (Instruction::TXA, AddressingMode::Implied),
        // TXS
        0x9A => (Instruction::TXS, AddressingMode::Implied),
        // TYA
        0x98 => (Instruction::TYA, AddressingMode::Implied),
        // BRK - Followed by an unused byte
        0x00 => (Instruction::BRK, AddressingMode::Immediate),

        _ => {
            panic!("Invalid opcode: ${opcode}", opcode=format!("{:X}", opcode));
        }
    }
}

/// Returns a `String` representation of the instruction and the given addressing mode
pub fn disassemble(instr: Instruction, mode: AddressingMode, data: &[u8]) -> String {
    // TODO: Fix up ASM syntax
    match mode {
        AddressingMode::Accumulator     => format!("{:?} A       ", instr),
        AddressingMode::Implied         => format!("{:?}         ", instr),
        AddressingMode::Immediate       => format!("{:?} #{:02X}     ", instr, data[0]),
        AddressingMode::Relative        => format!("{:?} +{:02X}     ", instr, data[0]),
        AddressingMode::ZeroPage        => format!("{:?} {:02X}      ", instr, data[0]),
        AddressingMode::ZeroPageX       => format!("{:?} {:02X},X    ", instr, data[0]),
        AddressingMode::ZeroPageY       => format!("{:?} {:02X},Y    ", instr, data[0]),
        AddressingMode::Absolute        => format!("{:?} {:04X}    ", instr, address(data)),
        AddressingMode::AbsoluteX       => format!("{:?} {:04X},X  ", instr, address(data)),
        AddressingMode::AbsoluteY       => format!("{:?} {:04X},Y  ", instr, address(data)),
        AddressingMode::Indirect        => format!("{:?} ({:04X})  ", instr, address(data)),
        AddressingMode::IndexedIndirect => format!("{:?} ({:02X},X)  ", instr, data[0]),
        AddressingMode::IndirectIndexed => format!("{:?} ({:02X}),Y  ", instr, data[0]),
    }
}

pub fn operands(data: &[u8], operand_len: usize) -> String {
    match operand_len {
        0 => format!("      {:02X}", data[0]),
        1 => format!("   {:02X} {:02X}", data[0], data[1]),
        2 => format!("{:02X} {:02X} {:02X}", data[0], data[1], data[2]),
        _ => panic!("Invalid operand size"),
    }
}

fn address(data: &[u8]) -> u16 {
    let hi = data[1] as u16;
    let lo = data[0] as u16;

    (hi << 8) | lo
}
