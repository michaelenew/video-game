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

/// Apex of a full hop above the feet, measured by running the simulation.
fn apex(class: Class) -> Fx {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut best = floor;
    for _ in 0..240 {
        w.advance([Input::new(Input::SPACE), Input::default()]);
        best = best.max(w.players[0].pos.y);
    }
    best.sub(floor)
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

/// Stand the fighter out of the way, by the wall, healed: a test about the
/// walk is not a test about the parasites.
fn park(w: &mut World) {
    w.players[0].pos = V3::new(m(140.0), Fx::ZERO, m(20.0));
    w.players[0].health = w.players[0].full_health();
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
            near(top(&b, part), want, 0.15),
            "{what} is at {:?}, not {want} m",
            top(&b, part)
        );
    }
    // The ankle reaches down to half a metre, so a level swing from a chest
    // at 1.2 m meets it; the belly is nine metres up, out of every hop.
    assert!(near(bottom(&b, ss::ankle_part(3)), 0.5, 0.1));
    assert!(near(bottom(&b, ss::PLASTRON), 8.6, 0.15));
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
        park(&mut w);
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

#[test]
fn every_ring_lands_on_the_beat_and_every_stamp_off_it() {
    // A fighter stands beside a leg and lets the creature do what it likes:
    // the ring comes out only in the dozen frames after a beat, and a stamp's
    // hit never inside the channel gap of one.
    let mut w = hunt();
    let gap = ss::Knob::ChannelGap.raw();
    let mut ring_frames = Vec::new();
    let mut stamp_frames = Vec::new();
    let mut since_beat = 999;
    for f in 0..6000i32 {
        let b = *w.monster().unwrap();
        // Stay four metres outside the mid left foot, healed.
        let foot = gait::foot_in_gait(&b, 2);
        w.players[0].pos = V3::new(foot.x, Fx::ZERO, foot.z.sub(m(4.0)));
        w.players[0].health = w.players[0].full_health();
        let before = b.stride;
        step(&mut w, Input::default());
        let b = *w.monster().unwrap();
        if gait::landed(before, b.stride).is_some() {
            since_beat = 0;
        } else {
            since_beat += 1;
        }
        if fight::ring_live(&w) {
            assert!(
                since_beat <= ss::Knob::RingFrames.raw(),
                "a ring out {since_beat} frames after the beat"
            );
            ring_frames.push(f);
        }
        let c = fight::leg::get(&b);
        if c.kind == fight::leg::STAMP && c.phase == fight::leg::ACTIVE {
            stamp_frames.push(f);
        }
    }
    assert!(!ring_frames.is_empty(), "no ring in a hundred seconds");
    assert!(
        !stamp_frames.is_empty(),
        "no stamp at a fighter beside its foot in a hundred seconds"
    );
    for s in &stamp_frames {
        let closest = ring_frames.iter().map(|r| (r - s).abs()).min().unwrap();
        assert!(closest > gap, "a stamp's hit {closest} frames from a ring");
    }
}

fn break_ankle(w: &mut World, leg: usize) {
    let b = w.monster_mut().unwrap();
    let health = b.part_health(ss::ankle_part(leg));
    b.take_hit(ss::ankle_part(leg), health);
}

#[test]
fn a_broken_ankle_buckles_for_a_third_of_what_broke_it() {
    let mut w = hunt();
    step(&mut w, Input::default());
    let b = w.monster_mut().unwrap();
    let ankle = ss::ankle_part(1);
    let whole = b.part_health(ankle);
    b.take_hit(ankle, whole);
    assert!(b.broken(ankle));
    assert_eq!(fight::buckles(b, 1), 0);
    let buckle = ss::Knob::BuckleHealth.raw();
    assert_eq!(buckle * 3, whole, "a buckle is not a third of a break");
    b.take_hit(ankle, buckle - 1);
    assert_eq!(fight::buckles(b, 1), 0);
    b.take_hit(ankle, 1);
    assert_eq!(fight::buckles(b, 1), 1);
}

#[test]
fn a_broken_ankle_limps_the_walk() {
    let mut w = hunt();
    for _ in 0..5 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let sound = w.monster().unwrap().speed;
    break_ankle(&mut w, 1);
    for _ in 0..5 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    let lame = w.monster().unwrap().speed;
    let want = sound.mul(ss::Knob::Limp.fx());
    assert!(
        lame.sub(want).abs().raw() < 64,
        "one broken ankle walks {lame:?}, not {want:?}"
    );
    // It is the creature's turn to the anchors that hurt, not its health.
    assert_eq!(
        w.monster().unwrap().health,
        w.monster().unwrap().sp().health()
    );
}

