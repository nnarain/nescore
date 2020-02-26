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

nesinfo
-------

Display cartridge information contained in the `.nes` rom file.

```
nesinfo -f <ROM>
```

nesui
-----

Debugging UI for `nescore`

```
nesui -f <ROM>
```
