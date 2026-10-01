//! The Veilstalker: the rules `docs/design/creatures/veilstalker.md` pins, as
//! sentences (§10).
//!
//! Some set up the animal in front of one fighter and start a move; the rest
//! run whole hunts against a crude fighter that looks at it, walks about and
//! swings when it is near -- not the scripted hunter, which lives in `hunt`
//! and is held to what is drawn, but enough to make it throw everything it
//! has -- and check every frame.

use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::veilstalker::{self as vs, Knob, fight};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

/// A hunt against it, the second fighter out of it.
fn hunt() -> World {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::VEILSTALKER);
    w.players[1].health = 0;
    w
}

fn beast(w: &World) -> Monster {
    w.monsters[0].expect("the creature")
}

/// Open snow in the Ashwood, away from the trunks, the ash and the stream.
fn base() -> V3 {
    V3::new(Fx::from_int(-3), Fx::ZERO, Fx::from_int(-3))
}

fn at(x: i32, z: i32) -> V3 {
    base().add(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z)))
}

/// The animal at [`base`] facing +x, a fighter `dist` metres down +x facing
/// it, nothing on its mind.
fn duel(dist: i32) -> World {
    let mut w = hunt();
    w.players[0].pos = at(dist, 0);
    w.players[0].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = base();
    m.yaw = Fx::ZERO;
    m.brain.grace = 0;
    m.brain.think_left = 600;
    m.brain.seen = at(dist, 0);
    for _ in 0..12 {
        w.advance([look_at_it(&w); MAX_PLAYERS]);
        hold(&mut w);
    }
    w
}

/// Keep it where the test put it, thinking about nothing.
fn hold(w: &mut World) {
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.think_left = m.brain.think_left.max(60);
    if m.doing.free() {
        m.pos = base();
        m.yaw = Fx::ZERO;
        m.yaw_rate = Fx::ZERO;
        m.speed = Fx::ZERO;
    }
}

/// The fighter looking at where the animal is, no keys.
fn look_at_it(w: &World) -> Input {
    let to = beast(w).pos.sub(w.players[0].pos);
    let yaw = sim::math::atan2_turns(to.z, to.x);
    Input::aimed(0, (yaw.raw() as u32 & 0xFFFF) as u16)
}

/// Start a move as its brain would: on the frame it is chosen, so the frame
/// hook sees its first frame.
fn start(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind,
        left: vs::SPECIES.attack(kind).startup + 1,
    };
    m.hit_used = false;
    m.brain.last_move = kind;
}

/// **A crude fighter** for whole hunts: it looks at the animal, sweeping a
/// little either side, walks a slow circle, and swings when it is close.
fn crude(w: &World, f: u32) -> Input {
    let me = w.players[0];
    let m = beast(w);
    let to = m.pos.sub(me.pos);
    let sweep = Fx::ratio(((f / 50) % 5) as i32 - 2, 24);
    let yaw = sim::math::atan2_turns(to.z, to.x).add(sweep);
    let mut bits = 0;
    if (f / 120) % 3 == 0 {
        bits |= Input::A;
    }
    if to.flat_len().raw() < Fx::from_int(4).raw() && f % 30 == 0 {
        bits |= Input::LEFT;
    }
    Input::aimed(bits, (yaw.raw() as u32 & 0xFFFF) as u16)
}

/// Play `frames` of a hunt against the crude fighter, from `seed`, handing
/// every frame's before and after to `check`.
fn play(seed: u32, frames: u32, mut check: impl FnMut(&World, &World)) {
    let mut w = hunt();
    w.monsters[0].as_mut().unwrap().brain.rng = seed | 1;
    for f in 0..frames {
        if !matches!(w.phase, sim::state::Phase::Fighting) {
            break;
        }
        let before = w.clone();
        w.advance([crude(&w, f), Input::default()]);
        check(&before, &w);
    }
}

/// Did a strike begin this frame?
fn strike_began(before: &World, after: &World) -> Option<u8> {
    let (was, now) = (before.monsters[0]?, after.monsters[0]?);
    match now.doing {
        Doing::Startup { kind, left }
            if fight::strikes(kind) && left == vs::SPECIES.attack(kind).startup =>
        {
            (was.doing != now.doing).then_some(kind)
        }
        _ => None,
    }
}

const SEEDS: [u32; 4] = [1, 7, 23, 91];
const LONG: u32 = 9000;

// ---------------------------------------------------------------------------
// The veil
// ---------------------------------------------------------------------------

