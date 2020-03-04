//
// cpu/bus.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

use crate::common::{IoAccess, IoAccessRef};
use crate::mapper::Mapper;

pub struct CpuIoBus {
    ppu: IoAccessRef,
    mapper: Mapper,
}

fn mirror_address(addr: u16, base: u16, count: u16) -> u16 {
    base + (addr % count)
}

impl CpuIoBus {
    pub fn new(ppu_io: IoAccessRef, mapper: Mapper) -> Self {
        CpuIoBus {
            ppu: ppu_io,
            mapper: mapper,
        }
    }
}

impl IoAccess for CpuIoBus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3FFF => self.ppu.borrow().read_byte(mirror_address(addr, 0x2000, 8)),
            0x4000..=0x401F => {
                // APU and IO
                0
            },
            0x4020..=0xFFFF => self.mapper.borrow().read(addr),
            _ => {
                panic!("Invalid address range")
            }
        }
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3FFF => {
                // First 8 bytes are mirrored up to $3FFF
                self.ppu.borrow_mut().write_byte(mirror_address(addr, 0x2000, 8), data);
            },
            0x4014 => {
                self.ppu.borrow_mut().write_byte(addr, data);
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::{MapperControl};

    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn mirroring_function() {
        let addr1 = mirror_address(0x2000, 0x2000, 8);
        let addr2 = mirror_address(0x2008, 0x2000, 8);

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn ppu_mirrored_registers() {
        let ppu = Rc::new(RefCell::new(FakePpu::new()));
        let mapper = Rc::new(RefCell::new(FakeMapper::new()));

        let mut bus = CpuIoBus::new(ppu, mapper);

        for i in 0..8 {
            bus.write_byte(0x2000 + i, i as u8);
        }

        assert_eq!(bus.read_byte(0x2000), 0);
        assert_eq!(bus.read_byte(0x2001), 1);
        assert_eq!(bus.read_byte(0x2002), 2);
        assert_eq!(bus.read_byte(0x2003), 3);
        assert_eq!(bus.read_byte(0x2004), 4);
        assert_eq!(bus.read_byte(0x2005), 5);
        assert_eq!(bus.read_byte(0x2006), 6);
        assert_eq!(bus.read_byte(0x2007), 7);

        assert_eq!(bus.read_byte(0x2008), 0);
        assert_eq!(bus.read_byte(0x2009), 1);
        assert_eq!(bus.read_byte(0x200A), 2);
        assert_eq!(bus.read_byte(0x200B), 3);
        assert_eq!(bus.read_byte(0x200C), 4);
        assert_eq!(bus.read_byte(0x200D), 5);
        assert_eq!(bus.read_byte(0x200E), 6);
        assert_eq!(bus.read_byte(0x200F), 7);
    }

    //------------------------------------------------------------------------------------------------------------------
    // Helpers
    //------------------------------------------------------------------------------------------------------------------

    struct FakePpu {
        data: [u8; 10],
    }

    impl FakePpu {
        fn new() -> Self {
            FakePpu {
                data: [0; 10],
            }
        }
    }

    impl IoAccess for FakePpu {
        fn read_byte(&self, addr: u16) -> u8 {
            self.data[(addr as usize) - 0x2000]
        }

        fn write_byte(&mut self, addr: u16, data: u8) {
            self.data[(addr as usize) - 0x2000] = data;
        }
    }

    struct FakeMapper {
        data: [u8; 10],
    }

    impl FakeMapper {
        fn new() -> Self {
            FakeMapper {
                data: [0; 10],
            }
        }
    }

    impl MapperControl for FakeMapper {
        fn read(&self, addr: u16) -> u8 {
            self.data[addr as usize]
        }

        fn write(&mut self, addr: u16, data: u8) {
            self.data[addr as usize] = data;
        }

        fn read_chr(&self, _addr: u16) -> u8 {
            0
        }

        fn write_chr(&mut self, _addr: u16, _value: u8) {

        }
    }

}
