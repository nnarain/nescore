#![feature(test)]

extern crate test;

use test::Bencher;
use nescore::{Nes, Cartridge};

#[bench]
fn bench_emulate_frame(b: &mut Bencher) {
    let cart = Cartridge::from_path("tests/roms/nestest/nestest.nes").unwrap();
    let mut nes = Nes::new().with_cart(cart);

    b.iter(|| nes.emulate_frame());
}
