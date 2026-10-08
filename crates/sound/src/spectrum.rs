//! Looking at a sound: a small FFT, a spectrogram, and a spectral centroid.
//!
//! For the sheet (`examples/sheet.rs`) and for the tests that hold the
//! instruments to their physics -- a bigger body rings lower, an edge is
//! brighter than a club -- which are claims about where the energy is, and
//! this is how that is measured.

use crate::synth::RATE;

/// An in-place radix-2 FFT. `re.len()` must be a power of two.
pub fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two() && im.len() == n);
    // Bit reversal.
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -std::f32::consts::TAU / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * cr - im[b] * ci, re[b] * ci + im[b] * cr);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
        }
        len <<= 1;
    }
}

/// Magnitudes of one windowed frame of `samples` starting at `at`, `win`
/// long (a power of two): `win / 2` bins from 0 Hz to half the rate.
pub fn frame(samples: &[f32], at: usize, win: usize) -> Vec<f32> {
    let mut re = vec![0.0f32; win];
    let mut im = vec![0.0f32; win];
    for (i, r) in re.iter_mut().enumerate() {
        let s = samples.get(at + i).copied().unwrap_or(0.0);
        // Hann.
        let w = 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / win as f32).cos();
        *r = s * w;
    }
    fft(&mut re, &mut im);
    (0..win / 2)
        .map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt() / (win as f32 / 4.0))
        .collect()
}

/// A spectrogram: one magnitude frame every `hop` samples.
pub fn spectrogram(samples: &[f32], win: usize, hop: usize) -> Vec<Vec<f32>> {
    let mut out = Vec::new();
    let mut at = 0;
    // The last frame is padded with silence, so a sound shorter than a
    // window is still measured.
    while at < samples.len() {
        out.push(frame(samples, at, win));
        at += hop;
    }
    out
}

/// The frequency of a bin.
pub fn bin_hz(bin: usize, win: usize) -> f32 {
    bin as f32 * RATE as f32 / win as f32
}

/// Where the energy is, in Hz, over the whole sound: the power-weighted
/// mean frequency.
pub fn centroid(samples: &[f32]) -> f32 {
    let win = 2048;
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for f in spectrogram(samples, win, win / 2) {
        for (k, m) in f.iter().enumerate().skip(1) {
            let p = (*m as f64).powi(2);
            num += p * bin_hz(k, win) as f64;
            den += p;
        }
    }
    if den == 0.0 { 0.0 } else { (num / den) as f32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sine_peaks_in_its_own_bin() {
        let win = 1024;
        let f = bin_hz(100, win);
        let s: Vec<f32> = (0..win)
            .map(|i| (std::f32::consts::TAU * f * i as f32 / RATE as f32).sin())
            .collect();
        let m = frame(&s, 0, win);
        let peak = m
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert_eq!(peak, 100);
        assert!((centroid(&s) - f).abs() < f * 0.05, "{}", centroid(&s));
    }
}
