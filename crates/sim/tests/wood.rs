//! **The Great Wood** (`sim::arena::wood`, exploration 0019 §2b) and **the
//! trees grown in it** (`sim::valley::trees`). A tree is grown where it
//! stands: the same site grows the same tree; its limbs are as thick as what
//! they carry (the pipe model); its roots lie along the land, further
//! downhill than up; its crown leans off a neighbour's; the three kinds grow
//! three shapes; no tree grows into another or into the ground; and the
//! Great Tree's first forty metres are a stair of limbs every class climbs
//! with the plain jump. The road there is walked.

mod pilot;

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
        bole: 3000,
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
        // Nor under a shelf, where there is no room to stand.
        let roofed = t.pads.iter().any(|p| {
            let bottom = cm(p.at[1] - p.thick);
            m.y.raw() < bottom.raw()
                && m.y.raw() > bottom.sub(Fx::ratio(35, 10)).raw()
                && sim::math::wide_flat_len(V3::new(
                    m.x.sub(cm(p.at[0])),
                    Fx::ZERO,
                    m.z.sub(cm(p.at[2])),
                ))
                .raw()
                    < cm(p.outer + 50).raw()
        });
        d.raw() > trunk_at(t, m.y).add(Fx::ratio(1, 2)).raw() && !roofed
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
    let top = cm(up_to);
    let boards: Vec<Solid> = first_boards(t)
        .into_iter()
        .filter(|b| b.max.y.raw() < top.sub(Fx::ratio(3, 10)).raw())
        .collect();
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
            Some(b) => path.push(*b),
            _ => break,
        }
    }
    path
}

/// A pad of a tree, as the solid it is on the map.
fn pad_solid(p: &trees::Pad) -> Solid {
    near(p.at[0], p.at[2], 100)
        .into_iter()
        .find(|s| {
            matches!(s.shape, Shape::Round { .. })
                && s.max.y.raw() == cm(p.at[1]).raw()
                && s.middle().0.sub(cm(p.at[0])).abs().raw() < Fx::ratio(1, 10).raw()
                && s.middle().1.sub(cm(p.at[2])).abs().raw() < Fx::ratio(1, 10).raw()
        })
        .expect("the pad is on the map")
}

/// The Great Tree's first shelf: the lobe of it nearest the top of the
/// stair, and the step on to it -- from where the limb comes out from under
/// it, up and in over its edge.
fn onto_the_shelf(t: &Tree, last: &Solid) -> (trees::Pad, V3, V3) {
    let first = t.pads[0].at[1];
    let flat = |a: V3, p: &trees::Pad| {
        sim::math::wide_flat_len(V3::new(
            a.x.sub(cm(p.at[0])),
            Fx::ZERO,
            a.z.sub(cm(p.at[2])),
        ))
    };
    let lobes: Vec<trees::Pad> = t
        .pads
        .iter()
        .filter(|p| (p.at[1] - first).abs() < 300)
        .copied()
        .collect();
    let (lobe, from) = lobes
        .iter()
        .flat_map(|p| (60..=940).step_by(10).map(move |k| (*p, along(last, k))))
        .filter(|(p, a)| flat(*a, p).raw() > cm(p.outer + 60).raw())
        .min_by_key(|(p, a)| flat(*a, p).raw())
        .expect("the stair comes by a lobe");
    let inward = sim::math::wide_normalized(V3::new(
        cm(lobe.at[0]).sub(from.x),
        Fx::ZERO,
        cm(lobe.at[2]).sub(from.z),
    ));
    let gap = flat(from, &lobe).sub(cm(lobe.outer));
    (
        lobe,
        from,
        from.add(inward.scale(gap.add(Fx::ratio(15, 10)))),
    )
}

