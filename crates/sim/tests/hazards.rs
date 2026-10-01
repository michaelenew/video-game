//! Floor hazards (bestiary P4), stood up end to end on the dev creature, the
//! sentinel (`sim::species::sentinel`), whose kinds use every generic effect.
//!
//! What is pinned here is the machinery every hazard creature stands on: one
//! list that is both what is drawn and what is felt, the slow and the dodge
//! and the jump, the root, the pull, damage over time, fire lighting tar and
//! running along it, burnt tar becoming a solid that bodies stand on and the
//! aiming ray meets, smoke that blocks sight, a vent carried by the part it
//! lives on, a strand that is a line, and the list's bounds. See
//! `docs/design/hazards.md`.

use sim::aim::{self, Scene};
use sim::arena::{ArenaId, Terrain};
use sim::fixed::Fx;
use sim::hazard::{self, Hazard, HazardField, reach};
use sim::lore::CELLS;
use sim::monster::MAX_MONSTERS;
use sim::species::{SpeciesId, ridgeback, sentinel};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

const RIGHT: u16 = 0;

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

/// A sentinel hunt with nothing on the floor, the creature far off and
/// taking no notice: a floor to put hazards on and walk across.
fn floor(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::SENTINEL);
    w.lore.set_word(sentinel::word::LAID, 1);
    quiet(&mut w);
    // Along the open lane at z = 10, clear of the two platforms.
    w.players[0].pos = at(-10, 10);
    w.players[1].pos = at(-10, -10);
    w
}

/// The creature parked in a corner and still noticing nobody.
fn quiet(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.pos = at(10, -10);
        m.brain.grace = u16::MAX;
    }
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        quiet(w);
        w.advance([Input::aimed(bits, RIGHT), Input::default()]);
        for p in w.players.iter_mut() {
            p.health = p.full_health();
        }
    }
}

fn place(w: &mut World, h: Hazard) -> usize {
    hazard::place(&mut w.lore, h).expect("room for it")
}

fn sp() -> &'static sim::species::Species {
    &sentinel::SPECIES
}

/// How far fighter zero gets walking right for `frames`.
fn walked(w: &mut World, frames: u32) -> Fx {
    let from = w.players[0].pos.x;
    run(w, frames, Input::W);
    w.players[0].pos.x.sub(from)
}

// ---------------------------------------------------------------------------
// One list
// ---------------------------------------------------------------------------

#[test]
fn what_is_drawn_on_the_floor_is_the_hazard_list() {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SENTINEL);
    // The sentinel lays its demo floor on the first frame of the round.
    w.advance([Input::default(); MAX_PLAYERS]);
    let listed: Vec<(usize, Hazard)> = hazard::all(&w.lore).collect();
    assert!(listed.len() >= 5, "the demo floor has one of each kind");
    let placed: Vec<_> = w.terrain().floor.iter().copied().collect();
    assert_eq!(listed.len(), placed.len());
    for ((slot, h), p) in listed.iter().zip(placed.iter()) {
        assert_eq!(*slot, p.slot as usize);
        assert_eq!(h.index(), p.kind);
        assert_eq!(h.radius(), p.radius);
        if h.anchor == hazard::NO_ANCHOR {
            assert_eq!(h.centre(), p.a);
        }
    }
}

#[test]
fn a_hazard_is_one_cell_and_round_trips() {
    let mut w = floor(Class::Champion);
    let h = Hazard::strand(sentinel::STRAND, at(-3, 2), at(5, -7), Fx::ratio(3, 10));
    let slot = place(&mut w, h);
    assert_eq!(hazard::get(&w.lore, slot), h);
    let d = Hazard::disc(
        sentinel::TAR,
        V3::new(Fx::from_int(-120), Fx::ratio(3, 2), Fx::from_int(24)),
        Fx::from_int(12),
    );
    let slot = place(&mut w, d);
    let back = hazard::get(&w.lore, slot);
    assert_eq!(
        back.centre(),
        d.centre(),
        "a centre at the far end of the valley is kept to the centimetre"
    );
    assert_eq!(back.radius(), Fx::from_int(12));
}

