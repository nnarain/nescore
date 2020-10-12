//
// common/audio.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 12 2020
//

use sdl2::audio::AudioCallback;

use nescore::specs::{Sample, SampleBuffer, APU_OUTPUT_RATE};
use nescore::utils::sampler::DownSampler;

use std::collections::VecDeque;

const HOST_AUDIO_RATE: f32 = 44100.0;

pub struct AudioStreamSource {
    queue: VecDeque<Sample>,
    output_rate: f32,
}

impl Default for AudioStreamSource {
    fn default() -> Self {
        AudioStreamSource {
            queue: VecDeque::new(),
            output_rate: HOST_AUDIO_RATE,
        }
    }
}

impl AudioCallback for AudioStreamSource {
    type Channel = Sample;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        // Dequeue into output buffer
        for item in out.iter_mut() {
            if let Some(sample) = self.queue.pop_front() {
                *item = sample;
            }
            else {
                // Exit early if there is no more data
                break;
            }
        }
    }
}

impl AudioStreamSource {
    pub fn update(&mut self, buffer: SampleBuffer) {
        for sample in DownSampler::new(buffer, APU_OUTPUT_RATE, self.output_rate).into_iter() {
            self.queue.push_back(sample);
        }
    }
}
