//! The Galewing: the rules `docs/design/creatures/galewing.md` pins, as
//! sentences (§10).
//!
//! Most set the bird up somewhere and start a move as its brain would --
//! on the frame it is chosen, so the frame hook sees its first frame -- and
//! watch what it does to a fighter standing, crouching, dodging or riding.

use sim::aim::{self, Scene, Targets};
use sim::fixed::Fx;
use sim::monster::{Doing, Monster, mount_of};
use sim::species::SpeciesId;
use sim::species::galewing::{self as gw, Knob, fight, flight};
use sim::state::{Action, MAX_PLAYERS};
use sim::{Class, Input, V3, World};

/// A hunt against it, the second fighter out of it.
fn hunt() -> World {
    hunt_as(Class::Champion)
}

fn hunt_as(class: Class) -> World {
    let mut w = World::hunt_of([class; MAX_PLAYERS], SpeciesId::GALEWING);
    w.players[1].health = 0;
    // One frame, so its first-frame set-up has run.
    w.advance([Input::default(); MAX_PLAYERS]);
    w
}

fn beast(w: &World) -> Monster {
    w.monsters[0].expect("the creature")
}

/// The plateau's top.
fn plateau(w: &World) -> Fx {
    fight::base(w)
}

/// A point on the plateau, `x` and `z` metres from its middle.
fn on(w: &World, x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), plateau(w), Fx::from_int(z))
}

/// Put the bird in the air at `at`, heading `yaw`, thinking of nothing.
fn fly_at(w: &mut World, at: V3, yaw: Fx) {
    let f = flight::Flight {
        pos: at,
        speed: Knob::CruiseSpeed.fx(),
        vy: Fx::ZERO,
        yaw,
        yaw_rate: Fx::ZERO,
    };
    f.save(&mut w.lore);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = at;
    m.yaw = yaw;
    m.brain.grace = 0;
    m.brain.think_left = 900;
    fight::set_flag(m, fight::flag::ALOFT, true);
}

/// Keep its mind off choosing anything.
fn quiet(w: &mut World) {
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.think_left = m.brain.think_left.max(600);
}

/// Start a move as its brain would: on the frame it is chosen.
fn start(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind,
        left: gw::SPECIES.attack(kind).startup + 1,
    };
    m.hit_used = false;
    m.brain.last_move = kind;
}

/// The fighter standing at `at`, seen there by the bird.
fn stand(w: &mut World, at: V3) {
    w.players[0].pos = at;
    w.players[0].vel = V3::ZERO;
    w.players[0].grounded = true;
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.seen = at;
    m.brain.seen_vel = V3::ZERO;
    m.brain.target = 0;
}

/// A look along +x.
fn look() -> Input {
    Input::aimed(0, 0)
}

fn advance(w: &mut World, input: Input) {
    w.advance([input, Input::default()]);
    quiet(w);
}

/// The highest a full hop takes any class, over its feet.
fn highest_hop() -> Fx {
    let mut best = Fx::ZERO;
    for class in sim::class::ALL_CLASSES {
        let mut w = World::with_classes([class; MAX_PLAYERS]);
        for _ in 0..40 {
            w.advance([Input::default(); MAX_PLAYERS]);
        }
        let floor = w.players[0].pos.y;
        let held = Input::default().with(Input::SPACE);
        let mut top = floor;
        for _ in 0..240 {
            w.advance([held, Input::default()]);
            top = top.max(w.players[0].pos.y);
        }
        best = best.max(top.sub(floor));
    }
    best
}

/// The lowest point of any of its parts, in the world.
fn lowest(m: &Monster) -> Fx {
    let rig = m.rig();
    let mut low = Fx::from_int(10_000);
    for i in 0..gw::PART_COUNT {
        let sh = gw::SPECIES.shape(i);
        for c in 0..8 {
            let corner = V3::new(
                if c & 1 == 0 { sh.min.x } else { sh.max.x },
                if c & 2 == 0 { sh.min.y } else { sh.max.y },
                if c & 4 == 0 { sh.min.z } else { sh.max.z },
            );
            low = low.min(rig.part_to_world(i, corner).y);
        }
    }
    low
}

// ---------------------------------------------------------------------------
// Out of reach
// ---------------------------------------------------------------------------

