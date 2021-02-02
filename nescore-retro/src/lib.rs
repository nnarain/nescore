use nescore::{Nes, Cartridge, Button,
    specs::{DISPLAY_HEIGHT, DISPLAY_WIDTH, APU_OUTPUT_RATE, PixelFormat as NesCorePixelFormat},
    utils::sampler::DownSampler
};
use libretro_backend::{
    AudioVideoInfo, Core, CoreInfo, GameData, LoadGameResult,
    PixelFormat, Region, RuntimeHandle, JoypadButton,
    libretro_core
};

const HOST_PLAYBACK_RATE: f64 = 44100.0;

struct NescoreRetro {
    core: Nes,
    game_data: Option<GameData>,
}

impl Default for NescoreRetro {
    fn default() -> Self {
        NescoreRetro {
            core: Nes::default().pixel_format(NesCorePixelFormat::BGRA8),
            game_data: None,
        }
    }
}

impl Core for NescoreRetro {
    fn info() -> CoreInfo {
        CoreInfo::new("nescore", env!("CARGO_PKG_VERSION"))
            .supports_roms_with_extension("nes")
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if game_data.is_empty() {
            LoadGameResult::Failed(game_data)
        }
        else {
            let cart = if let Some(rom) = game_data.data() {
                Cartridge::from_slice(rom)
            }
            else if let Some(rom_path) = game_data.path() {
                Cartridge::from_path(rom_path)
            }
            else {
                unreachable!();
            };

            match cart {
                Ok(cart) => {
                    self.game_data = Some(game_data);
                    let tv_system = if cart.info.tv_system_pal { Region::PAL } else { Region::NTSC };
                    // FIXME: This will panic on invalid cartridge type
                    self.core.insert(cart);

                    LoadGameResult::Success(
                        AudioVideoInfo::new()
                            .video(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, 60.0, PixelFormat::ARGB8888)
                            .audio(HOST_PLAYBACK_RATE)
                            .region(tv_system)
                    )
                },
                Err(_) => LoadGameResult::Failed(game_data),
            }
        }
    }

    fn on_unload_game(&mut self) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        // Handle joypad key presses
        for btn in [JoypadButton::A, JoypadButton::B,
                    JoypadButton::Up, JoypadButton::Down, JoypadButton::Left, JoypadButton::Right,
                    JoypadButton::Select, JoypadButton::Start].iter() {
            // Check the button is support by nescore
            if let Ok(nescore_btn) = map_joypad(*btn) {
                self.core.input(nescore_btn, handle.is_joypad_button_pressed(0, *btn));
            }
        }

        // Run for a full frame
        let (framebuffer, audiobuffer) = self.core.emulate_frame();
        handle.upload_video_frame(framebuffer);

        // process audio to match host system and libretro api
        // downsample apu output
        // convert to i16 data type
        // convert to stereo
        let audiobuffer: Vec<i16> = {
            let mut buf: Vec<i16> = Vec::new();
            for sample in DownSampler::new(audiobuffer, APU_OUTPUT_RATE, HOST_PLAYBACK_RATE as f32)
                                        .into_iter()
                                        .map(|sample| sample as i16) {
                buf.push(sample);
                buf.push(sample);
            }

            buf
        };
        handle.upload_audio_frame(audiobuffer.as_slice());
    }

    fn on_reset(&mut self) {
        // TODO: Allow resetting the nescore
    }

    fn save_memory(&mut self) -> Option< &mut [u8] > {
        None
    }
}

fn map_joypad(button: JoypadButton) -> Result<Button, ()> {
    match button {
        JoypadButton::A => Ok(Button::A),
        JoypadButton::B => Ok(Button::B),
        JoypadButton::Up => Ok(Button::Up),
        JoypadButton::Down => Ok(Button::Down),
        JoypadButton::Left => Ok(Button::Left),
        JoypadButton::Right => Ok(Button::Right),
        JoypadButton::Start => Ok(Button::Select),
        _ => Err(()),
    }
}

libretro_core!(NescoreRetro);
