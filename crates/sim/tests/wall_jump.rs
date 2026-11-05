//! **The wall slide and the wall jump** (2026-10-16, `state::cling_to_the_wall`).
//!
//! What is pinned: a body falling against a wall slides unless the stick
//! pushes away -- along it or let go as much as into it (2026-10-20) --
//! at well under half a fall's speed but never stopping, and a fast fall is
//! grabbed down to the slide over a few frames; it stays on the wall for as
//! far as the wall goes, bleeds the run it brought along it toward nothing
//! (2026-11-04), turns round along it gradually, never gains speed going
//! round a column, and steering away leaves it; a slide to the floor costs no fall damage where the same fall
//! in the open does; space on a wall jumps up and away from it without giving
//! back the airdodge, and the way the stick points along the wall; a press
//! just before the wall is kept for it; off the same wall again, anywhere
//! along it, only once the body is `wall_jump_drop` below where it last
//! jumped off it; every wall jump in an airtime rises as high as the first
//! (2026-11-04), and two facing walls are climbed; and landing forgets it all.
//!
//! The walls are the range's tower: its north face (z = 6, x 17 to 23, a
//! clean 12 m from the floor) and the south face of its 9 m ledge (z = -6).
//! Every body here starts with the top out of a hand's reach, or the ledge
//! would catch it (`tests/ledge.rs`); how the stick starts and ends a slide
//! is pinned there too.

use sim::arena::ArenaId;
use sim::fixed::Fx;
use sim::tuning as t;
use sim::{Class, Input, V3, World};

/// Facing -z (into the north face from the north), and +z; and +x, along
/// the face.
const SOUTH: u16 = 49152;
const NORTH: u16 = 16384;
const EAST: u16 = 0;

/// A Champion in the air at height `y` beside the tower's north face, `x`
/// along it, drifting into it; the other fighter out of the way.
fn against_the_north_face(y: i32, x: Fx) -> World {
    let mut w = World::versus_in([Class::Champion; 2], ArenaId::RANGE);
    let p = &mut w.players[0];
    p.pos = V3::new(x, Fx::from_int(y), Fx::ratio(66, 10));
    p.vel = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-2));
    p.grounded = false;
    w.players[1].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::ZERO);
    w
}

fn mid_face() -> Fx {
    Fx::from_int(20)
}

/// The same body in the open, well clear of the tower.
fn in_the_open(y: i32) -> World {
    let mut w = against_the_north_face(y, mid_face());
    w.players[0].pos.z = Fx::from_int(14);
    w
}

fn idle(w: &mut World, frames: u32) {
    for _ in 0..frames {
        w.advance([Input::aimed(0, SOUTH), Input::default()]);
    }
}

/// Pushing into the north face (forward, looking south): what starts a slide
/// and keeps it.
fn lean_in(w: &mut World, frames: u32) {
    for _ in 0..frames {
        w.advance([Input::aimed(Input::W, SOUTH), Input::default()]);
    }
}

fn on_the_wall(w: &World) -> bool {
    w.players[0].wall_since == 0
}

#[test]
fn a_slide_is_plainly_slower_than_a_fall_and_never_stops() {
    let mut wall = against_the_north_face(9, mid_face());
    let mut open = in_the_open(9);
    for f in 0..40 {
        lean_in(&mut wall, 1);
        idle(&mut open, 1);
        if f >= 3 {
            assert!(on_the_wall(&wall), "came off the wall on frame {f}");
        }
    }
    let (slide, fall) = (wall.players[0].vel.y.neg(), open.players[0].vel.y.neg());
    assert!(
        slide.raw() < fall.mul(Fx::ratio(35, 100)).raw(),
        "sliding at {slide:?}, falling at {fall:?}: the slide is hardly slower"
    );
    assert!(
        slide.raw() > Fx::from_int(2).raw(),
        "sliding at {slide:?}: the wall is a perch, not a slide"
    );
}

