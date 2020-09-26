//
// utils/sampler.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Sep 19 2020
//
// TODO: Down Sampler trait?
use crate::specs::Sample;

/// Down Sampler
/// Reduce APU generated output to host system playback rate
#[derive(Debug)]
pub struct DownSampler {
    buffer: Vec<Sample>,
    rate: usize,
}

impl DownSampler {
    pub fn new(buffer: Vec<Sample>, input_rate: f32, output_rate: f32) -> Self {
        DownSampler {
            buffer,
            rate: (input_rate / output_rate) as usize,
        }
    }
}

impl IntoIterator for DownSampler {
    type Item = Sample;
    type IntoIter = std::iter::StepBy<std::vec::IntoIter<Sample>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter().step_by(self.rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample() {
        let buffer: Vec<Sample> = (0..100).map(|n| n as f32).collect();
        let count = DownSampler::new(buffer, 100.0, 10.0).into_iter().count();

        assert_eq!(count, 10);
    }
}
