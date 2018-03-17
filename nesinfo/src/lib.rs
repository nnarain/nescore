
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Format {
    INES,
    NES2
}

#[derive(Debug)]
pub struct Header {
    format: Format,
}

pub enum ParseError {
    InvalidSize(usize),
    InvalidSig,
    InvalidFormat
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidSig =>     write!(f, "Invalid signature at start of file. Expected `NES`. Not an NES ROM"),
            ParseError::InvalidSize(s) => write!(f, "Not enough data to parse header (Size: {})", s),
            ParseError::InvalidFormat =>  write!(f, "The detected header is not valid")
        }
    }
}

/// Parse NES ROM header
pub fn parse_header(rom_header: &[u8]) -> Result<Header, ParseError> {
    if rom_header.len() < 16 {
        return Err(ParseError::InvalidSize(rom_header.len()))
    }

    if !verify_signature(&rom_header[0..4]) {
        return Err(ParseError::InvalidSig)
    }

    let format = match get_format(rom_header) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    Ok(Header {
        format: format
    })
}

/// Get the NES ROM format
fn get_format(rom_header: &[u8]) -> Result<Format, ParseError> {
    let flag7 = rom_header[7];

    if (flag7 & 0x0Cu8) == 0x08u8 {
        return Ok(Format::NES2);
    }
    else {
        // if this is an INES format rom, bytes 8-15 should be $00
        let empty_bytes = &rom_header[8..16];

        if empty_bytes == [0, 0, 0, 0, 0, 0, 0, 0] {
            return Ok(Format::INES);
        }
        else {
            return Err(ParseError::InvalidFormat)
        }
    }
}

/// Verify the signature at the start of the file `NES<EOF>`
fn verify_signature(sig: &[u8]) -> bool {
    sig == [0x4E, 0x45, 0x53, 0x1A]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_format_nes2() {
        let mut rom_header = [0; 16];
        rom_header[7] = 0x08u8;

        let format = get_format(&rom_header[..]).unwrap();
        assert_eq!(format, Format::NES2);
    }

    #[test]
    fn test_get_format_ines() {
        let mut rom_header = [0; 16];
        rom_header[7] = 0x04u8;

        let format = get_format(&rom_header[..]).unwrap();
        assert_eq!(format, Format::INES);
    }
}
