//! Aiming with your feet off the ground.
//!
//! The report was that a shot thrown from the air, aimed down, went somewhere
//! nobody chose. Two separate things were wrong and they compounded, which is
//! why the symptom had no shape to it.
//!
//! **The shot was raised to the caster's own height.** A skillshot aimed at the
//! floor is lifted off it so that it goes through whoever is standing there
//! rather than into the dirt -- and it used to be lifted to the height *she*
//! was at, which standing up is level over the ground and five metres up is
//! level over nothing. That half is fixed, and it was fixed from the other end:
//! `aim::standing_middle` measures the lift from **the ground the ray met**,
//! which the platform case found first. The first test below is this file's
//! regression guard on the airborne case of the same rule, because a fix
//! arrived at from a dais deserves a test taken from the air.
//!
//! **And the camera shot up with her.** At a fixed downward pitch the patch of
//! floor under the crosshair is decided by how high the eye is, so an eye that
//! climbs five metres sweeps that patch metres further out while the player's
//! hand is perfectly still. That one is what the rest of this file is about:
//! the camera now declines to follow for as long as the rise is still carrying
//! her *toward* the crosshair. See `crate::camera`.
//!
//! What the two have in common is that the grounded game is untouched -- the
//! hold is zero metres whenever the pitch zones already have her at the
//! reticle, which is every steep aim and everything above level.

use sim::aim::{self, Scene};
use sim::{Class, Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

/// Degrees below the horizon, in the wire's own unit.
fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

/// An Elementalist on the clear lane, well away from the raised platforms, so
/// that a ray aimed down meets plain floor.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(12.0, 0.0, 8.0);
    w
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    }
}