#[test]
fn every_attack_decloaks_for_at_least_eighteen_frames_first() {
    let floor = Knob::DecloakFloor.raw();
    assert!(floor >= 18, "the floor itself is {floor}");
    for kind in vs::STRIKES {
        assert!(
            fight::decloak(kind) >= floor,
            "{} decloaks for {} frames",
            vs::MOVES[kind as usize].name,
            fight::decloak(kind)
        );
    }
    // And in play: every hit volume that comes out has been in view, ramping
    // up, for at least the floor's frames before it.
    for seed in SEEDS {
        let mut showing = 0;
        let mut checked = 0;
        play(seed, LONG, |before, after| {
            let (was, now) = (before.monsters[0].unwrap(), after.monsters[0].unwrap());
            if fight::veil(&now).raw() > 0 {
                showing += 1;
            } else {
                showing = 0;
            }
            let out =
                |m: &Monster| matches!(m.doing, Doing::Active { kind, .. } if fight::strikes(kind));
            if out(&now) && !out(&was) {
                let kind = now.doing.attacking().unwrap();
                if kind == vs::MIMIC {
                    return;
                }
                checked += 1;
                assert!(
                    showing >= floor,
                    "seed {seed}: {} came out after {showing} frames shown",
                    vs::MOVES[kind as usize].name
                );
            }
        });
        assert!(checked > 0, "seed {seed}: no strike came out to check");
    }
}

#[test]
fn it_never_commits_to_a_decloak_outside_the_target_s_glanced_view() {
    let mut checked = 0;
    for seed in SEEDS {
        play(seed, LONG, |before, after| {
            let Some(kind) = strike_began(before, after) else {
                return;
            };
            let m = before.monsters[0].unwrap();
            let target = (m.brain.target as usize).min(1);
            checked += 1;
            // What the brain read as it chose: the bits the frame before left.
            assert!(
                fight::view_of(&before.lore, target) & fight::view::IN_VIEW != 0
                    || kind == vs::MIMIC,
                "seed {seed}: {} committed outside the glanced view",
                vs::MOVES[kind as usize].name
            );
            // And the same question asked afresh, of where it stood and the
            // look it had glanced.
            if kind != vs::MIMIC {
                let ground = before.terrain();
                let stones = sim::stones::gather(&before.players);
                let scene = sim::aim::Scene {
                    stones: &stones,
                    players: &before.players,
                    effects: &before.effects,
                    quarry: &before.monsters,
                    critters: &before.critters,
                    arena: &ground,
                };
                let (pos, yaw) = fight::led_look(&before.lore, target);
                let cone = Knob::ViewCone.fx().add(Fx::ratio(1, 100));
                let mid = fight::middle_of(&before.monsters[0].unwrap());
                assert!(
                    sim::aim::in_view_from(pos, Fx::ZERO, yaw, mid, cone, &scene),
                    "seed {seed}: {} decloaked off the glanced screen",
                    vs::MOVES[kind as usize].name
                );
            }
        });
    }
    assert!(checked > 10, "only {checked} commits to check");
}

#[test]
fn a_cloaked_body_is_hit_exactly_like_a_visible_one() {
    // The same blow on the same body: once cloaked, once with every region
    // mottled so it is drawn whole.
    let blow = |mottled: bool| {
        let mut w = duel(3);
        if mottled {
            let m = w.monsters[0].as_mut().unwrap();
            for part in [vs::HEAD, vs::BARREL_L, vs::BARREL_R, vs::TAIL_BASE] {
                fight::struck(m, part, Knob::MottleDamage.raw());
            }
            m.own[fight::body::STRUCK] = 0;
            m.health = vs::SPECIES.health();
        }
        let shown = w.shown(0, vs::BARREL_L);
        let hp = beast(&w).health;
        let mut swung = false;
        for f in 0..60 {
            let mut input = look_at_it(&w);
            if f == 2 {
                input.bits |= Input::LEFT;
            }
            w.advance([input, Input::default()]);
            hold(&mut w);
            swung |= beast(&w).health < hp;
        }
        assert!(swung, "the blow landed");
        (shown, hp - beast(&w).health)
    };
    let (cloaked_shown, cloaked) = blow(false);
    let (shown, visible) = blow(true);
    assert_eq!(cloaked_shown, Fx::ZERO, "cloaked is drawn as nothing");
    assert_eq!(shown, Fx::ONE, "mottled is drawn whole");
    assert_eq!(cloaked, visible, "and the blow took the same either way");
}

