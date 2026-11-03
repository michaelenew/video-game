//! **Every fight, measured in reaches and jumps** (`sim::fightspace`,
//! `docs/design/fight-spaces.md`).
//!
//!     cargo run --release -p sim --bin fightspace
//!     cargo run --release -p sim --bin fightspace -- --plan quarry > quarry.svg
//!
//! One line a place: its size in reaches, how far its starts are from its
//! middle, how much of where you can stand is out of reach of the middle,
//! its levels a jump and a half apart, and what in it moves you. The
//! versus stages first, then every other place a fight is fought alone in.

use sim::arena::{self, Arena};
use sim::class::MOVERS;
use sim::fightspace::{self, Measure};
use sim::fixed::Fx;

fn tenths(v: Fx) -> String {
    let t = v.mul(Fx::from_int(10)).to_int();
    format!("{}.{}", t / 10, t.abs() % 10)
}

fn line(a: &Arena, m: &Measure, r: Fx, j: Fx) {
    let tiers: Vec<String> = m.tiers.iter().map(|&h| tenths(h)).collect();
    println!(
        "{:<16} {:>4}x{:<4}m  {:>3}x{:<3}R  start {:>4}m {:>3}R  beyond R {:>3}%  farthest {:>3}R  levels {} [{}] top {:>3}J  rings {:>2}/{:>2}/{:>2}  vents {} drifts {} hanging {:>2} ways {:>2} ({} to the point)",
        a.name,
        m.length.to_int(),
        m.width.to_int(),
        tenths(m.length.div(r)),
        tenths(m.width.div(r)),
        m.start_to_centre.to_int(),
        tenths(m.start_to_centre.div(r)),
        m.beyond_reach,
        tenths(m.farthest.div(r)),
        m.tiers.len(),
        tiers.join(" "),
        tenths(m.top().div(j)),
        m.rings[0],
        m.rings[1],
        m.rings[2],
        m.vents,
        m.drifts,
        m.hanging,
        m.ways,
        m.ways_to_centre,
    );
}

/// Metres to the plan's pixels.
const PX: i32 = 6;

/// A length in the plan's pixels, from metres in fixed point.
fn px(v: Fx) -> i32 {
    v.mul(Fx::from_int(PX)).to_int()
}

