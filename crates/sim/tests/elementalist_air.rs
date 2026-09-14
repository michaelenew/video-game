//! The Elementalist with her feet off the floor.
//!
//! Three moves on the three buttons she already uses, and the row is where her
//! feet are — the answer `docs/design/README.md` leaves open under **Aerials**,
//! given to a second class. What is asserted here is what a player would
//! notice if any of it broke:
//!
//! - the same buttons mean different moves in the air, and shift does not
//!   reach up there;
//! - both air shots **travel**, which nothing she throws standing up does;
//! - the Gale is worth what it has *become* — small at her hand, heavy at the
//!   tip — which is the whole of its spacing;
//! - Landfall's wind-up ends on the **floor** rather than on a frame count, so
//!   the descent is as long as the height she chose;
//! - and a foe can take her out of it on the way down, which is the point of
//!   the wind-up being long in the first place.
//!
//! See `docs/design/kits/elementalist.md` and `crate::gust`.

use sim::class::{Mechanic, Structure};
use sim::gust::Gale;
use sim::moves::elementalist as air;
use sim::state::{Action, SLOT_HEAVY, SLOT_POKE};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const E: u16 = Input::MECHANIC;
const SHIFT: u16 = Input::SHIFT;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

/// An Elementalist facing +X with clear floor in front of her, and somebody
/// far enough away to be out of everything unless a test moves them.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
}

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(a, LOOK_RIGHT, 0),
            Input::looking_at(b, LOOK_LEFT, 0),
        ]);
    }
}

/// Take her feet off the floor without moving her.
///
/// A fixture rather than a jump, because every one of these is about what the
/// buttons mean and where the shots go rather than about the arc she got up
/// there on. `grounded` is read when the input is, at the top of the frame, so
/// one press is all it takes.
fn aloft(w: &mut World, height: f32) {
    w.players[0].pos.y = metres(height);
    w.players[0].grounded = false;
}

/// Which move she is part-way through, whatever phase it is in.
fn doing(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. } => Some(kind),
        _ => None,
    }
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn shots(w: &World) -> Vec<sim::gust::Gust> {
    w.gusts.iter().flatten().copied().collect()
}

/// Ask `aim` something about the world as it stands, the way the simulation
/// builds the same question.
fn with_scene<T>(w: &World, ask: impl FnOnce(&sim::aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
}

/// The pitch that puts the crosshair on the floor at somebody's feet.
///
/// **Aim the way a player does**, rather than typing in an angle — the idiom
/// `tests/beam.rs` is built on, and for the same reason: an angle typed in is
/// an assertion about where the camera sits. Looking at the floor is also what
/// makes the shot travel *level* at her cast height (`aim::skillshot_path`),
/// which is what puts it through a standing body at any distance instead of
/// sailing over the far one.
fn crosshair_onto_the_floor_at(w: &World, target: V3, reach: Fx) -> i16 {
    (-800..0)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, LOOK_RIGHT, pitch);
            let seen = with_scene(w, |scene| sim::aim::sight(0, look, reach, scene));
            let gap =
                V3::new(seen.at.x.sub(target.x), Fx::ZERO, seen.at.z.sub(target.z)).flat_len();
            (gap.raw() < metres(0.5).raw()).then_some(pitch)
        })
        .expect("no pitch puts the crosshair anywhere near those feet")
}

/// What one press of `button` starts, with her feet wherever `height` puts
/// them.
fn pressing(button: u16, height: Option<f32>) -> Option<u8> {
    let mut w = elementalist();
    if let Some(h) = height {
        aloft(&mut w, h);
    }
    run(&mut w, 1, button, 0);
    doing(&w)
}

// ---------------------------------------------------------------------------
// The row is where her feet are
// ---------------------------------------------------------------------------

