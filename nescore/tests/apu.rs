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
#[ignore] // Failed because the first length clock is too soon
fn apu_len_table() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/apu_test/rom_singles/2-len_table.nes");
    common::run_test(&mut nes, "Length Counter test failed with");
}
