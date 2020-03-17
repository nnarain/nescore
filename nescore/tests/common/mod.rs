use nescore::{Nes, Cartridge};

pub fn init_nes(path: &str) -> Nes {
    Cartridge::from_path(path).map(|cart| Nes::default().with_cart(cart).debug_mode(true)).unwrap()
}

pub fn run_test(nes: &mut Nes, fail_msg: &str) {
    let mut result_text = String::from("");

    while !should_exit(&result_text) {
        nes.emulate_frame();
        result_text = read_result_text(&nes);
    }

    assert_eq!(result_text, "PASSED", "{}: \"{}\"", fail_msg, result_text);
}

fn should_exit(text: &String) -> bool {
    text.contains("PASS") || text.contains("FAIL")
}

// The result text is stored in VRAM.
pub fn read_result_text(nes: &Nes) -> String {
    const TEXT_ROW_POS: usize = 6;
    let text = (0..32)
               .map(|i| nes.read_tile(i, TEXT_ROW_POS) as char)
               .fold(String::from(""), |mut s, c|{
                   s.push(c);
                   s
               });

    String::from(text.trim())
}
