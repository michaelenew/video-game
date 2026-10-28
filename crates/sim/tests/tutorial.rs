//! **The tutorial halls** (`sim::tutorial`, `sim::arena::hall`): every class
//! on offer has one, built from its table; the dummy follows you room to
//! room and is never knocked out; the hall can be walked end to end; and a
//! task is ticked by doing it -- the real inputs, through the real step --
//! and said to have missed when it did not land.

use sim::arena::{self, ArenaId};
use sim::class::{MOVERS, ROTATION};
use sim::state::Phase;
use sim::tutorial::{self, Progress, Word};
use sim::{Class, Fx, Input, V3, World};

fn hall(class: Class) -> World {
    let t = tutorial::for_class(class).expect("a tutorial for every class on offer");
    World::versus_in([class], t.arena)
}

/// Stand fighter zero in room `k`, `back` metres in front of the dummy's
/// mark, facing it, and let the dummy follow.
fn stand_in(w: &mut World, k: usize, back: i32) {
    let t = tutorial::of(w.arena).unwrap();
    let mark = arena::hall::dummy_mark(t.rooms, k);
    let spot = V3::new(mark.at.x.sub(Fx::from_int(back)), Fx::ZERO, mark.at.z);
    let p = &mut w.players[0];
    p.pos = spot;
    p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    p.vel = V3::ZERO;
    for _ in 0..30 {
        w.advance([toward_dummy(w, 0); 2]);
    }
    // Back on the spot: a swinging dummy's count may have come round while
    // it took its place, and its thrust pushed her off it.
    let p = &mut w.players[0];
    p.pos = spot;
    p.vel = V3::ZERO;
    p.health = p.full_health();
}

