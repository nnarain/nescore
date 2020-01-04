//
// mapper/nrom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jan 03 2020
//


use super::MapperControl;
use crate::Cartridge;

use super::mem::Memory;

pub struct Nrom {
    prg_rom: Memory,
    prg_ram: [u8; 0x2000],
    chr_rom: [u8; 0x2000],
    mirror_rom: bool,
}

impl Nrom {
    pub fn from(cart: Cartridge) -> Self {
        let (info, prg_rom, chr_rom) = cart.to_parts();

        let mut chr_rom_arr = [0x0u8; 0x2000];
        chr_rom_arr.copy_from_slice(chr_rom.as_slice());

        Nrom {
            prg_rom: Memory::new(prg_rom, info.prg_rom_banks),
            prg_ram: [0; 0x2000],
            chr_rom: chr_rom_arr,
            mirror_rom: info.prg_rom_banks == 1,
        }
    }
}

impl MapperControl for Nrom {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize]
            },
            0x8000..=0xBFFF => {
                self.prg_rom.read(0, (addr - 0x8000) as usize)
            },
            0xC000..=0xFFFF => {
                let bank = if self.mirror_rom { 0 } else { 1 };
                self.prg_rom.read(bank, (addr - 0xC000) as usize)
            }
            _ => {
                panic!("Invalid address for mapper")
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize] = value,
            _ => {
                // No internal registers :O
            }
        }
    }
}