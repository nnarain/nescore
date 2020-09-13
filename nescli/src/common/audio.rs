//
// common/audio.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 12 2020
//

use sdl2::audio::AudioCallback;
use nescore::Sample;

pub struct AudioStreamSource {
    buffers: [Vec<Sample>; 2],
    buffer_idx: usize,
}

impl Default for AudioStreamSource {
    fn default() -> Self {
        AudioStreamSource {
            buffers: [vec![], vec![]],
            buffer_idx: 0,
        }
    }
}

impl AudioCallback for AudioStreamSource {
    type Channel = Sample;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for (i, out) in out.iter_mut().enumerate() {
            let buffer = &self.buffers[self.buffer_idx];
            if i < buffer.len() {
                *out = buffer[i];
            }
        }
    }
}

impl AudioStreamSource {
    pub fn update(&mut self, buffer: Vec<Sample>) {
        self.buffers[self.buffer_idx] = buffer;
        self.buffer_idx = (self.buffer_idx + 1) % self.buffers.len();
    }
}