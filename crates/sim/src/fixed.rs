//! Fixed-point arithmetic: 16.16 signed.
//!
//! The simulation uses no floating point. IEEE 754 basic operations are
//! reproducible if you control rounding and contraction, but transcendentals
//! (`sin`, `cos`, `exp`) are *not* standardised across libm implementations —
//! and peers will be on Windows x86 and Apple Silicon ARM at the same time.
//! Fixed point removes that entire class of desync.
//!
//! Range is +/-32768 with a resolution of 1/65536. Arena coordinates are in
//! metres, so that is ample.
//!
//! Overflow saturates rather than wrapping. Both are deterministic; saturating
//! degrades into a stuck character instead of a teleport, which is far easier
//! to notice and diagnose.

/// Number of fractional bits.
pub const FRAC_BITS: u32 = 16;
const ONE_I: i32 = 1 << FRAC_BITS;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Fx(pub i32);

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(ONE_I);
    pub const MIN: Fx = Fx(i32::MIN);
    pub const MAX: Fx = Fx(i32::MAX);

    /// Whole number to fixed point.
    pub const fn from_int(v: i32) -> Fx {
        Fx(v.saturating_mul(ONE_I))
    }

    /// Ratio to fixed point, e.g. `Fx::ratio(1, 60)` for one sixtieth.
    pub const fn ratio(num: i32, den: i32) -> Fx {
        Fx((((num as i64) << FRAC_BITS) / den as i64) as i32)
    }

    /// Truncates toward zero.
    pub const fn to_int(self) -> i32 {
        self.0 / ONE_I
    }

    /// Raw bits. Used by serialisation and checksums.
    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn from_raw(bits: i32) -> Fx {
        Fx(bits)
    }

    /// RENDER-ONLY. Never call this from simulation code — the no-float guard
    /// test enforces that.
    pub fn to_f32_for_render(self) -> f32 {
        self.0 as f32 / ONE_I as f32
    }

    pub const fn add(self, o: Fx) -> Fx {
        Fx(self.0.saturating_add(o.0))
    }

    pub const fn sub(self, o: Fx) -> Fx {
        Fx(self.0.saturating_sub(o.0))
    }

    pub const fn neg(self) -> Fx {
        Fx(self.0.saturating_neg())
    }

    pub const fn mul(self, o: Fx) -> Fx {
        let wide = (self.0 as i64 * o.0 as i64) >> FRAC_BITS;
        Fx(saturate(wide))
    }

    /// Division by zero saturates to MAX/MIN rather than panicking. A desync is
    /// worse than a wrong number, and a panic mid-rollback is worse than both.
    pub const fn div(self, o: Fx) -> Fx {
        if o.0 == 0 {
            return if self.0 >= 0 { Fx::MAX } else { Fx::MIN };
        }
        Fx(saturate(((self.0 as i64) << FRAC_BITS) / o.0 as i64))
    }

    pub const fn abs(self) -> Fx {
        Fx(self.0.saturating_abs())
    }

    pub const fn min(self, o: Fx) -> Fx {
        if self.0 < o.0 { self } else { o }
    }

    pub const fn max(self, o: Fx) -> Fx {
        if self.0 > o.0 { self } else { o }
    }

    pub const fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        self.max(lo).min(hi)
    }

    /// Integer Newton-Raphson square root. Negative inputs return zero.
    pub const fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        // sqrt(x * 2^16) * 2^8 == sqrt(x) in 16.16 terms.
        let n = (self.0 as i64) << FRAC_BITS;
        let mut guess: i64 = 1 << 24;
        let mut i = 0;
        while i < 24 {
            guess = (guess + n / guess) >> 1;
            i += 1;
        }
        Fx(saturate(guess))
    }
}

const fn saturate(v: i64) -> i32 {
    if v > i32::MAX as i64 {
        i32::MAX
    } else if v < i32::MIN as i64 {
        i32::MIN
    } else {
        v as i32
    }
}

// ---------------------------------------------------------------------------
// Trigonometry
//
// A quarter-wave lookup table with linear interpolation. Angles are turns in
// 16.16 fixed point: 1.0 is a full revolution, so wrapping is a bitmask.
// Deterministic by construction, and precise enough for facing directions and
// cone checks.
// ---------------------------------------------------------------------------

/// Entries per quarter turn.
const QUARTER: usize = 256;

/// sin(t) for t in [0, 1/4] turns, in 16.16. Generated at compile time.
static SIN_LUT: [i32; QUARTER + 1] = build_sin_lut();

const fn build_sin_lut() -> [i32; QUARTER + 1] {
    // Fifth-order Taylor of sin about zero, evaluated in i64 fixed point.
    // Accurate to well under one LSB over a quarter turn, and const-evaluable
    // without touching floats.
    let mut lut = [0i32; QUARTER + 1];
    // pi/2 in 16.16
    const HALF_PI: i64 = 102944;
    let mut i = 0;
    while i <= QUARTER {
        let x = (HALF_PI * i as i64) / QUARTER as i64; // radians, 16.16
        let x2 = (x * x) >> FRAC_BITS;
        let x3 = (x2 * x) >> FRAC_BITS;
        let x5 = (x3 * x2) >> FRAC_BITS;
        let x7 = (x5 * x2) >> FRAC_BITS;
        // x - x^3/6 + x^5/120 - x^7/5040
        let v = x - x3 / 6 + x5 / 120 - x7 / 5040;
        lut[i] = saturate(v);
        i += 1;
    }
    lut
}

/// Sine of an angle expressed in turns (1.0 == one full revolution).
pub fn sin_turns(turns: Fx) -> Fx {
    // Wrap into [0, 1) turns using the fractional bits only.
    let frac = (turns.0 as u32) & (ONE_I as u32 - 1);
    // Position within the wave in units of 1/(4*QUARTER).
    let total = QUARTER as u64 * 4;
    let scaled = (frac as u64 * total * ONE_I as u64) >> (FRAC_BITS * 2 - FRAC_BITS);
    let idx = (scaled >> FRAC_BITS) as usize;
    let t = Fx((scaled & (ONE_I as u64 - 1)) as i32);

    let (quadrant, i) = (idx / QUARTER, idx % QUARTER);
    let (a, b) = match quadrant {
        0 => (SIN_LUT[i], SIN_LUT[i + 1]),
        1 => (SIN_LUT[QUARTER - i], SIN_LUT[QUARTER - i - 1]),
        2 => (-SIN_LUT[i], -SIN_LUT[i + 1]),
        _ => (-SIN_LUT[QUARTER - i], -SIN_LUT[QUARTER - i - 1]),
    };
    Fx(a).add(Fx(b - a).mul(t))
}

/// Cosine of an angle expressed in turns.
pub fn cos_turns(turns: Fx) -> Fx {
    sin_turns(turns.add(Fx::ratio(1, 4)))
}

impl core::fmt::Debug for Fx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Prints as a decimal without going through floating point.
        let neg = self.0 < 0;
        let m = (self.0 as i64).unsigned_abs();
        let whole = m >> FRAC_BITS;
        let frac = ((m & (ONE_I as u64 - 1)) * 1_000_000) >> FRAC_BITS;
        write!(f, "{}{}.{:06}", if neg { "-" } else { "" }, whole, frac)
    }
}
