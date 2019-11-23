//
// clk.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Nov 21 2019
//

use crate::io::IoAccess;

/// A clockable component
pub trait Clockable {
    fn tick(&mut self, io: &mut dyn IoAccess);
}