#[test]
fn the_same_buttons_mean_different_moves_off_the_ground() {
    // The Champion's grid, read one class further: the button never changes
    // meaning -- left is the cheap one, right is the committed one -- and the
    // row is the situation. A player who has learnt her on the floor has
    // learnt most of her in the air.
    assert_eq!(pressing(L, None), Some(SLOT_POKE), "left click, standing");
    assert_eq!(pressing(R, None), Some(SLOT_HEAVY), "right click, standing");
    assert_eq!(
        pressing(L, Some(2.0)),
        Some(air::AIR_BOLT),
        "left click in the air should be the Air bolt"
    );
    assert_eq!(
        pressing(R, Some(2.0)),
        Some(air::GALE),
        "right click in the air should be the Gale"
    );
    assert_eq!(
        pressing(E, Some(2.0)),
        Some(air::LANDFALL),
        "`E` in the air should be Landfall"
    );
}

#[test]
fn the_mechanic_key_is_still_an_instant_on_the_floor() {
    // The half of `E` that did not change. Standing up it raises a stone with
    // no frames at all -- nothing to punish, because there is nothing there --
    // and only off the floor does it become a move with a wind-up.
    let mut w = elementalist();
    run(&mut w, 1, E, 0);
    assert_eq!(doing(&w), None, "raising a stone started a move");
    assert_eq!(stones_of(&w).len(), 1, "the mechanic key raised nothing");
}

#[test]
fn shift_does_not_reach_the_air_row() {
    // Shift plus left click is Fissure, a crack that races *along the ground*.
    // There is no airborne version of it, so the modifier is ignored rather
    // than being made to mean something it does not -- and ignoring it has to
    // come out as the Air bolt rather than as silence, or the input is eaten.
    assert_eq!(
        pressing(SHIFT | L, Some(2.0)),
        Some(air::AIR_BOLT),
        "shift in the air should throw what left click throws up there"
    );
}

// ---------------------------------------------------------------------------
// The two shots travel
// ---------------------------------------------------------------------------

#[test]
fn both_air_shots_leave_her_hand_and_fly() {
    // The thing that separates them from everything she throws standing up:
    // Bolt and Cataclysm are instant lines, resolved on the frame they come
    // out. These have a speed, so there is something to lead and something to
    // walk out of.
    for (button, gale) in [(L, Gale::Bolt), (R, Gale::Disc)] {
        let mut w = elementalist();
        aloft(&mut w, 2.0);
        let m = sim::moves::get(Class::Elementalist, gale.slot());
        run(&mut w, 1, button, 0);
        run(&mut w, m.startup as u32 + 1, 0, 0);

        let flying = shots(&w);
        assert_eq!(flying.len(), 1, "{:?}: nothing left her hand", gale);
        let first = flying[0];
        assert_eq!(first.gale, gale, "the wrong shot came out");

        run(&mut w, 6, 0, 0);
        let later = shots(&w);
        assert_eq!(later.len(), 1, "{:?}: the shot vanished", gale);
        assert!(
            later[0].travelled.raw() > first.travelled.raw(),
            "{:?}: the shot is not travelling",
            gale
        );
        assert!(
            later[0].pos.x.raw() > first.pos.x.raw(),
            "{:?}: the shot is not going the way she aimed it",
            gale
        );
    }
}

#[test]
fn the_air_bolt_outranges_everything_she_throws_standing_up() {
    // What being in the air buys. She is committed to an arc and cannot walk
    // out of what she started, so the shot has to be worth the position --
    // and reach is the thing this class trades in.
    let grounded = (0..4)
        .map(|slot| sim::moves::get(Class::Elementalist, slot).reach)
        .max_by_key(|r| r.raw())
        .expect("she has grounded moves");
    let air = sim::moves::get(Class::Elementalist, air::AIR_BOLT).reach;
    assert!(
        air.raw() > grounded.raw(),
        "the Air bolt reaches {:.1} m and something she can throw standing up reaches {:.1}",
        air.to_f32_for_render(),
        grounded.to_f32_for_render()
    );
}

#[test]
fn a_shot_expires_at_its_own_range() {
    // It has to end on something. A shot aimed at nothing in particular is the
    // common case, and the honest answer is the range in its own row.
    let mut w = elementalist();
    aloft(&mut w, 2.0);
    let m = sim::moves::get(Class::Elementalist, air::AIR_BOLT);
    run(&mut w, 1, L, 0);
    // Long enough to cover the whole range at the shot's own speed, and then
    // some.
    let legs = (m.reach.raw() as i64 * 60 / t::air_bolt_speed().raw().max(1) as i64) as u32;
    run(&mut w, m.startup as u32 + legs + 8, 0, 0);
    assert!(
        shots(&w).is_empty(),
        "the shot outlived its own range and is still in the air"
    );
}

