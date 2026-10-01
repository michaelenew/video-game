//! The Gnawers (`docs/design/creatures/gnawers.md`): the rules of the pack
//! that takes turns, pinned as relationships rather than values. Each test is
//! one line of the document's §10 list, or a rule it rests on.
//!
//! The machinery underneath -- critters, the ring, tokens, morale, the aim at
//! something short -- is pinned on the dev pack in `tests/critters.rs`; what
//! is here is the Gnawers' own: the crouch, the hamstring at a back, the
//! pile-on on a slow, the scatter's window, the Big One's maul and howl, the
//! gnaw and the scramble, and the reuse a brood will make of the gnawer.

use sim::aim::{self, Contact, Scene, Targets};
use sim::critter::{self, Critter, CritterField, MAX_CRITTERS, flag, is};
use sim::fixed::Fx;
use sim::pack::{self, PackKnob, mood};
use sim::species::SpeciesId;
use sim::species::gnawers::{self as g, Knob};
use sim::state::{Action, MAX_PLAYERS};
use sim::{Class, Input, V3, World};

fn hunt(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::GNAWERS);
    // One hunter: the second fighter is out of it, as the harness has it.
    w.players[1].health = 0;
    w
}

fn idle() -> [Input; MAX_PLAYERS] {
    [Input::default(); MAX_PLAYERS]
}

fn keep_up(w: &mut World) {
    w.players[0].health = w.players[0].full_health();
}

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

fn knob(k: Knob) -> i32 {
    g::knob(k)
}

fn knob_fx(k: Knob) -> Fx {
    g::knob_fx(k)
}

fn kill(w: &mut World, i: usize) {
    let hp = w.critters[i].health as i32;
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        i,
        hp + 1,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
}

fn leader(w: &World) -> usize {
    w.pack.unwrap().leader as usize
}

fn scene_of(w: &World) -> (sim::stones::Field, sim::arena::Terrain) {
    (sim::stones::gather(&w.players), w.terrain())
}

/// Where a body starts its windup this frame, if one does.
fn began(before: &World, after: &World, act: u8) -> Vec<usize> {
    (0..MAX_CRITTERS)
        .filter(|i| {
            let (b, a) = (&before.critters[*i], &after.critters[*i]);
            a.state == is::STARTUP && b.state != is::STARTUP && a.act == act
        })
        .collect()
}

/// A body's hit connecting this frame.
fn connected(before: &World, after: &World, act: u8) -> usize {
    (0..MAX_CRITTERS)
        .filter(|i| {
            let (b, a) = (&before.critters[*i], &after.critters[*i]);
            a.act == act
                && a.state == is::ACTIVE
                && a.has(flag::HIT_USED)
                && !(b.state == is::ACTIVE && b.has(flag::HIT_USED))
        })
        .count()
}

// ---------------------------------------------------------------------------
// The pack, and the snapshot
// ---------------------------------------------------------------------------

#[test]
fn a_full_pack_fits_the_snapshot() {
    let world = std::mem::size_of::<World>();
    assert!(world <= 4096, "a World is {world} bytes");
    // Six and the Big One, in the Commons, from the den.
    let w = hunt(Class::Champion);
    let p = w.pack.expect("a pack");
    assert_eq!(p.species, SpeciesId::GNAWERS);
    assert_eq!(w.arena, sim::arena::ArenaId::GNAWERS);
    let alive: Vec<&Critter> = w.critters.iter().filter(|c| c.alive()).collect();
    assert_eq!(alive.len(), 7);
    assert_eq!(alive.iter().filter(|c| c.kind == g::BIG_ONE).count(), 1);
    assert!(w.critters[leader(&w)].has(flag::LEADER));
    // Two hunters: three more from the den, the Gnawers' coop pack -- ten,
    // the most there is.
    let mut two = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GNAWERS);
    for _ in 0..3 {
        two.advance(idle());
    }
    assert_eq!(two.critters.iter().filter(|c| c.alive()).count(), 10);
    assert_eq!(
        two.pack.unwrap().token_cap() as i32,
        two.pack.unwrap().sp().pack_raw(PackKnob::Tokens) + knob(Knob::CoopTokens)
    );
}

#[test]
fn the_sizes_are_the_documents() {
    // A gnawer is below a fighter's hip and the Big One below the chest; a
    // fighter walks at seven and a gnawer darts at nine (§1).
    let sp = &g::SPECIES;
    let crown = critter::stat_fx(sp, g::GNAWER, CritterField::Height);
    let big = critter::stat_fx(sp, g::BIG_ONE, CritterField::Height);
    let hip = sim::tuning::body_height().mul(Fx::ratio(1, 2));
    assert!(crown.raw() < hip.raw(), "a gnawer stands {crown:?}");
    assert!(
        big.raw() < sim::tuning::cast_height().raw(),
        "the Big One stands {big:?}"
    );
    let dart = critter::stat_fx(sp, g::GNAWER, CritterField::Run);
    assert!(dart.raw() > sim::tuning::move_speed().raw());
    assert!(dart.raw() < sim::tuning::dodge_speed().raw());
    // One bite is under a twentieth of a fighter; the number that matters is
    // how many land in a second.
    assert!(sp.attack(g::DART).damage * 20 < 1000);
}

