use nescore::Cartridge;

#[test]
fn load_cart_from_file() {
    let cart = Cartridge::from_path("tests/roms/nestest/nestest.nes");
    assert_eq!(cart.is_ok(), true);
}

#[test]
fn load_cart_from_file_not_exist() {
    let cart = Cartridge::from_path("file/that/doesn't/exist");
    assert_eq!(cart.is_err(), true);
}
