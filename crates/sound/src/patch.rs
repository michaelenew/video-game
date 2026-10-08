//! The instruments: what each kind of sound is, as a few numbers, and how it
//! is rendered.
//!
//! The thesis is that **a sound is an excitation shaped by a resonator**, and
//! the game's numbers decide both. A blow is a burst of noise -- as long and
//! as dull as the blow is heavy and blunt -- driving the modes of whatever
//! was struck, pitched by its size. A telegraph is a rise exactly as long as
//! the startup it warns of. A footfall is a blow on the floor's material. So
//! each patch below is a handful of parameters the simulation already has,
//! and `Patch::render` is the only place a waveform is made.
//!
//! The reference body is a fighter: `size` is in metres of height, and a
//! struck thing's modes scale as `1 / size` from where a 1.8 m body puts
//! them, so a knee-high gnawer rings high and a nine-metre animal rings low
//! without anybody authoring either.

use crate::synth::{
    self, HighPass, LowPass, Noise, Pulse, Resonator, Sine, decay, frames, glide, lerp, rise,
    samples,
};

/// What was struck, or what a foot came down on. The ringing is this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Material {
    /// A fighter's body: a damped thud with almost no ring.
    Flesh,
    /// A creature's hide: deeper, a little more ring.
    Hide,
    /// A creature's armour or a breakable part: hard and dull, a few partials.
    Plate,
    /// A shield: bright, metallic, rings.
    Shield,
    /// Dressed stone, and the Elementalist's stones.
    Stone,
    /// Bare rock.
    Rock,
    /// Packed earth. The proving ground.
    Earth,
    Grass,
    Sand,
    Snow,
    Ash,
    Peat,
    Water,
    Wood,
}

impl Material {
    /// The arena's floor materials, as what they sound like underfoot.
    pub fn of_floor(m: sim::arena::Material) -> Material {
        use sim::arena::Material as F;
        match m {
            F::Ground => Material::Earth,
            F::Grass => Material::Grass,
            F::Rock => Material::Rock,
            F::Stone => Material::Stone,
            F::Sand => Material::Sand,
            F::Snow => Material::Snow,
            F::Ash => Material::Ash,
            F::Peat => Material::Peat,
            F::Water => Material::Water,
            F::Wood => Material::Wood,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Material::Flesh => "flesh",
            Material::Hide => "hide",
            Material::Plate => "plate",
            Material::Shield => "shield",
            Material::Stone => "stone",
            Material::Rock => "rock",
            Material::Earth => "earth",
            Material::Grass => "grass",
            Material::Sand => "sand",
            Material::Snow => "snow",
            Material::Ash => "ash",
            Material::Peat => "peat",
            Material::Water => "water",
            Material::Wood => "wood",
        }
    }