/// **The wall grabs a fall**: a body arriving at full fall speed is brought
/// down to the slide within a few frames, not left to fall past at speed.
#[test]
fn a_fast_fall_is_grabbed_by_the_wall() {
    let mut w = against_the_north_face(9, mid_face());
    w.players[0].vel.y = t::fall_cap().mul(Class::Champion.mobility().fall_cap);
    let arrived = w.players[0].vel.y.neg();
    lean_in(&mut w, 12);
    assert!(on_the_wall(&w));
    let now = w.players[0].vel.y.neg();
    assert!(
        now.raw() < arrived.mul(Fx::ratio(1, 3)).raw(),
        "arrived at {arrived:?}, still {now:?} twelve frames on"
    );
}

/// **Touching a wall is enough** (2026-10-20): the stick along the wall, or
/// let go, starts a slide as surely as into it. Only away does not.
#[test]
fn along_the_wall_or_let_go_starts_a_slide_and_away_does_not() {
    for (what, bits, aim, slides) in [
        ("along", Input::W, EAST, true),
        ("let go", 0, SOUTH, true),
        ("into", Input::W, SOUTH, true),
        ("away", Input::W, NORTH, false),
    ] {
        let mut w = against_the_north_face(9, mid_face());
        for _ in 0..6 {
            w.advance([Input::aimed(bits, aim), Input::default()]);
        }
        assert_eq!(w.players[0].sliding, slides, "the stick {what}");
    }
}

/// **A slide bleeds the run** (2026-11-04, the owner: "wall slides need to
/// naturally and pretty quickly reduce lateral momentum towards 0"). A body
/// carried along the wall onto it, the stick into the wall or let go, has
/// lost most of that speed half a second later; the stick along the wall
/// holds it near the steer instead.
#[test]
fn a_slide_bleeds_the_run_along_the_wall() {
    for (what, bits, aim) in [("into", Input::W, SOUTH), ("let go", 0, SOUTH)] {
        let mut w = against_the_north_face(9, Fx::from_int(17));
        w.players[0].vel.x = Fx::from_int(7);
        for _ in 0..30 {
            w.advance([Input::aimed(bits, aim), Input::default()]);
        }
        assert!(w.players[0].sliding, "the stick {what}: not on the slide");
        let along = w.players[0].vel.x;
        assert!(
            along.raw() < Fx::from_int(2).raw() && along.raw() >= 0,
            "the stick {what}: the run along the wall is still {along:?}"
        );
    }
    let mut w = against_the_north_face(9, Fx::from_int(17));
    w.players[0].vel.x = Fx::from_int(7);
    for _ in 0..30 {
        w.advance([Input::aimed(Input::W, EAST), Input::default()]);
    }
    let along = w.players[0].vel.x;
    let steer = t::wall_slide_steer();
    assert!(
        along.sub(steer).abs().raw() < Fx::ratio(3, 2).raw(),
        "the stick along: {along:?}, the steer {steer:?}"
    );
}

/// **Turning round on a wall is a turn, not a flip** (2026-11-04, the owner:
/// "changing direction during a wall slide just flips the sign of your
/// lateral momentum"). A run along the wall, the stick pushed the other
/// way: a few frames on it is still going the first way, slower; three
/// quarters of a second on it is going the new way.
#[test]
fn turning_round_on_a_wall_is_gradual() {
    let mut w = against_the_north_face(9, Fx::from_int(19));
    lean_in(&mut w, 4);
    for _ in 0..30 {
        w.advance([Input::aimed(Input::W, EAST), Input::default()]);
    }
    let before = w.players[0].vel.x;
    assert!(
        before.raw() > Fx::from_int(3).raw(),
        "never ran: {before:?}"
    );
    // The other way along it.
    const WEST: u16 = 32768;
    for _ in 0..5 {
        w.advance([Input::aimed(Input::W, WEST), Input::default()]);
    }
    assert!(w.players[0].sliding);
    let soon = w.players[0].vel.x;
    assert!(
        soon.raw() > 0 && soon.raw() < before.raw(),
        "five frames into the turn: {before:?} to {soon:?}"
    );
    for _ in 0..40 {
        w.advance([Input::aimed(Input::W, WEST), Input::default()]);
    }
    let p = &w.players[0];
    assert!(p.sliding, "off the slide at {:?}, {:?}", p.pos, p.vel);
    assert!(
        w.players[0].vel.x.raw() < Fx::from_int(-2).raw(),
        "three quarters of a second on, still {:?}",
        w.players[0].vel.x
    );
}