/// **A plan of a place, from above, as SVG**: every box coloured by the
/// height of its top (dark low, light high), boxes that hang outlined in
/// dashes, updrafts in blue, drifts in orange at both stops, the point, the
/// starts, and the rings of a fifth, two fifths and one reach round the
/// middle. North (+z) is up.
fn plan(a: &'static Arena, r: Fx, j: Fx) {
    let m = fightspace::measure(a, r, j);
    let b = a.bounds;
    let (w, h) = (px(b.hi_x.sub(b.lo_x)), px(b.hi_z.sub(b.lo_z)));
    let x = |v: Fx| px(v.sub(b.lo_x));
    let z = |v: Fx| px(b.hi_z.sub(v));
    let top = a
        .solids()
        .iter()
        .map(|s| s.max.y.sub(m.centre.y))
        .chain([Fx::ONE])
        .max_by_key(|y| y.raw())
        .unwrap_or(Fx::ONE);
    let shade = |y: Fx| {
        let t = (y
            .sub(m.centre.y)
            .max(Fx::ZERO)
            .mul(Fx::from_int(200))
            .div(top))
        .to_int();
        let g = 40 + t.clamp(0, 200);
        format!("rgb({g},{g},{})", (g + 20).min(255))
    };
    let ground = arena::Terrain::bare(a);
    println!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="sans-serif" font-size="11">"#
    );
    println!(r#"<rect width="{w}" height="{h}" fill="rgb(70,96,64)"/>"#);
    let mut solids: Vec<_> = a.solids().iter().collect();
    solids.sort_by_key(|s| s.max.y.raw());
    for s in solids {
        let dash = if ground.hangs(s) {
            r#" stroke-dasharray="4 3""#
        } else {
            ""
        };
        let style = format!(
            r#"fill="{}" stroke="black" stroke-width="1"{dash}"#,
            shade(s.max.y)
        );
        let (cx, cz) = (
            sim::math::midpoint(s.min.x, s.max.x),
            sim::math::midpoint(s.min.z, s.max.z),
        );
        match s.shape {
            arena::Shape::Round { bottom, top, .. } => println!(
                r#"<circle cx="{}" cy="{}" r="{}" {style}/>"#,
                x(cx),
                z(cz),
                px(bottom.max(top))
            ),
            arena::Shape::Turned {
                half,
                half_wide,
                cos,
                sin,
            } => {
                let (l, wd) = (px(half), px(half_wide));
                let deg = -(sim::math::atan2_turns(sin, cos).mul(Fx::from_int(360))).to_int();
                println!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" transform="rotate({deg} {} {})" {style}/>"#,
                    x(cx) - l,
                    z(cz) - wd,
                    2 * l,
                    2 * wd,
                    x(cx),
                    z(cz)
                );
            }
            arena::Shape::Box => println!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" {style}/>"#,
                x(s.min.x),
                z(s.max.z),
                px(s.max.x.sub(s.min.x)),
                px(s.max.z.sub(s.min.z))
            ),
        }
        println!(
            r#"<text x="{}" y="{}" fill="black">{}</text>"#,
            x(cx) - 8,
            z(cz) + 4,
            tenths(s.max.y.sub(m.centre.y))
        );
    }
    for v in sim::valley::vents(a.id) {
        println!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="rgba(80,160,255,0.5)" stroke="rgb(40,90,255)"/><text x="{}" y="{}" fill="rgb(20,40,160)">^{}</text>"#,
            x(v.at.0),
            z(v.at.1),
            px(v.radius).max(3),
            x(v.at.0) + 6,
            z(v.at.1) - 6,
            tenths(v.top.sub(m.centre.y))
        );
    }
    for d in sim::versus::drifts(a.id) {
        for s in [d.solid, d.at(d.wait as u32 + d.go as u32)] {
            println!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgba(255,150,40,0.6)" stroke="rgb(200,90,0)"/>"#,
                x(s.min.x),
                z(s.max.z),
                px(s.max.x.sub(s.min.x)).max(2),
                px(s.max.z.sub(s.min.z)).max(2)
            );
        }
    }
    let (cx, cz) = (x(m.centre.x), z(m.centre.z));
    for (k, n) in [(1, 5), (2, 5), (1, 1)] {
        println!(
            r#"<circle cx="{cx}" cy="{cz}" r="{}" fill="none" stroke="rgba(255,255,255,0.6)" stroke-dasharray="8 6"/>"#,
            px(r.mul(Fx::ratio(k, n)))
        );
    }
    if let Some(hill) = sim::versus::of(a.id).and_then(|s| s.hill) {
        println!(
            r#"<circle cx="{cx}" cy="{cz}" r="{}" fill="rgba(255,220,60,0.7)" stroke="black"/>"#,
            px(hill.radius)
        );
    }
    for (k, mark) in a.spawns.versus.iter().enumerate() {
        let colour = if k == 0 {
            "rgb(60,120,255)"
        } else {
            "rgb(255,140,40)"
        };
        println!(
            r#"<circle cx="{}" cy="{}" r="6" fill="{colour}" stroke="white" stroke-width="2"/>"#,
            x(mark.at.x),
            z(mark.at.z)
        );
    }
    println!(
        r#"<text x="6" y="14" fill="white">{} -- {}x{} m, rings at 0.2, 0.4 and 1 reach ({} m); numbers are tops over the starts</text></svg>"#,
        a.name,
        m.length.to_int(),
        m.width.to_int(),
        r.to_int()
    );
}

fn main() {
    let (r, j) = (fightspace::reach(), fightspace::jump());
    let args: Vec<String> = std::env::args().collect();
    if let Some(k) = args.iter().position(|a| a == "--plan") {
        let name = args.get(k + 1).map(String::as_str).unwrap_or("quarry");
        match arena::named(name) {
            Some(a) => plan(a, r, j),
            None => eprintln!("no arena named {name}"),
        }
        return;
    }
    println!(
        "reach R = {} m: the movers' longest pointed reaches are",
        tenths(r)
    );
    for c in MOVERS {
        println!(
            "  {:<14} {} m  (full jump {} m)",
            c.name(),
            tenths(fightspace::pointed_reach(c)),
            tenths(fightspace::full_jump(c))
        );
    }
    println!(
        "jump J = {} m; a level is a jump and a half, {} m, over the one below\n",
        tenths(j),
        tenths(j.add(j.div(Fx::from_int(2))))
    );
    println!("-- versus stages --");
    for s in sim::versus::STAGES {
        let a = s.arena.get();
        line(a, &fightspace::measure(a, r, j), r, j);
    }
    println!("\n-- every other fight fought alone --");
    for a in arena::all() {
        let fought = a.creature.is_some() || a.id == arena::ArenaId::PROVING_GROUND;
        if fought && sim::versus::of(a.id).is_none() {
            line(a, &fightspace::measure(a, r, j), r, j);
        }
    }
}
