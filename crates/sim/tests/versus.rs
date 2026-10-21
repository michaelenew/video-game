//! **Versus modes** (2026-10-21, `sim::versus`, `docs/design/versus.md`):
//! floating duel stages you can be knocked off, and King of the Hill.

use sim::arena::{self, ArenaId};
use sim::class::Class;
use sim::fixed::Fx;
use sim::math::V3;
use sim::state::Phase;
use sim::tuning as t;
use sim::versus::{self, Kind, NOBODY};
use sim::{Input, World};

/// Nobody pressing anything, for `N` seats.
fn idle<const N: usize>() -> [Input; N] {
    [Input::default(); N]
}

fn stand(w: &mut World, i: usize, at: V3) {
    let ground = w
        .terrain()
        .ground_under(V3::new(at.x, at.y.add(Fx::ONE), at.z));
    w.players[i].pos = V3::new(at.x, ground, at.z);
    w.players[i].vel = V3::ZERO;
}

/// Somewhere well off the point, on the floor: each side's own start.
fn home(w: &mut World, i: usize) {
    let mark = arena::Mark::seat(&w.arena().spawns.versus, i);
    stand(w, i, mark.at);
}

fn hill_of(id: ArenaId) -> versus::Hill {
    versus::of(id).and_then(|s| s.hill).expect("a hill")
}

#[test]
fn every_stage_is_registered_and_named_for_what_it_is() {
    assert_eq!(versus::STAGES.len(), 9);
    let duels = versus::STAGES.iter().filter(|s| s.kind == Kind::Duel);
    let hills = versus::STAGES.iter().filter(|s| s.kind == Kind::Hill);
    assert_eq!(duels.clone().count(), 4);
    assert_eq!(hills.clone().count(), 5);
    for s in versus::STAGES {
        let a = arena::lookup(s.arena).expect("registered");
        assert!(a.creature.is_none(), "{}", a.name);
        assert_eq!(arena::named(&a.slug()).map(|x| x.id), Some(s.arena));
        assert!(!s.theme.is_empty());
    }
    // A duel stage floats over a pit and has no point; a hill is the other
    // way round, and stands on ground with a rim for its edge.
    for s in duels {
        assert!(s.pit.is_some() && s.hill.is_none());
        assert!(s.arena.get().rim.is_none());
    }
    for s in hills {
        assert!(s.pit.is_none() && s.hill.is_some());
        assert!(s.arena.get().rim.is_some());
    }
    // Stepping through them visits every one and comes back round.
    let mut at = versus::after(ArenaId::PROVING_GROUND);
    for _ in 0..versus::STAGES.len() {
        at = versus::after(at.arena);
    }
    assert_eq!(at.arena, versus::STAGES[0].arena);
}

#[test]
fn a_floating_stage_hangs_wholly_above_its_pit_and_starts_you_on_it() {
    for s in versus::STAGES.iter().filter(|s| s.kind == Kind::Duel) {
        let a = s.arena.get();
        let pit = s.pit().unwrap();
        for solid in a.solids() {
            assert!(
                solid.min.y.raw() > pit.raw() && solid.hangs(),
                "{}: {solid:?} reaches down past the pit",
                a.name
            );
        }
        let w = World::versus_in([Class::Champion; 4], s.arena);
        for i in 0..4 {
            let p = w.players[i];
            assert!(
                p.pos.y.raw() > pit.raw(),
                "{}: seat {i} starts below the pit",
                a.name
            );
            // On the island, not over the drop.
            let under = w.terrain().ground_under(p.pos);
            assert_eq!(under, p.pos.y, "{}: seat {i}", a.name);
        }
    }
}

