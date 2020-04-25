use criterion::{criterion_main, criterion_group, Criterion};
use nescore::{Nes, Cartridge};

fn bench_emulate_frame(c: &mut Criterion) {
    let cart = Cartridge::from_path("tests/roms/nestest/nestest.nes").unwrap();
    let mut nes = Nes::default().with_cart(cart);

    c.bench_function("Nes::emulate_frame()", |b| {
        b.iter(|| {
            nes.emulate_frame();
        });
    });
}

criterion_group!(benches, bench_emulate_frame);
criterion_main!(benches);
