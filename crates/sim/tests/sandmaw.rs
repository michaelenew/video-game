//! The Sandmaw's rules: a body under the sand that has none, what it hears and
//! feels and nothing else, the rise-bite's marker over the noise, rock it
//! cannot pass under, the sinkhole, the spray, the tail, the swallow, the
//! beach and the tooth ring. Sentences that state the rule, as
//! `docs/design/creatures/sandmaw.md` §10 lists them; a failure here is a bug
//! or a design decision that changed without the document changing with it.

use sim::fixed::Fx;
use sim::hazard::{self, Hazard, HazardField};
use sim::monster::{Doing, Monster};
use sim::noise::{self, NoiseKind};
use sim::species::sandmaw::{self, Knob, fight};
use sim::species::{FightField, SpeciesId};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn sp() -> &'static sim::species::Species {
    &sandmaw::SPECIES
}

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn dist(a: V3, b: V3) -> Fx {
    sim::math::wide_flat_dist(a, b)
}

/// A hunt with one fighter in it.
fn hunt(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::SANDMAW);
    w.players[1].health = 0;
    w
}

/// Past the moment it takes to notice anyone.
fn awake(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.grace = 0;
    }
}

fn worm(w: &World) -> Monster {
    w.monsters[0].expect("a hunt has a worm")
}

/// The worm standing out of the sand at a point, between moves, taking no
/// notice of anything.
fn stand_at(w: &mut World, x: i32, z: i32, yaw: Fx) {
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(x, z);
    m.yaw = yaw;
    m.brain.grace = 0;
    m.brain.think_left = u16::MAX;
    m.doing = Doing::Prowl;
    m.own[fight::body::POSTURE] = fight::posture::STANDING;
    m.own[fight::body::UP_FOR] = 0;
}

fn start(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.grace = 0;
    m.doing = Doing::Startup {
        kind,
        left: sp().attack(kind).startup,
    };
    m.hit_used = false;
}

/// Frames with the worm playing out what it is doing, starting nothing new.
fn play(w: &mut World, frames: u32, input: Input) {
    for _ in 0..frames {
        if let Some(m) = w.monsters[0].as_mut() {
            if m.doing.free() {
                m.brain.think_left = u16::MAX;
            }
        }
        w.advance([input, Input::default()]);
    }
}

/// Frames with the worm free to do as it likes.
fn live(w: &mut World, frames: u32, input: Input) {
    for _ in 0..frames {
        w.advance([input, Input::default()]);
    }
}

/// Facing east, along +x.
const EAST: u16 = 0;
/// Facing north, along +z.
const NORTH: u16 = 1 << 14;

// ---------------------------------------------------------------------------
// A body that is not there
// ---------------------------------------------------------------------------

