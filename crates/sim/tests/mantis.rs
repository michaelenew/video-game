//! The Mantis: the rules `docs/design/creatures/mantis.md` pins, as sentences.
//!
//! Most of these set it up in a clear strip of the Shrine in front of one
//! fighter, let its eyes fill (it sees nothing for its first fifteen frames),
//! and then play the move the design names -- with the fighter's buttons where
//! the rule is about what a fighter does, and with a blow handed straight to
//! its guard where the rule is about the guard's arithmetic.

use sim::fixed::Fx;
use sim::monster::{Blow, Doing, Guarded, Monster};
use sim::species::SpeciesId;
use sim::species::mantis::{self, Knob, fight, habit, sight};
use sim::state::{MAX_PLAYERS, Player};
use sim::{Class, Input, V3, World};

/// Knobs are process-wide, and two of these tests set its eyes: every test
/// here holds this while it runs.
static KNOBS: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    KNOBS.lock().unwrap_or_else(|e| e.into_inner())
}

/// A hunt against the Mantis, the second fighter out of it.
fn hunt(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::MANTIS);
    w.players[1].health = 0;
    w
}

/// Where the duels are fought: west of the middle of the court, clear of the
/// columns.
fn base() -> V3 {
    V3::new(Fx::from_int(-4), Fx::ZERO, Fx::ZERO)
}

fn at(x: i32, z: i32) -> V3 {
    base().add(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z)))
}

/// Keep it where the test put it, thinking about nothing: no move of its own
/// choosing. Its reflexes -- the guard, the coil, the counter -- are the
/// frame hook's and still run.
fn hold_still(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.think_left = m.brain.think_left.max(60);
        if m.doing.free() {
            m.pos = base();
            m.yaw = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.speed = Fx::ZERO;
        }
    }
}

/// It at [`base`] facing +x, a fighter `dist` metres down +x facing it, its
/// prayer over and its eyes full.
fn duel(class: Class, dist: i32) -> World {
    let mut w = hunt(class);
    w.advance([idle(); MAX_PLAYERS]);
    {
        let m = w.monsters[0].as_mut().unwrap();
        m.doing = Doing::Prowl;
        m.brain.grace = 0;
    }
    place(&mut w, dist);
    for _ in 0..40 {
        w.advance([idle(); MAX_PLAYERS]);
        place(&mut w, dist);
        hold_still(&mut w);
    }
    w
}

fn place(w: &mut World, dist: i32) {
    w.players[0].pos = at(dist, 0);
    w.players[0].vel = V3::ZERO;
    w.players[0].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = base();
    m.yaw = Fx::ZERO;
}

/// Looking at it (down -x), no keys.
fn idle() -> Input {
    Input::aimed(0, 0x8000)
}

fn it(w: &World) -> Monster {
    w.monsters[0].unwrap()
}

fn me(w: &World) -> Player {
    w.players[0]
}

/// Set how late it sees, for a test about its eyes.
fn eyes(lo: i32, hi: i32) {
    let sp = &mantis::SPECIES;
    sim::oven::set_species_raw(
        sp.id,
        sim::species::Common::ALL.len() + Knob::SightMin as usize,
        lo,
    );
    sim::oven::set_species_raw(
        sp.id,
        sim::species::Common::ALL.len() + Knob::SightMax as usize,
        hi,
    );
}

/// Put its eyes back as baked.
fn eyes_as_baked() {
    let sp = &mantis::SPECIES;
    for k in [Knob::SightMin, Knob::SightMax] {
        let i = sim::species::Common::ALL.len() + k as usize;
        sim::oven::set_species_raw(sp.id, i, sp.tuned[i]);
    }
}

/// Raise its guard now, held past its parry.
fn guard_up(w: &mut World, parry: bool) {
    let a = mantis::SPECIES.attack(mantis::GUARD);
    let m = w.monsters[0].as_mut().unwrap();
    let held = if parry {
        0
    } else {
        Knob::ParryWindow.raw() as u16 + 2
    };
    m.doing = Doing::Active {
        kind: mantis::GUARD,
        left: a.active - held,
    };
}

/// A blow from `from`, by fighter zero's Champion sword.
fn sword_from(from: V3) -> Blow {
    Blow {
        from,
        unblockable: false,
        who: 0,
        class: Class::Champion,
        kind: 0,
    }
}