// ---------------------------------------------------------------------------
// The aim at something short (gnawers.md §1a, built with the foundation)
// ---------------------------------------------------------------------------

/// One gnawer held still `metres` in front of a fighter at the lane
/// `critcheck` uses, the crosshair on its middle.
fn one_gnawer(class: Class, metres: i32, kind: u8) -> (World, Input) {
    let mut w = hunt(class);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    let mut c = Critter::new(
        sp,
        kind,
        sim::critcheck::lane().add(V3::new(Fx::from_int(metres), Fx::ZERO, Fx::ZERO)),
        1 << 15,
    );
    c.state = is::FLINCH;
    c.timer = u16::MAX;
    c.health = i16::MAX;
    w.critters[0] = c;
    w.pack.as_mut().unwrap().grace = u16::MAX;
    w.players[0].pos = sim::critcheck::lane();
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    for _ in 0..2 {
        w.advance([Input::looking_at(0, 0, 0), Input::default()]);
    }
    let middle = w.critters[0].body(sp).middle();
    let pitch = aim::look_onto_closely(w.players[0].pos, 0, w.players[0].aloft, middle);
    (w, Input::looking_at(0, 0, pitch))
}

#[test]
fn where_only_fighters_stand_every_aim_is_unchanged() {
    // In the Gnawers' own arena, nothing short under the crosshair: "there"
    // is a fighter's height and no swing stoops. The pinned hunts hold the
    // rest of it bit for bit.
    let mut w = hunt(Class::Elementalist);
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    let (field, ground) = scene_of(&w);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    for pitch in [-12000i16, -6000, -2000, 0, 3000] {
        let look = Input::looking_at(0, 0, pitch);
        let stands = aim::stands_at(0, look, &scene);
        assert_eq!(stands.height, sim::tuning::body_height());
        assert_eq!(aim::stoop(look, true, stands), Fx::ZERO);
    }
}

#[test]
fn a_skillshot_aimed_through_a_gnawer_lands_on_the_gnawer() {
    for kind in [g::GNAWER, g::BIG_ONE] {
        for metres in [3, 6, 9] {
            let (w, look) = one_gnawer(Class::Elementalist, metres, kind);
            let (field, ground) = scene_of(&w);
            let scene = Scene {
                stones: &field,
                players: &w.players,
                effects: &w.effects,
                quarry: &w.monsters,
                critters: &w.critters,
                arena: &ground,
            };
            let path = aim::skillshot_path(0, look, Fx::from_int(20), &scene);
            let middle = w.critters[0].body(w.critters.sp()).middle();
            assert!(
                path.to.y.sub(middle.y).abs().raw() < Fx::ratio(1, 50).raw(),
                "kind {kind} at {metres} m: the shot ends {} m up",
                path.to.y.to_f32_for_render()
            );
            let met = aim::first_along(
                path,
                Fx::ratio(1, 10),
                0,
                &scene,
                Targets::none().quarry(true),
            );
            assert!(
                matches!(met, Some(Contact::Critter { index: 0, .. })),
                "kind {kind} at {metres} m it met {met:?}"
            );
        }
    }
}

#[test]
fn a_standing_swing_pointed_at_a_gnawer_dips_to_meet_it() {
    let (w, look) = one_gnawer(Class::Champion, 2, g::GNAWER);
    let (field, ground) = scene_of(&w);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let p = w.players[0];
    let at_it = aim::stands_at(0, look, &scene);
    assert_eq!(at_it.height, w.critters[0].body(w.critters.sp()).height);
    let dipped = aim::swing_path(
        p.pos,
        p.facing,
        look,
        true,
        Fx::from_int(2),
        aim::Hand::Centre,
        at_it,
        sim::V3::Y,
    );
    assert!(
        dipped.dir().y.raw() < 0,
        "pointed at a gnawer, the swing stayed level"
    );
}

#[test]
fn a_swing_pointed_over_a_gnawer_stays_level() {
    // Over it, at the Big One's head say: a look just under the horizon, the
    // crosshair on the floor far behind. Inside the dead zone, and nothing
    // short under the crosshair.
    let (w, _) = one_gnawer(Class::Champion, 2, g::GNAWER);
    let (field, ground) = scene_of(&w);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let p = w.players[0];
    let over = Input::looking_at(0, 0, -800);
    let past = aim::stands_at(0, over, &scene);
    assert_eq!(past.height, sim::tuning::body_height());
    let level = aim::swing_path(
        p.pos,
        p.facing,
        over,
        true,
        Fx::from_int(2),
        aim::Hand::Centre,
        past,
        sim::V3::Y,
    );
    assert_eq!(
        level.dir().y,
        Fx::ZERO,
        "pointed over the gnawer, the swing dipped"
    );
}

