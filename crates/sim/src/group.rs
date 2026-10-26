//! **Groups** (exploration 0029 §13, 2026-10-26): the players near enough to
//! touch each other share a world, and the world a group plays is the one a
//! match has always been -- at most `MAX_PLAYERS` fighters, one hunt, one
//! frame of reference. Two groups far apart share nothing: no inputs, no
//! rollbacks, a fight each. What this module holds is the two moments a
//! group's world changes hands:
//!
//! - **A split** needs nothing here. Each side goes on with a copy of the
//!   world as it stood, and the fighters of the other side are fed
//!   `Travel::LEFT` from then on -- what a seat whose player has gone has
//!   always been (`World::away`): out of the fight, parked, sent nothing.
//! - **A merge** ([`merge`]) is two groups' worlds made one as the groups
//!   come near: one is the base, the other's fighters moved into its free
//!   slots, brought into its coordinates (`World::shift`), with what they
//!   own (effects, bolts, gusts, shards, ground) and what they say of each
//!   other (who holds whom, whose bleed, whose curse) carried over, and
//!   each kept by its id (`World::ids`). A pure function of the two worlds,
//!   so every machine of the merged group makes the same one.
//!
//! **Two hunts do not merge**: a world holds one, and a group in the middle
//! of one is the base when it meets a group that is not. Two groups each in
//! a hunt stay two groups until one of them is over.

use crate::effects::Effect;
use crate::state::{MAX_PLAYERS, Player, World};

/// Why two worlds were not made one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Both are in a hunt, and a world holds one.
    BothHunting,
    /// They are not on one map: two arenas, or one in the valley and one
    /// not, have no coordinates in common.
    NotOneMap,
    /// More fighters between them than a world has slots.
    Full,
}

/// Two worlds made one, and where each one's fighters went: `from_a[j]` is
/// the merged slot of `a`'s slot `j`, for every fighter in it.
#[derive(Clone, Debug)]
pub struct Merged {
    pub world: World,
    pub from_a: [Option<usize>; MAX_PLAYERS],
    pub from_b: [Option<usize>; MAX_PLAYERS],
}

/// The fighters of a world a merge moves: in the fight, and nobody's dummy.
fn fighters(w: &World) -> impl Iterator<Item = usize> + '_ {
    (0..MAX_PLAYERS).filter(|&i| w.is_present(i) && !w.is_dummy(i))
}

/// **Two groups' worlds made one.** The base is `a`, unless `b` is in a hunt
/// and `a` is not; the other's fighters go into the base's free slots, in
/// order, in the base's coordinates, at the base's frame. Refused for two
/// hunts, for two places with no map in common, and for more fighters than
/// slots.
pub fn merge(a: &World, b: &World) -> Result<Merged, Refused> {
    if a.hunting() && b.hunting() {
        return Err(Refused::BothHunting);
    }
    let swapped = b.hunting();
    let (base, other) = if swapped { (b, a) } else { (a, b) };
    // Into the base's coordinates: on the valley's map, the difference of
    // the two frames' origins; in one arena, none.
    let d = match (base.valley.on, other.valley.on) {
        (true, true) => other.map_origin().sub(base.map_origin()),
        (false, false) if base.arena == other.arena => crate::math::V3::ZERO,
        _ => return Err(Refused::NotOneMap),
    };
    let mut o = other.clone();
    o.shift(d);
    let movers: Vec<usize> = fighters(&o).collect();
    let free: Vec<usize> = (0..MAX_PLAYERS)
        .filter(|&k| !base.is_present(k) || base.is_dummy(k))
        .collect();
    if movers.len() > free.len() {
        return Err(Refused::Full);
    }
    let mut map = [None; MAX_PLAYERS];
    for (j, k) in movers.iter().zip(free.iter()) {
        map[*j] = Some(*k);
    }
    let mut w = base.clone();
    for &j in &movers {
        let k = map[j].expect("mapped above");
        w.seat_join(k, o.players[j].class);
        w.players[k] = moved(&o.players[j], &map);
        w.ids[k] = o.ids[j];
        w.course[k] = o.course[j];
        w.valley.cairn[k] = o.valley.cairn[j];
        w.valley.keep.eggs[k] = o.valley.keep.eggs[j];
        w.picked[k] = None;
    }
    // What they own, into the base's pools where there is room. What a pool
    // full drops is spectacle: a merge happens between groups far apart.
    for e in o.effects.iter().flatten() {
        if let Some(k) = owner(e.owner, &map)
            && let Some(slot) = w.effects.iter_mut().find(|s| s.is_none())
        {
            *slot = Some(Effect {
                owner: k,
                // Whom it struck was counted in the other world's slots.
                struck: Default::default(),
                drank: Default::default(),
                ..*e
            });
        }
    }
    for b in o.bolts.iter().flatten() {
        if let Some(k) = owner(b.owner, &map)
            && let Some(slot) = w.bolts.iter_mut().find(|s| s.is_none())
        {
            let mut b = *b;
            b.owner = k;
            *slot = Some(b);
        }
    }
    for g in o.gusts.iter().flatten() {
        if let Some(k) = owner(g.owner, &map)
            && let Some(slot) = w.gusts.iter_mut().find(|s| s.is_none())
        {
            let mut g = *g;
            g.owner = k;
            *slot = Some(g);
        }
    }
    for s in o.debris.iter().flatten() {
        if let Some(k) = owner(s.owner, &map)
            && let Some(slot) = w.debris.iter_mut().find(|x| x.is_none())
        {
            let mut s = *s;
            s.owner = k;
            *slot = Some(s);
        }
    }
    for p in o.ground.iter().filter(|p| p.live()) {
        if let Some(k) = owner(p.owner, &map)
            && let Some(slot) = w.ground.iter_mut().find(|x| !x.live())
        {
            let mut p = *p;
            p.owner = k;
            *slot = p;
        }
    }
    // The journey is both groups': what either has beaten is beaten.
    w.valley.beaten |= o.valley.beaten;
    let mut stay = [None; MAX_PLAYERS];
    for i in fighters(base) {
        stay[i] = Some(i);
    }
    let (from_a, from_b) = if swapped { (map, stay) } else { (stay, map) };
    Ok(Merged {
        world: w,
        from_a,
        from_b,
    })
}

/// A slot of the other world, as the merged world's: `None` for one that
/// did not move, or nobody's (`u8::MAX`, the creature, the world's own).
fn owner(seat: u8, map: &[Option<usize>; MAX_PLAYERS]) -> Option<u8> {
    map.get(seat as usize).copied().flatten().map(|k| k as u8)
}

/// A fighter moved: what it says of other fighters -- who holds it, whose
/// bleed and whose curse it carries -- said of their new slots, or of
/// nobody where they did not come; and no creature to ride.
fn moved(p: &Player, map: &[Option<usize>; MAX_PLAYERS]) -> Player {
    let mut p = *p;
    let seat = |s: u8| owner(s, map).unwrap_or(u8::MAX);
    if (p.held_by as usize) < MAX_PLAYERS {
        p.held_by = seat(p.held_by);
    }
    if (p.bled_by as usize) < MAX_PLAYERS {
        p.bled_by = seat(p.bled_by);
    }
    if p.curse != crate::dual::NO_CURSE {
        p.curse = match owner(crate::dual::cursed_by(p.curse) as u8, map) {
            Some(k) => crate::dual::curse_of(crate::dual::stacks(p.curse), k as usize),
            None => crate::dual::NO_CURSE,
        };
    }
    p.mount = crate::monster::NO_MOUNT;
    p
}