    /// The modes of a thing of this material at the reference size: each a
    /// frequency, **how long it rings** (the time its amplitude takes to
    /// fall by `e`, in seconds), and how loud. Empty for things that do not
    /// ring at all and are heard only as their burst.
    ///
    /// A ring time rather than a Q, because a Q at a frequency *is* a ring
    /// time (`Q = pi * f * tau`) and the time is the thing an ear hears: a
    /// shield rings for half a second whatever note it rings at.
    fn modes(self) -> &'static [(f32, f32, f32)] {
        match self {
            Material::Flesh => &[(150.0, 0.012, 1.0), (260.0, 0.008, 0.35)],
            Material::Hide => &[(95.0, 0.03, 1.0), (170.0, 0.02, 0.5), (300.0, 0.012, 0.2)],
            Material::Plate => &[(380.0, 0.10, 1.0), (870.0, 0.08, 0.6), (1400.0, 0.06, 0.3)],
            Material::Shield => &[
                (620.0, 0.55, 1.0),
                (1710.0, 0.40, 0.7),
                (3350.0, 0.25, 0.4),
                (5200.0, 0.15, 0.2),
            ],
            Material::Stone => &[(540.0, 0.18, 1.0), (860.0, 0.14, 0.6), (1300.0, 0.10, 0.35)],
            Material::Rock => &[(330.0, 0.09, 1.0), (610.0, 0.07, 0.5), (1050.0, 0.05, 0.25)],
            Material::Earth => &[(105.0, 0.025, 1.0), (190.0, 0.015, 0.3)],
            Material::Grass => &[(120.0, 0.015, 0.6)],
            Material::Sand => &[],
            Material::Snow => &[],
            Material::Ash => &[],
            Material::Peat => &[(70.0, 0.04, 1.0)],
            Material::Water => &[],
            Material::Wood => &[(240.0, 0.08, 1.0), (480.0, 0.06, 0.5), (770.0, 0.045, 0.3)],
        }
    }

    /// The burst: how bright the contact itself is (a low-pass cutoff, in Hz,
    /// before the blow's own sharpness moves it), how long it lasts relative
    /// to a blow's, how loud it is against the modes, and how much of the
    /// low thump a heavy blow carries -- all of it on something soft, little
    /// on something that rings instead.
    fn burst(self) -> (f32, f32, f32, f32) {
        match self {
            Material::Flesh => (900.0, 1.0, 1.0, 1.0),
            Material::Hide => (700.0, 1.2, 1.0, 1.0),
            Material::Plate => (2500.0, 0.6, 0.8, 0.5),
            Material::Shield => (5000.0, 0.4, 0.5, 0.25),
            Material::Stone => (3500.0, 0.5, 0.7, 0.4),
            Material::Rock => (2500.0, 0.6, 0.9, 0.6),
            Material::Earth => (600.0, 1.3, 1.0, 1.0),
            Material::Grass => (1800.0, 1.6, 1.0, 0.8),
            Material::Sand => (4000.0, 2.5, 1.0, 0.4),
            Material::Snow => (1200.0, 3.0, 1.0, 0.6),
            Material::Ash => (900.0, 2.6, 1.0, 0.5),
            Material::Peat => (350.0, 2.2, 1.0, 1.0),
            Material::Water => (3000.0, 3.5, 1.0, 0.5),
            Material::Wood => (1500.0, 0.8, 0.9, 0.5),
        }
    }
}

/// A blow: something struck something.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    /// How hard, nought to one. A poke is about 0.2, a hammer about 0.9.
    pub weight: f32,
    /// Nought is blunt, one is an edge: how bright the contact is.
    pub sharp: f32,
    pub material: Material,
    /// How big the struck thing is, in metres. A fighter is 1.8.
    pub size: f32,
}

/// A body moving through the air: a swing, a dodge, a jump. Rising, it is a
/// telegraph that climbs for exactly `frames`; falling, it is the swing
/// itself, a short loud sweep down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Whoosh {
    pub frames: u16,
    pub size: f32,
    pub rising: bool,
    /// How loud, nought to one.
    pub weight: f32,
}

/// A creature gathering itself: a growl that rises for `frames`, pitched by
/// its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Growl {
    pub frames: u16,
    pub size: f32,
}

/// A clean chime: the parry. Nothing else in the game makes a sound like it,
/// which is the point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub pitch: f32,
}

/// Stone coming out of the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rumble {
    pub frames: u16,
    pub size: f32,
}

/// Fire catching.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crackle {
    pub seconds: f32,
}

/// Blood: a wet, dull burst with a drop in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wet {
    pub weight: f32,
}

/// Wind: the Elementalist's air, a wing's downwash.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gust {
    pub frames: u16,
    pub weight: f32,
}

/// Every instrument the game has.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Patch {
    Strike(Strike),
    Whoosh(Whoosh),
    Growl(Growl),
    Ring(Ring),
    Rumble(Rumble),
    Crackle(Crackle),
    Wet(Wet),
    Gust(Gust),
}

/// The reference body: a fighter's height, in metres.
pub const FIGHTER: f32 = 1.8;

impl Patch {
    /// Render to samples at [`synth::RATE`], peak at the patch's own loudness.
    pub fn render(&self) -> Vec<f32> {
        match self {
            Patch::Strike(s) => strike(s),
            Patch::Whoosh(w) => whoosh(w),
            Patch::Growl(g) => growl(g),
            Patch::Ring(r) => ring(r),
            Patch::Rumble(r) => rumble(r),
            Patch::Crackle(c) => crackle(c),
            Patch::Wet(w) => wet(w),
            Patch::Gust(g) => gust(g),
        }
    }

