//! **Slopes, steps and lips** (2026-10-13, from play): "any kind of slope or
//! tiny step has crazy and frustrating effects on movement". Each test is a
//! place on the valley's map where it happened, or one like it, and what a
//! body does there now. See `docs/design/feel-log.md` for the day, and
//! `arena::resolve_among` and `too_steep_to_climb` for the rules.
//!
//! The places are found on the map by their coordinates. The first three come
//! off the replay the report was made with (`versus-bulwark-163400f`, played on
//! build `86d35ac`): a snow cap on a rock in the Saddle at frame 160899, and the
//! bank under the Shrine with a seam at its top at frame 162677.

use sim::arena::ArenaId;
use sim::class::{Ghost, Mechanic};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

/// A world in the Saddle, with fighter one standing on the floor at a point
/// of the map (in centimetres) and fighter two well out of the way.
fn standing(class: Class, x: i32, z: i32) -> World {
    let mut w = World::arrive(
        [class, Class::Bulwark],
        sim::valley::Journey::default(),
        ArenaId::SADDLE,
        Some(0),
        1,
    );
    let o = w.map_origin();
    let (x, z) = (Fx::ratio(x, 100).sub(o.x), Fx::ratio(z, 100).sub(o.z));
    let ground = w.terrain();
    let y = ground.floor_below(V3::new(x, Fx::from_int(1000), z));
    w.players[0].pos = V3::new(x, y, z);
    w.players[0].vel = V3::ZERO;
    w.players[0].grounded = true;
    w.players[1].pos = w.players[0]
        .pos
        .add(V3::new(Fx::from_int(40), Fx::ZERO, Fx::ZERO));
    if let Mechanic::Shadow(_) = w.players[0].mechanic {
        w.players[0].mechanic = Mechanic::Shadow(sim::class::Shadow::attending(
            w.players[0].pos,
            w.players[0].facing,
        ));
    }
    w
}

/// Which way is up the land where fighter one stands, flat and of unit length.
fn uphill(w: &World) -> V3 {
    let p = w.players[0].pos;
    let ground = w.terrain();
    let e = Fx::ratio(1, 2);
    let gx = ground
        .relief_at(p.x.add(e), p.z)
        .sub(ground.relief_at(p.x.sub(e), p.z));
    let gz = ground
        .relief_at(p.x, p.z.add(e))
        .sub(ground.relief_at(p.x, p.z.sub(e)));
    V3::new(gx, Fx::ZERO, gz).normalized()
}

/// The mouse's turn that faces along `d`.
fn facing(d: V3) -> u16 {
    let (x, z) = (
        d.x.to_f32_for_render() as f64,
        d.z.to_f32_for_render() as f64,
    );
    ((z.atan2(x) / std::f64::consts::TAU).rem_euclid(1.0) * 65536.0) as u16
}

fn across(d: V3) -> V3 {
    V3::new(d.z.neg(), Fx::ZERO, d.x)
}

fn walk(w: &mut World, bits: u16, aim: u16) {
    w.advance([Input::looking_at(bits, aim, 0), Input::default()]);
}

fn walking() -> Fx {
    t::move_speed().mul(Fx::ratio(9, 10))
}

/// Where the land is as steep as `n` in a hundred across a metre, open of
/// anything standing on it for a few metres either way: found by
/// `slope_bench`, which is not kept.
const GENTLE: (i32, i32) = (88_700, 11_400);
const HILLSIDE: (i32, i32) = (113_100, -13_200);
const STEEP: (i32, i32) = (106_100, -5_000);
const MOUNTAIN: (i32, i32) = (62_100, -5_200);

#[test]
fn a_snow_cap_on_a_rock_is_walked_up() {
    // Frame 160893 of the replay: walking east along a rock's top towards a
    // cap of snow two hands high on it. She stopped dead against it.
    let mut w = standing(Class::Champion, 95_283, -2_825);
    let cap = w
        .terrain()
        .around(w.players[0].pos, Fx::from_int(4))
        .find(|s| {
            let rise = s.max.y.sub(w.players[0].pos.y);
            rise.raw() > 0 && rise.raw() <= t::step_over().raw()
        })
        .expect("the snow cap is still on the rock");
    for _ in 0..30 {
        walk(&mut w, Input::W, 0);
        assert!(
            w.players[0].vel.flat_len().raw() >= walking().raw(),
            "stopped at {:?}",
            w.players[0].pos
        );
    }
    let p = w.players[0].pos;
    assert!(p.x.raw() > cap.min.x.raw(), "never got on to it: {p:?}");
    assert!(
        p.y.raw() >= cap.max.y.raw() || p.x.raw() > cap.max.x.raw(),
        "walked through it rather than up it: {p:?}"
    );
}