/// A Mantis standing at the origin facing +x, guard up past its parry.
fn guarding() -> Monster {
    let mut m = Monster::new(SpeciesId::MANTIS);
    let a = mantis::SPECIES.attack(mantis::GUARD);
    m.doing = Doing::Active {
        kind: mantis::GUARD,
        left: a.active - Knob::ParryWindow.raw() as u16 - 2,
    };
    m
}

fn v(x: i32, y: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::from_int(y), Fx::from_int(z))
}

// ---------------------------------------------------------------------------
// §4 · The guard
// ---------------------------------------------------------------------------

#[test]
fn a_blow_into_its_front_is_blocked_and_deals_nothing() {
    let _knobs = lock();
    let mut m = guarding();
    let health = m.health;
    let (dealt, g) = m.take_blow(mantis::THORAX_PART, 80, &sword_from(v(2, 0, 0)));
    assert_eq!(g, Guarded::Blocked);
    assert_eq!(dealt, 0, "no chip");
    assert_eq!(m.health, health);
    assert_eq!(
        m.part_health(mantis::BLADE_L_PART),
        mantis::SPECIES.part_health(),
        "no blade damage from a block"
    );
}

#[test]
fn a_hit_from_above_the_guard_or_behind_it_lands() {
    let _knobs = lock();
    // Behind it.
    let mut m = guarding();
    assert_eq!(
        m.take_blow(mantis::THORAX_PART, 80, &sword_from(v(-2, 0, 0)))
            .1,
        Guarded::Lands
    );
    // High above it: a fighter coming down from five metres up, a metre and
    // a half out.
    let mut m = guarding();
    let above = V3::new(Fx::ratio(3, 2), Fx::from_int(5), Fx::ZERO);
    assert_eq!(
        m.take_blow(mantis::HEAD, 80, &sword_from(above)).1,
        Guarded::Lands
    );
    // Right under it: a pillar at its feet.
    let mut m = guarding();
    let under = V3::new(Fx::ratio(1, 4), Fx::ZERO, Fx::ZERO);
    assert_eq!(
        m.take_blow(mantis::THORAX_PART, 80, &sword_from(under)).1,
        Guarded::Lands
    );
    // And a fighter at its own height in front, a hop off the floor, is
    // blocked.
    let mut m = guarding();
    let hop = V3::new(Fx::from_int(3), Fx::ONE, Fx::ZERO);
    assert_eq!(
        m.take_blow(mantis::THORAX_PART, 80, &sword_from(hop)).1,
        Guarded::Blocked
    );
}

#[test]
fn an_unblockable_into_a_raised_guard_breaks_it_and_into_an_open_one_only_hits() {
    let _knobs = lock();
    let breaker = Blow {
        unblockable: true,
        kind: 2,
        class: Class::Bulwark,
        ..sword_from(v(2, 0, 0))
    };
    let mut m = guarding();
    let (dealt, g) = m.take_blow(mantis::THORAX_PART, 80, &breaker);
    assert_eq!(g, Guarded::Broke);
    assert!(dealt > 0, "a break lands");
    assert!(
        matches!(m.doing, Doing::Recovery { kind: mantis::BROKEN, left } if left == Knob::BreakStagger.frames()),
        "broken, for {} frames: {:?}",
        Knob::BreakStagger.raw(),
        m.doing
    );
    // Its arms are worth more in the stagger.
    assert!(fight::hide(&m, mantis::BLADE_R_PART).raw() > Fx::ONE.raw());

    let mut open = Monster::new(SpeciesId::MANTIS);
    let (dealt, g) = open.take_blow(mantis::THORAX_PART, 80, &breaker);
    assert_eq!(g, Guarded::Lands);
    assert!(dealt > 0);
    assert!(!matches!(open.doing.attacking(), Some(mantis::BROKEN)));

    // In prayer, the break is longer.
    let mut praying = Monster::new(SpeciesId::MANTIS);
    praying.doing = Doing::Active {
        kind: mantis::PRAYER,
        left: 40,
    };
    assert_eq!(
        praying.take_blow(mantis::THORAX_PART, 80, &breaker).1,
        Guarded::Broke
    );
    assert!(
        matches!(praying.doing, Doing::Recovery { kind: mantis::BROKEN, left } if left == Knob::PrayerStagger.frames())
    );
}

