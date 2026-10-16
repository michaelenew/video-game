//! **The Great Wood** (`sim::arena::wood`, exploration 0019 §2b) and **the
//! trees grown in it** (`sim::valley::trees`). A tree is grown where it
//! stands: the same site grows the same tree; its limbs are as thick as what
//! they carry (the pipe model); its roots lie along the land, further
//! downhill than up; its crown leans off a neighbour's; the three kinds grow
//! three shapes; no tree grows into another or into the ground; and the
//! Great Tree's first forty metres are a stair of limbs every class climbs
//! with the plain jump. The road there is walked.

use sim::arena::wood::{self, GIANTS};
use sim::arena::{ArenaId, Shape, Solid};
use sim::valley;
use sim::valley::trees::{self, Kind, NONE, Neighbour, Part, Site, Tree};
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

/// A hillside: falling a metre in five toward +x.
fn slope(x: i32, _z: i32) -> i32 {
    -x / 5
}

fn flat(_x: i32, _z: i32) -> i32 {
    0
}

/// A tree of `kind` grown on `ground` at the origin.
fn grown(kind: Kind, ground: &dyn Fn(i32, i32) -> i32, neighbours: &[Neighbour]) -> Tree {
    let (height, girth) = match kind {
        Kind::Sentinel => (17000, 800),
        Kind::Elder => (14000, 700),
        Kind::Strangler => (10500, 800),
    };
    trees::grow(
        kind,
        &Site {
            at: (0, 0),
            ground,
            neighbours,
            height,
            girth,
            seed: 41,
        },
    )
}

const KINDS: [Kind; 3] = [Kind::Sentinel, Kind::Elder, Kind::Strangler];

#[test]
fn the_same_site_grows_the_same_tree() {
    for kind in KINDS {
        let (a, b) = (grown(kind, &slope, &[]), grown(kind, &slope, &[]));
        assert_eq!(a.nodes.len(), b.nodes.len(), "{kind:?}");
        for (p, q) in a.nodes.iter().zip(b.nodes.iter()) {
            assert_eq!((p.at, p.parent, p.radius), (q.at, q.parent, q.radius));
        }
        assert_eq!(a.leaves, b.leaves);
    }
}

/// **The pipe model**: a node's cross-section is the sum of the
/// cross-sections it carries on, give or take what rounding to the
/// centimetre costs.
#[test]
fn a_limb_is_as_thick_as_what_it_carries() {
    for kind in KINDS {
        let t = grown(kind, &flat, &[]);
        let mut carried = vec![0i64; t.nodes.len()];
        let mut forks = vec![0u32; t.nodes.len()];
        for n in &t.nodes {
            if n.parent != NONE && matches!(n.part, Part::Trunk | Part::Limb | Part::Twig) {
                carried[n.parent as usize] += (n.radius as i64).pow(2);
                forks[n.parent as usize] += 1;
            }
        }
        for (i, n) in t.nodes.iter().enumerate() {
            if forks[i] == 0 || n.radius < 40 {
                continue;
            }
            let own = (n.radius as i64).pow(2);
            let off = (own - carried[i]).abs();
            assert!(
                off * 100 <= own * 8 + 400,
                "{kind:?} node {i}: {} cm across carries {} cm",
                n.radius,
                carried[i].isqrt()
            );
        }
    }
}

/// **The roots lie on the land**, half buried, beyond the buttresses where
/// they flare up the trunk -- and run further down the hill than up it.
#[test]
fn the_roots_lie_along_the_land_and_run_downhill() {
    for kind in KINDS {
        let t = grown(kind, &slope, &[]);
        let girth = t.nodes[0].radius;
        let (mut down, mut up) = (0i64, 0i64);
        for n in t.nodes.iter().filter(|n| n.part == Part::Root) {
            let (x, z) = (n.at[0], n.at[2]);
            let out = ((x as i64).pow(2) + (z as i64).pow(2)).isqrt() as i32;
            if out > girth * 2 {
                let lie = n.at[1] - slope(x, z);
                assert!(
                    (-n.radius..=10).contains(&lie),
                    "{kind:?}: a root {} cm thick lies {lie} cm off the land, {} m out",
                    n.radius,
                    out / 100
                );
            }
            down = down.max(x as i64);
            up = up.max(-x as i64);
        }
        assert!(
            down > up,
            "{kind:?}: its roots run {} m downhill and {} m up",
            down / 100,
            up / 100
        );
    }
}

