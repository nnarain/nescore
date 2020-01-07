use nescore::{Nes, Cartridge};

#[test]
fn nestest() {
    let cart = Cartridge::from_path("tests/roms/nestest.nes").unwrap();

    // Set the CPU entry point to $C000 for nestest "automation" mode
    let mut nes = Nes::new().with_cart(cart).entry(0xC000);

    while !nestest_complete(&nes) {
        nes.emulate_frame();
    }

    let error_code = nes.read_cpu_ram(0x02);
    assert_eq!(error_code, 0x00, "nestest exited with code {}", error_code);
}

fn nestest_complete(nes: &Nes) -> bool {
    true
}
