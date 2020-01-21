use nescore::{Nes, Cartridge};

const TILE_NO_O: u8 = 0x4F;
const TILE_NO_K: u8 = 0x4B;

const TILE_NO_ZERO_OFFSET: u8 = 0x30;

const MAP_ADDRESS_FIRST_TEST_LOCATION: u16 = 0x20A4;
const MAP_ADDRESS_TILE_OFFSET: u16 = 0x20;

///
/// Runs the nestest ROM test
///
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

    for test in vec![0, 1, 2] {
        let error_code = get_test_status(&nes, test);
        assert_eq!(error_code, 0x00, "Error code ${:X}", error_code);
    }
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

fn get_test_status(nes: &Nes, test_number: usize) -> usize {
    // Determine the memory address of the first byte of the test result
    let test_result_location = MAP_ADDRESS_FIRST_TEST_LOCATION + (test_number as u16 * MAP_ADDRESS_TILE_OFFSET);

    // Read the test result bytes
    let byte1 = nes.read_ppu_memory(test_result_location);
    let byte2 = nes.read_ppu_memory(test_result_location + 1);

    // If the result is "OK", return Ok
    if (byte1, byte2) == (TILE_NO_O, TILE_NO_K) {
        0
    }
    else {
        // Otherwise, return the error code
        get_error_code(byte1, byte2)
    }
}

fn get_error_code(b1: u8, b2: u8) -> usize {
    use std::usize;
    let s = format!("{}{}", btos(b1), btos(b2));
    usize::from_str_radix(&s, 16).unwrap()
}

fn btos(b: u8) -> String {
    match b {
        0x30..=0x39 => {
            (b - 0x30).to_string()
        },
        // FIXME: Bad text converting...
        0x41 => String::from("A"),
        0x42 => String::from("B"),
        0x43 => String::from("C"),
        0x44 => String::from("D"),
        0x45 => String::from("E"),
        0x46 => String::from("F"),
        _ => panic!("Invalid input"),
    }
}
