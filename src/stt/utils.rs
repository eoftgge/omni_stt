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

/// Root mean square of the chunk, on the same 0..32767 scale as the samples.
pub fn rms(buffer: &[i16]) -> u32 {
    if buffer.is_empty() {
        return 0;
    }

    let sum_squares: u64 = buffer.iter().map(|&s| (s as i64 * s as i64) as u64).sum();
    (sum_squares / buffer.len() as u64).isqrt() as u32
}

pub fn is_silent(buffer: &[i16], threshold: u32) -> bool {
    rms(buffer) < threshold
}