#[test]
fn knocked_off_a_floating_stage_is_out() {
    let mut w = World::versus_in([Class::Champion, Class::Bulwark], ArenaId::ANVIL);
    // Off the edge, in the air beside the island, falling.
    w.players[1].pos = V3::new(Fx::from_int(22), Fx::from_int(38), Fx::ZERO);
    let mut out = false;
    for _ in 0..600 {
        w.advance(idle::<2>());
        if let Phase::RoundOver { winner, .. } = w.phase {
            assert_eq!(winner, 0);
            out = true;
            break;
        }
    }
    assert!(out, "a fall off the stage never ended the round");
    assert_eq!(w.players[0].rounds_won, 1);
}

#[test]
fn standing_on_a_stage_is_not_falling_off_it() {
    for s in versus::STAGES.iter().filter(|s| s.kind == Kind::Duel) {
        let mut w = World::versus_in([Class::Champion; 2], s.arena);
        for _ in 0..240 {
            w.advance(idle::<2>());
        }
        assert_eq!(w.phase, Phase::Fighting, "{}", s.arena.get().name);
        assert!(w.players.iter().take(2).all(|p| p.health > 0));
    }
}

#[test]
fn the_point_is_standable_at_its_middle() {
    for s in versus::STAGES.iter().filter(|s| s.kind == Kind::Hill) {
        let hill = s.hill.unwrap();
        let a = s.arena.get();
        let up = V3::new(hill.at.x, hill.at.y.add(Fx::ONE), hill.at.z);
        let top = a.ground_under(up);
        assert!(
            hill.holds(V3::new(hill.at.x, top, hill.at.z)),
            "{}: the point's middle stands at {top:?}",
            a.name
        );
        // Neither start is on it.
        for m in a.spawns.versus {
            let feet = V3::new(m.at.x, a.ground_under(m.at), m.at.z);
            assert!(!hill.holds(feet), "{}", a.name);
        }
    }
}

#[test]
fn the_point_is_a_column_of_air_capped_by_what_stands_over_it() {
    for s in versus::STAGES.iter().filter(|s| s.kind == Kind::Hill) {
        let a = s.arena.get();
        let hill = s.hill.unwrap();
        let up = |m: Fx| V3::new(hill.at.x, hill.at.y.add(m), hill.at.z);
        // A hop's height over it is on it.
        assert!(hill.holds(up(Fx::from_int(3))), "{}", a.name);
        // Nothing hanging over the point can be stood on inside the
        // column: standing on a bridge or a lintel is never standing on it.
        let top = hill.at.y.add(hill.reach());
        for solid in a.solids() {
            let over = solid.over(hill.at.x, hill.at.z, hill.radius);
            let hangs = solid.min.y.raw() > hill.at.y.raw();
            assert!(
                !over || !hangs || solid.max.y.raw() > top.raw(),
                "{}: {solid:?} can be stood on inside the point's column",
                a.name
            );
        }
    }
    // Where a walkway crosses over the point, standing on it is not on it.
    for id in [ArenaId::GORGE, ArenaId::HENGE] {
        let hill = hill_of(id);
        let deck = id
            .get()
            .ground_under(V3::new(Fx::ZERO, Fx::from_int(9), Fx::ZERO));
        assert_eq!(deck, Fx::from_int(7), "{}", id.get().name);
        assert!(!hill.holds(V3::new(Fx::ZERO, deck, Fx::ZERO)));
    }
}

#[test]
fn hopping_on_the_point_keeps_capturing_it() {
    for class in sim::class::ALL_CLASSES {
        let id = ArenaId::QUARRY;
        let hill = hill_of(id);
        let mut w = World::versus_in([class, Class::Champion], id);
        w.hill.locked = 0;
        stand(&mut w, 0, hill.at);
        home(&mut w, 1);
        let mut aloft = false;
        // Hopping the whole time: a capture's frames and a little.
        for f in 0..t::hill_capture() as u32 + 60 {
            let jump = if f % 50 < 30 { Input::SPACE } else { 0 };
            w.advance([Input::aimed(jump, 0), Input::default()]);
            aloft |= !w.players[0].grounded;
            assert_eq!(
                w.hill.standing[0], 1,
                "{class:?} fell off the point on frame {f}"
            );
        }
        assert!(aloft, "{class:?} never left the ground");
        assert_eq!(w.hill.owner, 0, "{class:?}");
    }
}

