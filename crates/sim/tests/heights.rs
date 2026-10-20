//! **The High Plane** (`sim::arena::heights`, exploration 0019 §2c): a table
//! mountain 300 m up and the routes to it and over it. The plane stands where
//! the table says, over the mountains, with the land held up to its edge so
//! what is drawn is what is stood on; no stone of any route is in the land or
//! hangs in the way of another; the Sky Road's gaps are what its table says;
//! no plain jump crosses a gully.
//!
//! Since 2026-10-20 the Hanging Stones, the Crown and the High Line are laid
//! out to need a class's tool here and there, which the stand-in player
//! (`tests/pilot`) does not have. So each hop is held to what its table says
//! it asks: **a plain hop** is inside every mover's plain running jump caught
//! at the hands, on paper and jumped by the pilot; **a tool hop** is past
//! every mover's plain jump, airdodge and all, and inside the reach of the
//! tools `sim::envelope` measures -- the Dual mage's second jump, the
//! Champion's vault, the Elementalist's stone. The Reaver's and the Blood
//! mage's tools, and the Elementalist's for distance, are not measured there,
//! so nothing here says they make a tool hop; the gaps were sized against the
//! measured three and the owner's word that every class has a tool for this.

mod pilot;

use std::sync::OnceLock;

use sim::arena::heights::{
    self, CROWN, CROWN_FOOT, CROWN_HOPS, FOOT, Hop, LADDER, LADDER_HOPS, Need, PLANE_AT,
    PLANE_HALF, SPAN, SPAN_HOPS, Stone,
};
use sim::arena::{ArenaId, Solid};
use sim::envelope::{self as e, Extra, Path};
use sim::valley::Journey;
use sim::{Class, Fx, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

#[test]
fn the_plane_stands_over_the_mountains_and_the_land_is_held_up_to_it() {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(PLANE_AT[0]), cm(PLANE_AT[2]));
    let floor = atlas.relief_at(x, z);
    assert!(
        floor.sub(cm(PLANE_AT[1])).abs().raw() < Fx::ONE.raw(),
        "the plane's floor is {} m",
        floor.to_f32_for_render()
    );
    // What the renderer draws there -- the land -- is the floor stood on.
    let land = atlas.land.as_ref().expect("land").height(x, z);
    assert!(
        land.sub(floor).abs().raw() < Fx::from_int(2).raw(),
        "the land is drawn at {} m under a floor at {}",
        land.to_f32_for_render(),
        floor.to_f32_for_render()
    );
    // Sixty metres past its south edge, the mountains, well below it.
    let below = atlas.relief_at(x, cm(PLANE_AT[2] - PLANE_HALF.1 - 6000));
    assert!(
        below.raw() < cm(PLANE_AT[1] - 3000).raw(),
        "sixty metres off its edge the land is at {} m",
        below.to_f32_for_render()
    );
}

/// Every stone of every route, on the map.
fn stones() -> Vec<Stone> {
    let mut all = vec![FOOT, CROWN_FOOT];
    all.extend(LADDER.iter().chain(SPAN.iter()).chain(CROWN.iter()));
    all.extend(heights::sky_road());
    all.extend(heights::high_line());
    all
}

#[test]
fn no_stone_of_a_route_is_in_the_land() {
    let atlas = sim::atlas::valley();
    for s in stones().iter().filter(|s| !s.grounded) {
        for (dx, dz) in [(-1, -1), (1, -1), (-1, 1), (1, 1), (0, 0)] {
            let (x, z) = (cm(s.x + dx * s.half), cm(s.z + dz * s.half));
            let ground = atlas.relief_at(x, z);
            assert!(
                ground.raw() < cm(s.top - 600).raw(),
                "a stone at ({}, {}) with its top at {} m is in the land, at {} m there",
                s.x / 100,
                s.z / 100,
                s.top / 100,
                ground.to_f32_for_render()
            );
        }
    }
}

