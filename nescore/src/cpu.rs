//
// cpu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 18 2019
//
pub mod bus;
pub mod format;
pub mod memorymap;
mod state;

// use bus::CpuIoBus;
use crate::common::{IoAccess, Clockable, Interrupt};
use state::*;
pub use state::{Instruction, AddressingMode};

use std::num::Wrapping;

#[cfg(feature="events")]
use std::sync::mpsc::Sender;

/// CPU Flags
enum Flags {
    Carry            = 1 << 0,
    Zero             = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal          = 1 << 3,
    Overflow         = 1 << 6,
    Negative         = 1 << 7,
}

#[cfg(feature="events")]
pub mod events {
    use super::{Instruction, AddressingMode};
    /// Representation of CPU and current instruction state
    #[derive(Debug)]
    pub struct InstructionData {
        pub instr: Instruction,
        pub mode: AddressingMode,
        pub opcode_data: [u8; 3],
        pub addr: u16,
        pub a: u8,
        pub x: u8,
        pub y: u8,
        pub p: u8,
        pub pc: u16,
        pub sp: u8,
    }
    /// Enum of CPU events
    pub enum CpuEvent {
        Instruction(InstructionData),
    }
}

const STACK_PAGE_OFFSET: u16 = 0x100;


/// NES Central Processing Unit
pub struct Cpu<Io: IoAccess> {
    a: u8,                          // General Purpose Accumulator
    x: u8,                          // Index register X
    y: u8,                          // Index register Y
    pc: u16,                        // Program Counter
    sp: u8,                         // Stack Pointer
    p: u8,                          // Flag register

    bus: Option<Io>,
    state: State,                   // Internal CPU cycle state

    interrupted: Option<Interrupt>, // Flag indicating the CPU was interrupt

    debug: bool,                    // Debug mode
    is_holding: bool,               // CPU is in an infinite loop state

    // Event logging
    #[cfg(feature="events")]
    logger: Option<Sender<events::CpuEvent>>,
}

impl<Io: IoAccess> Default for Cpu<Io> {
    fn default() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,
            p: 0x24,

            bus: None,
            state: State::Reset,

            interrupted: None,

            debug: false,
            is_holding: false,

            #[cfg(feature="events")]
            logger: None,
        }
    }
}

impl<Io: IoAccess> Cpu<Io> {
    pub fn load_bus(&mut self, bus: Io) {
        self.bus = Some(bus);
    }