#[test]
fn a_hill_game_starts_locked_with_both_clocks_full() {
    let w = World::versus_in([Class::Champion; 2], ArenaId::QUARRY);
    assert!(w.hill.on);
    assert_eq!(w.hill.owner, NOBODY);
    assert_eq!(w.hill.locked, t::hill_unlock());
    assert_eq!(w.hill.clock, [t::hill_clock(); 2]);
    // Nowhere else is there a hill.
    for a in [ArenaId::PROVING_GROUND, ArenaId::ANVIL, ArenaId::RING] {
        assert!(!World::versus_in([Class::Champion; 2], a).hill.on);
    }
}

#[test]
fn the_point_is_captured_held_and_won() {
    let id = ArenaId::GLACIER;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    stand(&mut w, 0, hill.at);
    home(&mut w, 1);
    // Locked: standing on it does nothing.
    for _ in 0..t::hill_unlock() {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.progress, 0);
    assert_eq!(w.hill.standing, [1, 0]);
    // Then one fighter alone takes it in a capture's time.
    for _ in 0..t::hill_capture() {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.owner, 0, "{:?}", w.hill);
    // Theirs, and their clock runs while it is; the other side's does not.
    let left = w.hill.clock[0];
    for _ in 0..60 {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.clock[0], left - 60);
    assert_eq!(w.hill.clock[1], t::hill_clock());
    // Theirs still with nobody on it.
    home(&mut w, 0);
    for _ in 0..60 {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.owner, 0);
    // Held to the end of the clock unchallenged, the round is theirs.
    w.hill.clock[0] = 2;
    for _ in 0..3 {
        w.advance(idle::<2>());
    }
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "{:?}",
        w.phase
    );
    assert_eq!(w.players[0].rounds_won, 1);
    assert_eq!(w.players[1].rounds_won, 0);
}

#[test]
fn both_sides_on_the_point_is_contested_and_nothing_moves() {
    let id = ArenaId::QUARRY;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    stand(&mut w, 0, hill.at);
    stand(
        &mut w,
        1,
        hill.at.add(V3::new(Fx::from_int(2), Fx::ZERO, Fx::ZERO)),
    );
    for _ in 0..t::hill_capture() * 2 {
        w.advance(idle::<2>());
    }
    assert!(w.hill.contested());
    assert_eq!(w.hill.owner, NOBODY);
    assert_eq!(w.hill.progress, 0);
}

#[test]
fn a_capture_is_taken_back_before_the_other_side_starts_its_own() {
    let id = ArenaId::DELTA;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    stand(&mut w, 0, hill.at);
    home(&mut w, 1);
    let half = t::hill_capture() / 2;
    for _ in 0..half {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.capper, 0);
    let got = w.hill.progress;
    assert!(got > 0);
    // The other side steps on alone: the first side's capture goes back.
    home(&mut w, 0);
    stand(&mut w, 1, hill.at);
    for _ in 0..half / 2 {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.capper, 0);
    assert!(w.hill.progress < got);
    for _ in 0..half {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.capper, 1, "{:?}", w.hill);
}

#[test]
fn a_clock_out_while_challenged_is_overtime() {
    let id = ArenaId::QUARRY;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    w.hill.owner = 0;
    w.hill.clock[0] = 1;
    stand(&mut w, 1, hill.at);
    home(&mut w, 0);
    for _ in 0..30 {
        w.advance(idle::<2>());
    }
    assert_eq!(w.phase, Phase::Fighting);
    assert!(w.hill.overtime());
    // Taken in overtime, it is theirs, and the game goes on on their clock.
    for _ in 0..t::hill_capture() {
        w.advance(idle::<2>());
    }
    assert_eq!(w.hill.owner, 1);
    assert_eq!(w.phase, Phase::Fighting);
}

