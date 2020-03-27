//
// instructions.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 23 2020
//

mod common;
use nescore::{Nes, Cartridge};


#[test]
fn nestest() {
    let cart = Cartridge::from_path("tests/roms/nestest/nestest.nes").unwrap();

    // Set the CPU entry point to $C000 for nestest "automation" mode
    let mut nes = Nes::default().with_cart(cart).entry(0xC000).debug_mode(true);
    // According to nestest logs the test ends at $C66E
    nes.run_until(0xC66E);

    // Fetch error codes
    let official_opcode_result = nes.read_cpu_ram(0x02);
    let unofficial_opcode_result = nes.read_cpu_ram(0x03);

    assert_eq!(official_opcode_result, 0, "Official opcodes exited with code ${:02X}", official_opcode_result);
    assert_eq!(unofficial_opcode_result, 0, "Unofficial opcodes exited with code ${:02X}", unofficial_opcode_result);
}

#[test]
fn nes_instr_implied() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/01-implied.nes");
    common::run_test(&mut nes, "Implied instructions exited with");
}

#[test]
fn nes_instr_immediate() {
    // Fail on implementation for ARR and AXS instructions. Apparently these are sketchy
    // https://wiki.nesdev.com/w/index.php/Talk:Programming_with_unofficial_opcodes#ARR_and_AXS_instructions
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/02-immediate.nes");
    common::run_test_with_ignore(&mut nes, "Immediate instructions exited with", vec![String::from("arr"), String::from("axs")]);
}

#[test]
fn nes_instr_zeropage() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/03-zero_page.nes");
    common::run_test(&mut nes, "Zeropage instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_zp_xy() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/04-zp_xy.nes");
    common::run_test(&mut nes, "Zeropage XY instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_absolute() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/05-absolute.nes");
    common::run_test(&mut nes, "Absolute instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_abs_xy() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/06-abs_xy.nes");
    common::run_test(&mut nes, "Aboslute XY instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_ind_x() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/07-ind_x.nes");
    common::run_test(&mut nes, "Indirect X instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_ind_y() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/08-ind_y.nes");
    common::run_test(&mut nes, "Indirect Y instructions exited with");
}

#[test]
fn nes_instr_branches() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/09-branches.nes");
    common::run_test(&mut nes, "Branch instructions exited with");
}

#[test]
#[ignore]
fn nes_instr_stack() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/10-stack.nes");
    common::run_test(&mut nes, "Branch instructions exited with");
}
#[test]
#[ignore]
fn nes_instr_special() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/nes_instr_test/rom_singles/11-special.nes");
    common::run_test(&mut nes, "Branch instructions exited with");
}
