//
// cart.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 19 2019
//

use std::fmt;

use std::io;
use std::io::prelude::*;
use std::fs::File;

pub const PRG_ROM_BANK_SIZE: usize = kb!(16);
pub const CHR_ROM_BANK_SIZE: usize = kb!(8);

#[derive(Debug, PartialEq, Clone)]
pub enum Format {
    INES,
    NES2
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Format::INES => write!(f, "INES"),
            Format::NES2 => write!(f, "NES2"),
        }
    }
}

pub enum ParseError {
    InvalidSize(usize),
    InvalidSig,
    InvalidFormat
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidSig     => write!(f, "Invalid signature at start of file. Expected `NES`. Not an NES ROM"),
            ParseError::InvalidSize(s) => write!(f, "Not enough data to parse header (Size: {})", s),
            ParseError::InvalidFormat  => write!(f, "The detected header is not valid")
        }
    }
}

#[derive(Debug)]
pub enum CartridgeError {
    // FailedToOpen(io::Error),
    // FailedToLoad(io::Error),
    ReadFail(io::Error),
    InvalidRom(ParseError),
}

#[derive(Debug)]
pub struct CartridgeInfo {
    pub format: Format,
    pub prg_rom_banks: usize,    // 16kB units
    pub chr_rom_banks: usize,    // 8kB units (0 means board uses CHR RAM)
    pub mapper: usize,           // Mapper Number
    pub four_screen_mode: bool,  // Four screen mode
    pub trainer: bool,           // Trainer present
    pub battback_sram: bool,     // Battery backed SRAM at $6000-$7000
    pub mirror_v: bool,          // Vertical mirroring if true, horizontal if false
    pub vs_unisystem: bool,      // VS Unisystem
    pub playchoice10: bool,      // PlayChoice
    pub tv_system_pal: bool,     // NTSC if false, PAL if true
    pub tv_system_ext: usize,    // Unofficial TV supper, 0 - NTSC, 1 - PAL, 2 - Dual Compat
    
    // below are NES 2.0 only
    pub submapper: usize,        // Submapper number
    pub mapper_planes: usize,    // Mapper planes
    pub batt_prg_ram: usize,     // Amount of battery backed PRG RAM
    pub prg_ram: usize,          // Amount of non-battery backed PRG RAM
    pub batt_chr_ram: usize,     // Amount of battery backed CHR RAM
    pub chr_ram: usize,          // Amount of non-battery backed CHR RAM
}

impl CartridgeInfo {
    pub fn from(rom: &[u8]) -> Result<Self, CartridgeError> {
        parse_header(rom)
    }
}

impl fmt::Display for CartridgeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mirroring = if self.mirror_v { String::from("Vertical") } else { String::from("Horizontal") };
        let tv_system = if self.tv_system_pal { String::from("PAL") } else { String::from("NTSC") };

        write!(f,
        "
        Format:              {}
        PRG ROM Banks:       {}
        CHR ROM Banks:       {}
        Mapper:              {}
        Four Screen Mode:    {}
        Trainer:             {}
        Battery Backed SRAM: {}
        Mirroring:           {}
        TV System:           {}
        ", 
        self.format, self.prg_rom_banks, self.chr_rom_banks, get_mapper_name(self.mapper),
        self.four_screen_mode, self.trainer, self.battback_sram, mirroring, tv_system)
    }
}

fn get_mapper_name(mapper: usize) -> String {
    match mapper {
        0 => format!("NROM (Mapper {})", mapper),
        1 => format!("MMC1 (Mapper {})", mapper),
        2 => format!("UNROM (Mapper {})", mapper),
        3 => format!("CNROM (Mapper {})", mapper),
        4 => format!("MMC3 (Mapper {})", mapper),
        5 => format!("MMC5 (Mapper {})", mapper),
        7 => format!("AOROM (Mapper {})", mapper),
        9 => format!("MMC2 (Mapper {})", mapper),
        10 => format!("MMC4 (Mapper {})", mapper),
        11 => format!("Color Dreams (Mapper {})", mapper),
        16 => format!("Bandai (Mapper {})", mapper),
        _ => format!("Mapper {}", mapper)
    }
}

