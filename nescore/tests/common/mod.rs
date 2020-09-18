use nescore::{Nes, Cartridge};

pub fn init_nes(path: &str) -> Nes {
    Cartridge::from_path(path).map(|cart| Nes::default().with_cart(cart).debug_mode(false)).unwrap()
}

pub fn run_test(nes: &mut Nes, fail_msg: &str) {
    run_test_with_ignore(nes, fail_msg, vec![]);
}

pub fn run_test_with_ignore(nes: &mut Nes, fail_msg: &str, ignore: Vec<String>) {
    let mut result_text = String::from("");

    while !should_exit(&result_text) {
        nes.emulate_frame();
        result_text = read_result_text(&nes);
    }

    // Run another few times to let the test ROM finish writing text to the screen
    for _ in 0..5 {
        nes.emulate_frame();
    }

    result_text = read_result_text(&nes);

    let test_passed = result_text.contains("passed");
    // FIXME: I don't think this will reveal new breakages
    let ignore_failed = ignore.iter().fold(false, |r, s| r | result_text.contains(s));

    assert!(test_passed || ignore_failed, "{}:\n{}", fail_msg, result_text);
}

fn should_exit(text: &str) -> bool {
    text.contains("pass") || text.contains("fail")
}

// The result text is stored in VRAM.
pub fn read_result_text(nes: &Nes) -> String {
    let text = (0..30).map(|row|{
        (0..32)
        .map(|col| nes.read_tile(0x2000, col, row) as char)
        .fold(String::from(""), |s, c| format!("{}{}", s, c))
    })
    .fold(String::from(""), |a, s| format!("{}\n{}", a, String::from(s.trim())))
    .to_ascii_lowercase();

    String::from(text.trim())
}
