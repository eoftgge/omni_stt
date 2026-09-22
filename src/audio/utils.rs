/// Root-mean-square of the chunk, on the same 0..32767 scale as the samples.
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