#[test]
fn a_buried_sandmaw_has_no_hurtbox() {
    let mut w = hunt(Class::Champion);
    live(&mut w, 60, Input::default());
    let m = worm(&w);
    assert!(fight::under(&m), "it opens under the sand");
    let rig = m.rig();
    for part in 0..sp().parts.len() {
        assert!(
            !rig.there(part),
            "{} is out of the sand while it swims",
            sp().parts[part].name
        );
        assert_eq!(w.shown(0, part), Fx::ZERO, "and is drawn");
    }
    // Nothing swung at the wake reaches anything.
    let over = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
    assert!(
        m.part_struck(over, Fx::from_int(4), Fx::from_int(3))
            .is_none()
    );
    assert!(
        m.part_struck_along(
            V3::new(m.pos.x, Fx::from_int(5), m.pos.z),
            V3::new(Fx::ZERO, Fx::ONE.neg(), Fx::ZERO),
            Fx::from_int(20),
            Fx::ONE,
        )
        .is_none()
    );
    let walked = m.resolve(over, Fx::ratio(1, 2), Fx::ratio(18, 10));
    assert_eq!(walked.pos, over, "and nothing stops a body walking over it");

    // **Every frame of the tell of a move from below is under the sand too**:
    // the warning is the floor's, and the body comes up with the hit.
    for kind in [sandmaw::RISE, sandmaw::BREACH, sandmaw::UNDERTOW] {
        let a = sp().attack(kind);
        for left in 0..=a.startup {
            let mut m = Monster::new(SpeciesId::SANDMAW);
            m.doing = Doing::Startup { kind, left };
            let rig = m.rig();
            let out: Vec<&str> = (0..sp().parts.len())
                .filter(|p| rig.there(*p))
                .map(|p| sp().parts[p].name)
                .collect();
            assert!(
                out.is_empty(),
                "{} {left} frames before its hit has {out:?} out of the sand",
                a.name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// What it perceives
// ---------------------------------------------------------------------------

#[test]
fn a_crouch_walk_makes_no_noise_and_a_walk_does() {
    let mut w = hunt(Class::Champion);
    w.monsters[0].as_mut().unwrap().pos = at(14, 14);
    let footfalls = |w: &World| {
        noise::all(&w.lore)
            .filter(|n| n.who == 0 && n.kind == NoiseKind::Footfall)
            .count()
    };
    live(&mut w, 120, Input::aimed(Input::W | Input::CROUCH, NORTH));
    assert_eq!(footfalls(&w), 0, "a crouch-walk is silent");
    live(&mut w, 60, Input::aimed(Input::W, NORTH));
    assert!(footfalls(&w) > 0, "a walk is not");
}

#[test]
fn a_fighter_standing_still_outside_the_feel_radius_is_never_perceived() {
    // Standing still in a corner of the Pan for half a minute. The worm may
    // come looking -- its search is what makes standing still a time and not
    // a place -- but it can only ever have taken them from its feel.
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(-15, -15);
    let feel = sp().fight_fx(FightField::FeelRadius);
    for _ in 0..1800 {
        w.advance([Input::default(); MAX_PLAYERS]);
        let m = worm(&w);
        if let Some(a) = fight::attended(&w.lore) {
            if a.frame == w.frame && a.who == 0 {
                assert!(a.felt(), "it heard a fighter who made no noise");
                let head = fight::head_flat(&m);
                assert!(
                    dist(head, w.players[0].pos).raw() <= feel.raw(),
                    "it felt a fighter {:?} from its head",
                    dist(head, w.players[0].pos)
                );
            }
        }
        assert!(
            noise::all(&w.lore).all(|n| n.who != 0),
            "standing still made a noise"
        );
    }
}

/// A run of everything a fighter does: walking, stopping, jumping, dodging,
/// crouch-walking, swinging into the sand, by frame.
fn busy(f: u32) -> Input {
    let aim = ((f / 240) as u16).wrapping_mul(0x3100);
    match f % 240 {
        0..=59 => Input::aimed(Input::W, aim),
        60..=89 => Input::aimed(0, aim),
        90..=95 => Input::aimed(Input::SPACE, aim),
        96..=139 => Input::aimed(Input::W | Input::CROUCH, aim),
        140..=143 => Input::aimed(Input::SHIFT | Input::D, aim),
        144..=179 => Input::aimed(0, aim),
        180..=183 => Input::aimed(Input::LEFT, aim),
        _ => Input::aimed(Input::S, aim),
    }
}

#[test]
fn it_never_takes_a_position_from_a_body_it_did_not_hear_or_feel() {
    for class in [Class::Champion, Class::DualMage, Class::ShadowReaver] {
        let mut w = hunt(class);
        let mut seen = worm(&w).brain.seen;
        for f in 0..4000 {
            let before = w.clone();
            w.advance([busy(f), Input::default()]);
            for p in w.players.iter_mut() {
                p.health = p.full_health();
            }
            let m = worm(&w);
            let now = m.brain.seen;
            if now == seen {
                continue;
            }
            seen = now;
            // A new sample: where a noise in the ring was made, or where a
            // body it feels is standing -- as the frame began, when it glanced.
            let heard = noise::all(&before.lore).any(|n| n.pos() == now);
            let ground = before.terrain();
            let was = before.monster().copied().unwrap();
            let felt = before.players.iter().any(|p| {
                p.pos == now && fight::feels(&was, fight::head_flat(&was), p.pos, &ground)
            });
            assert!(
                heard || felt,
                "{class:?}, frame {f}: it took {now:?}, which nobody made a noise at \
                 and nobody it feels stands on"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The rise-bite
// ---------------------------------------------------------------------------

/// Hunts in which the rise comes up again and again, round the islands as
/// well as in the open: a fighter landing jumps all over the Pan.
fn rises(seed: u32, mut each: impl FnMut(&World)) {
    let mut w = hunt(Class::Champion);
    w.monsters[0].as_mut().unwrap().brain.rng = seed | 1;
    let spots = [
        (-4, -4),
        (-1, -7),
        (5, 0),
        (-4, 4),
        (8, 3),
        (-7, 7),
        (0, 0),
        (10, -3),
        (-12, 0),
        (5, 12),
    ];
    for f in 0..6000u32 {
        let (x, z) = spots[((f / 180) as usize + seed as usize) % spots.len()];
        let p = w.players[0];
        let to = at(x, z).sub(flat(p.pos));
        let aim = sim::math::atan2_turns(to.z, to.x);
        let bits = if to.flat_len().raw() > Fx::ONE.raw() {
            Input::W
        } else if f % 50 < 4 {
            Input::SPACE
        } else {
            0
        };
        w.advance([
            Input::aimed(bits, (aim.raw() & 0xFFFF) as u16),
            Input::default(),
        ]);
        for p in w.players.iter_mut() {
            p.health = p.full_health();
        }
        each(&w);
    }
}

#[test]
fn the_rise_bite_marker_always_covers_the_noise_it_acted_on() {
    let mut checked = 0;
    for seed in 0..4 {
        rises(seed, |w| {
            let m = worm(w);
            let Doing::Startup { kind, left } = m.doing else {
                return;
            };
            if kind != sandmaw::RISE || left != sp().attack(sandmaw::RISE).startup - 1 {
                return;
            }
            let acted = fight::acted_on(&w.lore).expect("a rise acts on something");
            let t = m
                .telegraph()
                .expect("the marker is drawn from the first frame");
            // The island's edge is the one rise not at a noise: it comes up
            // beside a fighter on rock to spit, and bites nobody there.
            if acted.on_rock {
                return;
            }
            assert!(
                dist(t.anchor, acted.at).raw() <= t.radius.raw(),
                "the marker at {:?}, {:?} across, missed what it acted on at {:?}",
                t.anchor,
                t.radius,
                acted.at
            );
            checked += 1;
        });
    }
    assert!(checked > 10, "only {checked} rises to check");
}

/// Make one loud landing at a point, then do `after` from the next frame.
fn land_then(w: &mut World, after: Input, frames: u32) -> i32 {
    let before = w.players[0].health;
    for _ in 0..30 {
        w.advance([Input::new(Input::SPACE), Input::default()]);
        if w.players[0].grounded && w.players[0].since_landed > 0 {
            break;
        }
    }
    play_free(w, frames, after);
    before - w.players[0].health
}

fn play_free(w: &mut World, frames: u32, input: Input) {
    for _ in 0..frames {
        w.advance([input, Input::default()]);
    }
}

/// The worm under the sand nine metres from the fighter, awake, and with
/// nothing on its mind.
fn waiting() -> World {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(-8, -12);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(1, -12);
    awake(&mut w);
    live(&mut w, 2, Input::default());
    w
}

#[test]
fn a_fighter_who_walks_off_at_once_clears_the_rise_bite() {
    let mut w = waiting();
    let rose = std::cell::Cell::new(false);
    let lost = land_then(&mut w, Input::aimed(Input::W, NORTH), 1);
    assert_eq!(lost, 0);
    for _ in 0..90 {
        w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
        let m = worm(&w);
        if matches!(
            m.doing,
            Doing::Active {
                kind: sandmaw::RISE,
                ..
            }
        ) {
            rose.set(true);
        }
    }
    assert!(rose.get(), "the landing did not bring it up");
    assert_eq!(
        w.players[0].health,
        w.players[0].full_health(),
        "walking off at once was not enough"
    );
}

#[test]
fn a_spit_slowed_fighter_can_still_walk_off_the_marker() {
    let mut w = waiting();
    let rose = std::cell::Cell::new(false);
    w.players[0].slow(Knob::SpitSlowFrames.raw() as u16, Knob::SpitSlow.fx());
    land_then(&mut w, Input::aimed(Input::W, NORTH), 1);
    for _ in 0..90 {
        w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
        if matches!(
            worm(&w).doing,
            Doing::Active {
                kind: sandmaw::RISE,
                ..
            }
        ) {
            rose.set(true);
        }
    }
    assert!(rose.get(), "the landing did not bring it up");
    assert_eq!(
        w.players[0].health,
        w.players[0].full_health(),
        "slowed by the spit, walking off was not enough"
    );
}

#[test]
fn standing_on_the_noise_is_bitten() {
    // The other half of the rule: loud, then staying, is how you get bitten.
    let mut w = waiting();
    land_then(&mut w, Input::default(), 90);
    assert!(
        w.players[0].health < w.players[0].full_health(),
        "it did not come up under a fighter who stayed where they landed"
    );
}

#[test]
fn nothing_passes_under_rock() {
    let body = sp().fight_fx(FightField::BodyRadius);
    let mut frames = 0;
    for seed in 0..3 {
        rises(seed, |w| {
            let m = worm(w);
            // In the air over a lane, or lying where it came down, it is
            // not under anything.
            if matches!(
                m.doing,
                Doing::Active {
                    kind: sandmaw::BREACH,
                    ..
                } | Doing::Toppled { .. }
            ) {
                return;
            }
            let ground = w.terrain();
            for solid in ground.solids() {
                if solid.max.y.raw() <= sp().fight_fx(FightField::StepOver).raw() {
                    continue;
                }
                let gap = sim::math::flat_box_gap(m.pos, solid.min, solid.max);
                assert!(
                    gap.raw() >= body.sub(Fx::ratio(1, 20)).raw(),
                    "its head at {:?} is {gap:?} from rock, {:?}, posture {}, acted {:?}, attended {:?}",
                    m.pos,
                    m.doing,
                    fight::posture_of(&m),
                    fight::acted_on(&w.lore),
                    fight::attended(&w.lore)
                );
            }
            frames += 1;
        });
    }
    assert!(frames > 0);
}

// ---------------------------------------------------------------------------
// The undertow
// ---------------------------------------------------------------------------

/// A sinkhole at a point, as the undertow lays it.
fn sinkhole(w: &mut World, x: i32, z: i32) {
    let r = hazard::stat_fx(sp(), fight::SINKHOLE, HazardField::Radius);
    hazard::place(&mut w.lore, Hazard::disc(fight::SINKHOLE, at(x, z), r));
}

/// The worm well away, under the sand and taking no notice.
fn away(w: &mut World) {
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(14, 14);
    m.brain.grace = u16::MAX;
}

#[test]
fn an_airborne_fighter_is_not_pulled_by_the_undertow() {
    // Standing in it, pulled toward its middle.
    let mut w = hunt(Class::Champion);
    away(&mut w);
    w.players[0].pos = at(-2, 0);
    sinkhole(&mut w, 0, 0);
    play(&mut w, 20, Input::default());
    assert!(
        w.players[0].pos.x.raw() > Fx::from_int(-2).raw(),
        "standing in it, nothing pulled"
    );

    // Jumping straight up from the same place: nothing but the jump.
    let mut w = hunt(Class::Champion);
    away(&mut w);
    w.players[0].pos = at(-2, 0);
    sinkhole(&mut w, 0, 0);
    w.advance([Input::new(Input::SPACE), Input::default()]);
    let took_off = w.players[0].pos;
    for _ in 0..12 {
        w.advance([Input::new(Input::SPACE), Input::default()]);
        if w.players[0].grounded {
            break;
        }
        assert_eq!(
            flat(w.players[0].pos),
            flat(took_off),
            "the air is not sand: an airborne fighter drifted"
        );
    }
}

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

#[test]
fn a_hop_from_the_undertow_centre_clears_its_edge_on_every_class() {
    // From the middle of the sinkhole, walking out and jumping at once -- the
    // answer §2 gives -- every class lands outside it.
    let r = hazard::stat_fx(sp(), fight::SINKHOLE, HazardField::Radius);
    for class in sim::class::ALL_CLASSES {
        let mut w = hunt(class);
        away(&mut w);
        w.players[0].pos = at(0, 0);
        sinkhole(&mut w, 0, 0);
        play(&mut w, 2, Input::default());
        play(&mut w, 6, Input::aimed(Input::W, EAST));
        let mut landed = None;
        for _ in 0..120 {
            play(&mut w, 1, Input::aimed(Input::W | Input::SPACE, EAST));
            if w.players[0].grounded && w.players[0].since_landed == 1 {
                landed = Some(w.players[0].pos);
                break;
            }
        }
        let landed = landed.unwrap_or(w.players[0].pos);
        assert!(
            dist(landed, at(0, 0)).raw() > r.raw(),
            "{class:?} landed {:?} from the middle of a sinkhole {r:?} across",
            dist(landed, at(0, 0))
        );
    }
}

// ---------------------------------------------------------------------------
// Standing
// ---------------------------------------------------------------------------

#[test]
fn a_crouching_fighter_ducks_the_tail_lash() {
    for crouch in [false, true] {
        let mut w = hunt(Class::Champion);
        stand_at(&mut w, 0, 0, Fx::ZERO);
        // Behind it -- it faces east -- at the middle of the sweep.
        w.players[0].pos = at(-5, 0);
        let input = if crouch {
            Input::new(Input::CROUCH)
        } else {
            Input::default()
        };
        play(&mut w, 4, input);
        start(&mut w, sandmaw::LASH);
        let full = w.players[0].health;
        let a = sp().attack(sandmaw::LASH);
        play(&mut w, (a.startup + a.active + 2) as u32, input);
        let hit = w.players[0].health < full;
        assert_eq!(
            hit,
            !crouch,
            "crouching {crouch}: the lash {} them",
            if hit { "hit" } else { "missed" }
        );
    }
}

#[test]
fn the_spit_slows_and_never_covers_the_camera() {
    let mut w = hunt(Class::Champion);
    stand_at(&mut w, 0, 0, Fx::ZERO);
    w.players[0].pos = at(5, 0);
    play(&mut w, 2, Input::default());
    start(&mut w, sandmaw::SPIT);
    // Its shape is drawn on the floor through the tell.
    play(&mut w, 2, Input::default());
    let marks = w.marks();
    let rings: Vec<_> = marks
        .iter()
        .filter(|m| m.look == sim::species::MarkLook::Warning)
        .collect();
    assert_eq!(rings.len(), fight::SPIT_DISCS, "the cone is drawn");
    assert!(
        rings.iter().all(|m| m.height == Fx::ZERO),
        "the spray is drawn on the sand, never in the air in front of the camera"
    );
    let full = w.players[0].health;
    let a = sp().attack(sandmaw::SPIT);
    play(&mut w, (a.startup + a.active) as u32, Input::default());
    assert!(w.players[0].health < full, "the spit missed");
    assert!(w.players[0].slowed > 0, "and did not slow");
    assert_eq!(w.players[0].slow_mul, Knob::SpitSlow.fx());
}

#[test]
fn rock_between_stops_the_spit() {
    // Behind an island's boulder, the spray does not reach.
    let (ix, iz) = sim::arena::sandmaw::ISLANDS[2];
    let (ix, iz) = (ix / 100, iz / 100);
    let mut w = hunt(Class::Champion);
    // The worm west of the island, facing east across it; the fighter in the
    // boulder's lee, on the island.
    stand_at(&mut w, ix - 6, iz, Fx::ZERO);
    w.players[0].pos = V3::new(Fx::from_int(ix + 1), Fx::ratio(1, 2), Fx::from_int(iz));
    play(&mut w, 4, Input::default());
    start(&mut w, sandmaw::SPIT);
    let full = w.players[0].health;
    let a = sp().attack(sandmaw::SPIT);
    play(&mut w, (a.startup + a.active + 2) as u32, Input::default());
    assert_eq!(
        w.players[0].health, full,
        "the spit went through the boulder"
    );
}

// ---------------------------------------------------------------------------
// The swallow
// ---------------------------------------------------------------------------

/// The worm standing, a fighter in its mouth, and the swallow landing.
fn swallowed() -> World {
    let mut w = hunt(Class::Champion);
    w.players[1].health = w.players[1].full_health();
    w.players[1].pos = at(-10, 10);
    stand_at(&mut w, 0, 0, Fx::ZERO);
    play(&mut w, 2, Input::default());
    let m = worm(&w);
    let mut tel = m;
    tel.doing = Doing::Active {
        kind: sandmaw::SWALLOW,
        left: sp().attack(sandmaw::SWALLOW).active,
    };
    let (mouth, _, _, _) = tel.hit_volume().expect("the swallow has a mouth");
    w.players[0].pos = flat(mouth);
    start(&mut w, sandmaw::SWALLOW);
    let a = sp().attack(sandmaw::SWALLOW);
    for _ in 0..(a.startup + a.active + 2) {
        play(&mut w, 1, Input::default());
        if fight::holding(&w.lore).is_some() {
            break;
        }
    }
    assert_eq!(fight::holding(&w.lore), Some(0), "the swallow took nobody");
    w
}

/// Frames held so far.
fn held_for(w: &World) -> u32 {
    w.lore.word(fight::word::HOLD_T)
}

/// Run until the hold has lasted `t` frames.
fn hold_until(w: &mut World, t: u32) {
    while held_for(w) < t && fight::holding(&w.lore).is_some() {
        play(w, 1, Input::default());
    }
}

/// One press of the special, inside.
fn press(w: &mut World) {
    play(w, 1, Input::new(Input::SPECIAL));
    play(w, 1, Input::default());
}

#[test]
fn the_swallow_is_escaped_on_a_gulp_and_extra_presses_cost_the_gulp() {
    let every = Knob::GulpEvery.raw() as u32;

    // A press inside the first gulp's window: out, for the bite and a gulp.
    let mut w = swallowed();
    let full = w.players[0].full_health();
    hold_until(&mut w, every);
    press(&mut w);
    assert_eq!(fight::holding(&w.lore), None, "a press on the gulp is out");
    let ended = w.lore.word(fight::word::ENDED);
    assert_eq!(ended & 0xFF, 1, "out on the first gulp");
    assert_eq!(
        full - w.players[0].health,
        sp().attack(sandmaw::SWALLOW).damage + Knob::GulpDamage.raw(),
        "the first gulp costs the bite and a gulp"
    );
    assert!(
        matches!(worm(&w).doing, Doing::Flinch { .. }),
        "and it flinches, up"
    );

    // A press before the window spends the first gulp: pressing again in its
    // window does nothing, and only the second gulp lets them out.
    let mut w = swallowed();
    hold_until(&mut w, every / 2);
    press(&mut w);
    hold_until(&mut w, every);
    press(&mut w);
    assert!(
        fight::holding(&w.lore).is_some(),
        "an early press did not cost the gulp"
    );
    hold_until(&mut w, every * 2);
    press(&mut w);
    assert_eq!(
        fight::holding(&w.lore),
        None,
        "the next gulp is a fresh one"
    );
    assert_eq!(w.lore.word(fight::word::ENDED) & 0xFF, 2);

    // Never pressed: held to the end and spat out, for all of it.
    let mut w = swallowed();
    let full = w.players[0].full_health();
    hold_until(&mut w, Knob::HoldFrames.raw() as u32 + 1);
    play(&mut w, 2, Input::default());
    assert_eq!(fight::holding(&w.lore), None);
    let gulps = Knob::HoldFrames.raw() / Knob::GulpEvery.raw();
    assert_eq!(
        full - w.players[0].health,
        sp().attack(sandmaw::SWALLOW).damage
            + Knob::GulpDamage.raw() * gulps
            + Knob::SpitOutDamage.raw(),
        "missed throughout, the swallow costs everything"
    );
}

#[test]
fn a_teammate_hitting_the_head_frees_the_swallowed() {
    let mut w = swallowed();
    hold_until(&mut w, 5);
    w.monsters[0]
        .as_mut()
        .unwrap()
        .take_hit(sandmaw::HEAD_PART, 30);
    play(&mut w, 2, Input::default());
    assert_eq!(
        fight::holding(&w.lore),
        None,
        "a blow on the head frees them"
    );
    assert_eq!(
        w.lore.word(fight::word::ENDED) >> 8,
        fight::end::RESCUED,
        "rescued"
    );
}

#[test]
fn a_blow_into_the_open_mouth_gags_it() {
    let mut w = hunt(Class::Champion);
    stand_at(&mut w, 0, 0, Fx::ZERO);
    start(&mut w, sandmaw::SWALLOW);
    play(&mut w, 6, Input::default());
    let teeth = sp().break_slot(sandmaw::TEETH).unwrap();
    let before = worm(&w).breaks[teeth];
    w.monsters[0].as_mut().unwrap().take_hit(sandmaw::TEETH, 20);
    let m = worm(&w);
    assert!(matches!(m.doing, Doing::Flinch { .. }), "not gagged");
    assert_eq!(
        before - m.breaks[teeth],
        Fx::from_int(20)
            .mul(Knob::VulnTeeth.fx())
            .mul(Knob::GagTeeth.fx())
            .to_int(),
        "a blow into the open mouth is worth half again on the teeth"
    );
}

// ---------------------------------------------------------------------------
// The beach
// ---------------------------------------------------------------------------

#[test]
fn a_rise_into_a_stone_beaches_it() {
    let mut w = hunt(Class::Elementalist);
    w.players[0].pos = at(-12, -12);
    awake(&mut w);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(0, 0);
    m.aim_at(at(3, 0));
    start(&mut w, sandmaw::RISE);
    // A stone raised into the circle through the tell.
    if let sim::class::Mechanic::Structures(mut slots) = w.players[0].mechanic {
        slots[0] = Some(sim::class::Structure::raised(at(3, 0)));
        w.players[0].mechanic = sim::class::Mechanic::Structures(slots);
    } else {
        panic!("the Elementalist has no stones");
    }
    let a = sp().attack(sandmaw::RISE);
    play(&mut w, (a.startup + 3) as u32, Input::default());
    let m = worm(&w);
    assert!(
        matches!(m.doing, Doing::Toppled { .. }),
        "it rose into a stone and is {:?}",
        m.doing
    );
    assert_eq!(w.lore.word(fight::word::ROUTE), fight::route::STONE);
}

#[test]
fn a_rise_throws_a_planted_shield_clear() {
    let mut w = hunt(Class::Bulwark);
    w.players[0].pos = at(-12, -12);
    awake(&mut w);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(0, 0);
    m.aim_at(at(3, 0));
    let weight = Fx::ONE;
    w.players[0].mechanic = sim::class::Mechanic::Shield(sim::class::Shield::Planted {
        pos: at(3, 0).add(V3::new(Fx::ratio(1, 2), Fx::ZERO, Fx::ZERO)),
        weight,
    });
    start(&mut w, sandmaw::RISE);
    let a = sp().attack(sandmaw::RISE);
    play(&mut w, (a.startup + 3) as u32, Input::default());
    let m = worm(&w);
    assert!(
        !matches!(m.doing, Doing::Toppled { .. }),
        "a shield does not beach it"
    );
    let sim::class::Mechanic::Shield(sim::class::Shield::Planted { pos, .. }) =
        w.players[0].mechanic
    else {
        panic!("the shield is no longer planted");
    };
    assert!(
        dist(pos, at(3, 0)).raw() > fight::rise_radius(&m).raw(),
        "the shield was not thrown clear"
    );
}

#[test]
fn the_beached_back_is_inside_every_class_hop() {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(-12, -12);
    stand_at(&mut w, 0, 0, Fx::ZERO);
    w.monsters[0].as_mut().unwrap().doing = Doing::Toppled {
        left: sp().topple_frames(),
    };
    // Settled on its side, its back's highest point over the whole writhe.
    let mut highest = Fx::ZERO;
    for _ in 0..sp().topple_frames() - 10 {
        play(&mut w, 1, Input::default());
        let m = worm(&w);
        if !matches!(m.doing, Doing::Toppled { left } if left < sp().topple_frames() - 30) {
            continue;
        }
        let rig = m.rig();
        for part in [sandmaw::SEG_N2, sandmaw::SEG_ROOT, sandmaw::SEG_T2] {
            let sh = sp().shape(part);
            let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
            let top = rig.part_to_world(part, V3::new(mid.x, sh.max.y, mid.z)).y;
            highest = highest.max(top);
        }
    }
    for class in sim::class::ALL_CLASSES {
        let hop = apex(class);
        assert!(
            highest.raw() < hop.raw(),
            "{class:?} hops {hop:?}; the beached back is at {highest:?}"
        );
    }
}

#[test]
fn a_broken_tooth_ring_shrinks_the_rise_bite_for_the_rest_of_the_fight() {
    let mut w = hunt(Class::Champion);
    awake(&mut w);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(0, 0);
    m.aim_at(at(3, 0));
    start(&mut w, sandmaw::RISE);
    let whole = worm(&w).telegraph().unwrap().radius;
    // Broken through the swallow's open mouth.
    stand_at(&mut w, 0, 0, Fx::ZERO);
    start(&mut w, sandmaw::SWALLOW);
    play(&mut w, 4, Input::default());
    let health = sp().part_health();
    w.monsters[0]
        .as_mut()
        .unwrap()
        .take_hit(sandmaw::TEETH, health * 2);
    assert!(worm(&w).broken(sandmaw::TEETH), "the ring did not break");
    let m = w.monsters[0].as_mut().unwrap();
    m.own[fight::body::POSTURE] = fight::posture::BURIED;
    m.aim_at(at(3, 0));
    start(&mut w, sandmaw::RISE);
    let broken = worm(&w).telegraph().unwrap().radius;
    assert!(
        broken.raw() < whole.raw(),
        "the circle is {broken:?} with the ring broken, {whole:?} whole"
    );
    assert_eq!(broken, Knob::RiseBroken.fx());
}

// ---------------------------------------------------------------------------
// The ring
// ---------------------------------------------------------------------------

#[test]
fn the_noise_ring_is_bounded_and_does_not_allocate() {
    let mut w = hunt(Class::Champion);
    let room = w.lore.layout().noises as usize;
    for i in 0..100u32 {
        noise::make(
            &mut w.lore,
            NoiseKind::Dodge,
            at((i % 20) as i32 - 10, 0),
            0,
            0,
            i,
        );
        assert!(noise::all(&w.lore).count() <= room);
    }
    assert_eq!(noise::all(&w.lore).count(), room);
    // The newest are the ones kept.
    assert!(noise::all(&w.lore).all(|n| n.born >= 100 - room as u32));
    // The ring is cells in the snapshot: a `World` with it full is a flat
    // copy (`tests/budget.rs` holds every frame to no allocation at all).
    let copy = w.clone();
    assert_eq!(copy.lore, w.lore);
}