#[test]
fn overtime_ends_the_frame_the_last_challenger_steps_off() {
    let id = ArenaId::QUARRY;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    w.hill.owner = 0;
    w.hill.clock[0] = 1;
    stand(&mut w, 1, hill.at);
    home(&mut w, 0);
    // Half a capture in, in overtime.
    for _ in 0..t::hill_capture() / 2 {
        w.advance(idle::<2>());
    }
    assert!(w.hill.overtime());
    assert!(w.hill.progress > 0);
    // Off it: the capture they had under way does not hold the round open.
    home(&mut w, 1);
    w.advance(idle::<2>());
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "{:?} {:?}",
        w.phase,
        w.hill
    );
}

#[test]
fn contested_in_overtime_holds_the_round_open_and_moves_nothing() {
    let id = ArenaId::GLACIER;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    w.hill.owner = 0;
    w.hill.clock[0] = 1;
    // The challengers' capture part way, and both sides on the point.
    w.hill.capper = 1;
    w.hill.progress = sim::versus::Koth::full() / 3;
    let before = w.hill.progress;
    stand(&mut w, 0, hill.at);
    stand(
        &mut w,
        1,
        hill.at.add(V3::new(Fx::from_int(2), Fx::ZERO, Fx::ZERO)),
    );
    for _ in 0..t::hill_capture() * 2 {
        w.advance(idle::<2>());
    }
    assert_eq!(w.phase, Phase::Fighting);
    assert!(w.hill.contested() && w.hill.overtime());
    // No movement toward either side.
    assert_eq!(
        (w.hill.owner, w.hill.capper, w.hill.progress),
        (0, 1, before)
    );
}

#[test]
fn contested_with_a_capture_under_way_moves_nothing() {
    let id = ArenaId::DELTA;
    let hill = hill_of(id);
    let mut w = World::versus_in([Class::Champion; 2], id);
    w.hill.locked = 0;
    w.hill.owner = 0;
    w.hill.capper = 1;
    w.hill.progress = sim::versus::Koth::full() / 2;
    let before = w.hill.progress;
    stand(&mut w, 0, hill.at);
    stand(
        &mut w,
        1,
        hill.at.add(V3::new(Fx::from_int(2), Fx::ZERO, Fx::ZERO)),
    );
    for _ in 0..120 {
        w.advance(idle::<2>());
    }
    assert!(w.hill.contested());
    assert_eq!(
        (w.hill.owner, w.hill.capper, w.hill.progress),
        (0, 1, before)
    );
}

#[test]
fn a_second_fighter_of_a_side_captures_faster() {
    let id = ArenaId::GLACIER;
    let hill = hill_of(id);
    let alone = {
        let mut w = World::versus_in([Class::Champion; 4], id);
        w.hill.locked = 0;
        stand(&mut w, 0, hill.at);
        for i in 1..4 {
            home(&mut w, i);
        }
        w.advance(idle::<4>());
        w.hill.progress
    };
    let two = {
        let mut w = World::versus_in([Class::Champion; 4], id);
        w.hill.locked = 0;
        stand(&mut w, 0, hill.at);
        stand(
            &mut w,
            2,
            hill.at.add(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(2))),
        );
        home(&mut w, 1);
        home(&mut w, 3);
        w.advance(idle::<4>());
        assert_eq!(w.hill.standing, [2, 0]);
        w.hill.progress
    };
    assert!(two > alone, "{two} against {alone}");
}

