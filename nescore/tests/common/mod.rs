use nescore::{Nes, Cartridge};

pub fn init_nes(path: &str) -> Nes {
    Cartridge::from_path(path).map(|cart| Nes::default().with_cart(cart).debug_mode(false)).unwrap()
}

pub fn run_test(nes: &mut Nes, text_row: usize, fail_msg: &str) {
    let mut result_text = String::from("");

    while !should_exit(&result_text) {
        nes.emulate_frame();
        result_text = read_result_text(&nes, text_row);
    }

    // Run another few times to let the test ROM finish writing text to the screen
    for _ in 0..5 {
        nes.emulate_frame();
    }

    result_text = read_result_text(&nes, text_row);

    assert_eq!(result_text, "passed", "{}: \"{}\"", fail_msg, result_text);
}

fn should_exit(text: &String) -> bool {
    text.contains("pass") || text.contains("fail")
}

// The result text is stored in VRAM.
pub fn read_result_text(nes: &Nes, text_row: usize) -> String {
    let text = (0..32)
               .map(|i| nes.read_tile(0x2000, i, text_row) as char)
               .fold(String::from(""), |mut s, c|{
                   s.push(c);
                   s
               });

    String::from(text.trim()).to_ascii_lowercase()
}
