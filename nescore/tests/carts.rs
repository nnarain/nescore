use nescore::{Cartridge, cart::{CartridgeError, CartridgeLoader, LoaderError}};

#[test]
fn load_cart_from_file() {
    let result = Cartridge::from_path("tests/roms/nestest/nestest.nes");
    assert_eq!(result.is_ok(), true);
}

#[test]
fn load_cart_from_file_not_exist() {
    let result = Cartridge::from_path("file/that/doesn't/exist");
    assert_eq!(result.is_err(), true);

    let err = result.err().unwrap();
    assert!(matches!(err, CartridgeError::ReadFail(_)));
}

#[test]
fn loader_from_file() {
    let result = CartridgeLoader::default().rom_path("tests/roms/nestest/nestest.nes").load();
    assert!(result.is_ok());
}

#[test]
fn loader_from_file_not_exist() {
    let result = CartridgeLoader::default().rom_path("file/that/doesn't/exist").load();
    assert!(result.is_err());

    let err = result.err().unwrap();
    assert!(matches!(err, LoaderError::LoadCartridge(CartridgeError::ReadFail(_))));
}
