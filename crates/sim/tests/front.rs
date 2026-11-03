//! **Five points in a row** (2026-11-03, `sim::versus::Front`,
//! `docs/design/versus.md` §1): the rules of a five-point map, one at a
//! time, on the Kilnworks -- read off its line, so they hold however the
//! map is built.

use sim::arena::{ArenaId, Mark};
use sim::class::Class;
use sim::fixed::Fx;
use sim::math::V3;
use sim::state::Phase;
use sim::tuning as t;
use sim::versus::{self, Front, NOBODY, POINTS};
use sim::{Input, World};

const ID: ArenaId = ArenaId::KILNWORKS;

fn idle<const N: usize>() -> [Input; N] {
    [Input::default(); N]
}

fn line() -> &'static versus::Line {
    versus::of(ID).and_then(|s| s.front).expect("a line")
}

/// Stand fighter `i` on whatever is under `at`.
fn stand(w: &mut World, i: usize, at: V3) {
    let ground = w
        .terrain()
        .ground_under(V3::new(at.x, at.y.add(Fx::ONE), at.z));
    w.players[i].pos = V3::new(at.x, ground, at.z);
    w.players[i].vel = V3::ZERO;
}

/// Stand fighter `i` on point `k`.
fn on_point(w: &mut World, i: usize, k: usize) {
    stand(w, i, line().points[k].at);
}

/// Stand fighter `i` back at their side's own start, on no point.
fn home(w: &mut World, i: usize) {
    let mark = Mark::seat(&w.arena().spawns.versus, i);
    stand(w, i, mark.at);
}

/// A two-a-side game with the lock over and everybody at home.
fn open_game() -> World {
    let mut w = World::versus_in([Class::Champion; 4], ID);
    w.front.locked = 0;
    for i in 0..4 {
        home(&mut w, i);
    }
    w
}

/// Advance `frames`, keeping the fighters where they were put: nobody
/// presses anything, so nobody moves.
fn run(w: &mut World, frames: u32) {
    for _ in 0..frames {
        w.advance(idle::<4>());
    }
}

#[test]
fn a_game_starts_with_two_points_each_and_the_middle_locked() {
    let w = World::versus_in([Class::Champion; 2], ID);
    assert!(w.front.on);
    assert!(!w.hill.on);
    assert_eq!(w.front.owner, [0, 0, NOBODY, 1, 1]);
    assert_eq!(w.front.locked, t::front_unlock());
    assert_eq!(w.front.clock, t::front_clock());
    // Nowhere else plays it.
    assert!(
        !World::versus_in([Class::Champion; 2], ArenaId::QUARRY)
            .front
            .on
    );
    assert_eq!(Front::start(ArenaId::PROVING_GROUND), Front::NONE);
}

#[test]
fn nothing_is_taken_while_the_middle_is_locked() {
    let mut w = World::versus_in([Class::Champion; 2], ID);
    on_point(&mut w, 0, 2);
    home(&mut w, 1);
    for _ in 0..t::front_unlock() {
        w.advance(idle::<2>());
    }
    assert_eq!(w.front.progress[2], 0);
    assert_eq!(w.front.owner[2], NOBODY);
}

#[test]
fn the_middle_is_taken_and_the_clock_given_back() {
    let mut w = open_game();
    w.front.clock = 1000;
    on_point(&mut w, 0, 2);
    run(&mut w, t::hill_capture() as u32);
    assert_eq!(w.front.owner[2], 0, "{:?}", w.front);
    // A capture gives time back, up to the round's whole length.
    assert!(w.front.clock > 1000 - t::hill_capture() as u32);
}

#[test]
fn only_the_next_point_along_a_line_can_be_taken() {
    let mut w = open_game();
    // The west side on the east side's second point, the middle still
    // nobody's: not next along their line, so nothing happens.
    on_point(&mut w, 0, 3);
    run(&mut w, t::hill_capture() as u32 + 30);
    assert_eq!(w.front.owner[3], 1);
    assert_eq!(w.front.progress[3], 0);
    assert_eq!(w.front.standing[3], [0, 0]);
    // With the middle theirs, it is next, and it falls.
    w.front.owner[2] = 0;
    run(&mut w, t::hill_capture() as u32);
    assert_eq!(w.front.owner[3], 0, "{:?}", w.front);
}

#[test]
fn a_side_can_take_back_the_point_it_lost() {
    let mut w = open_game();
    w.front.owner[2] = 0;
    // The middle is the west side's, next to the east's second point, so
    // it is open to the east side.
    assert!(w.front.open(2, 1));
    assert!(!w.front.open(1, 1));
    on_point(&mut w, 1, 2);
    run(&mut w, t::hill_capture() as u32);
    assert_eq!(w.front.owner[2], 1, "{:?}", w.front);
}

