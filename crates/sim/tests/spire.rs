//! **The Spire** (`sim::arena::spire`, exploration 0019 §2a): Hearth's tower,
//! a shaft 315 m tall with an inverted dome on it 90 m across. It stands
//! where the table says, the chimney up the dome's stem is open, the dome's
//! underside is a thing a shot meets, nothing jutting from the shaft is in
//! the vine's way -- and a fighter climbs the vine, on up the chimney's, and
//! stands among the ruins 353 m up.

use sim::arena::spire::{self, AT, CHIMNEY, DOME_FROM, RIM, SHAFT, TOP};
use sim::arena::{ArenaId, Shape};
use sim::valley;
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

fn world() -> World {
    let j = valley::Journey {
        beaten: u32::MAX,
        ..valley::Journey::default()
    };
    World::arrive(
        [Class::Champion, Class::Bulwark],
        j,
        ArenaId::SPIRE,
        Some(0),
        1,
    )
}

/// The map's solids near the shaft.
fn near() -> Vec<sim::arena::Solid> {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(AT.0), cm(AT.1));
    let r = cm(RIM + 500);
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect()
}

/// The nearest solid a ray meets, in the world's frame.
fn ray(w: &World, from: V3, dir: V3, limit: Fx) -> Option<Fx> {
    w.terrain()
        .along(from, dir, limit, Fx::ONE)
        .filter_map(|s| sim::math::ray_hits_solid(from, dir, &s, Fx::ZERO))
        .min()
}

#[test]
fn the_spire_stands_with_its_dome_and_its_chimney() {
    let solids = near();
    let at_axis = |s: &&sim::arena::Solid| {
        let (x, z) = s.middle();
        x.sub(cm(AT.0)).abs().raw() < Fx::ONE.raw() && z.sub(cm(AT.1)).abs().raw() < Fx::ONE.raw()
    };
    let shaft = solids
        .iter()
        .filter(at_axis)
        .find(|s| {
            matches!(s.shape, Shape::Round { hollow, .. } if hollow == Fx::ZERO)
                && s.max.y.to_int() > 200
        })
        .expect("a shaft");
    assert_eq!(shaft.max.y.to_int(), SHAFT.2 / 100);
    let rings: Vec<_> = solids
        .iter()
        .filter(at_axis)
        .filter(|s| {
            matches!(s.shape, Shape::Round { hollow, .. } if hollow.raw() > 0)
                && s.min.y.to_int() >= DOME_FROM / 100
        })
        .collect();
    assert_eq!(rings.len(), 8, "the dome's slabs");
    let top = rings.iter().map(|s| s.max.y).max().unwrap();
    assert_eq!(
        top.to_int(),
        TOP / 100,
        "the dome's top is {} m up",
        top.to_int()
    );
    let widest = rings.iter().map(|s| s.max.x.sub(s.min.x)).max().unwrap();
    assert!(
        widest.to_int() >= 2 * RIM / 100 - 1,
        "the dome is {} m across",
        widest.to_int()
    );

    // Straight down the chimney from over the top: nothing until the shaft's
    // top, 70 m below the dome's.
    let w = world();
    let o = w.map_origin();
    let from = V3::new(cm(AT.0), cm(TOP + 1000), cm(AT.1)).sub(o);
    let scene_hit = ray(
        &w,
        from,
        V3::new(Fx::ZERO, Fx::from_int(-1), Fx::ZERO),
        Fx::from_int(500),
    );
    let d = scene_hit.expect("meets the shaft's top");
    assert!(
        d.sub(cm(TOP + 1000 - SHAFT.2)).abs().raw() < Fx::ONE.raw(),
        "down the chimney met something {} m down",
        d.to_f32_for_render()
    );
    // And up from the plaza, out under the rim: the dome's underside.
    let from = V3::new(cm(AT.0 + 3500), Fx::from_int(2), cm(AT.1)).sub(o);
    let up = ray(
        &w,
        from,
        V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO),
        Fx::from_int(500),
    );
    let d = up.expect("meets the dome");
    assert!(
        d.to_int() > DOME_FROM / 100 && d.to_int() < TOP / 100,
        "met the dome {} m up",
        d.to_int()
    );
}

