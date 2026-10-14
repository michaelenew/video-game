//! **The crowd field**: what the world knows of the players too far away to
//! be bodies, and how it is allowed to know it.
//! `docs/design/exploration/0018_crowds.md` §2 and §4.
//!
//! Layer one -- the regions -- carries every input of everyone near you and
//! is exact to the frame. This is layer two. Once an **epoch** (`E` frames)
//! each region's confirmed state is summed into one fixed-size
//! [`Aggregate`]: how many fighters, where their middle is, which way they
//! drift, how hurt they are, which doors are open. Aggregates travel by
//! cells and leaders, any number of hops, however late that is; and the
//! simulation reads them only through a **lookahead** `L`: the aggregate of
//! epoch `e` is fed to frame `e·E + L` and no earlier. If every hop is
//! covered by `L`, the aggregate had arrived before that frame was first
//! simulated, nothing about it is ever predicted, and nothing is ever rolled
//! back for it (0018, S1). The field it feeds is a pure function of the fed
//! aggregates and the frame, so two peers holding the same bytes hold the
//! same field, bit for bit, and it is in the checksum (S2).
//!
//! **The aggregates are fed, not kept.** Like inputs, they live outside the
//! snapshot, on a tape in the session layer (`crates/regions`), keyed by the
//! frame they are due; `World::advance_with` takes the frame's due
//! aggregates beside its inputs, and a rollback re-feeds them the way it
//! re-feeds inputs. What the snapshot keeps is the field: each stream's last
//! accepted aggregate and the slow quantities filtered from it.
//!
//! **Slow by construction** (S3): what the simulation reads is not the
//! aggregate but a first-order lag of it with time constant `τ`. With `τ ≥
//! L`, a value read `L` late is within `L/τ` of the current one, so the
//! world reacts to the far crowd as it would with the crowd in view -- just
//! slowly, the way a tide is slow. `τ ≥ L` is asserted at the Oven.
//!
//! Fixed point, no allocation, a few hundred bytes of snapshot.

use crate::fixed::Fx;
use crate::math::V3;
use crate::region::{Grid, RegionId};
use crate::state::World;
use crate::tuning as t;

/// How many streams a world follows at once: its own regions' neighbours
/// and a super-region or two. A sound grid puts a body in at most three
/// regions with at most twelve distinct neighbours between them.
pub const MAX_STREAMS: usize = 16;

/// One region's confirmed state, summed: twenty-two bytes an epoch.
/// **Associative** -- a super-region's aggregate is the sum of its
/// children's ([`Aggregate::sum`]) -- which is what makes cells of cells
/// nothing but summing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Aggregate {
    pub region: RegionId,
    /// Which epoch of the world's clock: frame `epoch * E`.
    pub epoch: u16,
    /// Fighters standing in the region's hexagon.
    pub fighters: u16,
    /// Their middle, in decimetres from the hexagon's centre.
    pub middle: [i16; 2],
    /// Their mean velocity on the floor, in decimetres a second, clamped.
    pub drift: [i8; 2],
    /// Health missing from those fighters, in hundredths of a full bar
    /// each, summed: how hurt the crowd is.
    pub hurt: u16,
    /// Creatures alive in the region.
    pub creatures: u8,
    /// A bit per creature species beaten, as the valley's journey keeps it,
    /// OR'd across the fighters: the doors a crowd has opened.
    pub doors: u32,
}

impl Aggregate {
    pub const NONE: Aggregate = Aggregate {
        region: RegionId { q: 0, r: 0 },
        epoch: 0,
        fighters: 0,
        middle: [0; 2],
        drift: [0; 2],
        hurt: 0,
        creatures: 0,
        doors: 0,
    };