#[test]
fn every_class_can_touch_a_gnawer_with_its_auto() {
    // `critcheck --species gnawers` as an assertion: every class's auto
    // touches a gnawer at two metres, crosshair on its middle (M2's "every
    // auto touches"). The Dual mage's dark auto is the one lunge that runs
    // through a body inside two metres -- `critters.md` §7, a person's call
    // -- so hers is asked at three, where she throws it from.
    let rows = sim::critcheck::table(SpeciesId::GNAWERS, g::GNAWER);
    for class in sim::class::ALL_CLASSES {
        let auto = if class == Class::BloodMage {
            sim::moves::blood::SWEEP
        } else {
            0
        };
        let row = rows
            .iter()
            .find(|r| r.class == class && r.kind == auto)
            .unwrap();
        let at = if class == Class::DualMage { 3 } else { 2 };
        let k = sim::critcheck::METRES
            .iter()
            .position(|m| *m == at)
            .unwrap();
        assert!(
            row.trials[k].hurt,
            "{}'s auto does not touch a gnawer at {at} m: {:?}",
            class.name(),
            row.trials
        );
    }
}

// ---------------------------------------------------------------------------
// Bites and tokens (§2, §5)
// ---------------------------------------------------------------------------

/// A hunt with the fighter standing still in the open middle of the Commons,
/// kept alive, for `frames`, handing every frame to `look`.
fn standing_in_the_open(class: Class, frames: u32, mut look: impl FnMut(&World, &World)) -> World {
    let mut w = hunt(class);
    w.players[0].pos = at(0, -2);
    for _ in 0..frames {
        keep_up(&mut w);
        let before = w.clone();
        w.advance(idle());
        look(&before, &w);
    }
    w
}

#[test]
fn only_the_token_holders_bite() {
    let mut darts = 0;
    let mut hamstrings = 0;
    standing_in_the_open(Class::Bulwark, 3000, |before, after| {
        let pack = after.pack.unwrap();
        for i in 0..MAX_CRITTERS {
            let c = &after.critters[i];
            let needs = matches!(c.act, g::DART | g::HAMSTRING);
            if c.alive() && c.attacking() && needs {
                assert!(c.has(flag::TOKEN), "a {} without a token", c.act);
            }
        }
        // A token is only handed out while one is free. (A howl's rally
        // running out can leave a third holder holding until its move ends:
        // what is pinned is the handing out.)
        let held = |w: &World| w.critters.iter().filter(|c| c.has(flag::TOKEN)).count();
        let new_holders = (0..MAX_CRITTERS)
            .filter(|i| {
                after.critters[*i].has(flag::TOKEN) && !before.critters[*i].has(flag::TOKEN)
            })
            .count();
        if new_holders > 0 {
            let then = before.pack.unwrap();
            assert!(
                held(before) + new_holders <= then.token_cap().max(pack.token_cap()),
                "a token handed out with {} of {} already held",
                held(before),
                then.token_cap()
            );
        }
        darts += began(before, after, g::DART).len();
        hamstrings += began(before, after, g::HAMSTRING).len();
    });
    // In the open, facing away from where it came from, the ring goes round
    // to the back: mostly hamstrings.
    assert!(
        darts + hamstrings > 5,
        "only {darts} dart-bites and {hamstrings} hamstrings in fifty seconds"
    );
    assert!(
        hamstrings > 0,
        "never a hamstring at a fighter standing in the open"
    );
}

#[test]
fn a_dart_bite_is_a_crouch_of_twenty_four_frames_from_in_front() {
    // The tell is the move (§2): twenty-four frames, answerable on sight; and
    // it is thrown at somebody facing it.
    let sp = &g::SPECIES;
    assert!(sp.attack(g::DART).startup >= 24);
    let mut seen = 0;
    let mut w = hunt(Class::Bulwark);
    w.players[0].pos = at(0, -14);
    for _ in 0..3000 {
        keep_up(&mut w);
        let before = w.clone();
        w.advance([facing_the_den(), Input::default()]);
        let after = &w;
        let before = &before;
        for i in began(before, after, g::DART) {
            let c = after.critters[i];
            let p = after.players[0];
            let from = V3::new(c.pos.x.sub(p.pos.x), Fx::ZERO, c.pos.z.sub(p.pos.z));
            assert!(
                p.facing.dot(from.normalized()).raw() > 0,
                "a dart-bite from behind"
            );
            seen += 1;
        }
    }
    assert!(seen > 0);
}

/// Facing the den with your back to the south wall: the ring is in front,
/// and the bites are dart-bites.
fn facing_the_den() -> Input {
    Input::aimed(0, 1 << 14)
}

#[test]
fn a_hit_in_the_crouch_knocks_the_gnawer_out_of_its_lunge() {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(0, -14);
    let mut stopped = 0;
    for _ in 0..3000 {
        keep_up(&mut w);
        let crouching: Vec<usize> = (0..MAX_CRITTERS)
            .filter(|i| {
                let c = &w.critters[*i];
                c.alive() && c.state == is::STARTUP && c.act == g::DART && c.timer > 2
            })
            .collect();
        for i in crouching {
            // The lightest blow there is.
            pack::hurt(
                &mut w.pack,
                &mut w.critters,
                i,
                1,
                V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
                Fx::ZERO,
                Fx::ZERO,
            );
            let c = w.critters[i];
            assert_eq!(c.state, is::FLINCH, "a hit in the crouch did not stop it");
            assert!(!c.has(flag::TOKEN), "a stopped crouch kept its token");
            stopped += 1;
        }
        w.advance([facing_the_den(), Input::default()]);
    }
    assert!(stopped > 0);
}

