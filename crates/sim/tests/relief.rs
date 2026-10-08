//! The terrain relief (forms.md, "Relief"): the hand-placed rises and dips
//! some arenas have on top of their flat floor. These live here rather than
//! beside the tables because their thresholds are test standards, not
//! tuning values, and `knobs.rs` reads every line of the simulation.

use sim::arena::{ArenaId, relief};
use sim::fixed::Fx;

#[test]
fn a_bump_is_its_height_at_its_crown_and_nothing_at_its_rim() {
    let b = relief::of(ArenaId::HORNBACK)[0];
    let top = relief::height_at(ArenaId::HORNBACK, b.x, b.z);
    assert_eq!(top, b.height);
    // The west rim, out past the arena's edge where no other bump reaches.
    let rim = relief::height_at(ArenaId::HORNBACK, b.x.sub(b.radius), b.z);
    assert_eq!(rim.raw(), 0);
}

#[test]
fn a_flat_arena_is_flat() {
    assert_eq!(
        relief::height_at(ArenaId::PROVING_GROUND, Fx::from_int(3), Fx::from_int(-2)).raw(),
        0
    );
}

#[test]
fn relief_is_gentle_everywhere() {
    // Nowhere steeper than a body can walk: no more than a tenth of a
    // metre of rise over half a metre of floor, and nowhere over a metre, in
    // any arena with a creature in it. **The valley's reaches climb**: their
    // floor may rise a quarter of a metre over half a metre of floor (the
    // scree) and tens of metres in all -- and steeper under a cliff, where
    // no foot reaches it, which is how a reach climbs from one level to the
    // next (`relief::Ramp`).
    for a in sim::arena::all() {
        if relief::is_flat(a.id) {
            continue;
        }
        let climbs =
            sim::valley::place(a.id).is_some_and(|p| matches!(p.kind, sim::valley::Kind::Reach));
        let (step, most) = if climbs {
            (Fx::ratio(1, 4), Fx::from_int(40))
        } else {
            (Fx::ratio(1, 10), Fx::ONE)
        };
        let under_rock = |x: Fx, z: Fx| {
            a.solids()
                .iter()
                .any(|s| !s.hangs() && s.over(x, z, Fx::ratio(1, 2)))
        };
        let b = a.bounds;
        let mut x = b.lo_x;
        while x.raw() < b.hi_x.raw() {
            let mut z = b.lo_z;
            while z.raw() < b.hi_z.raw() {
                if climbs && under_rock(x, z) {
                    z = z.add(Fx::ratio(1, 2));
                    continue;
                }
                let h = relief::height_at(a.id, x, z);
                let hx = relief::height_at(a.id, x.add(Fx::ratio(1, 2)), z);
                let hz = relief::height_at(a.id, x, z.add(Fx::ratio(1, 2)));
                assert!(
                    hx.sub(h).abs().raw() <= step.raw(),
                    "{} at {x:?},{z:?}",
                    a.name
                );
                assert!(
                    hz.sub(h).abs().raw() <= step.raw(),
                    "{} at {x:?},{z:?}",
                    a.name
                );
                assert!(
                    h.abs().raw() <= most.raw(),
                    "{} at {x:?},{z:?} is {h:?}",
                    a.name
                );
                z = z.add(Fx::ratio(1, 2));
            }
            x = x.add(Fx::ratio(1, 2));
        }
    }
}

/// Everything the fight lays on the floor is laid at the plain, and a rise
/// may run into a solid's base but the floor may never fall away under one.
#[test]
fn relief_keeps_clear_of_what_stands_on_the_floor() {
    use sim::arena::{Area, Material};
    // Three centimetres: a bump's rim, where it joins the plain.
    let hair = Fx::ratio(3, 100);
    let at = |id: ArenaId, x: Fx, z: Fx| relief::height_at(id, x, z);
    for a in sim::arena::all() {
        if relief::is_flat(a.id) {
            continue;
        }
        for s in a.solids() {
            let corners = [
                (s.min.x, s.min.z),
                (s.min.x, s.max.z),
                (s.max.x, s.min.z),
                (s.max.x, s.max.z),
                (
                    s.min.x.add(s.max.x).div(Fx::from_int(2)),
                    s.min.z.add(s.max.z).div(Fx::from_int(2)),
                ),
            ];
            for (x, z) in corners {
                let h = at(a.id, x, z);
                assert!(
                    h.raw() >= hair.neg().raw(),
                    "{}: the floor falls away under a solid at {x:?},{z:?} by {h:?}",
                    a.name
                );
                // A low thing -- a step, a trunk, an island -- sits on the
                // plain, or the floor would swallow it.
                if !s.hangs() && s.max.y.sub(s.min.y).raw() <= Fx::ratio(6, 10).raw() {
                    assert!(
                        h.raw() <= hair.raw(),
                        "{}: a rise swallows a low solid at {x:?},{z:?} by {h:?}",
                        a.name
                    );
                }
            }
        }
        // Water lies level.
        for r in a.regions {
            if r.material != Material::Water {
                continue;
            }
            let points: [(Fx, Fx); 5] = match r.area {
                Area::Rect { lo, hi } => [
                    (lo.0, lo.1),
                    (lo.0, hi.1),
                    (hi.0, lo.1),
                    (hi.0, hi.1),
                    (
                        lo.0.add(hi.0).div(Fx::from_int(2)),
                        lo.1.add(hi.1).div(Fx::from_int(2)),
                    ),
                ],
                Area::Disc { at: c, radius } => [
                    (c.0, c.1),
                    (c.0.add(radius), c.1),
                    (c.0.sub(radius), c.1),
                    (c.0, c.1.add(radius)),
                    (c.0, c.1.sub(radius)),
                ],
            };
            for (x, z) in points {
                let h = at(a.id, x, z);
                assert!(
                    h.abs().raw() <= hair.raw(),
                    "{}: water on a slope at {x:?},{z:?} ({h:?})",
                    a.name
                );
            }
        }
        // Sites stand on the plain.
        for site in a.sites {
            for &(x, z) in site.route {
                let h = at(a.id, Fx::ratio(x, 100), Fx::ratio(z, 100));
                assert!(
                    h.abs().raw() <= hair.raw(),
                    "{}: site {} on a slope at {x},{z}",
                    a.name,
                    site.name
                );
            }
        }
        // The meadow's boulders are hazard cells, not solids, and break.
        for &(x, z) in sim::species::hornback::rules::boulders(a.id) {
            if a.creature != Some(sim::species::SpeciesId::HORNBACK) {
                break;
            }
            for (dx, dz) in [(0, 0), (-125, -125), (125, -125), (-125, 125), (125, 125)] {
                let h = at(a.id, Fx::ratio(x + dx, 100), Fx::ratio(z + dz, 100));
                assert!(
                    h.abs().raw() <= hair.raw(),
                    "{}: boulder at {x},{z} on a slope ({h:?})",
                    a.name
                );
            }
        }
    }
}
