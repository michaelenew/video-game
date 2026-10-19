//! **The Blood mage's ground** (2026-10-18): the territory her blood claims.
//!
//! One substance, laid in **patches** -- discs on the floor, each with an
//! owner and the frames it has left. Her abilities lay a patch wherever they
//! touch, a stain is ground while it stands and leaves a patch when it fades,
//! and a heart holds ground round itself (`state::World::mark_the_blood`
//! asks all three). Two patches of hers near each other grow a bridge, and a
//! patch that touches another of hers fades slower: scattered close
//! together, the sparks join into a field; far apart, they go out. See
//! `docs/design/exploration/0021_blood_mage_territory.md`.
//!
//! A table of its own in the snapshot rather than effects: there are dozens
//! of them, and a fight's effects should not have to make room.

use crate::fixed::Fx;
use crate::math::V3;
use crate::tuning as t;

/// How many patches the world holds, every fighter's together. Past it the
/// patch with least life left makes way (`lay`). Forty-eight of sixteen bytes
/// is what the snapshot's cap had room for (`tests/budget.rs`).
pub const MAX_PATCHES: usize = 48;
// A heart's hold is worked out in masks of patches (`held_by_hearts`).
const _: () = assert!(MAX_PATCHES <= 64);

/// A patch's radius is kept in this many bits of fraction -- thirty-seconds
/// of a metre, in a byte, so the widest is just under eight metres.
const RADIUS_SHIFT: u32 = 11;

/// One patch: a disc on the floor at `at`, `radius()` wide, `owner`'s, with
/// `life` frames left. A `life` of nought is an empty slot. Sixteen bytes:
/// the radius is a byte (`RADIUS_SHIFT`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Patch {
    pub at: V3,
    radius: u8,
    pub owner: u8,
    pub life: u16,
}

impl Patch {
    /// Is there a patch in this slot?
    pub const fn live(&self) -> bool {
        self.life > 0
    }

    /// How wide it is.
    pub const fn radius(&self) -> Fx {
        Fx::from_raw((self.radius as i32) << RADIUS_SHIFT)
    }

    fn set_radius(&mut self, r: Fx) {
        self.radius = (r.raw() >> RADIUS_SHIFT).clamp(1, u8::MAX as i32) as u8;
    }

    /// Does it cover the point `at` -- inside its disc, flat, and within
    /// `tuning::ground_slack` of its height?
    pub fn covers(&self, at: V3) -> bool {
        self.live() && within(self.at, self.radius(), at)
    }
}

/// The world's ground.
pub type Ground = [Patch; MAX_PATCHES];

/// Which patches a heart holds this frame, a bit a slot (`step` answers it,
/// `World::ground_held` keeps it): what the renderer draws whole however
/// little life is left in it, and what a full table never makes way with.
pub type Held = u64;

/// Is the patch in slot `k` held?
pub const fn is_held(held: Held, k: usize) -> bool {
    held & (1 << k) != 0
}

/// No ground at all.
pub const NONE: Ground = [Patch {
    at: V3::ZERO,
    radius: 0,
    owner: 0,
    life: 0,
}; MAX_PATCHES];

/// A disc of somebody's ground that is not a patch -- a stain standing, a
/// heart -- which a patch counts as joined to.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Disc {
    pub at: V3,
    pub radius: Fx,
    pub owner: u8,
    /// A heart: it **holds** every patch joined to it, however far through
    /// other patches and stains, from running out (`held_by_hearts`).
    pub holds: bool,
}

impl Disc {
    pub fn covers(&self, at: V3) -> bool {
        within(self.at, self.radius, at)
    }
}

/// Is `at` inside the disc of `radius` at `centre`, flat, and near enough its
/// height to be standing on it?
fn within(centre: V3, radius: Fx, at: V3) -> bool {
    let flat = V3::new(at.x.sub(centre.x), Fx::ZERO, at.z.sub(centre.z)).flat_len();
    flat.raw() <= radius.raw() && at.y.sub(centre.y).abs().raw() <= t::ground_slack().raw()
}

/// Flat distance between two points.
fn apart(a: V3, b: V3) -> Fx {
    V3::new(a.x.sub(b.x), Fx::ZERO, a.z.sub(b.z)).flat_len()
}

/// **Lay a patch** of `owner`'s at `at`, `radius` wide, for `life` frames.
///
/// Onto one of hers it mostly overlaps, it **merges**: that patch grows to
/// cover both, up to `tuning::ground_widest`, and keeps the longer life. Past
/// the table's size the patch with least life left makes way -- hers first,
/// so one fighter's ground never pushes out another's while she has any.
pub fn lay(ground: &mut Ground, owner: u8, at: V3, radius: Fx, life: u16) {
    lay_sparing(ground, 0, owner, at, radius, life);
}