    /// Set the CPU's program counter
    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
        // move to fetch state, as we no longer need to read the reset vector
        self.state = State::Fetch;
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    #[cfg(feature="events")]
    pub fn set_event_sender(&mut self, sender: Sender<events::CpuEvent>) {
        self.logger = Some(sender);
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    /// Determine if in an infinite loop state
    pub fn is_holding(&self) -> bool {
        self.is_holding
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        self.read_u8(addr)
    }

    /// Execute the current cycle given the internal state
    fn run_cycle(&mut self, state: State) -> State {
        match state {
            State::Reset => {
                // Read the PC address from the RESET vector
                self.pc = self.read_u16(memorymap::RESET_VECTOR);
                State::Fetch
            },
            State::Fetch => {
                if let Some(int_type) = self.interrupted {
                    self.interrupt(int_type);
                    State::Fetch
                }
                else {
                    let opcode = self.fetch();
                    self.get_execute_state(opcode)
                }
            },
            State::Execute(ref instr, ref mode, ref opcode_data, ref cycle) => {

                let total_cycles = cycle_count(*instr, *mode);

                if *cycle < total_cycles-1 {
                    return State::Execute(*instr, *mode, *opcode_data, *cycle + 1);
                }

                let operand_data = &opcode_data[1..];

                if self.debug {
                    #[cfg(feature = "events")]
                    {
                        let data = events::InstructionData {
                            instr: instr.clone(),
                            mode: mode.clone(),
                            opcode_data: opcode_data.clone(),
                            addr: self.pc - (mode.operand_len() + 1) as u16,
                            a: self.a,
                            x: self.x,
                            y: self.y,
                            p: self.p,
                            pc: self.pc,
                            sp: self.sp,
                        };

                        if let Some(ref logger) = self.logger {
                            match logger.send(events::CpuEvent::Instruction(data)) {
                                Ok(_) => {},
                                Err(_) => self.logger = None,
                            }
                        }
                    }
                }
                
                // Apply addressing mode
                let addressing_result = match mode {
                    AddressingMode::Immediate       => self.immediate(operand_data),
                    AddressingMode::ZeroPage        => self.zeropage(operand_data),
                    AddressingMode::ZeroPageX       => self.zeropage_x(operand_data),
                    AddressingMode::ZeroPageY       => self.zeropage_y(operand_data),
                    AddressingMode::Absolute        => self.absolute(operand_data),
                    AddressingMode::AbsoluteX       => self.absolute_x(operand_data),
                    AddressingMode::AbsoluteY       => self.absolute_y(operand_data),
                    AddressingMode::IndexedIndirect => self.indexed_indirect(operand_data),
                    AddressingMode::IndirectIndexed => self.indirect_indexed(operand_data),
                    AddressingMode::Indirect        => self.indirect(operand_data),
                    AddressingMode::Relative        => self.relative(operand_data),
                    AddressingMode::Accumulator     => self.accumulator(),
                    AddressingMode::Implied         => AddressingModeResult::Implied,
                };

                let read_mem = |addr: u16| -> u8 {
                    self.read_u8(addr)
                };

                match instr {
                    Instruction::NOP => {},
                    Instruction::CLC => self.clc(),
                    Instruction::CLD => self.cld(),
                    Instruction::CLI => self.cli(),
                    Instruction::CLV => self.clv(),
                    Instruction::DEX => self.dex(),
                    Instruction::DEY => self.dey(),
                    Instruction::INX => self.inx(),
                    Instruction::INY => self.iny(),
                    Instruction::PHA => self.pha(),
                    Instruction::PHP => self.php(),
                    Instruction::PLA => self.pla(),
                    Instruction::PLP => self.plp(),
                    Instruction::RTI => self.rti(),
                    Instruction::RTS => self.rts(),
                    Instruction::SEC => self.sec(),
                    Instruction::SED => self.sed(),
                    Instruction::SEI => self.sei(),
                    Instruction::TAX => self.tax(),
                    Instruction::TAY => self.tay(),
                    Instruction::TSX => self.tsx(),
                    Instruction::TXA => self.txa(),
                    Instruction::TXS => self.txs(),
                    Instruction::TYA => self.tya(),
                    Instruction::BRK => self.brk(),
                    Instruction::ALR => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.alr(byte.unwrap());
                    },
                    Instruction::ARR => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.arr(byte.unwrap());
                    },
                    Instruction::AXS => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.axs(byte.unwrap());
                    },
                    Instruction::LDA => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.lda(byte.unwrap())
                    },
                    Instruction::LAX => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.lax(byte.unwrap())
                    },
                    Instruction::JMP => {
                        let addr = addressing_result.to_address();
                        self.jmp(addr.unwrap())
                    },
                    Instruction::ADC => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.adc(byte.unwrap())
                    },
                    Instruction::AND => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.and(byte.unwrap())
                    },
                    Instruction::ANC => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.anc(byte.unwrap())
                    }
                    Instruction::BCC => {
                        let offset = addressing_result.to_offset();
                        self.bcc(offset.unwrap())
                    },
                    Instruction::BCS => {
                        let offset = addressing_result.to_offset();
                        self.bcs(offset.unwrap())
                    },
                    Instruction::BEQ => {
                        let offset = addressing_result.to_offset();
                        self.beq(offset.unwrap())
                    },
                    Instruction::BNE => {
                        let offset = addressing_result.to_offset();
                        self.bne(offset.unwrap())
                    },
                    Instruction::BMI => {
                        let offset = addressing_result.to_offset();
                        self.bmi(offset.unwrap())
                    },
                    Instruction::BPL => {
                        let offset = addressing_result.to_offset();
                        self.bpl(offset.unwrap())
                    },
                    Instruction::BVC => {
                        let offset = addressing_result.to_offset();
                        self.bvc(offset.unwrap())
                    },
                    Instruction::BVS => {
                        let offset = addressing_result.to_offset();
                        self.bvs(offset.unwrap())
                    },
                    Instruction::BIT => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.bit(byte.unwrap())
                    },
                    Instruction::CMP => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.cmp(byte.unwrap())
                    },
                    Instruction::CPX => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.cpx(byte.unwrap())
                    },
                    Instruction::CPY => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.cpy(byte.unwrap())
                    },
                    Instruction::EOR => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.eor(byte.unwrap())
                    },
                    Instruction::LDX => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.ldx(byte.unwrap())
                    },
                    Instruction::LDY => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.ldy(byte.unwrap())
                    },
                    Instruction::SBC => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.sbc(byte.unwrap())
                    },
                    Instruction::ORA => {
                        let byte = addressing_result.to_byte(read_mem);
                        self.ora(byte.unwrap())
                    },
                    Instruction::JSR => {
                        let addr = addressing_result.to_address();
                        self.jsr(addr.unwrap())
                    },
                    Instruction::STA => {
                        let v = self.sta();
                        self.write_result(addressing_result, v);
                    },
                    Instruction::STX => {
                        let v = self.stx();
                        self.write_result(addressing_result, v);
                    },
                    Instruction::STY => {
                        let v = self.sty();
                        self.write_result(addressing_result, v);
                    },
                    Instruction::SAX => {
                        let v = self.sax();
                        self.write_result(addressing_result, v);
                    },
                    Instruction::SHY => {
                        let byte = high_byte!(addressing_result.to_address().unwrap()) as u8;
                        let v = self.shy(byte.wrapping_add(1));
                        self.write_result(addressing_result, v);
                    },
                    Instruction::SHX => {
                        let byte = high_byte!(addressing_result.to_address().unwrap()) as u8;
                        let v = self.shx(byte.wrapping_add(1));
                        self.write_result(addressing_result, v);
                    },
                    Instruction::ASL => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.asl(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::ROR => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.ror(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::ROL => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.rol(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::LSR => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.lsr(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::INC => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.inc(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::DEC => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.dec(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::DCP => {
                        let byte = addressing_result.to_byte(read_mem);
                        let v = self.dcp(byte.unwrap());
                        self.write_result(addressing_result, v);
                    },
                    Instruction::ISB => {
                        let byte = addressing_result.to_byte(read_mem);
                        let m = self.isb(byte.unwrap());
                        self.write_result(addressing_result, m);
                    },
                    Instruction::SLO => {
                        let byte = addressing_result.to_byte(read_mem);
                        let m = self.slo(byte.unwrap());
                        self.write_result(addressing_result, m);
                    },
                    Instruction::RLA => {
                        let byte = addressing_result.to_byte(read_mem);
                        let m = self.rla(byte.unwrap());
                        self.write_result(addressing_result, m);
                    },
                    Instruction::SRE => {
                        let byte = addressing_result.to_byte(read_mem);
                        let m = self.sre(byte.unwrap());
                        self.write_result(addressing_result, m);
                    },
                    Instruction::RRA => {
                        let byte = addressing_result.to_byte(read_mem);
                        let m = self.rra(byte.unwrap());
                        self.write_result(addressing_result, m);
                    }
                }

                State::Fetch
            },
        }
    }

    //------------------------------------------------------------------------------------------------------------------
    // Opcode Decoding
    //------------------------------------------------------------------------------------------------------------------

    /// Convert opcode into instruction and addressing mode and return an execute state
    fn get_execute_state(&mut self, opcode: u8) -> State {
        let (instr, mode) = match opcode {
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
                panic!("Invalid opcode: ${opcode} at ${addr}", opcode=format!("{:X}", opcode), addr=format!("{:04X}", self.pc-1));
            }
        };

        let operand_data = self.fetch_operand_data(mode.operand_len());
        let opcode_data: [u8; 3] = [opcode, operand_data[0], operand_data[1]];

        State::Execute(instr, mode, opcode_data, 0)
    }

    fn fetch_operand_data(&mut self, num_bytes: usize) -> [u8; 2] {
        let mut operand_data = [0u8; 2];

        for i in 0..num_bytes {
            operand_data[i] = self.read_next_u8();
        }

        operand_data
    }

    //------------------------------------------------------------------------------------------------------------------
    // Instruction Implementation
    //------------------------------------------------------------------------------------------------------------------

    /// Load Accumulator
    fn lda(&mut self, a: u8) {
        self.a = a;
        self.update_flags(self.a);
    }

    fn lax(&mut self, m: u8) {
        self.a = m;
        self.x = m;

        self.update_flags(self.a);
    }

    fn sax(&self) -> u8 {
        self.a & self.x
    }

    fn dcp(&mut self, m: u8) -> u8 {
        let m = m.wrapping_sub(1);
        self.cmp(m);

        m
    }

    /// Jump
    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// ADC - Add with Carry
    fn adc(&mut self, m: u8) {
        // A,Z,C,N = A+M+C
        let a = self.a as u16;
        let m = m as u16;
        let c = self.get_carry() as u16;

        let r = a + m + c;
        let is_carry = r > 0xFF;
    
        let sign_bit = bit_is_set!(r, 7);
        let v = bit_is_set!(a, 7) != sign_bit && bit_is_set!(m, 7) != sign_bit;

        self.a = (r & 0x0FF) as u8;

        self.update_flags_with_carry(self.a, is_carry);
        self.set_flag_bit(Flags::Overflow, v);
    }

    /// AND - Logical AND
    fn and(&mut self, m: u8) {
        // A,Z,N = A&M
        self.a &= m;

        self.update_flags(self.a);
    }

    /// ANC - AND, b7 -> C
    fn anc(&mut self, m: u8) {
        self.and(m);
        self.set_flag_bit(Flags::Carry, bit_is_set!(self.a, 7));
    }

    /// ASL - Arithmetic shift left
    fn asl(&mut self, m: u8) -> u8 {
        let c = bit_is_set!(m, 7);

        let r = m << 1;

        self.set_zero_flag(r);
        self.set_negative_flag(r);
        self.set_flag_bit(Flags::Carry, c);

        r
    }

    /// ALR
    fn alr(&mut self, m: u8) {
        self.and(m);
        self.a = self.lsr(self.a);
    }

    /// ARR
    fn arr(&mut self, m: u8) {
        self.and(m);

        let b6 = bit_as_value!(self.a, 6);
        let b7 = bit_as_value!(self.a, 7);

        let v = b6 ^ b7;

        let c = self.get_carry();
        self.a >>= 1;

        self.a |= c << 7;

        self.update_flags(self.a);
        self.set_flag_bit(Flags::Overflow, v != 0);
    }

    /// AXS - A & X - M -> X
    fn axs(&mut self, m: u8) {
        let r = self.a & self.x;
        let r = r.wrapping_sub(m);

        self.x = r;

        self.set_zero_flag(r);
    }

    fn sta(&mut self) -> u8 {
        self.a
    }

    fn bcc(&mut self, offset: u8) {
        self.branch(!self.get_flag_bit(Flags::Carry), offset);
    }

    fn bcs(&mut self, offset: u8) {
        self.branch(self.get_flag_bit(Flags::Carry), offset);
    }

    fn beq(&mut self, offset: u8) {
        self.branch(self.get_flag_bit(Flags::Zero), offset);
    }

    fn bne(&mut self, offset: u8) {
        self.branch(!self.get_flag_bit(Flags::Zero), offset);
    }

    fn bmi(&mut self, offset: u8) {
        self.branch(self.get_flag_bit(Flags::Negative), offset);
    }

    /// BPL - Branch if Positive
    fn bpl(&mut self, offset: u8) {
        self.branch(!self.get_flag_bit(Flags::Negative), offset);
    }

    fn bvc(&mut self, offset: u8) {
        self.branch(!self.get_flag_bit(Flags::Overflow), offset);
    }

    fn bvs(&mut self, offset: u8) {
        self.branch(self.get_flag_bit(Flags::Overflow), offset);
    }

    /// BIT - Bit Test
    fn bit(&mut self, m: u8) {
        let r = self.a & m;

        // Copy bit 6 to V flag, and bit 7 to N flag
        self.set_flag_bit(Flags::Overflow, bit_is_set!(m, 6));
        self.set_flag_bit(Flags::Negative, bit_is_set!(m, 7));

        self.set_flag_bit(Flags::Zero, r == 0);
    }

    fn clc(&mut self) {
        self.set_flag_bit(Flags::Carry, false);
    }

    fn cld(&mut self) {
        self.set_flag_bit(Flags::Decimal, false);
    }

    fn cli(&mut self) {
        self.set_flag_bit(Flags::InterruptDisable, false);
    }

    fn clv(&mut self) {
        self.set_flag_bit(Flags::Overflow, false);
    }

    fn cmp(&mut self, m: u8) {
        self.compare(self.a, m);
    }

    fn cpx(&mut self, m: u8) {
        self.compare(self.x, m);
    }

    fn cpy(&mut self, m: u8) {
        self.compare(self.y, m);
    }

    fn dec(&mut self, m: u8) -> u8 {
        self.decrement(m)
    }

    fn dex(&mut self) {
        self.x = self.decrement(self.x);
    }

    fn dey(&mut self) {
        self.y = self.decrement(self.y);
    }

    fn inc(&mut self, m: u8) -> u8 {
        self.increment(m)
    }

    fn inx(&mut self) {
        self.x = self.increment(self.x);
    }

    fn iny(&mut self) {
        self.y = self.increment(self.y);
    }

    fn eor(&mut self, m: u8) {
        self.a ^= m;
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    fn ldx(&mut self, m: u8) {
        self.x = m;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    fn ldy(&mut self, m: u8) {
        self.y = m;
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
    }

    fn pha(&mut self) {
        self.push(self.a);
    }

    fn php(&mut self) {
        // The value of $30 is OR'ed in the status register for the 'B' flag values
        // http://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        self.push(self.p | 0x30);
    }

    fn pla(&mut self) {
        self.a = self.pull();
        self.update_flags(self.a);
    }

    fn plp(&mut self) {
        self.p = self.pull();
    }

    fn lsr(&mut self, m: u8) -> u8 {
        let c = bit_is_set!(m, 0);

        let r = m >> 1;

        self.set_zero_flag(r);
        self.set_negative_flag(r);
        self.set_flag_bit(Flags::Carry, c);

        r
    }

    fn ora(&mut self, m: u8) {
        self.a |= m;

        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    fn ror(&mut self, m: u8) -> u8 {
        let current_carry = self.get_flag_bit(Flags::Carry);
        let new_carry = bit_is_set!(m, 0);

        let mut r = m >> 1;

        if current_carry {
            bit_set!(r, 7);
        }

        self.set_flag_bit(Flags::Carry, new_carry);
        self.set_zero_flag(r);
        self.set_negative_flag(r);

        r
    }

    fn rol(&mut self, m: u8) -> u8 {
        let current_carry = self.get_flag_bit(Flags::Carry);
        let new_carry = bit_is_set!(m, 7);

        let mut r = m << 1;

        if current_carry {
            bit_set!(r, 0);
        }

        self.set_flag_bit(Flags::Carry, new_carry);
        self.set_zero_flag(r);
        self.set_negative_flag(r);

        r
    }

    fn rla(&mut self, m: u8) -> u8 {
        let m = self.rol(m);
        self.a &= m;
        self.update_flags(self.a);
        m
    }

    fn rra(&mut self, m: u8) -> u8 {
        let m = self.ror(m);
        self.adc(m);

        m
    }

    fn sre(&mut self, m: u8) -> u8 {
        let m = self.lsr(m);
        self.eor(m);
        m
    }

    fn rti(&mut self) {
        self.p = self.pull();
        self.pc = self.pull16();

        self.set_flag_bit(Flags::InterruptDisable, false);
    }

    fn jsr(&mut self, addr: u16) {
        //     JSR
        //     #  address R/W description
        //    --- ------- --- -------------------------------------------------
        //     1    PC     R  fetch opcode, increment PC
        //     2    PC     R  fetch low address byte, increment PC
        //     3  $0100,S  R  internal operation (predecrement S?)
        //     4  $0100,S  W  push PCH on stack, decrement S
        //     5  $0100,S  W  push PCL on stack, decrement S
        //     6    PC     R  copy low address byte to PCL, fetch high address
        //                    byte to PCH
        self.push16(self.pc-1);
        self.pc = addr;
    }

    fn rts(&mut self) {
        //  RTS
        //  #  address R/W description
        // --- ------- --- -----------------------------------------------
        //  1    PC     R  fetch opcode, increment PC
        //  2    PC     R  read next instruction byte (and throw it away)
        //  3  $0100,S  R  increment S
        //  4  $0100,S  R  pull PCL from stack, increment S
        //  5  $0100,S  R  pull PCH from stack
        //  6    PC     R  increment PC
        self.pc = self.pull16();
        self.pc = self.pc.wrapping_add(1);
    }

    fn sbc(&mut self, m: u8) {
        let m = Wrapping(m as u16);
        let c = Wrapping(1u16) - Wrapping(self.get_carry() as u16);
        let a = Wrapping(self.a as u16);

        let r = a - m - c;

        // Carry set when result is [0, 255]
        let c = bit_is_clear!(r.0, 8);
        
        let sign_bit = bit_is_set!(r.0, 7);
        let v = bit_is_set!(a.0, 7) != sign_bit && bit_is_set!(!m.0, 7) != sign_bit;

        self.a = (r.0 & 0xFF) as u8;
        self.set_flag_bit(Flags::Carry, c);
        self.set_flag_bit(Flags::Overflow, v);
        self.update_flags(self.a);
    }

    /// Increase memory by one and subtract from the accumulator with borrow
    fn isb(&mut self, m: u8) -> u8 {
        let m = m.wrapping_add(1);
        self.sbc(m);

        m
    }

    fn slo(&mut self, m: u8) -> u8 {
        let m = self.asl(m);
        self.ora(m);

        m
    }

    fn sec(&mut self) {
        self.set_flag_bit(Flags::Carry, true);
    }

    fn sed(&mut self) {
        self.set_flag_bit(Flags::Decimal, true);
    }

    fn sei(&mut self) {
        self.set_flag_bit(Flags::InterruptDisable, true);
    }

    fn shy(&self, m: u8) -> u8 {
        self.y & m
    }

    fn shx(&self, m: u8) -> u8 {
        self.x & m
    }

    fn stx(&mut self) -> u8 {
        self.x
    }

    fn sty(&mut self) -> u8 {
        self.y
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.set_zero_flag(self.y);
        self.set_negative_flag(self.y);
    }

    fn tsx(&mut self) {
        self.x = self.sp as u8;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.set_negative_flag(self.a);
        self.set_zero_flag(self.a);
    }

    fn txs(&mut self) {
        self.sp = self.x
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.set_negative_flag(self.a);
        self.set_zero_flag(self.a);
    }

    fn brk(&mut self) {
        // Docs for BRK say to add 2 to the PC when it goes on the stack.
        // This is to skip the unused byte after the BRK
        // The PC is already in the right spot
        self.push16(self.pc);
        // OR with $30 to set the B flag
        self.push(self.p | 0x30);

        self.pc = self.read_u16(memorymap::IRQ_VECTOR);

        self.set_flag_bit(Flags::InterruptDisable, true);
    }

    //------------------------------------------------------------------------------------------------------------------
    // Addressing Modes
    //------------------------------------------------------------------------------------------------------------------

    /// Immediate Addressing.
    /// Put current PC value on the address bus
    fn immediate(&self, data: &[u8]) -> AddressingModeResult {
        AddressingModeResult::Byte(data[0])
    }

    fn accumulator(&self) -> AddressingModeResult {
        AddressingModeResult::Byte(self.a)
    }

    /// Absolute Addressing.
    /// Fetch the address to read from the next two bytes
    fn absolute(&mut self, data: &[u8]) -> AddressingModeResult {
        AddressingModeResult::Address(((data[1] as u16) << 8) | data[0] as u16)
    }

    /// Absolute Addressing Indexed by X
    fn absolute_x(&mut self, data: &[u8]) -> AddressingModeResult {
        self.absolute_i(data, self.x)
    }

    /// Absolute Addressing Indexed by Y
    fn absolute_y(&mut self, data: &[u8]) -> AddressingModeResult {
        self.absolute_i(data, self.y)
    }

    fn absolute_i(&mut self, data: &[u8], i: u8) -> AddressingModeResult {
        let addr = ((data[1] as u16) << 8) | data[0] as u16;
        let addr = (Wrapping(addr) + Wrapping(i as u16)).0;
        AddressingModeResult::Address(addr)
    }

    /// Zero Page Addressing
    /// Fetch the next byte and put it on the address bus
    fn zeropage(&mut self, data: &[u8]) -> AddressingModeResult {
        AddressingModeResult::Address(data[0] as u16)
    }

    /// Zero Page Index X Addressing.
    fn zeropage_x(&mut self, data: &[u8]) -> AddressingModeResult {
        self.zeropage_i(data, self.x)
    }

    /// Zero Page Index Y Addressing
    fn zeropage_y(&mut self, data: &[u8]) -> AddressingModeResult {
        self.zeropage_i(data, self.y)
    }

    fn zeropage_i(&mut self, data: &[u8], i: u8) -> AddressingModeResult {
        AddressingModeResult::Address(((data[0] as u16) + i as u16) & 0xFF)
    }

    /// Indexed Indirect Addressing
    fn indexed_indirect(&mut self, data: &[u8]) -> AddressingModeResult {
        let ptr = ((data[0] as u16) + (self.x as u16)) & 0xFF;

        let addr = self.indirect_read(ptr);

        AddressingModeResult::Address(addr)
    }

    /// Indirect Indexed Addressing
    fn indirect_indexed(&mut self, data: &[u8]) -> AddressingModeResult {
        let ptr = data[0] as u16;

        let addr = self.indirect_read(ptr);
        let addr = addr.wrapping_add(self.y as u16);

        AddressingModeResult::Address(addr)
    }

    /// Indirect
    /// Only applicable to JMP instruction
    fn indirect(&self, data: &[u8]) -> AddressingModeResult {
        let ptr = ((data[1] as u16) << 8) | data[0] as u16;

        let addr = self.indirect_read(ptr);

        AddressingModeResult::Address(addr)
    }

    fn indirect_read(&self, ptr: u16) -> u16 {
        // Note: The PCH will always be fetched from the same page
        // as PCL, i.e. page boundary crossing is not handled.
        let page = ptr & 0xFF00;
        let addr_lo = ptr;
        let addr_hi = page | ((ptr + 0x01) & 0x00FF);

        let lo = self.read_u8(addr_lo) as u16;
        let hi = self.read_u8(addr_hi) as u16;

        (hi << 8) | lo
    }

    fn relative(&mut self, data: &[u8]) -> AddressingModeResult {
        AddressingModeResult::Offset(data[0])
    }

    //------------------------------------------------------------------------------------------------------------------
    // Flags Register
    //------------------------------------------------------------------------------------------------------------------

    // fn interrupts_disabled(&self) -> bool {
    //     self.get_flag_bit(Flags::InterruptDisable)
    // }

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
    fn branch(&mut self, cond_met: bool, offset: u8) {
        if cond_met {
            let offset = offset as i8;
            let offset = offset as i16;
            let base_addr = self.pc as i16;

            self.pc = (Wrapping(base_addr) + Wrapping(offset)).0 as u16;
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
    fn push(&mut self, data: u8) {
        // The stack is always stored in page 1
        self.write_u8((self.sp as u16) + STACK_PAGE_OFFSET, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pull a value off the stack
    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let data = self.read_u8((self.sp as u16) + STACK_PAGE_OFFSET);

        data
    }

    fn push16(&mut self, data: u16) {
        let hi = high_byte!(data) as u8;
        let lo = low_byte!(data) as u8;

        self.push(hi);
        self.push(lo);
    }

    fn pull16(&mut self) -> u16 {
        let lo = self.pull() as u16;
        let hi = self.pull() as u16;

        (hi << 8) | lo
    }

    //------------------------------------------------------------------------------------------------------------------
    // Base CPU Read/Write Operations
    //------------------------------------------------------------------------------------------------------------------

    /// Fetch the next opcode and increment the program counter
    fn fetch(&mut self) -> u8 {
        self.read_next_u8()
    }

    fn write_result(&mut self, addressing_result: AddressingModeResult, value: u8) {
        let mode = match self.state {
            State::Execute(_, mode, _, _) => mode,
            _ => panic!("Must be in execution state!"),
        };

        if mode == AddressingMode::Accumulator {
            self.a = value;
        }
        else {
            self.write_u8(addressing_result.to_address().unwrap(), value);
        }
    }

    fn read_next_u8(&mut self) -> u8 {
        let byte = self.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);

        byte
    }

    fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read_u8(addr) as u16;
        let hi = self.read_u8(addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_u8(&self, addr: u16) -> u8 {
        if let Some(ref bus) = self.bus {
            bus.read_byte(addr)
        }
        else {
            panic!("Attempt to read while bus is not loaded");
        }
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        if let Some(ref mut bus) = self.bus {
            bus.write_byte(addr, value);
        }
    }

    fn interrupt(&mut self, int_type: Interrupt) {
        self.interrupted = None;

        self.push16(self.pc);
        self.push(self.p);

        // Load the vector for the specified interrupt
        self.pc = match int_type {
            Interrupt::Nmi => self.read_u16(memorymap::NMI_VECTOR),
            // Interrupt::Irq => self.read_u16(memorymap::IRQ_VECTOR),
        };

        self.set_flag_bit(Flags::InterruptDisable, true);
    }
}

impl<Io: IoAccess> IoAccess for Cpu<Io> {
    fn raise_interrupt(&mut self, interrupt_type: Interrupt) {
        if self.interrupted.is_none() {
            self.interrupted = Some(interrupt_type);
        }
    }
}

impl<Io: IoAccess> Clockable for Cpu<Io> {
    /// Execute one CPU cycle
    fn tick(&mut self) {
        // Get the current PC
        let prev_pc = self.pc;
        // Implement one cycle of the CPU using a state machine
        // Execute the cycle based on the current CPU state and return the next CPU state
        self.state = self.run_cycle(self.state);
        // Is the PC pointing at the same location?
        self.is_holding = prev_pc == self.pc;
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
        let mut cpu = init_cpu(vec![]);
        cpu.pc = 0x0001;

        simple_test_base(&mut cpu, 0);

        assert_eq!(cpu.pc, 0x4020);
    }

    #[test]
    fn nop() {
        let prg = vec![0xEA];
        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.pc, 0x4021);
    }

    #[test]
    fn lda_immediate() {
        let prg = vec![
            0xA9, 0xA5 // LDA $A5
        ];

        let cpu = simple_test(prg, 2);

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
        let prg = vec![
            0xA5, 0x02, // LDA ($02)
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0xDE);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_zeropage_x() {
        let prg = vec![
            0xB5, 0x02, // LDA $02, X
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x03, 0xDE);
        cpu.x = 0x0001;

        simple_test_base(&mut cpu, 4);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_x() {
        let prg = vec![
            0xBD, 0x23, 0x40, // LDA $0003, X
            0x00, 0xDE,       // Data: $DE
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0x0001;

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_absolute_y() {
        let prg = vec![
            0xB9, 0x23, 0x40, // LDA $4023, Y
            0x00, 0xDE,       // Data: $DE
        ];

        let mut cpu = init_cpu(prg);
        cpu.y = 0x0001;

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indexed_indirect() {
        let prg = vec![
            0xA1, 0x02, // LDA ($0002, X)
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x03, 0x05);
        cpu.write_u8(0x04, 0x00);
        cpu.write_u8(0x05, 0xDE);

        cpu.x = 0x0001;

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_indirect_indexed() {
        let prg = vec![
            0xB1, 0x02, // LDA ($0002), Y
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0x05);
        cpu.write_u8(0x03, 0x00);
        cpu.write_u8(0x06, 0xDE);

        cpu.y = 0x0001;

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.a, 0xDE);
    }

    #[test]
    fn lda_flags_zero() {
        let prg = vec![
            0xA9, 0x00 // LDA $00
        ];

        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.p, Flags::Zero as u8);
    }

    #[test]
    fn lda_flags_negative() {
        let prg = vec![
            0xA9, 0x80 // LDA $00
        ];

        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.p, Flags::Negative as u8);
    }

    #[test]
    fn lax() {
        let prg = vec![
            0xA7, 0x02, // LAX $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0xDE);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0xDE);
        assert_eq!(cpu.x, 0xDE);
    }

    #[test]
    fn sax() {
        let prg = vec![
            0x87, 0x02, // SAX $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;
        cpu.x = 0x00;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.read_u8(0x02), 0x00);
    }

    #[test]
    fn dcp() {
        let prg = vec![
            0xC7, 0x02, // DCP $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0x01);

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.read_u8(0x02), 0x00);
    }

    #[test]
    fn jmp_absolute() {
        let prg = vec![
            0x4C, 0x00, 0x10 // JMP $1000
        ];

       let cpu = simple_test(prg, 3);

        assert_eq!(cpu.pc, 0x1000);
    }

    #[test]
    fn jmp_indirect() {
        let prg = vec![
            0x6C, 0x23, 0x40, // LDA JMP ($0003)
            0x00, 0x10,       // Address: $1000
        ];

        let cpu = simple_test(prg, 5);

        assert_eq!(cpu.pc, 0x1000);
    }

    #[test]
    fn jmp_indirect_page_cross() {
        let prg = vec![
            0x6C, 0xFF, 0x00, // JMP ($00FF)
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x0FF, 0xAD);
        cpu.write_u8(0x00, 0xDE);

        simple_test_base(&mut cpu, 5);
        
        assert_eq!(cpu.pc, 0xDEAD);
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
        let prg = vec![
            0x69, 0x01, // ADC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(mask_is_set!(cpu.p, Flags::Carry as u8), true);
    }

    #[test]
    fn adc_immediate_with_carry_set() {
        let prg = vec![
            0x69, 0xFF, // ADC $FF; a=$0  -> a=$FF
            0x69, 0x01, // ADC $01; a=$FF -> a=00, c=1
            0x69, 0x00, // ADC $00; a=$00 -> a=$01, c=0
        ];

        let mut cpu = init_cpu(prg);

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.a, 0x01);
        assert_eq!(mask_is_clear!(cpu.p, Flags::Carry as u8), true);
    }

    #[test]
    fn adc_overflow_1() {
        let prg = vec![
            0x69, 0x01, // ADC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x02);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), false);
    }

    #[test]
    fn adc_overflow_2() {
        let prg = vec![
            0x69, 0xFF, // ADC $FF
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), false);
    }

    #[test]
    fn adc_overflow_3() {
        let prg = vec![
            0x69, 0x01, // ADC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x7F;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), true);
    }

    #[test]
    fn adc_overflow_4() {
        let prg = vec![
            0x69, 0xFF, // ADC $FF
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x80;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x7F);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), true);
    }

    #[test]
    fn and_immediate() {
        let prg = vec![
            0x29, 0xF0, // AND $F0
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0xF0);
        assert_eq!(mask_is_set!(cpu.p, Flags::Negative as u8), true);
    }

    #[test]
    fn and_immediate_zero_set() {
        let prg = vec![
            0x29, 0x00, // AND $F0
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;

        simple_test_base(&mut cpu, 3);

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

        let cpu = simple_test(prg, 4);

        assert_eq!(cpu.a, 0x01);
        assert_eq!(mask_is_clear!(cpu.p, Flags::Zero as u8), true);
    }

    #[test]
    fn asl_accumulator() {
        let prg = vec![
            0x0A, // ASL
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x02);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), false);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), false);
    }

    #[test]
    fn asl_accumulator_is_zero() {
        let prg = vec![
            0x0A, // ASL
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x80;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), false);
    }

    #[test]
    fn asl_accumulator_carry_and_negative_set() {
        let prg = vec![
            0x0A, // ASL
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xC0;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), true);
    }

    #[test]
    fn sta_zeropage() {
        let prg = vec![
            0x85, 0x05, // STA $05
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xDE;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.read_u8(0x05), 0xDE);
    }

    #[test]
    fn bcc_carry_not_set() {
        let prg = vec![
            0x90, 0x02, // BCC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Carry as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bcc_carry_set() {
        let prg = vec![
            0x90, 0x02, // BCC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Carry as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bcs_carry_not_set() {
        let prg = vec![
            0xB0, 0x02, // BSC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Carry as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bcs_carry_set() {
        let prg = vec![
            0xB0, 0x02, // BSC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Carry as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn beq_zero_not_set() {
        let prg = vec![
            0xF0, 0x02, // BEQ $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Zero as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn beq_zero_set() {
        let prg = vec![
            0xF0, 0x02, // BEQ $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Zero as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bne_zero_not_set() {
        let prg = vec![
            0xD0, 0x02, // BNE $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Zero as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bne_zero_not_set_negative_offset() {
        let prg = vec![
            0xD0, 0xFE, // BNE $FE
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Zero as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4020);
    }

    #[test]
    fn bne_zero_set() {
        let prg = vec![
            0xD0, 0x02, // BNE $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Zero as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bmi_negative_not_set() {
        let prg = vec![
            0x30, 0x02, // BMI $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Negative as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bmi_negative_set() {
        let prg = vec![
            0x30, 0x02, // BMI $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Negative as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bpl_negative_not_set() {
        let prg = vec![
            0x10, 0x02, // BPL $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Negative as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bpl_negative_set() {
        let prg = vec![
            0x10, 0x02, // BPL $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Negative as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bvc_overflow_not_set() {
        let prg = vec![
            0x50, 0x02, // BVC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bvc_overflow_set() {
        let prg = vec![
            0x50, 0x02, // BVC $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bvs_overflow_not_set() {
        let prg = vec![
            0x70, 0x02, // BVS $02
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4022);
    }

    #[test]
    fn bvs_overflow_set() {
        let prg = vec![
            0x70, 0x02, // BVS $02
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.pc, 0x4024);
    }

    #[test]
    fn bit_check_mask() {
        let prg = vec![
            0x24, 0x02, // BIT $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;
        cpu.write_u8(0x02, 0x01);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
    }

    #[test]
    fn bit_check_mask_not_set() {
        let prg = vec![
            0x24, 0x02, // BIT $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;
        cpu.write_u8(0x02, 0x00);

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn bit_check_mask_absolute() {
        let prg = vec![
            0x2C, 0x02, 0x00, // BIT $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;
        cpu.write_u8(0x02, 0x01);

        simple_test_base(&mut cpu, 4);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), false);
    }

    #[test]
    fn clear_flags_instructions() {
        let prg = vec![
            0x18, // CLC
            0xD8, // CLD
            0x58, // CLI
            0xB8, // CLV
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Carry as u8);
        mask_set!(cpu.p, Flags::Decimal as u8);
        mask_set!(cpu.p, Flags::InterruptDisable as u8);
        mask_set!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 8);

        assert_eq!(cpu.get_flag_bit(Flags::Carry), false);
        assert_eq!(cpu.get_flag_bit(Flags::Decimal), false);
        assert_eq!(cpu.get_flag_bit(Flags::InterruptDisable), false);
        assert_eq!(cpu.get_flag_bit(Flags::Overflow), false);
    }

    #[test]
    fn set_flags_instructions() {
        let prg = vec![
            0x38, // SEC
            0xF8, // SED
            0x78, // SEI
        ];

        let mut cpu = init_cpu(prg);
        mask_clear!(cpu.p, Flags::Carry as u8);
        mask_clear!(cpu.p, Flags::Decimal as u8);
        mask_clear!(cpu.p, Flags::InterruptDisable as u8);
        mask_clear!(cpu.p, Flags::Overflow as u8);

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
        assert_eq!(cpu.get_flag_bit(Flags::Decimal), true);
        assert_eq!(cpu.get_flag_bit(Flags::InterruptDisable), true);
    }

    #[test]
    fn cmp_zero_set() {
        let prg = vec![
            0xC9, 0x02, // CMP $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x02;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn cmp_negative_set() {
        let prg = vec![
            0xC9, 0x01, // CMP $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x00;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.get_flag_bit(Flags::Negative), true);
    }

    #[test]
    fn dec_mem() {
        let prg = vec![
            0xC6, 0x02, // DEC $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0x01);

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.read_u8(0x02), 0x00);
    }

    #[test]
    fn dex() {
        let prg = vec![
            0xCA, // DEX
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn dey() {
        let prg = vec![
            0x88, // DEY
        ];

        let mut cpu = init_cpu(prg);
        cpu.y = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn inc_mem() {
        let prg = vec![
            0xE6, 0x02, // INC $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x02, 0xFF);

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.read_u8(0x02), 0x00);
    }

    #[test]
    fn inx() {
        let prg = vec![
            0xE8, // INX
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0xFF;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn iny() {
        let prg = vec![
            0xC8, // INY
        ];

        let mut cpu = init_cpu(prg);
        cpu.y = 0xFF;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn eor() {
        let prg = vec![
            0x49, 0xFF, // EOR $FF
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
    }

    #[test]
    fn ldx_immediate() {
        let prg = vec![
            0xA2, 0xA5 // LDX $A5
        ];

        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.x, 0xA5);
    }

    #[test]
    fn ldx_absolute() {
        let prg = vec![
            0xAE, 0xFF, 0x07, // LDX $07FF
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x07FF, 0xA5);

        simple_test_base(&mut cpu, 4);

        assert_eq!(cpu.x, 0xA5);
    }

    #[test]
    fn ldy_immediate() {
        let prg = vec![
            0xA0, 0xA5 // LDY $A5
        ];

        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.y, 0xA5);
    }

    #[test]
    fn pha() {
        let prg = vec![
            0x48, // PHA
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xFF;
        cpu.sp = 0x0A;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.sp, 0x09);
        assert_eq!(cpu.read_u8(0x10A), 0xFF);
    }

    #[test]
    fn pla() {
        let prg = vec![
            0x68,       // PLA
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x10A, 0xFF);
        cpu.sp = 0x09;

        simple_test_base(&mut cpu, 4);

        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.sp, 0x0A);
    }


    #[test]
    fn php() {
        let prg = vec![
            0x08, // PHP
        ];

        let mut cpu = init_cpu(prg);
        cpu.p = 0xFF;
        cpu.sp = 0x0A;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.sp, 0x09);
        assert_eq!(cpu.read_u8(0x10A), 0xFF);
    }

    #[test]
    fn plp() {
        let prg = vec![
            0x28, // PLP
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x10A, 0xFF);
        cpu.sp = 0x09;

        simple_test_base(&mut cpu, 4);

        assert_eq!(cpu.p, 0xFF);
        assert_eq!(cpu.sp, 0x0A);
    }

    #[test]
    fn lsr_zero_set() {
        let prg = vec![
            0x4A, // LSR A
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.get_flag_bit(Flags::Zero), true);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
    }

    #[test]
    fn ora() {
        let prg = vec![
            0x09, 0x0F, // ORA $0F
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xF0;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0xFF);
    }

    #[test]
    fn ror_carry() {
        let prg = vec![
            0x6A, // ROR A
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Carry as u8);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
        assert_eq!(cpu.get_flag_bit(Flags::Negative), true);
    }

    #[test]
    fn rol_carry() {
        let prg = vec![
            0x2A, // ROL A
        ];

        let mut cpu = init_cpu(prg);
        mask_set!(cpu.p, Flags::Carry as u8);
        cpu.a = 0x81;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x03);
        assert_eq!(cpu.get_flag_bit(Flags::Carry), true);
    }

    #[test]
    fn rti() {
        let prg = vec![
            0x40, // RTI
        ];

        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x10A, 0xDE);
        cpu.write_u8(0x109, 0xAD);
        cpu.write_u8(0x108, 0xA1);
        cpu.sp = 0x0007;

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.p, 0xA1);
        assert_eq!(cpu.sp, 0x0A);
        assert_eq!(cpu.pc, 0xDEAD);
    }

    #[test]
    fn jsr() {
        let prg = vec![
            0x20, 0xAD, 0xDE, // JSR $DEAD
        ];

        let mut cpu = init_cpu(prg);
        cpu.sp = 0x000A;

        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.pc, 0xDEAD);
        assert_eq!(cpu.read_u8(0x010A), 0x40);
        assert_eq!(cpu.read_u8(0x0109), 0x22);
    }

    #[test]
    fn sbc() {
        let prg = vec![
            0xE9, 0x01, // SBC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0xFF);
    }

    #[test]
    fn sbc2() {
        let prg = vec![
            0xE9, 0x40, // SBC $40
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x40;
        mask_set!(cpu.p, Flags::Carry as u8);

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(mask_is_set!(cpu.p, Flags::Carry as u8), true);
        assert_eq!(mask_is_set!(cpu.p, Flags::Zero as u8), true);
        assert_eq!(mask_is_set!(cpu.p, Flags::Overflow as u8), false);
        assert_eq!(mask_is_set!(cpu.p, Flags::Negative as u8), false);
    }

    #[test]
    fn sbc_overflow_1() {
        let prg = vec![
            0xE9, 0x01, // SBC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x00;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(mask_is_set!(cpu.p, Flags::Overflow as u8), false);
    }

    #[test]
    fn sbc_overflow_2() {
        let prg = vec![
            0xE9, 0x01, // SBC $01
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x80;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0x7E);
        assert_eq!(mask_is_set!(cpu.p, Flags::Overflow as u8), true);
    }

    #[test]
    fn isb() {
        let prg = vec![
            0xE7, 0x02, // ISB $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0x01;
        cpu.write_u8(0x02, 0x00);

        simple_test_base(&mut cpu, 5);

        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.read_u8(0x02), 0x01);
    }

    #[test]
    fn rts() {
        let prg = vec![
            0x60, // RTS
        ];
        
        let mut cpu = init_cpu(prg);
        cpu.write_u8(0x10A, 0xDE);
        cpu.write_u8(0x109, 0xAC);
        cpu.sp = 0x0008;


        simple_test_base(&mut cpu, 6);

        assert_eq!(cpu.sp, 0x0A);
        assert_eq!(cpu.pc, 0xDEAD);
    }

    #[test]
    fn stx() {
        let prg = vec![
            0x86, 0x02, // STX $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0xA5;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.read_u8(0x02), 0xA5);
    }

    #[test]
    fn tax() {
        let prg = vec![
            0xAA, // TAX
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.x, 0xA5);
    }

    
    #[test]
    fn tay() {
        let prg = vec![
            0xA8, // TAY
        ];

        let mut cpu = init_cpu(prg);
        cpu.a = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.y, 0xA5);
    }

    
    #[test]
    fn tsx() {
        let prg = vec![
            0xBA, // TSX
        ];

        let mut cpu = init_cpu(prg);
        cpu.sp = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.x, 0xA5);
    }

    #[test]
    fn txa() {
        let prg = vec![
            0x8A, // TXA
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0xA5);
    }

    #[test]
    fn txs() {
        let prg = vec![
            0x9A, // TXS
        ];

        let mut cpu = init_cpu(prg);
        cpu.x = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.sp, 0xA5);
    }

    #[test]
    fn tya() {
        let prg = vec![
            0x98, // TYA
        ];

        let mut cpu = init_cpu(prg);
        cpu.y = 0xA5;

        simple_test_base(&mut cpu, 2);

        assert_eq!(cpu.a, 0xA5);
    }

    #[test]
    fn sty() {
        let prg = vec![
            0x84, 0x02, // STY $02
        ];

        let mut cpu = init_cpu(prg);
        cpu.y = 0xA5;

        simple_test_base(&mut cpu, 3);

        assert_eq!(cpu.read_u8(0x02), 0xA5);
    }

    #[test]
    fn brk() {
        // TODO: Test the BRK instruction
    }

    #[test]
    fn is_holding() {
        let prg = vec![
            0x4C, 0x20, 0x40, // JMP $4020; Infinite loop
        ];

        let cpu = simple_test(prg, 2);

        assert_eq!(cpu.is_holding(), true);
    }

    #[test]
    fn enter_interrupt() {
        // TODO: Interrupt test
    }

    ///-----------------------------------------------------------------------------------------------------------------
    /// Helper functions
    ///-----------------------------------------------------------------------------------------------------------------
    mod helper {
        use super::*;

        pub struct FakeBus {
            memmap: Vec<u8>, // ROM
        }

        impl Default for FakeBus {
            fn default() -> Self {
                FakeBus {
                    memmap: vec![],
                }
            }
        }

        impl FakeBus {
            pub fn from(prg_rom: Vec<u8>) -> Self {
                let mut rom = vec![0x00; 0x10000];
                // insert prg memory
                for (i, byte) in prg_rom.iter().enumerate() {
                    rom[0x4020 + i] = *byte;
                }

                // insert reset vector
                rom[0xFFFC] = 0x20;
                rom[0xFFFD] = 0x40;

                // insert NMI interrupt vector
                rom[0xFFFA] = 0x20;
                rom[0xFFFB] = 0x40;

                FakeBus {
                    memmap: rom,
                }
            }
        }

        impl IoAccess for FakeBus {
            fn read_byte(&self, addr: u16) -> u8 {
                self.memmap[addr as usize]
            }

            fn write_byte(&mut self, addr: u16, data: u8) {
                self.memmap[addr as usize] = data;
            }
        }

        pub fn simple_test<'a>(prg: Vec<u8>, ticks: usize) -> Cpu<FakeBus> {
            let mut cpu = init_cpu(prg);
            cpu.p = 0x00;

            simple_test_base(&mut cpu, ticks);

            cpu
        }

        pub fn simple_test_base(cpu: &mut Cpu<FakeBus>, ticks: usize) {
            run_cpu(cpu, ticks);
        }

        pub fn init_cpu(prg: Vec<u8>) -> Cpu<FakeBus> {
            let bus = FakeBus::from(prg);
            let mut cpu: Cpu<FakeBus> = Cpu::default();
            cpu.load_bus(bus);
            cpu
        }

        pub fn run_cpu(cpu: &mut Cpu<FakeBus>, ticks: usize) {
            // Tick CPU once to exit Reset state
            cpu.tick();

            // Tick CPU the expect number of times
            for _ in 0..ticks {
                cpu.tick();
            }
        }
    }

}