#[test]
fn two_broken_ankles_on_one_side_bring_the_rim_within_every_hop_of_the_stair() {
    let mut w = hunt();
    step(&mut w, Input::default());
    break_ankle(&mut w, 0);
    assert!(
        !matches!(w.monster().unwrap().doing, Doing::Stumble { .. }),
        "one ankle stumbled it"
    );
    break_ankle(&mut w, 2);
    let b = *w.monster().unwrap();
    assert!(
        matches!(b.doing, Doing::Stumble { .. }),
        "two ankles on a side did not stumble it"
    );
    assert_eq!(fight::stumble_side(&b), -1);
    for _ in 0..ss::Knob::Settle.raw() + 5 {
        step(&mut w, Input::default());
    }
    let b = *w.monster().unwrap();
    // The low rim is at six metres, the broken thighs a ramp from the floor
    // to it, its first tread inside the Bulwark's hop.
    assert!(
        near(top(&b, ss::rim_part(1, -1)), 6.4, 0.6),
        "the low rim is at {:?}",
        top(&b, ss::rim_part(1, -1))
    );
    let bulwark = apex(Class::Bulwark);
    let rig = b.rig();
    let sh = b.sp().shape(ss::thigh_part(0));
    let knee = rig
        .part_to_world(ss::thigh_part(0), V3::new(sh.max.x, sh.max.y, Fx::ZERO))
        .y;
    assert!(
        knee.raw() < bulwark.raw(),
        "the first tread {knee:?} is out of the Bulwark's hop {bulwark:?}"
    );
    // And the walk has stopped.
    let x = b.pos.x;
    step(&mut w, Input::default());
    assert_eq!(
        w.monster().unwrap().pos.x,
        x,
        "it walked on while stumbling"
    );
}

/// A fighter of `class` at the knee of a stumbling side's broken fore leg,
/// walking in up the stair and hopping whenever its feet are down: whether
/// they reach the rim before the stumble ends.
fn climbs_the_stair(class: Class) -> bool {
    let mut w = hunt_as(class);
    step(&mut w, Input::default());
    break_ankle(&mut w, 0);
    break_ankle(&mut w, 2);
    for _ in 0..ss::Knob::Settle.raw() {
        step(&mut w, Input::default());
    }
    let b = *w.monster().unwrap();
    let rig = b.rig();
    let sh = b.sp().shape(ss::thigh_part(0));
    let knee = rig.part_to_world(ss::thigh_part(0), V3::new(sh.max.x, sh.max.y, Fx::ZERO));
    // Three and a half metres out past the knee.
    w.players[0].pos = V3::new(knee.x, Fx::ZERO, knee.z.sub(m(3.5)));
    w.players[0].vel = V3::ZERO;
    // Looking in across the body: +z is the creature's right, from its left.
    let aim = (Fx::ratio(1, 4).raw() as u32 & 0xFFFF) as u16;
    for _ in 0..600 {
        let p = w.players[0];
        let on_rim = p.aboard() && ss::shell_side(sim::monster::mount_part(p.mount)) != 0;
        if on_rim {
            return true;
        }
        let mut bits = Input::W;
        if p.grounded || p.vel.y.raw() > 0 {
            bits |= Input::SPACE;
        }
        step(&mut w, Input::aimed(bits, aim));
        if !matches!(w.monster().unwrap().doing, Doing::Stumble { .. }) {
            break;
        }
    }
    false
}

#[test]
fn the_stumble_stair_is_climbable_by_the_bulwark() {
    assert!(
        climbs_the_stair(Class::Bulwark),
        "the Bulwark could not climb the stair"
    );
    assert!(
        climbs_the_stair(Class::Champion),
        "the Champion could not climb the stair"
    );
}

