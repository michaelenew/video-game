//! The Siegeshell's rules, pinned (`docs/design/creatures/siegeshell.md` §10).
//!
//! Sentences that state the rule; each one a thing the design says and the
//! harness or a person would feel if it stopped being true.

use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::siegeshell::{self as ss, fight, gait};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn m(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0) as i32)
}

/// The world height of a part's top face, at the middle of it.
fn top(beast: &Monster, part: usize) -> Fx {
    let sh = beast.sp().shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    beast
        .rig()
        .part_to_world(
            part,
            V3::new(mid(sh.min.x, sh.max.x), sh.max.y, mid(sh.min.z, sh.max.z)),
        )
        .y
}

/// The world height of a part's bottom face, at its lowest corner.
fn bottom(beast: &Monster, part: usize) -> Fx {
    let sh = beast.sp().shape(part);
    let rig = beast.rig();
    let mut lo = Fx::MAX;
    for x in [sh.min.x, sh.max.x] {
        for z in [sh.min.z, sh.max.z] {
            lo = lo.min(rig.part_to_world(part, V3::new(x, sh.min.y, z)).y);
        }
    }
    lo
}

fn hunt_as(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::SIEGESHELL);
    // The second fighter is out of it, as the harness has it alone.
    w.players[1].health = 0;
    w
}

fn hunt() -> World {
    hunt_as(Class::Champion)
}

fn step(w: &mut World, input: Input) {
    w.advance([input, Input::default()]);
}

/// Hold the body to its walk: no body move, and nothing on the legs'
/// channel, so a test about the walk or the beat is about that.
fn walk_only(w: &mut World) {
    let beast = w.monster_mut().expect("a hunt has a creature");
    if beast.doing.attacking().is_some() {
        beast.doing = Doing::Prowl;
    }
    beast.brain.think_left = u16::MAX;
    fight::leg::clear(beast);
}

fn near(a: Fx, b: f32, slack: f32) -> bool {
    (a.to_f32_for_render() - b).abs() <= slack
}

// ---------------------------------------------------------------------------
// The body
// ---------------------------------------------------------------------------

#[test]
fn it_stands_as_tall_as_its_document_says() {
    let mut w = hunt();
    step(&mut w, Input::default());
    let b = *w.monster().unwrap();
    let cases = [
        (ss::rim_part(1, -1), 14.0, "the rim"),
        (ss::FLANK_LOWER_L, 16.0, "the lower flank"),
        (ss::FLANK_UPPER_R, 18.0, "the upper flank"),
        (ss::PLATEAU_MID, 20.0, "the plateau"),
        (ss::CROWN_PART, 22.0, "the crown"),
        (ss::anchor_part(0), 24.5, "an anchor's top"),
        (ss::ankle_part(0), 3.5, "an ankle's top"),
    ];
    for (part, want, what) in cases {
        assert!(
            near(top(&b, part), want, 0.05),
            "{what} is at {:?}, not {want} m",
            top(&b, part)
        );
    }
    // The ankle reaches down to half a metre, so a level swing from a chest
    // at 1.2 m meets it; the belly is nine metres up, out of every hop.
    assert!(near(bottom(&b, ss::ankle_part(3)), 0.5, 0.05));
    assert!(near(bottom(&b, ss::PLASTRON), 8.6, 0.05));
    // Its feet are twenty-eight metres apart.
    let (l, r) = (gait::foot(&b, 2), gait::foot(&b, 3));
    assert!(near(sim::math::wide_flat_dist(l, r), 28.0, 0.3));
}