/// [`lay`], making way past the table's size only with a patch no heart
/// `held` (2026-10-19): a home base is never pushed out by a spark laid
/// somewhere else. With every patch held, nothing new is laid.
pub fn lay_sparing(ground: &mut Ground, held: Held, owner: u8, at: V3, radius: Fx, life: u16) {
    if life == 0 || radius.raw() <= 0 {
        return;
    }
    let radius = radius.min(t::ground_widest());
    for p in ground.iter_mut() {
        if !p.live() || p.owner != owner || at.y.sub(p.at.y).abs().raw() > t::ground_slack().raw() {
            continue;
        }
        let far = apart(p.at, at);
        // Mostly inside it: its middle within half of the bigger of the two.
        if far.raw() * 2 <= p.radius().max(radius).raw() {
            p.set_radius(p.radius().max(far.add(radius)).min(t::ground_widest()));
            p.life = p.life.max(life);
            return;
        }
    }
    let mut born = Patch {
        at,
        radius: 0,
        owner,
        life,
    };
    born.set_radius(radius);
    if let Some(p) = ground.iter_mut().find(|p| !p.live()) {
        *p = born;
        return;
    }
    // Full: the weakest of hers, or else the weakest of anybody's -- never
    // one a heart holds.
    let weakest = |mine: bool| {
        ground
            .iter()
            .enumerate()
            .filter(|(k, _)| !is_held(held, *k))
            .filter(|(_, p)| !mine || p.owner == owner)
            .min_by_key(|(_, p)| p.life)
            .map(|(k, p)| (k, p.life))
    };
    if let Some((k, left)) = weakest(true).or_else(|| weakest(false))
        && left < life
    {
        ground[k] = born;
    }
}

/// Is `at` on `owner`'s ground -- a patch, or one of `discs` -- by the
/// patches alone and the discs given?
pub fn covers(ground: &Ground, discs: &[Disc], owner: u8, at: V3) -> bool {
    ground.iter().any(|p| p.owner == owner && p.covers(at))
        || discs.iter().any(|d| d.owner == owner && d.covers(at))
}

/// Is `at` within `by` of `owner`'s ground, flat -- the edge of a patch or a
/// disc no further than that? Height is not asked: a thrall's leash is a
/// distance across the floor.
pub fn near(ground: &Ground, discs: &[Disc], owner: u8, at: V3, by: Fx) -> bool {
    ground
        .iter()
        .any(|p| p.live() && p.owner == owner && apart(p.at, at).raw() <= p.radius().add(by).raw())
        || discs
            .iter()
            .any(|d| d.owner == owner && apart(d.at, at).raw() <= d.radius.add(by).raw())
}

/// The middle of the patch or disc of `owner`'s whose edge is nearest `at`,
/// if she has any ground at all.
pub fn nearest(ground: &Ground, discs: &[Disc], owner: u8, at: V3) -> Option<V3> {
    let mut best: Option<(V3, i32)> = None;
    let mut offer = |centre: V3, radius: Fx| {
        let gap = apart(centre, at).sub(radius).raw();
        if best.is_none_or(|(_, b)| gap < b) {
            best = Some((centre, gap));
        }
    };
    for p in ground.iter().filter(|p| p.live() && p.owner == owner) {
        offer(p.at, p.radius());
    }
    for d in discs.iter().filter(|d| d.owner == owner) {
        offer(d.at, d.radius);
    }
    best.map(|(centre, _)| centre)
}

/// **The ground, one frame.** Every patch loses a frame of life -- one in
/// `tuning::ground_joined_every` while it touches another of its owner's, or
/// a disc of theirs (`discs`: her stains and hearts) -- and every
/// `tuning::ground_bridge_every` frames, two patches of one owner's whose
/// edges are within `tuning::ground_bridge` get one laid over the gap.
///
/// **A heart holds the ground joined to it** (2026-10-19): a patch joined to
/// one of its owner's hearts, through any run of patches and stains, still
/// ages, but does not run out -- it stops a frame short. When the heart
/// goes, what was overdue goes on the next frame, and the rest keeps what it
/// had left.
///
/// Gives back which patches are held, for the world to keep.
pub fn step(ground: &mut Ground, discs: &[Disc], frame: u32) -> Held {
    let every = t::ground_joined_every().max(1) as u32;
    let snapshot = *ground;
    let held = held_by_hearts(&snapshot, discs);
    for (k, p) in ground.iter_mut().enumerate() {
        if !p.live() {
            continue;
        }
        let joined = snapshot.iter().enumerate().any(|(j, o)| {
            j != k
                && o.live()
                && o.owner == p.owner
                && apart(o.at, p.at).raw() <= o.radius().add(p.radius()).raw()
        }) || discs.iter().any(|d| {
            d.owner == p.owner && apart(d.at, p.at).raw() <= d.radius.add(p.radius()).raw()
        });
        if (!joined || frame % every == 0) && !(is_held(held, k) && p.life <= 1) {
            p.life -= 1;
        }
    }
    let bridge_every = t::ground_bridge_every().max(1) as u32;
    if frame % bridge_every == 0 {
        bridge(ground, held);
    }
    held
}

