extern crate nesinfo;

#[macro_use]
extern crate serde_derive;
extern crate docopt;

use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

use docopt::Docopt;


#[derive(Debug, Deserialize)]
struct ProgramOptions {
    pub arg_input: String
}

fn main() {
    // get command line options
    let options = get_options();
    // load the rom header
    match load_header(&options.arg_input) {
        Ok(rom_header) => {
            match nesinfo::parse_header(&rom_header[..]) {
                Ok(header) => {
                    println!("{}", header);
                },
                Err(e) => println!("{:?}", e)
            }
        },
        Err(e) => println!("{}", e)
    }
}

fn load_header(file: &String) -> Result<[u8; 32], Box<Error>> {
    let mut file = File::open(file)?;
    let mut buffer = [0; 32];
    file.read(&mut buffer)?;

    Ok(buffer)
}

fn get_options() -> ProgramOptions {
    const USAGE: &'static str = "
    nesinfo

    Usage:
      nesinfo <input>
      nesinfo (-h | --help)

    Options:
      -h --help     Show help
    ";

    Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit())
}