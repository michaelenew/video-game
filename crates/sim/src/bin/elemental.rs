//! The Elementalist's instrument: the v2 kit, measured.
//!
//!     cargo run -p sim --bin elemental
//!     cargo run -p sim --bin elemental -- charge lift ring spray quake break
//!
//! Runs scripted exchanges against the training dummy and prints every number
//! `docs/design/plans/elementalist-v2.md` asks about: what a hold of `Q` is
//! worth and what it costs in ground, how far a hold of `E` runs the crack,
//! how high the Updraft takes each class, what the two rings do at each
//! distance, what fire does to an air shot, where the Quake and the Tremor put
//! their stone, and what a dodge into a stone leaves behind. It is how a change
//! to the class is self-evaluated in seconds, and the first thing a reviewer
//! runs. Every criterion in the plan is a number printed here.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers it.

use sim::class::{Class, Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::moves::elementalist as e;
use sim::state::{Action, SLOT_COMMITTED, SLOT_SPECIAL};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

fn main() {
    let asked: Vec<String> = std::env::args().skip(1).collect();
    let wanted = |name: &str| asked.is_empty() || asked.iter().any(|a| a == name);

    println!("The Elementalist, v2, measured.\n");
    println!(
        "walk {} m/s  |  a hold crawls at {}% of it  |  pillar burns {} over its life  |  \
         stone radius {} m",
        tenths(t::move_speed()),
        sim::moves::get(Class::Elementalist, SLOT_SPECIAL).mobility,
        t::pillar_burn_total(),
        tenths(t::structure_radius()),
    );
    println!();

    if wanted("charge") {
        charge();
    }
    if wanted("lift") {
        lift();
    }
    if wanted("ring") {
        ring();
    }
    if wanted("spray") {
        spray();
    }
    if wanted("quake") {
        quake();
    }
    if wanted("break") {
        break_through();
    }

    println!(
        "A hold is paid for in ground: the safe charge distance is what a walking opponent\n\
         covers while she stands there, and a Strike thrown from closer than that is a\n\
         Strike she is hit out of. Everything else here is the shape of a rule rather\n\
         than a number to like -- the ring reaches, the stone comes up under her, the\n\
         dodge leaves a scar -- and the number is there so a change to it is seen.\n\
         Record what you change in docs/design/feel-log.md."
    );
}

// ---------------------------------------------------------------------------
// The charges
// ---------------------------------------------------------------------------

/// For holds of nothing, a quarter, a half, three quarters and the full cap:
/// what the Strike hits for and what it leaves of the pillar; how far the
/// crack runs; how far she drifts at the crawl; and how much ground an
/// opponent walking at full speed covers while she holds.
fn charge() {
    let pillar = sim::moves::get(Class::Elementalist, SLOT_SPECIAL);
    let crack = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    println!("charge -- Q held is the Strike, E held is the crack");
    println!(
        "  {:>6}{:>8}{:>8}{:>12}{:>10}{:>10}{:>12}",
        "hold", "frames", "strike", "pillar left", "crack m", "drift m", "safe gap m"
    );
    for quarter in 0..=4u16 {
        let hold = pillar.channel * quarter / 4;
        // The Strike, on a dummy standing where the pillar lands.
        let mut w = elementalist();
        let spot = metres(-2000, 0, 8000);
        w.players[1].pos = spot;
        let full = w.players[1].health;
        let pitch = crosshair_onto_the_floor_at(&w, spot, pillar.reach);
        let stood = w.players[0].pos;
        strike(&mut w, pitch, hold);
        let hit = full - w.players[1].health;
        let left = effects_of(&w, EffectKind::FirePillar)
            .first()
            .map_or(0, |p| p.life);
        let drift = flat_between(stood, w.players[0].pos);
        // The crack, held the same number of frames past the churn.
        let crawl_hold = crack.channel * quarter / 4;
        let ran = crack_after(crawl_hold);
        // A walking opponent covers this much while the pillar's startup and
        // the hold go by. A Strike thrown from closer is a Strike she is hit
        // out of.
        let frames = pillar.startup + hold;
        let gap = t::move_speed()
            .mul(sim::DT)
            .mul(Fx::from_int(frames as i32));
        println!(
            "  {:>5}%{:>8}{:>8}{:>12}{:>10}{:>10}{:>12}",
            quarter * 25,
            frames,
            hit,
            left,
            tenths(ran),
            hundredths(drift),
            tenths(gap)
        );
    }
    println!(
        "  a tap is the pillar as built ({} burn over {}f); a full hold is worth x{} of the \
         burn and leaves nothing standing",
        t::pillar_burn_total(),
        t::pillar_life(),
        tenths(t::strike_worth())
    );
    println!();
}

/// How far the crack runs from a stone held `hold` frames past its churn.
fn crack_after(hold: u16) -> Fx {
    let mut w = elementalist();
    run(&mut w, 1, e::keys::EARTH, 0);
    let raised = stones_of(&w)[0].at;
    for _ in 0..200 {
        if matches!(w.players[0].action, Action::Channel { .. }) {
            break;
        }
        run(&mut w, 1, e::keys::EARTH, 0);
    }
    for _ in 0..hold {
        run(&mut w, 1, e::keys::EARTH, 0);
    }
    run(&mut w, 1, 0, 0);
    let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    run(&mut w, (m.startup + 1) as u32, 0, 0);
    stones_of(&w)
        .first()
        .map_or(Fx::ZERO, |s| s.at.x.sub(raised.x))
}

// ---------------------------------------------------------------------------
// Updraft and Downdraft
// ---------------------------------------------------------------------------

/// Updraft's apex for every class standing in the column, and for a stone;
/// Downdraft's descent from a full hop against a plain fall from the same
/// height.
fn lift() {
    println!("lift -- space and right click is the Updraft, F in the air the Downdraft");
    println!("  {:<14}{:>10}", "in the column", "apex m");
    for class in sim::class::ALL_CLASSES {
        let mut w = World::with_classes([Class::Elementalist, class]);
        w.players[0].pos = metres(-6000, 0, 8000);
        w.players[1].pos = metres(-5000, 0, 8000);
        throw(&mut w, Input::SPACE | e::keys::WIND);
        let mut apex = Fx::ZERO;
        let mut hers = Fx::ZERO;
        for _ in 0..90 {
            run(&mut w, 1, 0, 0);
            apex = apex.max(w.players[1].pos.y);
            hers = hers.max(w.players[0].pos.y);
        }
        println!("  {:<14}{:>10}", class.name(), hundredths(apex));
        if class == Class::Elementalist {
            println!(
                "  {:<14}{:>10}   (herself, in her own column)",
                "",
                hundredths(hers)
            );
        }
    }
    // A stone standing in the column, two metres out: inside it, and clear of
    // her own feet.
    let mut w = elementalist();
    sim::stones::raise(&mut w.players[0], Structure::raised(metres(-4000, 0, 8000)));
    run(&mut w, 30, 0, 0);
    let stood = stones_of(&w)[0].at.y;
    throw(&mut w, Input::SPACE | e::keys::WIND);
    let mut apex = stood;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        if let Some(s) = stones_of(&w).first() {
            apex = apex.max(s.at.y);
        }
    }
    println!(
        "  {:<14}{:>10}   (a stone, lofted from {})",
        "stone",
        hundredths(apex),
        hundredths(stood)
    );
    // Downdraft from a full hop, against the fall she would have had.
    let hop = t::jump_speed();
    let mut plain = elementalist();
    plain.players[0].vel.y = hop;
    plain.players[0].grounded = false;
    let mut fell = 0;
    for _ in 0..240 {
        run(&mut plain, 1, 0, 0);
        fell += 1;
        if plain.players[0].grounded {
            break;
        }
    }
    let mut w = elementalist();
    w.players[0].vel.y = hop;
    w.players[0].grounded = false;
    // Press F at the top of the hop.
    let mut driven = 0;
    let mut top = Fx::ZERO;
    for _ in 0..240 {
        driven += 1;
        let rising = w.players[0].vel.y.raw() > 0;
        run(&mut w, 1, if rising { 0 } else { Input::KEY_F }, 0);
        top = top.max(w.players[0].pos.y);
        if w.players[0].grounded {
            break;
        }
    }
    println!(
        "  a full hop of {} m lands in {}f on its own and in {}f with the Downdraft pressed \
         at the top",
        hundredths(top),
        fell,
        driven
    );
    println!();
}

