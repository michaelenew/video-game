//! **The valley's sleeping creatures, drawn** (`sim::valley::sight`): every
//! range's herd where its wander has it this frame, and every room's
//! creature on its marks, so you can see them from across the valley and go
//! to them. None of them is in the world; each is asked of `sim` once a
//! frame, a pure function of the frame number, and drawn with the same
//! parts and coats as the live hunt's (`beast`, `critters`).
//!
//! Only the nearest few are drawn -- [`BODIES`] bodies and [`PACKS`] packs --
//! nearest the first fighter first: a herd past the fog is not seen anyway.

use bevy::prelude::*;
use sim::arena::ArenaId;
use sim::critter::Critters;
use sim::monster::Monster;
use sim::valley::sight::{self, Sight};

/// How many sleeping bodies are drawn at once.
pub const BODIES: usize = 10;
/// How many sleeping packs are drawn at once.
pub const PACKS: usize = 3;

/// This frame's sightings, in the world's coordinates.
#[derive(Resource)]
pub struct Sighted {
    /// A body, and which of its herd it is (the second is drawn darker).
    pub bodies: [Option<(Monster, usize)>; BODIES],
    pub packs: [Option<Critters>; PACKS],
    /// Each place's creatures as its hunt starts, worked out the first time
    /// they are wanted (a hunt is a world to build), by arena id.
    bases: Vec<Option<Option<Sight>>>,
    /// The frame and the place the world was in when this was worked out.
    at: Option<(u32, ArenaId)>,
}

impl Default for Sighted {
    fn default() -> Sighted {
        Sighted {
            bodies: [None; BODIES],
            packs: [None; PACKS],
            bases: vec![None; sim::arena::COUNT],
            at: None,
        }
    }
}

/// Work out what is sighted this frame.
pub fn update(sim: Res<crate::Sim>, mut sighted: ResMut<Sighted>) {
    let world = &sim.cur;
    if !world.valley.on {
        if sighted.at.is_some() {
            sighted.bodies = [None; BODIES];
            sighted.packs = [None; PACKS];
            sighted.at = None;
        }
        return;
    }
    if sighted.at == Some((world.frame, world.arena)) {
        return;
    }
    sighted.at = Some((world.frame, world.arena));
    sighted.bodies = [None; BODIES];
    sighted.packs = [None; PACKS];
    let atlas = sim::atlas::valley();
    let origin = world.map_origin();
    let me = world.players[0].pos.add(origin);
    // The live hunt is the world's; a range beaten not long ago is empty.
    let live = world.hunting().then(|| world.den_id());
    let mut places: Vec<(i64, ArenaId, sim::V3)> = atlas
        .places
        .iter()
        .filter(|p| Some(p.arena) != live && !world.resting(p.arena))
        .filter(|p| {
            matches!(
                sim::valley::place(p.arena).map(|q| q.kind),
                Some(sim::valley::Kind::Room(_))
            )
        })
        .map(|p| {
            let d = p.at.sub(me);
            let flat = sim::math::wide_flat_len(sim::V3::new(d.x, sim::Fx::ZERO, d.z));
            (flat.raw() as i64, p.arena, p.at)
        })
        .collect();
    places.sort_by_key(|p| p.0);
    let ground = world.terrain();
    let (mut b, mut k) = (0, 0);
    for (_, id, at) in places {
        if b >= BODIES && k >= PACKS {
            break;
        }
        let slot = &mut sighted.bases[id.0 as usize];
        let Some(base) = *slot.get_or_insert_with(|| sight::base(id)) else {
            continue;
        };
        let bodies = base.monsters.iter().flatten().count();
        let pack = base.critters.any();
        if (bodies > 0 && b + bodies > BODIES) || (pack && k >= PACKS) {
            continue;
        }
        let mut seen = base;
        sight::stand(&mut seen, world.frame, at.sub(origin), &ground);
        for (n, m) in seen.monsters.iter().flatten().enumerate() {
            sighted.bodies[b] = Some((*m, n));
            b += 1;
        }
        if pack {
            sighted.packs[k] = Some(seen.critters);
            k += 1;
        }
    }
}