fn yaw_to(from: V3, to: V3) -> u16 {
    let d = to.sub(from);
    (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16
}

/// These buttons, with the crosshair on the dummy's chest.
fn toward_dummy(w: &World, bits: u16) -> Input {
    let me = &w.players[0];
    let at = w.players[1].pos.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
    let aim = yaw_to(me.pos, at);
    let pitch = sim::aim::look_onto(me.pos, aim, me.aloft, me.clear, at);
    Input::looking_at(bits, aim, pitch)
}

/// Press these buttons for `held` frames, then let go and wait `after`,
/// feeding every frame to the tally; everything said, in order.
fn press(w: &mut World, coach: &mut Progress, bits: u16, held: u32, after: u32) -> Vec<Word> {
    let mut said = Vec::new();
    for f in 0..held + after {
        let input = toward_dummy(w, if f < held { bits } else { 0 });
        let before = w.clone();
        w.advance([input, Input::default()]);
        if let Some(word) = coach.observe(&before, w, 0) {
            said.push(word);
        }
    }
    said
}

#[test]
fn every_class_on_offer_has_a_hall_built_from_its_table() {
    for class in ROTATION {
        let t = tutorial::for_class(class).unwrap();
        assert_eq!(t.class, class);
        let a = t.arena.get();
        assert_eq!(a.id, t.arena, "{} is registered", class.name());
        assert!(tutorial::of(t.arena).is_some_and(|u| u.class == class));
        assert!(t.rooms.len() <= tutorial::MAX_ROOMS);
        assert!(!t.what.is_empty() && !t.plays.is_empty() && !t.wins.is_empty());
        // The premise, then a room for each click and one for the keys.
        let basics = t.rooms.iter().take_while(|r| !r.advanced).count();
        assert_eq!(
            basics,
            5,
            "{}: premise, three clicks, the keys",
            class.name()
        );
        assert!(
            t.rooms[basics..].iter().all(|r| r.advanced),
            "{}: the advanced rooms come last, all of them",
            class.name()
        );
        assert!(
            t.rooms.len() > basics,
            "{} has advanced rooms",
            class.name()
        );
        let first = &t.rooms[basics];
        assert!(
            first.intro.contains("advanced") && first.intro.contains("without"),
            "{}: the first advanced room says it can be played without them",
            class.name()
        );
        for r in t.rooms {
            assert!(
                !r.tasks.is_empty(),
                "{}: {} has a task",
                class.name(),
                r.title
            );
            assert!(r.tasks.len() <= tutorial::MAX_TASKS, "{}", r.title);
        }
        // The hall is as long as its rooms.
        let (_, far) = tutorial::span(t.rooms.len() - 1);
        assert!(a.bounds.hi_x.raw() > far.raw());
    }
    assert!(tutorial::for_class(Class::Bulwark).is_none());
}

#[test]
fn a_hall_opens_with_no_pick_and_never_ends_a_round() {
    for class in ROTATION {
        let mut w = hall(class).drafting(true);
        assert!(matches!(w.phase, Phase::Fighting), "no pick in a hall");
        stand_in(&mut w, 1, 2);
        assert!(w.is_dummy(1) && w.foes(0, 1), "the dummy can be hit");
        w.players[0].health = 1;
        w.players[1].health = 1;
        let mut coach = Progress::default();
        press(&mut w, &mut coach, Input::LEFT, 2, 60);
        assert!(matches!(w.phase, Phase::Fighting), "{}", class.name());
        assert!(w.players[0].health > 1);
        assert_eq!(w.players[1].health, w.players[1].full_health());
    }
}

#[test]
fn the_dummy_follows_you_into_every_room() {
    for class in ROTATION {
        let mut w = hall(class);
        let t = tutorial::of(w.arena).unwrap();
        for k in 0..t.rooms.len() {
            let (lo, hi) = tutorial::span(k);
            w.players[0].pos = V3::new(lo.add(Fx::ONE), Fx::ZERO, Fx::from_int(6));
            w.players[0].vel = V3::ZERO;
            w.advance([Input::default(); 2]);
            let d = w.players[1].pos;
            assert!(
                d.x.raw() >= lo.raw() && d.x.raw() < hi.raw(),
                "{}: the dummy is in room {k}",
                class.name()
            );
        }
    }
}

#[test]
fn every_mover_walks_the_hall_end_to_end() {
    for class in MOVERS {
        let mut w = hall(class);
        let t = tutorial::of(w.arena).unwrap();
        // Down the north lane, past every shelf, wall and gap.
        w.players[0].pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ratio(11, 4));
        let walk = Input::looking_at(Input::W, 0, 0);
        for _ in 0..60 * 120 {
            w.advance([walk, Input::default()]);
        }
        let last = t.rooms.len() - 1;
        assert_eq!(
            tutorial::room_at(t, w.players[0].pos.x),
            last,
            "{} walked to the last room: at {:?}",
            class.name(),
            w.players[0].pos
        );
    }
}

#[test]
fn a_jump_in_the_first_room_is_ticked() {
    for class in ROTATION {
        let mut w = hall(class);
        let mut coach = Progress::default();
        for _ in 0..10 {
            w.advance([Input::default(); 2]);
        }
        let said = press(&mut w, &mut coach, Input::SPACE, 2, 60);
        assert!(
            said.contains(&Word::Done {
                room: 0,
                task: 0,
                times: 1
            }),
            "{}: {said:?}",
            class.name()
        );
        // And again: the second time is counted.
        press(&mut w, &mut coach, 0, 0, 90);
        press(&mut w, &mut coach, Input::SPACE, 2, 90);
        assert_eq!(coach.times(class, 0, 0), 2, "{}", class.name());
    }
}

/// One move per class that strikes the dummy: the room, the task, the
/// buttons, how long they are held, and how far from the dummy to stand.
const LANDED: [(Class, usize, usize, u16, u32, i32); 5] = [
    (Class::Champion, 1, 0, Input::LEFT, 2, 2),
    (Class::ShadowReaver, 1, 0, Input::LEFT, 2, 4),
    (Class::Elementalist, 2, 0, Input::MIDDLE, 2, 6),
    (Class::BloodMage, 1, 0, Input::LEFT, 2, 6),
    (Class::DualMage, 1, 0, Input::LEFT, 2, 4),
];

