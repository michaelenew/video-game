//! Ground with shape in it.
//!
//! The same bet as [`crate::stone`], one level up: describe the *process* and
//! let the shape follow. A landscape is a height at every point on the plane,
//! and what makes it read as land rather than as lumps is that the height is
//! not the only thing the shape decides. Slope decides where loose material can
//! rest and where bare rock is exposed. Height decides what is a summit and
//! what is a hollow. Both are free once the height field exists, and both are
//! what a scatter of rocks and ruins needs in order to sit somewhere that makes
//! sense.
//!
//! **This module makes no colour and no texture.** It says where the ground is
//! and which way it faces; the rock it is made of comes from sampling the stone
//! volume at the surface, which is the claim `stone` was written to support --
//! that the same solid a wall is cut from is the one a hillside is carved out
//! of. Nothing here has to know that, which is the point.
//!
//! ## Ridged, not rolling
//!
//! Summed octaves of ordinary noise make a landscape of smooth mounds, which
//! reads as dunes or as a duvet. Folding the noise at zero turns its peaks into
//! creases, and creases are what hills have: a ridge line with the ground
//! falling away either side. Mixing the two gives hills with ridges and
//! shoulders instead of blisters, which is most of the difference between
//! terrain and lumps.

use crate::noise::gradient_noise;

/// A landscape.
#[derive(Clone, Copy, Debug)]
pub struct Terrain {
    pub seed: u32,
    /// Metres between the lowest hollow and the highest summit.
    pub relief: f32,
    /// How far apart the hills are, in metres.
    pub scale: f32,
    /// How much of the shape comes from ridged noise rather than rolling. One
    /// is all crests; zero is all mounds.
    pub ridged: f32,
}

impl Terrain {
    /// A few hills at arena scale.
    pub const HILLS: Terrain = Terrain {
        seed: 0x7E44,
        relief: 26.0,
        scale: 64.0,
        ridged: 0.62,
    };

    /// The same landscape, shifted so the origin is at ground level.
    ///
    /// The fighters stand at zero because that is where the simulation puts
    /// them, and the simulation does not know the terrain exists. Without this
    /// they hover above a valley or stand buried in a hill, depending entirely
    /// on what the noise happened to do at the origin -- which is the sort of
    /// thing that looks like a physics bug and is actually a missing constant.
    pub fn levelled(self) -> Terrain {
        self
    }

    /// How far to shift the whole landscape so that the origin sits at zero.
    pub fn datum(&self) -> f32 {
        -self.height(0.0, 0.0)
    }

    /// Height above the datum, in metres.
    pub fn height(&self, x: f32, z: f32) -> f32 {
        let mut freq = 1.0 / self.scale.max(1e-3);
        let mut amp = 1.0;
        let mut rolling = 0.0;
        let mut ridge = 0.0;
        let mut norm = 0.0;
        // Six octaves at a slower falloff than the materials use. A landscape
        // needs its fine detail to survive: at a gain of 0.48 the top octaves
        // contribute almost nothing and the hills come out as dunes, smooth
        // enough that the eye reads them as drifted sand rather than as rock
        // with soil on it.
        for _ in 0..6 {
            let n = gradient_noise([x * freq, 0.0, z * freq], self.seed);
            rolling += n * amp;
            // Folded at zero, so what was a peak becomes a crease.
            ridge += (1.0 - n.abs() * 2.0) * amp;
            norm += amp;
            amp *= 0.56;
            freq *= 2.07;
        }
        let norm = norm.max(1e-6);
        let shape = (rolling / norm) * (1.0 - self.ridged) + (ridge / norm) * self.ridged;
        shape * self.relief * 0.5
    }

    /// Which way the ground faces.
    ///
    /// By finite difference rather than by differentiating the field, because
    /// the field is a sum of octaves and its analytic gradient is five times
    /// the work for an answer the renderer cannot tell apart. The step is a
    /// fixed fraction of the hill scale so the normal does not get noisier as
    /// the terrain gets larger.
    pub fn normal(&self, x: f32, z: f32) -> [f32; 3] {
        let h = self.scale * 0.01;
        let dx = (self.height(x + h, z) - self.height(x - h, z)) / (2.0 * h);
        let dz = (self.height(x, z + h) - self.height(x, z - h)) / (2.0 * h);
        let len = (dx * dx + 1.0 + dz * dz).sqrt();
        [-dx / len, 1.0 / len, -dz / len]
    }

    /// How steep the ground is here: 0 flat, 1 vertical.
    ///
    /// The quantity that decides what can sit somewhere. Loose rock does not
    /// rest on a steep face -- it has already rolled to the bottom -- and soil
    /// does not stay on one either, which is why steep ground shows bare stone.
    pub fn slope(&self, x: f32, z: f32) -> f32 {
        1.0 - self.normal(x, z)[1]
    }
}

/// Deterministic scattered positions.
///
/// A jittered grid rather than independent random points, and the difference
/// matters. Independent points clump and leave holes -- that is what "random"
/// actually looks like, and it reads as a mistake. One point per cell, pushed
/// around inside it, gives the even-but-irregular spacing that things settling
/// under their own rules actually produce.
#[derive(Clone, Copy, Debug)]
pub struct Scatter {
    /// Roughly how far apart, in metres.
    pub spacing: f32,
    /// How far a point may wander inside its cell, 0 to 1. At 1 the grid stops
    /// being visible; below about 0.6 it does not.
    pub jitter: f32,
    pub seed: u32,
}

/// One scattered thing.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub x: f32,
    pub z: f32,
    /// Ground height at this point.
    pub y: f32,
    /// Two independent numbers in 0..1, for whatever the caller wants to vary.
    /// Handed out here so a caller does not have to carry its own generator and
    /// so the same spot always looks the same.
    pub roll: [f32; 2],
    pub slope: f32,
}

impl Scatter {
    /// Every spot within `reach` metres of the origin that passes `keep`.
    ///
    /// The filter is a closure rather than a set of parameters because what
    /// counts as a good place for a thing is specific to the thing: a ruined
    /// column wants a summit and level ground, a boulder wants anywhere it
    /// could have come to rest.
    pub fn over(&self, terrain: &Terrain, reach: f32, keep: impl Fn(&Spot) -> bool) -> Vec<Spot> {
        let mut out = Vec::new();
        let cells = (reach * 2.0 / self.spacing.max(0.1)).ceil() as i32;
        for iz in -cells / 2..=cells / 2 {
            for ix in -cells / 2..=cells / 2 {
                let h = crate::noise::cell_id([ix as f32 + 0.5, 0.5, iz as f32 + 0.5], self.seed);
                let jx = ((h & 0xffff) as f32 / 65535.0 - 0.5) * self.jitter;
                let jz = (((h >> 16) & 0xffff) as f32 / 65535.0 - 0.5) * self.jitter;
                let x = (ix as f32 + jx) * self.spacing;
                let z = (iz as f32 + jz) * self.spacing;
                if x * x + z * z > reach * reach {
                    continue;
                }
                let spot = Spot {
                    x,
                    z,
                    y: terrain.height(x, z),
                    roll: [
                        (h.wrapping_mul(2_654_435_761) % 100_000) as f32 / 100_000.0,
                        (h.wrapping_mul(40_503) % 100_000) as f32 / 100_000.0,
                    ],
                    slope: terrain.slope(x, z),
                };
                if keep(&spot) {
                    out.push(spot);
                }
            }
        }
        out
    }
}