    /// A short name for a sheet or a log.
    pub fn name(&self) -> String {
        match self {
            Patch::Strike(s) => format!(
                "{} w{:.1} s{:.1} {:.1}m",
                s.material.name(),
                s.weight,
                s.sharp,
                s.size
            ),
            Patch::Whoosh(w) => format!(
                "{} {}f {:.1}m",
                if w.rising { "windup" } else { "swing" },
                w.frames,
                w.size
            ),
            Patch::Growl(g) => format!("growl {}f {:.1}m", g.frames, g.size),
            Patch::Ring(r) => format!("ring {:.0}Hz", r.pitch),
            Patch::Rumble(r) => format!("rumble {}f {:.1}m", r.frames, r.size),
            Patch::Crackle(c) => format!("crackle {:.1}s", c.seconds),
            Patch::Wet(w) => format!("wet w{:.1}", w.weight),
            Patch::Gust(g) => format!("gust {}f", g.frames),
        }
    }
}

/// Modes scale as `1 / size` from the reference body, softened a little so a
/// forty-metre colossus does not vanish below hearing.
fn size_ratio(size: f32) -> f32 {
    (FIGHTER / size.max(0.2)).powf(0.8)
}

fn strike(s: &Strike) -> Vec<f32> {
    let weight = s.weight.clamp(0.0, 1.0);
    let sharp = s.sharp.clamp(0.0, 1.0);
    let (bright, burst_len, burst_gain, thump_gain) = s.material.burst();
    // The contact: 3 ms for a flick, 30 ms for a hammer; the blunter the
    // longer. Its brightness is the material's, moved by the edge.
    let burst = (0.003 + 0.028 * weight * (1.2 - 0.6 * sharp)) * burst_len;
    let ratio = size_ratio(s.size);
    // The contact is duller on a bigger thing too: more mass meets the blow.
    let cutoff = bright * (0.5 + 1.5 * sharp) * (1.3 - 0.5 * weight) * ratio.powf(0.5);
    // Heavy and big ring longer: a blow that puts more in takes longer to
    // come out, and a bigger body has more of it to come out of.
    let ring_scale = (0.6 + 0.8 * weight) * (s.size / FIGHTER).powf(0.3);
    let longest = s
        .material
        .modes()
        .iter()
        .map(|(_, tau, _)| tau * ring_scale)
        .fold(0.0f32, f32::max);
    let length = 0.06 + 0.25 * weight + 0.08 * (s.size / FIGHTER).min(3.0) + longest * 3.0;
    let n = samples(length);
    let mut out = vec![0.0f32; n];
    let mut noise = Noise::new(0x5EED_0001 ^ (s.material as u32) << 8);
    let mut low = LowPass::new(cutoff);
    let mut high = HighPass::new(40.0 + 400.0 * sharp);
    let mut modes: Vec<(Resonator, f32)> = s
        .material
        .modes()
        .iter()
        .map(|(f, tau, g)| {
            let f = (f * ratio).max(36.0);
            // The ring time, as the resonator's own damping: Q = pi f tau.
            let q = std::f32::consts::PI * f * tau * ring_scale;
            // A resonator at constant peak gain passes less of a burst the
            // narrower it is, so a long-ringing mode would vanish: its gain
            // is the inverse of its width, which is what lets a shield ring
            // over the contact that struck it.
            let r = Resonator::new(f, q);
            (r, g * r.impulse_gain())
        })
        .collect();
    // The burst is many impulses; a mode summed over them comes out about
    // this many times louder than over one, so the modes are brought back
    // to the burst's own scale.
    let mode_level = 1.2 / (samples(burst) as f32).sqrt().max(1.0);
    // The body of a heavy blow: a low thump under everything, pitched by
    // the struck thing's size.
    let mut thump = Sine::default();
    let thump_f = 55.0 * ratio.powf(0.5);
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let exciting =
            decay(t, burst * 0.5) * if t < burst { 1.0 } else { 0.0 } + decay(t, burst) * 0.3;
        let x = high.tick(low.tick(noise.sample())) * exciting;
        // An edge is heard as its contact: more of the burst, less of the
        // body's low modes and almost none of the thump. A club is the
        // other way about.
        let mut y = x * burst_gain * (0.6 + 1.2 * sharp);
        for (r, gain) in modes.iter_mut() {
            // Each mode rings out at its own rate: the resonator's damping
            // is the material's ring time, so nothing else shapes it.
            y += r.tick(x) * *gain * mode_level * (0.6 + 0.4 * weight) * (1.1 - 0.6 * sharp);
        }
        y += thump.tick(thump_f)
            * weight
            * weight
            * 0.7
            * thump_gain
            * (1.0 - 0.7 * sharp)
            * decay(t, 0.05 + 0.08 * weight);
        *out = y;
    }
    synth::finish(&mut out, 0.35 + 0.65 * weight);
    out
}

