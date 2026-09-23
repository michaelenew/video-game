//! The tally: the Shadow Reaver's v2, driven by named scripts.
//!
//!     cargo run -p sim --bin tally                every script, summarised
//!     cargo run -p sim --bin tally pattern        one script, frame by frame
//!
//! Nobody can feel a number, but the relationships the v2 design asks for are
//! all things this prints in a second: that the shadow's copies land at range
//! once it aims itself, that marks climb to the cap and fade away when left,
//! that crossing to the shadow and swinging cashes them for a burst, and that
//! standing in melee with the shadow at her heel marks nothing. It is how the
//! class is self-evaluated between the human checkpoints in
//! `docs/design/plans/shadow-reaver-v2.md`, and the first thing a reviewer runs.
//!
//! The scripts place bodies and the shadow directly rather than playing a whole
//! match to get them there -- the questions are about the tally, not about
//! walking. Where a script *does* play the real thing (the send, the dash) it
//! says so, because those are the parts whose numbers a player will feel.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers all of it.

use sim::class::{Class, Ghost, Mechanic, Shadow};
use sim::oven::{self, Scalar};
use sim::state::{Action, SLOT_COMMITTED, SLOT_POKE};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

/// Frames between swings in the scripts that swing on a rhythm: the repeat
/// lockout plus a breath, which is about as fast as a person throws the same
/// button when they are not mashing.
const RHYTHM: u32 = 36;
/// How long `range` runs: enough swings for a rate to mean something.
const RANGE_SWINGS: u32 = 24;
/// The dummy's lap round the shadow in `range`, in frames.
const LAP: u32 = 150;

/// A named script, and whether to print it frame by frame.
type Script = (&'static str, fn(bool));

fn main() {
    let only = std::env::args().nth(1);
    let loud = only.is_some();
    let scripts: [Script; 5] = [
        ("range", range),
        ("stall", stall),
        ("pattern", |loud| pattern(loud, SLOT_POKE)),
        ("greedy", |loud| pattern(loud, SLOT_COMMITTED)),
        ("stick", stick),
    ];
    println!(
        "Shadow Reaver v2  --  cap {}, one fades every {}f, any hit of hers cashes at x{} a mark, full tally staggers {}f",
        t::mark_cap(),
        t::mark_fade(),
        hundredths(t::mark_worth()),
        t::cash_stagger(),
    );
    println!(
        "health {} of {}  |  copy {}% out there, {}% at her heel\n",
        t::health_of(Class::ShadowReaver),
        t::max_health(),
        percent(t::shadow_echo()),
        percent(t::shadow_echo_attending()),
    );
    for (name, script) in scripts {
        if only.as_deref().is_some_and(|o| o != name) {
            continue;
        }
        println!("{name}");
        script(loud);
        println!();
    }
    if only.is_none() {
        println!("cash-in by mark count");
        cash_table();
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A placement in the arena, in tenths of a metre. Script geometry, not
/// tuning: where a dummy is put for a measurement.
fn metres(tenths: i32) -> Fx {
    Fx::ratio(tenths, 10)
}

/// A fraction of a reach, for the same kind of placement.
fn share(num: i32, den: i32) -> Fx {
    Fx::ratio(num, den)
}

fn shadow(w: &World) -> Shadow {
    match w.players[0].mechanic {
        Mechanic::Shadow(s) => s,
        _ => panic!("player one is not the Reaver"),
    }
}

fn put_the_shadow_at(w: &mut World, at: V3) {
    let mut s = shadow(w);
    s.pos = at;
    s.doing = Ghost::Waiting;
    w.players[0].mechanic = Mechanic::Shadow(s);
}

/// A Reaver on open floor facing +X, and a dummy -- a Champion, the roster's
/// benchmark body -- standing well out of the way.
fn open_floor() -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Champion]);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, metres(80));
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(metres(-120), Fx::ZERO, metres(-120));
    step(&mut w, 20, 0, 0, 0);
    w
}

fn step(w: &mut World, frames: u32, bits: u16, yaw: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, yaw, pitch), Input::default()]);
    }
}