/// **Circling, nothing of it is inside anybody's jump** from the plateau:
/// its lowest part, wing tips banked down included, is above the highest
/// full hop plus a body.
#[test]
fn nothing_on_the_circling_galewing_is_inside_any_standing_jump() {
    let mut w = hunt();
    let spot = on(&w, -20, -20);
    stand(&mut w, spot);
    let reach = plateau(&w)
        .add(highest_hop())
        .add(sim::tuning::body_height());
    let mut seen = 0;
    for _ in 0..900 {
        advance(&mut w, Input::default());
        let m = beast(&w);
        if !(fight::aloft(&m) && m.doing.free()) {
            continue;
        }
        seen += 1;
        assert!(
            lowest(&m).raw() > reach.raw(),
            "its lowest part at {:?} is inside a hop to {:?}",
            lowest(&m),
            reach
        );
    }
    assert!(seen > 600, "it circled for {seen} frames");
}

// ---------------------------------------------------------------------------
// The Stoop
// ---------------------------------------------------------------------------

/// A Stoop at a fighter standing at `at`, from overhead and to one side.
fn stoop_at(w: &mut World, at: V3) {
    let from = at.add(V3::new(Fx::from_int(-14), Fx::from_int(16), Fx::ZERO));
    fly_at(w, from, Fx::ZERO);
    stand(w, at);
    start(w, gw::STOOP);
    let m = w.monsters[0].as_mut().unwrap();
    m.aim_at(at);
    fight::set_aim_height(m, at.y);
}

/// Play a Stoop out, the fighter's input chosen each frame from how many
/// frames are left until its hit; did it land?
fn stooped(at: V3, mut act: impl FnMut(i32) -> Input) -> bool {
    let mut w = hunt();
    let at = V3::new(at.x, plateau(&w), at.z);
    stoop_at(&mut w, at);
    let a = gw::SPECIES.attack(gw::STOOP);
    let hp = w.players[0].health;
    for _ in 0..(a.startup + a.active + 2) {
        let until = match beast(&w).doing {
            Doing::Startup { left, .. } => left as i32 + 1,
            Doing::Active { .. } => 0,
            _ => -1,
        };
        advance(&mut w, act(until));
    }
    w.players[0].health < hp
}

/// **The Stoop is dodged, not walked out of** (§2): from the middle of its
/// circle, a walk begun as the aim locks is short of out; a dodge timed to
/// the hit is clean; from the circle's edge a walk works.
#[test]
fn the_stoop_is_dodged_not_walked_out_of() {
    let lock = Knob::StoopLock.raw();
    let away = Input::aimed(Input::W, 0);
    let middle = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-8));
    // Standing in the middle, walking out as the aim locks.
    assert!(
        stooped(middle, |until| if until <= lock { away } else { look() }),
        "walked out of the middle of a Stoop"
    );
    // A dodge with its invulnerable frames over the hit.
    assert!(
        !stooped(middle, |until| {
            if until == 3 {
                Input::aimed(Input::W | Input::SHIFT, 0)
            } else {
                look()
            }
        }),
        "a dodge at the hit was struck"
    );
    // From the edge of its circle, a walk begun with the tell is out in
    // time: the circle follows slower than a walk.
    let r = gw::SPECIES.attack(gw::STOOP).hit_radius;
    let edge = V3::new(r.sub(Fx::ratio(1, 2)), Fx::ZERO, Fx::from_int(-8));
    assert!(
        !stooped(edge, |_| away),
        "walking out from its edge did not work"
    );
    assert!(Knob::StoopTrack.fx().raw() < sim::tuning::move_speed().raw());
}

// ---------------------------------------------------------------------------
// The talons and the carry
// ---------------------------------------------------------------------------

/// A talon pass at a fighter standing in the open, played out with `input`
/// held: were they caught?
fn passed(input: Input) -> (bool, World) {
    let mut w = hunt();
    let at = on(&w, 0, -8);
    let from = at.add(V3::new(Fx::from_int(-20), Fx::from_int(14), Fx::ZERO));
    fly_at(&mut w, from, Fx::ZERO);
    stand(&mut w, at);
    start(&mut w, gw::TALON);
    let a = gw::SPECIES.attack(gw::TALON);
    let mut caught = false;
    for _ in 0..(a.startup + a.active + 2) {
        advance(&mut w, input);
        caught |= fight::carried(&w.lore) == Some(0);
        if caught {
            break;
        }
    }
    (caught, w)
}

