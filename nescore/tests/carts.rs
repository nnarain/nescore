use nescore;

use std::io;
use std::io::prelude::*;
use std::fs::File;

#[test]
fn load_cart_from_file() {
    let cart = nescore::Cartridge::from_path("tests/roms/nestest.nes");
    assert_eq!(cart.is_ok(), true);
}

#[test]
fn load_cart_from_file_not_exist() {
    let cart = nescore::Cartridge::from_path("file/that/doesn't/exist");
    assert_eq!(cart.is_err(), true);
}
