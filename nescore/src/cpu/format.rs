//
// cpu/format.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jan 10 2020
//

use super::{Instruction, AddressingMode};

pub fn disassemble(instr: Instruction, mode: AddressingMode, data: &[u8]) -> String {
    // TODO: Fix up ASM syntax
    match mode {
        AddressingMode::Accumulator                          => format!("{:?} A       ", instr),
        AddressingMode::Implied                              => format!("{:?}         ", instr),
        AddressingMode::Immediate | AddressingMode::Relative => format!("{:?} {:02X}      ", instr, data[0]),
        AddressingMode::ZeroPage                             => format!("{:?} ({:02X})    ", instr, data[0]),
        AddressingMode::ZeroPageX                            => format!("{:?} ({:02X},X)  ", instr, data[0]),
        AddressingMode::ZeroPageY                            => format!("{:?} ({:02X},Y)  ", instr, data[0]),
        AddressingMode::Absolute                             => format!("{:?} {:04X}    ", instr, address(data)),
        AddressingMode::AbsoluteX                            => format!("{:?} ({:04X},X)  ", instr, address(data)),
        AddressingMode::AbsoluteY                            => format!("{:?} ({:04X},Y)", instr, address(data)),
        AddressingMode::Indirect                             => format!("{:?} ({:04X})  ", instr, address(data)),
        AddressingMode::IndexedIndirect                      => format!("{:?} ({:02X},X)  ", instr, data[0]),
        AddressingMode::IndirectIndexed                      => format!("{:?} ({:02X}),Y  ", instr, data[0]),
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
