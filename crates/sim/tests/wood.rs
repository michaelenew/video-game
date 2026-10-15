//! **The Great Wood** (`sim::arena::wood`, exploration 0019 §2b): giant
//! trees south of Hearth, climbed in three bands. The road there is walked,
//! every tree stands where the table says with its crown and its cairn, no
//! branch runs into another tree, and the walking band is jumped -- by every
//! class, with the plain jump -- from the floor to forty metres up.

use sim::arena::wood::{self, GIANTS};
use sim::arena::{ArenaId, Material, Shape, Solid};
use sim::valley;
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

fn open_journey() -> valley::Journey {
    valley::Journey {
        beaten: u32::MAX,
        ..valley::Journey::default()
    }
}

/// Every solid of the map near a tree, on the map.
fn near_tree(g: &wood::Giant, reach: i32) -> Vec<Solid> {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(g.x), cm(g.z));
    let r = cm(reach);
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect()
}

/// A tree's branches: its turned boards whose inner end is in its trunk,
/// lowest first.
fn branches(g: &wood::Giant) -> Vec<Solid> {
    let mut out: Vec<Solid> = near_tree(g, 6000)
        .into_iter()
        .filter(|s| {
            let Shape::Turned { half, cos, sin, .. } = s.shape else {
                return false;
            };
            let (mx, mz) = s.middle();
            // The inner end: back along its length from its middle.
            let ix = mx.sub(cos.mul(half));
            let iz = mz.sub(sin.mul(half));
            let d = sim::math::wide_flat_len(V3::new(ix.sub(cm(g.x)), Fx::ZERO, iz.sub(cm(g.z))));
            d.raw() <= cm(g.radius * 3 / 2).raw()
        })
        .collect();
    out.sort_by_key(|s| s.max.y.raw());
    out
}

#[test]
fn every_tree_stands_with_its_trunk_its_crown_and_its_cairn() {
    for (k, g) in GIANTS.iter().enumerate() {
        let near = near_tree(g, 3000);
        let round: Vec<&Solid> = near
            .iter()
            .filter(|s| {
                let (x, z) = s.middle();
                matches!(s.shape, Shape::Round { .. })
                    && x.sub(cm(g.x)).abs().raw() < Fx::ONE.raw()
                    && z.sub(cm(g.z)).abs().raw() < Fx::ONE.raw()
            })
            .collect();
        let trunk = round
            .iter()
            .filter(|s| s.material == Material::Wood)
            .count();
        let crown = round
            .iter()
            .find(|s| s.material == Material::Grass)
            .unwrap_or_else(|| panic!("tree {k} has no crown"));
        assert_eq!(trunk, 5, "tree {k}: its flare and four lengths");
        let tall = crown.max.y.to_int();
        assert!(
            (140..=210).contains(&tall),
            "tree {k} is {tall} m tall: the Wood's are 150 to 200"
        );
        let wide = 2 * g.radius / 100;
        assert!(wide >= 13, "tree {k} is {wide} m across");
        assert!(
            near.iter()
                .any(|s| s.material == Material::Snow
                    && s.min.y.raw() > crown.max.y.sub(Fx::ONE).raw()),
            "tree {k} has no cairn on its crown"
        );
    }
}

#[test]
fn no_branch_runs_into_another_tree() {
    for (k, g) in GIANTS.iter().enumerate() {
        for b in branches(g) {
            let Shape::Turned { half, cos, sin, .. } = b.shape else {
                unreachable!()
            };
            let (mx, mz) = b.middle();
            let tip = (mx.add(cos.mul(half)), mz.add(sin.mul(half)));
            for (j, o) in GIANTS.iter().enumerate() {
                if j == k {
                    continue;
                }
                let d = sim::math::wide_flat_len(V3::new(
                    tip.0.sub(cm(o.x)),
                    Fx::ZERO,
                    tip.1.sub(cm(o.z)),
                ));
                assert!(
                    d.raw() > cm(o.radius * 14 / 10).raw(),
                    "a branch of tree {k} ends {} m from tree {j}'s axis",
                    d.to_f32_for_render()
                );
            }
        }
    }
}

#[test]
fn the_walking_band_is_a_staircase_the_plain_jump_climbs() {
    for (k, g) in GIANTS.iter().enumerate() {
        let walk: Vec<Solid> = branches(g)
            .into_iter()
            .filter(|s| s.max.y.raw() < Fx::from_int(41).raw())
            .collect();
        assert!(
            walk.len() >= 14,
            "tree {k}: {} walking branches",
            walk.len()
        );
        for pair in walk.windows(2) {
            let rise = pair[1].max.y.sub(pair[0].max.y);
            assert!(
                rise.raw() <= cm(230).raw(),
                "tree {k}: {} m between walking branches",
                rise.to_f32_for_render()
            );
        }
    }
}

/// A point along a turned board's top: `t` thousandths from its inner end
/// to its outer.
fn along(s: &Solid, t: i32) -> V3 {
    let Shape::Turned { half, cos, sin, .. } = s.shape else {
        unreachable!()
    };
    let (mx, mz) = s.middle();
    let off = half.mul(Fx::ratio(2 * t - 1000, 1000));
    V3::new(mx.add(cos.mul(off)), s.max.y, mz.add(sin.mul(off)))
}