#[test]
fn the_dead_stand_back_up_at_their_start_on_a_hill() {
    let id = ArenaId::QUARRY;
    let mut w = World::versus_in([Class::Champion; 2], id);
    stand(&mut w, 1, V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
    w.players[1].health = 0;
    w.advance(idle::<2>());
    // Not the last one standing: a hill is not won that way.
    assert_eq!(w.phase, Phase::Fighting);
    assert!(w.hill.respawn[1] > 0);
    for _ in 0..t::hill_respawn() + 1 {
        w.advance(idle::<2>());
    }
    assert!(w.players[1].health > 0);
    let mark = arena::Mark::seat(&w.arena().spawns.versus, 1);
    let d = w.players[1].pos.sub(mark.at);
    assert!(
        d.flat_len().raw() < Fx::ONE.raw(),
        "stood up at {:?}",
        w.players[1].pos
    );
}

#[test]
fn on_a_hill_a_side_cannot_hurt_its_own() {
    let w = World::versus_in([Class::Champion; 4], ArenaId::DELTA);
    assert!(w.foes(0, 1) && w.foes(0, 3) && w.foes(1, 2));
    assert!(!w.foes(0, 2) && !w.foes(1, 3) && !w.foes(0, 0));
    assert_eq!(w.allies(0), 1 << 2);
    assert_eq!(w.allies(3), 1 << 1);
    // A duel stage is everybody against everybody, as versus always was.
    let w = World::versus_in([Class::Champion; 4], ArenaId::CALDERA);
    assert!(w.foes(0, 2) && w.foes(1, 3));
    assert_eq!(w.allies(0), 0);
}

#[test]
fn a_hill_game_is_deterministic() {
    let run = || {
        let id = ArenaId::GLACIER;
        let hill = hill_of(id);
        let mut w = World::versus_in([Class::Champion; 2], id);
        w.hill.locked = 0;
        stand(&mut w, 0, hill.at);
        for _ in 0..t::hill_capture() + 120 {
            w.advance(idle::<2>());
        }
        w.checksum()
    };
    assert_eq!(run(), run());
}

// ---------------------------------------------------------------------------
// Every route walked
// ---------------------------------------------------------------------------

/// A route walked: the arena, where the walk starts, and its waypoints, all
/// in centimetres.
type Route = (ArenaId, (i32, i32, i32), &'static [(i32, i32, i32)]);

/// How long the walker holds a jump: to the top of a held hop.
const HOLD: u32 = 30;

/// A point in centimetres: (x, y, z), y the height the feet arrive at.
fn cm(p: (i32, i32, i32)) -> V3 {
    V3::new(
        Fx::ratio(p.0, 100),
        Fx::ratio(p.1, 100),
        Fx::ratio(p.2, 100),
    )
}

/// **Walk a route**: `class` stood at `from` on whatever is under it, then
/// walking at each waypoint in turn -- forward held, the look on it, jumping
/// over and over (held most of a second, let go a sixth) -- until its feet
/// are within a body of it at its height. `Err` names the waypoint it never
/// reached.
fn walk(
    id: ArenaId,
    class: Class,
    from: (i32, i32, i32),
    route: &[(i32, i32, i32)],
) -> Result<(), String> {
    let mut w = World::versus_in([class, Class::Champion], id);
    w.hill.locked = u16::MAX;
    stand(&mut w, 0, cm(from));
    home(&mut w, 1);
    for (k, &goal) in route.iter().enumerate() {
        let to = cm(goal);
        let mut arrived = false;
        // Frames into the jump under way (0: none); frames walking without
        // getting anywhere.
        let (mut held, mut stuck) = (0u32, 0u32);
        for _ in 0..900 {
            let p = w.players[0];
            let d = to.sub(p.pos);
            let level = d.y.abs().raw() < Fx::ratio(15, 100).raw();
            if p.grounded && level && d.flat_len().raw() < Fx::ONE.raw() {
                arrived = true;
                break;
            }
            let above = d.y.raw() > Fx::ratio(15, 100).raw();
            // A drop just ahead, toward the goal: jump it, as a player at
            // the edge of a gap does, rather than walk off.
            let ahead = {
                let flat = V3::new(d.x, Fx::ZERO, d.z).normalized();
                let at = p.pos.add(flat.scale(Fx::ratio(7, 10)));
                w.terrain().ground_under(V3::new(at.x, p.pos.y, at.z))
            };
            let edge = ahead.raw() < p.pos.y.sub(Fx::ratio(3, 2)).raw()
                && d.flat_len().raw() > Fx::from_int(2).raw();
            let over = d.flat_len().raw() < Fx::ratio(4, 10).raw();
            stuck = if p.grounded && p.vel.flat_len().raw() < Fx::ratio(1, 2).raw() {
                stuck + 1
            } else {
                0
            };
            if p.grounded && held >= HOLD {
                held = 0;
            }
            // A jump is pressed off the ground and held to its top, and only
            // when the goal is higher or the walk is blocked.
            let jump = if held > 0 && held < HOLD {
                held += 1;
                Input::SPACE
            } else if held == 0 && p.grounded && (above || stuck > 20 || edge) {
                held = 1;
                Input::SPACE
            } else {
                if held > 0 {
                    held += 1;
                }
                0
            };
            let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
            let bits = if over { jump } else { Input::W | jump };
            w.advance([Input::aimed(bits, aim), Input::default()]);
        }
        if !arrived {
            return Err(format!(
                "{} never reached waypoint {k} {goal:?}: stuck at {:?}",
                id.get().name,
                w.players[0].pos
            ));
        }
    }
    Ok(())
}

/// Every climb and every hidden way on the three hill maps, walked by the
/// class with the shortest jump: nobody is shut out of high ground in a
/// team game. Each map is turned half a turn about its middle, so one half
/// walked is both.
#[test]
fn every_way_on_a_hill_map_is_walked_by_the_shortest_jump() {
    let routes: &[Route] = &[
        // Quarry: the point; the north bench by its stair, and by the
        // ledge from the point's side; the tor; the cutting end to end, and
        // out of it through the slot onto the point.
        (ArenaId::QUARRY, (-2500, 0, 0), &[(0, 45, 0)]),
        (
            ArenaId::QUARRY,
            (-2300, 0, 1350),
            &[(-1600, 130, 1350), (-1330, 270, 1350), (-700, 400, 1350)],
        ),
        (
            ArenaId::QUARRY,
            (800, 0, 500),
            &[(800, 200, 1075), (800, 400, 1650)],
        ),
        (
            ArenaId::QUARRY,
            (-1500, 0, -1500),
            &[(-2000, 220, -1500), (-2300, 450, -1500)],
        ),
        (ArenaId::QUARRY, (-1700, 0, 1650), &[(1700, 0, 1650)]),
        (
            ArenaId::QUARRY,
            (-1700, 0, 1650),
            &[(0, 0, 1650), (0, 0, 1350), (0, 45, 300)],
        ),
        // Glacier: the dome by its stair and by the ledge; the shelf by its
        // stair; the melt cave under the point; the cave under the shelf.
        (
            ArenaId::GLACIER,
            (-1600, 0, 700),
            &[(-900, 150, 700), (-300, 300, 700)],
        ),
        (
            ArenaId::GLACIER,
            (0, 0, 1700),
            &[(0, 150, 1050), (0, 300, 500)],
        ),
        (
            ArenaId::GLACIER,
            (-3700, 0, 1350),
            &[(-3400, 180, 1350), (-3150, 360, 1350), (-2700, 550, 1650)],
        ),
        (ArenaId::GLACIER, (-1600, 0, 0), &[(1600, 0, 0)]),
        (ArenaId::GLACIER, (-3400, 0, 1950), &[(-900, 0, 1950)]),
        // Delta: the dyke from either face; the aqueduct by its stair, to
        // its broken end; the sluice from the south lane to the breach and
        // onto the point; through the breach.
        (
            ArenaId::DELTA,
            (-2600, 0, 1000),
            &[(-2050, 150, 1000), (-1600, 300, 1000)],
        ),
        (
            ArenaId::DELTA,
            (-600, 0, -1100),
            &[(-1150, 150, -1100), (-1600, 300, -1100)],
        ),
        (
            ArenaId::DELTA,
            (-3700, 0, 1500),
            &[
                (-3300, 180, 1500),
                (-3100, 360, 1500),
                (-2700, 570, 1500),
                (300, 570, 1500),
            ],
        ),
        (
            ArenaId::DELTA,
            (-1600, 0, -2600),
            &[(-1600, 0, -500), (-1600, 0, 0), (0, 45, 0)],
        ),
        (ArenaId::DELTA, (-2800, 0, 0), &[(0, 45, 0)]),
        // Gorge: the point along the river; the north cliff-top by its
        // west ledges and by its east ones, then across the bridge over the
        // point to the south cliff-top; the cave from the bay to its mouth
        // by the point.
        (ArenaId::GORGE, (-2800, 0, 0), &[(0, 45, 0)]),
        (
            ArenaId::GORGE,
            (-2300, 0, 400),
            &[(-2300, 230, 780), (-2300, 700, 1400)],
        ),
        (
            ArenaId::GORGE,
            (1500, 0, 400),
            &[
                (1500, 230, 780),
                (1500, 700, 1400),
                (0, 700, 1200),
                (0, 700, 0),
                (0, 700, -1300),
            ],
        ),
        (
            ArenaId::GORGE,
            (-3400, 0, 700),
            &[
                (-3400, 0, 1450),
                (-150, 0, 1450),
                (-150, 0, 700),
                (0, 45, 200),
            ],
        ),
        // Henge: the west stone by its stair, along the lintel over the
        // point and on to the north stone; the north stone from outside;
        // the barrow end to end and on to the point.
        (
            ArenaId::HENGE,
            (-2200, 0, 0),
            &[
                (-1600, 230, 0),
                (-1400, 470, 0),
                (-1100, 700, 0),
                (-500, 700, 0),
                (0, 700, 0),
                (0, 700, 1100),
            ],
        ),
        (
            ArenaId::HENGE,
            (0, 0, 2200),
            &[(0, 230, 1600), (0, 470, 1400), (0, 700, 1100)],
        ),
        (
            ArenaId::HENGE,
            (-3300, 0, -1300),
            &[(-1500, 0, -1300), (0, 45, 0)],
        ),
    ];
    let mut failed = Vec::new();
    for (id, from, route) in routes {
        if let Err(e) = walk(*id, Class::Bulwark, *from, route) {
            failed.push(e);
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// The high places on the floating stages, walked by every class that moves
/// (`sim::class::MOVERS`).
#[test]
fn every_height_on_a_floating_stage_is_reached() {
    let routes: &[Route] = &[
        (ArenaId::ANVIL, (600, 4000, -500), &[(600, 4370, 0)]),
        (ArenaId::ANVIL, (-500, 4000, 0), &[(-1650, 3900, 0)]),
        (ArenaId::ZIGGURAT, (0, 4000, 600), &[(0, 3850, 1300)]),
        (ArenaId::ANVIL, (-500, 4000, 0), &[(0, 4045, 0)]),
        (
            ArenaId::ZIGGURAT,
            (-850, 4000, 0),
            &[(-450, 4100, 0), (0, 4200, 0)],
        ),
        (ArenaId::CALDERA, (-600, 4000, -600), &[(0, 4125, 0)]),
        (ArenaId::TWIN_ISLES, (-700, 4000, 550), &[(0, 4380, 550)]),
        (
            ArenaId::TWIN_ISLES,
            (-1000, 4000, 300),
            &[(-1000, 4100, -300)],
        ),
        (
            ArenaId::TWIN_ISLES,
            (-900, 4000, 0),
            &[(0, 4000, 0), (900, 4000, 0)],
        ),
    ];
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        for (id, from, route) in routes {
            if let Err(e) = walk(*id, class, *from, route) {
                failed.push(format!("{class:?}: {e}"));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