fn whoosh(w: &Whoosh) -> Vec<f32> {
    let len = frames(w.frames.max(2));
    let ratio = size_ratio(w.size).powf(0.5);
    let n = samples(len + 0.03);
    let mut out = vec![0.0f32; n];
    let mut noise = Noise::new(0x5EED_0002);
    let mut band = Resonator::new(400.0, 3.0);
    let (f0, f1) = if w.rising {
        (180.0 * ratio, 1400.0 * ratio)
    } else {
        (1600.0 * ratio, 350.0 * ratio)
    };
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let f = glide(t, len, f0, f1);
        band.set(f, 2.5);
        let env = if w.rising {
            // Gathering: soft, then louder, then cut at the moment the
            // startup ends -- the ear reads how far along it is.
            rise(t, len) * if t < len { 1.0 } else { decay(t - len, 0.015) }
        } else {
            // The swing: a fast rise to a peak a third of the way in, and a
            // tail.
            let peak = len * 0.3;
            if t < peak {
                rise(t, peak)
            } else {
                decay(t - peak, len * 0.4)
            }
        };
        *out = band.tick(noise.sample()) * env;
    }
    synth::finish(&mut out, (0.25 + 0.5 * w.weight).min(1.0));
    out
}

fn growl(g: &Growl) -> Vec<f32> {
    let len = frames(g.frames.max(4));
    let n = samples(len + 0.08);
    let mut out = vec![0.0f32; n];
    let size = g.size.max(0.3);
    // The pitch: a fighter-sized throat at 110 Hz, a nine-metre one at the
    // bottom of hearing. It climbs a fifth over the gathering.
    let f0 = (110.0 * (FIGHTER / size).powf(0.6)).clamp(22.0, 260.0);
    // Formants scale less steeply: a big throat is still a throat.
    let fr = (FIGHTER / size).powf(0.35);
    let mut source = Pulse::default();
    let mut noise = Noise::new(0x5EED_0003);
    let mut f1 = Resonator::new(420.0 * fr, 8.0);
    let mut f2 = Resonator::new(1150.0 * fr, 10.0);
    let mut f3 = Resonator::new(2400.0 * fr, 12.0);
    let mut breath = LowPass::new(1200.0 * fr);
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let pitch = glide(t, len, f0, f0 * 1.5) * (1.0 + 0.02 * (t * 37.0).sin());
        let x = source.tick(pitch) + breath.tick(noise.sample()) * 0.25;
        let env = rise(t, len) * if t < len { 1.0 } else { decay(t - len, 0.03) };
        *out = (f1.tick(x) * 1.0 + f2.tick(x) * 0.6 + f3.tick(x) * 0.3) * env;
    }
    synth::finish(&mut out, 0.7);
    out
}

fn ring(r: &Ring) -> Vec<f32> {
    let n = samples(0.6);
    let mut out = vec![0.0f32; n];
    let mut a = Sine::default();
    let mut b = Sine::default();
    let mut c = Sine::default();
    let mut noise = Noise::new(0x5EED_0004);
    let mut click = Resonator::new(r.pitch * 3.0, 6.0);
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let x = noise.sample() * if t < 0.004 { 1.0 } else { 0.0 };
        *out = a.tick(r.pitch) * decay(t, 0.25)
            + b.tick(r.pitch * 2.51) * 0.5 * decay(t, 0.12)
            + c.tick(r.pitch * 4.2) * 0.2 * decay(t, 0.06)
            + click.tick(x) * 2.0;
    }
    synth::finish(&mut out, 0.8);
    out
}

