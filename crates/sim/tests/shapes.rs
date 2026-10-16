//! **Round and turned solids** (2026-10-15, exploration 0019 VALE-7: "I'd
//! prefer we have more shapes to work with"). A trunk is round, a dome is
//! slabs of a cone, a branch is a box turned to any bearing -- and each is
//! what a body stands on, walks into and shoots, exactly where it is drawn.

use sim::arena::{ArenaId, Material, Shape, Solid, Terrain};
use sim::fixed::Fx;
use sim::math::V3;

fn m(v: i32) -> Fx {
    Fx::from_int(v)
}

fn near(a: Fx, b: Fx, within: Fx) -> bool {
    a.sub(b).abs().raw() <= within.raw()
}

fn ground_with(s: Solid) -> Terrain {
    let mut t = Terrain::bare(ArenaId::PROVING_GROUND.get());
    t.raise(s);
    t
}

fn column() -> Solid {
    // Ten metres across, twenty tall, at (0, 0).
    Solid::round(m(0), m(0), m(0), m(20), m(5), m(5), Material::Wood)
}

#[test]
fn a_ray_meets_a_column_at_its_skin_not_its_box() {
    let c = column();
    // Straight at it along x: the skin at 5 m from the axis.
    let d = sim::math::ray_hits_solid(
        V3::new(m(-20), m(3), m(0)),
        V3::new(m(1), m(0), m(0)),
        &c,
        Fx::ZERO,
    )
    .expect("hits");
    assert!(
        near(d, m(15), Fx::ratio(1, 50)),
        "met at {}",
        d.to_f32_for_render()
    );
    // Along the diagonal past the box's corner but outside the circle: a miss.
    let k = Fx::ratio(7071, 10000);
    let from = V3::new(m(-20), m(3), m(-20).add(m(6).div(k)));
    let miss = sim::math::ray_hits_solid(from, V3::new(k, Fx::ZERO, k), &c, Fx::ZERO);
    assert!(miss.is_none(), "the box's corner is air: {miss:?}");
}

#[test]
fn a_ray_meets_a_big_round_solid_from_far_away() {
    // A dome's slab, ninety metres across, seen from two hundred metres off:
    // squares 16.16 cannot hold.
    let slab = Solid::round(m(0), m(0), m(300), m(305), m(30), m(45), Material::Stone);
    let d = sim::math::ray_hits_solid(
        V3::new(m(-250), m(302), m(0)),
        V3::new(m(1), m(0), m(0)),
        &slab,
        Fx::ZERO,
    )
    .expect("hits");
    // At y = 302 the radius is 30 + 15 * 2/5 = 36.
    assert!(
        near(d, m(214), Fx::ratio(1, 10)),
        "met at {}",
        d.to_f32_for_render()
    );
}

#[test]
fn a_ray_down_onto_a_tapering_trunk_meets_its_top() {
    let trunk = Solid::round(m(0), m(0), m(0), m(30), m(8), m(5), Material::Wood);
    let d = sim::math::ray_hits_solid(
        V3::new(m(1), m(50), m(1)),
        V3::new(m(0), m(-1), m(0)),
        &trunk,
        Fx::ZERO,
    )
    .expect("hits");
    assert!(near(d, m(20), Fx::ratio(1, 50)));
    // Down beside the top but over the wider foot: it meets the slanting side.
    let side = sim::math::ray_hits_solid(
        V3::new(m(7), m(50), m(0)),
        V3::new(m(0), m(-1), m(0)),
        &trunk,
        Fx::ZERO,
    )
    .expect("meets the side");
    // r(y) = 8 - 3 y / 30 = 7 at y = 10.
    assert!(
        near(side, m(40), Fx::ratio(1, 5)),
        "side at {}",
        side.to_f32_for_render()
    );
}

#[test]
fn a_box_shaped_solid_answers_as_a_box_always_did() {
    let b = Solid::cm([0, 0, 0], [400, 300, 200], Material::Stone);
    assert_eq!(b.shape, Shape::Box);
    let from = V3::new(m(-3), m(1), m(1));
    let dir = V3::new(m(1), m(0), m(0));
    assert_eq!(
        sim::math::ray_hits_solid(from, dir, &b, Fx::ZERO),
        sim::math::ray_hits_box(from, dir, b.min, b.max)
    );
}