pub struct Cartridge {
    pub info: CartridgeInfo,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    bat_ram: Vec<u8>,
}

impl Cartridge {
    pub fn from(rom: Vec<u8>) -> Result<Cartridge, CartridgeError> {
        CartridgeInfo::from(rom.as_slice()).map(|info| {
            // Determine the number of bytes for PRG ROM and CHR ROM
            let prg_rom_size = info.prg_rom_banks * PRG_ROM_BANK_SIZE;
            let chr_rom_size = info.chr_rom_banks * CHR_ROM_BANK_SIZE;

            // Determine offset of the PRG ROM in the buffer
            let header_bytes = 16;
            let trainer_bytes = if info.trainer { 512 } else { 0 };
            let prg_rom_offset = header_bytes + trainer_bytes;

            // Get a slice for the program ROM
            let prg_rom = rom[prg_rom_offset..prg_rom_offset+prg_rom_size].to_vec();
            // Get a slice for the character ROM
            let chr_rom = rom[(prg_rom_offset+prg_rom_size)..(prg_rom_offset+prg_rom_size+chr_rom_size)].to_vec();

            Cartridge::from_parts(info, prg_rom, chr_rom, vec![])
        })
    }

    pub fn from_path(path: &str) -> Result<Cartridge, CartridgeError> {
        load_file(path)
            .map_err(CartridgeError::ReadFail)
            .and_then(Cartridge::from)
    }

    /// Construct a Cartridge from parts
    pub fn from_parts(info: CartridgeInfo, prg_rom: Vec<u8>, chr_rom: Vec<u8>, bat_ram: Vec<u8>) -> Self {
        Cartridge {
            info,
            prg_rom,
            chr_rom,
            bat_ram,
        }
    }

    /// Consume the cartridge and return the info, program ROM and character ROM
    pub fn into_parts(self) -> (CartridgeInfo, Vec<u8>, Vec<u8>, Vec<u8>) {
        (self.info, self.prg_rom, self.chr_rom, self.bat_ram)
    }

    pub fn add_battery_ram(mut self, batt: Vec<u8>) -> Self {
        self.bat_ram = batt;
        self
    }
}

#[derive(Debug)]
pub enum LoaderError {
    NoRomProvided,
    LoadCartridge(CartridgeError),
    LoadSave(io::Error),
}

/// Cartridge Loader Helper
#[derive(Default)]
pub struct CartridgeLoader {
    rom_path: Option<String>,
    sav_path: Option<String>,
}

impl CartridgeLoader {
    pub fn load(self) -> Result<Cartridge, LoaderError> {
        let cart_result = self.rom_path
            .map_or(Err(LoaderError::NoRomProvided), |path| {
                load_file(&path)
                    .map_err(CartridgeError::ReadFail)
                    .and_then(Cartridge::from)
                    .map_err(LoaderError::LoadCartridge)
            });

        // This.. This could probably be better...
        match cart_result {
            Ok(cart) => {
                match self.sav_path {
                    Some(path) => {
                        match load_file(&path) {
                            Ok(buf) => {
                                Ok(cart.add_battery_ram(buf))
                            },
                            Err(_) => Ok(cart),
                        }
                    },
                    None => Ok(cart),
                }
            },
            Err(e) => Err(e),
        }
    }

    pub fn rom_path(mut self, path: &str) -> Self {
        self.rom_path = Some(path.to_string());
        self
    }

    pub fn save_path(mut self, path: &str) -> Self {
        self.sav_path = Some(path.to_string());
        self
    }
}

fn load_file(path: &str) -> Result<Vec<u8>, io::Error> {
    match File::open(path) {
        Ok(ref mut file) => {
            let mut buffer: Vec<u8> = Vec::new();

            match file.read_to_end(&mut buffer) {
                Ok(_) => Ok(buffer),
                Err(e) => Err(e),
            }
        },
        Err(e) => Err(e),
    }
}

// Parse NES ROM header
fn parse_header(rom_header: &[u8]) -> Result<CartridgeInfo, CartridgeError> {
    if rom_header.len() < 16 {
        return Err(CartridgeError::InvalidRom(ParseError::InvalidSize(rom_header.len())))
    }

    if !verify_signature(&rom_header[0..4]) {
        return Err(CartridgeError::InvalidRom(ParseError::InvalidSig))
    }

    get_rom_info(rom_header)
}

