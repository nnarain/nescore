//
// branching.rs
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