// ---------------------------------------------------------------------------
// The two rings
// ---------------------------------------------------------------------------

/// The air ring's push on a dummy at three distances; the fire ring's radius
/// over time and its damage at the same three distances; and that the pillar
/// it put out is gone.
fn ring() {
    println!(
        "ring -- landing under the Downdraft breaks the air outward; into fire, a ring of fire"
    );
    println!(
        "  {:<8}{:>14}{:>14}{:>14}",
        "at m", "air push m", "fire dmg", "fire push m"
    );
    for far in [1500, 3000, 5000] {
        let stand = metres(-6000 + far, 0, 8000);
        // The air ring.
        let mut w = elementalist();
        aloft(&mut w, 1500);
        w.players[1].pos = stand;
        throw(&mut w, Input::KEY_F);
        land(&mut w);
        run(&mut w, 30, 0, 0);
        let pushed = w.players[1].pos.x.sub(stand.x);
        // The fire ring, with a grown pillar under where she lands.
        let mut w = elementalist();
        let mut pillar = Effect::cast(
            EffectKind::FirePillar,
            1,
            Class::Elementalist,
            SLOT_SPECIAL,
            metres(-6000, 0, 8000),
            V3::ZERO,
            Fx::ZERO,
        );
        pillar.age = pillar.life / 2;
        w.effects[0] = Some(pillar);
        w.players[1].pos = stand;
        let full = w.players[1].health;
        aloft(&mut w, 2000);
        throw(&mut w, Input::KEY_F);
        land(&mut w);
        run(&mut w, 40, 0, 0);
        println!(
            "  {:<8}{:>14}{:>14}{:>14}",
            tenths(Fx::ratio(far, 1000)),
            hundredths(pushed),
            full - w.players[1].health,
            hundredths(w.players[1].pos.x.sub(stand.x))
        );
    }
    // The fire ring's growth, and the pillar's fate.
    let mut w = elementalist();
    let mut pillar = Effect::cast(
        EffectKind::FirePillar,
        1,
        Class::Elementalist,
        SLOT_SPECIAL,
        metres(-6000, 0, 8000),
        V3::ZERO,
        Fx::ZERO,
    );
    pillar.age = pillar.life / 2;
    w.effects[0] = Some(pillar);
    aloft(&mut w, 2000);
    throw(&mut w, Input::KEY_F);
    land(&mut w);
    let mut radii = Vec::new();
    for _ in 0..30 {
        run(&mut w, 1, 0, 0);
        match effects_of(&w, EffectKind::FireRing).first() {
            Some(r) => radii.push(tenths(r.ring_radius())),
            None => break,
        }
    }
    println!(
        "  the fire ring's radius by frame: {} (reach {} m at {} m/s, {} wide)",
        radii.join(" "),
        tenths(t::fire_ring_reach()),
        tenths(t::fire_ring_speed()),
        tenths(t::fire_ring_width())
    );
    println!(
        "  the pillar she landed in: {}",
        if effects_of(&w, EffectKind::FirePillar).is_empty() {
            "gone"
        } else {
            "STILL BURNING"
        }
    );
    println!();
}