#[test]
fn the_great_tree_s_first_forty_metres_are_a_stair_to_its_shelf() {
    let atlas = sim::atlas::valley();
    let t = &atlas.trees[0];
    let shelf = t.pads[0];
    let path = stair(t, shelf.at[1]);
    let last = *path.last().unwrap();
    let (lobe, p, _) = onto_the_shelf(t, &last);
    let below = cm(lobe.at[1]).sub(last.max.y);
    let across = sim::math::wide_flat_len(V3::new(
        p.x.sub(cm(lobe.at[0])),
        Fx::ZERO,
        p.z.sub(cm(lobe.at[2])),
    ))
    .sub(cm(lobe.outer));
    assert!(
        across.raw() <= Fx::from_int(3).raw(),
        "the stair's top is {} m from the shelf",
        across.to_f32_for_render()
    );
    assert!(
        below.raw() <= cm(230).raw(),
        "the stair stops {} m under the shelf",
        below.to_f32_for_render()
    );
    for pair in path.windows(2) {
        let (gap, p, q) = nearest(t, &pair[0], &pair[1]);
        assert!(
            gap.raw() <= Fx::ratio(9, 2).raw(),
            "{} m between steps, from {p:?} to {q:?}",
            gap.to_f32_for_render()
        );
    }
}

/// **One hop from board to board**, jumped: stood on `from` where it comes
/// nearest `to`, a run-up of nothing to two metres back, at `to`'s nearest
/// point, the jump held, braked or steered onto it -- tried every way
/// before it is given up, as somebody climbing would.
fn hop(w: &mut World, t: &Tree, from: &Solid, to: &Solid) -> Result<(), String> {
    let (_, p, q) = nearest(t, from, to);
    let Shape::Turned { cos, sin, .. } = to.shape else {
        unreachable!()
    };
    hop_at(w, from, to, (p, q), (cos, sin))
}

/// [`hop`] from `p` on `from` to `q` on `to` (map points), trying landings
/// a metre or two along `slide` either way.
fn hop_at(
    w: &mut World,
    from: &Solid,
    to: &Solid,
    (p, q0): (V3, V3),
    (cos, sin): (Fx, Fx),
) -> Result<(), String> {
    let o = w.map_origin();
    let (p, q0) = (p.sub(o), q0.sub(o));
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
        pl.ledge = sim::state::Ledge::NONE;
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
            // A hop that falls short and catches the limb's edge climbs it:
            // the stick into the face, and up.
            let edge = pl.ledge.edge.at;
            let on_to = to.max.y.sub(edge.y).abs().raw() <= Fx::ratio(1, 10).raw();
            let (bits, aim) = if pl.ledge.is_some() && on_to {
                let n = pl.ledge.edge.normal;
                (
                    Input::W,
                    sim::math::atan2_turns(n.z.neg(), n.x.neg()).raw() as u16,
                )
            } else {
                (bits, aim)
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
            None => last = format!("never landed: at {:?}", w.players[0].pos),
        }
    }
    Err(last)
}

