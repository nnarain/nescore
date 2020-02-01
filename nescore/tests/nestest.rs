use nescore::{Nes, Cartridge};

///
/// Runs the nestest ROM test
///
#[test]
fn nestest() {
    let cart = Cartridge::from_path("tests/roms/nestest/nestest.nes").unwrap();

    // Set the CPU entry point to $C000 for nestest "automation" mode
    let mut nes = Nes::new().with_cart(cart).entry(0xC000).debug_mode(true);
    // According to nestest logs the test ends at $C66E
    nes.run_until(0xC66E);

    // Fetch error codes
    let official_opcode_result = nes.read_cpu_ram(0x02);
    let unofficial_opcode_result = nes.read_cpu_ram(0x03);

    assert_eq!(official_opcode_result, 0, "Official opcodes exited with code ${:02X}", official_opcode_result);
    assert_eq!(unofficial_opcode_result, 0, "Unofficial opcodes exited with code ${:02X}", unofficial_opcode_result);
}
