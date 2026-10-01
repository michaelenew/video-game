//! Small bodies and packs (bestiary P3), and the two changes to the aiming
//! model they force (A1 and A2). Proven on the dev pack, the gnats
//! (`sim::species::gnats`): six gnats 0.6 m at the crown and a queen.
//!
//! What is pinned here is the machinery every pack creature stands on -- the
//! tokens, the ring, morale, spawning, standing on a monster, the hit test and
//! the overlay agreeing -- and the promise the aim change makes: that a short
//! body is touched wherever a fighter would be, and that where only fighters
//! stand nothing moved at all. See `docs/design/critters.md`.

use sim::aim::{self, Contact, Scene, Targets};
use sim::critter::{self, Critter, CritterField, MAX_CRITTERS, flag, is};
use sim::fixed::Fx;
use sim::monster::{MAX_MONSTERS, NO_PART};
use sim::pack::{self, mood};
use sim::species::{SpeciesId, gnats};
use sim::state::{MAX_PLAYERS, Phase, hitbox};
use sim::{Class, Input, V3, World};

fn gnats(class: Class) -> World {
    World::hunt_of([class; MAX_PLAYERS], SpeciesId::GNATS)
}

fn idle() -> [Input; MAX_PLAYERS] {
    [Input::default(); MAX_PLAYERS]
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

/// Keep a fighter alive and standing still, so a long run measures the pack
/// rather than the round ending.
fn keep_up(w: &mut World) {
    for p in w.players.iter_mut() {
        p.health = p.full_health();
    }
}

// ---------------------------------------------------------------------------
// The bodies, and the snapshot
// ---------------------------------------------------------------------------

#[test]
fn a_critter_is_forty_eight_bytes_and_a_full_pack_fits_the_snapshot() {
    assert_eq!(std::mem::size_of::<Critter>(), 48);
    let world = std::mem::size_of::<World>();
    assert!(world <= 4096, "a World is {world} bytes");
    // Every slot full: the Gnawers' coop pack is ten, the most there is.
    let mut w = gnats(Class::Champion);
    for i in 0..MAX_CRITTERS {
        if !w.critters[i].present() {
            let at = V3::new(Fx::from_int(i as i32), Fx::ZERO, Fx::ZERO);
            pack::spawn(
                w.pack.as_mut().unwrap(),
                &mut w.critters,
                gnats::GNAT,
                at,
                0,
            );
        }
    }
    assert!(w.critters.iter().all(Critter::present));
    for _ in 0..60 {
        w.advance(idle());
    }
}

#[test]
fn a_pack_hunt_musters_its_pack_and_builds_no_monster() {
    let w = gnats(Class::Bulwark);
    assert!(w.monsters.iter().all(Option::is_none));
    assert!(w.hunting());
    let pack = w.pack.expect("a pack");
    assert_eq!(pack.species, SpeciesId::GNATS);
    assert_eq!(w.critters.iter().filter(|c| c.alive()).count(), 7);
    let leader = pack.leader as usize;
    assert!(w.critters[leader].has(flag::LEADER));
    assert_eq!(w.critters[leader].kind, gnats::QUEEN);
    // And a restart is the same fight.
    let again = w.restarted([Class::Bulwark; MAX_PLAYERS]);
    assert_eq!(again.pack.map(|p| p.species), Some(SpeciesId::GNATS));
    assert_eq!(again.critters.iter().filter(|c| c.alive()).count(), 7);
}

// ---------------------------------------------------------------------------
// The brain
// ---------------------------------------------------------------------------

#[test]
fn the_ring_is_cut_round_the_fighters_back() {
    // In the open, in the middle of the proving ground: walls and platforms
    // at a ring's radius are what *empty* places, which is the next test.
    let mut w = gnats(Class::Bulwark);
    w.players[0].pos = V3::ZERO;
    for _ in 0..30 {
        w.advance(idle());
    }
    let pack = w.pack.unwrap();
    let sp = pack.sp();
    let seen = pack.seen[0];
    let me = w.players[0];
    let (mut behind, mut ahead) = (0, 0);
    for c in w
        .critters
        .iter()
        .filter(|c| c.alive() && !c.has(flag::LEADER))
    {
        assert_ne!(
            c.slot,
            critter::NO_SLOT,
            "a member with no place on the ring"
        );
        let at = pack::ring_point(sp, &seen, c.slot);
        let off = at.sub(me.pos);
        if off.dot(me.facing).raw() < 0 {
            behind += 1;
        } else {
            ahead += 1;
        }
    }
    assert!(
        behind >= ahead,
        "the ring should fill the back first: {behind} behind, {ahead} ahead"
    );
}

#[test]
fn tokens_limit_how_many_bite_at_once() {
    let mut w = gnats(Class::Bulwark);
    let cap = w.pack.unwrap().token_cap();
    assert_eq!(cap, 2, "the gnats are tuned to two tokens");
    let mut most = 0;
    let mut bites = 0;
    for _ in 0..1800 {
        keep_up(&mut w);
        w.advance(idle());
        let holding = w.critters.iter().filter(|c| c.has(flag::TOKEN)).count();
        let biting = w
            .critters
            .iter()
            .filter(|c| c.attacking() && c.act == gnats::BITE)
            .count();
        assert!(holding <= cap, "{holding} tokens out of {cap}");
        assert!(biting <= holding, "a bite without a token");
        most = most.max(biting);
        bites += w
            .critters
            .iter()
            .filter(|c| c.state == is::ACTIVE && c.timer == 1)
            .count();
    }
    assert_eq!(most, cap, "the pack never used every token it had");
    assert!(bites > 4, "nothing bit in half a minute");
}

#[test]
fn the_leap_needs_no_token_and_waits_for_somebody_in_trouble() {
    let mut w = gnats(Class::Champion);
    let mut leapt = 0;
    for _ in 0..1800 {
        keep_up(&mut w);
        let before = w.critters;
        let seen = w.pack.unwrap().seen;
        w.advance(idle());
        for (i, c) in w.critters.iter().enumerate() {
            if c.state == is::STARTUP && before[i].state != is::STARTUP && c.act == gnats::LEAP {
                let s = seen[c.target as usize];
                // The glance this decision was made on, or the one taken this
                // frame: either saw the fighter in trouble.
                let now = w.pack.unwrap().seen[c.target as usize];
                assert!(s.down || s.slowed || now.down || now.slowed);
                assert!(!c.has(flag::TOKEN), "a leap took a token");
                leapt += 1;
            }
        }
    }
    assert!(
        leapt > 0,
        "a bitten fighter is in trouble; somebody should leap"
    );
}

#[test]
fn a_death_scatters_the_rest_for_the_window_it_promises() {
    let mut w = gnats(Class::Bulwark);
    w.players[0].pos = V3::ZERO;
    for _ in 0..90 {
        w.advance(idle());
    }
    let sp = w.pack.unwrap().sp();
    let frames = sp.pack_raw(pack::PackKnob::ScatterFrames) as u32;
    let me = w.players[0].pos;
    let spread = |w: &World| {
        let alive: Vec<&Critter> = w.critters.iter().filter(|c| c.alive()).collect();
        alive
            .iter()
            .map(|c| c.pos.sub(me).flat_len().raw() as i64)
            .sum::<i64>()
            / alive.len() as i64
    };
    let before = spread(&w);
    kill(&mut w, 0);
    assert_eq!(w.pack.unwrap().mood, mood::SCATTERED);
    for _ in 0..frames - 1 {
        w.advance(idle());
        assert_eq!(w.pack.unwrap().mood, mood::SCATTERED);
    }
    assert!(spread(&w) > before, "they did not back off");
    w.advance(idle());
    w.advance(idle());
    assert_eq!(w.pack.unwrap().mood, mood::HUNTING);
}

#[test]
fn half_dead_routs_and_a_pack_left_alone_comes_back() {
    let mut w = gnats(Class::Bulwark);
    for _ in 0..30 {
        w.advance(idle());
    }
    // Four of seven is over the half.
    for i in 0..4 {
        kill(&mut w, i);
    }
    assert_eq!(w.pack.unwrap().mood, mood::ROUTED);
    // Nobody near the den: they come back after the regroup.
    let regroup = w.pack.unwrap().sp().pack_raw(pack::PackKnob::RegroupFrames) as u32;
    for _ in 0..regroup + 2 {
        // The fighter well away from home.
        w.players[0].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ZERO);
        w.advance(idle());
    }
    let p = w.pack.unwrap();
    assert_eq!(p.mood, mood::HUNTING, "a pack left alone regroups");
    assert_eq!(p.lost, 0);
    assert_eq!(p.mustered, 3, "counted again against who came back");
}