// ---------------------------------------------------------------------------
// Cinder spray and fire in the air
// ---------------------------------------------------------------------------

/// Where the ember bursts with nothing in the way and against a stone; an Air
/// bolt's and a Gale's damage plain and lit; and a stone lit by the cloud.
fn spray() {
    println!("spray -- F standing; air shots through fire come out lit");
    let m = sim::moves::get(Class::Elementalist, e::CINDER);
    // Nothing in the way.
    let mut w = elementalist();
    let from = w.players[0].pos;
    throw(&mut w, Input::KEY_F);
    fly_out(&mut w, 0, 120);
    let open = clouds(&w)
        .first()
        .map_or(Fx::ZERO, |c| flat_between(from, c.pos));
    // Against a stone.
    let mut w = elementalist();
    let stone_at = metres(-1000, 0, 8000);
    sim::stones::raise(&mut w.players[0], Structure::raised(stone_at));
    run(&mut w, 30, 0, 0);
    throw(&mut w, Input::KEY_F);
    fly_out(&mut w, 0, 120);
    let blocked = clouds(&w)
        .first()
        .map_or(Fx::ZERO, |c| flat_between(from, c.pos));
    let lit = stones_of(&w).first().is_some_and(|s| s.lit > 0);
    println!(
        "  the ember bursts {} m out with nothing in the way (range {} m), and {} m out \
         against a stone standing {} m away -- which is {}",
        tenths(open),
        tenths(m.reach),
        tenths(blocked),
        tenths(flat_between(from, stone_at)),
        if lit { "lit by it" } else { "NOT LIT" }
    );
    println!(
        "  the cloud: radius {} m, {}f, {} a tick",
        tenths(t::embers_radius()),
        t::embers_life(),
        t::embers_damage()
    );
    println!("  {:<10}{:>8}{:>8}", "shot", "plain", "lit");
    for (name, button) in [("Air bolt", e::keys::WEAK_PUSH), ("Gale", Input::RIGHT)] {
        let plain = air_shot_through(button, false);
        let lit = air_shot_through(button, true);
        println!("  {:<10}{:>8}{:>8}", name, plain, lit);
    }
    println!(
        "  lit is worth +{}% and bursts into a cloud of radius {} m where it lands",
        percent(t::lit_bonus()),
        tenths(t::lit_burst_radius())
    );
    // The Gale and a stone.
    let mut w = elementalist();
    sim::stones::raise(&mut w.players[0], Structure::raised(stone_at));
    run(&mut w, 30, 0, 0);
    let before = stones_of(&w)[0].at.x;
    aloft(&mut w, 1000);
    let reach = sim::moves::get(Class::Elementalist, e::GALE).reach;
    let pitch = crosshair_onto_the_floor_at(&w, metres(3000, 0, 8000), reach);
    throw_looking(&mut w, Input::RIGHT, pitch);
    fly_out(&mut w, pitch, 60);
    println!(
        "  a Gale past a stone shoves it {} m along its travel (x{} of the bolt's kick)",
        hundredths(stones_of(&w)[0].at.x.sub(before)),
        tenths(t::gale_stone_push())
    );
    println!();
}

