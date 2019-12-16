//
// cpu/state.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 03 2019
//
pub trait AddressingModeData {
    fn value(&mut self) -> u8;
}

#[derive(Copy, Clone)]
struct AbsoluteAddressingMode;
impl AddressingModeData for AbsoluteAddressingMode {
    fn value(&mut self) -> u8 {
        0
    }
}

#[derive(Copy, Clone, PartialEq)]
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
    IndirectIndexed
}

#[derive(Copy, Clone)]
pub enum Instruction {
    NOP,
    LDA,
    JMP,
    ADC,
    AND,
}

#[derive(Copy, Clone)]
pub enum State {
    Reset,
    Fetch,
    Execute(Instruction, AddressingMode, u8)
}