#[test]
fn a_routed_pack_that_is_watched_stays_routed() {
    let mut w = gnats(Class::Bulwark);
    for _ in 0..30 {
        w.advance(idle());
    }
    for i in 0..4 {
        kill(&mut w, i);
    }
    let home = w.pack.unwrap().home;
    let regroup = w.pack.unwrap().sp().pack_raw(pack::PackKnob::RegroupFrames) as u32;
    for _ in 0..regroup + 60 {
        w.players[0].pos = home;
        keep_up(&mut w);
        w.advance(idle());
    }
    assert_eq!(w.pack.unwrap().mood, mood::ROUTED);
}

#[test]
fn the_leaders_death_breaks_the_pack_and_the_hunt_is_won_when_they_are_gone() {
    let mut w = gnats(Class::Bulwark);
    for _ in 0..30 {
        w.advance(idle());
    }
    let leader = w.pack.unwrap().leader as usize;
    kill(&mut w, leader);
    assert_eq!(w.pack.unwrap().mood, mood::BROKEN);
    let mut won = false;
    for _ in 0..900 {
        w.advance(idle());
        if let Phase::RoundOver { winner, .. } = w.phase {
            won = winner == 0;
            break;
        }
    }
    assert!(
        won,
        "the survivors should have gone home and ended the hunt"
    );
    assert!(w.critters.iter().all(|c| !c.alive()));
}

