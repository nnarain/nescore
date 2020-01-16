//
// cpu/state.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 03 2019
//

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AddressingModeResult {
    Byte(u8), Address(u16), Offset(u8), Implied
}

impl AddressingModeResult {
    pub fn to_address(&self) -> Option<u16> {
        match *self {
            AddressingModeResult::Address(a) => Some(a),
            _ => None,
        }
    }

    pub fn to_byte<A2B>(&self, mut a2b: A2B) -> Option<u8> where A2B: FnMut(u16) -> u8 {
        match *self {
            AddressingModeResult::Byte(b) => Some(b),
            AddressingModeResult::Address(a) => {
                let b = a2b(a);
                Some(b)
            },
            _ => None,
        }
    }

    pub fn to_offset(&self) -> Option<u8> {
        match *self {
            AddressingModeResult::Offset(o) => Some(o),
            _ => None,
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

                        _ => unreachable!("Matching absolute instructions to categories"),
                    }
                }
            }
        },
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
            match instr.category() {
                InstructionCategory::Read => 4,
                InstructionCategory::ReadModifyWrite => 6,
                InstructionCategory::Write => 4,
                _ => unreachable!("Matching Absolute indexed addressing to categories"),
            }
        },
        AddressingMode::ZeroPage => {
            match instr.category() {
                InstructionCategory::Read => 2,
                InstructionCategory::ReadModifyWrite => 4,
                InstructionCategory::Write => 2,
                _ => unreachable!("Matching ZeroPage to categories"),
            }
        },
        AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => {
            match instr.category() {
                InstructionCategory::Read => 3,
                InstructionCategory::ReadModifyWrite => 5,
                InstructionCategory::Write => 3,
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
                _ => unreachable!("Matching indexed indirected instructions to categories"),
            }
        },
        AddressingMode::IndirectIndexed => {
            match instr.category() {
                InstructionCategory::Read => 5,
                InstructionCategory::ReadModifyWrite => 7,
                InstructionCategory::Write => 5,
                _ => unreachable!("Matching indirected indexed to categories"),
            }
        },
        AddressingMode::Indirect => {
            // JMP
            4
        }
        AddressingMode::Accumulator | AddressingMode::Immediate => 1,
    }
}

#[derive(Copy, Clone)]
pub enum State {
    Reset,
    Fetch,
    Execute(Instruction, AddressingMode, [u8; 3], usize),
}