/// **Crouch** (§2): the talons clear a crouched fighter by twenty
/// centimetres and catch a standing one.
#[test]
fn a_crouched_fighter_passes_under_the_talons_and_a_standing_one_does_not() {
    assert!(passed(look()).0, "a standing fighter was not caught");
    assert!(
        !passed(look().with(Input::CROUCH)).0,
        "a crouched fighter was caught"
    );
    let crouched = sim::tuning::body_height().mul(sim::tuning::crouch_height_scale());
    assert!(crouched.raw() < Knob::TalonHeight.fx().raw());
    assert!(sim::tuning::body_height().raw() > Knob::TalonHeight.fx().raw());
}

/// **Hit the legs** (§2): damage to its legs while it carries somebody
/// drops them where it is.
#[test]
fn hitting_the_legs_drops_the_carried_fighter() {
    let (caught, mut w) = passed(look());
    assert!(caught);
    // Up off the ground.
    for _ in 0..40 {
        advance(&mut w, look());
    }
    assert_eq!(fight::carried(&w.lore), Some(0), "let go by itself");
    let hp = beast(&w).health;
    w.monsters[0]
        .as_mut()
        .unwrap()
        .take_hit(gw::LEG_L, Knob::CarryFreeDamage.raw());
    assert!(beast(&w).health < hp);
    advance(&mut w, look());
    assert_eq!(
        fight::carried(&w.lore),
        None,
        "the legs were hit and held on"
    );
    // And they fall from where they were.
    let p = w.players[0];
    assert!(!p.grounded && p.pos.y.raw() > plateau(&w).raw());
    // Three hits do it too, whatever they are worth.
    let (_, mut w) = passed(look());
    for _ in 0..5 {
        advance(&mut w, look());
    }
    for _ in 0..Knob::CarryFreeHits.raw() {
        assert_eq!(fight::carried(&w.lore), Some(0));
        w.monsters[0].as_mut().unwrap().take_hit(gw::TALON_R, 1);
        advance(&mut w, look());
    }
    assert_eq!(fight::carried(&w.lore), None);
}

// ---------------------------------------------------------------------------
// The Downwash
// ---------------------------------------------------------------------------

/// A Downwash over `under`, with the fighter at `at`, played out: where did
/// they end up?
fn washed(under: V3, at: V3, input: Input) -> V3 {
    let mut w = hunt();
    let under = V3::new(under.x, plateau(&w), under.z);
    let at = V3::new(at.x, plateau(&w), at.z);
    let over = under.add(V3::new(Fx::ZERO, Knob::HoverHeight.fx(), Fx::ZERO));
    fly_at(&mut w, over, Fx::ZERO);
    stand(&mut w, at);
    // Its point is chosen as it commits, from where the fighter is: set
    // where the test wants it, after the commit.
    start(&mut w, gw::DOWNWASH);
    advance(&mut w, input);
    w.lore.set_word(fight::word::WASH_AT, fight::word_of(under));
    let a = gw::SPECIES.attack(gw::DOWNWASH);
    for _ in 0..(a.startup + a.active + 1) {
        advance(&mut w, input);
    }
    w.players[0].pos
}

/// **The lee** (§2): behind a standing stone, the push is nothing.
#[test]
fn the_downwash_does_not_move_a_fighter_behind_a_solid() {
    // The stone at (-12, -8): the bird hangs north of it, the fighter stands
    // close behind it to the south.
    let under = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-2));
    let behind = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ratio(-95, 10));
    let ended = washed(under, behind, look());
    assert!(
        sim::math::wide_flat_dist(ended, behind).raw() < Fx::ratio(1, 10).raw(),
        "pushed from behind the stone: {behind:?} to {ended:?}"
    );
    // In the open, at the same distance, it is blown out to the ring's
    // edge.
    let open = V3::new(Fx::from_int(-5), Fx::ZERO, Fx::ratio(-95, 10));
    let under_open = V3::new(Fx::from_int(-5), Fx::ZERO, Fx::from_int(-2));
    let ended = washed(under_open, open, look());
    let out = sim::math::wide_flat_dist(ended, under_open);
    assert!(
        out.raw() >= Knob::WashRadius.fx().sub(Fx::ONE).raw(),
        "not pushed in the open: {open:?} to {ended:?}"
    );
}