#[test]
fn the_list_never_holds_more_than_its_layout_and_a_cap_takes_the_oldest() {
    let mut w = floor(Class::Champion);
    let room = hazard::room(&w.lore);
    assert_eq!(room, sentinel::FIGHT.layout.hazards as usize);
    assert!(room <= hazard::MAX_HAZARDS);
    assert!(room <= CELLS);
    for i in 0..room + 4 {
        hazard::place(
            &mut w.lore,
            Hazard::disc(sentinel::TAR, at(i as i32 - 8, 0), Fx::ONE),
        );
        run(&mut w, 1, 0);
    }
    assert_eq!(
        hazard::all(&w.lore).count(),
        room,
        "full, and the extra four replaced the oldest"
    );
    // Slag is capped at six: a seventh takes the oldest's place.
    let mut w = floor(Class::Champion);
    for i in 0..7 {
        let mut s = Hazard::disc(sentinel::SLAG, at(i * 3 - 9, 9), Fx::ONE);
        s.state = 1;
        place(&mut w, s);
        run(&mut w, 1, 0);
    }
    let slag = hazard::all(&w.lore)
        .filter(|(_, h)| h.index() == sentinel::SLAG)
        .count();
    assert_eq!(slag, 6);
}

// ---------------------------------------------------------------------------
// What it does to a body
// ---------------------------------------------------------------------------

#[test]
fn tar_slows_the_walk_and_shortens_the_dodge_but_keeps_its_invulnerability() {
    let mut clear = floor(Class::Champion);
    let free = walked(&mut clear, 40);

    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(-6, 10), Fx::from_int(6)),
    );
    let slowed = walked(&mut w, 40);
    let want = hazard::stat_fx(sp(), sentinel::TAR, HazardField::Walk);
    assert!(
        slowed.raw() < free.mul(want).add(Fx::ratio(1, 2)).raw() && slowed.raw() > 0,
        "in tar it walked {slowed:?}, on clear floor {free:?}"
    );

    // A dodge from clear floor and from inside the tar.
    let dodge = |w: &mut World| {
        let from = w.players[0].pos.x;
        run(w, 1, Input::SHIFT | Input::W);
        let invulnerable = w.players[0].action.invulnerable();
        run(w, 30, 0);
        (w.players[0].pos.x.sub(from), invulnerable)
    };
    let mut clear = floor(Class::Champion);
    run(&mut clear, 30, 0);
    let (far, _) = dodge(&mut clear);
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(-10, 10), Fx::from_int(3)),
    );
    run(&mut w, 30, 0);
    let (near, invulnerable) = dodge(&mut w);
    assert!(invulnerable, "a dodge out of tar is still a dodge");
    assert!(
        near.raw() < far.mul(Fx::ratio(3, 4)).raw(),
        "the dodge from tar went {near:?}, from clear floor {far:?}"
    );
}

#[test]
fn a_snare_roots_the_walk_the_dodge_and_the_jump() {
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::SNARE, at(-10, 10), Fx::from_int(2)),
    );
    run(&mut w, 5, 0);
    let moved = walked(&mut w, 30);
    assert!(
        moved.abs().raw() < Fx::ratio(1, 10).raw(),
        "rooted, it still walked {moved:?}"
    );
    run(&mut w, 1, Input::SPACE);
    run(&mut w, 5, 0);
    assert!(
        w.players[0].pos.y.raw() < Fx::ratio(1, 10).raw(),
        "rooted, it still jumped"
    );
}

#[test]
fn a_sinkhole_pulls_a_body_on_the_floor_and_not_one_in_the_air() {
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::SINKHOLE, at(-7, 10), Fx::from_int(4)),
    );
    run(&mut w, 30, 0);
    assert!(
        w.players[0].pos.x.raw() > Fx::from_int(-9).raw(),
        "standing still in it, it was pulled toward the middle"
    );

    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::SINKHOLE, at(-7, 10), Fx::from_int(4)),
    );
    run(&mut w, 1, Input::SPACE);
    let x = w.players[0].pos.x;
    run(&mut w, 6, Input::SPACE);
    assert!(!w.players[0].grounded);
    assert_eq!(w.players[0].pos.x, x, "in the air nothing pulled it");
}

#[test]
fn burning_tar_hurts_whoever_stands_in_it_and_the_creature_too() {
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::disc(sentinel::BURNING, at(-10, 10), Fx::from_int(2)),
    );
    let creature_at = w.monsters[0].unwrap().pos;
    place(
        &mut w,
        Hazard::disc(sentinel::BURNING, creature_at, Fx::from_int(3)),
    );
    let before = w.monsters[0].unwrap().health;
    let mut hurt = 0;
    for _ in 0..30 {
        quiet(&mut w);
        let was = w.players[0].health;
        w.advance([Input::default(); MAX_PLAYERS]);
        hurt += was - w.players[0].health;
    }
    let tick = hazard::stat(sp(), sentinel::BURNING, HazardField::Damage);
    assert!(hurt >= tick * 2, "thirty frames in fire took {hurt}");
    assert!(
        w.monsters[0].unwrap().health < before,
        "the fire under the creature burned it"
    );
}

