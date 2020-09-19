//
// mapper/mmc1.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Dec 26 2019
//

use super::{MapperControl, Mirroring};
use crate::cart::{Cartridge, PRG_ROM_BANK_SIZE, CHR_ROM_BANK_SIZE};

use super::mem::Memory;

const PRG_RAM_SIZE: usize = 0x2000;
const SHIFT_REGISTER_INIT_VALUE: u8 = 0x10;

/// Program ROM Bank Mode Options
#[derive(Debug)]
enum PrgRomBankMode {
    Switch32K,
    SwitchC000,
    Switch8000,
}

#[derive(Debug)]
enum ChrBankMode {
    Switch8K,
    Switch4K,
}

/// MMC1 Mapper
pub struct Mmc1 {
    prg_rom: Memory,             // Program ROM
    prg_ram: [u8; PRG_RAM_SIZE], // Program RAM
    chr_data: Memory,

    shift_register: u8,

    mirroring: Mirroring,
    prg_rom_bank_mode: PrgRomBankMode,
    chr_bank_mode: ChrBankMode,

    prg_bank_selection: usize,
    chr_bank0_selection: usize,
    chr_bank1_selection: usize,
}

impl From<Cartridge> for Mmc1 {
    fn from(cart: Cartridge) -> Self {
        let (_, prg_rom, chr_rom, sav_ram) = cart.into_parts();

        // If no CHR ROM is provided, use 8Kb of CHR RAM
        let chr_data = if chr_rom.is_empty() {
            vec![0x00u8; kb!(8)]
        }
        else {
            chr_rom
        };

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

        Mmc1{
            prg_rom: Memory::new(prg_rom, PRG_ROM_BANK_SIZE),
            prg_ram,
            chr_data: Memory::new(chr_data, CHR_ROM_BANK_SIZE),

            shift_register: SHIFT_REGISTER_INIT_VALUE,

            mirroring: Mirroring::OneScreenLower,
            prg_rom_bank_mode: PrgRomBankMode::Switch8000,
            chr_bank_mode: ChrBankMode::Switch4K,

            prg_bank_selection: 0,
            chr_bank0_selection: 0,
            chr_bank1_selection: 0,
        }
    }
}

impl Mmc1 {
    fn load_shift_register(&mut self, data: u8) -> Option<u8> {
        let final_write = bit_is_set!(self.shift_register, 0);

        self.shift_register = (self.shift_register >> 1) | ((data & 0x01) << 4);

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
            0 | 1 => PrgRomBankMode::Switch32K,
            2     => PrgRomBankMode::SwitchC000,
            3     => PrgRomBankMode::Switch8000,
            _ => unreachable!(),
        };

        self.chr_bank_mode = match bit_is_set!(flags, 4) {
            false => ChrBankMode::Switch8K,
            true  => ChrBankMode::Switch4K,
        };

        let prg_switch_size = match self.prg_rom_bank_mode {
            PrgRomBankMode::Switch32K                               => kb!(32),
            PrgRomBankMode::Switch8000 | PrgRomBankMode::SwitchC000 => kb!(16),
        };

        let chr_switch_size = match self.chr_bank_mode {
            ChrBankMode::Switch8K => kb!(8),
            ChrBankMode::Switch4K => kb!(4),
        };

