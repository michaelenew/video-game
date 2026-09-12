//! Things a move leaves behind.
//!
//! Until now every attack was an instant: a hitbox that existed for a few
//! frames and was gone. Two classes are built on the opposite idea — a fire
//! pillar that grows where you put it, a drain field that punishes standing
//! still. Those need to outlive the move that made them.
//!
//! **The Elementalist's structures are not in here**, and that is deliberate.
//! They are a cap-of-three resource owned by her mechanic, with no clock. Being
//! in this array gave them a lifetime and a second list to disagree with, and
//! the result was that her special silently stopped working ten seconds after
//! she pressed the button that enables it.
//!
//! **A fixed array, not a `Vec`.** Effects are simulation state, so they are
//! snapshotted and restored on every rollback; a heap allocation per frame of
//! re-simulation would be the most expensive thing in the tick. A hard cap also
//! makes "what happens when you spam it" a decision rather than an emergent
//! property: the oldest goes.

use crate::fixed::Fx;
use crate::math::V3;
use crate::tuning as t;

pub const MAX_EFFECTS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    /// Elementalist. Narrow and short at first, then grows: a wide, punishing
    /// base and a taller column above it.
    FirePillar,
    /// Blood mage. A field that drains and slows anyone standing in it.
    BlackSpike,
}

impl EffectKind {
    pub const fn name(self) -> &'static str {
        match self {
            EffectKind::FirePillar => "fire pillar",
            EffectKind::BlackSpike => "black spike",
        }
    }

    /// Which move index spawns this, for the move tables.
    pub const fn from_code(code: u8) -> Option<EffectKind> {
        match code {
            1 => Some(EffectKind::FirePillar),
            2 => Some(EffectKind::BlackSpike),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    /// Who made it. An effect never hurts its owner.
    pub owner: u8,
    pub pos: V3,
    /// Frames since it appeared. Growth is a function of this, so it is a pure
    /// function of the snapshot and rollback reproduces it exactly.
    pub age: u16,
    pub life: u16,
}

impl Effect {
    /// How far through its life, as a fraction.
    ///
    /// Fixed point, so growth curves are deterministic. Saturates at one rather
    /// than running past, since an effect is removed the frame it expires.
    pub fn progress(&self) -> Fx {
        if self.life == 0 {
            return Fx::ONE;
        }
        let p = Fx::from_int(self.age as i32).div(Fx::from_int(self.life as i32));
        if p.raw() > Fx::ONE.raw() { Fx::ONE } else { p }
    }

    /// The two volumes a fire pillar occupies: a wide base and a taller, only
    /// slightly wider column above it.
    ///
    /// Two rather than one because they are different threats. The base is what
    /// catches someone walking into it; the column is what stops them jumping
    /// over. A single growing cylinder would make those one decision, and the
    /// pillar would be either useless against a jump or unavoidable on the
    /// ground.
    pub fn pillar_volumes(&self) -> (Pillar, Pillar) {
        let grown = self.progress();
        let base = Pillar {
            radius: lerp(
                t::pillar_base_radius_start(),
                t::pillar_base_radius(),
                grown,
            ),
            bottom: Fx::ZERO,
            top: t::pillar_base_height(),
        };
        // The column takes the top four fifths, and widens far less than the
        // base does -- it grows *up* rather than *out*.
        let column = Pillar {
            radius: lerp(t::pillar_radius_start(), t::pillar_column_radius(), grown),
            bottom: t::pillar_base_height(),
            top: lerp(t::pillar_height_start(), t::pillar_height(), grown),
        };
        (base, column)
    }

    /// Radius of a field effect. Drain fields do not grow; they are a place.
    pub fn field_radius(&self) -> Fx {
        match self.kind {
            EffectKind::BlackSpike => t::spike_radius(),
            EffectKind::FirePillar => self.pillar_volumes().0.radius,
        }
    }

    /// Whether this effect damages on a given frame.
    ///
    /// A field that hit every frame would do sixty times its listed damage a
    /// second, so damage-over-time ticks on a cadence and the number in the
    /// move table is per tick.
    pub fn ticks_now(&self) -> bool {
        let every = t::effect_tick_frames().max(1);
        self.age > 0 && self.age % every == 0
    }
}

/// One cylindrical slab of a pillar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pillar {
    pub radius: Fx,
    pub bottom: Fx,
    pub top: Fx,
}

impl Pillar {
    /// Whether a fighter standing at `pos` with the given body is inside.
    ///
    /// Flat distance against the radius, and a vertical overlap test against
    /// the slab — which is the one place in the game where height genuinely
    /// decides a hit, and the reason a pillar can be jumped over while a sword
    /// cannot be ducked.
    pub fn contains(&self, centre: V3, pos: V3, body_radius: Fx, body_height: Fx) -> bool {
        let flat = V3::new(pos.x.sub(centre.x), Fx::ZERO, pos.z.sub(centre.z)).flat_len();
        if flat.raw() > self.radius.add(body_radius).raw() {
            return false;
        }
        let feet = pos.y;
        let head = pos.y.add(body_height);
        head.raw() >= centre.y.add(self.bottom).raw() && feet.raw() <= centre.y.add(self.top).raw()
    }
}

fn lerp(from: Fx, to: Fx, at: Fx) -> Fx {
    from.add(to.sub(from).mul(at))
}

/// The nearest fire pillar a shot from `from` toward `to` is aimed through,
/// and how far along the shot it sits.
///
/// Tested against the base -- the wider of a pillar's two volumes, and the
/// one that decides whether you can walk into it at all. See
/// `docs/design/kits/elementalist.md`.
pub fn first_fire_pillar_along(
    effects: &[Option<Effect>; MAX_EFFECTS],
    from: V3,
    to: V3,
) -> Option<Fx> {
    let mut best: Option<Fx> = None;
    for slot in effects.iter() {
        let Some(e) = slot else { continue };
        if e.kind != EffectKind::FirePillar {
            continue;
        }
        let Some(dist) = crate::math::ray_hits_flat(from, to, e.pos, e.field_radius()) else {
            continue;
        };
        if best.map_or(true, |d| dist.raw() < d.raw()) {
            best = Some(dist);
        }
    }
    best
}