/// Climb a vine to its top, holding jump and pressing at the shaft's axis so
/// the feet keep to it; `Ok` with the frames it took.
fn climb(w: &mut World, top: Fx, frames: u32) -> Result<u32, String> {
    let o = w.map_origin();
    for f in 0..frames {
        let p = w.players[0].pos.add(o);
        if p.y.raw() >= top.raw() {
            return Ok(f);
        }
        let (dx, dz) = (cm(AT.0).sub(p.x), cm(AT.1).sub(p.z));
        let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
        w.advance([
            Input::looking_at(Input::SPACE | Input::W, aim, 0),
            Input::new(0),
        ]);
    }
    let p = w.players[0].pos.add(o);
    Err(format!(
        "got to {} m of {}",
        p.y.to_f32_for_render(),
        top.to_f32_for_render()
    ))
}

/// Walk toward a map point at the fighter's height for a while, jumping
/// when `jump`, and return where she ends.
fn walk(w: &mut World, to: (Fx, Fx), frames: u32, jump: bool) -> V3 {
    let o = w.map_origin();
    for f in 0..frames {
        let p = w.players[0].pos.add(o);
        let (dx, dz) = (to.0.sub(p.x), to.1.sub(p.z));
        if sim::math::wide_flat_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(1, 2).raw() {
            break;
        }
        let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
        let bits = Input::W | if jump && f < 20 { Input::SPACE } else { 0 };
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
    }
    w.players[0].pos.add(o)
}

/// The plaza's height at the shaft: what every height in the table is above.
fn base() -> Fx {
    near()
        .iter()
        .filter(|s| matches!(s.shape, Shape::Round { hollow, .. } if hollow == Fx::ZERO))
        .map(|s| s.max.y)
        .filter(|y| y.to_int() > 200)
        .min()
        .unwrap()
        .sub(cm(SHAFT.2))
}

#[test]
fn a_fighter_climbs_to_the_ruins_by_the_vines() {
    let mut w = world();
    let o = w.map_origin();
    let b = base();
    // At the foot of the outside vine, on the plaza.
    let foot = V3::new(cm(AT.0), Fx::ZERO, cm(AT.1 - SHAFT.0 - 60));
    w.players[0].pos = foot.sub(o);
    w.players[0].health = 1000;
    // Up the outside vine and on up the chimney's: the shaft's top is
    // narrower than the chimney, so the one vine runs into the other.
    climb(&mut w, b.add(cm(TOP)), 60 * 200).expect("up the vines");
    // Out of the chimney's mouth, onto the dome's top.
    let p = walk(
        &mut w,
        (cm(AT.0 + 120), cm(AT.1 - CHIMNEY - 400)),
        180,
        true,
    );
    assert!(
        p.y.sub(b.add(cm(TOP))).abs().raw() < Fx::ratio(1, 5).raw(),
        "ended at {} m, not on the dome's top at {}",
        p.y.to_f32_for_render(),
        TOP / 100
    );
    assert!(w.players[0].health > 0);
    assert_eq!(w.arena, ArenaId::SPIRE);
}

#[test]
fn nothing_jutting_from_the_shaft_is_across_the_vine() {
    for v in spire::VINES {
        for s in near() {
            if !matches!(s.shape, Shape::Turned { .. }) {
                continue;
            }
            let (x, z) = s.middle();
            let inside = x.raw() > v.min.x.raw()
                && x.raw() < v.max.x.raw()
                && z.raw() > v.min.z.raw()
                && z.raw() < v.max.z.raw()
                && s.max.y.raw() > v.min.y.raw()
                && s.min.y.raw() < v.max.y.raw();
            assert!(
                !inside,
                "a stone at {} m is in a vine's way",
                s.max.y.to_f32_for_render()
            );
        }
    }
}
