//! **Being hunted.** The Ridgeback's threat, stated as relationships.
//!
//! Added 2026-09-25 with the pass that made it chase, track, stagger and root.
//! A report from play said it could be walked away from, walked out of, and
//! distracted, and that a fighter at range could backpedal and shoot it for
//! free. Each test here is one of those, turned into a property that has to
//! survive every future change to the numbers. See
//! `docs/design/monsters.md` §"Threat modes".

use sim::fixed::Fx;
use sim::monster::{self, Doing, Monster, Quarry};
use sim::state::{Action, MAX_PLAYERS};
use sim::{Class, Input, V3, World};

const PLUS_X: u16 = 0;
const PLUS_Z: u16 = 1 << 14;

fn quarry(pos: V3, vel: V3) -> Quarry {
    Quarry {
        pos,
        vel,
        alive: true,
        aboard: false,
        stunned: false,
    }
}

/// A hunt with one fighter, the creature at `at` facing +x and holding still,
/// and the fighter standing at `me`.
fn standoff(at: V3, me: V3) -> World {
    let mut w = World::hunt([Class::Champion; MAX_PLAYERS]);
    w.players[1].health = 0;
    let mut beast = Monster::new();
    beast.pos = at;
    beast.yaw = Fx::ZERO;
    beast.doing = Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    beast.brain.seen = me;
    w.monster = Some(beast);
    w.players[0].pos = me;
    w.players[0].grounded = true;
    w
}

/// Throw a move at the fighter and run it to the end, feeding the fighter's
/// inputs from `hand`, which is given the frame since the move began. Returns
/// whether the fighter lost health.
fn survive(mut w: World, kind: u8, hand: impl Fn(u32) -> Input) -> bool {
    let before = w.players[0].health;
    let m = monster::attack(kind);
    let beast = w.monster.as_mut().unwrap();
    beast.doing = Doing::Startup {
        kind,
        left: m.startup,
    };
    for f in 0..m.total() as u32 + 2 {
        w.monster.as_mut().unwrap().brain.think_left = u16::MAX;
        w.advance([hand(f), Input::default()]);
    }
    w.players[0].health == before
}

// ---------------------------------------------------------------------------
// You cannot outrun it
// ---------------------------------------------------------------------------

#[test]
fn nothing_a_fighter_does_on_foot_outruns_it() {
    // The complaint was that a fighter's walk beat the fastest thing it did.
    // Its gallop has to beat a walk by a margin, and its charge has to beat
    // the fastest thing a fighter has at all.
    let walk = sim::tuning::move_speed();
    let gallop = sim::tuning::gallop_speed();
    let charge = monster::attack(monster::CHARGE).advance;
    assert!(
        gallop.raw() > walk.mul(Fx::ratio(3, 2)).raw(),
        "it gallops at {gallop:?} and a fighter walks at {walk:?}"
    );
    assert!(
        charge.raw() > sim::tuning::dodge_speed().raw(),
        "its charge ({charge:?}) is slower than a dodge ({:?})",
        sim::tuning::dodge_speed()
    );
}

#[test]
fn a_fighter_backing_away_is_run_down() {
    // **It chases.** A fighter walking straight away from it at full speed,
    // from just outside its preferred distance, with every move it has locked
    // away so that this is pursuit and nothing else. It used to ask for a
    // stroll at seven metres and a walk at eight, so a backpedalling fighter
    // stayed just out of reach for as long as they liked -- and the gap grew.
    let mut beast = Monster::new();
    beast.pos = V3::new(Fx::from_int(-9), Fx::ZERO, Fx::ZERO);
    beast.brain.cooldown = [u16::MAX; monster::MOVES];
    let walk = sim::tuning::move_speed();
    let start = sim::tuning::prowl_range().add(Fx::ONE);
    let mut pos = beast.pos.add(V3::new(start, Fx::ZERO, Fx::ZERO));
    let vel = V3::new(walk, Fx::ZERO, Fx::ZERO);
    for _ in 0..90 {
        pos = pos.add(vel.scale(sim::DT));
        beast.step(&[quarry(pos, vel)]);
    }
    let gap = pos.sub(beast.pos).flat_len();
    assert!(
        gap.raw() <= start.raw(),
        "a fighter walking away from {start:?} m was {gap:?} m away a second and \
         a half later"
    );
}