/// Do two discs touch, flat? Squared, so no root is taken: it is asked
/// for every pair of patches each frame a heart stands.
fn touching(a: V3, ra: Fx, b: V3, rb: Fx) -> bool {
    let dx = (a.x.raw() - b.x.raw()) as i64;
    let dz = (a.z.raw() - b.z.raw()) as i64;
    let r = (ra.raw() + rb.raw()) as i64;
    dx * dx + dz * dz <= r * r
}

/// Which patches a heart holds: joined to one of its owner's hearts in
/// `discs`, through patches and the other discs (her stains). Who touches
/// whom is worked out once, as a mask of patches for each patch and each
/// disc, and the hold spreads through the masks until nothing more joins.
/// Nothing at all when there is no heart, which is most frames.
fn held_by_hearts(ground: &Ground, discs: &[Disc]) -> Held {
    if !discs.iter().any(|d| d.holds) {
        return 0;
    }
    const MOST_DISCS: usize = crate::effects::MAX_EFFECTS;
    let discs = &discs[..discs.len().min(MOST_DISCS)];
    // Each patch's neighbours, and each disc's, as masks of patches.
    let mut next = [0u64; MAX_PATCHES];
    for (k, p) in ground.iter().enumerate() {
        if !p.live() {
            continue;
        }
        for (o, q) in ground.iter().enumerate().skip(k + 1) {
            if q.live() && q.owner == p.owner && touching(p.at, p.radius(), q.at, q.radius()) {
                next[k] |= 1 << o;
                next[o] |= 1 << k;
            }
        }
    }
    let mut on = [0u64; MOST_DISCS];
    for (j, d) in discs.iter().enumerate() {
        for (k, p) in ground.iter().enumerate() {
            if p.live() && p.owner == d.owner && touching(d.at, d.radius, p.at, p.radius()) {
                on[j] |= 1 << k;
            }
        }
    }
    let mut held = 0u64;
    let mut reached = [false; MOST_DISCS];
    for (j, d) in discs.iter().enumerate() {
        if d.holds {
            reached[j] = true;
            held |= on[j];
        }
    }
    loop {
        let mut grown = held;
        for (k, n) in next.iter().enumerate() {
            if held & (1 << k) != 0 {
                grown |= n;
            }
        }
        for j in 0..discs.len() {
            if !reached[j] && on[j] & grown != 0 {
                reached[j] = true;
                grown |= on[j];
            }
        }
        if grown == held {
            break;
        }
        held = grown;
    }
    held
}

/// Lay a bridge over every gap of `tuning::ground_bridge` or less between
/// two patches of one owner's, as long-lived as the shorter-lived of the two.
fn bridge(ground: &mut Ground, held: Held) {
    let gap_most = t::ground_bridge();
    let snapshot = *ground;
    for (i, a) in snapshot.iter().enumerate() {
        if !a.live() {
            continue;
        }
        for b in snapshot.iter().skip(i + 1) {
            if !b.live() || b.owner != a.owner {
                continue;
            }
            let far = apart(a.at, b.at);
            let gap = far.sub(a.radius()).sub(b.radius());
            if gap.raw() <= 0 || gap.raw() > gap_most.raw() || far.raw() <= 0 {
                continue;
            }
            // The middle of the gap, between the two edges.
            let way = V3::new(b.at.x.sub(a.at.x), Fx::ZERO, b.at.z.sub(a.at.z)).normalized();
            let edge = a.at.add(way.scale(a.radius()));
            let half = Fx::from_raw(gap.raw() / 2);
            let mid = edge.add(way.scale(half));
            let y = Fx::from_raw((a.at.y.raw() >> 1) + (b.at.y.raw() >> 1));
            let at = V3::new(mid.x, y, mid.z);
            // A little past both edges, so the three are one.
            let radius = half.add(t::ground_bridge_overlap());
            lay_sparing(ground, held, a.owner, at, radius, a.life.min(b.life));
        }
    }
}

/// Every patch moved by `d`: a change of frame.
pub fn shift(ground: &mut Ground, d: V3) {
    for p in ground.iter_mut().filter(|p| p.live()) {
        p.at = p.at.add(d);
    }
}