#[test]
fn a_move_that_lands_is_ticked_and_one_that_misses_is_told() {
    for (class, room, task, bits, held, back) in LANDED {
        let mut w = hall(class);
        let mut coach = Progress::default();
        stand_in(&mut w, room, back);
        let said = press(&mut w, &mut coach, bits, held, 100);
        assert!(
            said.iter()
                .any(|s| matches!(s, Word::Done { room: r, task: k, .. } if *r as usize == room && *k as usize == task)),
            "{}: it landed: {said:?}",
            class.name()
        );
        assert_eq!(coach.times(class, room, task), 1);

        // Turned away from the dummy, the same move misses.
        let mut w = hall(class);
        let mut coach = Progress::default();
        stand_in(&mut w, room, back);
        let mut said = Vec::new();
        let away = Input::looking_at(0, 1 << 15, 0);
        for _ in 0..40 {
            let before = w.clone();
            w.advance([away, Input::default()]);
            coach.observe(&before, &w, 0);
        }
        for f in 0..held + 100 {
            let before = w.clone();
            let input = Input::looking_at(if f < held { bits } else { 0 }, 1 << 15, 0);
            w.advance([input, Input::default()]);
            if let Some(word) = coach.observe(&before, &w, 0) {
                said.push(word);
            }
        }
        assert!(
            said.iter()
                .any(|s| matches!(s, Word::Missed { room: r, task: k } if *r as usize == room && *k as usize == task)),
            "{}: it missed: {said:?}",
            class.name()
        );
        assert_eq!(coach.times(class, room, task), 0);
    }
}

#[test]
fn a_task_only_counts_in_its_own_room() {
    // The Champion's jump is a task of the first room only.
    let mut w = hall(Class::Champion);
    let mut coach = Progress::default();
    stand_in(&mut w, 1, 4);
    press(&mut w, &mut coach, Input::SPACE, 2, 60);
    assert_eq!(coach.times(Class::Champion, 0, 0), 0);
}

#[test]
fn a_hall_is_deterministic() {
    let run = || {
        let mut w = hall(Class::Elementalist);
        let mut rng = 0x5eed_u64;
        for _ in 0..600 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let input = Input::looking_at((rng & 0x0fff) as u16, (rng >> 16) as u16, 0);
            w.advance([input, Input::default()]);
        }
        w.checksum()
    };
    assert_eq!(run(), run());
}

#[test]
fn the_halls_sit_where_nothing_else_is() {
    for t in tutorial::ALL {
        assert!(arena::lookup(t.arena).is_some());
        assert!(sim::valley::ranges::index(t.arena).is_none());
        assert!(t.arena.0 < arena::COUNT as u8);
    }
    assert_ne!(ArenaId::HALL_CHAMPION, ArenaId::HENGE);
}

/// Where the crosshair is for a step of a script.
#[derive(Clone, Copy)]
enum Look {
    /// On the dummy's chest.
    Dummy,
    /// Straight down at her own feet.
    Down,
    /// Along the hall, `pitch` turns of 1/65536 above level.
    Ahead(i16),
}

/// A script: buttons, for this many frames, looking here.
type Script = &'static [(u16, u32, Look)];

/// Run a script in room `room`, `back` metres in front of the dummy, and
/// return how many times the task was ticked.
fn run(class: Class, room: usize, task: usize, back: i32, script: Script) -> u16 {
    let mut w = hall(class);
    let mut coach = Progress::default();
    stand_in(&mut w, room, back);
    for &(bits, frames, look) in script {
        for _ in 0..frames {
            let input = match look {
                Look::Dummy => toward_dummy(&w, bits),
                Look::Down => Input::looking_at(bits, 0, -(Input::QUARTER_TURN as i16) + 400),
                Look::Ahead(pitch) => Input::looking_at(bits, 0, pitch),
            };
            let before = w.clone();
            w.advance([input, Input::default()]);
            coach.observe(&before, &w, 0);
        }
    }
    coach.times(class, room, task)
}

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const SP: u16 = Input::SPACE;
const W: u16 = Input::W;
const SH: u16 = Input::SHIFT;

