//
// mapper/unrom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 27 2019
//

// +-----------------+
// ¦ Mapper 2: UNROM ¦
// +-----------------+
// 
// +---------------+         +------------------------------------------+
// ¦ $8000 - $FFFF +---------¦ PPPPPPPP                                 ¦
// +---------------+         ¦ +------+                                 ¦
//                           ¦    ¦                                     ¦
//                           ¦    ¦                                     ¦
//                           ¦    +------- Select 16K ROM bank at $8000 ¦
//                           +------------------------------------------+
// 
// Notes: - When the cart is first started, the first 16K ROM bank in the cart
//           is loaded into $8000, and the LAST 16K ROM bank is loaded into
//           $C000. This last 16K bank is permanently "hard-wired" to $C000,
//           and it cannot be swapped.
//        - This mapper has no provisions for VROM; therefore, all carts
//           using it have 8K of VRAM at PPU $0000.
//        - Most carts with this mapper are 128K. A few, mostly Japanese
//           carts, such as Final Fantasy 2 and Dragon Quest 3, are 256K.
//        - Overall, this is one of the easiest mappers to implement in
//           a NES emulator.
// http://tuxnes.sourceforge.net/mappers-0.80.txt

use super::MapperControl;
use super::mem::Memory;
use crate::cart::{Cartridge, PRG_ROM_BANK_SIZE};

/// UNROM Mapper
pub struct Unrom {
    prg_rom: Memory,
    prg_ram: [u8; 0x2000],
    _chr_ram: Memory,
    rom_bank_selection: usize, // Select ROM bank
}

impl Unrom {
    pub fn from(cart: Cartridge) -> Self {
        // Extract info and ROM data, VROM is unused
        let (_, prg_rom, _) = cart.to_parts();

        Unrom{
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            prg_ram: [0; 0x2000],
            _chr_ram: Memory::new(vec![0; 8 * 1024], 8 * 1024),
            rom_bank_selection: 0,
        }
    }
}

impl MapperControl for Unrom {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000u16) as usize]
            },
            0x8000..=0xBFFF => {
                self.prg_rom.read(self.rom_bank_selection, (addr - 0x8000) as usize)
            },
            0xC000..=0xFFFF => {
                self.prg_rom.read_last((addr - 0x8000) as usize)
            }
            _ => { 0 }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = data;
            },
            0x8000..=0xFFFF => {
                self.rom_bank_selection = data as usize;
            },
            _ => {
                
            }
        }
    }

    fn read_chr(&self, _addr: u16) -> u8 {
        0
    }

    fn write_chr(&mut self, _addr: u16, _value: u8) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_select() {
        let mut data = vec![0; 20];
        data[10] = 0xDE;

        let mut unrom = init_unrom(data, 10);
        // Select ROM bank 1
        unrom.write(0x8000, 1);

        assert_eq!(unrom.read(0x8000), 0xDE);
    }

    fn init_unrom(data: Vec<u8>, bank_size: usize) -> Unrom {
        Unrom {
            prg_rom: Memory::new(data, bank_size),
            prg_ram: [0; 0x2000],
            _chr_ram: Memory::new(vec![0; 20], 10),
            rom_bank_selection: 0,
        }
    }
}
