# NES

[![Build Status](https://travis-ci.org/nnarain/nes.svg?branch=master)](https://travis-ci.org/nnarain/nes)
[![codecov](https://codecov.io/gh/nnarain/nes/branch/master/graph/badge.svg)](https://codecov.io/gh/nnarain/nes)

NES emulator and tools

Build
-----

```
cargo build
```

nescore
-------

Core library for emulating the NES.

nesinfo
-------

Display cartridge information contained in the `.nes` rom file.

```
nesinfo -f <file path>
```

nesdisasm
---------

An NES ROM disassembly tool.