/// **Round a column, a slide never speeds up** (2026-11-04, from play). The
/// push into the wall that holds a body on it is, on a curved face, a little
/// along the next frame's face; read back as the run, every frame round a
/// column added to it. The Spire's shaft, below its stair.
#[test]
fn a_slide_round_a_column_never_speeds_up() {
    use sim::arena::spire::{AT, SHAFT};
    let j = sim::valley::Journey {
        beaten: u32::MAX,
        ..sim::valley::Journey::default()
    };
    let mut w = World::arrive(
        [Class::Champion, Class::Bulwark],
        j,
        ArenaId::SPIRE,
        Some(0),
        1,
    );
    let o = w.map_origin();
    let cm = sim::arena::cm;
    let axis = V3::new(cm(AT.0), Fx::ZERO, cm(AT.1)).sub(o);
    let axis = V3::new(axis.x, Fx::ZERO, axis.z);
    // On the shaft's north side, just off its skin, moving round it east.
    let r = cm(SHAFT.0).add(t::body_radius()).add(Fx::ratio(1, 50));
    // The plaza at the shaft, in the world's frame: the shaft's top less
    // its height.
    let atlas = sim::atlas::valley();
    let (mx, mz) = (cm(AT.0), cm(AT.1));
    let reach = Fx::from_int(20);
    let base = atlas
        .near(
            (mx.sub(reach), mz.sub(reach)),
            (mx.add(reach), mz.add(reach)),
        )
        .map(|i| atlas.solids[i as usize])
        .filter(
            |s| matches!(s.shape, sim::arena::Shape::Round { hollow, .. } if hollow == Fx::ZERO),
        )
        .map(|s| s.max.y)
        .filter(|y| y.to_int() > 200)
        .min()
        .expect("the shaft")
        .sub(cm(SHAFT.2))
        .sub(o.y);
    let mut start = axis.add(V3::new(Fx::ZERO, Fx::ZERO, r));
    start.y = base.add(Fx::from_int(9));
    let speed = Fx::from_int(6);
    {
        let p = &mut w.players[0];
        p.pos = start;
        p.vel = V3::new(speed, Fx::ZERO, Fx::from_int(-1));
        p.grounded = false;
        w.players[1].pos = axis.add(V3::new(Fx::from_int(60), Fx::ZERO, Fx::ZERO));
    }
    let mut slid = 0;
    let mut fastest = Fx::ZERO;
    for f in 0..45 {
        w.advance([Input::aimed(0, SOUTH), Input::default()]);
        let p = &w.players[0];
        assert!(!p.grounded, "landed on frame {f}");
        if !p.sliding {
            continue;
        }
        slid += 1;
        // Round the axis: the flat speed across the line to it.
        let rel = V3::new(p.pos.x.sub(axis.x), Fx::ZERO, p.pos.z.sub(axis.z));
        let d = rel.flat_len();
        let round = p.vel.x.mul(rel.z).sub(p.vel.z.mul(rel.x)).div(d).abs();
        fastest = fastest.max(round);
        assert!(
            round.raw() <= speed.raw(),
            "frame {f}: going round at {round:?}, arrived at {speed:?}"
        );
    }
    assert!(slid > 30, "slid {slid} frames of 45");
    let p = &w.players[0];
    let rel = V3::new(p.pos.x.sub(axis.x), Fx::ZERO, p.pos.z.sub(axis.z));
    let round = p
        .vel
        .x
        .mul(rel.z)
        .sub(p.vel.z.mul(rel.x))
        .div(rel.flat_len())
        .abs();
    assert!(
        round.raw() < Fx::from_int(2).raw(),
        "still going round at {round:?} (fastest {fastest:?})"
    );
}