/// **Nothing of one hop hangs in the way of another**: no stone is over or
/// under another within nine metres -- a jump's height and a body's -- and
/// none is inside a gully's walls. The routes are laid by walking a table,
/// round squares and along bows, so this is what would catch one loop of a
/// column coming down on the last.
#[test]
fn no_stone_hangs_in_the_way_of_another() {
    let all = stones();
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            let near = |p: i32, q: i32| (p - q).abs() < a.half + b.half + 150;
            if near(a.x, b.x) && near(a.z, b.z) {
                assert!(
                    (a.top - b.top).abs() >= 900,
                    "a stone at ({}, {}, {} m) and one at ({}, {}, {} m)",
                    a.x / 100,
                    a.z / 100,
                    a.top / 100,
                    b.x / 100,
                    b.z / 100,
                    b.top / 100
                );
            }
        }
    }
    let walls: Vec<Solid> = heights::gullies().flat_map(|(_, w)| w).collect();
    for s in &all {
        let top = top_of(s);
        for w in &walls {
            let apart = |lo: Fx, hi: Fx, a: Fx, b: Fx| hi.raw() <= a.raw() || b.raw() <= lo.raw();
            let clear = apart(top.min.x, top.max.x, w.min.x, w.max.x)
                || apart(top.min.z, top.max.z, w.min.z, w.max.z)
                || apart(top.min.y, top.max.y, w.min.y, w.max.y);
            assert!(
                clear,
                "a stone at ({}, {}) is in a gully's wall",
                s.x / 100,
                s.z / 100
            );
        }
    }
}

#[test]
fn the_sky_road_runs_from_the_needle_s_dome_to_the_plane() {
    let road = heights::sky_road();
    let first = road[0];
    let last = *road.last().unwrap();
    let axis = sim::arena::spire::AT;
    let from_axis =
        (((first.x - axis.0) as i64).pow(2) + ((first.z - axis.1) as i64).pow(2)).isqrt();
    assert!(
        (from_axis as i32) < sim::arena::spire::RIM + first.half + 300,
        "the first island is {} m from the Needle's axis",
        from_axis / 100
    );
    assert!(
        first.top <= sim::arena::spire::TOP,
        "the first island is over the dome"
    );
    // The last is a jump short of the plane's south-west corner, over it.
    assert!(last.top > PLANE_AT[1], "the last island is under the plane");
    let to_plane = PLANE_AT[2] - PLANE_HALF.1 - (last.z + last.half);
    assert!(
        (0..2600).contains(&to_plane),
        "the last island is {} m short of the plane",
        to_plane / 100
    );
}

/// A stone's top as a box, on the map: what a hop is judged against.
fn top_of(s: &Stone) -> Solid {
    Solid::cm(
        [s.x - s.half, s.top - 100, s.z - s.half],
        [s.x + s.half, s.top, s.z + s.half],
        sim::arena::Material::Rock,
    )
}

#[test]
fn the_span_ends_a_hop_short_of_the_plane_and_above_it() {
    let last = SPAN.last().unwrap();
    let edge = PLANE_AT[2] - PLANE_HALF.1;
    let short = edge - (last.z + last.half);
    assert!(
        (100..500).contains(&short),
        "the span ends {} m short of the plane",
        short as f32 / 100.0
    );
    let over = last.top - PLANE_AT[1];
    assert!(
        (200..900).contains(&over),
        "the span's last stone is {} m over the plane",
        over as f32 / 100.0
    );
}