#[test]
fn a_box_higher_than_a_step_is_still_a_wall() {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    // The lowest box on the proving ground that is taller than a step.
    let wall = *sim::arena::proving_ground::ARENA
        .solids()
        .iter()
        .filter(|s| s.min.y.raw() <= 0 && s.max.y.raw() > t::step_over().raw())
        .min_by_key(|s| s.max.y.raw())
        .expect("a box taller than a step");
    let mid_z = Fx::from_raw(wall.min.z.raw() / 2 + wall.max.z.raw() / 2);
    w.players[0].pos = V3::new(wall.min.x.sub(Fx::from_int(3)), Fx::ZERO, mid_z);
    w.players[1].pos = V3::new(wall.min.x.sub(Fx::from_int(30)), Fx::ZERO, mid_z);
    for _ in 0..90 {
        walk(&mut w, Input::W, 0);
    }
    let p = w.players[0].pos;
    assert!(p.y.raw() == 0, "walked up a box {:?} high", wall.max.y);
    assert!(p.x.raw() < wall.min.x.raw(), "walked through it: {p:?}");
}

#[test]
fn the_bank_under_the_shrine_is_walked_up_over_its_seam() {
    // Frames 162670-162800 of the replay: walking north up the bank under the
    // Shrine, which had a seam a hand high at its top. She flickered on and
    // off the floor up it, stopped dead at the seam, and then crept up a
    // tenth of a metre every fourth frame while sliding sideways. The
    // Shrine's path went on 2026-10-22, when it became a station; the same
    // walk is now up the foothills north of the Saddle's road beneath it.
    let mut w = standing(Class::Champion, 97_000, 2_000);
    let north = Input::QUARTER_TURN;
    let from = w.players[0].pos;
    for _ in 0..60 {
        walk(&mut w, Input::W, north);
    }
    let p = w.players[0].pos;
    assert!(
        p.z.sub(from.z).raw() > Fx::from_int(4).raw(),
        "got {:.2} m up the bank",
        p.z.sub(from.z).to_f32_for_render()
    );
    assert!(p.y.raw() > from.y.add(Fx::ratio(1, 2)).raw(), "{p:?}");
}

#[test]
fn walking_up_a_walkable_slope_keeps_her_speed_and_her_feet() {
    for spot in [GENTLE, HILLSIDE, STEEP] {
        let mut w = standing(Class::Champion, spot.0, spot.1);
        let aim = facing(uphill(&w));
        let from = w.players[0].pos;
        for f in 0..45 {
            walk(&mut w, Input::W, aim);
            let p = &w.players[0];
            assert!(p.grounded, "{spot:?}: off the floor {f} frames up");
            assert!(
                p.vel.flat_len().raw() >= walking().raw(),
                "{spot:?}: slowed to {:.2} m/s {f} frames up",
                p.vel.flat_len().to_f32_for_render()
            );
        }
        assert!(
            w.players[0].pos.y.raw() > from.y.raw(),
            "{spot:?}: never climbed"
        );
    }
}

#[test]
fn walking_and_dodging_down_a_walkable_slope_keeps_her_feet_on_it() {
    // A fixed eight centimetres of give was one frame of a walk down a slope
    // of 0.7: anything steeper, or faster, came down in hops.
    for spot in [GENTLE, HILLSIDE, STEEP] {
        for bits in [Input::W, Input::W | Input::SHIFT] {
            let mut w = standing(Class::Champion, spot.0, spot.1);
            let aim = facing(uphill(&w).scale(Fx::ONE.neg()));
            for f in 0..t::dodge_frames() as u32 {
                walk(&mut w, bits, aim);
                assert!(
                    w.players[0].grounded,
                    "{spot:?}: off the floor {f} frames down, holding {bits:x}"
                );
            }
        }
    }
}

