use nescore::{Nes, Cartridge};

use nescore::{Instruction, AddressingMode};
use nescore::events::CpuEvent;


#[test]
fn cpu_event() {
    let prg = vec![
                          // Wait for VBlank
        0xA9, 0x80,       // LDA $80
        0x2D, 0x02, 0x20, // AND $2002
        0xF0, 0xF9,       // BEQ $FE

        0xA9, 0x00,       // LDA $00
        0xF0, 0xFE,       // BEQ $FE
    ];

    let cart = init_cart(prg);
    let mut nes = Nes::default().with_cart(cart).entry(0x8000).debug_mode(true);

    let rx = nes.cpu_event_channel();

    nes.emulate_frame();

    let event1 = rx.recv().unwrap();

    match event1 {
        CpuEvent::Instruction(data) => {
            assert_eq!(data.instr, Instruction::LDA);
            assert_eq!(data.mode, AddressingMode::Immediate);
            assert_eq!(data.a, 0x00);
        },
    }

    let event2 = rx.recv().unwrap();

    match event2 {
        CpuEvent::Instruction(data) => {
            assert_eq!(data.instr, Instruction::AND);
            assert_eq!(data.mode, AddressingMode::Absolute);
            assert_eq!(data.a, 0x80)
        },
    }
}

fn  init_cart(mut prg_rom: Vec<u8>) -> Cartridge {
    let header = init_header(1, 1);

    // pattern data
    let mut chr_rom = [0x00u8; 0x2000];
    chr_rom[0x10] = 0x80;
    chr_rom[0x27] = 0x01;

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