    /// Two aggregates of the same epoch as one, under `region`: counts add,
    /// middles and drifts add weighted by count, doors OR. The middle is
    /// kept relative to `region`'s centre, so children are shifted by
    /// their offset from it first (`Grid::centre_offset`).
    pub fn sum(region: RegionId, epoch: u16, parts: &[(Aggregate, [i32; 2])]) -> Aggregate {
        let mut out = Aggregate {
            region,
            epoch,
            ..Aggregate::NONE
        };
        let mut n = 0i32;
        let mut mid = [0i32; 2];
        let mut drift = [0i32; 2];
        for (a, offset) in parts {
            let w = a.fighters as i32;
            n += w;
            mid[0] += w * (a.middle[0] as i32 + offset[0]);
            mid[1] += w * (a.middle[1] as i32 + offset[1]);
            drift[0] += w * a.drift[0] as i32;
            drift[1] += w * a.drift[1] as i32;
            out.fighters = out.fighters.saturating_add(a.fighters);
            out.hurt = out.hurt.saturating_add(a.hurt);
            out.creatures = out.creatures.saturating_add(a.creatures);
            out.doors |= a.doors;
        }
        if n > 0 {
            out.middle = [
                (mid[0] / n).clamp(-32_768, 32_767) as i16,
                (mid[1] / n).clamp(-32_768, 32_767) as i16,
            ];
            out.drift = [
                (drift[0] / n).clamp(-128, 127) as i8,
                (drift[1] / n).clamp(-128, 127) as i8,
            ];
        }
        out
    }

    /// The frame an aggregate of this epoch is fed to: `epoch * E + L`.
    pub fn due(&self) -> u32 {
        self.epoch as u32 * t::crowd_epoch() + t::crowd_lookahead()
    }
}

/// One stream the field follows: a region, what it last said, and the slow
/// quantities filtered from it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Stream {
    pub region: RegionId,
    /// Followed at all: a stream the world has been told to listen to.
    pub on: bool,
    /// The last aggregate accepted, and so the epoch heard through.
    pub last: Aggregate,
    /// Fighters, filtered with `τ_crowd`: what "how many are over there"
    /// reads as.
    pub crowd: Fx,
    /// Hurt, filtered with `τ_heat`: how much blood is being spilt there.
    pub heat: Fx,
    /// The drift, filtered with `τ_crowd`, in metres a second.
    pub flow: [Fx; 2],
}

impl Stream {
    pub const NONE: Stream = Stream {
        region: RegionId { q: 0, r: 0 },
        on: false,
        last: Aggregate::NONE,
        crowd: Fx::ZERO,
        heat: Fx::ZERO,
        flow: [Fx::ZERO; 2],
    };

    /// One frame of the filters toward what was last accepted.
    fn step(&mut self) {
        let crowd = Fx::from_int(self.last.fighters as i32);
        let heat = Fx::from_int(self.last.hurt as i32);
        let flow = [
            Fx::ratio(self.last.drift[0] as i32, 10),
            Fx::ratio(self.last.drift[1] as i32, 10),
        ];
        self.crowd = lag(self.crowd, crowd, t::crowd_tau());
        self.heat = lag(self.heat, heat, t::crowd_heat_tau());
        self.flow = [
            lag(self.flow[0], flow[0], t::crowd_tau()),
            lag(self.flow[1], flow[1], t::crowd_tau()),
        ];
    }
}

/// A first-order lag: `v` toward `target` by a share of the gap each frame
/// such that the gap is a third of itself after `tau` frames -- fixed
/// point, exact on every machine.
fn lag(v: Fx, target: Fx, tau: u32) -> Fx {
    let tau = tau.max(1) as i32;
    let gap = target.sub(v);
    v.add(gap.div(Fx::from_int(tau)))
}

/// **The field**: every stream followed, in the snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Field {
    pub streams: [Stream; MAX_STREAMS],
}

impl Field {
    pub const NONE: Field = Field {
        streams: [Stream::NONE; MAX_STREAMS],
    };

    /// Any stream followed at all. A world that follows none hashes as it
    /// did before there was a field.
    pub fn any(&self) -> bool {
        self.streams.iter().any(|s| s.on)
    }

    /// Follow a region: a slot for it, kept until forgotten. Nothing if
    /// every slot is taken.
    pub fn follow(&mut self, region: RegionId) -> Option<usize> {
        if let Some(i) = self.streams.iter().position(|s| s.on && s.region == region) {
            return Some(i);
        }
        let i = self.streams.iter().position(|s| !s.on)?;
        self.streams[i] = Stream {
            region,
            on: true,
            ..Stream::NONE
        };
        Some(i)
    }

    pub fn forget(&mut self, region: RegionId) {
        for s in self.streams.iter_mut() {
            if s.on && s.region == region {
                *s = Stream::NONE;
            }
        }
    }