#[test]
fn a_body_walked_into_a_column_stops_at_its_skin_and_slides_round() {
    let ground = ground_with(column());
    let r = sim::tuning::body_radius();
    // Walking along +x at its side, a little off the axis.
    let mut pos = V3::new(m(-8), Fx::ZERO, m(2));
    let mut vel = V3::new(m(7), Fx::ZERO, Fx::ZERO);
    let mut least = m(100);
    for _ in 0..120 {
        pos = pos.add(vel.scale(sim::DT));
        let out = ground.resolve_sized(pos, vel, true, r, sim::tuning::body_height());
        pos = out.pos;
        vel = out.vel;
        let d = sim::math::wide_flat_len(V3::new(pos.x, Fx::ZERO, pos.z));
        least = least.min(d);
        // Keep pushing on along x, as a held key does.
        vel.x = m(7);
    }
    assert!(
        least.raw() >= m(5).add(r).sub(Fx::ratio(1, 20)).raw(),
        "got {} from the axis, inside the skin",
        least.to_f32_for_render()
    );
    assert!(
        pos.x.raw() > m(5).raw(),
        "slid round it and on: x = {}",
        pos.x.to_f32_for_render()
    );
}

#[test]
fn a_body_stands_on_a_round_top_and_falls_past_its_edge() {
    let c = Solid::round(m(0), m(0), m(0), m(4), m(5), m(5), Material::Wood);
    let ground = ground_with(c);
    // Over the top, in the circle.
    assert_eq!(ground.ground_under(V3::new(m(3), m(10), m(3))), m(4));
    // Over the box's corner, outside the circle: the floor.
    assert_eq!(
        ground.ground_under(V3::new(Fx::ratio(46, 10), m(10), Fx::ratio(46, 10))),
        Fx::ZERO
    );
}

#[test]
fn a_turned_branch_is_where_it_points() {
    // Thirty metres long, two wide, pointing north-east (an eighth of a turn).
    let b = Solid::turned(
        m(0),
        m(0),
        m(9),
        m(10),
        m(15),
        m(1),
        Fx::ratio(1, 8),
        Material::Wood,
    );
    let ground = ground_with(b);
    // Along its length: on it.
    let along = V3::new(m(7), m(20), m(7));
    assert_eq!(ground.ground_under(along), m(10));
    // In its box but well off its line: air.
    let off = V3::new(m(7), m(20), m(-7));
    assert_eq!(ground.ground_under(off), Fx::ZERO);
    // A ray down onto its line meets its top.
    let d =
        sim::math::ray_hits_solid(along, V3::new(m(0), m(-1), m(0)), &b, Fx::ZERO).expect("hits");
    assert!(near(d, m(10), Fx::ratio(1, 50)));
    assert!(sim::math::ray_hits_solid(off, V3::new(m(0), m(-1), m(0)), &b, Fx::ZERO).is_none());
}

#[test]
fn a_body_runs_along_a_turned_branch_and_off_its_end() {
    let b = Solid::turned(
        m(0),
        m(0),
        m(9),
        m(10),
        m(15),
        m(2),
        Fx::ratio(1, 8),
        Material::Wood,
    );
    let ground = ground_with(b);
    let r = sim::tuning::body_radius();
    let h = sim::tuning::body_height();
    let k = Fx::ratio(7071, 10000);
    let mut pos = V3::new(m(-8).mul(k), m(10), m(-8).mul(k));
    let mut vel = V3::new(m(7).mul(k), Fx::ZERO, m(7).mul(k));
    let mut grounded = true;
    let mut on = 0;
    for _ in 0..300 {
        vel.y = vel.y.sub(m(20).mul(sim::DT));
        pos = pos.add(vel.scale(sim::DT));
        let out = ground.resolve_sized(pos, vel, grounded, r, h);
        pos = out.pos;
        vel = out.vel;
        grounded = out.grounded;
        if grounded && pos.y == m(10) {
            on += 1;
        }
        vel.x = m(7).mul(k);
        vel.z = m(7).mul(k);
    }
    assert!(on > 60, "stood on it for {on} frames");
    assert!(
        pos.y.raw() < m(5).raw(),
        "and fell off its far end: y = {}",
        pos.y.to_f32_for_render()
    );
}
