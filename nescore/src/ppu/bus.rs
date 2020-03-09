//
// ppu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 11 2020
//


use crate::common::{IoAccess, IoAccessRef};
use crate::mapper::Mapper;

const INTERNAL_RAM: usize = 0x1000;

pub struct PpuIoBus {
    cpu: IoAccessRef,
    mapper: Mapper,

    nametable_ram: [u8; INTERNAL_RAM],
    palette_ram: [u8; 256],

    vertical_mirroring: bool,
}

impl PpuIoBus {
    pub fn new(cpu_io: IoAccessRef, mapper: Mapper, mirror_v: bool) -> Self {
        PpuIoBus {
            cpu: cpu_io,
            mapper: mapper,

            nametable_ram: [0x00; INTERNAL_RAM],
            palette_ram: [0x00; 256],

            vertical_mirroring: mirror_v,
        }
    }
}

impl IoAccess for PpuIoBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.mapper.borrow().read_chr(addr),
            0x2000..=0x2FFF => {
                self.nametable_ram[(helpers::calc_nametable_addr(addr, self.vertical_mirroring) - 0x2000) as usize]
            },
            0x3000..=0x3EFF => {
                self.nametable_ram[(helpers::calc_nametable_addr(addr - 0x1000, self.vertical_mirroring) - 0x2000) as usize]
            },
            0x3F00..=0x3FFF => self.palette_ram[(addr - 0x3F00) as usize],

            _ => panic!("Invalid read ${:04X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.mapper.borrow_mut().write_chr(addr, value),
            0x2000..=0x2FFF => {
                self.nametable_ram[(helpers::calc_nametable_addr(addr, self.vertical_mirroring) - 0x2000) as usize] = value
            },
            0x3000..=0x3EFF => {
                self.nametable_ram[(helpers::calc_nametable_addr(addr - 0x1000, self.vertical_mirroring) - 0x2000) as usize] = value;
            },
            0x3F00..=0x3FFF => self.palette_ram[(addr - 0x3F00) as usize] = value,

            _ => panic!("Invalid write ${:04X}={:02X}", addr, value),
        }
    }

    fn raise_interrupt(&mut self) {
        self.cpu.borrow_mut().raise_interrupt();
    }
}

mod helpers {
    pub fn calc_nametable_addr(addr: u16, vertically_mirrored: bool) -> u16 {
        if vertically_mirrored {
            match addr {
                0x2000..=0x27FF => addr + 0x800,
                _ => addr,
            }
        }
        else {
            match addr {
                0x2000..=0x23FF | 0x2800..=0x2BFF => addr + 0x400,
                _ => addr,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::MapperControl;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn read_write_nametable_mirroring_v() {
        let mut bus = init_bus(true);

        bus.write_byte(0x2000, 1);
        assert_eq!(bus.read_byte(0x2800), 1);
        bus.write_byte(0x2400, 2);
        assert_eq!(bus.read_byte(0x2C00), 2);
    }

    #[test]
    fn read_write_nametable_mirroring_h() {
        let mut bus = init_bus(false);

        bus.write_byte(0x2000, 1);
        assert_eq!(bus.read_byte(0x2400), 1);
        bus.write_byte(0x2800, 2);
        assert_eq!(bus.read_byte(0x2C00), 2);
    }

    #[test]
    fn nametable_mirroring() {
        let mut bus = init_bus(false);

        for i in 0..0x400 {
            bus.write_byte((0x2000 + i) as u16, i as u8);
        }

        for i in 0..0x400 {
            let value = bus.read_byte((0x3000 + i) as u16);
            assert_eq!(value, i as u8);
        }
    }

    #[test]
    fn horizontal_mirroring() {
        assert_eq!(helpers::calc_nametable_addr(0x2000, false), 0x2400);
        assert_eq!(helpers::calc_nametable_addr(0x2400, false), 0x2400);
        assert_eq!(helpers::calc_nametable_addr(0x2800, false), 0x2C00);
        assert_eq!(helpers::calc_nametable_addr(0x2C00, false), 0x2C00);
    }

    #[test]
    fn vertical_mirroring() {
        assert_eq!(helpers::calc_nametable_addr(0x2000, true), 0x2800);
        assert_eq!(helpers::calc_nametable_addr(0x2800, true), 0x2800);
        assert_eq!(helpers::calc_nametable_addr(0x2400, true), 0x2C00);
        assert_eq!(helpers::calc_nametable_addr(0x2C00, true), 0x2C00);
    }

    //------------------------------------------------------------------------------------------------------------------
    // Helpers
    //------------------------------------------------------------------------------------------------------------------

    fn init_bus(mirror_v: bool) -> PpuIoBus {
        let cpu = Rc::new(RefCell::new(FakeCpu::default()));
        let mapper = Rc::new(RefCell::new(FakeMapper::new()));

        PpuIoBus::new(cpu.clone(), mapper.clone(), mirror_v)
    }

    struct FakeCpu {
        interrupted: bool,
    }

    impl Default for FakeCpu {
        fn default() -> Self {
            FakeCpu {
                interrupted: false,
            }
        }
    }

    impl IoAccess for FakeCpu {
        fn raise_interrupt(&mut self) {
            self.interrupted = true;
        }
    }

    struct FakeMapper {
        prg: [u8; 10],
        chr: [u8; 10],
    }

    impl FakeMapper {
        fn new() -> Self {
            FakeMapper {
                prg: [0; 10],
                chr: [0; 10],
            }
        }
    }

    impl MapperControl for FakeMapper {
        fn read(&self, addr: u16) -> u8 {
            self.prg[addr as usize]
        }

        fn write(&mut self, addr: u16, data: u8) {
            self.prg[addr as usize] = data;
        }

        fn read_chr(&self, addr: u16) -> u8 {
            self.chr[addr as usize]
        }

        fn write_chr(&mut self, addr: u16, value: u8) {
            self.chr[addr as usize] = value;
        }
    }
}