/// **The wall jump goes where the stick points** (2026-10-20): along the
/// wall, out and forward at least `wall_jump_carry`; straight out, nothing
/// along it.
#[test]
fn a_wall_jump_goes_the_way_the_stick_points_along_the_wall() {
    let mut w = against_the_north_face(8, mid_face());
    lean_in(&mut w, 20);
    w.advance([
        Input::aimed(Input::W | Input::SPACE, EAST),
        Input::default(),
    ]);
    let p = &w.players[0];
    assert_eq!(p.wall_jumps, 1);
    assert!(p.vel.z.raw() > 0, "not pushed off the wall: {:?}", p.vel);
    // A frame of steering the slide's way before the press is not counted:
    // the carry is the floor of it.
    assert!(
        p.vel.x.raw() >= t::wall_jump_carry().mul(Fx::ratio(95, 100)).raw(),
        "along the wall at {:?}, wanted {:?}",
        p.vel.x,
        t::wall_jump_carry()
    );

    let mut w = against_the_north_face(8, mid_face());
    lean_in(&mut w, 20);
    w.advance([
        Input::aimed(Input::W | Input::SPACE, NORTH),
        Input::default(),
    ]);
    let p = &w.players[0];
    assert_eq!(p.wall_jumps, 1);
    assert!(
        p.vel.x.abs().raw() < Fx::ratio(1, 2).raw(),
        "straight out went along it: {:?}",
        p.vel
    );
}

/// **A press just before the wall is kept for it** (2026-10-20,
/// `wall_jump_buffer`): jump pressed a few frames out from the wall, and the
/// wall jump comes the frame the wall is there.
#[test]
fn a_press_just_before_the_wall_is_kept_for_it() {
    let mut w = against_the_north_face(8, mid_face());
    // Half a metre out, closing at 4 m/s; pressed at 0.3 m, four or five
    // frames before the face.
    w.players[0].pos.z = Fx::ratio(71, 10);
    w.players[0].vel.z = Fx::from_int(-4);
    let mut pressed = false;
    let mut touched_at = None;
    for f in 0..20 {
        let near = w.players[0].pos.z.raw() < Fx::ratio(69, 10).raw();
        let bits = if near && !pressed {
            assert!(!on_the_wall(&w), "pressed on the wall, not before it");
            pressed = true;
            Input::SPACE
        } else {
            0
        };
        w.advance([Input::aimed(bits, SOUTH), Input::default()]);
        if touched_at.is_none() && w.players[0].wall_since == 0 {
            touched_at = Some(f);
        }
        if w.players[0].wall_jumps == 1 {
            break;
        }
    }
    assert!(pressed && touched_at.is_some(), "never reached the wall");
    assert_eq!(w.players[0].wall_jumps, 1, "the press was not kept");
    assert!(w.players[0].vel.z.raw() > 0, "{:?}", w.players[0].vel);
}

#[test]
fn a_slide_to_the_floor_costs_nothing_where_the_fall_would() {
    let mut wall = against_the_north_face(9, mid_face());
    let mut open = in_the_open(9);
    // Dropped as from twenty metres up, so the open fall hurts.
    for w in [&mut wall, &mut open] {
        w.players[0].falling_from(Fx::from_int(20));
    }
    let full = wall.players[0].health;
    for _ in 0..240 {
        lean_in(&mut wall, 1);
        idle(&mut open, 1);
    }
    for w in [&wall, &open] {
        assert!(w.players[0].grounded && w.players[0].pos.y == Fx::ZERO);
    }
    assert!(full - open.players[0].health > 0, "the open fall was free");
    assert_eq!(full - wall.players[0].health, 0, "the slide hurt");
}