/// The pitch, looking along `yaw`, that puts her crosshair on the shadow.
/// Searched through `aim::pointing_at` rather than solved, so the script asks
/// the game the question the dash asks.
fn crosshair_on_the_shadow(w: &World, yaw: u16) -> Option<i16> {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    };
    let at = shadow(w).pos;
    (-89..=89).map(degrees).find(|&pitch| {
        sim::aim::pointing_at(
            0,
            Input::looking_at(0, yaw, pitch),
            at,
            t::shadow_lock_cone(),
            &scene,
        )
    })
}

fn degrees(d: i32) -> i16 {
    (d * 65536 / 360) as i16
}

/// The yaw, as the wire carries it, that faces from `from` to `to`.
fn yaw_towards(from: V3, to: V3) -> u16 {
    let d = to.sub(from);
    // A turn is 65536 on the wire and in 16.16 alike, so the bits carry over.
    sim::math::atan2_turns(d.z, d.x).raw() as u16
}

// ---------------------------------------------------------------------------
// range -- does the copy land at range, with and without the self-aim?
// ---------------------------------------------------------------------------

/// The shadow six metres ahead of her, a dummy walking a circle round it inside
/// the copy's reach, and the Reaver swinging at the air on a rhythm. Run twice:
/// with the self-aim, and with the shadow back on her yaw.
fn range(loud: bool) {
    let before = t::shadow_aims();
    for aims in [false, true] {
        oven::set_scalar(Scalar::ShadowAims, aims as i32);
        let (landed, thrown) = range_once(loud);
        println!(
            "  self-aim {:<3}  copies landed {landed:>2} of {thrown}  ({}%)",
            if aims { "on" } else { "off" },
            landed * 100 / thrown.max(1),
        );
    }
    oven::set_scalar(Scalar::ShadowAims, before as i32);
}

fn range_once(loud: bool) -> (u32, u32) {
    let mut w = open_floor();
    let out = w.players[0]
        .pos
        .add(V3::new(metres(60), Fx::ZERO, Fx::ZERO));
    put_the_shadow_at(&mut w, out);
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    // Inside reach from the shadow by a margin, so a copy that is pointed at
    // him lands and one that is not does not.
    let radius = poke.reach.mul(share(3, 4)).add(t::body_radius());
    let (mut landed, mut thrown) = (0, 0);
    let mut was_used = false;
    for frame in 0..(RANGE_SWINGS * RHYTHM) {
        let angle = Fx::ratio((frame % LAP) as i32, LAP as i32);
        let at = out.add(V3::new(
            sim::fixed::cos_turns(angle).mul(radius),
            Fx::ZERO,
            sim::fixed::sin_turns(angle).mul(radius),
        ));
        w.players[1].pos = at;
        w.players[1].vel = V3::ZERO;
        w.players[1].health = w.players[1].full_health();
        let bits = if frame % RHYTHM < 2 { Input::LEFT } else { 0 };
        if frame % RHYTHM == 0 {
            thrown += 1;
        }
        step(&mut w, 1, bits, 0, 0);
        let used = shadow(&w).echo_used;
        if used && !was_used {
            landed += 1;
            if loud {
                println!(
                    "    f{frame:<4} copy landed, him at {} of the lap",
                    hundredths(angle)
                );
            }
        }
        was_used = used;
    }
    (landed, thrown)
}

// ---------------------------------------------------------------------------
// stall -- the fade
// ---------------------------------------------------------------------------

fn stall(loud: bool) {
    let mut w = open_floor();
    for _ in 0..t::mark_cap() {
        sim::shadow::mark(&mut w.players[1]);
    }
    let mut frame = 0u32;
    let mut line = format!("  marks {}", w.players[1].marks);
    while w.players[1].marks > 0 && frame < 60 * 60 {
        let was = w.players[1].marks;
        step(&mut w, 1, 0, 0, 0);
        frame += 1;
        if w.players[1].marks != was {
            line.push_str(&format!(" -> {} at f{frame}", w.players[1].marks));
            if loud {
                println!("    f{frame:<4} a mark faded, {} left", w.players[1].marks);
            }
        }
    }
    println!("{line}");
    println!(
        "  a full tally left alone is gone in {frame}f ({}.{}s)",
        frame / 60,
        frame % 60 * 10 / 60
    );
}