#[test]
fn a_hamstring_is_only_thrown_at_a_back() {
    let mut seen = 0;
    standing_in_the_open(Class::Bulwark, 4000, |before, after| {
        for i in began(before, after, g::HAMSTRING) {
            let c = after.critters[i];
            // The pack's glance of the facing: what it decided on.
            let s = after.pack.unwrap().seen[0];
            let facing = V3::from_turns(Fx::from_raw(s.facing as i32));
            let from = V3::new(c.pos.x.sub(s.pos.x), Fx::ZERO, c.pos.z.sub(s.pos.z));
            assert!(
                facing.dot(from.normalized()).raw() <= knob_fx(Knob::RearCos).raw(),
                "a hamstring from the front"
            );
            seen += 1;
        }
    });
    assert!(seen > 0, "no hamstring at an open back in a minute");
}

#[test]
fn the_hamstring_marker_is_inside_the_floor_the_camera_shows() {
    // Thrown from `hamstring_reach` behind, its whole marker -- where the
    // hit will be, and its edge -- is on the screen of the fighter it is for,
    // looking level: the camera sits behind the shoulder, so behind the heels
    // is in view (§2).
    let mut w = hunt(Class::Champion);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    w.players[0].pos = at(0, 0);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    let a = sp.attack(g::HAMSTRING);
    let reach = a.ideal_range.add(a.range_span);
    let mut c = Critter::new(sp, g::GNAWER, V3::new(reach.neg(), Fx::ZERO, Fx::ZERO), 0);
    c.state = is::STARTUP;
    c.act = g::HAMSTRING;
    c.timer = a.startup;
    w.critters[0] = c;
    let t = w.critters[0].telegraph(sp).expect("a telegraph");
    let (field, ground) = scene_of(&w);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let look = Input::looking_at(0, 0, 0);
    let half = Fx::from_raw(9100);
    for edge in [
        t.anchor,
        t.anchor.sub(V3::new(t.radius, Fx::ZERO, Fx::ZERO)),
        t.anchor.add(V3::new(Fx::ZERO, Fx::ZERO, t.radius)),
        t.anchor.sub(V3::new(Fx::ZERO, Fx::ZERO, t.radius)),
    ] {
        assert!(
            aim::in_view(0, look, edge, half, &scene),
            "the hamstring's marker at {edge:?} is off the screen"
        );
    }
}

#[test]
fn a_landed_hamstring_slows_and_latches_and_a_dodge_sheds_it() {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(0, -2);
    let mut latched = None;
    for _ in 0..6000 {
        keep_up(&mut w);
        w.advance(idle());
        if let Some(i) = (0..MAX_CRITTERS).find(|i| g::latched(&w.critters[*i])) {
            latched = Some(i);
            break;
        }
    }
    let i = latched.expect("no hamstring latched in a hundred seconds");
    assert!(w.players[0].slowed > 0, "latched and not slowed");
    // Held for a moment, it bites.
    let before = w.players[0].health;
    for _ in 0..knob(Knob::LatchEvery) * 2 {
        w.advance(idle());
    }
    assert!(w.players[0].health < before, "a latch that does not bite");
    // A dodge sheds it, on the first frame one can be thrown.
    for _ in 0..120 {
        if w.players[0].action.actionable() && w.players[0].frozen == 0 {
            break;
        }
        w.advance(idle());
    }
    w.advance([Input::aimed(Input::SHIFT | Input::W, 0), Input::default()]);
    w.advance(idle());
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "no dodge came out"
    );
    assert!(
        !g::latched(&w.critters[i]),
        "a dodge did not shed the latch"
    );
}

// ---------------------------------------------------------------------------
// The pile-on (§2)
// ---------------------------------------------------------------------------

/// The Commons with the pack ringed round a fighter in the open, before the
/// grace is out: everybody in reach and nobody committed.
fn ringed(class: Class) -> World {
    let mut w = hunt(class);
    w.players[0].pos = at(0, -2);
    w.pack.as_mut().unwrap().grace = 600;
    for _ in 0..400 {
        keep_up(&mut w);
        w.advance(idle());
    }
    let p = w.pack.as_mut().unwrap();
    p.grace = 0;
    w
}

#[test]
fn a_slowed_fighter_draws_every_gnawer_in_range() {
    let mut w = ringed(Class::Champion);
    let me = w.players[0].pos;
    let radius = knob_fx(Knob::PileonRadius);
    let near: Vec<usize> = (0..MAX_CRITTERS)
        .filter(|i| {
            let c = &w.critters[*i];
            c.alive()
                && !c.has(flag::LEADER)
                && c.state == is::PROWL
                && c.pos.sub(me).flat_len().raw() <= radius.raw()
        })
        .collect();
    assert!(near.len() >= 3, "only {} in range", near.len());
    w.players[0].slow(120, Fx::ratio(6, 10));
    let mut leapt = [false; MAX_CRITTERS];
    for _ in 0..40 {
        keep_up(&mut w);
        let before = w.clone();
        w.advance(idle());
        for i in began(&before, &w, g::PILE_ON) {
            leapt[i] = true;
        }
    }
    for i in near {
        assert!(leapt[i], "gnawer {i} in range did not join the pile-on");
    }
    // Sequenced: no two leap on the same frame.
    let mut times = Vec::new();
    for c in w
        .critters
        .iter()
        .filter(|c| c.act == g::PILE_ON && c.state == is::STARTUP)
    {
        times.push(c.timer);
    }
    times.sort();
    assert!(
        times.windows(2).all(|t| t[0] != t[1]),
        "two leapers on the same frame: {times:?}"
    );
}

