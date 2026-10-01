//! The Bulwark's thrown shield against creatures and critters (`CLASS-5` in
//! `docs/design/review.md`, resolved): thrown, it strikes the first body it
//! meets and plants there empty; recalled, it strikes each it passes through
//! once and comes on home (`docs/design/kits/bulwark.md`, *Throw / Recall*).
//! Through the swing's own path -- `state::shield_hitbox`, which the overlay
//! draws -- so each creature's own rules decide what the blow does: its guard,
//! its parts, whether a part is there at all.

use sim::aim;
use sim::bulwark;
use sim::class::{Class, Mechanic, Shield};
use sim::critter::{Critter, is};
use sim::fixed::Fx;
use sim::monster::Doing;
use sim::species::mantis::{self, Knob as MantisKnob};
use sim::species::sandmaw::fight as maw;
use sim::species::{SpeciesId, gnats};
use sim::state::{MAX_PLAYERS, shield_hitbox};
use sim::tuning as t;
use sim::{Input, V3, World};

const EAST: u16 = 0;

fn throw(look: Input) -> Input {
    look.with(Input::MECHANIC)
}

fn shield(w: &World) -> Shield {
    w.players[0].shield().expect("a Bulwark carries a shield")
}

fn set_weight(w: &mut World, weight: Fx) {
    let Mechanic::Shield(s) = w.players[0].mechanic else {
        panic!("no shield");
    };
    w.players[0].mechanic = Mechanic::Shield(s.with_weight(weight));
}

/// A gnat hunt with the gnats cleared, a Bulwark at the critcheck lane
/// looking down +x, and a gnat held still at each of `at` metres down it.
fn gnats_at(at: &[i32]) -> World {
    let mut w = World::hunt_of([Class::Bulwark; MAX_PLAYERS], SpeciesId::GNATS);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    for (i, metres) in at.iter().enumerate() {
        let mut c = Critter::new(
            sp,
            gnats::GNAT,
            sim::critcheck::lane().add(V3::new(Fx::from_int(*metres), Fx::ZERO, Fx::ZERO)),
            1 << 15,
        );
        c.state = is::FLINCH;
        c.timer = u16::MAX;
        c.health = i16::MAX;
        w.critters[i] = c;
    }
    w.pack.as_mut().unwrap().grace = u16::MAX;
    w.players[0].pos = sim::critcheck::lane();
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(12));
    for _ in 0..2 {
        w.advance([Input::looking_at(0, EAST, 0), Input::default()]);
    }
    w
}

/// The crosshair on gnat `i`'s middle.
fn onto_gnat(w: &World, i: usize) -> Input {
    let middle = w.critters[i].body(w.critters.sp()).middle();
    let pitch = aim::look_onto_closely(w.players[0].pos, EAST, w.players[0].aloft, middle);
    Input::looking_at(0, EAST, pitch)
}

fn lost(w: &World, i: usize) -> i32 {
    i16::MAX as i32 - w.critters[i].health as i32
}

/// The shield planted `metres` down the lane, on the floor.
fn plant_at(w: &mut World, metres: i32, weight: Fx) {
    let at = sim::critcheck::lane().add(V3::new(Fx::from_int(metres), Fx::ZERO, Fx::ZERO));
    let floor = w.terrain().ground_under(at);
    w.players[0].mechanic = Mechanic::Shield(Shield::Planted {
        pos: V3::new(at.x, floor, at.z),
        weight,
    });
}