fn rumble(r: &Rumble) -> Vec<f32> {
    let len = frames(r.frames.max(6));
    let n = samples(len + 0.25);
    let mut out = vec![0.0f32; n];
    let ratio = size_ratio(r.size);
    let mut noise = Noise::new(0x5EED_0005);
    let mut low = LowPass::new(70.0 * ratio.powf(0.5));
    let mut gravel = Resonator::new(1400.0 * ratio.powf(0.5), 5.0);
    let mut grit = Noise::new(0x5EED_0006);
    let mut modes: Vec<Resonator> = Material::Stone
        .modes()
        .iter()
        .map(|(f, q, _)| Resonator::new(f * ratio, *q))
        .collect();
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let x = noise.sample();
        let env = if t < len {
            lerp(t, len * 0.3, 0.3, 1.0)
        } else {
            decay(t - len, 0.12)
        };
        // Sparse clicks: grit shearing. Denser as it comes up.
        let density = 0.004 + 0.02 * (t / len).min(1.0);
        let pop = if grit.sample().abs() < density {
            grit.sample()
        } else {
            0.0
        };
        let mut y = low.tick(x) * 6.0 * env + gravel.tick(pop) * 1.5 * env;
        // The settle: the stone's own modes, struck as it locks in place.
        if t >= len {
            let hit = if t - len < 0.003 { noise.sample() } else { 0.0 };
            for m in modes.iter_mut() {
                y += m.tick(hit) * 2.0 * decay(t - len, 0.08);
            }
        }
        *out = y;
    }
    synth::finish(&mut out, 0.7);
    out
}

fn crackle(c: &Crackle) -> Vec<f32> {
    let n = samples(c.seconds.max(0.1));
    let mut out = vec![0.0f32; n];
    let mut noise = Noise::new(0x5EED_0007);
    let mut roar = LowPass::new(350.0);
    let mut pops = Noise::new(0x5EED_0008);
    let mut pop_band = Resonator::new(2800.0, 4.0);
    let mut pop_left = 0usize;
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let env = rise(t, 0.08)
            * if t > c.seconds - 0.15 {
                decay(t - (c.seconds - 0.15), 0.06)
            } else {
                1.0
            };
        if pop_left == 0 && pops.sample().abs() < 0.0012 {
            pop_left = samples(0.002 + 0.004 * pops.sample().abs());
        }
        let pop = if pop_left > 0 {
            pop_left -= 1;
            pops.sample()
        } else {
            0.0
        };
        *out = (roar.tick(noise.sample()) * 5.0 + pop_band.tick(pop) * 2.5) * env;
    }
    synth::finish(&mut out, 0.5);
    out
}

fn wet(w: &Wet) -> Vec<f32> {
    let weight = w.weight.clamp(0.0, 1.0);
    let n = samples(0.12 + 0.2 * weight);
    let mut out = vec![0.0f32; n];
    let mut noise = Noise::new(0x5EED_0009);
    let mut low = LowPass::new(900.0);
    let mut plop = Sine::default();
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        low.set(glide(t, 0.1, 1400.0, 300.0));
        let splash = low.tick(noise.sample()) * decay(t, 0.03 + 0.05 * weight);
        let drop = plop.tick(glide(t, 0.08, 320.0, 70.0)) * decay(t, 0.05) * (0.5 + weight);
        *out = splash + drop;
    }
    synth::finish(&mut out, 0.3 + 0.5 * weight);
    out
}

