# NES

[![Build Status](https://github.com/nnarain/nes/workflows/Build/badge.svg)](https://github.com/nnarain/nes/actions)
[![codecov](https://codecov.io/gh/nnarain/nes/branch/develop/graph/badge.svg)](https://codecov.io/gh/nnarain/nes)

NES emulator and tools

Build
-----

```
cargo build
```

To test, pull in submodules and run `cargo test`

```
git submodule update --init --recursive
cargo test
```

nescore
-------

Core library for emulating the NES.

The basics so far:

```rust
use nescore::{Nes, Cartridge, Button};

fn main() {
    let cart = Cartridge::from_path("/path/to/rom.nes").unwrap();
    let mut nes = Nes::default().with_cart(cart);

    // Run the NES for a single frame. Audio is TODO
    let framebuffer = nes.emulate_frame();

    // Standard controller input
    nes.input(Button::A, true);

    // Update display on platform of your choice
    // ...
}
```

nescli
------

Some tooling for interacting with ROM files.

```
nescli run    <ROM> # Run the ROM file
nescli run -d <ROM> # Run the ROM file with CPU debug output

nescli info <ROM> # Display cartridge header information
nescli img  <ROM> # Dump CHR ROM to a PNG file
```