#[test]
fn letting_go_partway_down_costs_only_the_rest() {
    // Slid from nine metres to about six, then steered off: the fall is
    // measured from where it let go, so it costs less than the whole drop.
    let mut wall = against_the_north_face(9, mid_face());
    let mut open = in_the_open(9);
    for w in [&mut wall, &mut open] {
        w.players[0].falling_from(Fx::from_int(20));
    }
    let full = wall.players[0].health;
    while wall.players[0].pos.y.raw() > Fx::from_int(6).raw() {
        lean_in(&mut wall, 1);
    }
    for _ in 0..240 {
        wall.advance([Input::aimed(Input::W, NORTH), Input::default()]);
        idle(&mut open, 1);
    }
    let (slid, fell) = (full - wall.players[0].health, full - open.players[0].health);
    assert!(
        slid < fell,
        "let go at 8 m cost {slid}, the whole fall {fell}"
    );
}

#[test]
fn steering_away_leaves_the_wall() {
    let mut w = against_the_north_face(9, mid_face());
    lean_in(&mut w, 6);
    assert!(on_the_wall(&w));
    for _ in 0..20 {
        w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
    }
    assert!(!on_the_wall(&w), "still on it");
    assert!(w.players[0].pos.z.raw() > Fx::ratio(66, 10).raw());
}

#[test]
fn a_wall_jump_goes_up_and_away_and_gives_nothing_back() {
    let mut w = against_the_north_face(8, mid_face());
    lean_in(&mut w, 20);
    assert!(on_the_wall(&w) && w.players[0].vel.y.raw() < 0);
    w.players[0].air_dodged = true;
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    let p = &w.players[0];
    assert!(p.vel.y.raw() > 0, "no rise: {:?}", p.vel);
    assert!(p.vel.z.raw() > 0, "not pushed off the wall: {:?}", p.vel);
    assert!(p.air_dodged, "the wall jump gave the airdodge back");
    assert_eq!(p.wall_jumps, 1);
    // A little height, not a jump's.
    let rise = p.vel.y;
    let takeoff = t::jump_speed().mul(Class::Champion.mobility().jump);
    assert!(
        rise.raw() < takeoff.raw(),
        "{rise:?} against a takeoff of {takeoff:?}"
    );
}

#[test]
fn a_wall_jump_is_still_there_a_few_frames_after_leaving() {
    let mut w = against_the_north_face(8, mid_face());
    lean_in(&mut w, 20);
    w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
    w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
    assert!(!on_the_wall(&w));
    w.advance([
        Input::aimed(Input::W | Input::SPACE, NORTH),
        Input::default(),
    ]);
    assert_eq!(w.players[0].wall_jumps, 1, "the grace was not there");
}

