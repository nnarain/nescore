//
// info.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 13 2020
//

use clap::Clap;
use nescore::Cartridge;

#[derive(Clap)]
pub struct Options {
    /// ROM file
    rom: String,
}

pub fn dispatch(opts: Options) {
    let (info, _, _) = Cartridge::from_path(&opts.rom).unwrap().to_parts();
    println!("{}", info);
}