fn with_scene<T>(w: &World, ask: impl FnOnce(&Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
}

fn air_bolt_reach() -> Fx {
    sim::moves::get(Class::Elementalist, sim::moves::elementalist::AIR_BOLT).reach
}

/// Where the crosshair is, and where a shot aimed along it would end.
fn pointed(w: &World, pitch: i16) -> (aim::Sighted, aim::Path) {
    let look = Input::looking_at(0, LOOK_RIGHT, pitch);
    let reach = air_bolt_reach();
    with_scene(w, |scene| {
        (
            aim::sight(0, look, reach, scene),
            aim::skillshot_path(0, look, reach, scene),
        )
    })
}

/// Where the eye is for player one, right now.
fn eye(w: &World, pitch: i16) -> V3 {
    sim::camera::eye(
        w.players[0].pos,
        Input::looking_at(0, LOOK_RIGHT, pitch),
        w.players[0].aloft,
    )
}

// ---------------------------------------------------------------------------
// The shot goes where the crosshair is
// ---------------------------------------------------------------------------

#[test]
fn a_shot_aimed_at_the_floor_lands_on_it_from_any_height() {
    // **The bug, as a property, taken from the air.** Put the crosshair on a
    // patch of floor and the shot has to arrive over *that patch*, at a height
    // that catches somebody standing on it -- however far above the floor the
    // caster happens to be. It used to arrive at her own height, which from
    // five metres up is metres above the thing she pointed at, and from the
    // seat the shot simply left in a direction nobody chose.
    //
    // The rule itself is `aim::standing_middle`, and it was arrived at from the
    // other end: from the top of a platform the same mistake sent every shot
    // over the head of anybody on the floor. What is pinned here is that the
    // fix holds in the air too, where the height is not a platform's and
    // changes every frame.
    //
    // Up to the top of the floatiest jump in the game. Higher than that a
    // shallow aim runs out of range before it runs out of sky, which is the
    // move's own reach rather than anything about the rule.
    let mut landed_at: Option<Fx> = None;
    for height in [0.0, 0.5, 2.0, 5.0, 7.0] {
        let mut w = elementalist();
        w.players[0].pos.y = metres(height);
        w.players[0].grounded = height == 0.0;
        let (seen, shot) = pointed(&w, down(30));
        assert_eq!(
            seen.met,
            aim::Met::Ground,
            "at {height} m the fixture's crosshair was not on the floor at all"
        );
        let wide = V3::new(shot.to.x.sub(seen.at.x), Fx::ZERO, shot.to.z.sub(seen.at.z)).flat_len();
        assert_eq!(
            wide.raw(),
            0,
            "at {height} m up, the crosshair was on ({:.2}, {:.2}) and the shot went to \
             ({:.2}, {:.2}) -- {:.2} m wide of it",
            seen.at.x.to_f32_for_render(),
            seen.at.z.to_f32_for_render(),
            shot.to.x.to_f32_for_render(),
            shot.to.z.to_f32_for_render(),
            wide.to_f32_for_render()
        );

        // Inside a body standing on that spot, which is the whole of what the
        // lift is for. Asserted as the *property* rather than against the
        // number, so that retuning where in a body a shot lands does not come
        // back as a failure about the air.
        let over = shot.to.y.sub(seen.at.y);
        assert!(
            over.raw() > 0 && over.raw() < sim::tuning::body_height().raw(),
            "at {height} m up, the shot arrived {:.2} m over the spot it was aimed at, \
             and a body standing there is {:.2} m tall",
            over.to_f32_for_render(),
            sim::tuning::body_height().to_f32_for_render()
        );

        // And the same height every time: the lift is measured from the ground
        // the ray met, so how high the caster is may not enter into it. This is
        // the assertion the old rule failed.
        match landed_at {
            None => landed_at = Some(over),
            Some(before) => assert_eq!(
                over.raw(),
                before.raw(),
                "the shot landed {:.2} m over the spot from {height} m up and {:.2} m \
                 over it from the floor -- the caster's own height is still in the rule",
                over.to_f32_for_render(),
                before.to_f32_for_render()
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// The camera holds still while you climb
// ---------------------------------------------------------------------------

/// How far the camera declines to follow a fighter upward, at this pitch: the
/// whole of the hold, read off the two ends of it.
fn hold_at(pitch: i16) -> Fx {
    let turns = Input::looking_at(0, LOOK_RIGHT, pitch).pitch_turns();
    sim::camera::ball(turns, Fx::ZERO)
        .centre
        .sub(sim::camera::ball(turns, Fx::ONE).centre)
}

#[test]
fn looking_down_a_little_the_camera_has_room_to_hold() {
    // The case the whole thing is for: a shallow downward aim, which is where
    // an Elementalist spends her air time and where the sweep was worst.
    let room = hold_at(down(30));
    assert!(
        room.raw() > metres(1.0).raw(),
        "the camera will only hold still for {:.2} m of climb, which is less than a \
         short hop",
        room.to_f32_for_render()
    );
}

#[test]
fn the_hold_runs_out_as_you_look_further_down() {
    // **The geometry answering the awkward case on its own.** The further down
    // you look, the further up the screen the floor zone has already carried
    // you, and the less room there is left to carry you with. At the bottom of
    // the range the fighter is at the crosshair with their feet on the ground,
    // so the hold is nothing at all -- which is what you want when the thing
    // you are looking at is the patch of floor you are standing over. Holding
    // the camera there would put you *above* the reticle you are aiming with.
    let mut last = hold_at(down(30));
    for degrees in [40, 50, 60, 70, 80, 85] {
        let room = hold_at(down(degrees));
        assert!(
            room.raw() <= last.raw(),
            "looking {degrees} degrees down left {:.2} m of room, more than the \
             {:.2} m a shallower angle had",
            room.to_f32_for_render(),
            last.to_f32_for_render()
        );
        last = room;
    }
    assert_eq!(
        hold_at(down(85)).raw(),
        0,
        "at the look-down limit the camera still held {:.2} m of climb back, which \
         would carry the fighter over the crosshair",
        hold_at(down(85)).to_f32_for_render()
    );
}

#[test]
fn looking_level_or_up_there_is_nothing_to_hold_either() {
    // Past the turn the pitch zones are already handing over to the fighter's
    // own eye, which frames them higher than this ever would.
    for pitch in [0, -down(20), -down(60)] {
        assert_eq!(
            hold_at(pitch).raw(),
            0,
            "the camera held {:.2} m back while looking level or up",
            hold_at(pitch).to_f32_for_render()
        );
    }
}

#[test]
fn the_camera_barely_moves_while_a_jump_is_still_climbing() {
    // The sentence the design asks for, measured: *you* move toward the
    // crosshair and the camera does not move at all. Held to the part of the
    // climb the hold covers -- past that it follows again, which is the next
    // test.
    let mut w = elementalist();
    let pitch = down(30);
    let eye_standing = eye(&w, pitch);
    let mut worst = Fx::ZERO;
    let mut climbed = Fx::ZERO;
    run(&mut w, 1, Input::SPACE, pitch);
    for _ in 0..11 {
        run(&mut w, 1, Input::SPACE, pitch);
        climbed = w.players[0].pos.y;
        let moved = eye(&w, pitch).y.sub(eye_standing.y).abs();
        if moved.raw() > worst.raw() {
            worst = moved;
        }
    }
    assert!(
        climbed.raw() > metres(2.5).raw(),
        "the fixture only climbed {:.2} m, so it is not testing a climb",
        climbed.to_f32_for_render()
    );
    assert!(
        worst.raw() < metres(0.5).raw(),
        "she climbed {:.2} m and the camera went up {:.2} m with her -- it is supposed \
         to be holding still while she rises toward the crosshair",
        climbed.to_f32_for_render(),
        worst.to_f32_for_render()
    );
}

#[test]
fn the_crosshair_keeps_pointing_at_the_same_patch_of_floor() {
    // **What the hold actually buys**, and the reason it is worth having at
    // all: the spot under the crosshair is set by how high the eye is, so an
    // eye that holds still is an aim that holds still. This is the complaint
    // stated as a measurement -- the player's hand never moves, so neither
    // should the thing they are pointing at.
    let mut w = elementalist();
    let pitch = down(30);
    let (standing, _) = pointed(&w, pitch);
    run(&mut w, 1, Input::SPACE, pitch);
    let mut worst = Fx::ZERO;
    for _ in 0..11 {
        run(&mut w, 1, Input::SPACE, pitch);
        let (seen, _) = pointed(&w, pitch);
        let wandered = V3::new(
            seen.at.x.sub(standing.at.x),
            Fx::ZERO,
            seen.at.z.sub(standing.at.z),
        )
        .flat_len();
        if wandered.raw() > worst.raw() {
            worst = wandered;
        }
    }
    assert!(
        worst.raw() < metres(1.0).raw(),
        "the aim point wandered {:.2} m across the floor during a climb the player \
         never steered",
        worst.to_f32_for_render()
    );
}

#[test]
fn past_the_hold_the_camera_follows_again() {
    // It is a hold, not a detachment. Once the rise has carried her up to the
    // crosshair there is nothing left to gain by staying behind, and a camera
    // that kept sinking would leave her off the top of the screen.
    let mut w = elementalist();
    let pitch = down(30);
    let eye_standing = eye(&w, pitch);
    run(&mut w, 1, Input::SPACE, pitch);
    run(&mut w, 30, Input::SPACE, pitch);
    let peak = w.players[0].pos.y;
    let risen = eye(&w, pitch).y.sub(eye_standing.y);
    assert!(
        peak.raw() > metres(4.0).raw(),
        "the fixture only reached {:.2} m",
        peak.to_f32_for_render()
    );
    assert!(
        risen.raw() > metres(1.0).raw(),
        "at the top of a {:.1} m jump the camera had still not started following, \
         which would leave her climbing out of frame",
        peak.to_f32_for_render()
    );
    assert!(
        risen.raw() < peak.raw(),
        "the camera went up {:.2} m for a {:.2} m jump -- it is following exactly \
         rather than holding anything back",
        risen.to_f32_for_render(),
        peak.to_f32_for_render()
    );
}

#[test]
fn a_short_hop_is_held_the_whole_way() {
    // The case that has to feel ordinary, because it is the one that happens
    // by accident. A tap of the jump button should not swing the view at all:
    // the whole hop fits inside the hold, so the camera simply stays where it
    // was and she bobs up toward the reticle and back.
    let mut w = elementalist();
    let pitch = down(30);
    let eye_standing = eye(&w, pitch);
    let mut worst = Fx::ZERO;
    let mut peak = Fx::ZERO;
    // A tap: one frame of the button, then let go.
    run(&mut w, 1, Input::SPACE, pitch);
    for _ in 0..70 {
        run(&mut w, 1, 0, pitch);
        peak = peak.max(w.players[0].pos.y);
        worst = worst.max(eye(&w, pitch).y.sub(eye_standing.y).abs());
    }
    assert!(
        peak.raw() > metres(1.0).raw(),
        "the tap only got {:.2} m off the ground",
        peak.to_f32_for_render()
    );
    assert!(
        worst.raw() < metres(0.6).raw(),
        "a {:.2} m hop moved the camera {:.2} m",
        peak.to_f32_for_render(),
        worst.to_f32_for_render()
    );
}

#[test]
fn the_framing_lets_go_slower_than_it_takes_hold() {
    // Terminal velocity is eighteen metres a second, so the last few metres of
    // a long fall arrive in a handful of frames. Height alone would snap the
    // framing through its whole travel in those frames; letting go slower than
    // it takes hold turns that into a glide. The two rates are knobs, and
    // setting them equal turns the asymmetry off.
    let up = sim::camera::aloft_step(Fx::ZERO, Fx::ONE);
    let back = Fx::ONE.sub(sim::camera::aloft_step(Fx::ONE, Fx::ZERO));
    assert!(
        up.raw() > 0 && back.raw() > 0,
        "the framing does not move at all: {up:?} up, {back:?} back"
    );
    assert!(
        up.raw() >= back.raw(),
        "the framing lets go faster than it takes hold, which is the wrong way round: \
         a fall is what needs the smoothing"
    );
}

#[test]
fn the_framing_is_an_s_curve_in_height() {
    // Flat at the bottom so a hop barely moves it, flat at the top so arriving
    // at full height is not a jolt, and monotone in between so there is never a
    // height at which climbing further pulls the camera back.
    let sample = |m: f32| sim::camera::aloft_target(metres(m));
    let mut last = Fx::ZERO;
    for step in 0..=20 {
        let now = sample(step as f32 * 0.5);
        assert!(
            now.raw() >= last.raw(),
            "climbing from {:.1} m to {:.1} m moved the framing backwards",
            (step - 1) as f32 * 0.5,
            step as f32 * 0.5
        );
        last = now;
    }
    assert_eq!(
        sample(0.0).raw(),
        0,
        "standing on the floor is not framed as standing"
    );
    assert!(
        // Within the bisection's own resolution of one: `Curve::at` finds its
        // parameter by twenty halvings, so the top of the curve arrives a
        // fraction short of exactly one and always will.
        sample(40.0).raw() > Fx::ratio(99, 100).raw(),
        "no height at all reaches the airborne framing: {:.3} at forty metres",
        sample(40.0).to_f32_for_render()
    );
    // The S: the first quarter of the climb buys less than a quarter of the
    // swing, and the last quarter buys less than a quarter of what is left.
    let quarter = sim::camera::aloft_target(metres(0.75));
    assert!(
        quarter.raw() < Fx::ratio(1, 4).raw(),
        "the framing is linear off the floor rather than easing in"
    );
}

// ---------------------------------------------------------------------------
// It is simulation state, not decoration
// ---------------------------------------------------------------------------

#[test]
fn two_peers_framing_differently_is_a_desync_rather_than_a_mystery() {
    // The camera decides where abilities land, so the framing is in the
    // checksum like every other number that does. A value with memory that was
    // left out of the snapshot would also come back wrong from a rollback --
    // the same failure, arriving as a jump that re-simulated into a different
    // shot.
    let mut w = elementalist();
    run(&mut w, 1, Input::SPACE, down(30));
    run(&mut w, 10, Input::SPACE, down(30));
    let honest = w.checksum();
    assert!(
        w.players[0].aloft.raw() > 0,
        "the fixture never left the ground"
    );
    w.players[0].aloft = Fx::ZERO;
    assert_ne!(
        w.checksum(),
        honest,
        "two peers framing the fight differently checksum the same, so the desync \
         would arrive later as abilities landing in different places"
    );
}