#[test]
fn it_keeps_the_target_it_has() {
    // **It does not get distracted.** Nearest-wins turned it away from the
    // fighter it was trading with whenever somebody else drifted a metre
    // nearer -- in the game, the training dummy standing idle, which looked
    // like the animal losing interest and lumbering off.
    let mut beast = Monster::new();
    let mine = quarry(V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO), V3::ZERO);
    let nearer = quarry(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(5)), V3::ZERO);
    beast.brain.target = 0;
    beast.brain.think_left = u16::MAX;
    for _ in 0..60 {
        beast.step(&[mine, nearer]);
    }
    assert_eq!(
        beast.brain.target, 0,
        "it switched to somebody a metre nearer"
    );

    let much_nearer = quarry(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(2)), V3::ZERO);
    for _ in 0..60 {
        beast.step(&[mine, much_nearer]);
    }
    assert_eq!(
        beast.brain.target, 1,
        "it ignored somebody standing under its chin"
    );
}

// ---------------------------------------------------------------------------
// You cannot walk out of it -- and you can dodge it
// ---------------------------------------------------------------------------

/// Walk sideways the whole way through, starting on the first frame of the
/// windup.
fn walk_across(_: u32) -> Input {
    Input::aimed(Input::W, PLUS_Z)
}

/// A dodge sideways, pressed on frame `at` of the move.
fn dodge_at(at: u32) -> impl Fn(u32) -> Input {
    move |f| {
        if f == at {
            Input::aimed(Input::W | Input::SHIFT, PLUS_Z)
        } else {
            Input::aimed(0, PLUS_Z)
        }
    }
}

fn walk_does_not_dodge_does(kind: u8, range: Fx) {
    let at = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    let me = at.add(V3::new(range, Fx::ZERO, Fx::ZERO));
    let name = monster::MOVE_NAMES[kind as usize];
    assert!(
        !survive(standoff(at, me), kind, walk_across),
        "walking sideways out of the {name} from {range:?} m got clear of it"
    );
    let m = monster::attack(kind);
    let escaped = (0..m.startup as u32 + m.active as u32)
        .any(|f| survive(standoff(at, me), kind, dodge_at(f)));
    assert!(
        escaped,
        "no dodge, pressed on any frame of the {name}, got clear of it from {range:?} m"
    );
}

#[test]
fn the_bite_is_dodged_not_walked_out_of() {
    walk_does_not_dodge_does(monster::BITE, monster::attack(monster::BITE).ideal_range);
}

#[test]
fn the_charge_is_dodged_not_walked_out_of() {
    walk_does_not_dodge_does(monster::CHARGE, Fx::from_int(10));
}

#[test]
fn the_slam_is_dodged_not_walked_out_of() {
    walk_does_not_dodge_does(monster::SLAM, monster::attack(monster::SLAM).ideal_range);
}

// ---------------------------------------------------------------------------
// The long-range answer, and the set-ups
// ---------------------------------------------------------------------------

#[test]
fn the_spray_reaches_a_fighter_at_range_and_pins_them() {
    // **This is how it takes out somebody playing at range.** Fourteen metres
    // is past everything else it has but the charge, and the spray has to
    // arrive there and hold them where they are.
    let at = V3::new(Fx::from_int(-7), Fx::ZERO, Fx::ZERO);
    let me = V3::new(Fx::from_int(7), Fx::ZERO, Fx::ZERO);
    let mut w = standoff(at, me);
    let spray = monster::attack(monster::SPRAY);
    w.monster.as_mut().unwrap().doing = Doing::Startup {
        kind: monster::SPRAY,
        left: spray.startup,
    };
    let mut pinned_at = None;
    for _ in 0..spray.total() as u32 {
        w.monster.as_mut().unwrap().brain.think_left = u16::MAX;
        // Trying to leave, the whole time.
        w.advance([Input::aimed(Input::W, PLUS_X), Input::default()]);
        if pinned_at.is_none() && matches!(w.players[0].action, Action::Held { .. }) {
            pinned_at = Some(w.players[0].pos);
        }
    }
    let pinned = pinned_at.expect("the spray did not root a fighter fourteen metres away");
    let moved = w.players[0].pos.sub(pinned).flat_len();
    assert!(
        moved.raw() < Fx::ratio(1, 10).raw() || !matches!(w.players[0].action, Action::Held { .. }),
        "a rooted fighter walked {moved:?} m while held"
    );
}

