//! The Bulwark's weight, driven by named scripts.
//!
//!     cargo run -p sim --bin weight              every script
//!     cargo run -p sim --bin weight load stomp   the ones named
//!
//! Nobody can feel a number, but the relationships `bulwark-v2.md` asks for --
//! a block stores its damage, a parry stores more, a full shield empties on a
//! clock, a heavy one is shoved less, the creature's blows load it through the
//! same guard test -- are all things this prints in a second. It is how the
//! mechanic is self-evaluated between the human checkpoints in
//! `docs/design/plans/bulwark-v2.md`, and the first thing a reviewer runs.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers all of it.

use sim::bulwark;
use sim::class::{ALL_CLASSES, Class, Mechanic};
use sim::monster::{self, Doing};
use sim::state::{Action, MAX_PLAYERS};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const SCRIPTS: &[&str] = &[
    "openers", "load", "decay", "pushback", "stomp", "slam", "wall",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let chosen: Vec<&str> = if args.is_empty() {
        SCRIPTS.to_vec()
    } else {
        args.iter().map(|a| a.as_str()).collect()
    };

    println!(
        "weight: holds {}  |  full empties in {}f ({}/s)  |  parry loads x{}  |  pushback at the cap x{}  |  bulwark health {} of {}\n",
        t::weight_cap().to_int(),
        t::weight_drain_frames(),
        tenths(
            t::weight_cap()
                .mul(Fx::from_int(sim::TICK_HZ as i32))
                .div(Fx::from_int(t::weight_drain_frames().max(1)))
        ),
        tenths(t::parry_load()),
        tenths(t::heavy_pushback()),
        t::health_of(Class::Bulwark),
        t::max_health(),
    );

    for name in chosen {
        match name {
            "openers" => openers(),
            "load" => load(),
            "decay" => decay(),
            "pushback" => pushback(),
            "stomp" => stomp(),
            "slam" => slam(),
            "wall" => wall(),
            other => println!("{other}: no such script (have {})\n", SCRIPTS.join(", ")),
        }
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// One frame of buttons for two fighters looking straight at each other, the
/// attacker on the left.
fn face_off(attacker: u16, bulwark: u16) -> [Input; 2] {
    [
        Input::aimed(attacker, 0),
        Input::aimed(bulwark, 2 * Input::QUARTER_TURN),
    ]
}

/// A fixture position, in whole metres. The places this instrument stands
/// people are where it looks, not numbers anybody tunes -- which is what the
/// Oven's literal check is guarding -- so they are written through these two
/// rather than as `Fx` literals.
fn metres(n: i32) -> Fx {
    Fx::from_int(n)
}

/// A fixture position, in tenths of a metre.
fn decimetres(n: i32) -> Fx {
    Fx::ratio(n, 10)
}

/// Somebody in reach of a Bulwark on their right, mid-arena, so a shove has
/// room to be a shove rather than a wall.
fn facing(attacker: Class) -> World {
    let mut w = World::with_classes([attacker, Class::Bulwark]);
    w.players[0].pos.x = metres(-2);
    w.players[1].pos.x = decimetres(-5);
    w
}

fn weight(w: &World) -> Fx {
    bulwark::weight(&w.players[1])
}

fn set_weight(w: &mut World, weight: Fx) {
    if let Mechanic::Shield(s) = w.players[1].mechanic {
        w.players[1].mechanic = Mechanic::Shield(s.with_weight(weight));
    }
}

/// What one exchange did to the shield.
#[derive(Clone, Copy, Default)]
struct Took {
    /// The biggest single-frame rise in weight: the deposit.
    deposit: i32,
    health: i32,
    parried: bool,
}

/// Run `frames` of one attacker pressing `attack` on the frames it says, the
/// Bulwark guarding on the frames `guard` says.
fn exchange(
    w: &mut World,
    frames: i32,
    attack: impl Fn(i32) -> u16,
    guard: impl Fn(i32) -> bool,
) -> Took {
    let mut took = Took::default();
    let before = w.players[1].health;
    for f in 0..frames {
        let was = weight(w);
        let b = if guard(f) { Input::RIGHT } else { 0 };
        w.advance(face_off(attack(f), b));
        took.deposit = took.deposit.max(weight(w).sub(was).to_int());
        took.parried |= w.players[1].parried > 0;
    }
    took.health = before - w.players[1].health;
    took
}

/// One press of the opener into a guard raised `early` frames before it (a
/// negative number raises it after). Forty frames of lead-in so the guard can
/// be up long before.
fn one_press(attacker: Class, early: i32) -> Took {
    let mut w = facing(attacker);
    exchange(
        &mut w,
        100,
        |f| if f == 40 { Input::LEFT } else { 0 },
        |f| f >= 40 - early,
    )
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

/// Every class's opener, blocked and parried.
fn openers() {
    println!("openers -- each class's LMB, once, into a guard");
    println!("  class          unblocked  blocked -> weight  parried -> weight");
    for class in ALL_CLASSES {
        let open = one_press(class, -1000);
        let block = one_press(class, 30);
        let parry = (-15..30).map(|e| one_press(class, e)).find(|t| t.parried);
        let parried = match parry {
            Some(p) => format!("{:>6}", p.deposit),
            None => "    --".to_string(),
        };
        println!(
            "  {:<14} {:>9}  {:>16}  {:>17}{}",
            class.name(),
            open.health,
            block.deposit,
            parried,
            if block.health > 0 {
                "  (guard took damage)"
            } else {
                ""
            },
        );
    }
    println!();
}

/// Block five hits of a Champion's sword string, then parry one.
fn load() {
    println!("load -- a Champion sword string, five blocked, then a parry");
    let mut w = facing(Class::Champion);
    // Guard up and settled.
    exchange(&mut w, 30, |_| 0, |_| true);
    let mut blocked = 0;
    let mut sum = 0;
    let mut frame = 0;
    while blocked < 5 && frame < 600 {
        let was = weight(&w);
        let free = w.players[0].action.actionable();
        w.advance(face_off(if free { Input::LEFT } else { 0 }, Input::RIGHT));
        // Held at its mark, so the string keeps reaching.
        w.players[0].pos.x = metres(-2);
        w.players[1].pos.x = decimetres(-5);
        let now = weight(&w);
        let deposit = now.sub(was).to_int();
        if deposit > 0 {
            blocked += 1;
            sum += deposit;
            println!(
                "  frame {frame:>4}: blocked, +{deposit:>4}  ->  weight {:>4}  (sum of deposits {sum})",
                now.to_int()
            );
        }
        frame += 1;
    }
    // The deposits and the weight differ by the drain between blows, and by
    // nothing else -- which is the relationship this script exists to show.
    println!(
        "  weight {} = deposits {sum} less {} drained over {frame} frames",
        weight(&w).to_int(),
        sum - weight(&w).to_int()
    );
    // Then a parry: try each moment to raise the guard from a fresh swing, and
    // keep the first that parries.
    let at = w.clone();
    let parry = (0..30).find_map(|late| {
        let mut w = at.clone();
        // Past the repeat lockout, so the press is a swing and not refused.
        exchange(&mut w, 40, |_| 0, |_| false);
        let before = weight(&w);
        let took = exchange(
            &mut w,
            40,
            |f| if f == 0 { Input::LEFT } else { 0 },
            |f| f >= late,
        );
        took.parried.then_some((late, before, weight(&w), took))
    });
    match parry {
        Some((late, before, after, took)) => println!(
            "  parried (guard raised {late}f after the press): +{}  ->  weight {} from {}",
            took.deposit,
            after.to_int(),
            before.to_int()
        ),
        None => println!("  no timing in thirty frames parried the swing"),
    }
    println!(
        "  never past the cap of {}: {}\n",
        t::weight_cap().to_int(),
        if weight(&w).raw() <= t::weight_cap().raw() {
            "yes"
        } else {
            "NO"
        }
    );
}

/// A full shield, and nothing landing on it.
fn decay() {
    println!("decay -- full, then nothing");
    let mut w = facing(Class::Champion);
    w.players[0].pos.x = metres(-4);
    set_weight(&mut w, t::weight_cap());
    let mut line = String::from(" ");
    let mut empty_at = None;
    for f in 1..=(t::weight_drain_frames() + 120) {
        w.advance(face_off(0, 0));
        if f % 60 == 0 {
            line.push_str(&format!(" {}s:{}", f / 60, weight(&w).to_int()));
        }
        if empty_at.is_none() && weight(&w).raw() == 0 {
            empty_at = Some(f);
        }
    }
    println!("{line}");
    match empty_at {
        Some(f) => println!(
            "  empty at frame {f}; the knob says {}\n",
            t::weight_drain_frames()
        ),
        None => println!("  never emptied\n"),
    }
}

/// One blocked sword swing, at five weights.
fn pushback() {
    println!("pushback -- one blocked Champion sword swing, at five weights");
    println!("  the shove is the knockback plus the swing's own step, which walks the");
    println!("  Champion into the guard and moves both bodies whatever the shield weighs");
    println!("  weight   curve   knockback   shoved");
    for fifth in 0..=4 {
        let load = t::weight_cap().mul(Fx::ratio(fifth, 4));
        let mut w = facing(Class::Champion);
        exchange(&mut w, 30, |_| 0, |_| true);
        set_weight(&mut w, load);
        let curve = bulwark::pushback(&w.players[1]);
        let from = w.players[1].pos;
        let mut kicked = Fx::ZERO;
        let mut took = Took::default();
        for f in 0..40 {
            let step = exchange(
                &mut w,
                1,
                |_| if f == 0 { Input::LEFT } else { 0 },
                |_| true,
            );
            took.health += step.health;
            if kicked.raw() == 0 && matches!(w.players[1].action, Action::BlockStun { .. }) {
                kicked = w.players[1].vel.flat_len();
            }
        }
        println!(
            "  {:>6}   x{}   {} m/s    {} cm{}",
            load.to_int(),
            tenths(curve),
            tenths(kicked),
            centimetres(w.players[1].pos.sub(from).flat_len()),
            if took.health > 0 {
                "  (guard took damage)"
            } else {
                ""
            },
        );
    }
    println!();
}

/// Two Bulwarks, the one on the left holding `weight`, the other a dummy in
/// reach of his Slam.
fn slam_at(weight: Fx) -> World {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    w.players[0].pos.x = metres(-2);
    w.players[1].pos.x = decimetres(-5);
    if let Mechanic::Shield(s) = w.players[0].mechanic {
        w.players[0].mechanic = Mechanic::Shield(s.with_weight(weight));
    }
    w
}

/// What one Slam did: damage, the widest its volume was drawn, what the
/// shield held after, and whether the dummy was left staggered.
struct Slammed {
    damage: i32,
    radius: Fx,
    after: Fx,
    staggered: bool,
    fell: Fx,
}

/// Run a world with player one pressing `bits` on the frames `press` says,
/// until his Slam is over, and report it.
fn slam_run(w: &mut World, frames: i32, press: impl Fn(i32, &World) -> u16) -> Slammed {
    let before = w.players[1].health;
    let mut radius = Fx::ZERO;
    let mut staggered = false;
    let mut fell = Fx::ZERO;
    for f in 0..frames {
        let bits = press(f, w);
        w.advance(face_off(bits, 0));
        if let Some(h) = sim::state::hitbox(&w.players[0]) {
            if matches!(w.players[0].action, Action::Active { kind, .. } if kind == sim::state::SLOT_COMMITTED)
            {
                radius = radius.max(h.radius);
            }
        }
        fell = fell.max(w.players[0].slam_fall);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }
    Slammed {
        damage: before - w.players[1].health,
        radius,
        after: bulwark::weight(&w.players[0]),
        staggered,
        fell,
    }
}

/// Slam at five weights, and Slam out of a leap to a thrown shield.
fn slam() {
    println!("slam -- a standing Slam into an unguarded dummy, at five weights");
    println!("  weight   damage   shake radius   weight after   staggered");
    for fifth in 0..=4 {
        let mut w = slam_at(t::weight_cap().mul(Fx::ratio(fifth, 4)));
        let held = bulwark::weight(&w.players[0]);
        let r = slam_run(&mut w, 60, |f, _| if f == 0 { Input::MIDDLE } else { 0 });
        println!(
            "  {:>6}   {:>6}   {:>9} m   {:>12}   {}",
            held.to_int(),
            r.damage,
            tenths(r.radius),
            r.after.to_int(),
            if r.staggered { "yes" } else { "no" },
        );
    }
    // Crouching does not duck a shake.
    let mut w = slam_at(Fx::ZERO);
    let before = w.players[1].health;
    for f in 0..60 {
        w.advance(face_off(
            if f == 0 { Input::MIDDLE } else { 0 },
            Input::CROUCH,
        ));
    }
    println!(
        "  into a crouch: {} damage{}",
        before - w.players[1].health,
        if before == w.players[1].health {
            "  (DUCKED)"
        } else {
            ""
        }
    );

    // Out of a leap: throw the shield, leap to it, slam on the way down.
    println!("  out of a leap to a thrown shield, empty:");
    let leap = |f: i32, w: &World| -> u16 {
        let p = &w.players[0];
        match f {
            0 => Input::MECHANIC,
            6 => Input::MECHANIC,
            _ if !p.grounded && p.vel.y.raw() < 0 && p.action.actionable() => Input::MIDDLE,
            _ => 0,
        }
    };
    let standing = sim::moves::get(Class::Bulwark, sim::state::SLOT_COMMITTED).damage;
    for (label, load) in [("empty", Fx::ZERO), ("full", t::weight_cap())] {
        let r = arriving(load, leap);
        println!(
            "    {label}: caught it in the air, fell at {} m/s into the slam: {} damage, shake {} m, weight after {}{}",
            tenths(r.fell),
            r.damage,
            tenths(r.radius),
            r.after.to_int(),
            if r.staggered { ", staggered" } else { "" },
        );
    }
    // Out of a full jump: the most fall a Slam from level ground can carry.
    let jump = |f: i32, w: &World| -> u16 {
        let p = &w.players[0];
        match f {
            0..=20 => Input::SPACE,
            _ if !p.grounded && p.vel.y.raw() < 0 && p.action.actionable() => Input::MIDDLE,
            _ => 0,
        }
    };
    let r = arriving(Fx::ZERO, jump);
    println!(
        "  out of a full jump, empty: fell at {} m/s into it: {} damage, against {standing} standing",
        tenths(r.fell),
        r.damage,
    );
    println!();
}

/// A Slam thrown by a script that moves the Bulwark before it lands: run it
/// once with nobody there to find where he comes down, then again with the
/// dummy standing where the shake lands.
fn arriving(load: Fx, press: impl Fn(i32, &World) -> u16 + Copy) -> Slammed {
    let mut dry = slam_at(load);
    dry.players[1].pos.x = sim::arena::ARENA_HALF;
    slam_run(&mut dry, 90, press);
    let landed = dry.players[0].pos;
    let mut w = slam_at(load);
    let reach = sim::moves::get(Class::Bulwark, sim::state::SLOT_COMMITTED).reach;
    w.players[1].pos.x = landed.x.add(reach);
    slam_run(&mut w, 90, press)
}

/// Two fighters in the open lane down the middle of the arena, clear of both
/// platforms: a Bulwark holding `load` at the south end looking north, and
/// somebody of `class` parked well off to the side.
fn in_the_lane(load: Fx, class: Class) -> World {
    let mut w = World::with_classes([Class::Bulwark, class]);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, metres(-12));
    w.players[1].pos = V3::new(metres(12), Fx::ZERO, metres(12));
    if let Mechanic::Shield(s) = w.players[0].mechanic {
        w.players[0].mechanic = Mechanic::Shield(s.with_weight(load));
    }
    w
}

/// One frame of buttons in the lane: the Bulwark looking north, the other
/// fighter looking south at him.
fn lane(bulwark: u16, other: u16) -> [Input; 2] {
    [
        Input::aimed(bulwark, Input::QUARTER_TURN),
        Input::aimed(other, 3 * Input::QUARTER_TURN),
    ]
}

/// The throw at five weights, and the wall it plants.
fn wall() {
    println!("wall -- the throw into an unguarded Bulwark standing on its line, at five weights");
    println!("  weight   arrives   damage   knocked down   planted with");
    // Where the shield is eight frames into an empty throw: the dummy stands
    // there, so every throw below is aimed exactly at it.
    let mut dry = in_the_lane(Fx::ZERO, Class::Bulwark);
    dry.advance(lane(Input::MECHANIC, 0));
    for _ in 0..8 {
        dry.advance(lane(0, 0));
    }
    let Some(on_the_line) = dry.players[0].shield().and_then(|s| s.world_pos()) else {
        println!("  the shield was not thrown\n");
        return;
    };
    for fifth in 0..=4 {
        let load = t::weight_cap().mul(Fx::ratio(fifth, 4));
        let mut w = in_the_lane(load, Class::Bulwark);
        w.players[1].pos = V3::new(on_the_line.x, Fx::ZERO, on_the_line.z);
        let before = w.players[1].health;
        let mut arrived = None;
        let mut down = false;
        for f in 0..90 {
            w.advance(lane(if f == 0 { Input::MECHANIC } else { 0 }, 0));
            if arrived.is_none() && w.players[1].health < before {
                arrived = Some(f);
            }
            down |= matches!(w.players[1].action, Action::Stagger { .. });
        }
        let planted = match w.players[0].shield() {
            Some(sim::class::Shield::Planted { weight, .. }) => weight.to_int().to_string(),
            _ => "--".to_string(),
        };
        println!(
            "  {:>6}   {:>6}f   {:>6}   {:>12}   {planted}",
            load.to_int(),
            arrived.map_or("--".to_string(), |f| f.to_string()),
            before - w.players[1].health,
            if down { "yes" } else { "no" },
        );
    }

    println!(
        "  planted clear of anybody; an Elementalist walks into it, then fires a Bolt through it"
    );
    println!("  at the Bulwark behind -- and the same again after the shield is recalled:");
    println!("  weight   radius   height   walking   bolt      recalled: walking   bolt");
    for fifth in 0..=4 {
        let load = t::weight_cap().mul(Fx::ratio(fifth, 4));
        let standing = wall_trial(load, false);
        let recalled = wall_trial(load, true);
        let said = |stopped: bool| if stopped { "stopped" } else { "through" };
        println!(
            "  {:>6}   {:>4} m   {:>4} m   {:<8}  {:<8}            {:<8}  {}",
            load.to_int(),
            tenths(standing.radius),
            tenths(standing.height),
            said(standing.walked_stopped),
            said(standing.bolt_stopped),
            said(recalled.walked_stopped),
            said(recalled.bolt_stopped),
        );
    }
    println!();
}

/// What one wall did.
struct Walled {
    radius: Fx,
    height: Fx,
    walked_stopped: bool,
    bolt_stopped: bool,
}

/// Plant a shield holding `load` in the lane, optionally recall it, then walk
/// an Elementalist at where it stood and have her fire a Bolt at the Bulwark
/// standing behind it.
fn wall_trial(load: Fx, recall: bool) -> Walled {
    let mut w = in_the_lane(load, Class::Elementalist);
    w.advance(lane(Input::MECHANIC, 0));
    for _ in 0..60 {
        w.advance(lane(0, 0));
    }
    let Some(sim::class::Shield::Planted { pos, .. }) = w.players[0].shield() else {
        println!("  the shield never planted");
        return Walled {
            radius: Fx::ZERO,
            height: Fx::ZERO,
            walked_stopped: false,
            bolt_stopped: false,
        };
    };
    let solid = sim::stones::gather(&w.players)[0].expect("a planted shield is in the field");
    if recall {
        w.advance(lane(Input::MECHANIC, 0));
        w.advance(lane(0, 0));
    }
    // The Bulwark two metres behind it, the Elementalist four in front and
    // walking at it.
    let behind = V3::new(pos.x, Fx::ZERO, pos.z.sub(metres(2)));
    let front = V3::new(pos.x, Fx::ZERO, pos.z.add(metres(4)));
    w.players[0].pos = behind;
    w.players[1].pos = front;
    for _ in 0..90 {
        w.advance(lane(0, Input::W));
        w.players[0].pos = behind;
    }
    let walked_stopped = w.players[1].pos.z.raw() > pos.z.add(solid.radius()).raw();
    // Back to her mark, and shoot -- at the pitch that puts her crosshair on
    // him with nothing in the way, found by trying them against the same
    // scene with the shield recalled. A Bolt that misses with no wall there
    // says nothing about the wall.
    w.players[1].pos = front;
    let mut open = w.clone();
    if let Mechanic::Shield(s) = open.players[0].mechanic {
        if !s.in_hand() {
            open.players[0].mechanic =
                Mechanic::Shield(sim::class::Shield::Held { weight: s.weight() });
        }
    }
    let on_him = (-300..300)
        .step_by(5)
        .map(|tenth| (tenth * 65536 / 3600) as i16)
        .find(|pitch| bolt_lands(&open, behind, *pitch));
    Walled {
        radius: solid.radius(),
        height: solid.height(),
        walked_stopped,
        bolt_stopped: on_him.is_none_or(|pitch| !bolt_lands(&w, behind, pitch)),
    }
}

/// Does a Bolt fired by the second fighter at `pitch` reach the Bulwark held
/// at `behind`?
fn bolt_lands(w: &World, behind: V3, pitch: i16) -> bool {
    let mut w = w.clone();
    let before = w.players[0].health;
    for f in 0..60 {
        let bits = if f == 0 { Input::LEFT } else { 0 };
        w.advance([
            Input::aimed(0, Input::QUARTER_TURN),
            Input::looking_at(bits, 3 * Input::QUARTER_TURN, pitch),
        ]);
        w.players[0].pos = behind;
    }
    w.players[0].health < before
}

/// A Bulwark with his guard up at each creature move's own range, facing it.
fn stomp() {
    println!("stomp -- each creature move, into a guard at its own ideal range");
    println!("  move           damage  taken  deposit  parried");
    for kind in 0..monster::MOVES as u8 {
        let m = monster::attack(kind);
        let mut w = World::hunt([Class::Bulwark; MAX_PLAYERS]);
        let beast = w.monster.expect("a hunt has a creature");
        w.players[0].pos = beast
            .rig()
            .to_world(V3::new(m.ideal_range, Fx::ZERO, Fx::ZERO));
        w.players[0].grounded = true;
        // Parked in a corner, so the second fighter is not a second target.
        let corner = sim::arena::ARENA_HALF;
        w.players[1].pos = V3::new(corner, w.players[1].pos.y, corner);
        let toward = beast.pos.sub(w.players[0].pos);
        let aim = (sim::math::atan2_turns(toward.z, toward.x).raw() & 0xFFFF) as u16;
        w.players[0].facing = V3::new(toward.x, Fx::ZERO, toward.z).normalized();
        w.players[0].action = Action::Guard { held: 60 };
        w.monster.as_mut().unwrap().doing = Doing::Startup {
            kind,
            left: m.startup,
        };
        let before = w.players[0].health;
        let mut deposit = 0;
        let mut parried = false;
        for _ in 0..m.total() {
            w.monster.as_mut().unwrap().brain.think_left = u16::MAX;
            let was = bulwark::weight(&w.players[0]);
            w.advance([Input::aimed(Input::RIGHT, aim), Input::default()]);
            deposit = deposit.max(bulwark::weight(&w.players[0]).sub(was).to_int());
            parried |= w.players[0].parried > 0;
        }
        let taken = before - w.players[0].health;
        let reached = taken > 0 || deposit > 0;
        println!(
            "  {:<13} {:>7}  {:>5}  {:>7}  {}{}",
            m.name,
            m.damage,
            taken,
            deposit,
            if parried { "yes" } else { "no" },
            if reached {
                ""
            } else {
                "  (did not reach from in front)"
            },
        );
    }
    println!();
}

/// Metres as whole centimetres, rounded, in integer arithmetic.
fn centimetres(v: Fx) -> i64 {
    (v.raw() as i64 * 100 + (1 << 15)) >> 16
}

fn tenths(v: Fx) -> String {
    // Rounded, not truncated: 4.199 should read as 4.2, not 4.1.
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    format!("{}.{}", t / 10, (t % 10).abs())
}
