use clap::{App, Arg};

use std::io::prelude::*;
use std::fs::File;
use std::io::Error;

#[derive(Debug)]
struct ProgramOptions {
    pub filepath: String
}

fn main() {
    // get command line options
    match get_options() {
        Ok(options) => {
            match load_header(&options.filepath) {
                Ok(rom_header) => {
                    match nescore::cart::CartridgeInfo::from(&rom_header[..]) {
                        Ok(info) => println!("{}", info),
                        Err(e) => println!("{:?}", e)
                    }
                },
                Err(e) => println!("{:?}", e)
            }
        },
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn load_header(file: &String) -> Result<[u8; 32], Box<Error>> {
    let mut file = File::open(file)?;
    let mut buffer = [0; 32];
    file.read(&mut buffer)?;

    Ok(buffer)
}

fn get_options() -> Result<ProgramOptions, String> {
    let matches = App::new("nesinfo")
                        .version("1.0.0")
                        .author("Natesh Narain")
                        .about("Print iNES and NES2.0 cartridge header information")
                        .arg(Arg::with_name("file")
                                .short("f")
                                .long("file")
                                .value_name("FILE")
                                .help("NES ROM file")
                                .takes_value(true)
                                .required(true))
                        .get_matches();

    if let Some(opt) = matches.value_of("file") {
        Ok(
            ProgramOptions{
                filepath: opt.to_string()
            }
        )
    }
    else {
        Err(String::from("Error parsing options"))
    }
}