/// Fighter one somewhere near `near` on land with no footing and nothing
/// standing on it: a mountainside, bare.
fn on_a_mountainside(near: (i32, i32)) -> World {
    for ring in 0..40 {
        for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            let (x, z) = (near.0 + dx * ring * 100, near.1 + dz * ring * 100);
            let w = standing(Class::Champion, x, z);
            let p = w.players[0].pos;
            let ground = w.terrain();
            let bare = ground.relief_at(p.x, p.z) == p.y
                && !ground
                    .around(p, Fx::from_int(3))
                    .any(|s| s.over(p.x, p.z, Fx::from_int(3)));
            let steep = |q: V3| ground.downhill(q.x, q.z).is_some();
            let up = uphill(&w);
            let all_steep = (-2..=2).all(|k| {
                let q = p.add(across(up).scale(Fx::from_int(k)));
                steep(q)
            });
            if bare && all_steep {
                return w;
            }
        }
    }
    panic!("no bare mountainside near {near:?}");
}

#[test]
fn ground_with_no_footing_is_crossed_but_never_climbed() {
    let mut w = on_a_mountainside(MOUNTAIN);
    let up = uphill(&w);
    // Half up it and half along it. Wherever she is on ground with no
    // footing she never goes up; she slides down it, and off it on to
    // ground with footing, which she walks up as she likes.
    let slant = up.add(across(up)).normalized();
    let from = w.players[0].pos;
    for f in 0..60 {
        let was = w.players[0].pos;
        let bare = w.terrain().downhill(was.x, was.z).is_some();
        walk(&mut w, Input::W, facing(slant));
        let rose = w.players[0].pos.y.sub(was.y);
        assert!(
            !bare || rose.raw() <= Fx::ratio(1, 100).raw(),
            "rose {:.3} m up ground with no footing, {f} frames in",
            rose.to_f32_for_render()
        );
    }
    let along = w.players[0].pos.sub(from);
    let crossed = along.x.mul(across(up).x).add(along.z.mul(across(up).z));
    assert!(
        crossed.raw() > Fx::from_int(2).raw(),
        "stuck: {:.2} m along the face in a second",
        crossed.to_f32_for_render()
    );
}

#[test]
fn a_seam_taller_than_a_step_is_a_ledge() {
    // The Mire's ledge meets its rim's bank either side, a metre and more
    // above it: a seam, not a slope, and too tall to step. Six metres out
    // from its door (2026-10-22; until then the Mireback's clearing meeting
    // the land east of its door, which the station's ledge replaced).
    let (mut w, rise) = (52_000..54_000)
        .step_by(25)
        .find_map(|x| {
            let w = standing(Class::Champion, x, -7_600);
            let ground = w.terrain();
            let p = w.players[0].pos;
            let past = ground.relief_at(p.x.add(Fx::ratio(3, 10)), p.z);
            let rise = past.sub(p.y);
            let open = ground.relief_at(p.x, p.z) == p.y;
            (open && rise.raw() > t::step_over().add(Fx::ratio(1, 4)).raw()).then_some((w, rise))
        })
        .expect("the Mire's ledge has no seam along its side");
    let from = w.players[0].pos;
    for _ in 0..30 {
        walk(&mut w, Input::W, 0);
    }
    assert!(
        w.players[0].pos.y.raw() <= from.y.add(Fx::ratio(1, 20)).raw(),
        "walked up a seam {:.2} m tall",
        rise.to_f32_for_render()
    );
}

/// The Reaver's dash jump thrown along `dir` from where she stands, the press
/// made as she closes on the shadow, as `reaver::the_dash_jump_launches_her`
/// throws it: the fastest she rises in the jump.
fn dash_jump(w: &mut World, dir: V3) -> Fx {
    let aim = facing(dir);
    for _ in 0..20 {
        walk(w, 0, aim);
    }
    let ground = w.terrain();
    let out = w.players[0].pos.add(dir.scale(Fx::from_int(6)));
    let out = V3::new(out.x, ground.relief_at(out.x, out.z), out.z);
    let Mechanic::Shadow(mut s) = w.players[0].mechanic else {
        panic!("not the Reaver")
    };
    s.pos = out;
    s.doing = Ghost::Waiting;
    w.players[0].mechanic = Mechanic::Shadow(s);
    let pitch = crosshair_onto(w, aim, out);
    let close = t::shadow_dash_speed().mul(sim::DT).mul(Fx::from_int(2));
    let mut pressing = false;
    let mut fastest = Fx::ZERO;
    for _ in 0..60 {
        let me = &w.players[0];
        let Mechanic::Shadow(s) = me.mechanic else {
            unreachable!()
        };
        pressing |= s.dash > 0 && s.pos.sub(me.pos).len().raw() <= close.raw();
        let bits = if pressing {
            Input::W | Input::SPACE
        } else {
            Input::SHIFT | Input::W
        };
        w.advance([Input::looking_at(bits, aim, pitch), Input::default()]);
        // Once the dash is over: the dash itself flies up a hill to a
        // shadow on it, and that is not a jump.
        let Mechanic::Shadow(s) = w.players[0].mechanic else {
            unreachable!()
        };
        if pressing && s.dash == 0 {
            // Only the dash jump's own flight: three quarters of the dash
            // since 2026-10-18 brings her down on an uphill inside the window,
            // and the jump still held then is an ordinary one.
            if w.players[0].grounded && fastest.raw() > 0 {
                break;
            }
            fastest = fastest.max(w.players[0].vel.y);
        }
    }
    assert!(pressing, "the dash never reached the shadow");
    fastest
}