// ---------------------------------------------------------------------------
// Fire, and what it leaves
// ---------------------------------------------------------------------------

#[test]
fn fire_runs_along_tar_that_touches_and_stops_where_it_does_not() {
    let mut w = floor(Class::Champion);
    let a = place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(0, 6), Fx::from_int(2)),
    );
    let b = place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(3, 6), Fx::from_int(2)),
    );
    let c = place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(6, 6), Fx::from_int(2)),
    );
    let apart = place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(12, 6), Fx::from_int(2)),
    );
    // Light the first.
    let mut lit = hazard::get(&w.lore, a);
    lit.kind = sentinel::BURNING + 1;
    hazard::set(&mut w.lore, a, lit);
    run(&mut w, 40, 0);
    for slot in [b, c] {
        assert_ne!(
            hazard::get(&w.lore, slot).index(),
            sentinel::TAR,
            "touching tar caught"
        );
    }
    assert_eq!(
        hazard::get(&w.lore, apart).index(),
        sentinel::TAR,
        "tar three metres off did not"
    );
}

#[test]
fn a_fire_pillar_lights_tar() {
    let mut w = floor(Class::Elementalist);
    let slot = place(
        &mut w,
        Hazard::disc(sentinel::TAR, at(-10, 10), Fx::from_int(3)),
    );
    // Fire as the world counts it: a pillar standing in the tar.
    let fire = [hazard::Fire {
        at: at(-9, 10),
        radius: Fx::ONE,
    }];
    hazard::step(&mut w.lore, &w.monsters, &fire);
    assert_eq!(hazard::get(&w.lore, slot).index(), sentinel::BURNING);
}

#[test]
fn a_burnt_pool_leaves_slag_that_bodies_stand_on_and_the_ray_meets() {
    let mut w = floor(Class::Champion);
    let slot = place(
        &mut w,
        Hazard::disc(sentinel::BURNING, at(0, 8), Fx::from_int(3)),
    );
    let life = hazard::stat(sp(), sentinel::BURNING, HazardField::Life);
    run(&mut w, life as u32 + 2, 0);
    let slag = hazard::get(&w.lore, slot);
    assert_eq!(slag.index(), sentinel::SLAG);
    assert_eq!(slag.state, 1, "one layer");
    assert_eq!(
        slag.radius(),
        hazard::stat_fx(sp(), sentinel::SLAG, HazardField::Radius)
    );

    let ground = w.terrain();
    assert_eq!(ground.raised().len(), 1);
    let top = hazard::stat_fx(sp(), sentinel::SLAG, HazardField::Solid);
    assert_eq!(
        ground.ground_under(at(0, 8)),
        top,
        "a body over it stands on it"
    );
    // A body walking into it is stopped by it, as by any wall.
    let r = ground.resolve(
        at(0, 8).add(V3::new(Fx::ZERO, Fx::ratio(1, 2), Fx::ZERO)),
        V3::ZERO,
        true,
    );
    assert!(r.pos.y.raw() >= top.raw() || r.wall);
    // And the aiming ray meets it.
    let fighters = w.players;
    let effects = w.effects;
    let scene = Scene {
        stones: &sim::stones::gather(&fighters),
        players: &fighters,
        effects: &effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    assert!(!aim::line_clear(
        at(-4, 8).add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)),
        at(4, 8).add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)),
        &scene
    ));
}

#[test]
fn slag_burnt_again_is_a_layer_higher() {
    let mut w = floor(Class::Champion);
    let mut s = Hazard::disc(sentinel::SLAG, at(0, 8), Fx::ONE);
    s.state = 1;
    let first = place(&mut w, s);
    place(
        &mut w,
        Hazard::disc(sentinel::BURNING, at(0, 8), Fx::from_int(2)),
    );
    let life = hazard::stat(sp(), sentinel::BURNING, HazardField::Life);
    run(&mut w, life as u32 + 2, 0);
    let slag: Vec<_> = hazard::all(&w.lore)
        .filter(|(_, h)| h.index() == sentinel::SLAG)
        .collect();
    assert_eq!(slag.len(), 1, "the new mound joined the old one");
    assert_eq!(slag[0].0, first);
    assert_eq!(slag[0].1.state, 2);
    let layer = hazard::stat_fx(sp(), sentinel::SLAG, HazardField::Solid);
    assert_eq!(
        w.terrain().ground_under(at(0, 8)),
        layer.mul(Fx::from_int(2))
    );
}

// ---------------------------------------------------------------------------
// Sight, anchors, lines
// ---------------------------------------------------------------------------