/// The middle of a tree's crown, flat, about its foot: halfway between its
/// furthest reaches each way.
fn crown_middle(t: &Tree) -> (i64, i64) {
    let crown = || t.nodes.iter().filter(|n| n.part != Part::Root);
    let mid = |k: usize| {
        let lo = crown().map(|n| n.at[k] - t.base[k]).min().unwrap() as i64;
        let hi = crown().map(|n| n.at[k] - t.base[k]).max().unwrap() as i64;
        (lo + hi) / 2
    };
    (mid(0), mid(2))
}

/// **A crown grows away from a neighbour's**: the same tree, with a great
/// one forty metres to the east, has its crown pushed west.
#[test]
fn a_crown_leans_off_its_neighbour() {
    let next = [Neighbour {
        at: (4000, 0),
        girth: 900,
        crown: 5000,
    }];
    for kind in KINDS {
        let alone = crown_middle(&grown(kind, &flat, &[]));
        let crowded = crown_middle(&grown(kind, &flat, &next));
        assert!(
            crowded.0 + 300 < alone.0,
            "{kind:?}: its crown's middle is {} m east alone and {} m crowded",
            alone.0 / 100,
            crowded.0 / 100
        );
    }
}

/// **Three kinds, three shapes**: a Sentinel tall over a narrow cone, an
/// Elder as wide as it is high, a Strangler far wider than high and propped.
#[test]
fn the_three_kinds_grow_three_shapes() {
    let shape = |t: &Tree| {
        let top = t.nodes.iter().map(|n| n.at[1]).max().unwrap() - t.base[1];
        let wide = t
            .nodes
            .iter()
            .filter(|n| n.part != Part::Root)
            .map(|n| ((n.at[0] as i64).pow(2) + (n.at[2] as i64).pow(2)).isqrt())
            .max()
            .unwrap() as i32;
        let props = t.nodes.iter().filter(|n| n.part == Part::Prop).count();
        (top, wide, props)
    };
    let (s_top, s_wide, s_props) = shape(&grown(Kind::Sentinel, &flat, &[]));
    let (e_top, e_wide, e_props) = shape(&grown(Kind::Elder, &flat, &[]));
    let (g_top, g_wide, g_props) = shape(&grown(Kind::Strangler, &flat, &[]));
    assert!(s_top > s_wide * 3, "a Sentinel {s_top} high, {s_wide} out");
    assert!(
        e_wide * 3 > e_top && e_wide < e_top,
        "an Elder {e_top} high, {e_wide} out"
    );
    assert!(g_wide > g_top, "a Strangler {g_top} high, {g_wide} out");
    assert_eq!((s_props, e_props), (0, 0));
    assert!(g_props > 0, "a Strangler without props");
}

/// **No tree grows into another, or into the ground**: none of the Wood's
/// limbs within a neighbour's trunk, and none below the land.
#[test]
fn no_tree_grows_into_another_or_into_the_land() {
    let atlas = sim::atlas::valley();
    let land = atlas.land.as_ref().expect("land");
    let ground = |x: i32, z: i32| {
        let h = land.height(cm(x), cm(z));
        (h.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32
    };
    assert_eq!(atlas.trees.len(), GIANTS.len());
    for (k, t) in atlas.trees.iter().enumerate() {
        for n in t
            .nodes
            .iter()
            .filter(|n| matches!(n.part, Part::Limb | Part::Twig))
        {
            assert!(
                n.at[1] >= ground(n.at[0], n.at[2]) + 100,
                "tree {k}: a limb at {:?} is in the land",
                n.at
            );
            for (j, o) in GIANTS.iter().enumerate() {
                if j == k || n.at[1] > atlas.trees[j].base[1] + o.height {
                    continue;
                }
                let d = (((n.at[0] - o.x) as i64).pow(2) + ((n.at[2] - o.z) as i64).pow(2)).isqrt()
                    as i32;
                assert!(
                    d > o.radius + n.radius,
                    "tree {k}: a limb at {:?} is in tree {j}'s trunk",
                    n.at
                );
            }
        }
    }
}

#[test]
fn every_tree_has_its_cairns_and_its_wood_is_drawn_not_boxed() {
    let atlas = sim::atlas::valley();
    for (k, t) in atlas.trees.iter().enumerate() {
        assert!(!t.cairns.is_empty(), "tree {k} has no cairn");
    }
    // The tree's own boxes are hidden from the renderer, which draws the
    // tree; nothing else on the map is.
    let hidden = atlas.hidden.iter().filter(|h| **h).count();
    let mut wood_boxes = Vec::new();
    for t in &atlas.trees {
        trees::solids(t, &mut wood_boxes);
    }
    assert_eq!(hidden, wood_boxes.len());
}

/// Every solid of the map near a point, on the map.
fn near(x: i32, z: i32, reach: i32) -> Vec<Solid> {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(x), cm(z));
    let r = cm(reach);
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect()
}