/// Pull rom info from header
fn get_rom_info(rom_header: &[u8]) -> Result<CartridgeInfo, CartridgeError> {

    let format = match get_format(rom_header) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let mut info = CartridgeInfo {
        format: format.clone(),
        prg_rom_banks: 0,
        chr_rom_banks: 0,
        mapper: 0,
        four_screen_mode: false,
        trainer: false,
        battback_sram: false,
        mirror_v: false,
        vs_unisystem: false,
        playchoice10: false,
        tv_system_pal: false,
        tv_system_ext: 0,
        submapper: 0,
        mapper_planes: 0,
        batt_prg_ram: 0,
        prg_ram: 0,
        batt_chr_ram: 0,
        chr_ram: 0,
    };

    get_info_common(rom_header, &mut info);

    match format {
        Format::INES => get_info_ines(rom_header, &mut info),
        Format::NES2 => get_info_nes2(rom_header, &mut info),
    }

    Ok(info)
}

fn get_info_common(rom_header: &[u8], info: &mut CartridgeInfo) {
    // get program and character bank info
    let prg_rom_banks = rom_header[4] as usize;
    let chr_rom_banks = rom_header[5] as usize;

    let flag6 = rom_header[6];
    let flag7 = rom_header[7];

    // vertical mirroring flag
    let mirror_v = bit_is_set!(flag6, 0);
    // battery backed SRAM flag
    let battback_sram = bit_is_set!(flag6, 1);
    // trainer
    let trainer = bit_is_set!(flag6, 2);
    // four screen mode
    let four_screen_mode = bit_is_set!(flag6, 3);

    let vs_unisystem = bit_is_set!(flag7, 0);
    let playchoice10 = bit_is_set!(flag7, 1);

    // mapper number
    let mapper_lo = flag6 >> 4;
    let mapper_hi = flag7 & 0xF0;
    let mapper = mapper_hi | mapper_lo;

    info.prg_rom_banks = prg_rom_banks;
    info.chr_rom_banks = chr_rom_banks;
    info.mirror_v = mirror_v;
    info.battback_sram = battback_sram;
    info.trainer = trainer;
    info.four_screen_mode = four_screen_mode;
    info.mapper = mapper as usize;
    info.vs_unisystem = vs_unisystem;
    info.playchoice10 = playchoice10;
}

/// Get info from INES formatted ROM
fn get_info_ines(rom_header: &[u8], info: &mut CartridgeInfo) {
    // TV system support
    info.tv_system_pal = (rom_header[9] & 0x01u8) != 0;
    info.tv_system_ext = match rom_header[10] & 0x03u8 {
        0     => 0,
        2     => 1,
        1 | 3 => 2,
        _     => 0
    };
}

/// Get info from INES formatted ROM
fn get_info_nes2(rom_header: &[u8], info: &mut CartridgeInfo) {
    // additional mapper info
    let submapper = (rom_header[8] & 0xF0u8) >> 4;
    let mapper_planes = rom_header[8] & 0x0Fu8;

    info.submapper = submapper as usize;
    info.mapper_planes = mapper_planes as usize;

    // extend PRG and CHR rom size
    let prg_rom_hi_bits = rom_header[9] & 0x0Fu8;
    let chr_rom_hi_bits = (rom_header[9] & 0xF0u8) >> 4;

    info.prg_rom_banks |= (prg_rom_hi_bits as usize) << 8;
    info.chr_rom_banks |= (chr_rom_hi_bits as usize) << 8;

    // PRG RAM size
    info.batt_prg_ram = (rom_header[10] >> 4) as usize;
    info.prg_ram = (rom_header[10] & 0x0Fu8) as usize;

    // CHR RAM size
    info.batt_chr_ram = (rom_header[11] >> 4) as usize;
    info.chr_ram = (rom_header[11] & 0x0Fu8) as usize;

    // TV system
    info.tv_system_ext = if (rom_header[12] & 0x01) != 0 {
        0
    }
    else {
        1
    }
}