#[test]
fn a_hit_paints_the_hide_where_it_landed_and_the_paint_moves_with_it() {
    let mut w = duel(3);
    let hp = beast(&w).health;
    for f in 0..40 {
        let mut input = look_at_it(&w);
        if f == 2 {
            input.bits |= Input::LEFT;
        }
        w.advance([input, Input::default()]);
        hold(&mut w);
        if beast(&w).health < hp {
            break;
        }
    }
    assert!(beast(&w).health < hp, "the blow landed");
    // The frame hook paints it on the frame after: blows land after the
    // creature's own step.
    w.advance([look_at_it(&w), Input::default()]);
    hold(&mut w);
    let paint = fight::paints(&w).next().expect("a mark where it landed");
    let m = beast(&w);
    let lit = fight::paint_at(&m, &paint);
    // On the side the blow came from: nearer the fighter than the body's
    // middle is.
    let me = w.players[0].pos;
    assert!(
        sim::math::wide_flat_dist(lit, me).raw() < sim::math::wide_flat_dist(m.pos, me).raw(),
        "the mark is on the side it was struck from"
    );
    assert!(
        w.shown(0, paint.part).raw() > 0,
        "the painted part is drawn"
    );
    // Turn the body a quarter: the mark goes with it.
    let mut turned = m;
    turned.yaw = turned.yaw.add(sim::math::QUARTER_TURN);
    let moved = fight::paint_at(&turned, &paint);
    assert!(
        sim::math::wide_flat_dist(moved, lit).raw() > Fx::ratio(1, 2).raw(),
        "the mark rides the hide"
    );
    // And it goes out after its six seconds.
    for _ in 0..Knob::PaintFrames.raw() + 2 {
        w.advance([look_at_it(&w), Input::default()]);
        hold(&mut w);
    }
    assert!(fight::paints(&w).next().is_none(), "the paint has faded");
}