#[test]
fn a_dodge_taken_on_the_slow_clears_the_pile_on() {
    // Slowed, the fighter sees the pile-on begin a reaction later and dodges
    // out, away from the pack: every leaper is locked to the lane it was
    // called on, so the dodge's travel takes it out of all of them, and the
    // slow does not slow a dodge.
    let mut w = ringed(Class::Champion);
    w.players[0].slow(120, Fx::ratio(6, 10));
    let mut landed = 0;
    let mut dodged = false;
    let mut seen_at = None;
    for f in 0..150u32 {
        keep_up(&mut w);
        let before = w.clone();
        // A reaction after the first crouch of it is seen, out of the ring:
        // away from where the pack is, along the lane it is not on.
        let input = match seen_at {
            Some(s) if f == s + 15 && !dodged => {
                dodged = true;
                let me = w.players[0].pos;
                let n = w.critters.iter().filter(|c| c.alive()).count() as i32;
                let sum = w
                    .critters
                    .iter()
                    .filter(|c| c.alive())
                    .fold(V3::ZERO, |a, c| a.add(c.pos.sub(me)));
                let away = V3::new(sum.x.neg(), Fx::ZERO, sum.z.neg())
                    .scale(Fx::ONE.div(Fx::from_int(n.max(1))));
                let yaw = sim::math::atan2_turns(away.z, away.x);
                Input::aimed(Input::SHIFT | Input::W, (yaw.raw() & 0xFFFF) as u16)
            }
            _ => Input::default(),
        };
        w.advance([input, Input::default()]);
        if seen_at.is_none() && !began(&before, &w, g::PILE_ON).is_empty() {
            seen_at = Some(f);
        }
        landed += connected(&before, &w, g::PILE_ON);
    }
    assert!(dodged, "no pile-on was called on a slowed fighter");
    assert_eq!(landed, 0, "a dodge out of the ring was still piled on");
}

#[test]
fn three_pile_on_bites_are_a_knockdown() {
    let mut w = ringed(Class::Champion);
    w.players[0].slow(240, Fx::ratio(6, 10));
    let mut bites = 0;
    let mut down = false;
    for _ in 0..120 {
        keep_up(&mut w);
        let before = w.clone();
        w.advance(idle());
        bites += connected(&before, &w, g::PILE_ON);
        if bites >= knob(Knob::PileonKnockdown) as usize
            && matches!(w.players[0].action, Action::Stagger { .. })
        {
            down = true;
        }
    }
    assert!(
        bites >= 3,
        "only {bites} pile-on bites on a fighter who stood there"
    );
    assert!(down, "three bites and still standing");
}

// ---------------------------------------------------------------------------
// The scatter and the window (§1, §4)
// ---------------------------------------------------------------------------

