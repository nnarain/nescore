use nescore::{Nes, Cartridge};

#[test]
fn nestest() {
    let cart = Cartridge::from_path("tests/roms/nestest.nes").unwrap();

    // Set the CPU entry point to $C000 for nestest "automation" mode
    let mut nes = Nes::new().with_cart(cart).entry(0xC000);

    // Run a few frames first...
    for _ in 0..10 {
        nes.emulate_frame();
    }

    while !nestest_complete(&nes) {
        nes.emulate_frame();
    }

    let error_code = nes.read_cpu_ram(0x02);
    assert_eq!(error_code, 0x00, "nestest exited with code {}", error_code);
}

fn nestest_complete(nes: &Nes) -> bool {
    // nestest reports error codes for each test it runs at address $0002
    // A value of $00, mean no error. There is not a flag for "all tests complete"
    // To verify the test is complete, VRAM will have to be checked to see what characters are used in the output message to the user
    // The text "~~ Run all tests" appears before the test is run
    // With either "Ok" or "Er" replacing the "~~" after the tests are complete
    // Therefore, if the "~~" is not present the test is over
    // The tile address is $2084, and the tile number is $2D
    nes.read_ppu_memory(0x2084) != 0x2D
}
