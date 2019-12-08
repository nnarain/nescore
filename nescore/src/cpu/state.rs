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

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Implied,
    Immediate,
    ZeroPage,
    Absolute,
    ZeroPageX,
    ZeroPageY,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed
}

impl AddressingMode {

}

#[derive(Copy, Clone)]
pub enum Instruction {
    NOP,
    LDA
}

#[derive(Copy, Clone)]
pub enum ExecuteSubState {
    AddressingMode,
    Instruction
}

#[derive(Copy, Clone)]
pub enum State {
    Reset,
    Fetch,
    Execute(Instruction, Option<AddressingMode>, u8, u8)
}