#[test]
fn every_class_climbs_the_great_tree_s_stair() {
    let atlas = sim::atlas::valley();
    let path = stair(&atlas.trees[0], atlas.trees[0].pads[0].at[1]);
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
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
            let t = &atlas.trees[0];
            let done = hop(&mut w, t, &pair[0], &pair[1]);
            if let Err(e) = done {
                failed.push(format!("{}: limb {n} to {}: {e}", class.name(), n + 1));
            }
        }
        // And off the top of the stair on to the shelf.
        let mut w = World::arrive(
            [class, Class::Champion],
            open_journey(),
            ArenaId::WOOD,
            Some(0),
            1,
        );
        let t = &atlas.trees[0];
        let last = *path.last().unwrap();
        let (lobe, p, q) = onto_the_shelf(t, &last);
        let slide = sim::math::wide_normalized(V3::new(q.z.sub(p.z), Fx::ZERO, p.x.sub(q.x)));
        if let Err(e) = hop_at(&mut w, &last, &pad_solid(&lobe), (p, q), (slide.x, slide.z)) {
            failed.push(format!("{}: on to the shelf: {e}", class.name()));
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

/// **Every broadleaf's crown is a roof you can walk**, with a lip round it
/// higher than the terrace inside, every terrace a step from the next; and
/// **every roof is reached** by a hanging root off its lip (`wood::ladders`,
/// 2026-10-16, where a liana hung): every class climbs its shelves from the
/// ground, landed or caught at the hands, and steps in over the lip.
/// A broadleaf's roof: its rim's pad, and the box of its top a body lands
/// on, on the side toward `toward` -- the lip's own top, halfway across it.
fn lip_toward(rim: &trees::Pad, toward: (Fx, Fx)) -> Solid {
    let (rx, rz) = (cm(rim.at[0]), cm(rim.at[2]));
    let into = sim::math::wide_normalized(V3::new(toward.0.sub(rx), Fx::ZERO, toward.1.sub(rz)));
    let at = V3::new(rx, Fx::ZERO, rz).add(into.scale(cm((rim.inner + rim.outer) / 2)));
    let h = Fx::ratio(3, 2);
    Solid {
        min: V3::new(at.x.sub(h), cm(rim.at[1] - 100), at.z.sub(h)),
        max: V3::new(at.x.add(h), cm(rim.at[1]), at.z.add(h)),
        material: sim::arena::Material::Grass,
        shape: Shape::Box,
    }
}

/// **Every broadleaf has a roof, and a way up to it that is not all climb**
/// (since 2026-10-22): a hanging root off its lip with bracket fungus up it
/// a catch apart, to a landing [`wood::LANDING_UP`] over the ground with a
/// cairn -- and from the landing an Elder's updraft or a Strangler's spring.
/// Every mover climbs the fungus to the landing and gets on to the roof the
/// way it is meant to: carried up the updraft and drifting in over the lip,
/// or jumping off the spring while it is still going up.
#[test]
fn every_broadleaf_has_a_roof_and_a_quick_way_up_to_it() {
    let atlas = sim::atlas::valley();
    let mut failed = Vec::new();
    for (k, t) in atlas.trees.iter().enumerate() {
        if t.kind == Kind::Sentinel {
            continue;
        }
        let roof: Vec<&trees::Pad> = t
            .pads
            .iter()
            .filter(|p| p.kind != trees::PadKind::Shelf)
            .collect();
        assert!(roof.len() >= 3, "tree {k} has a roof of {}", roof.len());
        let rim = *roof.last().unwrap();
        assert_eq!(rim.kind, trees::PadKind::Rim, "tree {k}");
        let inside = roof[roof.len() - 2];
        assert!(rim.at[1] > inside.at[1] + 80, "tree {k}: its lip is no lip");
        let shelves: Vec<&trees::Pad> = t
            .pads
            .iter()
            .filter(|p| p.kind == trees::PadKind::Shelf)
            .collect();
        assert!(shelves.len() > 3, "tree {k} has no way up");
        let landing = **shelves.last().unwrap();
        assert_eq!(landing.outer, wood::LANDING_R, "tree {k}: no landing");
        let ground = atlas.relief_at(cm(landing.at[0]), cm(landing.at[2]));
        assert!(
            (cm(landing.at[1]).sub(ground).to_int() - wood::LANDING_UP / 100).abs() <= 2,
            "tree {k}: its landing is {} m up",
            cm(landing.at[1]).sub(ground).to_int()
        );
        // From the ground beside the root's foot, up the shelves.
        let first = pad_solid(shelves[0]);
        let (fx, fz) = first.middle();
        let toward = V3::new(cm(rim.at[0]).sub(fx), Fx::ZERO, cm(rim.at[2]).sub(fz));
        let out = sim::math::wide_normalized(toward).scale(Fx::from_int(-4));
        let (gx, gz) = (fx.add(out.x), fz.add(out.z));
        let ground = atlas.relief_at(gx, gz);
        let mut route = vec![Solid::cm([0, 0, 0], [0, 0, 0], sim::arena::Material::Grass)];
        route[0].min = V3::new(gx.sub(Fx::ONE), ground.sub(Fx::ONE), gz.sub(Fx::ONE));
        route[0].max = V3::new(gx.add(Fx::ONE), ground, gz.add(Fx::ONE));
        route.extend(shelves.iter().map(|p| pad_solid(p)));
        let landing_box = *route.last().unwrap();
        let lip = lip_toward(rim, (cm(landing.at[0]), cm(landing.at[2])));
        // What takes you on: the updraft over the landing, or the spring
        // beside it.
        let draft = atlas.vents.iter().find(|v| {
            v.at.0.sub(cm(landing.at[0])).abs().raw() < Fx::ONE.raw()
                && v.at.1.sub(cm(landing.at[2])).abs().raw() < Fx::ONE.raw()
        });
        let spring = atlas.drifts.iter().find(|d| {
            d.solid.material == sim::arena::Material::Sand
                && d.solid.max.y.raw() == cm(landing.at[1]).raw()
                && d.solid.middle().0.sub(cm(landing.at[0])).abs().raw() < Fx::from_int(6).raw()
                && d.solid.middle().1.sub(cm(landing.at[2])).abs().raw() < Fx::from_int(6).raw()
        });
        match t.kind {
            Kind::Elder => assert!(draft.is_some(), "Elder {k} has no updraft"),
            _ => assert!(spring.is_some(), "Strangler {k} has no spring"),
        }
        for class in sim::class::MOVERS {
            let mut w = World::arrive([class, class], open_journey(), ArenaId::WOOD, Some(0), 1);
            let mut ok = true;
            for (n, pair) in route.windows(2).enumerate() {
                let (a, b) = (pilot::local(&w, pair[0]), pilot::local(&w, pair[1]));
                if let Err(e) = pilot::jump(&mut w, a, b) {
                    failed.push(format!("tree {k}, {}: shelf {}: {e}", class.name(), n + 1));
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            let (from, to) = (pilot::local(&w, landing_box), pilot::local(&w, lip));
            let done = match (draft, spring) {
                (Some(_), _) => pilot::updraft(&mut w, from, to),
                (None, Some(d)) => pilot::spring(&mut w, d, to).map(|_| ()),
                _ => Err("nothing to take it on up".into()),
            };
            if let Err(e) = done {
                failed.push(format!("tree {k}, {}: on to the roof: {e}", class.name()));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **The quick ways up are the ways up**: from a landing, no plain jump --
/// run, jump, airdodge, on paper -- catches the lip; nor from a spring
/// stopped at its top, so the spring is jumped off while it rises.
#[test]
fn a_broadleaf_s_lip_is_out_of_reach_but_for_the_quick_way() {
    use sim::envelope::{self as e, Extra};
    let atlas = sim::atlas::valley();
    for class in sim::class::MOVERS {
        let mut paths = vec![e::off_the_edge(class, Extra::Nothing)];
        paths.extend(e::airdodge_paths(class));
        let apex = paths.iter().map(e::path_apex).fold(Fx::ZERO, Fx::max);
        let reach = apex.add(sim::tuning::ledge_reach_high());
        for t in atlas.trees.iter().filter(|t| t.kind != Kind::Sentinel) {
            let rim = t
                .pads
                .iter()
                .find(|p| p.kind == trees::PadKind::Rim)
                .unwrap();
            let landing = t
                .pads
                .iter()
                .rev()
                .find(|p| p.kind == trees::PadKind::Shelf)
                .unwrap();
            let from_spring = cm(rim.at[1] - wood::SPRING_SHORT);
            let from = if t.kind == Kind::Elder {
                cm(landing.at[1])
            } else {
                from_spring
            };
            assert!(
                reach.raw() < cm(rim.at[1]).sub(from).raw(),
                "{}: catches a lip {} m up",
                class.name(),
                cm(rim.at[1]).sub(from).to_f32_for_render()
            );
        }
    }
}

/// **A hoist up the Great Tree and up most Sentinels**: two cars from a
/// fungus landing on the first shelf to one level with the highest, and
/// every mover steps on from the shelf's landing and rides to the top.
#[test]
fn the_sentinels_hoists_carry_every_mover_to_the_top_shelf() {
    let atlas = sim::atlas::valley();
    let mut hoisted = 0;
    let mut failed = Vec::new();
    for (k, t) in atlas.trees.iter().enumerate() {
        if t.kind != Kind::Sentinel {
            continue;
        }
        let pads: Vec<&trees::Pad> = t
            .pads
            .iter()
            .filter(|p| p.outer == wood::HOIST_PAD)
            .collect();
        if pads.is_empty() {
            continue;
        }
        let (foot, head) = (pads[0], pads[1]);
        let car = atlas
            .drifts
            .iter()
            .find(|d| {
                d.solid.material == sim::arena::Material::Wood
                    && d.solid.max.y.raw() == cm(foot.at[1]).raw()
                    && (d.solid.middle().0.sub(cm(foot.at[0])).abs().raw() < Fx::from_int(8).raw())
                    && (d.solid.middle().1.sub(cm(foot.at[2])).abs().raw() < Fx::from_int(8).raw())
            })
            .expect("a car at the foot's landing");
        assert_eq!(
            car.travel.y.raw(),
            cm(head.at[1] - foot.at[1]).raw(),
            "tree {k}: its cars go to its head"
        );
        hoisted += 1;
        let top = pad_solid(head);
        for class in sim::class::MOVERS {
            let mut w = World::arrive([class, class], open_journey(), ArenaId::WOOD, Some(0), 1);
            let to = pilot::local(&w, top);
            if let Err(e) = pilot::ride(&mut w, car, to) {
                failed.push(format!("tree {k}, {}: {e}", class.name()));
            }
        }
    }
    assert!(hoisted >= 3, "{hoisted} Sentinels have hoists");
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **Nothing stands in a hoist's or a spring's way**: no tree's wood is in
/// what a car or a spring sweeps, a little in from its sides, but the
/// landings it stops at.
#[test]
fn nothing_stands_in_what_moves_in_the_wood() {
    let atlas = sim::atlas::valley();
    let mut wood_boxes = Vec::new();
    for t in &atlas.trees {
        trees::solids(t, &mut wood_boxes);
    }
    let skin = Fx::ratio(1, 10);
    for d in atlas.drifts.iter().filter(|d| {
        let (x, z) = d.solid.middle();
        atlas
            .place_at(x, z)
            .is_some_and(|p| p.arena == ArenaId::WOOD)
    }) {
        let mut room = d.swept();
        room.min = room.min.add(V3::new(skin, skin, skin));
        room.max = room.max.sub(V3::new(skin, skin, skin));
        room.max.y = room.max.y.add(Fx::from_int(2));
        for s in &wood_boxes {
            let meets = s.min.x.raw() < room.max.x.raw()
                && s.max.x.raw() > room.min.x.raw()
                && s.min.y.raw() < room.max.y.raw()
                && s.max.y.raw() > room.min.y.raw()
                && s.min.z.raw() < room.max.z.raw()
                && s.max.z.raw() > room.min.z.raw();
            let stop = s.max.y == d.solid.max.y || s.max.y == d.solid.max.y.add(d.travel.y);
            assert!(
                !meets
                    || stop
                    || s.side_push(
                        room.middle().0,
                        room.middle().1,
                        Fx::ratio(13, 10),
                        room.min.y,
                        room.max.y
                    )
                    .is_none(),
                "something of a tree from {} to {} m up is in a drift's way at ({}, {})",
                s.min.y.to_f32_for_render(),
                s.max.y.to_f32_for_render(),
                room.middle().0.to_f32_for_render(),
                room.middle().1.to_f32_for_render()
            );
        }
    }
}

/// **A limb is walked along, not stopped on** (2026-10-16, from play: "the
/// player just gets stuck on a small lip ... where a branch forks"). A limb
/// is drawn smooth and stood on as a stair of boards, so a riser between two
/// of them is a lip nobody can see. Walked along every limb of the Great
/// Tree, both ways, a body may be stopped by the trunk, by a buttress, by
/// something taller than a knee, or by a limb so close overhead that there
/// is no room to stand up on the next board -- never by a board of the limb
/// itself that comes up no higher than a knee.
#[test]
fn a_limb_is_walked_along_without_stopping_on_a_lip() {
    let atlas = sim::atlas::valley();
    let tree = &atlas.trees[0];
    let mut boards = Vec::new();
    trees::solids(tree, &mut boards);
    let base = World::arrive(
        [Class::Champion, Class::Bulwark],
        open_journey(),
        ArenaId::WOOD,
        Some(0),
        1,
    );
    let o = base.map_origin();
    let radius = sim::tuning::body_radius();
    let height = sim::tuning::body_height();
    let knee = Fx::ratio(6, 10);
    let limb = |s: &Solid| matches!(s.shape, Shape::Turned { half, half_wide, .. } if half.raw() >= half_wide.raw());
    let mut stopped = Vec::new();
    for s in boards.iter().filter(|s| limb(s)) {
        let Shape::Turned { half, cos, sin, .. } = s.shape else {
            unreachable!()
        };
        let (mx, mz) = s.middle();
        for way in [Fx::ONE, Fx::ONE.neg()] {
            let d = V3::new(cos.mul(way), Fx::ZERO, sin.mul(way));
            let back = half.mul(Fx::ratio(6, 10));
            let start = V3::new(mx.sub(d.x.mul(back)), s.max.y, mz.sub(d.z.mul(back))).sub(o);
            let mut w = base.clone();
            let pl = &mut w.players[0];
            pl.pos = start;
            pl.vel = V3::ZERO;
            pl.grounded = true;
            pl.fall_over = start.y;
            pl.health = 1000;
            w.players[1].pos = start.add(V3::new(Fx::from_int(60), Fx::ZERO, Fx::ZERO));
            let aim = sim::math::atan2_turns(d.z, d.x).raw() as u16;
            for f in 0..40 {
                let was = w.players[0].pos;
                w.advance([Input::looking_at(Input::W, aim, 0), Input::new(0)]);
                let p = w.players[0];
                if !p.grounded && f > 5 {
                    break;
                }
                let moved = p.pos.sub(was).flat_len();
                let stride = sim::tuning::move_speed().mul(sim::DT);
                if f <= 12 || !p.grounded || moved.raw() >= stride.mul(Fx::ratio(3, 10)).raw() {
                    continue;
                }
                // Stopped. By what?
                let ground = w.terrain();
                let ahead = p.pos.add(d.scale(Fx::ratio(1, 10)));
                let (feet, head) = (p.pos.y, p.pos.y.add(height));
                let lips: Vec<Solid> = ground
                    .around(ahead, Fx::ONE)
                    .filter(|b| {
                        b.max.y.raw() > feet.add(Fx::ratio(1, 100)).raw()
                            && b.min.y.raw() < head.raw()
                            && b.side_push(ahead.x, ahead.z, radius, feet, head).is_some()
                    })
                    .collect();
                let low = lips
                    .iter()
                    .all(|b| limb(b) && b.max.y.sub(feet).raw() <= knee.raw());
                let room = |top: Fx| {
                    !ground
                        .around(V3::new(p.pos.x, top, p.pos.z), radius)
                        .any(|c| {
                            let (lo, hi) = (top.add(Fx::ratio(1, 32)), top.add(height));
                            c.min.y.raw() < hi.raw()
                                && c.max.y.raw() > lo.raw()
                                && c.side_push(p.pos.x, p.pos.z, radius, lo, hi).is_some()
                        })
                };
                if !lips.is_empty() && low && lips.iter().all(|b| room(b.max.y)) {
                    let rise = lips.iter().map(|b| b.max.y.sub(feet).raw()).max().unwrap();
                    stopped.push(format!(
                        "at {:?}: a lip {} cm high",
                        p.pos.add(o),
                        rise * 100 / Fx::ONE.raw()
                    ));
                }
                break;
            }
        }
    }
    assert!(stopped.is_empty(), "{}", stopped.join("\n"));
}
