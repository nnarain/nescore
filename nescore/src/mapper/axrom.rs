//
// aorom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Apr 15 2021
//

use super::{MapperControl, Mirroring};
use crate::cart::Cartridge;

use super::mem::Memory;

pub struct Axrom {
    prg_rom: Memory,
    chr_ram: [u8; kb!(8)],
    bank_select: usize,
    single_screen_select: bool,
}

impl From<Cartridge> for Axrom {
    fn from(cart: Cartridge) -> Self {
        let (_, prg_rom, _, _) = cart.into_parts();

        Axrom {
            prg_rom: Memory::new(prg_rom, kb!(32)),
            chr_ram: [0; kb!(8)],
            bank_select: 0,
            single_screen_select: false,
        }
    }
}

impl MapperControl for Axrom {
    fn read(&self, addr: u16) -> u8 {
        if let 0x8000..=0xFFFF = addr {
            self.prg_rom.read(self.bank_select, (addr - 0x8000) as usize)
        }
        else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            let bank = data & 0x07;
            self.bank_select = bank as usize;

            self.single_screen_select = (data & 0x10) != 0;
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_ram[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        self.chr_ram[addr as usize] = value;
    }

    fn mirroring(&self) -> Option<Mirroring> {
        if self.single_screen_select {
            Some(Mirroring::OneScreenUpper)
        }
        else {
            None
        }
    }
}
