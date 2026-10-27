//! **Four fighters** (2026-10-14). A fight seats one to four; the seats past
//! those are parked out of the world. Versus with three or four is everybody
//! against everybody, last one standing; a hunt takes up to four hunters.
//! See `docs/design/architecture.md` §"Four fighters".

use sim::arena::{ArenaId, Mark};
use sim::class::Class;
use sim::effects::{Effect, LOTUS_BLADES, MAX_EFFECTS, VICTIMS, quarry_victim};
use sim::fixed::Fx;
use sim::species::SpeciesId;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Input, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

const FOUR: [Class; 4] = [
    Class::Bulwark,
    Class::Champion,
    Class::BloodMage,
    Class::Elementalist,
];

/// Eight since 2026-10-26: the fight the open world is sized for
/// (exploration 0029, NET-2). This file's fights are still four.
#[test]
fn eight_seats_is_the_most_there_is() {
    assert_eq!(MAX_PLAYERS, 8);
}

#[test]
fn a_fight_seats_as_many_as_it_names() {
    assert_eq!(World::with_classes([Class::Bulwark; 2]).present(), 2);
    assert_eq!(World::with_classes([Class::Bulwark; 3]).present(), 3);
    assert_eq!(World::with_classes(FOUR).present(), 4);
    assert_eq!(World::hunt([Class::Bulwark; 4]).present(), 4);
    // Versus keeps two bodies whatever is asked: one seat is somebody
    // training against the dummy.
    assert_eq!(
        World::with_classes([Class::Bulwark; 2]).seated(1).present(),
        2
    );
    // A hunt takes the seat out.
    assert_eq!(World::hunt([Class::Bulwark; 2]).seated(1).present(), 1);
    let w = World::with_classes(FOUR);
    assert_eq!(w.players.map(|p| p.class)[..FOUR.len()], FOUR);
}

#[test]
fn every_arena_stands_four_fighters_apart() {
    // Two marks a place, and the third and fourth beside them: four distinct
    // spots, none of them on top of another.
    for a in sim::arena::all() {
        let w = World::versus_in(FOUR, a.id);
        for i in 0..4 {
            for j in i + 1..4 {
                let gap = w.players[i].pos.sub(w.players[j].pos).flat_len();
                assert!(
                    gap.raw() > sim::tuning::body_radius().raw(),
                    "{}: fighters {i} and {j} start on top of each other",
                    a.name
                );
            }
        }
    }
}

#[test]
fn the_third_and_fourth_marks_flank_the_first_two() {
    let marks = [
        Mark::cm(-200, -100, (-200, 100)),
        Mark::cm(200, -100, (200, 100)),
    ];
    assert_eq!(Mark::seat(&marks, 0), marks[0]);
    assert_eq!(Mark::seat(&marks, 1), marks[1]);
    // Side by side, facing the same way: the line widens outward.
    let third = Mark::seat(&marks, 2);
    let fourth = Mark::seat(&marks, 3);
    assert!(third.at.x.raw() < marks[0].at.x.raw(), "{third:?}");
    assert!(fourth.at.x.raw() > marks[1].at.x.raw(), "{fourth:?}");
    assert_eq!(third.facing, marks[0].facing);
}

#[test]
fn a_seat_nobody_plays_is_out_of_the_world_and_stays_there() {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    for _ in 0..120 {
        w.advance([Input::aimed(Input::W, LOOK_RIGHT), Input::default()]);
    }
    for p in &w.players[2..] {
        assert_eq!(p.health, 0);
        assert!(p.pos.y.raw() < Fx::from_int(-100).raw(), "{:?}", p.pos);
    }
    // An input for a seat nobody is in changes nothing.
    let mut a = w.clone();
    let mut b = w.clone();
    a.advance([Input::default(); 4]);
    b.advance([
        Input::default(),
        Input::default(),
        Input::aimed(Input::LEFT | Input::W, LOOK_LEFT),
        Input::aimed(Input::SPACE, LOOK_RIGHT),
    ]);
    assert_eq!(a.checksum(), b.checksum());
}