/// The Great Tree's limbs where they leave the trunk: its turned boards
/// whose inner end is at its bark, lowest first -- longer than the short
/// ones a buttress root is stepped in.
fn first_boards(t: &Tree) -> Vec<Solid> {
    let girth = t.nodes[0].radius;
    let mut out: Vec<Solid> = near(t.base[0], t.base[2], 3000)
        .into_iter()
        .filter(|s| {
            let Shape::Turned { half, cos, sin, .. } = s.shape else {
                return false;
            };
            if half.raw() < cm(200).raw() {
                return false;
            }
            let (mx, mz) = s.middle();
            let ix = mx.sub(cos.mul(half));
            let iz = mz.sub(sin.mul(half));
            let d = sim::math::wide_flat_len(V3::new(
                ix.sub(cm(t.base[0])),
                Fx::ZERO,
                iz.sub(cm(t.base[2])),
            ));
            d.raw() <= cm(girth * 3 / 2).raw()
        })
        .collect();
    out.sort_by_key(|s| s.max.y.raw());
    out
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

/// How thick the trunk is at a height over the map (its radius).
fn trunk_at(t: &Tree, y: Fx) -> Fx {
    let y = (y.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
    let n = t
        .nodes
        .iter()
        .filter(|n| n.part == Part::Trunk)
        .min_by_key(|n| (n.at[1] - y).abs())
        .unwrap();
    cm(n.radius)
}

/// The nearest points of two boards' tops, flat, inset from their ends and
/// outside the bark: where somebody standing on one jumps for the other.
fn nearest(t: &Tree, from: &Solid, to: &Solid) -> (Fx, V3, V3) {
    let foot = (cm(t.base[0]), cm(t.base[2]));
    let clear = |m: V3| {
        let d = sim::math::wide_flat_len(V3::new(m.x.sub(foot.0), Fx::ZERO, m.z.sub(foot.1)));
        d.raw() > trunk_at(t, m.y).add(Fx::ratio(1, 2)).raw()
    };
    nearest_clear(from, to, &clear)
}

fn nearest_clear(from: &Solid, to: &Solid, clear: &dyn Fn(V3) -> bool) -> (Fx, V3, V3) {
    let mut best = (Fx::MAX, V3::ZERO, V3::ZERO);
    for a in (60..=940).step_by(40) {
        for b in (60..=940).step_by(40) {
            let (p, q) = (along(from, a), along(to, b));
            if !clear(p) || !clear(q) {
                continue;
            }
            let d = sim::math::wide_flat_len(q.sub(p));
            if d.raw() < best.0.raw() {
                best = (d, p, q);
            }
        }
    }
    best
}

/// **The stair**: from the lowest limb, each step the nearest limb a
/// little higher -- the way somebody climbing picks the next one.
fn stair(t: &Tree, up_to: i32) -> Vec<Solid> {
    let boards = first_boards(t);
    let top = cm(t.base[1] + up_to);
    let mut path = vec![boards[0]];
    loop {
        let last = *path.last().unwrap();
        let next = boards
            .iter()
            .filter(|b| {
                let rise = b.max.y.sub(last.max.y);
                rise.raw() > cm(60).raw() && rise.raw() <= cm(240).raw()
            })
            .min_by_key(|b| nearest(t, &last, b).0.raw());
        match next {
            Some(b) if last.max.y.raw() < top.raw() => path.push(*b),
            _ => break,
        }
    }
    path
}

#[test]
fn the_great_tree_s_first_forty_metres_are_a_stair() {
    let atlas = sim::atlas::valley();
    let t = &atlas.trees[0];
    let path = stair(t, 4000);
    let top = path.last().unwrap().max.y.sub(cm(t.base[1]));
    assert!(
        top.raw() >= Fx::from_int(38).raw(),
        "the stair stops {} m up",
        top.to_f32_for_render()
    );
    for pair in path.windows(2) {
        let gap = nearest(t, &pair[0], &pair[1]).0;
        eprintln!(
            "step to {} m, {} m across",
            pair[1].max.y.sub(cm(t.base[1])).to_f32_for_render(),
            gap.to_f32_for_render()
        );
        assert!(
            gap.raw() <= Fx::from_int(4).raw(),
            "{} m between steps",
            gap.to_f32_for_render()
        );
    }
}

/// **One hop from board to board**, jumped: stood on `from` where it comes
/// nearest `to`, a run-up of nothing to two metres back, at `to`'s nearest
/// point, the jump held, braked or steered onto it -- tried every way
/// before it is given up, as somebody climbing would.
fn hop(w: &mut World, t: &Tree, from: &Solid, to: &Solid) -> Result<(), String> {
    let o = w.map_origin();
    let (_, p, q0) = nearest(t, from, to);
    let (p, q0) = (p.sub(o), q0.sub(o));
    let Shape::Turned { cos, sin, .. } = to.shape else {
        unreachable!()
    };
    let local = |s: &Solid| Solid {
        min: s.min.sub(o),
        max: s.max.sub(o),
        ..*s
    };
    let (from, to) = (local(from), local(to));
    let mut last = String::new();
    // Where it comes nearest, or a metre or two along it either way.
    // A held jump or a tap of it, braked or steered.
    let tries = [(240, false), (240, true), (8, true), (8, false)]
        .into_iter()
        .flat_map(|(hold, steer)| {
            [0, 100, -100, 200]
                .into_iter()
                .flat_map(move |shift| [0, 60, 120, 200].map(move |run| (hold, steer, run, shift)))
        });
    for (hold, steer, run, shift) in tries {
        let q = q0.add(V3::new(cos.mul(cm(shift)), Fx::ZERO, sin.mul(cm(shift))));
        if !to.over(q.x, q.z, Fx::ZERO) {
            continue;
        }
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
        let mut took_off = 0;
        let mut landed = None;
        for f in 0..240 {
            let pl = &w.players[0];
            let (dx, dz) = (q.x.sub(pl.pos.x), q.z.sub(pl.pos.z));
            let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            // Far enough ahead that a jump with a startup leaves from the
            // limb, not off it.
            let ahead = pl.pos.add(d.scale(Fx::ratio(8, 10)));
            if !jumped && pl.grounded && (!from.over(ahead.x, ahead.z, Fx::ZERO) || f > 40) {
                jumped = true;
                took_off = f;
            }
            // Braked once over it; or steered onto the point and let go
            // there -- a long slow jump wants the second.
            let over = jumped && to.over(pl.pos.x, pl.pos.z, Fx::ZERO);
            let there =
                sim::math::wide_flat_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 10).raw();
            let walk = match (steer, over, jumped && there) {
                (false, true, _) => Input::S,
                (true, _, true) => 0,
                _ => Input::W,
            };
            let bits = walk
                | if jumped && f - took_off < hold {
                    Input::SPACE
                } else {
                    0
                };
            w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
            let pl = &w.players[0];
            if jumped && f > 12 && pl.grounded {
                landed = Some(pl.pos);
                break;
            }
        }
        match landed {
            // On it -- or on one higher still, which is further up the stair.
            Some(at) if at.y.raw() > to.max.y.sub(Fx::ratio(1, 10)).raw() => {
                return Ok(());
            }
            Some(at) => {
                last = format!(
                    "landed at {at:?}, not on the board at {} m",
                    to.max.y.to_f32_for_render()
                )
            }
            // Caught the vine up the trunk, which goes on past it: the slow
            // way up, and a way up.
            None if valley::vine_on(&w.terrain(), w.players[0].pos) => return Ok(()),
            None => last = format!("never landed: at {:?}", w.players[0].pos),
        }
    }
    Err(last)
}

#[test]
fn every_class_climbs_the_great_tree_s_stair() {
    let atlas = sim::atlas::valley();
    let path = stair(&atlas.trees[0], 4000);
    let mut failed = Vec::new();
    for class in sim::class::ALL_CLASSES {
        for (n, pair) in path.windows(2).enumerate() {
            // A fresh world each hop: a stone the Elementalist's jump brought
            // up last time is not on this limb.
            let mut w = World::arrive(
                [class, Class::Champion],
                open_journey(),
                ArenaId::WOOD,
                Some(0),
                1,
            );
            if let Err(e) = hop(&mut w, &atlas.trees[0], &pair[0], &pair[1]) {
                failed.push(format!("{}: limb {n} to {}: {e}", class.name(), n + 1));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **The road to the Wood** is walked: from the town's east gate, over the
/// footbridge, into the clearing.
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
    to.push((cm(0), cm(-26000)));
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
