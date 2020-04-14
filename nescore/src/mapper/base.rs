//
// mapper/base.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 28 2020
//

use super::{MapperControl, Mirroring};
use crate::cart::Cartridge;

const NAMETABLE_RAM_SIZE: usize = kb!(4);

/// Holds common mapper functionality
pub struct MapperBase<Mapper: MapperControl> {
    mapper: Mapper,

    // Nametable memory
    // Large enough for 4 nametables if mirroring is disabled
    // Normally, only the buffer 2 buffers are used, 1kb apart
    nametable_buffer: [u8; NAMETABLE_RAM_SIZE],
    palette_ram: [u8; 32],
    mirror_v: bool,
    four_screen: bool,
}

impl<Mapper: MapperControl + From<Cartridge>> From<Cartridge> for MapperBase<Mapper> {
    fn from(cart: Cartridge) -> Self {
        let mirror_v = cart.info.mirror_v;
        let four_screen = cart.info.four_screen_mode;

        MapperBase {
            mapper: Mapper::from(cart),

            // VRAM
            nametable_buffer: [0; NAMETABLE_RAM_SIZE],
            palette_ram: [0; 32],
            mirror_v,
            four_screen,
        }
    }
}

impl<Mapper: MapperControl> MapperControl for MapperBase<Mapper> {
    //------------------------------------------------------------------------------------------------------------------
    // PRG
    //------------------------------------------------------------------------------------------------------------------
    fn read(&self, addr: u16) -> u8 {
        self.mapper.read(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mapper.write(addr, data)
    }

    //------------------------------------------------------------------------------------------------------------------
    // CHR
    //------------------------------------------------------------------------------------------------------------------
    fn read_chr(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.mapper.read_chr(addr),
            0x2000..=0x2FFF => self.nametable_buffer[self.apply_mirroring(addr)],
            0x3000..=0x3EFF => self.nametable_buffer[self.apply_mirroring(addr - 0x1000)],
            0x3F00..=0x3F1F => self.palette_ram[(self.mirror_palette(addr) as usize) - 0x3F00],
            0x3F20..=0x3FFF => self.palette_ram[(self.mirror_palette(addr - 0x20) as usize) - 0x3F00],
            _ => panic!("Invalid address for VRAM: ${:04X}", addr),
        }
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.mapper.write_chr(addr, value),
            0x2000..=0x2FFF => self.nametable_buffer[self.apply_mirroring(addr)] = value,
            0x3000..=0x3EFF => self.nametable_buffer[self.apply_mirroring(addr - 0x1000)] = value,
            0x3F00..=0x3F1F => self.palette_ram[(self.mirror_palette(addr) as usize) - 0x3F00] = value & 0x3F,
            0x3F20..=0x3FFF => self.palette_ram[(self.mirror_palette(addr - 0x20) as usize) - 0x3F00] = value & 0x3F,
            _ => panic!("Invalid address for VRAM: ${:04X}", addr),
        }
    }
}

impl<Mapper: MapperControl> MapperBase<Mapper> {
    fn apply_mirroring(&self, addr: u16) -> usize {
        if self.four_screen {
            // In Four Screen Mode, mirroring is disabled
            addr as usize
        }
        else {
            helpers::calc_nametable_idx(addr, self.get_mirroring_type())
        }
    }

    fn get_mirroring_type(&self) -> Mirroring {
        self.mapper.mirroring().unwrap_or_else(|| if self.mirror_v { Mirroring::Vertical } else { Mirroring::Horizontal })
    }

    fn mirror_palette(&self, addr: u16) -> u16 {
        match addr {
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => addr - 0x10,
            _ => addr,
        }
    }
}

mod helpers {
    use super::Mirroring;

