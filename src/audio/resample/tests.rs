use super::*;

fn sine(frames: usize) -> Vec<f32> {
    (0..frames).map(|i| (i as f32 * 0.05).sin() * 0.5).collect()
}

/// Two summed sines at the input rate, scaled so the pair peaks at 0.9 and the
/// AGC never has to clip — clipping would smear energy across the spectrum and
/// pollute the very bins these tests read.
fn two_tones(rate: u32, low: f64, high: f64, frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|i| {
            let t = std::f64::consts::TAU * i as f64 / rate as f64;
            (((low * t).sin() + (high * t).sin()) * 0.45) as f32
        })
        .collect()
}

/// Amplitude of `freq` in `signal`, by a single-bin DFT.
fn amplitude_at(signal: &[i16], freq: f64, rate: f64) -> f64 {
    let (mut re, mut im) = (0.0, 0.0);
    for (i, &sample) in signal.iter().enumerate() {
        let phase = std::f64::consts::TAU * freq * i as f64 / rate;
        re += sample as f64 * phase.cos();
        im += sample as f64 * phase.sin();
    }
    2.0 * re.hypot(im) / signal.len() as f64
}

fn db(value: f64, reference: f64) -> f64 {
    20.0 * (value / reference).log10()
}

fn convert(input: &[f32], chunk: usize) -> Vec<i16> {
    let mut c = AudioConverter::new(48_000, 1);
    let mut out = Vec::new();
    for part in input.chunks(chunk) {
        c.push(part, &mut out);
    }
    out
}

fn convert_at(rate: u32, input: &[f32], chunk: usize) -> Vec<i16> {
    let mut c = AudioConverter::new(rate, 1);
    let mut out = Vec::new();
    for part in input.chunks(chunk) {
        c.push(part, &mut out);
    }
    out
}

#[test]
fn out_of_band_tone_does_not_fold_into_the_speech_band() {
    // 12 kHz cannot exist at 16 kHz. Undecimated, it mirrors onto 4 kHz at full
    // strength — right where fricatives live, so the recogniser cannot tell the
    // difference between it and real speech.
    let input = two_tones(48_000, 500.0, 12_000.0, 96_000);
    let out = convert(&input, 480);

    let settled = &out[out.len() / 2..];
    let reference = amplitude_at(settled, 500.0, 16_000.0);
    let alias = amplitude_at(settled, 4_000.0, 16_000.0);

    assert!(
        db(alias, reference) < -20.0,
        "alias at 4 kHz sits {:.1} dB below the 500 Hz reference, expected under -20",
        db(alias, reference)
    );
}

#[test]
fn the_speech_band_is_not_tilted() {
    let input = two_tones(48_000, 500.0, 3_000.0, 96_000);
    let out = convert(&input, 480);

    let settled = &out[out.len() / 2..];
    let low = amplitude_at(settled, 500.0, 16_000.0);
    let high = amplitude_at(settled, 3_000.0, 16_000.0);

    assert!(
        db(high, low).abs() < 0.5,
        "3 kHz sits {:.2} dB from 500 Hz; the anti-alias filter must not reach this far down",
        db(high, low)
    );
}

#[test]
fn non_integer_ratio_keeps_phase_across_buffers() {
    let out = convert_at(44_100, &sine(44_100), 512);
    assert!(
        out.len().abs_diff(16_000) <= 2,
        "44.1 кГц дали {} отсчётов вместо ~16000",
        out.len()
    );
}

#[test]
fn stereo_is_downmixed_without_changing_length() {
    let stereo: Vec<f32> = sine(48_000).iter().flat_map(|&s| [s, s]).collect();
    let mut c = AudioConverter::new(48_000, 2);
    let mut out = Vec::new();
    for part in stereo.chunks(512 * 2) {
        c.push(part, &mut out);
    }
    assert!(out.len().abs_diff(16_000) <= 2, "получили {}", out.len());
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

#[test]
fn target_peak_sets_the_output_level() {
    let mut converter = super::AudioConverter::new(16_000, 1).with_target_peak(0.45);
    let input: Vec<f32> = (0..16_000).map(|i| (i as f32 * 0.05).sin()).collect();
    let mut output = Vec::new();
    for block in input.chunks(160) {
        converter.push(block, &mut output);
    }

    // Second half only: the gain needs a few blocks to settle.
    let peak = output[8_000..].iter().map(|s| s.unsigned_abs()).max().unwrap();
    let level = peak as f32 / SCALE;
    assert!((level - 0.45).abs() < 0.05, "peak at {level}, expected 0.45");
}

#[test]
fn a_max_length_block_fits_the_preallocated_buffers() {
    let mut converter = AudioConverter::new(48_000, 2);
    let scratch = converter.mono.capacity();
    let block = vec![0.5; converter.max_block_len()];

    // Worst case: one sample short of a full chunk before the push.
    let mut output = Vec::with_capacity(crate::audio::CHUNK_CAPACITY);
    output.resize(crate::audio::CHUNK_SAMPLES - 1, 0);
    converter.push(&block, &mut output);

    assert_eq!(converter.mono.capacity(), scratch);
    assert_eq!(output.capacity(), crate::audio::CHUNK_CAPACITY);
}