#[test]
fn the_same_wall_again_only_from_lower_down() {
    // Five metres up: since 2026-11-04 a kick rises far enough that from
    // eight the hands met the tower's top and the body hung there.
    let mut w = against_the_north_face(5, mid_face());
    lean_in(&mut w, 10);
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    assert_eq!(w.players[0].wall_jumps, 1);
    let jumped_at = w.players[0].wall_jumped.at.y;
    // Back on to the same face, two metres along it. Turned round by hand:
    // however it is done in play -- strafing, an aerial's hang to buy the
    // time -- arriving back is what the rule is about.
    w.players[0].pos.x = Fx::from_int(22);
    w.players[0].vel.z = Fx::from_int(-4);
    let mut back = false;
    for _ in 0..90 {
        w.advance([Input::aimed(0, SOUTH), Input::default()]);
        assert!(!w.players[0].grounded, "came down before the wall");
        if on_the_wall(&w) {
            back = true;
            break;
        }
    }
    assert!(back, "never got back to the wall: {:?}", w.players[0].pos);
    // Higher than the drop allows: refused, and it slides on.
    assert!(w.players[0].pos.y.raw() > jumped_at.sub(t::wall_jump_drop()).raw());
    let before = w.players[0].vel;
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    let p = &w.players[0];
    assert_eq!(p.wall_jumps, 1, "jumped off the same wall from as high");
    assert!(
        p.vel.y.raw() < before.y.raw() && p.vel.z.raw() <= 0,
        "jumped off the same wall: {before:?} to {:?}",
        p.vel
    );
    // Slid the drop's worth below where it jumped: offered again.
    // Still rising, so the wall is not holding it yet: kept against it.
    for _ in 0..300 {
        if w.players[0].pos.y.raw() <= jumped_at.sub(t::wall_jump_drop()).raw() {
            break;
        }
        w.players[0].vel.z = w.players[0].vel.z.min(Fx::from_int(-1));
        idle(&mut w, 1);
        assert!(!w.players[0].grounded, "came down before the drop");
    }
    assert!(on_the_wall(&w), "off the wall: {:?}", w.players[0].pos);
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    let p = &w.players[0];
    assert_eq!(p.wall_jumps, 2, "refused from below the drop");
    assert!(p.vel.y.raw() > 0 && p.vel.z.raw() > 0, "{:?}", p.vel);
}

/// **Every kick as high as the first** (2026-11-04): until then each one in
/// an airtime was worth `0.7` of the one before.
#[test]
fn every_wall_jump_in_an_airtime_rises_as_high_as_the_first() {
    let mut w = against_the_north_face(8, mid_face());
    lean_in(&mut w, 10);
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    let first = w.players[0].vel.y;
    // Now against the ledge's south face, a different wall, falling.
    let p = &mut w.players[0];
    p.pos = V3::new(mid_face(), Fx::from_int(5), Fx::ratio(-651, 100));
    p.vel = V3::new(Fx::ZERO, Fx::from_int(-4), Fx::from_int(3));
    // Pushing into it, so it is a slide as the first was: both jumps leave
    // a slide, and so both frames after have had a slide's gravity.
    w.advance([Input::aimed(Input::W, NORTH), Input::default()]);
    w.advance([
        Input::aimed(Input::W | Input::SPACE, NORTH),
        Input::default(),
    ]);
    let p = &w.players[0];
    assert_eq!(p.wall_jumps, 2, "the second wall refused: {:?}", p.pos);
    assert!(p.vel.z.raw() < 0, "pushed the wrong way: {:?}", p.vel);
    // Both read a frame after the press, so both have had a frame of gravity.
    // No falloff (2026-11-04, the owner: "get rid of the decay per jump").
    let want = first;
    assert!(
        p.vel.y.sub(want).abs().raw() <= Fx::ratio(1, 100).raw(),
        "second rise {:?}, wanted {want:?}",
        p.vel.y
    );
}