#[test]
fn a_set_up_lasts_long_enough_for_what_follows_it() {
    // **The combos, as arithmetic.** A stagger that ends before the follow-up
    // can arrive is not a set-up, it is a shove. The sweep's stagger has to
    // outlast the rest of the sweep, the beat, and a bite; the spray's root
    // has to outlast the rest of the spray, the beat, and a charge crossing
    // the ground the spray is thrown across.
    let beat = sim::tuning::think_frames() as i32;
    let sweep = monster::attack(monster::SWEEP);
    let bite = monster::attack(monster::BITE);
    let after_sweep = sweep.active as i32 + sweep.recovery as i32 + beat + bite.startup as i32;
    assert!(
        sweep.hitstun as i32 >= after_sweep,
        "the sweep staggers for {} frames and a bite after it takes {after_sweep}",
        sweep.hitstun
    );
    let spray = monster::attack(monster::SPRAY);
    let charge = monster::attack(monster::CHARGE);
    let crossing = spray
        .ideal_range
        .sub(charge.hit_x)
        .sub(charge.hit_radius)
        .max(Fx::ZERO)
        .div(charge.advance.mul(sim::DT))
        .to_int();
    let after_spray =
        spray.active as i32 + spray.recovery as i32 + beat + charge.startup as i32 + crossing;
    assert!(
        spray.root as i32 >= after_spray,
        "the spray roots for {} frames and a charge after it takes {after_spray}",
        spray.root
    );
}

#[test]
fn a_staggered_target_is_pressed_and_otherwise_it_comes_about() {
    // After a move aimed behind it the creature takes a beat to turn round --
    // unless the target is staggered, in which case the beat would waste the
    // stagger.
    //
    // The pause is counted down from the frame it is set, so one frame of it
    // has already gone by the time the step returns.
    for (stunned, want) in [
        (false, sim::tuning::rear_pause() - 1),
        (true, sim::tuning::think_frames() - 1),
    ] {
        let mut beast = Monster::new();
        beast.doing = Doing::Recovery {
            kind: monster::SWEEP,
            left: 0,
        };
        beast.brain.seen_stunned = stunned;
        beast.brain.glance_left = u16::MAX;
        beast.step(&[]);
        assert_eq!(
            beast.brain.think_left,
            want,
            "after a sweep, with the target {}staggered",
            if stunned { "" } else { "not " }
        );
    }
}

#[test]
fn the_dead_are_not_carried() {
    // A fallen fighter the animal walks over stays on the floor, and one who
    // dies on its back comes off it. The creature gallops and charges now, and
    // it scooped up bodies often enough that the fight report counted them as
    // rides.
    let mut w = World::hunt([Class::Champion; MAX_PLAYERS]);
    let mut beast = w.monster.unwrap();
    beast.doing = Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    w.monster = Some(beast);
    w.players[1].health = 0;
    // Drop the body onto its back from just above it.
    let shape = monster::shape(monster::BARREL);
    let mid = shape.min.add(shape.max).scale(Fx::ratio(1, 2));
    w.players[1].pos = beast.world_of(
        monster::BARREL,
        V3::new(mid.x, shape.max.y.add(Fx::ratio(1, 8)), mid.z),
    );
    w.players[1].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[1].grounded = false;
    for _ in 0..30 {
        w.monster.as_mut().unwrap().brain.think_left = u16::MAX;
        w.advance([Input::default(); MAX_PLAYERS]);
        assert!(
            !w.players[1].aboard(),
            "a dead fighter landed on the creature and stayed"
        );
    }
    // And a rider who dies up there comes off.
    w.players[0].pos = w.players[1].pos;
    w.players[0].pos = beast.world_of(
        monster::BARREL,
        V3::new(mid.x, shape.max.y.add(Fx::ratio(1, 8)), mid.z),
    );
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[0].grounded = false;
    w.advance([Input::default(); MAX_PLAYERS]);
    assert!(
        w.players[0].aboard(),
        "fixture: the living rider did not land on it"
    );
    w.players[0].health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    assert!(
        !w.players[0].aboard(),
        "a rider killed on its back stayed on it"
    );
}