/// **What is drawn through the windup is where the hit lands** (§6): every
/// lee disc the Downwash's sign draws on its last windup frame is still
/// a lee when the push comes -- a fighter standing in one is not moved --
/// and the ring it draws is centred where the push blows from.
#[test]
fn what_is_drawn_through_the_windup_is_where_the_hit_lands() {
    let mut w = hunt();
    // The stone at (-12, -8), with the bird hanging just north of it.
    let under = V3::new(Fx::from_int(-12), plateau(&w), Fx::from_int(-4));
    let over = under.add(V3::new(Fx::ZERO, Knob::HoverHeight.fx(), Fx::ZERO));
    fly_at(&mut w, over, Fx::ZERO);
    let at = on(&w, -12, -3);
    stand(&mut w, at);
    start(&mut w, gw::DOWNWASH);
    advance(&mut w, look());
    w.lore.set_word(fight::word::WASH_AT, fight::word_of(under));
    let a = gw::SPECIES.attack(gw::DOWNWASH);
    while matches!(beast(&w).doing, Doing::Startup { left, .. } if left > 0) {
        advance(&mut w, look());
    }
    let mut drawn = sim::sign::Signs::NONE;
    fight::signs(&w, &mut drawn);
    let lees: Vec<V3> = drawn
        .all
        .iter()
        .flatten()
        .filter(|s| s.says == sim::sign::Says::Clear)
        .map(|s| s.at)
        .collect();
    assert!(!lees.is_empty(), "no lee drawn behind the stone");
    advance(&mut w, look());
    assert!(
        matches!(
            beast(&w).doing,
            Doing::Active {
                kind: gw::DOWNWASH,
                ..
            }
        ),
        "the push did not come after {} frames of windup: {:?}",
        a.startup,
        beast(&w).doing
    );
    let blows = fight::wash_point(&w);
    assert_eq!(
        (blows.x, blows.z),
        (under.x, under.z),
        "the push blows from somewhere the ring was not drawn"
    );
    let field = sim::stones::gather(&w.players);
    let ground = w.terrain();
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    for at in lees {
        let mut p = w.players[0];
        p.pos = at;
        p.crouching = false;
        let push = fight::wash_push(&scene, blows, &p);
        assert_eq!(push, V3::ZERO, "a lee drawn at {at:?} is blown {push:?}");
    }
}