#[test]
fn a_dash_jump_on_a_slope_takes_off_once() {
    // From play: "the shadow reaver's grounded dash to jump on any kind of
    // slope will rocket almost straight up". The ground rising under the
    // jump's first frame read as landing, and the held jump went again on
    // top of it: 39 m/s up, against 21 on the flat.
    let mut flat = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    flat.players[0].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    flat.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    let on_the_flat = dash_jump(&mut flat, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    for spot in [GENTLE, HILLSIDE] {
        let w = standing(Class::ShadowReaver, spot.0, spot.1);
        let up = uphill(&w);
        for dir in [up, up.scale(Fx::ONE.neg()), across(up)] {
            let mut w = w.clone();
            let rose = dash_jump(&mut w, dir);
            assert!(
                rose.raw() <= on_the_flat.raw(),
                "{spot:?} along {dir:?}: rose at {:.1} m/s, against {:.1} on the flat",
                rose.to_f32_for_render(),
                on_the_flat.to_f32_for_render()
            );
        }
    }
}

/// The pitch, looking along `aim`, that puts fighter one's crosshair on `at`.
/// Searched rather than solved, as `reaver.rs` does: the camera is `aim`'s.
fn crosshair_onto(w: &World, aim: u16, at: V3) -> i16 {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let ground = w.terrain();
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &ground,
    };
    for degrees in -89..=89 {
        let pitch = (degrees * 65536 / 360) as i16;
        if sim::aim::pointing_at(
            0,
            Input::looking_at(0, aim, pitch),
            at,
            t::shadow_lock_cone(),
            &scene,
        ) {
            return pitch;
        }
    }
    panic!("no pitch puts the crosshair on it");
}

#[test]
fn a_rush_keeps_to_a_slope_and_a_jump_out_of_it_takes_off_once() {
    // From play: "trying to walk, champion charge, jump, or jump attack up a
    // slope feels weird". The Rush runs at nearly three times a walk, so it
    // came down every slope in hops, and a jump thrown running up one met the
    // rising ground on its first frame and went twice.
    let takeoff = t::jump_speed().mul(Class::Champion.mobility().jump);
    for spot in [GENTLE, HILLSIDE, STEEP] {
        for sign in [Fx::ONE, Fx::ONE.neg()] {
            let mut w = standing(Class::Champion, spot.0, spot.1);
            let aim = facing(uphill(&w).scale(sign));
            let look = Input::looking_at(Input::MECHANIC, aim, down(15));
            w.advance([look, Input::default()]);
            let mut fastest = Fx::ZERO;
            for f in 0..12 {
                let bits = if f >= 6 { Input::SPACE } else { 0 };
                w.advance([Input::looking_at(bits, aim, down(15)), Input::default()]);
                let p = &w.players[0];
                if f < 6 {
                    assert!(
                        p.grounded,
                        "{spot:?} {}: the Rush left the floor {f} frames in",
                        if sign.raw() > 0 { "up" } else { "down" }
                    );
                }
                fastest = fastest.max(p.vel.y);
            }
            assert!(
                fastest.raw() <= takeoff.raw(),
                "{spot:?}: a jump out of a Rush rose at {:.1} m/s, against a takeoff of {:.1}",
                fastest.to_f32_for_render(),
                takeoff.to_f32_for_render()
            );
        }
    }
}

fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}
