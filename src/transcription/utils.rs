const SCALE: f32 = i16::MAX as f32;

/// it's function appends audio.
pub fn convert_audio_chunk(input: &[f32], output: &mut Vec<i16>, channels: u16, sample_rate: u32) {
    let ch = channels as usize;
    if ch == 0 || input.is_empty() {
        return;
    }

    let ratio = sample_rate as f32 / 16000.0;
    let total_frames = input.len() / ch;

    let mut peak: f32 = 0.0;
    for frame in input.chunks_exact(ch) {
        peak = peak.max((frame.iter().sum::<f32>() / ch as f32).abs());
    }

    const MAX_GAIN: f32 = 5.0;
    const TARGET_PEAK: f32 = 0.9;
    let gain = if peak > 0.001 {
        (TARGET_PEAK / peak).min(MAX_GAIN)
    } else {
        1.0
    };

    let target_frames = (total_frames as f32 / ratio) as usize;
    output.reserve(target_frames);
    for i in 0..target_frames {
        let src_idx = (i as f32 * ratio) as usize;
        if src_idx >= total_frames {
            break;
        }
        let start = src_idx * ch;
        let mono = input[start..start + ch].iter().sum::<f32>() / ch as f32;
        output.push(((mono * gain).clamp(-1.0, 1.0) * SCALE) as i16);
    }
}

pub fn is_punctuation_or_symbol(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_punctuation() || "。，！？；：「」『』《》、…—।॥".contains(c))
}

pub fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x3040..=0x309F).contains(&u)
        || (0x30A0..=0x30FF).contains(&u)
        || (0xAC00..=0xD7AF).contains(&u)
}
