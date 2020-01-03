//
// mapper/mmc1.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 26 2019
//




use super::MapperControl;
use crate::cart::{Cartridge, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};

use super::mem::Memory;

const PRG_RAM_SIZE: usize = 0x2000;
const SHIFT_REGISTER_INIT_VALUE: u8 = 0x10;

/// Mirroring Options
enum Mirroring {
    OneScreenLower, OneScreenUpper, Vertical, Horizontal,
}

/// Program ROM Bank Mode Options
enum PrgRomBankMode {
    SWITCH_32K,
    SWITCH_C000,
    SWITCH_8000,
}

enum ChrRomBankMode {
    SWITCH_8K,
    SWITCH_4K,
}

/// MMC1 Mapper
pub struct Mmc1 {
    prg_rom: Memory,             // Program ROM
    prg_ram: [u8; PRG_RAM_SIZE], // Program RAM
    chr_rom: Memory,

    shift_register: u8,

    mirroring: Mirroring,
    prg_rom_bank_mode: PrgRomBankMode,
    chr_rom_bank_mode: ChrRomBankMode,

    prg_bank_selection: usize,
    chr_bank0_selection: usize,
    chr_bank1_selection: usize,
}

impl Mmc1 {
    pub fn from(cart: Cartridge) -> Self {
        let (info, prg_rom, chr_rom) = cart.to_parts();

        Mmc1{
            prg_rom: Memory::new(prg_rom, info.prg_rom_banks, PRG_ROM_BANK_SIZE),
            prg_ram: [0; PRG_RAM_SIZE],
            chr_rom: Memory::new(chr_rom, info.chr_rom_banks, CHR_ROM_BANK_SIZE),

            shift_register: SHIFT_REGISTER_INIT_VALUE,

            mirroring: Mirroring::Vertical,
            prg_rom_bank_mode: PrgRomBankMode::SWITCH_8000,
            chr_rom_bank_mode: ChrRomBankMode::SWITCH_8K,

            prg_bank_selection: 0,
            chr_bank0_selection: 0,
            chr_bank1_selection: 0,
        }
    }

    fn load_shift_register(&mut self, data: u8) -> Option<u8> {
        let final_write = bit_is_set!(self.shift_register, 0);

        self.shift_register = (self.shift_register >> 1) | ((data & 0x01) << 5);

        if final_write {
            let sr = self.shift_register;
            self.shift_register = SHIFT_REGISTER_INIT_VALUE;

            Some(sr)
        }
        else {
            None
        }
    }

    fn write_control(&mut self, flags: u8) {
        self.mirroring = match flags & 0x03 {
            0 => Mirroring::OneScreenLower,
            1 => Mirroring::OneScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };

        self.prg_rom_bank_mode = match (flags >> 2) & 0x03 {
            0 | 1 => PrgRomBankMode::SWITCH_32K,
            2     => PrgRomBankMode::SWITCH_C000,
            3     => PrgRomBankMode::SWITCH_8000,
            _ => unreachable!(),
        };

        self.chr_rom_bank_mode = match mask_is_set!(flags, 4) {
            false => ChrRomBankMode::SWITCH_8K,
            true  => ChrRomBankMode::SWITCH_4K,
        };

        let prg_switch_size = match self.prg_rom_bank_mode {
            PrgRomBankMode::SWITCH_32K                        => kb!(32),
            PrgRomBankMode::SWITCH_8000 | PrgRomBankMode::SWITCH_C000 => kb!(16),
        };

        self.prg_rom.set_bank_size(prg_switch_size);
    }

    fn write_chr_bank0(&mut self, value: u8) {
        // Select 4 KB or 8 KB bank at PPU $0000
        self.chr_bank0_selection = (value & 0x1F) as usize;
    }

    fn write_chr_bank1(&mut self, value: u8) {
        // Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
        self.chr_bank1_selection = (value & 0x1F) as usize;
    }

    fn write_prg_bank(&mut self, value: u8) {
        self.prg_bank_selection = (value & 0x0F) as usize;
    }

    fn write_registers(&mut self, addr: u16, value: u8) {
        match addr {
            // Register 0
            0x8000..=0x9FFF => {
                self.write_control(value);
            },
            // Register 1
            0xA000..=0xBFFF => {
                self.write_chr_bank0(value);
            },
            // Register 2
            0xC000..=0xDFFF => {
                self.write_chr_bank1(value);
            },
            // Register 3
            0xE000..=0xFFFF => {
                self.write_prg_bank(value);
            },
            _ => {
                // TODO: Write to ROM? Ignore?
            }
        }
    }
}

impl MapperControl for Mmc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize]
            },
            0x8000..=0xFFFF => {
                match self.prg_rom_bank_mode {
                    PrgRomBankMode::SWITCH_32K => {
                        self.prg_rom.read(self.prg_bank_selection, (addr - 0x8000) as usize)
                    },
                    PrgRomBankMode::SWITCH_8000 => {
                        match addr {
                            0x8000..=0xBFFF => self.prg_rom.read(self.prg_bank_selection, (addr - 0x8000) as usize),
                            0xC000..=0xFFFF => self.prg_rom.read_last((addr - 0xC000) as usize),
                            _ => unreachable!(),
                        }
                    },
                    PrgRomBankMode::SWITCH_C000 => {
                        match addr {
                            0x8000..=0xBFFF => self.prg_rom.read_first((addr - 0x8000) as usize),
                            0xC000..=0xFFFF => self.prg_rom.read(self.prg_bank_selection, (addr - 0xC000) as usize),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            _ => panic!("Invalid address for MMC1 read"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if bit_is_set!(value, 7) {
            self.shift_register = SHIFT_REGISTER_INIT_VALUE;
            self.write_control(0x0C);
        }
        else {
            if let Some(value) = self.load_shift_register(value) {
                self.write_registers(addr, value & 0x01);
            }
        }
    }
}

#[cfg(test)]
mod tests {

}