// ---------------------------------------------------------------------------
// The Gale grows
// ---------------------------------------------------------------------------

#[test]
fn the_gale_opens_from_her_hand_to_its_full_size() {
    // The move, as a shape. It leaves small and arrives large, and the size it
    // has is the size the hit test uses -- `Gust::girth` is read by the hit
    // test and by the renderer alike.
    let full = sim::moves::get(Class::Elementalist, air::GALE).radius;
    let hand = Gale::Disc.swell(Fx::ZERO);
    let tip = Gale::Disc.swell(Fx::ONE);
    assert!(
        hand.raw() < tip.raw(),
        "the Gale does not open: {:.2} at the hand, {:.2} at the tip",
        hand.to_f32_for_render(),
        tip.to_f32_for_render()
    );
    assert_eq!(tip.raw(), Fx::ONE.raw(), "the Gale never reaches full size");
    assert_eq!(
        Gale::Bolt.swell(Fx::ZERO).raw(),
        Fx::ONE.raw(),
        "the Air bolt should be all of itself the whole way -- only the disc grows"
    );
    assert!(
        full.mul(hand).raw() > 0,
        "the Gale leaves her hand with no volume at all, so point blank is a whiff"
    );
}

#[test]
fn the_gale_is_full_size_well_before_it_is_out_of_range() {
    // **How far it goes and how fast it opens are two decisions**, and they are
    // two knobs. They were one for a day, with the growth measured against the
    // reach, which meant that bumping the range silently halved the disc's size
    // everywhere a fighter actually stands -- a retune nobody asked for wearing
    // a range change's clothes.
    let reach = sim::moves::get(Class::Elementalist, air::GALE).reach;
    assert!(
        t::gale_grow().raw() < reach.raw(),
        "the Gale only reaches full size at {:.1} m and its range is {:.1} m, so it \
         never gets there before it expires",
        t::gale_grow().to_f32_for_render(),
        reach.to_f32_for_render()
    );
    // And the opening itself is measured against that distance rather than the
    // reach: at the growth distance it is all of itself, whatever the range is.
    let at = |travelled: f32| {
        let mut shot = sim::gust::Gust {
            pos: V3::ZERO,
            dir: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            owner: 0,
            travelled: metres(travelled),
            gale: Gale::Disc,
        };
        shot.travelled = metres(travelled);
        shot.swell()
    };
    let grow = t::gale_grow().to_f32_for_render();
    assert_eq!(
        at(grow).raw(),
        Fx::ONE.raw(),
        "the disc is still opening at the distance it is supposed to be open by"
    );
    assert_eq!(
        at(grow * 1.5).raw(),
        Fx::ONE.raw(),
        "the disc kept growing past full size"
    );
    assert!(
        at(grow * 0.5).raw() < Fx::ONE.raw(),
        "the disc is already full size halfway through its opening"
    );
}