#[test]
fn a_critter_spawns_into_a_freed_slot_and_never_over_a_fresh_corpse() {
    let mut w = gnats(Class::Bulwark);
    let at = V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO);
    let brain = w.pack.as_mut().unwrap();
    // Seven mustered: three slots empty.
    for expect in 7..MAX_CRITTERS {
        assert_eq!(
            pack::spawn(brain, &mut w.critters, gnats::GNAT, at, 0),
            Some(expect)
        );
    }
    assert_eq!(
        pack::spawn(brain, &mut w.critters, gnats::GNAT, at, 0),
        None
    );
    kill(&mut w, 2);
    // The body is still lying there.
    let brain = w.pack.as_mut().unwrap();
    assert_eq!(
        pack::spawn(brain, &mut w.critters, gnats::GNAT, at, 0),
        None
    );
    let corpse = critter::stat(brain.sp(), gnats::GNAT, CritterField::Corpse) as u32;
    for _ in 0..corpse + 1 {
        keep_up(&mut w);
        w.advance(idle());
    }
    let brain = w.pack.as_mut().unwrap();
    assert_eq!(
        pack::spawn(brain, &mut w.critters, gnats::GNAT, at, 0),
        Some(2)
    );
    assert!(w.critters[2].alive());
}

#[test]
fn a_pack_fight_replays_exactly() {
    let script: Vec<[Input; MAX_PLAYERS]> = (0..600u32)
        .map(|f| {
            let bits = if f % 37 < 6 { 1 } else { 0 };
            [Input::aimed(bits, (f * 97) as u16), Input::default()]
        })
        .collect();
    let run = |from: &World| {
        let mut w = from.clone();
        for i in &script {
            w.advance(*i);
        }
        w.checksum()
    };
    let start = gnats(Class::Champion);
    assert_eq!(run(&start), run(&start));
    // And it is not the versus world's hash.
    assert_ne!(
        start.checksum(),
        World::with_classes([Class::Champion; MAX_PLAYERS]).checksum()
    );
}

// ---------------------------------------------------------------------------
// The world they stand in
// ---------------------------------------------------------------------------

#[test]
fn critters_stand_on_a_monsters_back_and_go_where_it_goes() {
    let mut w = World::hunt_with(
        [Class::Bulwark; MAX_PLAYERS],
        [Some(SpeciesId::RIDGEBACK), Some(SpeciesId::GNATS)],
    );
    assert!(w.monsters[0].is_some() && w.pack.is_some());
    let beast = w.monsters[0].unwrap();
    let part = beast.sp().part_named("barrel").unwrap();
    let top = {
        let sh = beast.sp().shape(part);
        beast.world_of(part, V3::new(Fx::ZERO, sh.max.y, Fx::ZERO))
    };
    // Dropped onto its back, and held there doing nothing of its own.
    let c = &mut w.critters[0];
    c.pos = V3::new(top.x, top.y.add(Fx::ratio(1, 10)), top.z);
    c.state = is::FLINCH;
    c.timer = u16::MAX;
    for _ in 0..3 {
        w.advance(idle());
    }
    assert_ne!(
        w.critters[0].mount, NO_PART,
        "it did not land on the animal"
    );
    // Wherever the animal goes, the critter is where its perch says.
    for _ in 0..240 {
        w.advance(idle());
        let c = w.critters[0];
        if c.mount == NO_PART {
            break;
        }
        let held = c.perched_at(&w.monsters).unwrap();
        assert!(held.sub(c.pos).len().raw() < Fx::ratio(1, 20).raw());
        assert!(c.pos.y.raw() > Fx::from_int(3).raw(), "not on its back");
    }
    assert_ne!(w.critters[0].mount, NO_PART, "it fell off a walking animal");
    let _ = MAX_MONSTERS;
}

