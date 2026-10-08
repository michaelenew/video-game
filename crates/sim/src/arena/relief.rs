//! The floor's relief: rises and dips, where an arena has them.
//!
//! Every arena's floor was one plane at `y = 0`, and the simulation said so in
//! exactly one place -- the floor term of `ground_among` -- with the aiming
//! ray's floor hit and the creature's fence agreeing by writing the same
//! zero. This is that zero made a function of where you stand: a short table
//! of **bumps** per arena, each a smooth hill or hollow, summed. Smooth so a
//! body walks up and down it without a step, and few so a frame of collision
//! costs a handful of multiplies more.
//!
//! A bump is `h · (1 - d²/r²)²` inside its radius and nothing outside: flat
//! at its crown and flat again at its rim, so it joins the plain without an
//! edge. Fixed point throughout; `d²/r²` is at most one, so nothing here can
//! overflow.
//!
//! **Where the bumps go is hand-authored and keeps clear** of everything the
//! fight lays on the floor -- hazards, boulders, sites, the water and the
//! trodden ground -- since those are drawn flat and the simulation puts them
//! at the floor it finds. The proving ground and every course stay flat.
//! See `docs/design/forms.md` §"Relief".

use super::ArenaId;
use crate::fixed::Fx;

/// One hill (or, with a negative height, one hollow).
#[derive(Clone, Copy, Debug)]
pub struct Bump {
    pub x: Fx,
    pub z: Fx,
    pub radius: Fx,
    pub height: Fx,
}

impl Bump {
    /// From centimetres, like the arenas' own tables.
    pub const fn cm(x: i32, z: i32, radius: i32, height: i32) -> Bump {
        Bump {
            x: Fx::ratio(x, 100),
            z: Fx::ratio(z, 100),
            radius: Fx::ratio(radius, 100),
            height: Fx::ratio(height, 100),
        }
    }
}

/// The low meadow (48 by 40): a long swell to the west where the hunters
/// walk in, a rise in the middle the two spawn on, a hollow in the south
/// where the herd grazes and another at the south-west corner, rises
/// against the bank and by the ford. Clear of the four boulders
/// (`species::hornback::rules::boulders`), the trunk and the ford.
const MEADOW: [Bump; 12] = [
    Bump::cm(-1700, -800, 900, 90),
    Bump::cm(-1100, -300, 600, 30),
    Bump::cm(-2000, -1300, 500, -20),
    Bump::cm(300, -100, 450, 25),
    Bump::cm(-1500, 1000, 500, -35),
    Bump::cm(-200, -400, 750, 60),
    Bump::cm(800, -1000, 600, -50),
    Bump::cm(1600, 1000, 600, 60),
    Bump::cm(-300, 1300, 400, 30),
    Bump::cm(1900, 0, 500, 35),
    Bump::cm(-1200, -1500, 450, -25),
    Bump::cm(600, 200, 500, 30),
];

/// The Commons (36 by 30): a rise to the east the two boulders stand in, a
/// swell in the open middle, a hollow to the south-west where the trail comes
/// in and a dimple in the north-east corner. Clear of the den's mouth, the
/// carcass and the fallen trunk; the floor may rise *into* a solid's base,
/// never fall away under one.
const COMMONS: [Bump; 9] = [
    Bump::cm(900, -200, 700, 65),
    Bump::cm(400, -700, 450, 25),
    Bump::cm(-500, 100, 400, -15),
    Bump::cm(-800, -1000, 450, -30),
    Bump::cm(-1200, 300, 450, 35),
    Bump::cm(500, 500, 400, 25),
    Bump::cm(0, -400, 600, 40),
    Bump::cm(1400, 600, 400, -25),
    Bump::cm(-1400, -1100, 350, 20),
];

/// The Pan (36 by 36): dunes against the north-west and south-east corners,
/// a scoop to the south, drifts at the other corners and a low one where the
/// hunters walk in. Clear of the three islands and their lee.
const PAN: [Bump; 9] = [
    Bump::cm(-1150, 1150, 620, 75),
    Bump::cm(-700, 1500, 400, 25),
    Bump::cm(1500, -700, 400, 25),
    Bump::cm(1200, -1200, 650, 75),
    Bump::cm(100, -1350, 450, -40),
    Bump::cm(-1300, -900, 500, 35),
    Bump::cm(1300, 1100, 500, 40),
    Bump::cm(-1300, 0, 500, 25),
    Bump::cm(300, 800, 400, -20),
];

/// The Ashwood (33 by 33 inside its wall): drifts of ash and snow between
/// the trunks and round the braziers, a hollow east where the creature
/// starts. Clear of the braziers (sites, which stand on the plain), the
/// stream, and never a hollow under a trunk.
const ASHWOOD: [Bump; 9] = [
    Bump::cm(0, 400, 500, 50),
    Bump::cm(400, 800, 350, 20),
    Bump::cm(-500, -1100, 350, 20),
    Bump::cm(-100, -700, 450, 35),
    Bump::cm(-1200, 1100, 450, 55),
    Bump::cm(900, 300, 400, -25),
    Bump::cm(-1100, -500, 400, 30),
    Bump::cm(1200, -1100, 450, 35),
    Bump::cm(0, 1350, 300, -20),
];

/// An arena's bumps. Empty for a flat floor, which is most of them.
pub fn of(id: ArenaId) -> &'static [Bump] {
    match id {
        ArenaId::HORNBACK => &MEADOW,
        ArenaId::GNAWERS => &COMMONS,
        ArenaId::SANDMAW => &PAN,
        ArenaId::VEILSTALKER => &ASHWOOD,
        _ => &[],
    }
}

/// The floor's height at a point: zero on a flat floor.
pub fn height_at(id: ArenaId, x: Fx, z: Fx) -> Fx {
    let mut h = Fx::ZERO;
    for b in of(id) {
        let dx = x.sub(b.x);
        let dz = z.sub(b.z);
        if dx.abs().raw() >= b.radius.raw() || dz.abs().raw() >= b.radius.raw() {
            continue;
        }
        let q = dx.mul(dx).add(dz.mul(dz)).div(b.radius.mul(b.radius));
        if q.raw() >= Fx::ONE.raw() {
            continue;
        }
        let s = Fx::ONE.sub(q);
        h = h.add(b.height.mul(s).mul(s));
    }
    h
}