    /// Determine the index into the nametable buffer
    /// Horizontal mirroring - A vertical arrangement of nametable buffers
    /// Vertical mirroring - A horizontal arrangement of nametable buffers
    pub fn calc_nametable_idx(addr: u16, mirror_type: Mirroring) -> usize {
        let idx = (addr & 0x03FF) as usize;

        // Get the buffer offset
        let offset = match mirror_type {
            Mirroring::Vertical => {
                match addr {
                    0x2000..=0x23FF | 0x2800..=0x2BFF => 0,
                    0x2400..=0x27FF | 0x2C00..=0x2FFF => kb!(1),
                    _ => panic!("Invalid address for nametable"),
                }
            },
            Mirroring::Horizontal => {
                match addr {
                    0x2000..=0x23FF | 0x2400..=0x27FF => 0,
                    0x2800..=0x2BFF | 0x2C00..=0x2FFF => kb!(1),
                    _ => panic!("Invalid address for nametable"),
                }
            },
            Mirroring::OneScreenLower | Mirroring::OneScreenUpper => {
                0
            }
        };

        offset + idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cart::CartridgeInfo;

    #[test]
    fn horizontal_mirroring() {
        assert_eq!(helpers::calc_nametable_idx(0x2000, Mirroring::Horizontal), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2400, Mirroring::Horizontal), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2800, Mirroring::Horizontal), kb!(1));
        assert_eq!(helpers::calc_nametable_idx(0x2C00, Mirroring::Horizontal), kb!(1));
    }

    #[test]
    fn vertical_mirroring() {
        assert_eq!(helpers::calc_nametable_idx(0x2000, Mirroring::Vertical), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2400, Mirroring::Vertical), kb!(1));
        assert_eq!(helpers::calc_nametable_idx(0x2800, Mirroring::Vertical), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2C00, Mirroring::Vertical), kb!(1));
    }

    #[test]
    fn one_screen_mirroring() {
        assert_eq!(helpers::calc_nametable_idx(0x2000, Mirroring::OneScreenLower), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2400, Mirroring::OneScreenLower), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2800, Mirroring::OneScreenLower), 0);
        assert_eq!(helpers::calc_nametable_idx(0x2C00, Mirroring::OneScreenLower), 0);
    }

    #[test]
    fn nametable_mirroring() {
        let mut mapper = init_mapper();

        for i in 0..0x400 {
            mapper.write_chr((0x2000 + i) as u16, i as u8);
        }

        for i in 0..0x400 {
            let value = mapper.read_chr((0x3000 + i) as u16);
            assert_eq!(value, i as u8);
        }
    }

    #[test]
    fn mirror_palette() {
        let mut mapper = init_mapper();

        for i in 0..32 {
            mapper.write_chr(0x3F00 + i, i as u8);
            assert_eq!(mapper.read_chr(0x3F20 + i), i as u8);
        }
    }

    #[test]
    fn special_palette_mirroring() {
        let mut mapper = init_mapper();

        mapper.write_chr(0x3F10, 0x01);
        assert_eq!(mapper.read_chr(0x3F00), 0x01);

        mapper.write_chr(0x3F14, 0x01);
        assert_eq!(mapper.read_chr(0x3F04), 0x01);

        mapper.write_chr(0x3F18, 0x01);
        assert_eq!(mapper.read_chr(0x3F08), 0x01);

        mapper.write_chr(0x3F1C, 0x01);
        assert_eq!(mapper.read_chr(0x3F0C), 0x01);
    }

    struct FakeMapper {

    }

    #[allow(unused)]
    impl MapperControl for FakeMapper {
        fn read(&self, addr: u16) -> u8 {0}
        fn read_chr(&self, addr: u16) -> u8 {0}
        fn write(&mut self, addr: u16, data: u8) {}
        fn write_chr(&mut self, addr: u16, value: u8) {}
    }

    impl From<Cartridge> for FakeMapper {
        fn from(_: Cartridge) -> Self {
            FakeMapper{}
        }
    }

    fn init_mapper() -> MapperBase<FakeMapper> {
        let header = init_header(1, 1);
        let info = CartridgeInfo::from(&header[..]).unwrap();

        let cart = Cartridge::from_parts(info, vec![0; kb!(16)], vec![0; kb!(8)]);
        MapperBase::<FakeMapper>::from(cart)
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