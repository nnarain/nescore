use nescore::{Nes, Cartridge};

#[test]
fn render_one_pixel() {
    let prg = vec![
                          // -- Set PPU ADDR to nametable address $2000
        0xA9, 0x20,       // LDA $20
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR
        0xA9, 0x00,       // LDA $00
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR

                          // -- Set PPU DATA to $01
        0xA9, 0x01,       // LDA $01
        0x8D, 0x07, 0x20, // STA $2007; PPU DATA

                          // -- Set PPU ADDR to attribute location $23C0
        0xA9, 0x23,       // LDA $23
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR
        0xA9, 0xC0,       // LDA $C0
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR

                          // -- Set PPU DATA to $01
        0xA9, 0x01,       // LDA $01
        0x8D, 0x07, 0x20, // STA $2007; PPU DATA

                          // -- Set PPU ADDR to palette location $3F05
        0xA9, 0x3F,       // LDA $3F
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR
        0xA9, 0x05,       // LDA $05
        0x8D, 0x06, 0x20, // STA $2006; PPU ADDR

                          // -- Set PPU DATA to $01
        0xA9, 0x01,       // LDA $01
        0x8D, 0x07, 0x20, // STA $2007; PPU DATA

        0x4C, 0x00, 0x80, // Loop
    ];

    let cart = init_cart(prg);
    let mut nes = Nes::default().with_cart(cart).entry(0x8000).debug_mode(true);

    let framebuffer = nes.emulate_frame();
    let rgb = (framebuffer[0], framebuffer[1], framebuffer[2]);

    assert_eq!(rgb, (0x00, 0x2D, 0x69));
}

fn  init_cart(mut prg_rom: Vec<u8>) -> Cartridge {
    let header = init_header(1, 1);

    // pattern data
    let mut chr_rom = [0x00u8; 0x2000];
    chr_rom[0x10] = 0x80;

    prg_rom.resize(0x4000, 0x00);

    let rom = [&header[..], &prg_rom[..], &chr_rom[..]].concat();

    Cartridge::from(rom).unwrap()
}

fn init_header(num_prg_banks: u8, num_chr_banks: u8) -> [u8; 16] {
    [
        0x4E, 0x45, 0x53, 0x1A, // NES<EOF>
        num_prg_banks,          // PRG ROM
        num_chr_banks,          // CHR ROM
        0x00,                   // Flag 6
        0x00,                   // Flag 7
        0x00,                   // Flag 8
        0x00,                   // Flag 9
        0x00,                   // Flag 10
        0x00,                   // Flag 11
        0x00,                   // Flag 12
        0x00,                   // Flag 13
        0x00,                   // Flag 14
        0x00,                   // Flag 15
    ]
}