        self.prg_rom.set_bank_size(prg_switch_size);
        self.chr_data.set_bank_size(chr_switch_size);
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
                    PrgRomBankMode::Switch32K => {
                        // Ignore lower bit of the PRG ROM back selection
                        let bank = self.prg_bank_selection >> 1;
                        self.prg_rom.read(bank, (addr - 0x8000) as usize)
                    },
                    PrgRomBankMode::Switch8000 => {
                        match addr {
                            0x8000..=0xBFFF => self.prg_rom.read(self.prg_bank_selection, (addr - 0x8000) as usize),
                            0xC000..=0xFFFF => self.prg_rom.read_last((addr - 0xC000) as usize),
                            _ => unreachable!(),
                        }
                    },
                    PrgRomBankMode::SwitchC000 => {
                        match addr {
                            0x8000..=0xBFFF => self.prg_rom.read_first((addr - 0x8000) as usize),
                            0xC000..=0xFFFF => self.prg_rom.read(self.prg_bank_selection, (addr - 0xC000) as usize),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            _ => panic!("Invalid address for MMC1 read: ${:04X}", addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = value;
            },
            0x8000..=0xFFFF => {
                if bit_is_set!(value, 7) {
                    // Any value with bit 7 set will load the shift register with its initial value
                    self.shift_register = SHIFT_REGISTER_INIT_VALUE;
                    self.write_control(0x0C);
                }
                else {
                    // Write to the shift register until it is full
                    if let Some(value) = self.load_shift_register(value) {
                        // Write to internal registers
                        self.write_registers(addr, value);
                    }
                }
            },
            _ => panic!("Invalid address for MMC1"),
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        match self.chr_bank_mode {
            ChrBankMode::Switch8K => {
                // Low bit ignored in this mode
                let bank = self.chr_bank0_selection >> 1;
                self.chr_data.read(bank, addr as usize)
            },
            ChrBankMode::Switch4K => {
                match addr {
                    0x0000..=0x0FFF => {
                        self.chr_data.read(self.chr_bank0_selection, addr as usize)
                    },
                    0x1000..=0x1FFF => {
                        self.chr_data.read(self.chr_bank1_selection, (addr - 0x1000) as usize)
                    },
                    _ => {
                        panic!("Invalid address of chr read: {:04X}", addr)
                    }
                }
            }
        }
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        match self.chr_bank_mode {
            ChrBankMode::Switch8K => {
                // Low bit ignored in this mode
                let bank = self.chr_bank0_selection >> 1;
                self.chr_data.write(bank, addr as usize, value);
            },
            ChrBankMode::Switch4K => {
                match addr {
                    0x0000..=0x0FFF => {
                        self.chr_data.write(self.chr_bank0_selection, addr as usize, value);
                    },
                    0x1000..=0x1FFF => {
                        self.chr_data.write(self.chr_bank1_selection, (addr - 0x1000) as usize, value);
                    },
                    _ => {
                        panic!("Invalid address of chr write: ${:04X}", addr)
                    }
                }
            }
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_chr_4k_switch() {
        let header = init_header(1, 1);
        let prg_rom = [0u8; kb!(16)];
        let mut chr_rom = [0u8; kb!(8)];

        chr_rom[0x0000] = 0xDE;
        chr_rom[0x1000] = 0xAD;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();
        let mut mmc1 = Mmc1::from(cart);

        // Set 4Kb switch mode
        write_register(&mut mmc1, 0x8000, 0x10);
        // Set first pattern area to bank 1
        write_register(&mut mmc1, 0xA000, 0x01);
        // Set second pattern area to bank 0
        write_register(&mut mmc1, 0xC000, 0x00);

        assert_eq!(mmc1.read_chr(0x0000), 0xAD);
        assert_eq!(mmc1.read_chr(0x1000), 0xDE);
    }

    #[test]
    fn read_prg_32k() {
        let header = init_header(2, 1);
        let mut prg_rom = [0u8; kb!(32)];
        let chr_rom = [0u8; kb!(8)];

        prg_rom[0] = 0xDE;
        prg_rom[prg_rom.len()-1] = 0xAD;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();
        let mut mmc1 = Mmc1::from(cart);

        // Set 32K switch mode
        write_register(&mut mmc1, 0x8000, 0x07);
        // ROM bank 0
        write_register(&mut mmc1, 0xE000, 0x00);

        assert_eq!(mmc1.read(0x8000), 0xDE);
        assert_eq!(mmc1.read(0xFFFF), 0xAD);
    }

    fn write_register(mmc1: &mut Mmc1, addr: u16, mut value: u8) {
        for _ in 0..5 {
            mmc1.write(addr, value & 0x01);
            value >>= 1;
        }
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