#[test]
fn critters_do_not_walk_through_the_arenas_solids() {
    let mut w = gnats(Class::Bulwark);
    let arena = w.arena();
    // The proving ground's first solid that stands on the floor.
    let solid = *arena
        .solids()
        .iter()
        .find(|s| s.min.y.raw() <= 0)
        .expect("a solid");
    let mid_z = solid.min.z.add(solid.max.z).mul(Fx::ratio(1, 2));
    let c = &mut w.critters[0];
    c.pos = V3::new(solid.min.x.sub(Fx::from_int(2)), Fx::ZERO, mid_z);
    c.state = is::FLINCH;
    c.timer = u16::MAX;
    for _ in 0..120 {
        // Shoved into it, every frame.
        w.critters[0].vel = V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO);
        w.advance(idle());
        let at = w.critters[0].pos;
        let inside = at.x.raw() > solid.min.x.raw()
            && at.x.raw() < solid.max.x.raw()
            && at.z.raw() > solid.min.z.raw()
            && at.z.raw() < solid.max.z.raw()
            && at.y.raw() < solid.max.y.sub(Fx::ratio(1, 10)).raw();
        assert!(!inside, "a critter inside a solid at {at:?}");
    }
}

// ---------------------------------------------------------------------------
// Hitting them: the one hit test, and the overlay drawing it
// ---------------------------------------------------------------------------

/// One gnat, held still `metres` in front of a fighter, crosshair on its
/// middle.
fn one_gnat(class: Class, metres: i32) -> (World, Input) {
    let mut w = gnats(class);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    let mut c = Critter::new(
        sp,
        gnats::GNAT,
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
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(12));
    for _ in 0..2 {
        w.advance([Input::looking_at(0, 0, 0), Input::default()]);
    }
    let middle = w.critters[0].body(sp).middle();
    let pitch = aim::look_onto_closely(w.players[0].pos, 0, w.players[0].aloft, middle);
    (w, Input::looking_at(0, 0, pitch))
}

#[test]
fn the_overlay_draws_the_volume_the_hit_test_uses() {
    // Every swing in the roster that lives in `state::hitbox`, at a step and
    // at two: on every frame, the body takes damage exactly when the volume
    // the overlay draws touches its box.
    let mut frames = 0;
    for class in sim::class::ALL_CLASSES {
        for kind in 0..sim::moves::slots(class) as u8 {
            let m = sim::moves::get(class, kind);
            if m.aim() != aim::Kind::Swing || !m.strikes() || m.damage <= 0 {
                continue;
            }
            for metres in [1, 2] {
                let (mut w, look) = one_gnat(class, metres);
                let sp = w.critters.sp();
                w.press(0, kind, look);
                for _ in 0..90 {
                    let before = w.critters[0];
                    w.advance([look, Input::default()]);
                    let hurt = w.critters[0].health < before.health;
                    let drawn =
                        hitbox(&w.players[0]).is_some_and(|hb| before.body(sp).touched_by(&hb));
                    let struck_already = before.has(flag::STRUCK);
                    if !struck_already {
                        assert_eq!(
                            hurt,
                            drawn,
                            "{} {} at {metres} m: the overlay and the hit test disagree",
                            class.name(),
                            m.name
                        );
                    }
                    frames += 1;
                    if hurt {
                        break;
                    }
                }
            }
        }
    }
    assert!(frames > 100);
}

#[test]
fn every_move_touches_a_gnat_wherever_it_touches_a_fighter() {
    // The instrument (`cargo run -p sim --bin critcheck`) as an assertion:
    // stood at 1, 2, 3 and 5 m with the crosshair on its middle, a gnat 0.6 m
    // tall is touched by every move at every distance a fighter standing
    // there would be -- except where the move carried the fighter through it
    // (fighters pass through critters), and one move whose blades are flat at
    // 0.9 m by its own design. That one is a question for a person, written
    // down in `docs/design/critters.md`; a new entry here is a regression.
    const KNOWN: &[(&str, &str)] = &[("Shadow Reaver", "Guillotine")];
    let rows = sim::critcheck::table(SpeciesId::GNATS, gnats::GNAT);
    let mut over = Vec::new();
    for row in &rows {
        let misses: Vec<i32> = row.over().collect();
        if misses.is_empty() {
            continue;
        }
        if KNOWN.contains(&(row.class.name(), row.name)) {
            continue;
        }
        over.push(format!("{} {} at {misses:?} m", row.class.name(), row.name));
    }
    assert!(
        over.is_empty(),
        "passes over a gnat where a fighter is hit: {over:?}"
    );
    // And every auto touches one at some distance.
    for class in sim::class::ALL_CLASSES {
        let auto = rows
            .iter()
            .find(|r| r.class == class && r.kind == 0)
            .unwrap();
        assert!(
            auto.touches(),
            "{}'s auto never touches a gnat",
            class.name()
        );
    }
}