// ---------------------------------------------------------------------------
// pattern / greedy -- send, mark, cross, cash
// ---------------------------------------------------------------------------

/// The fastest scripted run of the whole pattern, played through the real
/// inputs: right click sends the shadow, swings on a rhythm mark him from the
/// field, `shift` + forward with the crosshair on the shadow dashes to it, and
/// the first swing out of the dodge -- `slot` -- cashes.
///
/// The dummy stands beside where the shadow lands, still, the way the game's
/// own training dummy stands. What happens on arrival is reported rather than
/// fixed up: the dash's slide carries her past the shadow, so whether he is
/// still in reach when she is free to swing is one of the numbers.
fn pattern(loud: bool, slot: u8) {
    let mut w = open_floor();
    // The send, for real: right click, looking down enough that the crosshair
    // lands on the floor a few metres out.
    let send = sim::moves::get(Class::ShadowReaver, sim::state::SLOT_MECHANIC);
    step(&mut w, 2, Input::RIGHT, 0, degrees(-25));
    step(
        &mut w,
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
        0,
        0,
        degrees(-25),
    );
    let sent_at = w.frame;
    let out = shadow(&w).pos;
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    // Beside the shadow, on the far side of it from her so the dash carries
    // her towards him rather than away.
    w.players[1].pos = out.add(V3::new(
        poke.reach.mul(share(1, 2)),
        Fx::ZERO,
        poke.reach.mul(share(1, 2)),
    ));
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    let him = w.players[1].pos;
    if loud {
        println!(
            "    shadow out {}m ahead; him {}m from it",
            tenths(out.sub(w.players[0].pos).flat_len()),
            tenths(him.sub(out).flat_len())
        );
    }

    // Mark: swing at the air on a rhythm until the tally is full.
    let mut swings = 0;
    while w.players[1].marks < t::mark_cap() && swings < 20 {
        step(&mut w, 2, Input::LEFT, 0, 0);
        step(&mut w, RHYTHM - 2, 0, 0, 0);
        swings += 1;
        // He is the training dummy: he stays put and does not die.
        w.players[1].pos = him;
        w.players[1].vel = V3::ZERO;
        w.players[1].health = w.players[1].full_health();
        if loud {
            println!("    swing {swings}: marks {}", w.players[1].marks);
        }
    }
    let marked_at = w.frame;
    let marks = w.players[1].marks;

    // Cross: shift + forward, the crosshair on the shadow.
    let yaw = yaw_towards(w.players[0].pos, shadow(&w).pos);
    let Some(look) = crosshair_on_the_shadow(&w, yaw) else {
        println!("  no pitch puts the crosshair on the shadow");
        return;
    };
    step(&mut w, 1, Input::SHIFT | Input::W, yaw, look);
    // Until she arrives: the frame the shadow is collected.
    let mut dodge = 0;
    while shadow(&w).is_out() && dodge < 60 {
        step(&mut w, 1, 0, yaw, 0);
        dodge += 1;
    }
    let crossed = !shadow(&w).is_out();
    let gap = w.players[1]
        .pos
        .sub(w.players[0].pos)
        .flat_len()
        .sub(t::body_radius());
    let free_at = w.frame;
    if loud {
        println!(
            "    crossed: {crossed}; arrived {}f after the press, {}m from him",
            dodge + 1,
            tenths(gap)
        );
    }

    // Cash: turn to him and throw `slot` on the frame after she arrives, out
    // of the carry -- see `shadow::swing_out_of_the_carry`.
    let face = yaw_towards(w.players[0].pos, w.players[1].pos);
    let before = w.players[1].health;
    let bits = if slot == SLOT_COMMITTED {
        Input::MECHANIC
    } else {
        Input::LEFT
    };
    step(&mut w, 2, bits, face, 0);
    let mut dealt = 0;
    for _ in 0..40 {
        if loud {
            println!(
                "    f{} {:?} {}m from him, vel {}",
                w.frame,
                w.players[0].action,
                tenths(w.players[1].pos.sub(w.players[0].pos).flat_len()),
                tenths(w.players[0].vel.flat_len())
            );
        }
        if w.players[1].health < before {
            dealt = before - w.players[1].health;
            break;
        }
        step(&mut w, 1, 0, face, 0);
    }
    let staggered = matches!(w.players[1].action, Action::Stagger { .. });
    let m = sim::moves::get(Class::ShadowReaver, slot);
    println!(
        "  {} swings to {marks} marks  |  crossed {crossed}, arrived {}f after the dash, {}m from him",
        swings,
        dodge + 1,
        tenths(gap),
    );
    if dealt > 0 {
        println!(
            "  {} dealt {dealt} against a plain {} (x{}){}  |  marks left {}  |  send to cash {}f ({} marking, {} crossing)",
            m.name,
            m.damage,
            hundredths(Fx::ratio(dealt, m.damage.max(1))),
            if staggered { ", staggered" } else { "" },
            w.players[1].marks,
            w.frame - sent_at,
            marked_at - sent_at,
            free_at - marked_at,
        );
    } else {
        println!(
            "  {} missed: she was {}m past reach  |  marks left {}",
            m.name,
            tenths(gap.sub(m.reach)),
            w.players[1].marks
        );
    }
}

