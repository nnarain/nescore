//
// timing.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 16 2020
//
mod common;

#[test]
#[ignore]
fn branch_timing_branch_basics() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/branch_timing_tests/1.Branch_Basics.nes");
    common::run_test(&mut nes, "Branch timing basic test failed with");
}

#[test]
#[ignore]
fn branch_timing_branch_backward() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/branch_timing_tests/2.Backward_Branch.nes");
    common::run_test(&mut nes, "Branch backward test failed with");
}

#[test]
#[ignore]
fn branch_timing_branch_forward() {
    let mut nes = common::init_nes("tests/roms/nes-test-roms/branch_timing_tests/3.Forward_Branch.nes");
    common::run_test(&mut nes, "Forward backward test failed with");
}