#[test]
fn the_gale_is_a_frisbee_rather_than_a_ball() {
    // **The disc lies flat, and the hit test is what says so.** What the shot
    // occupies is a horizontal disc of its current radius sweeping along its
    // line: `aim::first_along` swells the victim's standing cylinder by the
    // girth in *radius* and never in height. So the same displacement catches
    // you sideways and does not catch you upward -- which is the whole of the
    // difference between a frisbee and a ball, and is the one thing the shape
    // has to mean.
    //
    // Asked of the volume directly, the way `gust::step` asks it, rather than
    // by standing somebody up in the air: a fighter parked four metres up with
    // nothing under him simply falls into the line and is hit on the way down,
    // which measures gravity rather than the disc.
    //
    // Pinned because the renderer has to draw the same thing. It drew the disc
    // face-on to its own travel for a while -- a picture of a wall of air
    // rather than of a frisbee -- and there was nothing to catch it.
    //
    // The drawing is tipped into the plane of the throw now (`place_discs`),
    // which costs this nothing: the disc's width lies along `dir x axis`, and
    // that is horizontal whichever way the throw is pitched. Sideways still
    // catches you and straight up still does not, which is what is measured
    // here.
    let girth = sim::moves::get(Class::Elementalist, air::GALE).radius;
    let mut w = elementalist();
    // A level line at chest height, nine metres of it, down the clear lane.
    let line = sim::aim::Path {
        from: at(-6.0, 1.0, 8.0),
        to: at(3.0, 1.0, 8.0),
    };
    // Well inside the disc's own radius, and well outside a body's.
    let off = girth.to_f32_for_render() * 0.6;

    w.players[1].pos = at(3.0, 0.0, 8.0 + off);
    let beside = with_scene(&w, |scene| {
        sim::aim::first_along(
            line,
            girth,
            0,
            scene,
            sim::aim::Targets::none().fighters(true),
        )
    });
    assert!(
        beside.is_some(),
        "somebody standing {off:.2} m to the side of the line was missed by a disc \
         {:.2} m across, so it has no width at all",
        girth.to_f32_for_render() * 2.0
    );

    // The same displacement, upward: his feet start that far above the line, so
    // he is clear of it by the same margin that caught him sideways.
    w.players[1].pos = at(3.0, 1.0 + off, 8.0);
    let above = with_scene(&w, |scene| {
        sim::aim::first_along(
            line,
            girth,
            0,
            scene,
            sim::aim::Targets::none().fighters(true),
        )
    });
    assert!(
        above.is_none(),
        "the same {off:.2} m of clearance missed sideways and connected upward -- the \
         disc is being tested as a ball, and the renderer draws it flat"
    );
}

#[test]
fn a_gale_is_worth_more_at_the_tip_than_at_the_hand() {
    // **The spacing, inverted.** Every other projectile in the game is worth
    // the same wherever it lands; this one is worth what it has become, which
    // is what makes standing close to her the answer to it. The same sentence
    // Flame spitter is written around, said as a thing that flies.
    let reach = sim::moves::get(Class::Elementalist, air::GALE).reach;
    let hurt = |gap: f32| {
        let mut w = elementalist();
        w.players[0].pos = at(-6.0, 0.0, 8.0);
        w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
        aloft(&mut w, 0.0);
        let pitch = crosshair_onto_the_floor_at(&w, w.players[1].pos, reach);
        let before = w.players[1].health;
        for f in 0..160 {
            let a = if f == 0 { R } else { 0 };
            w.advance([
                Input::looking_at(a, LOOK_RIGHT, pitch),
                Input::looking_at(0, LOOK_LEFT, 0),
            ]);
        }
        before - w.players[1].health
    };
    let near = hurt(3.0);
    let far = hurt(13.0);
    assert!(
        near > 0,
        "a disc thrown at somebody three metres away missed"
    );
    assert!(
        far > near,
        "the Gale dealt {far} at the tip and {near} at her hand -- the disc is not \
         growing into anything"
    );
}

// ---------------------------------------------------------------------------
// Landfall
// ---------------------------------------------------------------------------

#[test]
fn the_wind_up_waits_for_the_floor() {
    // **The one move in the game whose startup ends on a place.** Its frames
    // are what she is guaranteed to owe; the descent is over when her feet
    // arrive, which from twelve metres up takes longer than the row says. A
    // plunge that went active in mid-air would put the slab of rock in the sky.
    let mut w = elementalist();
    aloft(&mut w, 12.0);
    let m = sim::moves::get(Class::Elementalist, air::LANDFALL);
    run(&mut w, 1, E, 0);
    run(&mut w, m.startup as u32 + 2, 0, 0);

    assert!(
        matches!(w.players[0].action, Action::Startup { kind, .. } if kind == air::LANDFALL),
        "the plunge went active {} frames in, still {:.1} m off the floor",
        m.startup + 2,
        w.players[0].pos.y.to_f32_for_render()
    );
    assert!(
        stones_of(&w).is_empty(),
        "a slab came up before she arrived"
    );

    run(&mut w, 120, 0, 0);
    assert!(w.players[0].grounded, "she never landed");
    assert_eq!(
        stones_of(&w).len(),
        1,
        "she landed and drove nothing out of the floor"
    );
}