// ---------------------------------------------------------------------------
// stick -- the shadow at her heel, in melee
// ---------------------------------------------------------------------------

/// The same length of fight as `pattern`, standing in melee with the shadow
/// attending. What v2 asks is that this marks nothing and deals less per
/// second than the pattern's burst.
fn stick(loud: bool) {
    let mut w = open_floor();
    let me = w.players[0].pos;
    let him = V3::new(me.x.add(metres(12)), Fx::ZERO, me.z);
    w.players[1].pos = him;
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    step(&mut w, 20, 0, 0, 0);
    let mut dealt = 0;
    let mut most = 0;
    let swings = 8;
    for i in 0..swings {
        let before = w.players[1].health;
        step(&mut w, 2, Input::LEFT, 0, 0);
        step(&mut w, RHYTHM - 2, 0, 0, 0);
        dealt += before - w.players[1].health;
        most = most.max(w.players[1].marks);
        w.players[1].pos = him;
        w.players[1].vel = V3::ZERO;
        w.players[1].health = w.players[1].full_health();
        if loud {
            println!("    swing {}: {} dealt so far", i + 1, dealt);
        }
    }
    println!(
        "  {swings} swings in melee dealt {dealt} ({} a swing), most marks {most}",
        dealt / swings
    );
}

// ---------------------------------------------------------------------------
// The cash-in at every mark count
// ---------------------------------------------------------------------------

fn cash_table() {
    for slot in [SLOT_POKE, SLOT_COMMITTED] {
        let m = sim::moves::get(Class::ShadowReaver, slot);
        let row: Vec<String> = (0..=t::mark_cap())
            .map(|marks| {
                let d = Fx::from_int(m.damage)
                    .mul(sim::shadow::cash_multiple(marks))
                    .to_int();
                if sim::shadow::full_tally(marks) {
                    format!("{marks}: {d} + stagger")
                } else {
                    format!("{marks}: {d}")
                }
            })
            .collect();
        println!("  {:<12} {}", m.name, row.join("  |  "));
    }
    let weakest = sim::class::ALL_CLASSES
        .iter()
        .map(|&c| t::health_of(c))
        .min()
        .unwrap_or(0);
    println!("  the most fragile bar on the roster is {weakest}");
}

// ---------------------------------------------------------------------------
// Formatting, in integers
// ---------------------------------------------------------------------------

fn tenths(v: Fx) -> String {
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    let sign = if t < 0 { "-" } else { "" };
    format!("{sign}{}.{}", (t / 10).abs(), (t % 10).abs())
}

fn hundredths(v: Fx) -> String {
    let t = (v.raw() as i64 * 100 + (1 << 15)) >> 16;
    format!("{}.{:02}", t / 100, (t % 100).abs())
}

fn percent(v: Fx) -> i64 {
    (v.raw() as i64 * 100 + (1 << 15)) >> 16
}
