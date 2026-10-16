//! **The wall slide and the wall jump** (2026-10-16, `state::cling_to_the_wall`).
//!
//! What is pinned: a body falling against a wall and pushing into it slides,
//! a little slower than a fall and no slower than half of one; it stays on
//! the wall for as far as the wall goes, and steering away leaves it; a slide to
//! the floor costs no fall damage where the same fall in the open does; space
//! on a wall jumps up and away from it without giving back the airdodge;
//! off the same wall again, anywhere along it, only once the body is
//! `wall_jump_drop` below where it last jumped off it; each wall jump
//! in an airtime rises less than the one before; and landing forgets it all.
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

/// Facing -z (into the north face from the north), and +z.
const SOUTH: u16 = 49152;
const NORTH: u16 = 16384;

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
fn a_slide_is_a_little_slower_than_a_fall() {
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
        slide.raw() < fall.raw(),
        "sliding at {slide:?}, falling at {fall:?}: the wall did nothing"
    );
    assert!(
        slide.raw() > fall.mul(Fx::ratio(1, 2)).raw(),
        "sliding at {slide:?}, falling at {fall:?}: the wall is a perch, not a slide"
    );
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
    let mut w = against_the_north_face(8, mid_face());
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
    while w.players[0].pos.y.raw() > jumped_at.sub(t::wall_jump_drop()).raw() {
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

#[test]
fn each_wall_jump_in_an_airtime_rises_less() {
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
    let rise = t::jump_speed()
        .mul(Class::Champion.mobility().jump)
        .mul(t::wall_jump_rise());
    let want = rise.mul(t::wall_jump_falloff()).add(first.sub(rise));
    assert!(
        p.vel.y.sub(want).abs().raw() <= Fx::ratio(1, 100).raw(),
        "second rise {:?}, wanted {want:?}",
        p.vel.y
    );
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