/// **The Crown stands on the plane and tops the valley**: its foot on the
/// floor inside the plane's edge, every stone over the plane, and its island
/// higher than the Needle's dome.
#[test]
fn the_crown_stands_on_the_plane_and_tops_the_needle() {
    let over = |s: &Stone| {
        (s.x - s.half - (PLANE_AT[0] - PLANE_HALF.0)) > 300
            && (PLANE_AT[0] + PLANE_HALF.0) - (s.x + s.half) > 300
            && (s.z - s.half - (PLANE_AT[2] - PLANE_HALF.1)) > 300
            && (PLANE_AT[2] + PLANE_HALF.1) - (s.z + s.half) > 300
    };
    for s in std::iter::once(&CROWN_FOOT).chain(CROWN.iter()) {
        assert!(
            over(s),
            "a stone of the Crown at ({}, {}) is not over the plane",
            s.x / 100,
            s.z / 100
        );
    }
    let island = CROWN.last().unwrap();
    assert!(heights::rests(island), "the Crown's island has no cairn");
    assert!(
        island.top > sim::arena::spire::TOP,
        "the Crown is {} m up, under the Needle's {}",
        island.top / 100,
        sim::arena::spire::TOP / 100
    );
}

/// **The High Line leaves the Sky Road and joins it again**: off one of its
/// islands with a cairn, onto another eight stones on, a long hop at each
/// end -- and bowed out of the road's line, so no stone of it is near one of
/// the stretch it cuts short.
#[test]
fn the_high_line_leaves_the_sky_road_and_joins_it() {
    let road = heights::sky_road();
    let line = heights::high_line();
    let (a, b) = (road[heights::HIGH_LINE_FROM], road[heights::HIGH_LINE_TO]);
    assert!(
        a.half >= 600 && b.half >= 600,
        "it does not leave or join an island"
    );
    for (what, g) in [
        ("off the island it leaves", heights::gap(&a, &line[0])),
        (
            "onto the island it joins",
            heights::gap(line.last().unwrap(), &b),
        ),
    ] {
        assert!(
            (1500..2600).contains(&g),
            "the High Line's hop {what} is {} m",
            g as f32 / 100.0
        );
    }
    for s in &line {
        for r in &road[heights::HIGH_LINE_FROM + 1..heights::HIGH_LINE_TO] {
            let flat = (((s.x - r.x) as i64).pow(2) + ((s.z - r.z) as i64).pow(2)).isqrt() as i32;
            assert!(
                flat - s.half - r.half > 1000,
                "a stone of the High Line is {} m from one of the Sky Road's",
                (flat - s.half - r.half) / 100
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The hops, against the envelope
// ---------------------------------------------------------------------------

/// **One hop of a route**: where it is jumped from, where to, and what its
/// table says it asks (`None`: a gully).
#[derive(Clone, Copy)]
struct Leg {
    from: Stone,
    to: Stone,
    need: Option<Need>,
}

impl Leg {
    fn gap(&self) -> i32 {
        heights::gap(&self.from, &self.to)
    }
    fn rise(&self) -> i32 {
        self.to.top - self.from.top
    }
}

fn legs(from: Stone, hops: &[Hop], stones: &[Stone]) -> Vec<Leg> {
    let mut last = from;
    hops.iter()
        .zip(stones)
        .map(|(h, &to)| {
            let leg = Leg {
                from: last,
                to,
                need: h.need(),
            };
            last = to;
            leg
        })
        .collect()
}

/// The Hanging Stones, foot to the span's end.
fn hanging() -> Vec<Leg> {
    let mut all = legs(FOOT, &LADDER_HOPS, &LADDER);
    all.extend(legs(LADDER[LADDER.len() - 1], &SPAN_HOPS, &SPAN));
    all
}

fn crown() -> Vec<Leg> {
    legs(CROWN_FOOT, &CROWN_HOPS, &CROWN)
}

/// The High Line, island to island: every hop a tool's.
fn high_line() -> Vec<Leg> {
    let road = heights::sky_road();
    let mut at = vec![road[heights::HIGH_LINE_FROM]];
    at.extend(heights::high_line());
    at.push(road[heights::HIGH_LINE_TO]);
    at.windows(2)
        .map(|p| Leg {
            from: p[0],
            to: p[1],
            need: Some(Need::Tool),
        })
        .collect()
}

/// The Sky Road, island to island.
fn sky_road() -> Vec<Leg> {
    heights::sky_road()
        .windows(2)
        .map(|p| Leg {
            from: p[0],
            to: p[1],
            need: None,
        })
        .collect()
}

/// What each mover's feet do, measured once: the plain running jump, and
/// the same with the airdodge thrown at every timing.
struct Plain {
    class: Class,
    path: Path,
    dodged: Vec<Path>,
}

fn plain() -> &'static [Plain] {
    static P: OnceLock<Vec<Plain>> = OnceLock::new();
    P.get_or_init(|| {
        sim::class::MOVERS
            .iter()
            .map(|&class| Plain {
                class,
                path: e::off_the_edge(class, Extra::Nothing),
                dodged: e::airdodge_paths(class),
            })
            .collect()
    })
}

/// **Can a set of paths make a hop**: high enough to catch the top with 30 cm
/// to spare, and far enough at that height -- the crags' and the
/// Waterfall's rule.
fn makes(paths: &[Path], gap: i32, rise: i32) -> bool {
    let apex = paths.iter().map(e::path_apex).fold(Fx::ZERO, Fx::max);
    let (g, r) = (Fx::ratio(gap, 100), Fx::ratio(rise, 100));
    apex.add(sim::tuning::ledge_reach_high()).raw() >= r.add(Fx::ratio(3, 10)).raw()
        && e::widest_caught(paths, r).is_some_and(|w| w.raw() >= g.raw())
}

/// The tools `sim::envelope` measures, once.
struct Tools {
    /// The Dual mage's second jump, at every timing, her bars set.
    second: Vec<Path>,
    /// The Champion's vault: its apex and how far it carries.
    vault: (Fx, Fx),
    /// The Elementalist's one-stone jump: its apex.
    stone: Fx,
}

fn tools() -> &'static Tools {
    static T: OnceLock<Tools> = OnceLock::new();
    T.get_or_init(|| Tools {
        second: (19..64u32)
            .step_by(3)
            .map(|k| e::off_the_edge(Class::DualMage, Extra::SecondJump(k)))
            .collect(),
        vault: e::vault(),
        stone: e::stone_apexes().0,
    })
}

/// Which measured tools make a hop. Read generously where the measure is a
/// number rather than a path: the vault as its apex over the rise and its
/// carry, less a metre of run-up, over the gap and the rise together; the
/// stone as its apex over the rise, for a hop that is mostly up.
fn tools_for(gap: i32, rise: i32) -> Vec<&'static str> {
    let t = tools();
    let reach = sim::tuning::ledge_reach_high();
    let high = |apex: Fx| apex.add(reach).raw() >= cm(rise + 30).raw();
    let mut out = Vec::new();
    if makes(&t.second, gap, rise) {
        out.push("Dual mage's second jump");
    }
    if high(t.vault.0) && cm(gap + rise.max(0) + 100).raw() <= t.vault.1.raw() {
        out.push("Champion's vault");
    }
    if gap <= 350 && high(t.stone) {
        out.push("Elementalist's stone");
    }
    out
}

/// **Every plain hop is inside every mover's plain running jump, caught at
/// the hands**, with the crags' margin: the Hanging Stones' and the Crown's.
#[test]
fn every_plain_hop_is_inside_every_mover_s_jump_on_paper() {
    let mut failed = Vec::new();
    for (route, legs) in [("Hanging Stones", hanging()), ("Crown", crown())] {
        for (k, leg) in legs.iter().enumerate() {
            if leg.need != Some(Need::Jump) {
                continue;
            }
            for p in plain() {
                if !makes(std::slice::from_ref(&p.path), leg.gap(), leg.rise()) {
                    failed.push(format!(
                        "{}: {route} hop {k}, {} m out and {} m up",
                        p.class.name(),
                        leg.gap() as f32 / 100.0,
                        leg.rise() as f32 / 100.0
                    ));
                }
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **Every tool hop is past every plain jump** -- every mover's, with its
/// airdodge thrown at its best, given 30 cm both ways -- **and inside the
/// reach of some tool**: at least two of the measured three for the Hanging
/// Stones, which are one of the two ways up; at least one for the Crown and
/// the High Line, which are the hard ones.
#[test]
fn every_tool_hop_needs_a_tool_and_a_tool_reaches_it() {
    let mut failed = Vec::new();
    for (route, legs, least) in [
        ("Hanging Stones", hanging(), 2),
        ("Crown", crown(), 1),
        ("High Line", high_line(), 1),
    ] {
        for (k, leg) in legs.iter().enumerate() {
            if leg.need != Some(Need::Tool) {
                continue;
            }
            let (gap, rise) = (leg.gap(), leg.rise());
            for p in plain() {
                if makes(&p.dodged, gap - 30, rise - 30) {
                    failed.push(format!(
                        "{route} hop {k} ({} m out, {} m up) is a plain jump for the {}",
                        gap as f32 / 100.0,
                        rise as f32 / 100.0,
                        p.class.name()
                    ));
                }
            }
            let by = tools_for(gap, rise);
            println!(
                "{route} hop {k}: {} m out, {} m up -- {}",
                gap as f32 / 100.0,
                rise as f32 / 100.0,
                if by.is_empty() {
                    "no measured tool".to_string()
                } else {
                    by.join(", ")
                }
            );
            if by.len() < least {
                failed.push(format!(
                    "{route} hop {k} ({} m out, {} m up) is reached by {:?}",
                    gap as f32 / 100.0,
                    rise as f32 / 100.0,
                    by
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// How far inside the Dual mage's second jump a hop is, in centimetres --
/// the one measured tool that reaches every tool hop, so a common measure of
/// how hard each is: the less room, the harder.
fn room(leg: &Leg) -> i32 {
    let r = Fx::ratio(leg.rise(), 100);
    let w = e::widest_caught(&tools().second, r).map_or(0, e::to_cm);
    w - leg.gap()
}

/// **The Crown and the High Line are harder than the Sky Road**, by the room
/// the Dual mage's second jump has over each hop: the Crown's hardest hop
/// leaves less than the Sky Road's hardest, and the High Line leaves less
/// on the whole -- and both land on smaller stones.
#[test]
fn the_crown_and_the_high_line_are_harder_than_the_sky_road() {
    let past_plain = |l: &&Leg| !plain().iter().any(|p| makes(&p.dodged, l.gap(), l.rise()));
    let sky = sky_road();
    let sky: Vec<&Leg> = sky.iter().filter(past_plain).collect();
    let crown = crown();
    let crown: Vec<&Leg> = crown
        .iter()
        .filter(|l| l.need == Some(Need::Tool))
        .collect();
    let line = high_line();
    let line: Vec<&Leg> = line.iter().collect();
    let least = |legs: &[&Leg]| legs.iter().map(|l| room(l)).min().unwrap();
    let mean = |legs: &[&Leg]| legs.iter().map(|l| room(l)).sum::<i32>() / legs.len() as i32;
    let small = |legs: &[&Leg]| legs.iter().map(|l| l.to.half).sum::<i32>() / legs.len() as i32;
    println!(
        "room: Sky Road least {} mean {}, Crown least {} mean {}, High Line least {} mean {}",
        least(&sky),
        mean(&sky),
        least(&crown),
        mean(&crown),
        least(&line),
        mean(&line)
    );
    assert!(
        least(&crown) < least(&sky),
        "the Crown's hardest is no harder"
    );
    assert!(mean(&line) < mean(&sky), "the High Line is no harder");
    // The High Line's last hop lands on the Sky Road's island.
    let line_stones = &line[..line.len() - 1];
    assert!(small(&crown) < small(&sky) && small(line_stones) < small(&sky));
}

/// **A rest before every tool hop of the Hanging Stones**: a cairn on one of
/// the four stones before it, so a miss costs a hop or two.
#[test]
fn a_cairn_is_close_before_every_tool_hop_of_the_hanging_stones() {
    let legs = hanging();
    for (k, leg) in legs.iter().enumerate() {
        if leg.need != Some(Need::Tool) {
            continue;
        }
        let near = legs[k.saturating_sub(3)..=k]
            .iter()
            .any(|l| heights::rests(&l.from));
        assert!(near, "Hanging Stones hop {k} has no cairn before it");
    }
}

/// **No class's plain jump crosses a gully**, so the walls are the way: the
/// longest plain running jump there is, at the gully's drop, falls short of
/// its length even caught at the hands.
#[test]
fn no_plain_jump_crosses_a_gully() {
    use sim::arena::gully::{DROP, LONG};
    for p in plain() {
        let rise = Fx::ratio(-DROP, 100);
        let caught = e::widest_caught(std::slice::from_ref(&p.path), rise);
        assert!(
            caught.is_none_or(|c| c.raw() < cm(LONG - 50).raw()),
            "{} jumps a gully without a wall: {:?}",
            p.class.name(),
            caught
        );
    }
}

/// **Every mover makes every plain hop and every gully of the Hanging
/// Stones, and the Crown's plain hops**, jumped by the stand-in player: each
/// plain hop landed or caught and climbed, each gully crossed off its walls,
/// and off the span's last stone onto the plane. The tool hops are skipped --
/// the pilot has no tools -- and are held to the envelope above instead.
#[test]
fn every_mover_makes_the_plain_hops_and_the_gullies() {
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        let j = Journey {
            beaten: u32::MAX,
            ..Journey::default()
        };
        let mut w = World::arrive([class, class], j, ArenaId::BANK, None, 1);
        let o = w.map_origin();
        w.players[0].pos = V3::new(cm(FOOT.x), cm(FOOT.top), cm(FOOT.z)).sub(o);
        let mut kicked = Vec::new();
        let mut skipped = 0;
        for (route, legs) in [("Hanging Stones", hanging()), ("Crown", crown())] {
            for (k, leg) in legs.iter().enumerate() {
                let (from, to) = (
                    pilot::local(&w, top_of(&leg.from)),
                    pilot::local(&w, top_of(&leg.to)),
                );
                let done = match leg.need {
                    Some(Need::Tool) => {
                        skipped += 1;
                        continue;
                    }
                    Some(Need::Jump) => pilot::jump(&mut w, from, to),
                    None => {
                        let d = (
                            (leg.to.x - leg.from.x).signum(),
                            (leg.to.z - leg.from.z).signum(),
                        );
                        pilot::kick_across(&mut w, from, to, d).map(|n| kicked.push(n))
                    }
                };
                if let Err(e) = done {
                    failed.push(format!(
                        "{}: {route} hop {k} ({} m out, {} m up, {} m high): {e}",
                        class.name(),
                        leg.gap() as f32 / 100.0,
                        leg.rise() as f32 / 100.0,
                        leg.to.top / 100
                    ));
                }
            }
            if route == "Hanging Stones" {
                // And off the span's last stone onto the plane.
                let last = *SPAN.last().unwrap();
                let edge = PLANE_AT[2] - PLANE_HALF.1;
                let plane = Solid::cm(
                    [last.x - 600, PLANE_AT[1] - 100, edge + 200],
                    [last.x + 600, PLANE_AT[1], edge + 1400],
                    sim::arena::Material::Grass,
                );
                let (from, to) = (pilot::local(&w, top_of(&last)), pilot::local(&w, plane));
                if let Err(e) = pilot::jump(&mut w, from, to) {
                    failed.push(format!("{}: onto the plane: {e}", class.name()));
                } else if w.arena != ArenaId::HIGH_PLANE {
                    failed.push(format!("{}: ended in {}", class.name(), w.arena().name));
                }
            }
        }
        println!(
            "{}: kicks per gully {kicked:?}, {skipped} tool hops not jumped",
            class.name()
        );
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