#[test]
fn it_walks_the_valley_and_halts_at_the_siege_line() {
    let mut w = hunt();
    let start = w.monster().unwrap().pos;
    // Five minutes of walking, untouched, and a little more.
    for _ in 0..(60 * 330) {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let b = *w.monster().unwrap();
    assert!(fight::at_siege_line(&b), "it never reached the siege line");
    let walked = b.pos.x.sub(start.x);
    assert!(
        walked.to_f32_for_render() > 200.0,
        "it walked only {walked:?} m in five and a half minutes"
    );
    assert!(
        near(fight::to_wall(&w, &b), 20.0, 0.5),
        "it halted {:?} from the wall",
        fight::to_wall(&w, &b)
    );
    // It kept to the middle of the valley.
    assert!(
        b.pos.z.abs().to_f32_for_render() < 2.0,
        "it wandered to z {:?}",
        b.pos.z
    );
}

// ---------------------------------------------------------------------------
// The beat
// ---------------------------------------------------------------------------

#[test]
fn a_foot_is_planted_while_the_body_walks_over_it() {
    let mut w = hunt();
    for _ in 0..30 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let b = *w.monster().unwrap();
    let planted: Vec<usize> = (0..ss::LEG_COUNT)
        .filter(|l| gait::swinging(&b, *l).is_none())
        .collect();
    assert!(planted.len() >= 3, "fewer than a tripod on the floor");
    let before: Vec<V3> = planted.iter().map(|l| gait::foot(&b, *l)).collect();
    for _ in 0..40 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let b = *w.monster().unwrap();
    for (l, was) in planted.iter().zip(before) {
        if gait::swinging(&b, *l).is_some() {
            continue;
        }
        let now = gait::foot(&b, *l);
        assert!(
            sim::math::wide_flat_dist(now, was).raw() < m(0.05).raw(),
            "a planted foot slid {:?} m",
            sim::math::wide_flat_dist(now, was)
        );
    }
}

#[test]
fn a_beat_comes_every_three_seconds_at_the_walk() {
    let mut w = hunt();
    let mut beats = Vec::new();
    for f in 0..1200 {
        walk_only(&mut w);
        let before = w.monster().unwrap().stride;
        step(&mut w, Input::default());
        if gait::landed(before, w.monster().unwrap().stride).is_some() {
            beats.push(f);
        }
    }
    assert!(
        beats.len() >= 5,
        "only {} beats in twenty seconds",
        beats.len()
    );
    eprintln!("{beats:?}");
    for pair in beats.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(
            (176..=184).contains(&gap),
            "a beat {gap} frames after the last"
        );
    }
}

/// A fighter stood beside a leg, jumping or not on the beat, for `beats`
/// beats: what they lost.
fn beside_a_leg(jump: bool, beats: usize) -> i32 {
    let mut w = hunt();
    for _ in 0..3 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let mut lost = 0;
    let mut seen = 0;
    let mut frames = 0;
    while seen < beats && frames < 4000 {
        frames += 1;
        walk_only(&mut w);
        let b = *w.monster().unwrap();
        // Four metres outside the mid left ankle, walking with it.
        let foot = gait::foot_in_gait(&b, 2);
        let spot = V3::new(foot.x, w.players[0].pos.y, foot.z.sub(m(4.0)));
        if w.players[0].grounded {
            w.players[0].pos = spot;
        }
        let to = gait::frames_to_beat(&b).unwrap_or(999);
        let input = if jump && to == 4 {
            Input::new(Input::SPACE)
        } else {
            Input::default()
        };
        let before = w.monster().unwrap().stride;
        let health = w.players[0].health;
        step(&mut w, input);
        lost += health - w.players[0].health;
        if gait::landed(before, w.monster().unwrap().stride).is_some() {
            seen += 1;
        }
        // Heal between beats, so a test about the ring is not a test about dying.
        w.players[0].health = w.players[0].full_health();
    }
    lost
}

#[test]
fn a_fighter_who_jumps_the_beat_takes_nothing() {
    let standing = beside_a_leg(false, 6);
    assert!(
        standing > 0,
        "the ring never caught a fighter standing four metres from a foot"
    );
    assert_eq!(
        beside_a_leg(true, 6),
        0,
        "a fighter who jumped every ring still took {}",
        0
    );
}