    /// The stream for a region, if followed.
    pub fn stream(&self, region: RegionId) -> Option<&Stream> {
        self.streams.iter().find(|s| s.on && s.region == region)
    }

    /// **Feed the frame's due aggregates** and step the filters: called
    /// once a frame by `World::advance_with`, with exactly the aggregates
    /// whose `due` is this frame, in region order. An aggregate for a
    /// region not followed is ignored; one no newer than the last accepted
    /// is ignored, so a repeat on the wire is nothing.
    pub fn feed(&mut self, due: &[Aggregate]) {
        for a in due {
            if let Some(s) = self
                .streams
                .iter_mut()
                .find(|s| s.on && s.region == a.region)
                && (a.epoch > s.last.epoch || s.last == Aggregate::NONE)
            {
                s.last = *a;
            }
        }
        for s in self.streams.iter_mut().filter(|s| s.on) {
            s.step();
        }
    }

    /// **How many fighters the field says are in the regions next to
    /// `region`**, summed and filtered: what a creature hears of the valley
    /// filling next door. Never its own region, which it can see.
    pub fn crowd_around(&self, region: RegionId) -> Fx {
        let mut sum = Fx::ZERO;
        for n in region.neighbours() {
            if let Some(s) = self.stream(n) {
                sum = sum.add(s.crowd);
            }
        }
        sum
    }
}

impl Default for Field {
    fn default() -> Field {
        Field::NONE
    }
}

impl World {
    /// **This region's aggregate** of the world as it stands, for `epoch`:
    /// what a leader computes from the confirmed world at the epoch's
    /// frame. Every member computes the same bytes.
    pub fn aggregate(&self, grid: &Grid, region: RegionId, epoch: u16) -> Aggregate {
        let origin = self.map_origin();
        let (cx, cz) = grid.centre(region);
        let mut n = 0i32;
        let mut mid = [0i64; 2];
        let mut drift = [0i64; 2];
        let mut hurt = 0u32;
        let present = self.present();
        for (i, p) in self.players.iter().enumerate().take(present) {
            if !self.is_present(i) || p.health <= 0 {
                continue;
            }
            let at = p.pos.add(origin);
            if grid.home(at) != region {
                continue;
            }
            n += 1;
            mid[0] += dm(at.x.sub(cx)) as i64;
            mid[1] += dm(at.z.sub(cz)) as i64;
            drift[0] += dm(p.vel.x) as i64;
            drift[1] += dm(p.vel.z) as i64;
            let full = p.full_health().max(1);
            hurt += ((full - p.health).max(0) * 100 / full) as u32;
        }
        let creatures = self
            .monsters
            .iter()
            .flatten()
            .filter(|m| m.alive() && grid.home(m.pos.add(origin)) == region)
            .count() as u8;
        let mut a = Aggregate {
            region,
            epoch,
            fighters: n.min(u16::MAX as i32) as u16,
            hurt: hurt.min(u16::MAX as u32) as u16,
            creatures,
            doors: if self.valley.on {
                self.valley.beaten
            } else {
                0
            },
            ..Aggregate::NONE
        };
        if n > 0 {
            a.middle = [
                (mid[0] / n as i64).clamp(-32_768, 32_767) as i16,
                (mid[1] / n as i64).clamp(-32_768, 32_767) as i16,
            ];
            a.drift = [
                (drift[0] / n as i64).clamp(-128, 127) as i8,
                (drift[1] / n as i64).clamp(-128, 127) as i8,
            ];
        }
        a
    }
}

/// Metres to decimetres, rounded toward zero.
fn dm(v: Fx) -> i32 {
    (v.raw() as i64 * 10 / 65_536) as i32
}

/// A point on the floor from a region's centre and decimetres off it: where
/// an aggregate's middle is, for anything that wants to look that way.
pub fn middle_of(grid: &Grid, a: &Aggregate) -> V3 {
    let (cx, cz) = grid.centre(a.region);
    V3::new(
        cx.add(Fx::ratio(a.middle[0] as i32, 10)),
        Fx::ZERO,
        cz.add(Fx::ratio(a.middle[1] as i32, 10)),
    )
}