#[test]
fn a_death_scatters_the_rest_for_the_window_it_promises() {
    let mut w = ringed(Class::Champion);
    let me = w.players[0].pos;
    let victim = (0..MAX_CRITTERS)
        .find(|i| w.critters[*i].alive() && !w.critters[*i].has(flag::LEADER))
        .unwrap();
    let lead = leader(&w);
    let big_from = w.critters[lead].pos;
    kill(&mut w, victim);
    let p = w.pack.unwrap();
    assert_eq!(p.mood, mood::SCATTERED);
    let frames = p.sp().pack_raw(PackKnob::ScatterFrames);
    assert!(frames >= 60, "the window is {frames} frames");
    let mut attacked = false;
    for _ in 0..frames - 1 {
        keep_up(&mut w);
        let before = w.clone();
        w.advance(idle());
        attacked |=
            !began(&before, &w, g::DART).is_empty() || !began(&before, &w, g::HAMSTRING).is_empty();
        assert_eq!(w.pack.unwrap().mood, mood::SCATTERED);
    }
    assert!(!attacked, "something bit in the scatter");
    // The ring pushed out; the Big One only a little back.
    let out = sim::species::gnawers::knob_fx(Knob::LeaderScatter);
    for (i, c) in w.critters.iter().enumerate() {
        if !c.alive() {
            continue;
        }
        if i == lead {
            let moved = c.pos.sub(big_from).flat_len();
            assert!(
                moved.raw() <= out.add(Fx::ONE).raw(),
                "the Big One went {moved:?} in the scatter"
            );
        } else {
            let gap = c.pos.sub(me).flat_len();
            assert!(
                gap.raw() >= Fx::from_int(6).raw(),
                "a gnawer only {gap:?} away at the end of a scatter"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The Big One (§2, §4)
// ---------------------------------------------------------------------------

/// The Big One a few metres in front of a fighter, winding up `act`.
fn big_one_winding(class: Class, act: u8, metres: Fx) -> World {
    let mut w = hunt(class);
    let sp = w.critters.sp();
    let lead = leader(&w);
    for (i, c) in w.critters.iter_mut().enumerate() {
        if i != lead {
            *c = Critter::EMPTY;
        }
    }
    w.players[0].pos = at(0, -4);
    w.players[0].facing = V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE);
    let p = w.pack.as_mut().unwrap();
    p.grace = 0;
    let c = &mut w.critters[lead];
    c.pos = at(0, -4).add(V3::new(Fx::ZERO, Fx::ZERO, metres));
    c.yaw = 3 << 14; // facing -z: at the fighter
    c.vel = V3::ZERO;
    c.state = is::STARTUP;
    c.act = act;
    c.timer = sp.attack(act).startup;
    w
}

#[test]
fn the_maul_is_cleared_by_every_class_hop() {
    // The answer to the maul is a jump (§2): its volume stops at a metre, and
    // every class's hop clears a metre. Seen a reaction late and jumped then,
    // held, nobody is touched.
    for class in sim::class::ALL_CLASSES {
        let sp = &g::SPECIES;
        let a = sp.attack(g::MAUL);
        let mut w = big_one_winding(class, g::MAUL, a.ideal_range);
        let health = w.players[0].health;
        let mut hit = false;
        for f in 0..(a.startup + a.active + 10) as u32 {
            let input = if f >= 15 {
                Input::aimed(Input::SPACE, 1 << 14)
            } else {
                Input::aimed(0, 1 << 14)
            };
            let before = w.clone();
            w.advance([input, Input::default()]);
            hit |= connected(&before, &w, g::MAUL) > 0;
        }
        assert!(!hit, "{}'s hop did not clear the maul", class.name());
        assert_eq!(w.players[0].health, health);
        // And standing still, it lands: the hop is the answer, not luck.
        let mut w = big_one_winding(class, g::MAUL, a.ideal_range);
        let mut hit = false;
        for _ in 0..(a.startup + a.active + 10) as u32 {
            let before = w.clone();
            w.advance([Input::aimed(0, 1 << 14), Input::default()]);
            hit |= connected(&before, &w, g::MAUL) > 0;
        }
        assert!(hit, "a maul at a {} standing still missed", class.name());
    }
}

#[test]
fn any_hit_cancels_the_howl() {
    let mut w = big_one_winding(Class::Champion, g::HOWL, Fx::from_int(8));
    let lead = leader(&w);
    let tokens = w.pack.unwrap().token_cap();
    // Reared to a fighter's height while it howls: the shape everything aims
    // at (§6).
    let body = w.critters[lead].body(w.critters.sp());
    assert!(body.height.raw() >= knob_fx(Knob::HowlHeight).raw());
    for _ in 0..10 {
        w.advance(idle());
    }
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        lead,
        1,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    assert_eq!(w.critters[lead].state, is::FLINCH);
    for _ in 0..60 {
        w.advance(idle());
    }
    assert_eq!(
        w.pack.unwrap().token_cap(),
        tokens,
        "a cancelled howl still rallied"
    );
    // And left alone, one rallies.
    let mut w = big_one_winding(Class::Champion, g::HOWL, Fx::from_int(8));
    for _ in 0..60 {
        w.advance(idle());
    }
    assert_eq!(
        w.pack.unwrap().token_cap() as i32,
        knob(Knob::HowlTokens),
        "a howl that came out did not rally"
    );
}

#[test]
fn a_burst_past_its_strain_knocks_the_big_one_down() {
    let mut w = big_one_winding(Class::Champion, g::HOWL, Fx::from_int(8));
    let lead = leader(&w);
    let strain = knob(Knob::Strain);
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        lead,
        strain,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    assert!(
        g::stumbling(&w.critters[lead]),
        "past its strain and still up"
    );
    assert!(w.critters[lead].timer as i32 >= knob(Knob::StumbleFrames) - 1);
}

// ---------------------------------------------------------------------------
// Where you stand (§1, §3, §5)
// ---------------------------------------------------------------------------

#[test]
fn a_wall_at_your_back_empties_the_slots_behind_you() {
    // Back to the south wall, facing the den: the ring is a half-ring in
    // front.
    let mut w = hunt(Class::Bulwark);
    w.players[0].pos = at(0, -14);
    w.players[0].facing = V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE);
    w.pack.as_mut().unwrap().grace = 600;
    for _ in 0..200 {
        keep_up(&mut w);
        w.advance([Input::aimed(0, 1 << 14), Input::default()]);
    }
    let pack = w.pack.unwrap();
    let seen = pack.seen[0];
    let me = w.players[0];
    let mut placed = 0;
    for c in w
        .critters
        .iter()
        .filter(|c| c.alive() && !c.has(flag::LEADER) && c.slot != critter::NO_SLOT)
    {
        let p = pack::ring_point(&pack, &seen, c.slot);
        assert!(
            p.z.raw() > me.pos.z.sub(Fx::ONE).raw(),
            "a ring place behind a fighter backed against a wall"
        );
        placed += 1;
    }
    assert!(placed >= 4);
}

#[test]
fn the_pack_breaks_at_half_or_at_the_leader() {
    // The leader dead: broken for good.
    let mut w = ringed(Class::Champion);
    let lead = leader(&w);
    kill(&mut w, lead);
    assert_eq!(w.pack.unwrap().mood, mood::BROKEN);
    // Half the pack dead, the leader alive: routed.
    let mut w = ringed(Class::Champion);
    let lead = leader(&w);
    let members: Vec<usize> = (0..MAX_CRITTERS)
        .filter(|i| *i != lead && w.critters[*i].alive())
        .collect();
    let mustered = w.pack.unwrap().mustered as i32;
    let share = w.pack.unwrap().sp().pack_raw(PackKnob::RoutShare);
    let mut killed = 0;
    for i in members {
        kill(&mut w, i);
        killed += 1;
        if killed * 100 >= share * mustered {
            break;
        }
        assert_ne!(w.pack.unwrap().mood, mood::ROUTED, "routed at {killed}");
    }
    assert_eq!(w.pack.unwrap().mood, mood::ROUTED);
}

/// A pack routed, the fighter standing `from` the den.
fn routed(away: V3) -> World {
    let mut w = ringed(Class::Champion);
    let lead = leader(&w);
    let members: Vec<usize> = (0..MAX_CRITTERS)
        .filter(|i| *i != lead && w.critters[*i].alive())
        .collect();
    for i in members {
        if w.pack.unwrap().mood == mood::ROUTED {
            break;
        }
        kill(&mut w, i);
    }
    assert_eq!(w.pack.unwrap().mood, mood::ROUTED);
    let home = w.pack.unwrap().home;
    w.players[0].pos = V3::new(home.x.add(away.x), Fx::ZERO, home.z.add(away.z));
    w
}

#[test]
fn a_routed_pack_left_alone_comes_back_and_a_watched_one_does_not() {
    let regroup = g::SPECIES.pack_raw(PackKnob::RegroupFrames) as u32;
    // Left alone: twenty metres off, it comes back in its time.
    let mut w = routed(at(0, -22));
    for _ in 0..regroup + 30 {
        keep_up(&mut w);
        w.advance(idle());
    }
    assert_eq!(
        w.pack.unwrap().mood,
        mood::HUNTING,
        "a routed pack left alone stayed out"
    );
    // Watched: standing a few metres off the den's mouth, inside the clear
    // and outside the bay, it stays out.
    let mut w = routed(at(0, -7));
    for _ in 0..regroup + 30 {
        keep_up(&mut w);
        w.players[0].pos = {
            let home = w.pack.unwrap().home;
            V3::new(home.x, Fx::ZERO, home.z.sub(Fx::from_int(7)))
        };
        w.advance(idle());
    }
    assert_eq!(
        w.pack.unwrap().mood,
        mood::ROUTED,
        "a watched rout came back"
    );
}

#[test]
fn a_rout_cornered_at_the_den_turns() {
    let mut w = routed(at(0, -2));
    for _ in 0..30 {
        keep_up(&mut w);
        w.advance(idle());
    }
    assert_eq!(
        w.pack.unwrap().mood,
        mood::HUNTING,
        "cornered at the den, it did not turn"
    );
}

#[test]
fn a_stone_you_stand_on_falls_to_three_diggers() {
    let mut w = hunt(Class::Elementalist);
    // In the open, clear of the trunk: the pack has no path-finding, and a
    // digger the far side of a solid slides along it.
    let spot = at(4, -10);
    w.players[0].pos = spot;
    sim::stones::raise(&mut w.players[0], sim::class::Structure::raised(spot));
    // Let it rise, and stand on its top.
    w.pack.as_mut().unwrap().grace = 900;
    for _ in 0..120 {
        keep_up(&mut w);
        w.advance(idle());
    }
    let field = sim::stones::gather(&w.players);
    let stone = field.iter().flatten().next().copied().expect("a stone");
    assert!(
        w.players[0].pos.y.raw() >= stone.top().sub(Fx::ratio(1, 4)).raw(),
        "the fighter is not on the stone ({:?} against a top at {:?})",
        w.players[0].pos.y,
        stone.top()
    );
    w.pack.as_mut().unwrap().grace = 0;
    let mut felled_at = None;
    let mut most = 0;
    for f in 0..1800u32 {
        keep_up(&mut w);
        w.advance(idle());
        let digging = w
            .critters
            .iter()
            .filter(|c| c.alive() && c.state == is::ACTIVE && c.act == g::GNAW)
            .count();
        most = most.max(digging);
        if sim::stones::gather(&w.players).iter().flatten().count() == 0 {
            felled_at = Some(f);
            break;
        }
    }
    assert!(
        felled_at.is_some(),
        "nobody gnawed the stone down in thirty seconds"
    );
    assert_eq!(
        most as i32,
        knob(Knob::GnawDiggers),
        "{most} diggers at once"
    );
    // Whoever was on it comes down among them, staggered.
    assert!(
        matches!(w.players[0].action, Action::Stagger { .. }),
        "felled, the fighter was {:?}",
        w.players[0].action
    );
    // Three diggers bring it down in 120 frames of digging (§2).
    assert_eq!(knob(Knob::GnawWork) / knob(Knob::GnawDiggers), 120);
}

#[test]
fn a_gnawer_scrambles_up_to_a_fighter_on_the_trunk_and_a_hit_knocks_it_off() {
    let mut w = hunt(Class::Champion);
    // On the fallen trunk, the Commons' scramble.
    let trunk = *w
        .arena()
        .solids()
        .iter()
        .find(|s| s.material == sim::arena::Material::Wood)
        .expect("the trunk");
    let mid = V3::new(
        trunk.min.x.add(trunk.max.x).mul(Fx::ratio(1, 2)),
        trunk.max.y,
        trunk.min.z.add(trunk.max.z).mul(Fx::ratio(1, 2)),
    );
    let mut up = None;
    for _ in 0..3000 {
        keep_up(&mut w);
        w.players[0].pos = mid;
        w.players[0].vel = V3::ZERO;
        w.players[0].grounded = true;
        w.advance(idle());
        if let Some(i) = (0..MAX_CRITTERS).find(|i| {
            let c = &w.critters[*i];
            c.alive() && c.pos.y.raw() >= trunk.max.y.sub(Fx::ratio(1, 4)).raw()
        }) {
            up = Some(i);
            break;
        }
    }
    let i = up.expect("no gnawer scrambled up the trunk in fifty seconds");
    let _ = i;
    // While one clings to the edge, a hit knocks it off: it flinches out of
    // the scramble and never arrives.
    let mut w = hunt(Class::Champion);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        if c.alive() && !c.has(flag::LEADER) {
            c.state = is::STARTUP;
            c.act = g::SCRAMBLE;
            c.timer = sp.attack(g::SCRAMBLE).startup;
            break;
        }
    }
    let clinging = (0..MAX_CRITTERS)
        .find(|i| w.critters[*i].act == g::SCRAMBLE && w.critters[*i].state == is::STARTUP)
        .unwrap();
    pack::hurt(
        &mut w.pack,
        &mut w.critters,
        clinging,
        1,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
        Fx::ZERO,
    );
    assert_eq!(w.critters[clinging].state, is::FLINCH);
}

#[test]
fn a_fighter_out_of_reach_trees_the_pack() {
    // Feet above a leap's reach for long enough: the ring widens under them
    // and nobody bites.
    let mut w = ringed(Class::DualMage);
    let high = knob_fx(Knob::LeapReach).add(Fx::ONE);
    let mut bit = false;
    for _ in 0..240 {
        keep_up(&mut w);
        w.players[0].pos = V3::new(Fx::ZERO, high, Fx::from_int(-2));
        w.players[0].vel = V3::ZERO;
        let before = w.clone();
        w.advance(idle());
        bit |= !began(&before, &w, g::DART).is_empty();
    }
    assert!(g::treed_now(&w.pack.unwrap(), 0));
    let me = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-2));
    let wide = knob_fx(Knob::TreedRadius).sub(Fx::from_int(2));
    for c in w
        .critters
        .iter()
        .filter(|c| c.alive() && !c.has(flag::LEADER))
    {
        assert!(
            c.pos.sub(me).flat_len().raw() >= wide.raw(),
            "a gnawer bunched under a treed fighter"
        );
    }
    let _ = bit;
}