#[test]
fn the_champions_string_and_hold_are_ticked() {
    use Look::Dummy as D;
    // Three taps, each a beat after the last swing: the string.
    let string: Script = &[
        (L, 2, D),
        (0, 20, D),
        (L, 2, D),
        (0, 20, D),
        (L, 2, D),
        (0, 90, D),
    ];
    assert!(run(Class::Champion, 1, 1, 2, string) >= 1, "the second hit");
    assert!(run(Class::Champion, 1, 2, 2, string) >= 1, "the third hit");
    // Held: the Crescent, and not the tap.
    let hold: Script = &[(L, 30, D), (0, 90, D)];
    assert!(run(Class::Champion, 1, 3, 3, hold) >= 1, "the Crescent");
    assert_eq!(
        run(Class::Champion, 1, 0, 3, hold),
        0,
        "a hold is not a tap"
    );
    // The Rush.
    let rush: Script = &[(Input::MECHANIC, 2, D), (0, 40, D)];
    assert_eq!(run(Class::Champion, 4, 0, 6, rush), 1, "the Rush");
}

#[test]
fn the_reavers_shadow_send_and_dash_are_ticked() {
    use Look::Ahead;
    // Right click at the floor ahead: the shadow goes out.
    let send: Script = &[(R, 2, Ahead(-3000)), (0, 30, Ahead(-3000))];
    assert_eq!(run(Class::ShadowReaver, 3, 0, 8, send), 1, "the send");
    // Then shift toward it, crosshair on it: the dash.
    let dash: Script = &[
        (R, 2, Ahead(-3000)),
        (0, 30, Ahead(-3000)),
        (SH | W, 4, Ahead(-3000)),
        (0, 40, Ahead(-3000)),
    ];
    assert_eq!(run(Class::ShadowReaver, 4, 2, 8, dash), 1, "the dash");
}

#[test]
fn the_elementalists_structure_jump_is_ticked() {
    use Look::Down;
    // The rehearsal's timing: a stone at her feet, Space held from the
    // thirteenth frame after.
    let jump: Script = &[(L, 1, Down), (0, 12, Down), (SP, 60, Down), (0, 120, Down)];
    assert_eq!(
        run(Class::Elementalist, 5, 0, 6, jump),
        1,
        "the structure jump"
    );
    // A plain jump is not one.
    let plain: Script = &[(SP, 60, Down), (0, 120, Down)];
    assert_eq!(run(Class::Elementalist, 5, 0, 6, plain), 0);
    // And a raise on its own is the earth room's first task.
    let raise: Script = &[(L, 1, Down), (0, 30, Down)];
    assert_eq!(run(Class::Elementalist, 1, 0, 6, raise), 1, "a raise");
}

#[test]
fn the_dual_mages_hop_and_hold_are_ticked() {
    use Look::Dummy as D;
    let hop: Script = &[(L | W, 2, D), (W, 40, D)];
    assert_eq!(run(Class::DualMage, 1, 2, 6, hop), 1, "a hop");
    let hold: Script = &[(L, 20, D), (0, 90, D)];
    assert!(run(Class::DualMage, 1, 3, 6, hold) >= 1, "Chain lightning");
    assert_eq!(
        run(Class::DualMage, 1, 0, 6, hold),
        0,
        "a hold is not a tap"
    );
}

#[test]
fn the_blood_mages_holds_are_ticked() {
    use Look::Dummy as D;
    let bramble: Script = &[(L, 40, D), (0, 30, D)];
    assert_eq!(run(Class::BloodMage, 1, 1, 6, bramble), 1, "the Bramble");
    assert_eq!(
        run(Class::BloodMage, 1, 0, 6, bramble),
        0,
        "a hold is not a lash"
    );
    let sweep: Script = &[(R, 2, D), (0, 40, D)];
    assert!(run(Class::BloodMage, 3, 0, 2, sweep) >= 1, "a Sweep");
}