/// An airborne shot flown through a cloud, or not, at a dummy two metres past
/// it: what it hits for.
fn air_shot_through(button: u16, fire: bool) -> i32 {
    let mut w = elementalist();
    w.players[1].pos = metres(2000, 0, 8000);
    if fire {
        w.effects[0] = Some(Effect::cast(
            EffectKind::Embers,
            0,
            Class::Elementalist,
            e::CINDER,
            metres(-2000, 1250, 8000),
            V3::ZERO,
            t::embers_radius(),
        ));
    }
    aloft(&mut w, 1000);
    let slot = if button == e::keys::WEAK_PUSH {
        e::AIR_BOLT
    } else {
        e::GALE
    };
    let reach = sim::moves::get(Class::Elementalist, slot).reach;
    let pitch = crosshair_onto_the_floor_at(&w, w.players[1].pos, reach);
    let full = w.players[1].health;
    throw_looking(&mut w, button, pitch);
    fly_out(&mut w, pitch, 120);
    run(&mut w, 2, 0, 0);
    full - w.players[1].health
}

// ---------------------------------------------------------------------------
// Quake and Tremor
// ---------------------------------------------------------------------------

/// Stagger on a walking dummy and none on a standing one; the eruption's
/// damage; and where the stone comes up -- at the crosshair for Quake, under
/// her for Tremor -- with her height after the Tremor.
fn quake() {
    println!("quake -- the second side button is the Quake, R the Tremor");
    let m = sim::moves::get(Class::Elementalist, e::QUAKE);
    let spot = metres(-1000, 0, 8000);
    let mut w = elementalist();
    let pitch = crosshair_onto_the_floor_at(&w, spot, m.reach);
    let look = Input::looking_at(0, look_right(), pitch);
    let seen = with_scene(&w, |scene| sim::aim::sight(0, look, m.reach, scene)).at;
    throw_looking(&mut w, Input::SIDE_B, pitch);
    let patch = effects_of(&w, EffectKind::Quake)
        .first()
        .map_or(V3::ZERO, |q| q.pos);
    // Standing still in it.
    w.players[1].pos = spot;
    run(&mut w, 6, 0, 0);
    let still = w.players[1].action.stunned();
    // Walking through it.
    run(&mut w, 6, 0, Input::W);
    let walking = w.players[1].action.stunned();
    // The eruption, on somebody standing in it.
    let mut w = elementalist();
    throw_looking(&mut w, Input::SIDE_B, pitch);
    w.players[1].pos = spot;
    let full = w.players[1].health;
    run(&mut w, t::quake_shake() as u32 + 2, 0, 0);
    let erupted = full - w.players[1].health;
    let stone = stones_of(&w).first().map_or(V3::ZERO, |s| s.at);
    println!(
        "  the patch has radius {} m, shakes for {}f, staggers anything moving over {} m/s \
         for {}f",
        tenths(t::quake_radius()),
        t::quake_shake(),
        tenths(t::quake_still_speed()),
        t::quake_stagger()
    );
    println!(
        "  crosshair on the floor at x={}: patch at x={}; standing still in it: {}; walking \
         through it: {}",
        tenths(seen.x),
        tenths(patch.x),
        if still { "STAGGERED" } else { "fine" },
        if walking {
            "staggered"
        } else {
            "NOT STAGGERED"
        }
    );
    println!(
        "  the eruption hits for {} and leaves a stone at x={} (push {} m/s)",
        erupted,
        tenths(stone.x),
        tenths(t::quake_push())
    );
    // Tremor.
    let mut w = elementalist();
    let feet = w.players[0].pos;
    throw(&mut w, Input::KEY_R);
    let patch = effects_of(&w, EffectKind::Quake)
        .first()
        .map_or(V3::ZERO, |q| q.pos);
    let mut apex = Fx::ZERO;
    for _ in 0..(t::quake_shake() as u32 + 60) {
        run(&mut w, 1, 0, 0);
        apex = apex.max(w.players[0].pos.y);
    }
    let stone = stones_of(&w).first().map_or(V3::ZERO, |s| s.at);
    println!(
        "  Tremor: her feet at x={}, patch at x={}, stone at x={}, and it carried her to {} m",
        tenths(feet.x),
        tenths(patch.x),
        tenths(stone.x),
        hundredths(apex)
    );
    println!();
}