// ---------------------------------------------------------------------------
// The brood and the parasites (§10)
// ---------------------------------------------------------------------------

#[test]
fn a_gnawer_pack_runs_without_a_leader_and_spawns_into_a_freed_slot() {
    // What a brood will be: gnawers with no Big One, topped up mid-fight from
    // a point. The mind assumes neither a leader nor a full pack.
    let mut w = hunt(Class::Bulwark);
    w.players[0].pos = at(0, -2);
    let lead = leader(&w);
    w.critters[lead] = Critter::EMPTY;
    w.pack.as_mut().unwrap().leader = pack::NONE;
    let mut bites = 0;
    let mut spawned = 0;
    for f in 0..2400 {
        keep_up(&mut w);
        let before = w.clone();
        w.advance(idle());
        bites += began(&before, &w, g::DART).len() + began(&before, &w, g::HAMSTRING).len();
        // Every few seconds one dies and a sac bursts where it fell.
        if f % 300 == 299 {
            if let Some(i) = (0..MAX_CRITTERS).find(|i| w.critters[*i].alive()) {
                kill(&mut w, i);
            }
            let p = w.pack.as_mut().unwrap();
            if pack::spawn(p, &mut w.critters, g::GNAWER, at(4, 4), 0).is_some() {
                spawned += 1;
            }
        }
    }
    assert!(bites > 0, "a pack with no leader never bit");
    assert!(spawned > 0, "nothing spawned into a freed slot");
}

// ---------------------------------------------------------------------------
// In the world (§11)
// ---------------------------------------------------------------------------

#[test]
fn a_won_hunt_is_the_gnawers_trophy_at_its_temper() {
    // Every body down: the hunt is won, and what was beaten is the Gnawers,
    // at the temper it was fought at -- what the trophy case writes.
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GNAWERS).tempered(2);
    w.players[1].health = 0;
    assert_eq!(w.pack.unwrap().temper, 2);
    for i in 0..MAX_CRITTERS {
        if w.critters[i].alive() {
            kill(&mut w, i);
        }
    }
    for _ in 0..5 {
        w.advance(idle());
    }
    let (beaten, temper) = w.hunt_won().expect("a won hunt");
    assert!(beaten.contains(&Some(SpeciesId::GNAWERS)));
    assert_eq!(temper, 2);
}

#[test]
fn a_harder_temper_glances_sooner() {
    // Tempers come free (species.md §5, 11): the pack's glance and lead are
    // read through it.
    let plain = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GNAWERS);
    let hard = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GNAWERS)
        .tempered(sim::temper::HIGHEST);
    assert!(hard.pack.unwrap().glance_frames() < plain.pack.unwrap().glance_frames());
}
