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
    // Full: the weakest of hers, or else the weakest of anybody's.
    let weakest = |mine: bool| {
        ground
            .iter()
            .enumerate()
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
pub fn step(ground: &mut Ground, discs: &[Disc], frame: u32) {
    let every = t::ground_joined_every().max(1) as u32;
    let snapshot = *ground;
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
        if !joined || frame % every == 0 {
            p.life -= 1;
        }
    }
    let bridge_every = t::ground_bridge_every().max(1) as u32;
    if frame % bridge_every == 0 {
        bridge(ground);
    }
}

/// Lay a bridge over every gap of `tuning::ground_bridge` or less between
/// two patches of one owner's, as long-lived as the shorter-lived of the two.
fn bridge(ground: &mut Ground) {
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
            lay(ground, a.owner, at, radius, a.life.min(b.life));
        }
    }
}

/// Every patch moved by `d`: a change of frame.
pub fn shift(ground: &mut Ground, d: V3) {
    for p in ground.iter_mut().filter(|p| p.live()) {
        p.at = p.at.add(d);
    }
}
