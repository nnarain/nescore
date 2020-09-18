//
// common/audio.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 12 2020
//

use sdl2::audio::AudioCallback;

use nescore::specs::{Sample, SampleBuffer};

use std::collections::VecDeque;

#[derive(Default)]
pub struct AudioStreamSource {
    queue: VecDeque<Sample>,
}

impl AudioCallback for AudioStreamSource {
    type Channel = Sample;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for item in out.iter_mut() {
            if let Some(sample) = self.queue.pop_front() {
                *item = sample;
            }
            else {
                break;
            }
        }
    }
}

impl AudioStreamSource {
    pub fn update(&mut self, buffer: SampleBuffer) {
        for sample in buffer.iter() {
            self.queue.push_back(*sample);
        }
    }
}