#[test]
fn versus_with_four_is_the_last_one_standing() {
    let mut w = World::with_classes(FOUR);
    w.players[0].health = 0;
    w.advance([Input::default(); 4]);
    assert!(matches!(w.phase, Phase::Fighting), "three left is a fight");
    w.players[3].health = 0;
    w.advance([Input::default(); 4]);
    assert!(matches!(w.phase, Phase::Fighting), "two left is a fight");
    w.players[1].health = 0;
    w.advance([Input::default(); 4]);
    match w.phase {
        Phase::RoundOver { winner, .. } => assert_eq!(winner, 2),
        other => panic!("one standing should end the round: {other:?}"),
    }
    assert_eq!(w.players[2].rounds_won, 1);
    assert_eq!(w.players[0].rounds_won + w.players[1].rounds_won, 0);
}

#[test]
fn a_double_knockout_of_the_last_two_is_nobodys() {
    let mut w = World::with_classes([Class::Bulwark; 3]);
    w.players[0].health = 0;
    w.advance([Input::default(); 3]);
    w.players[1].health = 0;
    w.players[2].health = 0;
    w.advance([Input::default(); 3]);
    assert!(matches!(
        w.phase,
        Phase::RoundOver {
            winner: u8::MAX,
            ..
        }
    ));
}

#[test]
fn a_blow_lands_on_whoever_is_in_front_of_it() {
    // Fighter one walks up to the mark across and swings; the body standing
    // there is the third fighter's, not the second's.
    let mut w = World::with_classes([Class::Bulwark; 3]);
    let across = w.players[1];
    w.players[1].pos = w.players[1]
        .pos
        .add(sim::V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(6)));
    w.players[2].pos = across.pos;
    w.players[2].facing = across.facing;
    let before = w.players[2].health;
    for _ in 0..90 {
        w.advance([
            Input::aimed(Input::W, LOOK_RIGHT),
            Input::default(),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    for _ in 0..20 {
        w.advance([
            Input::aimed(Input::LEFT, LOOK_RIGHT),
            Input::default(),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    assert!(
        w.players[2].health < before,
        "the third fighter took nothing"
    );
    assert_eq!(w.players[1].health, w.players[1].full_health());
}

#[test]
fn four_hunters_hunt_together() {
    let mut w = World::hunt_of([Class::Champion; 4], SpeciesId::RIDGEBACK);
    for _ in 0..60 {
        w.advance([Input::default(); 4]);
    }
    assert!(w.hunting());
    for p in &w.players[..4] {
        assert!(p.health > 0, "a hunter is down before anything happened");
    }
    // The hunters cannot hurt each other: friendly fire is off with a
    // creature in the arena, for four as for two.
    assert!(!w.pvp());
}

#[test]
fn every_part_against_every_victim_has_its_own_bit() {
    // Twelve blades against four fighters and two creatures: no two share a
    // bit, so a blade never goes quiet because another one landed.
    let mut e = Effect::pool(0, Class::BloodMage, 0, sim::V3::ZERO, 0);
    let victims: Vec<usize> = (0..VICTIMS).chain([quarry_victim(1)]).collect();
    for part in 0..LOTUS_BLADES {
        for &v in &victims {
            e.forget_hits();
            e.take_hit(part, v);
            for other_part in 0..LOTUS_BLADES {
                for &other in &victims {
                    let same = (part, v) == (other_part, other)
                        || v >= VICTIMS && other >= VICTIMS && part == other_part;
                    assert_eq!(
                        e.already_hit(other_part, other),
                        same,
                        "part {part} on {v} is read as part {other_part} on {other}"
                    );
                }
            }
        }
    }
    // And a pool drunk from is remembered in every slot there is.
    for slot in 0..MAX_EFFECTS {
        e.forget_hits();
        e.mark_drank(slot);
        assert!(e.drank_from(slot));
        assert!(!e.drank_from((slot + 1) % MAX_EFFECTS));
    }
}

#[test]
fn the_effect_pool_is_large() {
    const { assert!(MAX_EFFECTS >= 128) };
}

#[test]
fn a_four_way_fight_replays_exactly() {
    let script: Vec<[Input; 4]> = (0..900u32)
        .map(|f| {
            std::array::from_fn(|i| {
                let r = (f.wrapping_mul(0x9E37_79B9) ^ (i as u32).wrapping_mul(0x85EB_CA6B))
                    .rotate_left(i as u32 * 7);
                Input::aimed((r & 0x1ff) as u16, (r >> 16) as u16)
            })
        })
        .collect();
    let run = || {
        let mut w = World::versus_in(FOUR, ArenaId::PROVING_GROUND);
        for i in &script {
            w.advance(*i);
        }
        w
    };
    let (a, b) = (run(), run());
    assert_eq!(a.checksum(), b.checksum());
    assert_eq!(a.players, b.players);
}