#[test]
fn a_region_that_has_taken_enough_never_cloaks_again() {
    let mut w = duel(8);
    {
        let m = w.monsters[0].as_mut().unwrap();
        fight::struck(m, vs::BARREL_L, Knob::MottleDamage.raw() - 1);
        m.own[fight::body::STRUCK] = 0;
    }
    for _ in 0..60 {
        w.advance([look_at_it(&w), Input::default()]);
        hold(&mut w);
    }
    assert_eq!(
        w.shown(0, vs::BARREL_L),
        Fx::ZERO,
        "one short is still cloaked"
    );
    {
        let m = w.monsters[0].as_mut().unwrap();
        fight::struck(m, vs::HAUNCH_L, 1);
        m.own[fight::body::STRUCK] = 0;
    }
    for _ in 0..600 {
        w.advance([look_at_it(&w), Input::default()]);
        hold(&mut w);
        for part in 0..vs::PART_COUNT {
            let want = if vs::REGION[part] == vs::REGION_LEFT {
                Fx::ONE
            } else {
                Fx::ZERO
            };
            assert_eq!(
                w.shown(0, part),
                want,
                "{} after the left flank mottled",
                vs::PARTS[part].name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The trail
// ---------------------------------------------------------------------------

#[test]
fn every_footfall_is_in_the_ring_and_a_rollback_lays_the_same_trail() {
    // Every forefoot that comes down on snow is in the ring the frame it does.
    let mut stamped = 0;
    play(5, 3000, |before, after| {
        let (was, now) = (before.monsters[0].unwrap(), after.monsters[0].unwrap());
        let half = 1u32 << 15;
        let crossed = was.stride as u32 / half != now.stride as u32 / half;
        let walking = now.speed.abs().raw() > 0
            && !fight::airborne(&now)
            && !fight::perched(&now)
            && now.pos.y.raw() == 0;
        let foot = vs::SPECIES.legs[((now.stride as u32 / half) % 2) as usize].foot;
        let at = now.world_of(foot, V3::ZERO);
        let snow = after.arena().floor_at(at.x, at.z) == sim::arena::Material::Snow;
        // On the snow *floor*: a foot that reaches over the cordwood wall as
        // the body is stopped against it comes down on the wall's top, which
        // is not snow and keeps no print.
        let down = after.terrain().ground_under(at).raw() == 0;
        if crossed && walking && snow && down {
            stamped += 1;
            assert!(
                fight::prints(after).any(|p| p.age == 0),
                "a footfall on snow at frame {} left no print",
                after.frame
            );
        }
    });
    assert!(stamped > 20, "only {stamped} footfalls to check");

    // A rollback re-simulates the same frames and lays the same trail.
    let mut w = hunt();
    let mut inputs = Vec::new();
    for f in 0..1500 {
        let i = crude(&w, f);
        inputs.push(i);
        w.advance([i, Input::default()]);
    }
    let saved = w.clone();
    for f in 1500..2100 {
        let i = crude(&w, f);
        inputs.push(i);
        w.advance([i, Input::default()]);
    }
    let first: Vec<_> = fight::prints(&w).collect();
    let mut again = saved;
    for i in &inputs[1500..] {
        again.advance([*i, Input::default()]);
    }
    let second: Vec<_> = fight::prints(&again).collect();
    assert_eq!(first, second, "the same trail");
    assert_eq!(w.checksum(), again.checksum(), "and the same world");
}

#[test]
fn a_real_decloak_stamps_its_feet_and_a_mimic_stamps_nothing() {
    let mut w = duel(8);
    let before: Vec<_> = fight::prints(&w).collect();
    start(&mut w, vs::LUNGE);
    w.advance([look_at_it(&w), Input::default()]);
    let new = fight::prints(&w).filter(|p| p.age == 0).count();
    assert_eq!(
        new,
        4,
        "a decloak sets all four feet (had {})",
        before.len()
    );

    let mut w = duel(8);
    let count = |w: &World| fight::prints(w).filter(|p| p.age < 40).count();
    let had = count(&w);
    start(&mut w, vs::MIMIC);
    for _ in 0..(vs::SPECIES.attack(vs::MIMIC).startup + 2) {
        w.advance([look_at_it(&w), Input::default()]);
        assert!(fight::ghost(&w).is_some() || !matches!(beast(&w).doing, Doing::Startup { .. }));
    }
    assert!(
        count(&w) <= had,
        "a mimic stamps nothing, and the animal stood still"
    );
}

#[test]
fn no_strike_follows_a_mimic_inside_the_quiet() {
    let quiet = Knob::MimicQuiet.raw() as u32;
    let mut mimics = 0;
    for seed in SEEDS {
        let mut ended: Option<u32> = None;
        // Start it wounded, so it mimics.
        let mut w = hunt();
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.rng = seed | 1;
        m.health = vs::SPECIES.health() / 2;
        for f in 0..LONG {
            if !matches!(w.phase, sim::state::Phase::Fighting) {
                break;
            }
            let before = w.clone();
            w.advance([crude(&w, f), Input::default()]);
            let (was, now) = (before.monsters[0].unwrap(), w.monsters[0].unwrap());
            let mimicking = |m: &Monster| m.doing.attacking() == Some(vs::MIMIC);
            if mimicking(&was) && !mimicking(&now) {
                mimics += 1;
                ended = Some(w.frame);
            }
            if let Some(kind) = strike_began(&before, &w) {
                if kind != vs::MIMIC {
                    if let Some(e) = ended {
                        assert!(
                            w.frame.saturating_sub(e) >= quiet,
                            "seed {seed}: {} {} frames after a mimic",
                            vs::MOVES[kind as usize].name,
                            w.frame - e
                        );
                    }
                }
            }
        }
    }
    assert!(mimics > 0, "it never mimicked");
}

// ---------------------------------------------------------------------------
// Fire, smoke, the retreat
// ---------------------------------------------------------------------------

#[test]
fn twenty_frames_in_fire_panics_it_and_it_is_shown_throughout() {
    let mut w = duel(9);
    sim::hazard::place(
        &mut w.lore,
        sim::hazard::Hazard::disc(fight::COALS, base(), Fx::from_int(2)),
    );
    let need = Knob::FireFrames.raw();
    let mut frames = 0;
    loop {
        w.advance([look_at_it(&w), Input::default()]);
        hold(&mut w);
        frames += 1;
        if matches!(beast(&w).doing, Doing::Toppled { .. }) {
            break;
        }
        assert!(frames <= need, "no panic after {frames} frames in fire");
    }
    assert!(frames >= need - 1, "it panicked after only {frames}");
    assert!(fight::panicked(&beast(&w)));
    // Shown whole, every frame, all the way to its feet.
    while matches!(beast(&w).doing, Doing::Toppled { .. })
        || beast(&w).doing.attacking() == Some(vs::GETUP)
    {
        for part in 0..vs::PART_COUNT {
            assert_eq!(w.shown(0, part), Fx::ONE);
        }
        w.advance([look_at_it(&w), Input::default()]);
    }
    assert_eq!(
        fight::panics(&w.lore)[3],
        1,
        "a panic by the brazier's coals"
    );
}

#[test]
fn it_never_strikes_out_of_its_own_smoke_or_into_it_early() {
    let mut smokes = 0;
    let margin = Fx::ratio(1, 2);
    for seed in SEEDS {
        let mut w = hunt();
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.rng = seed | 1;
        m.health = vs::SPECIES.health() / 2;
        for f in 0..LONG {
            if !matches!(w.phase, sim::state::Phase::Fighting) {
                break;
            }
            let before = w.clone();
            w.advance([crude(&w, f), Input::default()]);
            if fight::cloud(&w).is_some() && fight::cloud(&before).is_none() {
                smokes += 1;
            }
            let Some(kind) = strike_began(&before, &w).filter(|k| *k != vs::MIMIC) else {
                continue;
            };
            let Some((c, r, age)) = fight::cloud(&before) else {
                continue;
            };
            let m = before.monsters[0].unwrap();
            let target = before.players[(m.brain.target as usize).min(1)].pos;
            let d = |p: V3| sim::math::wide_flat_dist(c, p);
            let me_in = d(m.pos).raw() < r.sub(margin).raw();
            let them_out = d(target).raw() > r.add(margin).raw();
            let them_in = d(target).raw() < r.sub(margin).raw();
            assert!(
                !(me_in && them_out),
                "seed {seed}: {} out of its cloud at somebody outside it",
                vs::MOVES[kind as usize].name
            );
            assert!(
                !(them_in && age + 2 < Knob::SmokeQuiet.raw() as u32),
                "seed {seed}: {} into a cloud {age} frames old",
                vs::MOVES[kind as usize].name
            );
        }
    }
    assert!(smokes > 0, "it never smoked");
}

#[test]
fn after_two_hits_it_leaves_and_a_burst_in_the_recoil_keeps_it() {
    // Two hits, a moment apart: it recoils.
    let mut w = duel(8);
    let gap = Knob::HitGap.raw() as u32 + 1;
    for _ in 0..2 {
        {
            let m = w.monsters[0].as_mut().unwrap();
            let dealt = m.take_hit(vs::BARREL_L, 10);
            assert!(dealt > 0);
        }
        for _ in 0..gap {
            w.advance([look_at_it(&w), Input::default()]);
            if beast(&w).doing.attacking() == Some(vs::RETREAT) {
                break;
            }
        }
    }
    assert_eq!(
        beast(&w).doing.attacking(),
        Some(vs::RETREAT),
        "two hits and it leaves"
    );
    assert!(
        matches!(beast(&w).doing, Doing::Startup { .. }),
        "in its recoil"
    );
    assert_eq!(w.shown(0, vs::BARREL_R), Fx::ONE, "in full view");

    // The same, and a burst past the bar inside the recoil.
    let m = w.monsters[0].as_mut().unwrap();
    let bar = m.interrupt_bar();
    m.take_hit(vs::BARREL_L, bar + 10);
    assert!(
        matches!(m.doing, Doing::Flinch { left } if left as i32 == Knob::RetreatStagger.raw()),
        "a burst in the recoil staggers it: {:?}",
        m.doing
    );
    assert_eq!(fight::hits(m), 0, "and the engagement goes on");
}

#[test]
fn a_spear_s_lane_stops_following_at_its_lock() {
    let mut w = duel(5);
    start(&mut w, vs::SPEAR);
    let lock = Knob::SpearLock.raw();
    let mut locked_yaw = None;
    for e in 0..vs::SPECIES.attack(vs::SPEAR).startup as i32 {
        // The fighter walks sideways the whole tell.
        let mut input = look_at_it(&w);
        input.bits |= Input::A;
        w.advance([input, Input::default()]);
        let m = beast(&w);
        if e > lock + 1 {
            let y = *locked_yaw.get_or_insert(m.yaw);
            assert_eq!(m.yaw, y, "the lane moved after its lock, at frame {e}");
        }
    }
}

#[test]
fn the_pounce_s_circle_is_where_the_target_stood_and_does_not_follow() {
    let mut w = duel(9);
    {
        let m = w.monsters[0].as_mut().unwrap();
        m.aim_at(w.players[0].pos);
    }
    let stood = w.players[0].pos;
    let ground = w.terrain();
    {
        let m = w.monsters[0].as_mut().unwrap();
        fight::mark_pounce(m, stood, &ground);
    }
    start(&mut w, vs::POUNCE);
    for _ in 0..30 {
        let mut input = look_at_it(&w);
        input.bits |= Input::D;
        w.advance([input, Input::default()]);
        let t = beast(&w).telegraph().expect("the circle");
        assert!(sim::math::wide_flat_dist(t.anchor, stood).raw() < Fx::ratio(1, 10).raw());
    }
}