#[test]
fn the_descent_is_a_descent() {
    // It is driven rather than fallen, so that how long the opponent gets to
    // answer is set by the height she chose rather than by how long gravity
    // has had to work. The hang at the top is the guaranteed part of the
    // telegraph; the drop is the rest of it.
    let mut w = elementalist();
    aloft(&mut w, 12.0);
    let m = sim::moves::get(Class::Elementalist, air::LANDFALL);
    run(&mut w, 1, E, 0);

    let hanging = w.players[0].pos.y;
    run(&mut w, m.air_stall as u32 - 2, 0, 0);
    assert!(
        (w.players[0].pos.y.sub(hanging)).abs().raw() < metres(0.4).raw(),
        "she fell {:.2} m during the wind-up -- the hang is not holding her",
        hanging.sub(w.players[0].pos.y).to_f32_for_render()
    );

    let held = w.players[0].pos.y;
    run(&mut w, 6, 0, 0);
    assert!(
        w.players[0].pos.y.raw() < held.raw(),
        "the wind-up ran out and she is not coming down"
    );
}

#[test]
fn a_hit_on_the_way_down_ends_the_plunge() {
    // **Why the wind-up is long.** Everything about the descent is an ordinary
    // startup, so being hit during it interrupts it exactly the way being hit
    // during any other wind-up does -- and the slab she was about to drive up
    // never appears. The control is the same fixture with nobody swinging.
    let plunge = |interfere: bool| {
        let mut w = elementalist();
        w.players[0].pos = at(0.0, 0.0, 8.0);
        // Close enough for a Bash, which is a flat disc and does not care how
        // far above him she is.
        w.players[1].pos = at(1.0, 0.0, 8.0);
        aloft(&mut w, 6.0);
        run(&mut w, 1, E, 0);
        // Through the hang, so what lands, lands on the descent.
        let hang = sim::moves::get(Class::Elementalist, air::LANDFALL).air_stall as u32;
        run(&mut w, hang, 0, 0);
        run(&mut w, 40, 0, if interfere { L } else { 0 });
        w
    };

    let stopped = plunge(true);
    assert!(
        stones_of(&stopped).is_empty(),
        "she was hit on the way down and still drove a slab out of the floor"
    );
    assert!(
        !matches!(doing(&stopped), Some(k) if k == air::LANDFALL),
        "she was hit on the way down and is still plunging"
    );

    let unopposed = plunge(false);
    assert_eq!(
        stones_of(&unopposed).len(),
        1,
        "nobody touched her and the plunge still came to nothing -- the fixture \
         proves nothing about the interrupt"
    );
}

#[test]
fn landfall_drives_a_slab_up_in_front_of_her() {
    // Where it goes: a fixed distance along her facing, on the floor. **The
    // one structure in the game the crosshair does not place** -- she is
    // landing, not aiming -- so a mouse flick on the way down cannot put it
    // behind her.
    let mut w = elementalist();
    w.players[0].pos = at(0.0, 0.0, 8.0);
    aloft(&mut w, 4.0);
    run(&mut w, 1, E, 0);
    run(&mut w, 120, 0, 0);

    let slab = stones_of(&w);
    assert_eq!(slab.len(), 1, "no slab");
    let slab = slab[0];
    let ahead = slab.at.x.sub(w.players[0].pos.x);
    assert!(
        ahead.raw() > 0,
        "the slab came up {:.2} m behind her",
        ahead.neg().to_f32_for_render()
    );
    assert!(
        ahead.sub(t::landfall_ahead()).abs().raw() < metres(0.6).raw(),
        "the slab came up {:.2} m ahead, and the knob says {:.2}",
        ahead.to_f32_for_render(),
        t::landfall_ahead().to_f32_for_render()
    );
    assert!(
        slab.rise < t::structure_rise(),
        "the slab takes {} frames to come out and a raised stone takes {} -- it is \
         supposed to be quicker, because the plunge was its telegraph",
        slab.rise,
        t::structure_rise()
    );
}

