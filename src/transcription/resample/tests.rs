use super::*;

fn sine(frames: usize) -> Vec<f32> {
    (0..frames).map(|i| (i as f32 * 0.05).sin() * 0.5).collect()
}

fn convert(input: &[f32], chunk: usize) -> Vec<i16> {
    let mut c = AudioConverter::new(48_000, 1);
    let mut out = Vec::new();
    for part in input.chunks(chunk) {
        c.push(part, &mut out);
    }
    out
}

#[test]
fn output_length_does_not_depend_on_buffer_size() {
    let input = sine(48_000);
    let a = convert(&input, 512);
    let b = convert(&input, 480);
    assert!(
        a.len().abs_diff(b.len()) <= 1,
        "буферы по 512 дали {}, по 480 — {}",
        a.len(),
        b.len()
    );
}

#[test]
fn one_second_of_48k_yields_about_16000_samples() {
    let out = convert(&sine(48_000), 512);
    assert!(
        out.len().abs_diff(16_000) <= 2,
        "получили {} отсчётов вместо ~16000",
        out.len()
    );
}