// ---------------------------------------------------------------------------
// The dodge into a stone
// ---------------------------------------------------------------------------

/// Structure count before and after a dodge into a stone; the slow on a dummy
/// crossing where it stood; and burning ground when the stone was lit.
fn break_through() {
    println!("break -- a dodge toward a stone, crosshair on it, breaks through");
    for lit in [false, true] {
        let mut w = elementalist();
        let spot = metres(-4000, 0, 8000);
        sim::stones::raise(&mut w.players[0], Structure::raised(spot));
        run(&mut w, 30, 0, 0);
        if lit {
            sim::stones::light(&mut w.players, 0);
        }
        let before = stones_of(&w).len();
        let pitch = crosshair_onto_the_stone_at(&w, spot);
        w.advance([
            Input::looking_at(Input::SHIFT | Input::W, look_right(), pitch),
            Input::looking_at(0, look_left(), 0),
        ]);
        let after = stones_of(&w).len();
        let dodged = matches!(w.players[0].action, Action::Dodge { .. });
        // A dummy crossing where it stood.
        w.players[1].pos = spot;
        run(&mut w, 2, 0, 0);
        let slowed = w.players[1].slowed > 0;
        let scar = effects_of(&w, EffectKind::Rough);
        let embers = effects_of(&w, EffectKind::Embers).len();
        println!(
            "  {} stone: {} before, {} after ({}); the scar runs {} m; crossing it: {}; \
             embers left: {}",
            if lit { "a lit" } else { "an unlit" },
            before,
            after,
            if dodged { "she dodged" } else { "NO DODGE" },
            scar.first().map_or("none".to_string(), |s| tenths(s.reach)),
            if slowed {
                format!("slowed to x{}", hundredths(w.players[1].slow_mul))
            } else {
                "NOT SLOWED".to_string()
            },
            embers
        );
    }
    println!(
        "  reach {} m past the stone's edge, crosshair slack {} m",
        tenths(t::break_reach()),
        tenths(t::break_lock())
    );
    println!();
}