#[test]
fn the_counter_follows_only_a_parry() {
    let _knobs = lock();
    // Inside the parry window: parried, and the counter is under way, thrown
    // back along the parried line.
    let mut m = Monster::new(SpeciesId::MANTIS);
    m.doing = Doing::Active {
        kind: mantis::GUARD,
        left: mantis::SPECIES.attack(mantis::GUARD).active,
    };
    let from = v(2, 0, 1);
    assert_eq!(
        m.take_blow(mantis::THORAX_PART, 80, &sword_from(from)).1,
        Guarded::Parried
    );
    assert!(matches!(
        m.doing,
        Doing::Startup {
            kind: mantis::COUNTER,
            ..
        }
    ));
    assert_eq!(m.yaw, fight::yaw_to(&m, from), "along the parried line");
    // A block: no counter.
    let mut m = guarding();
    m.take_blow(mantis::THORAX_PART, 80, &sword_from(v(2, 0, 0)));
    assert!(!matches!(m.doing.attacking(), Some(mantis::COUNTER)));
    // And the brain never chooses it.
    assert!(mantis::MOVES[mantis::COUNTER as usize].never_chosen);
}

#[test]
fn a_move_faster_than_its_eyes_is_blocked_not_parried() {
    let _knobs = lock();
    // The Champion's sword (startup 6) into a guard it has up: blocked.
    eyes(15, 15);
    let mut w = duel(Class::Champion, 2);
    guard_up(&mut w, false);
    let before = it(&w).health;
    let mut parried = false;
    for f in 0..30 {
        let press = if f < 2 {
            idle().with(Input::LEFT)
        } else {
            idle()
        };
        w.advance([press, Input::default()]);
        place(&mut w, 2);
        parried |= matches!(it(&w).doing.attacking(), Some(mantis::COUNTER));
    }
    assert!(!parried, "the sword was parried");
    assert_eq!(it(&w).health, before, "the sword got through its guard");

    // The hammer (startup 20) from an open stance: it sees the swing begin
    // fifteen frames late, raises its guard, and the hit arrives inside its
    // parry window -- parried, and countered.
    let mut w = duel(Class::Champion, 2);
    let mut parried = false;
    for f in 0..40 {
        let press = if f < 2 {
            idle().with(Input::MIDDLE)
        } else {
            idle()
        };
        w.advance([press, Input::default()]);
        place(&mut w, 2);
        parried |= matches!(it(&w).doing.attacking(), Some(mantis::COUNTER));
    }
    assert!(parried, "a hammer it saw coming was not parried");

    // With its eyes at their latest, the same hammer is not seen in time.
    eyes(21, 21);
    let mut w = duel(Class::Champion, 2);
    let mut parried = false;
    for f in 0..40 {
        let press = if f < 2 {
            idle().with(Input::MIDDLE)
        } else {
            idle()
        };
        w.advance([press, Input::default()]);
        place(&mut w, 2);
        parried |= matches!(it(&w).doing.attacking(), Some(mantis::COUNTER));
    }
    eyes_as_baked();
    assert!(!parried, "a hammer slower than its eyes was parried");
}

// ---------------------------------------------------------------------------
// §5 · Its eyes
// ---------------------------------------------------------------------------

#[test]
fn it_never_sees_the_present() {
    let _knobs = lock();
    // A fighter walking a wide circle round it, never still, so every frame's
    // position is its own. Whatever its brain holds as where it saw them is
    // a position they had at least `SightMin` frames before.
    let mut w = duel(Class::Champion, 6);
    let mut history: Vec<(u32, V3)> = Vec::new();
    let mut worst = u32::MAX;
    for f in 0..400u32 {
        let yaw = ((f * 300) & 0xFFFF) as u16;
        let walk = Input::aimed(0, yaw).with(Input::W);
        w.advance([walk, Input::default()]);
        hold_still(&mut w);
        history.push((w.frame, w.players[0].pos));
        let seen = it(&w).brain.seen;
        let cm = |p: V3| (sim::lore::to_cm(p.x), sim::lore::to_cm(p.z));
        if let Some((frame, _)) = history.iter().rev().find(|(_, p)| cm(*p) == cm(seen)) {
            worst = worst.min(w.frame - frame);
        }
    }
    assert!(worst < u32::MAX, "it never saw them at all");
    assert!(
        worst >= Knob::SightMin.raw() as u32,
        "it saw where they had been only {worst} frames before"
    );
}

