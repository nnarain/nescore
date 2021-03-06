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

const CHR_RAM_SIZE: usize = kb!(8);

/// UNROM Mapper
pub struct Unrom {
    prg_rom: Memory,
    chr_ram: [u8; CHR_RAM_SIZE],
    rom_bank_selection: usize, // Select ROM bank
}


impl From<Cartridge> for Unrom {
    fn from(cart: Cartridge) -> Self {
        // Extract info and ROM data, VROM is unused
        let (_, prg_rom, _, _) = cart.into_parts();

        Unrom{
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            chr_ram: [0; CHR_RAM_SIZE],
            rom_bank_selection: 0,
        }
    }
}

impl MapperControl for Unrom {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                self.prg_rom.read(self.rom_bank_selection, (addr - 0x8000) as usize)
            },
            0xC000..=0xFFFF => {
                self.prg_rom.read_last((addr - 0xC000) as usize)
            }
            _ => { 0 }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.rom_bank_selection = (data & 0x0F) as usize;
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_ram[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        self.chr_ram[addr as usize] = value;
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

    #[test]
    fn irq() {
        let mut prg = vec![0; PRG_ROM_BANK_SIZE * 2];
        prg[(PRG_ROM_BANK_SIZE * 2)-2] = 0x01;
        prg[(PRG_ROM_BANK_SIZE * 2)-1] = 0x60;

        let unrom = init_unrom(prg, PRG_ROM_BANK_SIZE);
        let irq_lo = unrom.read(0xFFFE) as u16;
        let irq_hi = unrom.read(0xFFFF) as u16;

        let irq = (irq_hi << 8) | irq_lo;

        assert_eq!(irq, 0x6001);
    }

    fn init_unrom(data: Vec<u8>, bank_size: usize) -> Unrom {
        Unrom {
            prg_rom: Memory::new(data, bank_size),
            chr_ram: [0; CHR_RAM_SIZE],
            rom_bank_selection: 0,
        }
    }
}
