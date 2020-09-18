//
// mapper/nrom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jan 03 2020
//


use super::MapperControl;
use crate::cart::{Cartridge, PRG_ROM_BANK_SIZE};

use super::mem::Memory;

const PRG_RAM_SIZE: usize = kb!(8);
const CHR_DATA_SIZE: usize = kb!(8);

/// NROM Mapper
/// https://wiki.nesdev.com/w/index.php/NROM
pub struct Nrom {
    prg_rom: Memory,
    prg_ram: [u8; PRG_RAM_SIZE],
    chr_data: [u8; CHR_DATA_SIZE],
    mirror_rom: bool,
}

impl From<Cartridge> for Nrom {
    fn from(cart: Cartridge) -> Self {
        let (info, prg_rom, chr_rom, sav_ram) = cart.into_parts();

        let mut chr_rom_arr = [0x0u8; CHR_DATA_SIZE];
        for (i, byte) in chr_rom.iter().enumerate() {
            chr_rom_arr[i] = *byte;
        }

        // Copy sav ram to prg ram
        let mut prg_ram = [0u8; PRG_RAM_SIZE];
        for (i, b) in prg_ram.iter_mut().enumerate() {
            *b = if i < sav_ram.len() {
                sav_ram[i]
            }
            else {
                0
            };
        }

        Nrom {
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            prg_ram,
            chr_data: chr_rom_arr,
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
        if let 0x6000..=0x7FFF = addr {
            self.prg_ram[(addr - 0x6000) as usize] = value
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_data[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        self.chr_data[addr as usize] = value;
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
        assert_eq!(nrom.read(0xBFFF), 0xAD);
    }

    #[test]
    fn read_last_bank() {
        let header = init_header(2, 1);
        let mut prg_rom = [0x00; kb!(32)];
        let chr_rom = [0x00; kb!(8)];

        // Set IRQ Vector
        prg_rom[0x7FFE] = 0x01;
        prg_rom[0x7FFF] = 0x60;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let nrom: Nrom = Cartridge::from(rom).unwrap().into();

        let irq_lo = nrom.read(0xFFFE) as u16;
        let irq_hi = nrom.read(0xFFFF) as u16;
        let irq = irq_hi << 8 | irq_lo;

        assert_eq!(irq, 0x6001);
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
        assert_eq!(nrom.read(0xFFFF), 0xAD);
    }

    #[test]
    fn read_chr() {
        let header = init_header(1, 1);
        let prg_rom = [0u8; kb!(16)];
        let mut chr_rom = [0u8; kb!(8)];

        // Put markers in the PRG and CHR ROM data to identify the blocks after loading the cartridge
        chr_rom[0x00] = 0xDE;
        chr_rom[chr_rom.len()-1] = 0xAD;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();
        let nrom = Nrom::from(cart);

        assert_eq!(nrom.read_chr(0x0000), 0xDE);
        assert_eq!(nrom.read_chr(0x1FFF), 0xAD);
    }

    fn init_header(num_prg_banks: u8, num_chr_banks: u8) -> [u8; 16] {
        [
            0x4E, 0x45, 0x53, 0x1A, // NES<EOF>
            num_prg_banks,          // PRG ROM
            num_chr_banks,          // CHR ROM
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