#[test]
fn a_walk_is_not_a_commitment() {
    let _knobs = lock();
    assert!(!sight::Deed::Free.commits());
    assert!(!sight::Deed::Guard.commits());
    assert!(sight::Deed::Dodge.commits());
    assert!(sight::Deed::Aloft.commits());
    assert!(sight::Deed::Move(0).commits());
    // And a fighter walking about in front of a coil does not release it:
    // it runs to its end.
    let mut w = duel(Class::Champion, 7);
    coil(&mut w);
    let mut released_early = false;
    for f in 0..mantis::SPECIES.attack(mantis::LUNGE).startup as u32 {
        let dir = if (f / 20) % 2 == 0 {
            Input::A
        } else {
            Input::D
        };
        w.advance([idle().with(dir), Input::default()]);
        if let Doing::Startup {
            kind: mantis::LUNGE,
            left: 0,
        } = it(&w).doing
        {
            released_early |= fight::flags(&it(&w)) & fight::flag::ON_SIGHT != 0;
        }
    }
    assert!(!released_early, "a walk released the coil");
}

/// Start the coil on it now.
fn coil(w: &mut World) {
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind: mantis::LUNGE,
        left: mantis::SPECIES.attack(mantis::LUNGE).startup,
    };
    m.hit_used = false;
    m.brain.think_left = 600;
}

#[test]
fn the_coil_releases_on_a_commitment_it_saw_or_at_its_end() {
    let _knobs = lock();
    // A dodge, seen: released D frames after the dodge began, not before,
    // and never before its least coil.
    eyes(15, 15);
    let mut w = duel(Class::Champion, 7);
    coil(&mut w);
    let began = w.frame;
    let mut released = None;
    for f in 0..80u32 {
        let press = if (25..27).contains(&f) {
            idle().with(Input::SHIFT).with(Input::A)
        } else {
            idle()
        };
        w.advance([press, Input::default()]);
        if released.is_none()
            && matches!(
                it(&w).doing,
                Doing::Active {
                    kind: mantis::LUNGE,
                    ..
                }
            )
        {
            released = Some(w.frame - began);
        }
    }
    eyes_as_baked();
    let released = released.expect("the coil never released");
    assert!(
        released < mantis::SPECIES.attack(mantis::LUNGE).startup as u32,
        "it waited out its whole coil on a dodge it saw ({released})"
    );
    assert!(
        released >= 25 + 15,
        "it released {released} frames in, before it could have seen a dodge at 25"
    );
    // Nothing done: it runs to its end.
    let mut w = duel(Class::Champion, 7);
    coil(&mut w);
    let began = w.frame;
    let mut released = None;
    for _ in 0..80u32 {
        w.advance([idle(), Input::default()]);
        if released.is_none()
            && matches!(
                it(&w).doing,
                Doing::Active {
                    kind: mantis::LUNGE,
                    ..
                }
            )
        {
            released = Some(w.frame - began);
        }
    }
    assert_eq!(
        released,
        Some(mantis::SPECIES.attack(mantis::LUNGE).startup as u32 + 1),
        "a coil nobody committed in front of did not run to its end"
    );
}