#[test]
fn the_slab_leans_away_from_her() {
    // Forty-five degrees out of the floor, pointing away: up and back in equal
    // measure, which is what makes it a shove as well as a pop. The geometry on
    // its own, before anything is standing on it.
    let away = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    let slab = Structure::slammed(at(2.0, 0.0, 0.0), away);
    assert!(
        slab.erupt.x.raw() > 0,
        "the lean does not point away from her"
    );
    assert!(
        slab.erupt.y.raw() > 0,
        "the lean does not come out of the floor"
    );
    assert!(
        slab.erupt.x.sub(slab.erupt.y).abs().raw() < Fx::ratio(1, 20).raw(),
        "the lean is {:.2} out and {:.2} up, which is not the forty-five degrees \
         `landfall_tilt` asks for",
        slab.erupt.x.to_f32_for_render(),
        slab.erupt.y.to_f32_for_render()
    );
    assert_eq!(
        Structure::raised(at(2.0, 0.0, 0.0)).erupt,
        V3::ZERO,
        "an ordinary stone should come up square and throw nobody"
    );
}

#[test]
fn a_leaning_eruption_throws_you_out_and_up() {
    // And what the lean is *for*. An ordinary eruption does damage and a
    // stagger and leaves you standing where you were; this one clears the
    // space she has just landed in.
    //
    // The horizontal is the half that only the lean can explain: a stone
    // coming up dead under somebody puts them on top of it and hands them the
    // speed its cap is climbing at, which is straight up and nothing else.
    let mut w = elementalist();
    w.players[0].pos = at(0.0, 0.0, 8.0);
    w.players[1].pos = at(2.0, 0.0, 8.0);
    let slab = Structure::slammed(w.players[1].pos, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    w.players[0].mechanic = Mechanic::Structures([Some(slab), None, None]);

    let mut thrown = V3::ZERO;
    for _ in 0..40 {
        run(&mut w, 1, 0, 0);
        if w.players[1].vel.x.raw() > thrown.x.raw() {
            thrown = w.players[1].vel;
        }
    }
    assert!(
        thrown.x.raw() > 0,
        "the slab erupted under him and pushed him nowhere"
    );
    assert!(
        thrown.y.raw() > 0,
        "the slab erupted under him and never took him off the floor"
    );
}

#[test]
fn the_slam_staggers_where_she_lands() {
    // The hit the move itself carries: low damage, and a stagger over the
    // patch of floor she arrives on. It is the reason the slab is worth
    // setting up -- whoever it catches is not free to walk away from what
    // comes up next to them.
    let mut w = elementalist();
    w.players[0].pos = at(0.0, 0.0, 8.0);
    w.players[1].pos = at(1.2, 0.0, 8.0);
    aloft(&mut w, 4.0);
    let before = w.players[1].health;
    run(&mut w, 1, E, 0);
    run(&mut w, 60, 0, 0);
    assert!(
        w.players[1].health < before,
        "she landed on top of him and he did not feel it"
    );
}

#[test]
fn the_slab_spends_the_cap_of_three() {
    // It is a structure like any other, and the cap is the resource. A move
    // that raised a fourth for free would be a way around the only cost the
    // mechanic has.
    let mut w = elementalist();
    w.players[0].pos = at(0.0, 0.0, 8.0);
    for _ in 0..3 {
        run(&mut w, 1, E, 0);
        run(&mut w, 8, 0, 0);
    }
    assert_eq!(stones_of(&w).len(), 3, "the fixture did not fill the cap");

    aloft(&mut w, 4.0);
    run(&mut w, 1, E, 0);
    run(&mut w, 120, 0, 0);
    assert_eq!(
        stones_of(&w).len(),
        3,
        "Landfall raised a fourth structure past the cap of three"
    );
    assert!(
        stones_of(&w).iter().any(|s| s.erupt != V3::ZERO),
        "the slab is not among them, so the cap ate the new one instead of the oldest"
    );
}