fn gust(g: &Gust) -> Vec<f32> {
    let len = frames(g.frames.max(6));
    let n = samples(len + 0.1);
    let mut out = vec![0.0f32; n];
    let mut noise = Noise::new(0x5EED_000A);
    let mut band = Resonator::new(500.0, 1.5);
    for (i, out) in out.iter_mut().enumerate() {
        let t = i as f32 / synth::RATE as f32;
        let u = (t / len).min(1.0);
        let f = 300.0 + 700.0 * (u * std::f32::consts::PI).sin();
        band.set(f, 1.2);
        let env = (u * std::f32::consts::PI).sin().max(0.0)
            * if t > len { decay(t - len, 0.05) } else { 1.0 };
        *out = band.tick(noise.sample()) * env;
    }
    synth::finish(&mut out, 0.2 + 0.5 * g.weight);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0f32, |m, s| m.max(s.abs()))
    }

    use crate::spectrum::centroid;

    #[test]
    fn every_patch_renders_something_finite_at_its_peak() {
        let all = [
            Patch::Strike(Strike {
                weight: 0.5,
                sharp: 0.5,
                material: Material::Flesh,
                size: FIGHTER,
            }),
            Patch::Whoosh(Whoosh {
                frames: 20,
                size: FIGHTER,
                rising: true,
                weight: 0.5,
            }),
            Patch::Growl(Growl {
                frames: 30,
                size: 5.0,
            }),
            Patch::Ring(Ring { pitch: 1800.0 }),
            Patch::Rumble(Rumble {
                frames: 12,
                size: 2.0,
            }),
            Patch::Crackle(Crackle { seconds: 0.4 }),
            Patch::Wet(Wet { weight: 0.5 }),
            Patch::Gust(Gust {
                frames: 20,
                weight: 0.5,
            }),
        ];
        for p in all {
            let v = p.render();
            assert!(v.len() > 100, "{}", p.name());
            assert!(v.iter().all(|s| s.is_finite()), "{}", p.name());
            assert!(peak(&v) > 0.1, "{} is silent", p.name());
        }
    }

    #[test]
    fn a_heavier_blow_is_louder_and_longer() {
        let light = strike(&Strike {
            weight: 0.2,
            sharp: 0.3,
            material: Material::Flesh,
            size: FIGHTER,
        });
        let heavy = strike(&Strike {
            weight: 0.9,
            sharp: 0.3,
            material: Material::Flesh,
            size: FIGHTER,
        });
        assert!(peak(&heavy) > peak(&light));
        assert!(heavy.len() > light.len());
    }

    #[test]
    fn a_bigger_body_rings_lower() {
        let small = strike(&Strike {
            weight: 0.5,
            sharp: 0.3,
            material: Material::Hide,
            size: 0.6,
        });
        let big = strike(&Strike {
            weight: 0.5,
            sharp: 0.3,
            material: Material::Hide,
            size: 9.0,
        });
        assert!(
            centroid(&small) > centroid(&big) * 1.5,
            "{} vs {}",
            centroid(&small),
            centroid(&big)
        );
    }

    #[test]
    fn an_edge_is_brighter_than_a_club() {
        let edge = strike(&Strike {
            weight: 0.5,
            sharp: 1.0,
            material: Material::Flesh,
            size: FIGHTER,
        });
        let club = strike(&Strike {
            weight: 0.5,
            sharp: 0.0,
            material: Material::Flesh,
            size: FIGHTER,
        });
        assert!(
            centroid(&edge) > centroid(&club),
            "{} vs {}",
            centroid(&edge),
            centroid(&club)
        );
    }

    #[test]
    fn a_shield_rings_longer_than_flesh() {
        let shield = strike(&Strike {
            weight: 0.5,
            sharp: 0.5,
            material: Material::Shield,
            size: FIGHTER,
        });
        let flesh = strike(&Strike {
            weight: 0.5,
            sharp: 0.5,
            material: Material::Flesh,
            size: FIGHTER,
        });
        // Energy left after 80 ms, as a share of the peak.
        let late = |v: &[f32]| peak(&v[samples(0.08).min(v.len() - 1)..]) / peak(v);
        assert!(
            late(&shield) > late(&flesh) * 2.0,
            "{} vs {}",
            late(&shield),
            late(&flesh)
        );
    }

    #[test]
    fn a_telegraph_is_exactly_as_long_as_its_startup() {
        for f in [12u16, 30, 45] {
            let v = whoosh(&Whoosh {
                frames: f,
                size: FIGHTER,
                rising: true,
                weight: 0.5,
            });
            let len = frames(f);
            // Loudest in the last tenth before the cut, quiet soon after.
            let before = peak(&v[samples(len * 0.9)..samples(len)]);
            let after = peak(&v[samples(len + 0.025)..]);
            assert!(
                before > peak(&v) * 0.7,
                "{f} frames: {before} of {}",
                peak(&v)
            );
            assert!(after < before * 0.3, "{f} frames: {after} after the cut");
        }
    }
}
