//! The synthesis kit: a noise source, oscillators, a two-pole resonator, and
//! envelopes. Small on purpose -- every sound the game makes is one of a
//! handful of physical events, and each needs only a few of these.
//!
//! Everything renders at [`RATE`] into `f32` samples in `-1..1`.

/// Samples per second. The game plays these through the engine's mixer, which
/// resamples if the device runs at anything else.
pub const RATE: u32 = 44_100;

/// Seconds to samples.
pub fn samples(seconds: f32) -> usize {
    (seconds.max(0.0) * RATE as f32) as usize
}

/// Simulation frames to seconds, at the simulation's 60 Hz.
pub fn frames(frames: u16) -> f32 {
    frames as f32 / sim::TICK_HZ as f32
}

/// White noise, from a tiny xorshift. Seeded, so a sound rendered twice is
/// the same sound.
pub struct Noise(u32);

impl Noise {
    pub fn new(seed: u32) -> Noise {
        Noise(seed | 1)
    }

    /// Uniform in `-1..1`.
    pub fn sample(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// A two-pole band-pass resonator: one mode of a struck body, or one formant
/// of a throat. Constant peak gain, so `q` changes how long it rings and
/// nothing else about its loudness.
#[derive(Clone, Copy)]
pub struct Resonator {
    b0: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Resonator {
    pub fn new(freq: f32, q: f32) -> Resonator {
        let mut r = Resonator {
            b0: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        };
        r.set(freq, q);
        r
    }

    /// Retune without clearing the state, so a sweep is continuous.
    pub fn set(&mut self, freq: f32, q: f32) {
        let freq = freq.clamp(20.0, RATE as f32 * 0.45);
        let w = std::f32::consts::TAU * freq / RATE as f32;
        let alpha = w.sin() / (2.0 * q.max(0.3));
        let a0 = 1.0 + alpha;
        // Band-pass, constant 0 dB peak: b0 = alpha, b1 = 0, b2 = -alpha.
        self.b0 = alpha / a0;
        self.a1 = -2.0 * w.cos() / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// What an impulse of one comes out as, at its peak: the inverse of the
    /// filter's bandwidth. A narrow mode passes almost none of a burst, so a
    /// struck body's bright modes are scaled by this to be heard at all.
    pub fn impulse_gain(&self) -> f32 {
        1.0 / self.b0.max(1e-6)
    }

    pub fn tick(&mut self, x: f32) -> f32 {
        // Direct form II transposed, with b1 = 0 and b2 = -b0.
        let y = self.b0 * x + self.z1;
        self.z1 = -self.a1 * y + self.z2;
        self.z2 = -self.b0 * x - self.a2 * y;
        y
    }
}

/// A one-pole low-pass: the dullness of a blunt blow, the muffling of snow.
#[derive(Clone, Copy)]
pub struct LowPass {
    k: f32,
    z: f32,
}

impl LowPass {
    pub fn new(cutoff: f32) -> LowPass {
        let mut l = LowPass { k: 1.0, z: 0.0 };
        l.set(cutoff);
        l
    }

    pub fn set(&mut self, cutoff: f32) {
        let cutoff = cutoff.clamp(10.0, RATE as f32 * 0.45);
        self.k = 1.0 - (-std::f32::consts::TAU * cutoff / RATE as f32).exp();
    }

    pub fn tick(&mut self, x: f32) -> f32 {
        self.z += self.k * (x - self.z);
        self.z
    }
}

/// A one-pole high-pass: the brightness of an edge, the removal of rumble.
#[derive(Clone, Copy)]
pub struct HighPass {
    low: LowPass,
}

impl HighPass {
    pub fn new(cutoff: f32) -> HighPass {
        HighPass {
            low: LowPass::new(cutoff),
        }
    }

    pub fn tick(&mut self, x: f32) -> f32 {
        x - self.low.tick(x)
    }
}

/// A sine at a frequency that may move.
#[derive(Clone, Copy, Default)]
pub struct Sine {
    phase: f32,
}

impl Sine {
    pub fn tick(&mut self, freq: f32) -> f32 {
        self.phase = (self.phase + freq / RATE as f32).fract();
        (self.phase * std::f32::consts::TAU).sin()
    }
}

/// A pulse train: the glottis. A narrow pulse has every harmonic, which is
/// what the formants downstream need to work on.
#[derive(Clone, Copy, Default)]
pub struct Pulse {
    phase: f32,
}

impl Pulse {
    pub fn tick(&mut self, freq: f32) -> f32 {
        let step = freq / RATE as f32;
        self.phase += step;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            1.0
        } else {
            // A little decaying tail after the pulse: a softer, breathier
            // source than a bare impulse.
            -0.1 * (1.0 - self.phase)
        }
    }
}

/// `exp(-t / tau)`: how a struck thing dies away.
pub fn decay(t: f32, tau: f32) -> f32 {
    (-t / tau.max(1e-4)).exp()
}

/// A rise from 0 to 1 over `len` seconds, eased so it starts soft: a sound
/// gathering itself.
pub fn rise(t: f32, len: f32) -> f32 {
    let u = (t / len.max(1e-4)).clamp(0.0, 1.0);
    u * u
}

/// A straight line between two values over `len` seconds, held after.
pub fn lerp(t: f32, len: f32, from: f32, to: f32) -> f32 {
    let u = (t / len.max(1e-4)).clamp(0.0, 1.0);
    from + (to - from) * u
}

/// Geometric between two values: the right way to sweep a frequency.
pub fn glide(t: f32, len: f32, from: f32, to: f32) -> f32 {
    let u = (t / len.max(1e-4)).clamp(0.0, 1.0);
    from * (to / from).powf(u)
}

/// Bring a rendered sound to a peak of `peak`. Done once at the end of every
/// render, so a patch's internal gains only have to be right relative to
/// each other, and the loudness of one sound against another is a single
/// number the patch states.
pub fn finish(out: &mut [f32], peak: f32) {
    let max = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if max <= 1e-6 {
        return;
    }
    let g = peak.clamp(0.0, 1.0) / max;
    for s in out.iter_mut() {
        *s *= g;
    }
    // A short fade at the very end, so a sound cut off by its own length
    // does not click.
    let tail = samples(0.004).min(out.len());
    let n = out.len();
    for i in 0..tail {
        out[n - 1 - i] *= i as f32 / tail as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resonator_rings_at_its_frequency_and_dies() {
        let mut r = Resonator::new(440.0, 20.0);
        let mut out = Vec::new();
        for i in 0..samples(0.2) {
            out.push(r.tick(if i == 0 { 1.0 } else { 0.0 }));
        }
        // Count zero crossings in the first 50 ms: about 2 * 440 * 0.05 = 44.
        let n = samples(0.05);
        let crossings = out[..n]
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();
        assert!((38..=50).contains(&crossings), "{crossings} crossings");
        // And it has died away by the end.
        let late = out[samples(0.18)..]
            .iter()
            .fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(late < out[1].abs() * 0.2, "still ringing at {late}");
    }

    #[test]
    fn finish_brings_the_peak_to_what_was_asked() {
        let mut out: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin() * 0.01).collect();
        finish(&mut out, 0.8);
        let max = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((max - 0.8).abs() < 0.05, "peak {max}");
    }

    #[test]
    fn noise_is_seeded() {
        let mut a = Noise::new(7);
        let mut b = Noise::new(7);
        for _ in 0..100 {
            assert_eq!(a.sample(), b.sample());
        }
    }
}
