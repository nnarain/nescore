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
    common::run_test(&mut nes, 6, "Basic test failed with");
}

#[test]
fn sprite_zero_alignment() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/02.alignment.nes");
    common::run_test(&mut nes, 6, "Alignment test failed with");
}

#[test]
// #[ignore]
fn sprite_zero_corners() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/03.corners.nes");
    common::run_test(&mut nes, 6, "Corner test failed with");
}

#[test]
fn sprite_zero_flip() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/04.flip.nes");
    common::run_test(&mut nes, 6, "Flip test failed with");
}

#[test]
fn sprite_zero_left_clip() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/05.left_clip.nes");
    common::run_test(&mut nes, 6, "Left clip test failed with");
}

#[test]
#[ignore]
fn sprite_zero_right_edge() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/06.right_edge.nes");
    common::run_test(&mut nes, 6, "Right edge test failed with");
}

#[test]
#[ignore]
fn sprite_zero_screen_bottom() {
    // FIXME: I suspect this fails because the instruction and branch timing is off
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/07.screen_bottom.nes");
    common::run_test(&mut nes, 6, "Screen bottom test failed with");
}

#[test]
fn sprite_zero_double_height() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/08.double_height.nes");
    common::run_test(&mut nes, 6, "Double height test failed with");
}

#[test]
#[ignore]
fn sprite_zero_timing_basics() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/sprite_hit_tests_2005.10.05/09.timing_basics.nes");
    common::run_test(&mut nes, 6, "Timing basics test failed with");
}