#[test]
fn smoke_blocks_sight_and_fire_burns_it_off() {
    let mut w = floor(Class::Champion);
    let slot = place(
        &mut w,
        Hazard::disc(sentinel::SMOKE, at(0, 0), Fx::from_int(3)),
    );
    let ground = w.terrain();
    let fighters = w.players;
    let effects = w.effects;
    let field = sim::stones::gather(&fighters);
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let eye = |x: i32| V3::new(Fx::from_int(x), Fx::from_int(2), Fx::ZERO);
    assert!(
        !aim::sight_clear(eye(-8), eye(8), &scene),
        "through the cloud"
    );
    assert!(
        aim::line_clear(eye(-8), eye(8), &scene),
        "nothing solid there"
    );
    let past = |x: i32| V3::new(Fx::from_int(x), Fx::from_int(2), Fx::from_int(6));
    assert!(aim::sight_clear(past(-8), past(8), &scene), "beside it");
    let over = |x: i32| V3::new(Fx::from_int(x), Fx::from_int(6), Fx::ZERO);
    assert!(aim::sight_clear(over(-8), over(8), &scene), "over its top");

    let fire = [hazard::Fire {
        at: at(0, 0),
        radius: Fx::ONE,
    }];
    hazard::step(&mut w.lore, &w.monsters, &fire);
    assert_eq!(hazard::get(&w.lore, slot).index(), sentinel::FLASH);
    let flash = hazard::stat(sp(), sentinel::FLASH, HazardField::Life);
    run(&mut w, flash as u32 + 1, 0);
    assert!(!hazard::get(&w.lore, slot).present(), "burned off");
}

#[test]
fn a_vent_moves_with_the_part_it_is_on() {
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::on_part(sentinel::VENT, 0, ridgeback::RIDGE, V3::ZERO, Fx::ONE),
    );
    let where_now = |w: &World| w.terrain().floor.iter().next().unwrap().a;
    let beast = w.monsters[0].unwrap();
    assert_eq!(where_now(&w), beast.world_of(ridgeback::RIDGE, V3::ZERO));
    // Move the creature and turn it: the vent goes with the ridge.
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at(3, 2);
    m.yaw = Fx::ratio(1, 8);
    let beast = w.monsters[0].unwrap();
    assert_eq!(where_now(&w), beast.world_of(ridgeback::RIDGE, V3::ZERO));
    assert!(
        where_now(&w).y.raw() > Fx::from_int(3).raw(),
        "up on its back"
    );
}

#[test]
fn a_strand_is_a_line_on_the_floor() {
    let mut w = floor(Class::Champion);
    place(
        &mut w,
        Hazard::strand(sentinel::STRAND, at(-5, 0), at(5, 0), Fx::ratio(3, 10)),
    );
    let ground = w.terrain();
    let feel = |x: i32, z10: i32| {
        ground
            .floor
            .underfoot(
                V3::new(Fx::from_int(x), Fx::ZERO, Fx::ratio(z10, 10)),
                reach::FIGHTERS,
                true,
                0,
            )
            .walk
    };
    let slow = hazard::stat_fx(sp(), sentinel::STRAND, HazardField::Walk);
    assert_eq!(feel(0, 0), slow);
    assert_eq!(feel(4, 2), slow);
    assert_eq!(feel(0, 5), Fx::ONE, "half a metre off the line");
    assert_eq!(feel(7, 0), Fx::ONE, "past its end");
}

// ---------------------------------------------------------------------------
// It costs nothing where it is not used
// ---------------------------------------------------------------------------

#[test]
fn a_fight_without_hazards_has_no_floor_and_a_blank_lore() {
    for id in [SpeciesId::RIDGEBACK, SpeciesId::GNATS] {
        let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], id);
        for _ in 0..60 {
            w.advance([Input::aimed(Input::W, RIGHT); MAX_PLAYERS]);
        }
        assert!(w.terrain().floor.is_empty());
        assert!(w.terrain().raised().is_empty());
        assert!(w.lore.is_blank());
    }
    let versus = World::with_classes([Class::Champion; MAX_PLAYERS]);
    assert!(versus.lore.owner.is_none());
    let bare = Terrain::bare(ArenaId::PROVING_GROUND.get());
    assert!(bare.floor.is_empty());
    let _ = MAX_MONSTERS;
}

#[test]
fn a_hazard_on_the_floor_is_in_the_hash() {
    let a = floor(Class::Champion);
    let mut b = floor(Class::Champion);
    assert_eq!(a.state_checksum(), b.state_checksum());
    place(&mut b, Hazard::disc(sentinel::TAR, at(0, 0), Fx::ONE));
    assert_ne!(a.state_checksum(), b.state_checksum());
}
