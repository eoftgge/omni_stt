const SCALE: f32 = i16::MAX as f32;
const TARGET_RATE: f64 = 16_000.0;

pub struct AudioConverter {
    ratio: f64,
    channels: usize,
    pos: f64,
    prev: f32,
    gain: f32,
}

impl AudioConverter {
    const MAX_GAIN: f32 = 5.0;
    const TARGET_PEAK: f32 = 0.9;
    const ATTACK: f32 = 0.3;
    const RELEASE: f32 = 0.05;

    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            ratio: sample_rate as f64 / TARGET_RATE,
            channels: channels.max(1) as usize,
            pos: 0.0,
            prev: 0.0,
            gain: 1.0,
        }
    }

    pub fn push(&mut self, input: &[f32], output: &mut Vec<i16>) {
        let ch = self.channels;
        let frames = input.len() / ch;
        if frames == 0 {
            return;
        }

        let mono = |idx: usize| -> f32 {
            let start = idx * ch;
            input[start..start + ch].iter().sum::<f32>() / ch as f32
        };

        let mut peak = 0.0f32;
        for i in 0..frames {
            peak = peak.max(mono(i).abs());
        }
        let target = if peak > 0.001 {
            (Self::TARGET_PEAK / peak).min(Self::MAX_GAIN)
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
                mono(left_idx as usize)
            };
            let sample = left + (mono(right_idx as usize) - left) * frac;

            output.push(((sample * self.gain).clamp(-1.0, 1.0) * SCALE) as i16);
            self.pos += self.ratio;
        }

        self.prev = mono(frames - 1);
        self.pos -= frames as f64;
    }
}