/// Get the NES ROM format
fn get_format(rom_header: &[u8]) -> Result<Format, CartridgeError> {
    let flag7 = rom_header[7];

    if (flag7 & 0x0Cu8) == 0x08u8 {
        Ok(Format::NES2)
    }
    else {
        // if this is an INES format rom, bytes 8-15 should be $00
        let empty_bytes = &rom_header[12..16];

        if empty_bytes == [0, 0, 0, 0] {
            Ok(Format::INES)
        }
        else {
            Err(CartridgeError::InvalidRom(ParseError::InvalidFormat))
        }
    }
}

/// Verify the signature at the start of the file `NES<EOF>`
fn verify_signature(sig: &[u8]) -> bool {
    sig == [0x4E, 0x45, 0x53, 0x1A]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_header() -> [u8; 16] {
        [
            0x4E, 0x45, 0x53, 0x1A, // NES<EOF>
            0x0F,                   // PRG ROM
            0x0F,                   // CHR ROM
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

    #[test]
    #[should_panic]
    fn invalid_format() {
        let mut header = init_header();
        header[12] = 1;

        parse_header(&header[..]).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_sig() {
        let header: [u8; 16] = [0; 16];
        parse_header(&header[..]).unwrap();
    }

    #[test]
    #[should_panic]
    fn not_enough_data() {
        let header: [u8; 10] = [0; 10];
        parse_header(&header[..]).unwrap();
    }

    #[test]
    fn mirroring_vertical() {
        let mut header = init_header();
        header[6] |= 0x01;

        let info = parse_header(&header[..]).unwrap();
        assert_eq!(info.mirror_v, true);
    }

    #[test]
    fn mirroring_horizontal() {
        let mut header = init_header();
        header[6] |= 0x00;

        let info = parse_header(&header[..]).unwrap();
        assert_eq!(info.mirror_v, false);
    }

    #[test]
    fn mapper_number() {
        let mut header = init_header();
        header[7] |= 0xD0;
        header[6] |= 0xE0;

        let info = parse_header(&header[..]).unwrap();
        assert_eq!(info.mapper, 0xDE);
    }

    #[test]
    fn get_format_nes2() {
        let mut rom_header = [0; 16];
        rom_header[7] = 0x08u8;

        let format = get_format(&rom_header[..]).unwrap();
        assert_eq!(format, Format::NES2);
    }

    #[test]
    fn get_format_ines() {
        let mut rom_header = [0; 16];
        rom_header[7] = 0x04u8;

        let format = get_format(&rom_header[..]).unwrap();
        assert_eq!(format, Format::INES);
    }

    #[test]
    fn number_of_prg_and_chr_rom_banks() {
        let header = init_header();
        
        let info = parse_header(&header[..]).unwrap();
        assert_eq!(info.prg_rom_banks, 0x0F);
        assert_eq!(info.chr_rom_banks, 0x0F);
    }

    #[test]
    fn load_cart_from_vec() {
        const PRG_ROM_SIZE: usize = 15 * PRG_ROM_BANK_SIZE;
        const CHR_ROM_SIZE: usize = 15 * CHR_ROM_BANK_SIZE;

        let header = init_header();
        let mut prg_rom = [0u8; PRG_ROM_SIZE];
        let mut chr_rom = [0u8; CHR_ROM_SIZE];

        // Put markers in the PRG and CHR ROM data to identify the blocks after loading the cartridge
        prg_rom[0x00] = 0xDE;
        prg_rom[PRG_ROM_SIZE-1] = 0xAD;
        chr_rom[0x00] = 0xBE;
        chr_rom[CHR_ROM_SIZE-1] = 0xEF;

        let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

        let cart = Cartridge::from(rom).unwrap();

        assert_eq!(cart.prg_rom.len(), PRG_ROM_SIZE);
        assert_eq!(cart.prg_rom[0x00], 0xDE);
        assert_eq!(cart.prg_rom[PRG_ROM_SIZE-1], 0xAD);

        assert_eq!(cart.chr_rom.len(), CHR_ROM_SIZE);
        assert_eq!(cart.chr_rom[0x00], 0xBE);
        assert_eq!(cart.chr_rom[CHR_ROM_SIZE-1], 0xEF);
    }
}