/// Run until the shield is no longer in flight, or `frames` pass.
fn until_landed(w: &mut World, look: Input, frames: u32) {
    for _ in 0..frames {
        w.advance([look, Input::default()]);
        if !matches!(shield(w), Shield::Flying { .. }) {
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Thrown
// ---------------------------------------------------------------------------

#[test]
fn the_thrown_shield_hits_a_critter_it_meets() {
    let mut w = gnats_at(&[5]);
    let look = onto_gnat(&w, 0);
    w.advance([throw(look), Input::default()]);
    until_landed(&mut w, look, 90);
    assert_eq!(
        lost(&w, 0),
        bulwark::throw_damage(Fx::ZERO),
        "the gnat it was thrown at took the wrong blow"
    );
    let Shield::Planted { pos, weight } = shield(&w) else {
        panic!("the shield did not plant: {:?}", shield(&w));
    };
    assert_eq!(weight, Fx::ZERO, "it struck and kept its weight");
    let gnat = w.critters[0].pos;
    assert!(
        sim::math::wide_flat_dist(pos, gnat).raw() < Fx::from_int(2).raw(),
        "it planted {} m from what it struck",
        sim::math::wide_flat_dist(pos, gnat).to_f32_for_render()
    );
}

#[test]
fn the_thrown_shield_stops_at_the_first_critter_it_meets() {
    let mut w = gnats_at(&[3, 6]);
    let look = onto_gnat(&w, 1);
    w.advance([throw(look), Input::default()]);
    until_landed(&mut w, look, 90);
    assert!(lost(&w, 0) > 0, "it passed through the near gnat");
    assert_eq!(lost(&w, 1), 0, "it went on through to the far one");
}

#[test]
fn a_loaded_throw_spends_its_weight_on_a_critter_and_knocks_it_down() {
    let blow = |weight: Fx| {
        let mut w = gnats_at(&[5]);
        w.critters[0].timer = 2;
        set_weight(&mut w, weight);
        let look = onto_gnat(&w, 0);
        w.advance([throw(look), Input::default()]);
        let carried = bulwark::weight(&w.players[0]);
        for _ in 0..90 {
            if lost(&w, 0) > 0 {
                break;
            }
            // Held in its flinch, briefly, until the shield arrives.
            w.critters[0].state = is::FLINCH;
            w.critters[0].timer = 2;
            w.advance([look, Input::default()]);
        }
        (lost(&w, 0), w.critters[0].timer, carried, shield(&w))
    };
    let (light, light_down, _, _) = blow(Fx::ZERO);
    let (heavy, heavy_down, carried, after) = blow(t::weight_cap());
    assert!(
        bulwark::knocks_down(carried),
        "the fixture's throw is not loaded"
    );
    // What it carried when it struck, less a few frames of drain in flight.
    assert!(
        heavy <= bulwark::throw_damage(carried) && heavy > bulwark::throw_damage(carried) - 20,
        "a throw carrying {carried:?} dealt {heavy}"
    );
    assert!(
        heavy > light,
        "a loaded throw hit no harder: {heavy} and {light}"
    );
    assert!(
        heavy_down + 1 >= t::knockdown_frames() && heavy_down > light_down,
        "a loaded throw left it down {heavy_down} frames, an empty one {light_down}"
    );
    assert!(
        matches!(after, Shield::Planted { weight, .. } if weight == Fx::ZERO),
        "a loaded throw that struck kept its weight: {after:?}"
    );
}

#[test]
fn the_thrown_shield_hits_a_creature_it_meets() {
    use sim::species::ridgeback;
    let mut w = World::hunt([Class::Bulwark; MAX_PLAYERS]);
    w.players[1].health = 0;
    let hold = |w: &mut World| {
        let m = w.monster_mut().unwrap();
        m.doing = Doing::Prowl;
        m.brain.think_left = u16::MAX;
        m.brain.grace = 0;
        m.speed = Fx::ZERO;
        m.yaw_rate = Fx::ZERO;
    };
    hold(&mut w);
    w.advance([Input::default(); MAX_PLAYERS]);
    // Six metres out from a foreleg, along the line from the middle of the
    // animal through it, looking straight at it.
    let m = w.monsters[0].unwrap();
    let rig = m.rig();
    let sh = ridgeback::SPECIES.shape(ridgeback::FORELEG_L);
    let leg = rig.part_to_world(
        ridgeback::FORELEG_L,
        sh.min.add(sh.max).scale(Fx::ratio(1, 2)),
    );
    let out = V3::new(leg.x.sub(m.pos.x), Fx::ZERO, leg.z.sub(m.pos.z)).normalized();
    let stand = V3::new(leg.x, Fx::ZERO, leg.z).add(out.scale(Fx::from_int(6)));
    let yaw = sim::pack::yaw_of(out.scale(Fx::ONE.neg()));
    w.players[0].pos = V3::new(stand.x, w.terrain().ground_under(stand), stand.z);
    w.players[0].vel = V3::ZERO;
    w.players[0].facing = out.scale(Fx::ONE.neg());
    let pitch = aim::look_onto_closely(w.players[0].pos, yaw, w.players[0].aloft, leg);
    let look = Input::looking_at(0, yaw, pitch);
    let before = w.monsters[0].unwrap().health;
    hold(&mut w);
    w.advance([throw(look), Input::default()]);
    for _ in 0..90 {
        hold(&mut w);
        w.advance([look, Input::default()]);
        if !matches!(shield(&w), Shield::Flying { .. }) {
            break;
        }
    }
    let after = w.monsters[0].unwrap().health;
    assert!(
        after < before,
        "the shield thrown at its leg did not hurt it"
    );
    assert!(
        matches!(shield(&w), Shield::Planted { weight, .. } if weight == Fx::ZERO),
        "the shield did not plant where it struck: {:?}",
        shield(&w)
    );
    let Shield::Planted { pos, .. } = shield(&w) else {
        unreachable!()
    };
    assert!(
        sim::math::wide_flat_dist(pos, leg).raw() < Fx::from_int(3).raw(),
        "it planted {} m from the leg",
        sim::math::wide_flat_dist(pos, leg).to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// Recalled
// ---------------------------------------------------------------------------

#[test]
fn the_recall_strikes_each_body_once_and_comes_home() {
    let mut w = gnats_at(&[3, 5]);
    plant_at(&mut w, 8, Fx::ZERO);
    let look = Input::looking_at(0, EAST, 0);
    w.advance([throw(look), Input::default()]);
    until_landed(&mut w, look, 120);
    assert!(
        matches!(shield(&w), Shield::Held { .. }),
        "the recall did not come home through them: {:?}",
        shield(&w)
    );
    for i in 0..2 {
        assert_eq!(
            lost(&w, i),
            t::shield_damage(),
            "gnat {i} was not struck exactly once by the recall"
        );
    }
}

#[test]
fn the_overlay_draws_the_volume_the_shield_is_tested_with() {
    // On every frame of a recall through a gnat, the gnat is struck exactly
    // when the volume the overlay draws touches its box.
    let mut w = gnats_at(&[4]);
    let sp = w.critters.sp();
    plant_at(&mut w, 8, Fx::ZERO);
    let look = Input::looking_at(0, EAST, 0);
    let mut first = true;
    let mut frames = 0;
    loop {
        let before = w.critters[0];
        w.advance([if first { throw(look) } else { look }, Input::default()]);
        first = false;
        let hurt = w.critters[0].health < before.health;
        let drawn = shield_hitbox(&w.players[0]).is_some_and(|hb| before.body(sp).touched_by(&hb));
        assert_eq!(
            hurt, drawn,
            "frame {frames}: the overlay and the hit test disagree"
        );
        frames += 1;
        if hurt {
            break;
        }
        assert!(frames < 120, "the recall never reached the gnat");
    }
}

// ---------------------------------------------------------------------------
// Each creature's own rules
// ---------------------------------------------------------------------------

#[test]
fn a_buried_sandmaw_is_not_struck_by_the_shield_and_a_standing_one_is() {
    let recall_over = |buried: bool| {
        let mut w = World::hunt_of([Class::Bulwark; MAX_PLAYERS], SpeciesId::SANDMAW);
        w.players[1].health = 0;
        for _ in 0..60 {
            w.advance([Input::default(); MAX_PLAYERS]);
        }
        let spot = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
        let place = |w: &mut World| {
            let m = w.monsters[0].as_mut().unwrap();
            m.pos = spot;
            m.yaw = Fx::ZERO;
            m.brain.grace = 0;
            m.brain.think_left = u16::MAX;
            m.doing = Doing::Prowl;
            m.speed = Fx::ZERO;
            if !buried {
                m.own[maw::body::POSTURE] = maw::posture::STANDING;
                m.own[maw::body::UP_FOR] = 0;
            }
        };
        place(&mut w);
        assert_eq!(
            maw::under(&w.monsters[0].unwrap()),
            buried,
            "the fixture's worm is in the wrong place"
        );
        w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
        w.players[0].pos.y = w.terrain().ground_under(w.players[0].pos);
        let far = V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO);
        w.players[0].mechanic = Mechanic::Shield(Shield::Planted {
            pos: V3::new(far.x, w.terrain().ground_under(far), far.z),
            weight: Fx::ZERO,
        });
        let before = w.monsters[0].unwrap().health;
        let look = Input::looking_at(0, EAST, 0);
        w.advance([throw(look), Input::default()]);
        for _ in 0..120 {
            place(&mut w);
            w.advance([look, Input::default()]);
            if !matches!(shield(&w), Shield::Flying { .. }) {
                break;
            }
        }
        before - w.monsters[0].unwrap().health
    };
    assert_eq!(
        recall_over(true),
        0,
        "the shield struck a worm under the sand"
    );
    assert!(
        recall_over(false) > 0,
        "the shield missed a worm standing up"
    );
}

#[test]
fn the_mantis_guard_turns_a_thrown_shield_and_an_open_one_takes_it() {
    let thrown_at = |guard: bool| {
        let base = V3::new(Fx::from_int(-4), Fx::ZERO, Fx::ZERO);
        let mut w = World::hunt_of([Class::Bulwark; MAX_PLAYERS], SpeciesId::MANTIS);
        w.players[1].health = 0;
        w.advance([Input::default(); MAX_PLAYERS]);
        let place = |w: &mut World| {
            let m = w.monsters[0].as_mut().unwrap();
            m.pos = base;
            m.yaw = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.speed = Fx::ZERO;
            m.brain.grace = 0;
            m.brain.think_left = 600;
            if guard {
                let a = mantis::SPECIES.attack(mantis::GUARD);
                m.doing = Doing::Active {
                    kind: mantis::GUARD,
                    left: a.active - MantisKnob::ParryWindow.raw() as u16 - 2,
                };
            } else {
                m.doing = Doing::Prowl;
            }
        };
        place(&mut w);
        let stand = base.add(V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO));
        w.players[0].pos = stand;
        w.players[0].vel = V3::ZERO;
        w.players[0].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
        let middle = base.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
        let pitch = aim::look_onto_closely(stand, 1 << 15, w.players[0].aloft, middle);
        let look = Input::looking_at(0, 1 << 15, pitch);
        let before = w.monsters[0].unwrap().health;
        w.advance([throw(look), Input::default()]);
        for _ in 0..60 {
            place(&mut w);
            w.advance([look, Input::default()]);
            if !matches!(shield(&w), Shield::Flying { .. }) {
                break;
            }
        }
        (before - w.monsters[0].unwrap().health, shield(&w))
    };
    let (open, _) = thrown_at(false);
    assert!(open > 0, "the shield thrown at an open Mantis missed it");
    let (guarded, after) = thrown_at(true);
    assert_eq!(guarded, 0, "the shield got through its guard");
    assert!(
        matches!(after, Shield::Planted { weight, .. } if weight == Fx::ZERO),
        "a shield its guard turned did not drop where it struck: {after:?}"
    );
}

#[test]
fn in_versus_the_thrown_shield_still_strikes_a_fighter() {
    // The versus rule is unchanged: a fighter in its path is struck, by the
    // test it always was.
    let mut w = World::with_classes([Class::Bulwark, Class::Champion]);
    w.players[0].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(2), Fx::ZERO, Fx::ZERO);
    let before = w.players[1].health;
    let look = Input::looking_at(0, EAST, 0);
    w.advance([throw(look), Input::default()]);
    until_landed(&mut w, look, 90);
    assert!(w.players[1].health < before, "the throw missed the fighter");
}
