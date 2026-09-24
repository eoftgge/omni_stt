use crate::audio::FULL_SCALE_PEAK;

#[cfg(test)]
mod tests;

pub mod biquad;

const SCALE: f32 = i16::MAX as f32;
const TARGET_RATE: f64 = 16_000.0;

/// Just under the 8 kHz output Nyquist: -3 dB at the corner, -6 dB at Nyquist,
/// and everything below 5 kHz is left untouched.
const CUTOFF: f32 = 7_000.0;

/// Pole Qs of a 4th-order Butterworth, as a cascade of two biquads.
const BUTTERWORTH_Q: [f32; 2] = [0.541_196, 1.306_563];

pub struct AudioConverter {
    ratio: f64,
    channels: usize,
    pos: f64,
    prev: f32,
    gain: f32,
    target_peak: f32,
    anti_alias: Option<[biquad::Biquad; 2]>,
    mono: Vec<f32>,
    /// Frames `push` takes without growing `mono`; see `max_block_len`.
    max_frames: usize,
}

impl AudioConverter {
    const MAX_GAIN: f32 = 5.0;
    const ATTACK: f32 = 0.3;
    const RELEASE: f32 = 0.05;

    pub fn new(sample_rate: u32, channels: u16) -> Self {
        let ratio = sample_rate as f64 / TARGET_RATE;

        // At or below the target rate nothing is decimated and nothing can
        // alias; filtering anyway would only shave the top off a band we are
        // keeping in full.
        let anti_alias = (ratio > 1.0).then(|| {
            let fs = sample_rate as f32;
            BUTTERWORTH_Q.map(|q| biquad::Biquad::low_pass(fs, CUTOFF, q))
        });
        let max_frames = (sample_rate as usize / 10).max(1);

        Self {
            ratio,
            channels: channels.max(1) as usize,
            pos: 0.0,
            prev: 0.0,
            gain: 1.0,
            target_peak: FULL_SCALE_PEAK,
            anti_alias,
            max_frames,
            mono: Vec::with_capacity(max_frames),
        }
    }

    /// Lowers the level this converter aims for.
    ///
    /// Two captures that will be summed take half of full scale each, so the
    /// sum still fits instead of clipping on every loud moment. The `MAX_GAIN`
    /// ceiling keeps a quiet source from being dragged up to match a loud one.
    pub fn with_target_peak(mut self, target_peak: f32) -> Self {
        self.target_peak = target_peak;
        self
    }

    /// Longest input slice, in samples, that `push` handles without growing
    /// its scratch buffer: 100 ms. A push that long adds at most 1602 samples
    /// at 16 kHz, which is what keeps a chunk within `CHUNK_CAPACITY`.
    pub fn max_block_len(&self) -> usize {
        self.max_frames * self.channels
    }

    pub fn push(&mut self, input: &[f32], output: &mut Vec<i16>) {
        let ch = self.channels;
        let frames = input.len() / ch;
        if frames == 0 {
            return;
        }

        // Downmix once into a buffer we own: the filter below needs the samples
        // in order, and the interpolation after it needs random access.
        self.mono.clear();
        self.mono.reserve(frames);
        for i in 0..frames {
            let start = i * ch;
            self.mono
                .push(input[start..start + ch].iter().sum::<f32>() / ch as f32);
        }

        if let Some(sections) = &mut self.anti_alias {
            for sample in &mut self.mono {
                for section in sections.iter_mut() {
                    *sample = section.process(*sample);
                }
            }
        }

        let mut peak = 0.0f32;
        for &sample in &self.mono {
            peak = peak.max(sample.abs());
        }
        let target = if peak > 0.001 {
            (self.target_peak / peak).min(Self::MAX_GAIN)
        } else {
            self.gain
        };
        let alpha = if target < self.gain {
            Self::ATTACK
        } else {
            Self::RELEASE
        };
        self.gain += (target - self.gain) * alpha;

        output.reserve((frames as f64 / self.ratio).ceil() as usize + 1);

        while self.pos < frames as f64 {
            let base = self.pos.floor();
            let frac = (self.pos - base) as f32;
            let left_idx = base as isize;
            let right_idx = left_idx + 1;

            if right_idx >= frames as isize {
                break;
            }

            let left = if left_idx < 0 {
                self.prev
            } else {
                self.mono[left_idx as usize]
            };
            let sample = left + (self.mono[right_idx as usize] - left) * frac;

            output.push(((sample * self.gain).clamp(-1.0, 1.0) * SCALE) as i16);
            self.pos += self.ratio;
        }

        self.prev = self.mono[frames - 1];
        self.pos -= frames as f64;
    }
}
