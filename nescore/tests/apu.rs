//
// apu.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 03 2020
//
mod common;

#[test]
fn apu_len_ctr() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/1-len_ctr.nes");
    common::run_test(&mut nes, "Length Counter test failed with");
}

#[test]
#[ignore = "Times out"]
fn apu_len_ctr_blargg() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/blargg_apu_2005.07.30/01.len_ctr.nes");
    common::run_test(&mut nes, "Length Counter test failed with");
}

#[test]
fn apu_len_table() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/2-len_table.nes");
    common::run_test(&mut nes, "Length Counter test failed with");
}

#[test]
fn apu_irq() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/3-irq_flag.nes");
    common::run_test(&mut nes, "IRQ Flag test failed with");
}

#[test]
#[ignore]
fn apu_len_timing() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/5-len_timing.nes");
    common::run_test(&mut nes, "Length timing test failed with");
}

#[test]
#[ignore]
fn apu_irq_timing() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/6-irq_flag_timing.nes");
    common::run_test(&mut nes, "Irq timing test failed with");
}
