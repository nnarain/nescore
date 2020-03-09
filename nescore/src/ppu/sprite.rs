//
// ppu/sprite.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 07 2020
//

#[derive(Default, Copy, Clone)]
pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    attr: u8,
}

impl From<&[u8]> for Sprite {
    fn from(data: &[u8]) -> Self {
        Sprite {
            y: data[0],
            x: data[3],
            tile: data[1],
            attr: data[2],
        }
    }
}

impl Sprite {
    pub fn palette(&self) -> u8 {
        self.attr & 0x03
    }

    pub fn priority(&self) -> bool {
        bit_is_set!(self.attr, 5)
    }

    pub fn flip_v(&self) -> bool {
        bit_is_set!(self.attr, 7)
    }

    pub fn flip_h(&self) -> bool {
        bit_is_set!(self.attr, 6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_from_slice() {
        let data: [u8; 4] = [1, 2, 0, 3];
        let sprite = Sprite::from(&data[..]);

        assert_eq!(sprite.x, 3);
        assert_eq!(sprite.y, 1);
        assert_eq!(sprite.tile, 2);
    }


    #[test]
    fn sprite_attributes() {
        let data: [u8; 4] = [0, 0, 0xFF, 0];
        let sprite = Sprite::from(&data[..]);

        assert_eq!(sprite.palette(), 3);
        assert_eq!(sprite.priority(), true);
        assert_eq!(sprite.flip_v(), true);
        assert_eq!(sprite.flip_h(), true);
    }
}
