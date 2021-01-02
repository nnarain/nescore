//
// cnrom.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jan 01 2021
//

use super::MapperControl;
use crate::cart::{Cartridge, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};

use super::mem::Memory;

///
/// CNROM
///
/// * CPU $8000-$FFFF: 16 KB PRG ROM, fixed (if 16 KB PRG ROM used, then this is the same as $C000-$FFFF)
/// * CPU $C000-$FFFF: 16 KB PRG ROM, fixed
/// * PPU $0000-$1FFF: 8 KB switchable CHR ROM bank
///
/// https://wiki.nesdev.com/w/index.php/CNROM
///
pub struct Cnrom {
    prg_rom: Memory,
    chr_rom: Memory,
    chr_rom_bank: usize,
    prg_rom_banks: usize,
}

impl From<Cartridge> for Cnrom {
    fn from(cart: Cartridge) -> Self {
        let (info, prg_rom, chr_rom, _) = cart.into_parts();

        Cnrom {
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            chr_rom: Memory::new(chr_rom, CHR_ROM_BANK_SIZE),
            chr_rom_bank: 0,
            prg_rom_banks: info.prg_rom_banks,
        }
    }
}

impl MapperControl for Cnrom {
    fn read(&self, addr: u16) -> u8 {
        if addr >= 0xC000 {
            self.prg_rom.read_last((addr - 0xC000) as usize)
        }
        else if addr >= 0x8000 {
            if self.prg_rom_banks == 1 {
                self.prg_rom.read_last((addr - 0x8000) as usize)
            }
            else {
                self.prg_rom.read_first((addr - 0x8000) as usize)
            }
        }
        else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.chr_rom_bank = (data & 0x03) as usize;
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        if let 0x0000..=0x1FFF = addr {
            self.chr_rom.read(self.chr_rom_bank, addr as usize)
        }
        else {
            0
        }
    }

    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // No CHR RAM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prg_16k() {
        let mut prg = vec![0; PRG_ROM_BANK_SIZE * 2];
        let chr = vec![0; CHR_ROM_BANK_SIZE];

        prg[PRG_ROM_BANK_SIZE] = 0xDE;

        let cnrom = init_cnrom(prg, chr, 1);

        assert_eq!(cnrom.read(0x8000), 0xDE);
        assert_eq!(cnrom.read(0xC000), 0xDE);
    }

    #[test]
    fn prg_32k() {
        let mut prg = vec![0; PRG_ROM_BANK_SIZE * 2];
        let chr = vec![0; CHR_ROM_BANK_SIZE];

        prg[0] = 0xDE;
        prg[PRG_ROM_BANK_SIZE-1] = 0xAD;
        prg[PRG_ROM_BANK_SIZE] = 0xBE;
        prg[PRG_ROM_BANK_SIZE * 2 - 1] = 0xEF;

        let cnrom = init_cnrom(prg, chr, 2);

        assert_eq!(cnrom.read(0x8000), 0xDE);
        assert_eq!(cnrom.read(0xBFFF), 0xAD);
        assert_eq!(cnrom.read(0xC000), 0xBE);
        assert_eq!(cnrom.read(0xFFFF), 0xEF);
    }

    #[test]
    fn chr_bank_switching() {
        let prg = vec![0; PRG_ROM_BANK_SIZE * 2];
        let mut chr = vec![0; CHR_ROM_BANK_SIZE * 4];

        chr[0] = 0xDE;
        chr[CHR_ROM_BANK_SIZE-1] = 0xAD;
        chr[CHR_ROM_BANK_SIZE] = 0xBE;
        chr[CHR_ROM_BANK_SIZE * 2 - 1] = 0xEF;

        let mut cnrom = init_cnrom(prg, chr, 1);

        assert_eq!(cnrom.read_chr(0x0000), 0xDE);
        assert_eq!(cnrom.read_chr(0x1FFF), 0xAD);

        // Select bank 1
        cnrom.write(0x8000, 0x01);

        assert_eq!(cnrom.read_chr(0x0000), 0xBE);
        assert_eq!(cnrom.read_chr(0x1FFF), 0xEF);
    }

    fn init_cnrom(prg_rom: Vec<u8>, chr_rom: Vec<u8>, num_prg_banks: usize) -> Cnrom {
        Cnrom {
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            chr_rom: Memory::new(chr_rom, CHR_ROM_BANK_SIZE),
            chr_rom_bank: 0,
            prg_rom_banks: num_prg_banks,
        }
    }
}