#[test]
#[ignore]
fn probe_walk() {
    let mut w = hunt();
    for f in 0..(60 * 330) {
        walk_only(&mut w);
        park(&mut w);
        step(&mut w, Input::default());
        if f % 1200 == 0 {
            let b = *w.monster().unwrap();
            eprintln!(
                "f{f} x {:?} doing {:?} speed {:?} hp {} siege {} p0 {:?} hp0 {}",
                b.pos.x,
                b.doing,
                b.speed,
                b.health,
                fight::at_siege_line(&b),
                w.players[0].pos,
                w.players[0].health
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The ride
// ---------------------------------------------------------------------------

/// Put the fighter on a part's top, the way landing on it would, at a point
/// in its own frame (its middle when `at` is `None`).
fn board(w: &mut World, part: usize, at: Option<V3>) {
    let beast = *w.monster().unwrap();
    let sh = beast.sp().shape(part);
    let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
    let spot = at.unwrap_or(V3::new(mid.x, sh.max.y, mid.z));
    w.players[0].pos = beast.world_of(part, V3::new(spot.x, sh.max.y.add(Fx::ratio(1, 8)), spot.z));
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[0].grounded = false;
    step(w, Input::default());
    assert!(
        w.players[0].aboard(),
        "fixture: did not land on {}",
        beast.sp().parts[part].name
    );
}

/// Is the rider still aboard after the creature throws `kind` (mirrored or
/// not), holding `input`?
fn rides_out(part: usize, kind: u8, mirror: bool, input: Input) -> bool {
    let mut w = hunt();
    for _ in 0..3 {
        walk_only(&mut w);
        step(&mut w, Input::default());
    }
    board(&mut w, part, None);
    let total = ss::SPECIES.attack(kind).total();
    {
        let b = w.monster_mut().unwrap();
        b.doing = Doing::Startup {
            kind,
            left: ss::SPECIES.attack(kind).startup,
        };
        b.brain.mirror = mirror;
        b.brain.think_left = u16::MAX;
    }
    for _ in 0..total {
        fight::leg::clear(w.monster_mut().unwrap());
        step(&mut w, input);
        if !w.players[0].aboard() {
            return false;
        }
    }
    true
}

#[test]
fn the_shrug_throws_a_loose_rider_off_its_side_and_a_braced_one_holds() {
    let brace = Input::new(Input::CROUCH);
    for part in [ss::rim_part(1, 1), ss::FLANK_LOWER_R] {
        assert!(
            !rides_out(part, ss::SHRUG, false, Input::default()),
            "a shrug left a loose rider on the {}",
            ss::PARTS[part].name
        );
        assert!(
            rides_out(part, ss::SHRUG, false, brace),
            "a braced rider was thrown off the {} by a shrug",
            ss::PARTS[part].name
        );
    }
    // The other side, and the middle, are calm.
    for part in [ss::rim_part(1, -1), ss::PLATEAU_MID, ss::CROWN_PART] {
        assert!(
            rides_out(part, ss::SHRUG, false, Input::default()),
            "a shrug threw a loose rider off the {}",
            ss::PARTS[part].name
        );
    }
    // Mirrored, it throws the left.
    assert!(!rides_out(
        ss::rim_part(1, -1),
        ss::SHRUG,
        true,
        Input::default()
    ));
}

#[test]
fn the_shiver_throws_even_a_braced_rider_off_the_crown_and_not_off_the_plateau() {
    let brace = Input::new(Input::CROUCH);
    assert!(
        !rides_out(ss::CROWN_PART, ss::SHIVER, false, brace),
        "a braced rider held through a shiver"
    );
    assert!(
        rides_out(ss::PLATEAU_FORE, ss::SHIVER, false, Input::default()),
        "a shiver threw a rider off the plateau"
    );
}

#[test]
fn the_walk_never_throws_a_rider() {
    for part in [
        ss::rim_part(0, -1),
        ss::FLANK_UPPER_R,
        ss::PLATEAU_AFT,
        ss::CROWN_PART,
    ] {
        let mut w = hunt();
        for _ in 0..3 {
            walk_only(&mut w);
            step(&mut w, Input::default());
        }
        board(&mut w, part, None);
        for f in 0..1200 {
            walk_only(&mut w);
            step(&mut w, Input::default());
            assert!(
                w.players[0].aboard(),
                "the walk threw a rider off the {} on frame {f}",
                ss::PARTS[part].name
            );
        }
    }
}

/// The hardest a point on a part's top accelerates, in the part's frame, the
/// way the grip test reads it, through a move.
fn peak_throw(kind: u8, part: usize) -> f32 {
    let mut b = Monster::new(SpeciesId::SIEGESHELL);
    let a = ss::SPECIES.attack(kind);
    let sh = b.sp().shape(part);
    let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
    let local = V3::new(mid.x, sh.max.y, mid.z);
    let mut at = Vec::new();
    let mut frames = Vec::new();
    let total = a.total() as i32;
    for f in 0..total {
        let (doing, _) = if f < a.startup as i32 {
            (
                Doing::Startup {
                    kind,
                    left: (a.startup as i32 - f) as u16,
                },
                0,
            )
        } else if f < (a.startup + a.active) as i32 {
            (
                Doing::Active {
                    kind,
                    left: ((a.startup + a.active) as i32 - f) as u16,
                },
                0,
            )
        } else {
            (
                Doing::Recovery {
                    kind,
                    left: (total - f) as u16,
                },
                0,
            )
        };
        b.doing = doing;
        let rig = b.rig();
        at.push(rig.part_to_world(part, local));
        frames.push(rig.of(part));
    }
    let dt = 1.0 / 60.0;
    let mut best = 0.0f32;
    for i in 1..at.len() - 1 {
        let acc = at[i + 1].sub(at[i].scale(Fx::from_int(2))).add(at[i - 1]);
        let l = frames[i].rot.unapply(acc);
        let (x, y, z) = (
            l.x.to_f32_for_render(),
            l.y.to_f32_for_render().max(0.0),
            l.z.to_f32_for_render(),
        );
        best = best.max((x * x + y * y + z * z).sqrt() / (dt * dt));
    }
    best
}

#[test]
#[ignore]
fn probe_throws() {
    for (kind, name) in [(ss::SHRUG, "shrug"), (ss::SHIVER, "shiver")] {
        for part in [
            ss::rim_part(1, 1),
            ss::FLANK_LOWER_R,
            ss::FLANK_UPPER_R,
            ss::PLATEAU_MID,
            ss::CROWN_PART,
            ss::rim_part(1, -1),
        ] {
            eprintln!(
                "{name} {:20} {:.0}",
                ss::PARTS[part].name,
                peak_throw(kind, part)
            );
        }
    }
}
