//
// mapper/nrom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jan 03 2020
//


use super::MapperControl;
use crate::cart::{Cartridge, CartridgeInfo};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_first_bank() {
        let header = init_header(1, 1);
        let mut prg_rom = [0u8; kb!(16)];
        let chr_rom = [0u8; kb!(8)];

        // Put markers in the PRG and CHR ROM data to identify the blocks after loading the cartridge
        prg_rom[0x00] = 0xDE;
        prg_rom[prg_rom.len()-1] = 0xAD;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();

        let nrom = Nrom::from(cart);

        assert_eq!(nrom.read(0x8000), 0xDE);
    }

    #[test]
    fn read_mirrored() {
        let header = init_header(1, 1);
        let mut prg_rom = [0u8; kb!(16)];
        let chr_rom = [0u8; kb!(8)];

        // Put markers in the PRG and CHR ROM data to identify the blocks after loading the cartridge
        prg_rom[0x00] = 0xDE;
        prg_rom[prg_rom.len()-1] = 0xAD;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();

        let nrom = Nrom::from(cart);

        assert_eq!(nrom.read(0xC000), 0xDE);
    }

    fn init_header(num_prg_banks: u8, num_chr_banks: u8) -> [u8; 16] {
        [
            0x4E, 0x45, 0x53, 0x1A, // NES<EOF>
            num_prg_banks,                   // PRG ROM
            num_chr_banks,                   // CHR ROM
            0x00,                   // Flag 6
            0x00,                   // Flag 7
            0x00,                   // Flag 8
            0x00,                   // Flag 9
            0x00,                   // Flag 10
            0x00,                   // Flag 11
            0x00,                   // Flag 12
            0x00,                   // Flag 13
            0x00,                   // Flag 14
            0x00,                   // Flag 15
        ]
    }
}