/// **The eye** (§2): under it, the air goes straight down. Crouching
/// outside it only slows the slide.
#[test]
fn the_eye_of_the_downwash_is_still() {
    let under = V3::new(Fx::from_int(-5), Fx::ZERO, Fx::from_int(-2));
    let eye = under.add(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    let ended = washed(under, eye, look());
    assert!(
        sim::math::wide_flat_dist(ended, eye).raw() < Fx::ratio(1, 10).raw(),
        "moved in the eye: {eye:?} to {ended:?}"
    );
    let open = V3::new(Fx::from_int(3), Fx::ZERO, Fx::from_int(-2));
    let crouched = washed(under, open, look().with(Input::CROUCH));
    let stood = washed(under, open, look());
    let c = sim::math::wide_flat_dist(crouched, open);
    let s = sim::math::wide_flat_dist(stood, open);
    assert!(
        c.raw() > 0 && c.raw() < s.raw(),
        "crouched {c:?}, stood {s:?}"
    );
}

// ---------------------------------------------------------------------------
// The volley
// ---------------------------------------------------------------------------

/// **Leave the lane sideways** (§2): every point of the volley's lane is
/// under feathers for longer than a dodge is invulnerable, so a dodge
/// through it is struck.
#[test]
fn the_feather_rake_outlasts_a_dodge_at_every_point_of_its_lane() {
    let a = gw::SPECIES.attack(gw::VOLLEY);
    let len = Knob::VolleyLength.fx();
    let iframes = sim::tuning::dodge_iframes() as u32;
    for step in 0..=22 {
        let s = len.mul(Fx::from_int(step)).div(Fx::from_int(22));
        let covered = (0..a.active)
            .filter(|gone| {
                let (lo, hi) = fight::raked(*gone);
                s.raw() >= lo.raw() && s.raw() <= hi.raw()
            })
            .count() as u32;
        assert!(
            covered > iframes,
            "{s:?} along the lane is raked for {covered} frames; a dodge is safe for {iframes}"
        );
    }
}

// ---------------------------------------------------------------------------
// Wings
// ---------------------------------------------------------------------------

/// **Up high it breaks wings and never crashes** (§4): hits on the wings
/// aloft fill their bars, not the poise.
#[test]
fn wing_hits_aloft_break_wings_and_never_crash_it() {
    let mut w = hunt();
    let spot = on(&w, -20, -20);
    stand(&mut w, spot);
    for _ in 0..30 {
        advance(&mut w, Input::default());
    }
    let bar = fight::bar(&beast(&w), 0);
    for _ in 0..12 {
        w.monsters[0].as_mut().unwrap().take_hit(gw::BLADE_L, 200);
        advance(&mut w, Input::default());
        let m = beast(&w);
        assert_eq!(m.poise, 0, "wing hits aloft filled the poise");
        assert!(
            !matches!(m.doing, Doing::Toppled { .. }),
            "crashed from high up"
        );
    }
    let m = beast(&w);
    assert!(fight::bar(&m, 0) < bar);
    assert!(
        fight::broken(&m, 0),
        "twelve hits of 200 did not break a 1400 bar"
    );
    assert!(fight::aloft(&m), "one wing broken grounded it");
    // One wing broken: it cannot climb above its ceiling.
    for _ in 0..600 {
        advance(&mut w, Input::default());
    }
    let ceiling = plateau(&w).add(Knob::BrokenCeiling.fx());
    assert!(beast(&w).pos.y.raw() <= ceiling.add(Fx::ONE).raw());
}

/// **Both broken, it never flies again** (§4).
#[test]
fn both_wings_broken_it_never_leaves_the_ground_again() {
    let mut w = hunt();
    let spot = on(&w, -10, -10);
    stand(&mut w, spot);
    for side in [gw::BLADE_L, gw::BLADE_R] {
        w.monsters[0]
            .as_mut()
            .unwrap()
            .take_hit(side, Knob::WingBar.raw());
        advance(&mut w, Input::default());
    }
    assert!(fight::grounded_for_good(&beast(&w)));
    // It comes down...
    let mut landed = false;
    for _ in 0..600 {
        w.advance([Input::default(); MAX_PLAYERS]);
        if !fight::aloft(&beast(&w)) {
            landed = true;
            break;
        }
    }
    assert!(landed, "both wings broken and still in the air");
    // ...and never goes up again, through a whole fight's worth of frames.
    for _ in 0..3600 {
        w.advance([Input::default(); MAX_PLAYERS]);
        if !matches!(w.phase, sim::state::Phase::Fighting) {
            break;
        }
        let m = beast(&w);
        assert!(!fight::aloft(&m), "it flew with both wings broken");
        assert!(m.pos.y.raw() <= fight::ground_at(&w, m.pos).add(Fx::ONE).raw());
    }
}

// ---------------------------------------------------------------------------
// The ride
// ---------------------------------------------------------------------------

/// Put the fighter on part `part` of the bird, at the middle of its top.
fn board(w: &mut World, part: usize) {
    let m = beast(w);
    let sh = gw::SPECIES.shape(part);
    let local = V3::new(
        sim::math::half(sh.min.x.add(sh.max.x)),
        sh.max.y,
        sim::math::half(sh.min.z.add(sh.max.z)),
    );
    let p = &mut w.players[0];
    p.mount = mount_of(0, part);
    p.local = local;
    p.pos = m.world_of(part, local);
    p.grounded = true;
    p.vel = V3::ZERO;
    p.grip_settle = 10;
}

/// Ride a barrel roll on `part`, braced: thrown?
fn rolled_off(part: usize) -> bool {
    let mut w = hunt();
    let at = on(&w, 0, -10).add(V3::new(Fx::ZERO, Knob::RideClimb.fx(), Fx::ZERO));
    fly_at(&mut w, at, Fx::ZERO);
    board(&mut w, part);
    let braced = look().with(Input::CROUCH);
    for _ in 0..12 {
        advance(&mut w, braced);
    }
    assert!(w.players[0].aboard(), "fell off before the roll");
    start(&mut w, gw::ROLL);
    let a = gw::SPECIES.attack(gw::ROLL);
    for _ in 0..(a.startup + a.active + a.recovery) {
        advance(&mut w, braced);
        if !w.players[0].aboard() {
            return matches!(w.players[0].action, Action::HitStun { .. });
        }
    }
    false
}

/// **Be on the spine, braced** (§2): the roll throws riders off the wing
/// roots, braced or not, and nobody braced off the spine.
#[test]
fn the_roll_throws_riders_off_the_wing_roots_and_not_braced_riders_off_the_spine() {
    assert!(
        !rolled_off(gw::BACK),
        "a braced rider was thrown off the spine"
    );
    assert!(
        rolled_off(gw::ROOT_L),
        "a braced rider held on at the left wing root"
    );
    assert!(
        rolled_off(gw::ROOT_R),
        "a braced rider held on at the right wing root"
    );
}

/// **Step off at the swoop** (§5): low over the far side, a rider standing
/// anywhere on its back is under the free height of a fall.
#[test]
fn the_swoop_is_low_enough_to_step_off_for_free() {
    let mut w = hunt();
    let at = on(&w, 0, -10).add(V3::new(Fx::ZERO, Knob::SwoopHeight.fx(), Fx::ZERO));
    fly_at(&mut w, at, Fx::ZERO);
    let m = beast(&w);
    let rig = m.rig();
    let mut top = Fx::ZERO;
    for part in [gw::BACK, gw::BREAST, gw::ROOT_L, gw::ROOT_R] {
        let sh = gw::SPECIES.shape(part);
        for (x, z) in [(sh.min.x, sh.min.z), (sh.max.x, sh.max.z)] {
            top = top.max(rig.part_to_world(part, V3::new(x, sh.max.y, z)).y);
        }
    }
    // A wingbeat's heave on top of that, at its highest.
    let heaved = top.add(flight::heave_top());
    assert!(
        heaved.sub(plateau(&w)).raw() < sim::tuning::fall_free().raw(),
        "its back at the swoop is {:?} over the plateau",
        heaved.sub(plateau(&w))
    );
}

// ---------------------------------------------------------------------------
// The perch
// ---------------------------------------------------------------------------

/// Fly it onto its perch.
fn perch(w: &mut World) {
    let spot = on(w, -20, -20);
    stand(w, spot);
    start(w, gw::PERCH);
    for _ in 0..(gw::SPECIES.attack(gw::PERCH).total() + 4) {
        advance(w, Input::default());
    }
    assert!(fight::perched(&beast(w)), "it did not land on its perch");
}

/// **A hit on the perch takes frames off the rest** (§5), a frame for every
/// `1 / PerchShorten` damage.
#[test]
fn a_hit_on_the_perched_galewing_shortens_its_rest() {
    let mut w = hunt();
    perch(&mut w);
    let before = w.lore.word(fight::word::REST);
    advance(&mut w, Input::default());
    let calm = w.lore.word(fight::word::REST);
    assert_eq!(calm, before - 1, "the rest did not count down");
    let dealt = w.monsters[0].as_mut().unwrap().take_hit(gw::BACK, 400);
    advance(&mut w, Input::default());
    let after = w.lore.word(fight::word::REST);
    let cut = Fx::from_int(dealt).mul(Knob::PerchShorten.fx()).to_int() as u32;
    assert_eq!(
        after,
        calm - 1 - cut,
        "{dealt} damage took {} frames off",
        calm - 1 - after
    );
}

/// **The perch is announced long enough to climb the tower** (§9): the
/// announcement and the rest outlast eight hops up its ledges and a walk
/// round a side of it between each, for the lowest-jumping class.
#[test]
fn the_perch_is_announced_long_enough_to_climb_the_tower() {
    let mut w = World::with_classes([Class::Bulwark; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut hop = 0u32;
    let mut was_up = false;
    for f in 0..240 {
        let input = Input::default().with(Input::SPACE);
        w.advance([input, Input::default()]);
        let up = w.players[0].pos.y.sub(floor).raw() > Fx::ratio(3, 2).raw();
        let _ = f;
        if was_up && !up {
            hop = f;
            break;
        }
        was_up |= up;
    }
    assert!(hop > 0, "a hop never cleared a ledge");
    let walk = Fx::from_int(6)
        .div(sim::tuning::move_speed())
        .mul(Fx::from_int(60))
        .to_int() as u32;
    // Hopping while walking round to the next ledge: each ledge is the
    // longer of the two.
    let climb = 8 * hop.max(walk);
    let a = gw::SPECIES.attack(gw::PERCH);
    let window = a.startup as u32 + Knob::PerchRest.raw() as u32;
    assert!(
        window > climb,
        "{window} frames to climb a {climb}-frame tower"
    );
}

// ---------------------------------------------------------------------------
// Aiming at it
// ---------------------------------------------------------------------------

/// **Skillshots hit it** (§6): aimed at the circling bird from the plateau,
/// the air bolt's path meets it inside the bolt's reach. Bodies are not on
/// the crosshair's ray -- the shot's own path runs into it.
#[test]
fn a_skillshot_aimed_at_the_circling_galewing_hits_it_inside_its_reach() {
    let mut w = hunt_as(Class::Elementalist);
    let me = on(&w, 0, -10);
    stand(&mut w, me);
    let at = on(&w, 11, -10).add(V3::new(Fx::ZERO, Knob::CruiseAlt.fx(), Fx::ZERO));
    fly_at(&mut w, at, sim::math::QUARTER_TURN);
    advance(&mut w, look());
    let m = beast(&w);
    let middle = m.rig().bone[gw::bones::ROOT].at;
    let p = w.players[0];
    let yaw = sim::math::atan2_turns(middle.z.sub(p.pos.z), middle.x.sub(p.pos.x));
    let aim = (yaw.raw() as u32 & 0xFFFF) as u16;
    let pitch = aim::look_onto_closely(p.pos, aim, p.aloft, middle);
    let input = Input::looking_at(0, aim, pitch);
    let field = sim::stones::gather(&w.players);
    let ground = w.terrain();
    let scene = Scene {
        stones: &field,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let reach = Fx::from_int(22);
    let path = aim::skillshot_path(0, input, reach, &scene);
    let hit = aim::first_along(
        path,
        Fx::ratio(1, 4),
        0,
        &scene,
        Targets::none().quarry(true),
    );
    assert!(
        matches!(hit, Some(aim::Contact::Quarry { .. })),
        "a shot at the bird went {path:?} and met {hit:?}"
    );
}

// ---------------------------------------------------------------------------
// Whole hunts
// ---------------------------------------------------------------------------

/// A crude fighter for whole hunts: it walks about the plateau looking at
/// the bird, and swings when it is close.
fn crude(w: &World, f: u32) -> Input {
    let me = w.players[0];
    let m = beast(w);
    let to = m.pos.sub(me.pos);
    let yaw = sim::math::atan2_turns(to.z, to.x);
    let mut bits = 0;
    if (f / 90) % 3 == 0 {
        bits |= Input::A;
    }
    if (f / 200) % 4 == 1 {
        bits |= Input::D | Input::W;
    }
    if to.flat_len().raw() < Fx::from_int(6).raw() && f % 20 == 0 {
        bits |= Input::LEFT;
    }
    Input::aimed(bits, (yaw.raw() as u32 & 0xFFFF) as u16)
}

/// **A rollback flies the same flight**: two worlds fed the same inputs hash
/// the same at every frame.
#[test]
fn a_hunt_against_it_is_deterministic() {
    let mut a = hunt();
    let mut b = hunt();
    for f in 0..4000 {
        let i = crude(&a, f);
        a.advance([i, Input::default()]);
        b.advance([i, Input::default()]);
        assert_eq!(a.checksum(), b.checksum(), "diverged at frame {f}");
    }
}

/// **It throws what it has** against a fighter who walks about the
/// plateau, and perches when its wind runs low.
#[test]
fn it_throws_everything_it_has_from_the_air() {
    let mut seen = [false; gw::MOVE_COUNT];
    for seed in [1u32, 7, 23] {
        let mut w = hunt();
        w.monsters[0].as_mut().unwrap().brain.rng = seed | 1;
        for f in 0..7200 {
            if !matches!(w.phase, sim::state::Phase::Fighting) {
                break;
            }
            let i = crude(&w, f);
            w.advance([i, Input::default()]);
            // Keep the hunter alive: this is about what it throws.
            w.players[0].health = w.players[0].health.max(500);
            if let Some(k) = beast(&w).doing.attacking() {
                seen[k as usize] = true;
            }
        }
    }
    for kind in [
        gw::STOOP,
        gw::TALON,
        gw::PERCH,
        gw::LIFT,
        gw::SCREECH,
        gw::BUFFET,
    ] {
        assert!(
            seen[kind as usize],
            "never {}",
            gw::MOVES[kind as usize].name
        );
    }
}

/// **A swing on a banked back is level with the back** (§6, bestiary A4).
/// `aim::swing_path` measures its dead zone against the up of the surface
/// underfoot: a rider on a back banked forty degrees, looking across it a
/// little below the back's own horizon, swings along the back -- not into
/// the low wing and over the high one, as a world-level swing would. On a
/// level back the up is the floor's exactly, so nothing else changes.
#[test]
fn a_swing_on_a_banked_back_is_level_with_the_back() {
    let mut w = hunt();
    let at = on(&w, 0, -10).add(V3::new(Fx::ZERO, Knob::RideClimb.fx(), Fx::ZERO));
    fly_at(&mut w, at, Fx::ZERO);
    let reach = Fx::from_int(3);
    let swing = |w: &World, facing: V3, pitch: Fx| {
        let field = sim::stones::gather(&w.players);
        let ground = w.terrain();
        let scene = Scene {
            stones: &field,
            players: &w.players,
            effects: &w.effects,
            quarry: &w.monsters,
            critters: &w.critters,
            arena: &ground,
        };
        let up = aim::underfoot_up(0, &scene);
        let look = Input::looking_at(0, 0, pitch.raw() as i16);
        let p = &w.players[0];
        let path = aim::swing_path(
            p.pos,
            facing,
            look,
            true,
            reach,
            aim::Hand::Centre,
            aim::Stand::fighter(),
            up,
        );
        (path.to.sub(path.from), up)
    };

    // On the plateau: the floor's up, exactly, so the swing is what it was.
    let (_, up) = swing(&w, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO), Fx::ZERO);
    assert_eq!(up, V3::Y, "standing on the plateau is not the floor's up");
    board(&mut w, gw::BACK);

    // Banked forty degrees.
    let m = w.monsters[0].as_mut().unwrap();
    fight::set_bank(m, Fx::ratio(40, 360));
    let rig = beast(&w).rig();
    let up = rig.of(gw::BACK).rot.apply(V3::Y);
    assert!(
        up.y.raw() < Fx::ratio(9, 10).raw(),
        "the back did not bank: up {up:?}"
    );
    // Facing across the back, toward its low side.
    let side = V3::new(up.x, Fx::ZERO, up.z);
    let facing = side.scale(Fx::ONE.div(side.flat_len()));
    // Ten degrees under the back's own horizon that way.
    let lean = sim::math::atan2_turns(
        facing.sub(up.scale(facing.dot(up))).y,
        facing.sub(up.scale(facing.dot(up))).flat_len(),
    );
    let pitch = lean.sub(Fx::ratio(10, 360));
    let (dir, got) = swing(&w, facing, pitch);
    assert_eq!(got, up, "the swing was not given the back's up");
    let off = dir.dot(up).abs();
    assert!(
        off.raw() < reach.div(Fx::from_int(20)).raw(),
        "on a back banked forty degrees the swing leaves the back by {off:?} of {reach:?}"
    );
    // The world's level, for comparison: well into the back.
    let world_level = facing.scale(reach);
    assert!(
        world_level.dot(up).abs().raw() > reach.div(Fx::from_int(2)).raw(),
        "a world-level swing would have been level with this back too"
    );
}

/// **A crash ends in the lift, not in another crash**: once it has crashed,
/// wing hits do not fill its poise again until it has been back up to its
/// circle. Without it a rider on the roots, or a hunter at a wing, toppled
/// it again every time it stood, and the fight was over without it ever
/// leaving the floor.
#[test]
fn once_crashed_it_cannot_crash_again_before_it_has_flown() {
    let mut w = hunt();
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Prowl;
    fight::set_flag(m, fight::flag::ALOFT, false);
    fight::set_flag(m, fight::flag::LOW, true);
    m.poise = 0;
    fight::struck(m, gw::ROOT_L, 100);
    assert!(m.poise > 0, "a wing hit on the floor fills no poise at all");
    fight::set_flag(m, fight::flag::SPENT, true);
    let before = m.poise;
    fight::struck(m, gw::ROOT_L, 100);
    assert_eq!(m.poise, before, "a spent bird's poise filled again");
}

/// **A dead Galewing is a won hunt, with its trophy and its temper** (world
/// W1, W2): the trophy is written from `hunt_won`, and a temper is the same
/// bird fought cleverer -- it glances more often -- on the Cliffs.
#[test]
fn a_dead_galewing_is_a_won_hunt_with_its_trophy_and_its_temper() {
    let calm = hunt();
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::GALEWING).tempered(2);
    w.players[1].health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    assert_eq!(w.arena().id, sim::arena::ArenaId::GALEWING);
    assert!(beast(&w).glance_frames() < beast(&calm).glance_frames());
    assert_eq!(w.hunt_won(), None);
    w.monsters[0].as_mut().unwrap().health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    let (beaten, at) = w.hunt_won().expect("a dead bird is a won hunt");
    assert_eq!(beaten[0], Some(SpeciesId::GALEWING));
    assert_eq!(at, 2);
}