#[test]
fn every_hall_is_a_trip_away() {
    // The Esc menu's way in: the class first, then the trip -- from the
    // valley, where a run starts. An arena id past sixty-three used to be no
    // request at all, and the halls are past it.
    for t in tutorial::ALL {
        let mut w = World::arrive(
            [Class::Champion; sim::state::MAX_PLAYERS],
            sim::valley::Journey::default(),
            sim::valley::START,
            None,
            1,
        );
        let to = sim::input::Travel::arena(t.arena);
        assert_eq!(
            to.destination(),
            Some(sim::input::Destination::Arena(t.arena)),
            "{}",
            t.arena.get().name
        );
        w.advance([
            Input::default().travelling(sim::input::Travel::class(0, t.class)),
            Input::default(),
        ]);
        w.advance([Input::default().travelling(to), Input::default()]);
        assert_eq!(w.arena, t.arena, "{} is reached", t.arena.get().name);
        assert_eq!(w.players[0].class, t.class);
        assert!(w.is_present(1), "the dummy is in the hall");
    }
    // Every id a byte names travels and reads back as itself.
    for id in 0..=u8::MAX {
        let to = sim::input::Travel::arena(ArenaId(id));
        assert_eq!(
            to.destination(),
            Some(sim::input::Destination::Arena(ArenaId(id)))
        );
    }
}

/// The frames before the swinging dummy's thrust that a held middle click
/// may start on and clash it, from `gap` metres in the Champion's clash
/// room -- the dummy driven as the hall drives it.
fn clash_window(gap: i32) -> Vec<u32> {
    let t = tutorial::for_class(Class::Champion).unwrap();
    let room = t
        .rooms
        .iter()
        .position(|r| r.dummy == tutorial::Dummy::Swings)
        .expect("a clash room");
    let every = tutorial::SWING_EVERY;
    let mut ok = Vec::new();
    for early in 0..40 {
        let mut w = hall(Class::Champion);
        stand_in(&mut w, room, gap);
        let mut coach = Progress::default();
        let mut step = |w: &mut World, bits: u16| {
            let before = w.clone();
            // The dummy's thrust is the hall's: the step drives it.
            w.advance([toward_dummy(w, bits), Input::default()]);
            coach.observe(&before, w, 0);
        };
        // Up to `early` frames before the next thrust.
        while (w.frame + 1 + early) % every != 0 {
            step(&mut w, 0);
        }
        for f in 0..120 {
            step(&mut w, if f < 40 { Input::MIDDLE } else { 0 });
        }
        if coach.times(Class::Champion, room, 0) > 0 {
            ok.push(early);
        }
    }
    ok
}

#[test]
fn the_clash_room_can_be_timed_by_hand() {
    // Three or four metres off, a hold begun about a fifth of a second
    // before the thrust -- the panel's count reading 0.2 -- clashes it, on
    // more than one frame. With a sword up close it was one frame in a
    // hundred and forty: the hammer reached the dummy first.
    for gap in [3, 4] {
        let window = clash_window(gap);
        assert!(window.len() >= 4, "{gap} m: {window:?}");
        assert!(
            window.iter().any(|&f| (9..=15).contains(&f)),
            "{gap} m: the count's 0.2 is in it: {window:?}"
        );
    }
}

#[test]
fn breaking_through_a_stone_is_ticked() {
    // The mark a break leaves on her body is spent inside the step, so the
    // tick reads the stone gone and the broken ground on the dodge's first
    // frame. A stone raised right in front of her breaks; one raised too far
    // off for the dodge to reach is a plain dodge, and not ticked.
    let run = |pitch: i16| {
        let mut w = hall(Class::Elementalist);
        let mut coach = Progress::default();
        stand_in(&mut w, 4, 6);
        for (bits, n) in [(L, 1u32), (0, 40), (SH | W, 3), (0, 40)] {
            for _ in 0..n {
                let before = w.clone();
                w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
                coach.observe(&before, &w, 0);
            }
        }
        coach.times(Class::Elementalist, 4, 2)
    };
    assert_eq!(run(-12000), 1, "a stone close in front");
    assert_eq!(run(-4000), 0, "a stone out of the dodge's reach");
}