// ---------------------------------------------------------------------------
// Fixtures -- the same ones `tests/elementalist_v2.rs` uses
// ---------------------------------------------------------------------------

/// She looks along +X; the dummy faces her from the right, half a turn round.
fn look_right() -> u16 {
    0
}

fn look_left() -> u16 {
    Input::QUARTER_TURN << 1
}

/// A point given in millimetres, so this file can stay float-free.
fn metres(x: i32, y: i32, z: i32) -> V3 {
    V3::new(Fx::ratio(x, 1000), Fx::ratio(y, 1000), Fx::ratio(z, 1000))
}

/// An Elementalist facing +X with clear floor in front of her, and a Bulwark
/// far enough away to be out of everything unless a script moves them.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = metres(-6000, 0, 8000);
    w.players[1].pos = metres(11000, 0, 8000);
    w
}

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(a, look_right(), 0),
            Input::looking_at(b, look_left(), 0),
        ]);
    }
}

/// Take her feet off the floor without moving her, `mm` millimetres up.
fn aloft(w: &mut World, mm: i32) {
    w.players[0].pos.y = Fx::ratio(mm, 1000);
    w.players[0].grounded = false;
}

/// Run until she is back on the floor, and one frame more.
fn land(w: &mut World) {
    for _ in 0..120 {
        run(w, 1, 0, 0);
        if w.players[0].grounded {
            break;
        }
    }
    run(w, 1, 0, 0);
}

fn doing(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. }
        | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    }
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        return Vec::new();
    };
    slots.iter().flatten().copied().collect()
}

fn shots(w: &World) -> Vec<sim::gust::Gust> {
    w.gusts.iter().flatten().copied().collect()
}

