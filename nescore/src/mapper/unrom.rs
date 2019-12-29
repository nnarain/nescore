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
use crate::cart::Cartridge;


/// UNROM Mapper
pub struct Unrom {
    rom_bank_selection: usize, // Select ROM bank
}

impl Unrom {
    pub fn from(cart: Cartridge) -> Self {
        Unrom{
            rom_bank_selection: 0,
        }
    }
}

impl MapperControl for Unrom {
    fn read(&self, addr: u16) -> u8 {
        0
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.rom_bank_selection = data as usize;
            },
            _ => {
                
            }
        }
    }
}

#[cfg(test)]
mod tests {
    
}
