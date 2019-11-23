//
// io.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

pub trait IoAccess {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, data: u8);
}