#[test]
fn a_lunge_into_a_solid_stops_at_it() {
    let _knobs = lock();
    // Facing a column three metres off: the lane stops at it, the body stops
    // short of it, and it staggers.
    let mut w = hunt(Class::Champion);
    w.advance([idle(); MAX_PLAYERS]);
    let (cx, cz) = sim::arena::mantis::COLUMNS[0];
    let column = V3::new(
        sim::lore::from_cm(cx as i16),
        Fx::ZERO,
        sim::lore::from_cm(cz as i16),
    );
    let start = column.sub(V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO));
    {
        let m = w.monsters[0].as_mut().unwrap();
        m.doing = Doing::Prowl;
        m.pos = start;
        m.yaw = Fx::ZERO;
        m.brain.seen = column.add(V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO));
    }
    w.players[0].pos = column.add(V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO));
    coil(&mut w);
    let mut staggered = false;
    let mut furthest = Fx::ZERO;
    for _ in 0..120 {
        w.advance([idle(), Input::default()]);
        let m = it(&w);
        furthest = furthest.max(m.pos.x);
        staggered |= matches!(
            m.doing,
            Doing::Recovery {
                kind: mantis::STAGGER,
                ..
            }
        );
        if let Some(m) = w.monsters[0].as_mut() {
            m.brain.think_left = 60;
        }
    }
    assert!(staggered, "it ran into the column and did not stagger");
    let face = column.x.sub(Fx::ratio(6, 10));
    assert!(
        furthest.raw() <= face.raw(),
        "it went into the column: {} past its face at {}",
        furthest.to_f32_for_render(),
        face.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// §10 · Aiming
// ---------------------------------------------------------------------------

#[test]
fn a_skillshot_at_its_feet_meets_its_thorax() {
    let _knobs = lock();
    // A ground-aimed skillshot is raised to the middle of whatever stands
    // there -- a fighter's, 0.9 m -- and the line to it from a fighter
    // meets its thorax, which spans 0.7 to 2.4 m.
    let m = Monster::new(SpeciesId::MANTIS);
    let middle = sim::aim::standing_middle(m.pos, sim::tuning::body_height());
    for metres in [3, 6, 12] {
        let from = V3::new(Fx::from_int(metres), Fx::ONE, Fx::ZERO);
        let dir = middle.sub(from).normalized();
        let reach = middle.sub(from).len();
        let hit = m.part_struck_along(from, dir, reach.add(Fx::ONE), Fx::ratio(1, 10));
        assert_eq!(
            hit.map(|(p, _)| p),
            Some(mantis::THORAX_PART),
            "from {metres} m the shot met {:?}",
            hit.map(|(p, _)| mantis::PARTS[p].name)
        );
    }
    // And a level swing from a fighter's chest is at its thorax's height.
    let thorax = mantis::SPECIES.shape(mantis::THORAX_PART);
    let rig = m.rig();
    let low = rig.part_to_world(mantis::THORAX_PART, thorax.min).y;
    let high = rig.part_to_world(mantis::THORAX_PART, thorax.max).y;
    assert!(low.raw() < Fx::ratio(125, 100).raw() && high.raw() > Fx::ratio(125, 100).raw());
}

#[test]
fn the_guard_drawn_is_the_guard_it_has() {
    let _knobs = lock();
    // The fan drawn on the floor covers exactly what its guard covers: a
    // point under any strip is inside the cone; a point behind it is under
    // none.
    let mut w = duel(Class::Champion, 6);
    guard_up(&mut w, false);
    let signs = w.signs();
    let m = it(&w);
    let strips: Vec<_> = signs
        .iter()
        .filter(|s| s.says == sim::sign::Says::No)
        .collect();
    assert!(!strips.is_empty(), "its guard is up and nothing is drawn");
    for s in &strips {
        let tip = s.at.add(s.along.scale(s.length));
        assert!(m.covers(tip), "a strip is drawn where its guard is not");
    }
    let behind = m.pos.sub(V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO));
    assert!(!strips.iter().any(|s| s.covers(behind)));
    assert!(!m.covers(behind));
    let _ = habit::DEPTH;
    let _ = me(&w);
}

#[test]
fn earthbreaker_as_the_third_link_breaks_the_guard_its_first_two_raised() {
    // §7, the Champion's route: sword, sword into it -- it sees the first
    // link late and raises its guard, the second is blocked -- and
    // Earthbreaker, the hammer as the third link, lands inside its minimum
    // hold and breaks it.
    let _knobs = lock();
    let mut w = duel(Class::Champion, 2);
    let mut presses = [Input::LEFT, Input::LEFT, Input::MIDDLE].into_iter();
    let mut next = presses.next();
    let mut broke = false;
    let mut log = Vec::new();
    for _ in 0..140 {
        let free = me(&w).action.actionable();
        let press = match next {
            Some(b) if free => {
                next = presses.next();
                idle().with(b)
            }
            _ => idle(),
        };
        w.advance([press, Input::default()]);
        place(&mut w, 2);
        if let Some(m) = w.monsters[0].as_mut() {
            m.brain.think_left = m.brain.think_left.max(60);
        }
        log.push(format!(
            "{} {:?} {} | {:?} {:?} {}",
            w.frame,
            me(&w).action,
            me(&w).pos.x.sub(base().x).to_f32_for_render(),
            it(&w).doing,
            fight::mail(&it(&w), 0),
            it(&w).health
        ));
        broke |= matches!(
            it(&w).doing,
            Doing::Recovery {
                kind: mantis::BROKEN,
                ..
            }
        );
    }
    assert!(
        broke,
        "the string did not break its guard:\n{}",
        log.join("\n")
    );
}
