//
// img.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 13 2020
//
use clap::Clap;

use image::{RgbImage, Rgb};
use nescore::Cartridge;
use nescore::cart::CHR_ROM_BANK_SIZE;

const BYTES_PER_TILE: usize = 16;
const OUTPUT_TILES_PER_ROW: usize = 32;
const TILES_PER_BANK: usize = CHR_ROM_BANK_SIZE / BYTES_PER_TILE;

#[derive(Clap)]
pub struct Options {
    /// ROM file
    rom: String,
    /// Output file name
    #[clap(short = 'o', long = "output", default_value = "chr_rom.png")]
    output: String,
}

struct Tile {
    data: [u8; 64],
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            data: [0; 64],
        }
    }
}

impl Tile {
    pub fn from(data: &[u8]) -> Self {
        let mut tile = Tile::default();

        for y in 0..8 {
            let lo = data[y];
            let hi = data[y + 8];

            for x in (0..8).rev() {
                let b0 = (lo >> x) & 0x01;
                let b1 = (hi >> x) & 0x01;

                let p = (b1 << 1) | b0;

                tile.set_data(x, y, p);
            }
        }

        tile
    }

    pub fn set_data(&mut self, x: usize, y: usize, data: u8) {
        self.data[(y * 8) + x] = data;
    }

    pub fn get_data(&self, x: usize, y: usize) -> u8 {
        self.data[(y * 8) + x]
    }
}

struct TileProvider {
    data: Vec<u8>,  // CHR data
    tile_no: usize, // Current tile
    max_tiles: usize, // Max number of tiles
}

impl TileProvider {
    pub fn from(data: Vec<u8>) -> Self {
        let data_len = data.len();

        TileProvider {
            data,
            tile_no: 0,
            max_tiles: data_len / BYTES_PER_TILE,
        }
    }
}

impl Iterator for TileProvider {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tile_no != self.max_tiles {
            let offset = self.tile_no * BYTES_PER_TILE;
            self.tile_no += 1;
            Some(Tile::from(&self.data[offset..offset+BYTES_PER_TILE]))
        }
        else {
            None
        }
    }
}

fn pattern_to_color(pattern: u8) -> (u8, u8, u8) {
    match pattern {
        0 => (0, 0, 0),
        1 => (51, 51, 51),
        2 => (153, 153, 153),
        3 => (255, 255, 255),
        _ => panic!("invalid: {}", pattern),
    }
}

pub fn dispatch(opts: Options) {
    let (_, _, chr_rom, _) = Cartridge::from_path(&opts.rom).unwrap().into_parts();

    // Create a tile provider to iterate over tile data
    let provider = TileProvider::from(chr_rom);

    // Determine image size
    let width = OUTPUT_TILES_PER_ROW * 8;
    let height = TILES_PER_BANK / OUTPUT_TILES_PER_ROW * 8;

    // Create an image buffer to dump data to
    let mut img = RgbImage::new(width as u32, height as u32);

    for (i, tile) in provider.enumerate() {
        let tile_row = i / OUTPUT_TILES_PER_ROW;
        let tile_col = i % OUTPUT_TILES_PER_ROW;
        let img_row = tile_row * 8;
        let img_col = tile_col * 8;

        for x in 0..8 {
            for y in 0..8 {
                let ix = (img_col + x) as u32;
                let iy = (img_row + y) as u32;
                let (r, g, b) = pattern_to_color(tile.get_data(x, y));
                img.put_pixel(ix, iy, Rgb([r, g, b]));
            }
        }
    }

    img.save(&opts.output).unwrap();
}