fn effects_of(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

fn clouds(w: &World) -> Vec<Effect> {
    effects_of(w, EffectKind::Embers)
}

fn throw(w: &mut World, button: u16) {
    throw_looking(w, button, 0);
}

/// Press a button until the move it asks for is out of her hands, then let go.
fn throw_looking(w: &mut World, button: u16, pitch: i16) {
    let step = |w: &mut World, bits: u16| {
        w.advance([
            Input::looking_at(bits, look_right(), pitch),
            Input::looking_at(0, look_left(), 0),
        ]);
    };
    step(w, button);
    let Some(kind) = doing(w) else { return };
    let (startup, active, _) = sim::moves::frames(Class::Elementalist, kind);
    for _ in 0..(startup + active) {
        step(w, button);
    }
    step(w, 0);
}

/// Hold `Q` for `hold` frames past the pillar's startup, at a pitch, then let
/// go and run the active frames. A hold of zero is a tap.
fn strike(w: &mut World, pitch: i16, hold: u16) {
    let (startup, active, _) = sim::moves::frames(Class::Elementalist, SLOT_SPECIAL);
    let step = |w: &mut World, bits: u16| {
        w.advance([
            Input::looking_at(bits, look_right(), pitch),
            Input::looking_at(0, look_left(), 0),
        ]);
    };
    if hold == 0 {
        step(w, e::keys::FIRE);
        for _ in 0..(startup + active) {
            step(w, 0);
        }
        return;
    }
    // Walking forward through the hold, so the crawl is measured.
    for _ in 0..(startup + hold + 4) {
        if matches!(w.players[0].action, Action::Channel { held, .. } if held >= hold) {
            break;
        }
        step(w, e::keys::FIRE | Input::W);
    }
    for _ in 0..(active + 1) {
        step(w, 0);
    }
}

fn with_scene<T>(w: &World, ask: impl FnOnce(&sim::aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
    })
}

/// The pitch that puts the crosshair on the floor at a point, aimed the way a
/// player does. Looking at the floor is what makes a skillshot travel level.
fn crosshair_onto_the_floor_at(w: &World, target: V3, reach: Fx) -> i16 {
    (-800..0)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, look_right(), pitch);
            let seen = with_scene(w, |scene| sim::aim::sight(0, look, reach, scene));
            let gap = flat_between(seen.at, target);
            (gap.raw() < Fx::ratio(1, 2).raw()).then_some(pitch)
        })
        .expect("no pitch puts the crosshair anywhere near that point")
}

/// The pitch that puts the crosshair on a stone standing at `spot`.
fn crosshair_onto_the_stone_at(w: &World, spot: V3) -> i16 {
    // The same column the dodge itself asks about: the stone's base, with
    // the break-through's slack around it.
    let stone = stones_of(w)
        .into_iter()
        .find(|s| s.at.x == spot.x)
        .expect("a stone there");
    let middle = stone.at;
    (-800..400)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, look_right(), pitch);
            with_scene(w, |scene| {
                sim::aim::pointing_at(0, look, middle, t::break_lock(), scene).then_some(pitch)
            })
        })
        .expect("no pitch puts the crosshair on the stone")
}

/// Fly every shot out, at a held pitch, until none is left or `most` frames
/// have passed.
fn fly_out(w: &mut World, pitch: i16, most: u32) {
    for _ in 0..most {
        if shots(w).is_empty() {
            break;
        }
        w.advance([
            Input::looking_at(0, look_right(), pitch),
            Input::looking_at(0, look_left(), 0),
        ]);
    }
}

fn flat_between(a: V3, b: V3) -> Fx {
    V3::new(a.x.sub(b.x), Fx::ZERO, a.z.sub(b.z)).flat_len()
}

/// One decimal place, without touching floating point. The sign is printed
/// separately so a value between minus one and zero keeps it.
fn tenths(v: Fx) -> String {
    let t = ((v.raw() as i64).abs() * 10 + (1 << 15)) >> 16;
    format!("{}{}.{}", sign(v), t / 10, t % 10)
}

/// Two decimal places, the same way.
fn hundredths(v: Fx) -> String {
    let h = ((v.raw() as i64).abs() * 100 + (1 << 15)) >> 16;
    format!("{}{}.{:02}", sign(v), h / 100, h % 100)
}

fn sign(v: Fx) -> &'static str {
    if v.raw() < 0 { "-" } else { "" }
}

fn percent(v: Fx) -> i64 {
    (v.raw() as i64 * 100 + (1 << 15)) >> 16
}