/// **One hop from board to board**, jumped: stood on `from` where it comes
/// nearest `to`, a run-up of nothing to two metres back, at `to`'s nearest
/// point, the jump held, braking once over it.
fn hop(w: &mut World, from: &Solid, to: &Solid) -> Result<(), String> {
    let o = w.map_origin();
    let local = |s: &Solid| Solid {
        min: s.min.sub(o),
        max: s.max.sub(o),
        ..*s
    };
    let (from, to) = (local(from), local(to));
    // The nearest pair of points on the two boards' lines, inset from the
    // ends.
    let mut best = (Fx::MAX, V3::ZERO, V3::ZERO);
    for a in (60..=940).step_by(40) {
        for b in (60..=940).step_by(40) {
            let (p, q) = (along(&from, a), along(&to, b));
            let d = sim::math::wide_flat_len(q.sub(p));
            if d.raw() < best.0.raw() {
                best = (d, p, q);
            }
        }
    }
    let (_, p, q) = best;
    let mut last = String::new();
    for run in [0, 60, 120, 200] {
        let flat = V3::new(q.x.sub(p.x), Fx::ZERO, q.z.sub(p.z));
        let d = sim::math::wide_normalized(flat);
        let start = p.sub(d.scale(cm(run)));
        if !from.over(start.x, start.z, Fx::ZERO) {
            continue;
        }
        let pl = &mut w.players[0];
        pl.pos = start;
        pl.vel = V3::ZERO;
        pl.grounded = true;
        pl.facing = d;
        pl.fall_over = pl.pos.y;
        pl.health = 1000;
        let mut jumped = false;
        let mut landed = None;
        for f in 0..240 {
            let pl = &w.players[0];
            let (dx, dz) = (q.x.sub(pl.pos.x), q.z.sub(pl.pos.z));
            let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            let ahead = pl.pos.add(d.scale(Fx::ratio(4, 10)));
            if !jumped && pl.grounded && (!from.over(ahead.x, ahead.z, Fx::ZERO) || f > 40) {
                jumped = true;
            }
            let brake = jumped && to.over(pl.pos.x, pl.pos.z, Fx::ZERO);
            let walk = if brake { Input::S } else { Input::W };
            let bits = walk | if jumped { Input::SPACE } else { 0 };
            w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
            let pl = &w.players[0];
            if jumped && f > 12 && pl.grounded {
                landed = Some(pl.pos);
                break;
            }
        }
        match landed {
            Some(at)
                if to.over(at.x, at.z, Fx::ratio(1, 2))
                    && at.y.sub(to.max.y).abs().raw() < Fx::ratio(1, 10).raw() =>
            {
                return Ok(());
            }
            Some(at) => {
                last = format!(
                    "landed at {at:?}, not on the board at {} m",
                    to.max.y.to_f32_for_render()
                )
            }
            None => last = "never landed".into(),
        }
    }
    Err(last)
}

#[test]
fn every_class_climbs_the_great_trees_walking_band() {
    let g = &GIANTS[0];
    let walk: Vec<Solid> = branches(g)
        .into_iter()
        .filter(|s| s.max.y.raw() < Fx::from_int(41).raw())
        .collect();
    let mut failed = Vec::new();
    for class in sim::class::ALL_CLASSES {
        let mut w = World::arrive(
            [class, Class::Champion],
            open_journey(),
            ArenaId::WOOD,
            Some(0),
            1,
        );
        for (n, pair) in walk.windows(2).enumerate() {
            if let Err(e) = hop(&mut w, &pair[0], &pair[1]) {
                failed.push(format!("{}: branch {n} to {}: {e}", class.name(), n + 1));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **The road to the Wood** is walked: from the town's east gate, over the
/// footbridge, into the clearing to the Great Tree's foot.
#[test]
fn the_road_to_the_wood_can_be_walked() {
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        open_journey(),
        valley::START,
        Some(0),
        1,
    );
    let mut seen = vec![w.arena];
    let mut to: Vec<(Fx, Fx)> = wood::WAY.iter().map(|q| (cm(q.x), cm(q.z))).collect();
    to.push((cm(0), cm(-16000)));
    to.push((cm(0), cm(-30000)));
    for t in to {
        for _ in 0..1500 {
            let feet = w.players[0].pos.add(w.map_origin());
            let (dx, dz) = (t.0.sub(feet.x), t.1.sub(feet.z));
            if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
                break;
            }
            let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            w.advance([Input::looking_at(Input::W, aim, -1500), Input::new(0)]);
            if seen.last() != Some(&w.arena) {
                seen.push(w.arena);
            }
        }
        let feet = w.players[0].pos.add(w.map_origin());
        let off = sim::math::wide_len(V3::new(t.0.sub(feet.x), Fx::ZERO, t.1.sub(feet.z)));
        assert!(
            off.raw() < Fx::from_int(2).raw(),
            "stuck at {feet:?} in {} on the way to ({}, {})",
            w.arena().name,
            t.0.to_f32_for_render(),
            t.1.to_f32_for_render()
        );
    }
    assert_eq!(w.arena, ArenaId::WOOD, "ended in {}", w.arena().name);
    assert!(w.players[0].health > 0);
}