#[test]
fn both_sides_on_a_point_is_contested_and_defenders_push_back() {
    let mut w = open_game();
    on_point(&mut w, 0, 2);
    run(&mut w, 60);
    let part = w.front.progress[2];
    assert!(part > 0);
    // The other side steps on: nothing moves.
    on_point(&mut w, 1, 2);
    run(&mut w, 60);
    assert_eq!(w.front.progress[2], part);
    // A point's owners alone on it push a capture back.
    w.front.owner[3] = 1;
    w.front.owner[2] = 0;
    w.front.progress = [0; POINTS];
    home(&mut w, 0);
    on_point(&mut w, 1, 2);
    run(&mut w, 60);
    on_point(&mut w, 0, 2);
    home(&mut w, 1);
    // At the rate it was built: sixty frames of it take sixty back.
    run(&mut w, 30);
    assert!(w.front.progress[2] > 0 && w.front.owner[2] == 0);
    run(&mut w, 31);
    assert_eq!(w.front.capper[2], NOBODY, "{:?}", w.front);
    assert_eq!(w.front.progress[2], 0);
}

#[test]
fn owning_all_five_wins_the_round() {
    let mut w = open_game();
    w.front.owner = [0, 0, 0, 0, 1];
    on_point(&mut w, 0, 4);
    run(&mut w, t::hill_capture() as u32 + 1);
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "{:?}",
        w.phase
    );
    assert_eq!(w.players[0].rounds_won, 1);
    assert_eq!(w.players[2].rounds_won, 1);
    assert_eq!(w.players[1].rounds_won, 0);
}

#[test]
fn the_clock_running_out_goes_to_whoever_holds_more() {
    let mut w = open_game();
    w.front.owner = [0, 0, 0, 1, 1];
    w.front.clock = 2;
    run(&mut w, 3);
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "{:?}",
        w.phase
    );
    // Level, nobody's.
    let mut w = open_game();
    w.front.clock = 2;
    run(&mut w, 3);
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: NOBODY, .. }),
        "{:?}",
        w.phase
    );
}

#[test]
fn a_capture_under_way_holds_the_clock_open() {
    let mut w = open_game();
    w.front.clock = 2;
    on_point(&mut w, 0, 2);
    run(&mut w, 10);
    assert!(matches!(w.phase, Phase::Fighting), "{:?}", w.phase);
    assert_eq!(w.front.clock, 0);
    assert!(w.front.progress[2] > 0);
}

#[test]
fn the_dead_stand_back_up_a_point_behind_their_front() {
    assert_eq!(Front::start(ID).spawn_at(0), 0);
    assert_eq!(Front::start(ID).spawn_at(1), POINTS - 1);
    let mut w = open_game();
    // The west side has taken the middle: its spawn moves up to its second
    // point.
    w.front.owner[2] = 0;
    assert_eq!(w.front.spawn_at(0), 1);
    assert_eq!(w.front.spawn_at(1), 4);
    w.players[2].health = 0;
    run(&mut w, t::hill_respawn() as u32 + 2);
    assert!(w.players[2].health > 0);
    let at = Mark::seat(&[line().spawn(0, 1), line().spawn(1, 4)], 2).at;
    let p = w.players[2].pos;
    assert!(
        V3::new(p.x.sub(at.x), Fx::ZERO, p.z.sub(at.z))
            .flat_len()
            .raw()
            < Fx::ONE.raw(),
        "stood up at {p:?}, not {at:?}"
    );
    // The east side's spawn is the west side's turned.
    let east = line().spawn(1, 3);
    let west = line().spawn(0, 1);
    assert_eq!(east.at.x, west.at.x.neg());
    assert_eq!(east.at.z, west.at.z.neg());
}

#[test]
fn a_side_cannot_hurt_its_own_on_a_five_point_map() {
    let w = World::versus_in([Class::Champion; 4], ID);
    assert!(w.teams());
    assert!(!w.foes(0, 2));
    assert!(w.foes(0, 1));
}

#[test]
fn a_five_point_game_is_deterministic() {
    let play = || {
        let mut w = open_game();
        on_point(&mut w, 0, 2);
        on_point(&mut w, 1, 2);
        let mut sums = Vec::new();
        for f in 0..600u32 {
            if f == 200 {
                home(&mut w, 1);
            }
            w.advance(idle::<4>());
            sums.push(w.checksum());
        }
        sums
    };
    assert_eq!(play(), play());
}
