//
// sprite_zero_hit.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 16 2020
//
mod common;

#[test]
fn sprite_zero_basics() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/01.basics.nes");
    common::run_test(&mut nes, "Basic test failed with");
}

#[test]
fn sprite_zero_alignment() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/02.alignment.nes");
    common::run_test(&mut nes, "Alignment test failed with");
}

#[test]
fn sprite_zero_corners() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/03.corners.nes");
    common::run_test(&mut nes, "Corner test failed with");
}