/// **Two facing walls are climbed** (2026-11-04, the owner: "I want people
/// to be able to climb up wall jump sections"). The bench's first gully
/// lane (`arena::bench`): walls three metres apart, from 50 m to 72 m. Kicked
/// off one with the stick toward the other, and again the moment that one
/// is under the hand: every mover is six metres up within six kicks, never
/// having come down. (Until 2026-11-04 the falloff stopped a flue like this
/// at about a jump's height.)
#[test]
fn two_facing_walls_are_climbed() {
    for class in sim::class::MOVERS {
        let mut w = World::versus_in([class, Class::Champion], ArenaId::BENCH);
        // Between the walls of the lane at z = 160, against the south one.
        let start = V3::new(Fx::from_int(4), Fx::from_int(55), Fx::ratio(1590, 10));
        {
            let p = &mut w.players[0];
            p.pos = start;
            p.vel = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-2));
            p.grounded = false;
            w.players[1].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-400));
        }
        // Toward the wall it is against: south (-z) first.
        let mut toward = SOUTH;
        let mut kicks = 0;
        let goal = start.y.add(Fx::from_int(6));
        for _ in 0..600 {
            let p = &w.players[0];
            let press = p.wall_since == 0 && p.vel.y.raw() <= 0;
            let input = if press {
                toward = if toward == SOUTH { NORTH } else { SOUTH };
                Input::aimed(Input::W | Input::SPACE, toward)
            } else {
                Input::aimed(Input::W, toward)
            };
            let before = w.players[0].wall_jumps;
            w.advance([input, Input::default()]);
            if w.players[0].wall_jumps > before {
                kicks += 1;
            }
            assert!(!w.players[0].grounded, "{class:?}: came down");
            if w.players[0].pos.y.raw() >= goal.raw() || kicks > 6 {
                break;
            }
        }
        assert!(
            w.players[0].pos.y.raw() >= goal.raw() && kicks <= 6,
            "{class:?}: {kicks} kicks, at {:?} from {:?}",
            w.players[0].pos.y,
            start.y
        );
    }
}

#[test]
fn landing_forgets_the_wall() {
    let mut w = against_the_north_face(3, mid_face());
    lean_in(&mut w, 4);
    w.advance([Input::aimed(Input::SPACE, SOUTH), Input::default()]);
    assert_eq!(w.players[0].wall_jumps, 1);
    idle(&mut w, 120);
    assert!(w.players[0].grounded);
    idle(&mut w, 1);
    assert_eq!(w.players[0].wall_jumps, 0);
    assert!(!w.players[0].wall_jumped.is_some());
}

/// A Champion in the air at `pos`, drifting at `vel` (flat), the other
/// fighter out of the way.
fn in_the_air_at(pos: V3, vel: V3) -> World {
    let mut w = World::versus_in([Class::Champion; 2], ArenaId::RANGE);
    let p = &mut w.players[0];
    p.pos = pos;
    p.vel = vel;
    p.grounded = false;
    w.players[1].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::ZERO);
    w
}

/// **Only a wall is a wall** (2026-10-20, from play: the slide was drawn
/// "on very thin platforms where I wouldn't be able to slide at all"). A
/// face is slid on and jumped off only where it covers most of the body's
/// height (`wall_face_high`) and is wide enough across it
/// (`wall_face_wide`): the range's wooden posts are eight-tenths of a metre
/// across, and its low wall along the south edge (x 5 to 35, z 21 to 22)
/// is a metre and a half tall -- each stops the body, and neither is a slide.
#[test]
fn a_post_or_a_short_face_is_not_slid_on_or_jumped_off() {
    // North of the post at x -36, z -8.4 to -7.6, falling beside it.
    let post = in_the_air_at(
        V3::new(Fx::from_int(-36), Fx::from_int(3), Fx::ratio(-700, 100)),
        V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-3)),
    );
    // North of the low wall, its top level with the body's chest.
    let low = in_the_air_at(
        V3::new(Fx::from_int(20), Fx::ratio(9, 10), Fx::ratio(2040, 100)),
        V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(3)),
    );
    for (what, mut w, aim) in [("the post", post, SOUTH), ("the low wall", low, NORTH)] {
        let mut stopped = false;
        for _ in 0..8 {
            w.advance([Input::aimed(Input::W, aim), Input::default()]);
            assert!(!w.players[0].grounded, "{what}: landed");
            stopped |= w.players[0].vel.z == Fx::ZERO;
            assert!(!w.players[0].sliding, "{what}: slid");
            assert!(!on_the_wall(&w), "{what}: counted as a wall");
        }
        assert!(stopped, "{what}: never met it");
        w.advance([Input::aimed(Input::W | Input::SPACE, aim), Input::default()]);
        assert_eq!(w.players[0].wall_jumps, 0, "{what}: jumped off it");
    }
}
