use nescore::{Nes, Cartridge};

#[test]
fn branch_timing_branch_basics() {
    let cart = Cartridge::from_path("tests/roms/nes-test-roms/branch_timing_tests/1.Branch_Basics.nes").unwrap();
    let mut nes = Nes::new().with_cart(cart);

    while !nes.is_holding() {
        nes.emulate_frame();
    }

    // TODO: Requires NMI
    // According to branch_timing_tests/validation.a: Results are store in $F8
    // let result = nes.read_cpu_ram(0xF8);
    // assert_eq!(result, 1, "Branch Basics returned result {:02X}", result);
}