// ---------------------------------------------------------------------------
// A1 and A2: the aiming model, changed
// ---------------------------------------------------------------------------

#[test]
fn where_only_fighters_stand_every_aim_is_unchanged() {
    // Nothing short anywhere: "there" is a fighter's height and no swing
    // stoops. The pinned hunts (`ridgeback_pin.rs`, `hunt/tests/pin.rs`) hold
    // the rest of it bit for bit.
    let w = World::with_classes([Class::Elementalist; MAX_PLAYERS]);
    let field = sim::stones::gather(&w.players);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: w.arena(),
    };
    for pitch in [-12000i16, -6000, -2000, 0, 3000] {
        let look = Input::looking_at(0, 0, pitch);
        assert_eq!(
            aim::stands_at(0, look, &scene).height,
            sim::tuning::body_height()
        );
        assert_eq!(
            aim::stoop(look, true, aim::stands_at(0, look, &scene)),
            Fx::ZERO
        );
    }
}

#[test]
fn a_skillshot_aimed_through_a_gnat_lands_on_the_gnat() {
    for metres in [3, 6, 9] {
        let (w, look) = one_gnat(Class::Elementalist, metres);
        let field = sim::stones::gather(&w.players);
        let scene = Scene {
            stones: &field,
            players: &w.players,
            effects: &w.effects,
            quarry: &w.monsters,
            critters: &w.critters,
            arena: w.arena(),
        };
        let reach = Fx::from_int(20);
        let path = aim::skillshot_path(0, look, reach, &scene);
        // The line ends at the middle of the gnat standing there, not a
        // fighter's middle over its back.
        let middle = w.critters[0].body(w.critters.sp()).middle();
        assert!(
            path.to.y.sub(middle.y).abs().raw() < Fx::ratio(1, 50).raw(),
            "at {metres} m the shot ends {} m up",
            path.to.y.to_f32_for_render()
        );
        // And it runs into the gnat on the way (A2).
        let met = aim::first_along(
            path,
            Fx::ratio(1, 10),
            0,
            &scene,
            Targets::none().quarry(true),
        );
        assert!(
            matches!(met, Some(Contact::Critter { index: 0, .. })),
            "at {metres} m it met {met:?}"
        );
    }
}

#[test]
fn a_standing_swing_pointed_at_a_gnat_dips_and_one_pointed_over_it_stays_level() {
    let (w, look) = one_gnat(Class::Champion, 2);
    let field = sim::stones::gather(&w.players);
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: w.arena(),
    };
    let p = w.players[0];
    let reach = Fx::from_int(2);
    let at_it = aim::stands_at(0, look, &scene);
    assert_eq!(at_it.height, w.critters[0].body(w.critters.sp()).height);
    let dipped = aim::swing_path(p.pos, p.facing, look, true, reach, aim::Hand::Centre, at_it);
    assert!(
        dipped.dir().y.raw() < 0,
        "pointed at a gnat, the swing stayed level"
    );
    // Over it: a look just under the horizon, the crosshair on the floor far
    // behind it. Inside the dead zone, and nothing short under the crosshair.
    let over = Input::looking_at(0, 0, -800);
    let past = aim::stands_at(0, over, &scene);
    assert_eq!(past.height, sim::tuning::body_height());
    let level = aim::swing_path(p.pos, p.facing, over, true, reach, aim::Hand::Centre, past);
    assert_eq!(
        level.dir().y,
        Fx::ZERO,
        "pointed over the gnat, the swing dipped"
    );
    // And in the air the camera is followed exactly, short body or not.
    let airborne = aim::swing_path(
        p.pos,
        p.facing,
        look,
        false,
        reach,
        aim::Hand::Centre,
        at_it,
    );
    let plain = aim::swing_path(
        p.pos,
        p.facing,
        look,
        false,
        reach,
        aim::Hand::Centre,
        aim::Stand::fighter(),
    );
    assert_eq!(airborne, plain);
}
