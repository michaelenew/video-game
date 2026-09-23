//! The Blood mage's instrument: pools, drinks, grey and reach, measured.
//!
//!     cargo run -p sim --bin essence
//!
//! Runs a fixed scripted exchange against the training dummy and against the
//! creature, and prints every number the class's design asks about: the pool
//! each move leaves and how long it lives, what each move drinks when it lands
//! over a pool, the grey bar over time, and the scythe's reach at each level
//! of grey. It is how a change to the class is self-evaluated in seconds, and
//! the first thing a reviewer runs. See `docs/design/plans/blood-mage-v1.md`.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers it.

use sim::class::Class;
use sim::effects::Effect;
use sim::moves::blood as b;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

fn main() {
    println!("The Blood mage's essence, measured.\n");
    println!(
        "grey fades {}/s  |  scythe x{} reach, x{} damage at full grey  |  pools drain {}/s, \
         a full figure at {} essence, never under x{} of a body, cap {}",
        t::grey_fade(),
        tenths(t::grey_reach()),
        tenths(t::grey_damage()),
        t::pool_drain(),
        t::pool_full(),
        hundredths(t::pool_least()),
        t::pool_cap()
    );
    println!();

    reach_table();
    pool_table();
    drink_table();
    grey_over_time();
    against_the_creature();

    println!(
        "A pool's volume is the damage that made it and its life is that volume over the\n\
         drain rate, so a Reap's pool outlives a sweep's by their damage ratio. A drink is\n\
         the move's share of the pool, converted out of grey and never past it; grey is the\n\
         ceiling on every heal she has. Reach is the sweep's base times the grey curve, and\n\
         the blade is drawn at exactly that. Record what you change in docs/design/feel-log.md."
    );
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    looking(w, frames, a, 0, b);
}

fn looking(w: &mut World, frames: u32, a: u16, pitch: i16, b: u16) {
    for _ in 0..frames {
        // She looks along +x from the left; the dummy faces her from the
        // right, half a turn round.
        w.advance([
            Input::looking_at(a, 0, pitch),
            Input::aimed(b, Input::QUARTER_TURN << 1),
        ]);
    }
}

fn mage() -> World {
    World::with_classes([Class::BloodMage, Class::Bulwark])
}

fn pools(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.is_a_pool())
        .copied()
        .collect()
}

/// The dummy inside the scythe's reach, standing still.
fn in_reach(w: &mut World) {
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    w.players[1].pos = w.players[0].pos.add(V3::new(
        sweep.reach.mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        Fx::ZERO,
    ));
    w.players[1].vel = V3::ZERO;
    w.players[1].action = Action::Free;
}

fn wounded(w: &mut World, grey: i32) {
    w.players[0].health = t::max_health() - grey;
    w.players[0].grey = grey;
}

/// A pitch that points a skillshot at the dummy's middle. The same scan the
/// tests use: the crosshair's line is solved by `sim::aim`, and this only asks
/// which pitch lands it nearest the body.
fn pitch_at(w: &World, slot: u8, target: V3) -> i16 {
    let m = sim::moves::get(Class::BloodMage, slot);
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    };
    // A skillshot is pointed at the body's middle; a grounded move at its
    // feet, since that is where it lands.
    let grounded = matches!(m.aim(), sim::aim::Kind::Grounded);
    let middle = if grounded {
        target
    } else {
        sim::aim::standing_middle(target)
    };
    let miss = |pitch: i16| {
        let look = Input::looking_at(0, 0, pitch);
        if grounded {
            let path = sim::aim::grounded_path(0, look, m.reach, &scene);
            return path.to.sub(middle).len().raw();
        }
        let path = sim::aim::skillshot_path(0, look, m.reach, &scene);
        let dir = path.dir();
        let toward = middle.sub(path.from);
        let down = toward.dot(dir).max(Fx::ZERO).min(m.reach);
        path.from.add(dir.scale(down)).sub(middle).len().raw()
    };
    (-40..=80)
        .map(|step| -(step * 200) as i16)
        .min_by_key(|pitch| miss(*pitch))
        .expect("the scan is not empty")
}

/// Which button throws each of her moves, in the order the kit reads.
fn kit() -> [(u8, u16); 5] {
    [
        (b::SWEEP, Input::LEFT),
        (b::REAP, Input::RIGHT),
        (b::BLOODLETTER, Input::MIDDLE),
        (b::GRASP, Input::SPECIAL),
        (b::BLACK_SPIKE, Input::MECHANIC),
    ]
}

/// Throw one move at the dummy and wait for it to have finished happening.
/// Gives back the biggest pool seen while it happened -- what the hit spilled
/// before the drain started on it.
fn cast(w: &mut World, slot: u8, button: u16) -> Option<Effect> {
    let m = sim::moves::get(Class::BloodMage, slot);
    let pitch = pitch_at(w, slot, w.players[1].pos);
    // The Grasp is thrown by the release, wound to a depth; hold it long enough
    // to reach the dummy.
    let hold = if m.channels() {
        let want = w.players[1].pos.sub(w.players[0].pos).flat_len();
        let mut held = 0;
        while held < m.channel && m.reach_after(held).raw() < want.raw() {
            held += 1;
        }
        held as u32 + 1
    } else {
        2
    };
    looking(w, hold, button, pitch, 0);
    let flight = t::bloodletter_flight().max(t::grasp_flight()) as u32;
    let mut biggest: Option<Effect> = None;
    for _ in 0..m.whiff_cost() as u32 + flight {
        looking(w, 1, 0, pitch, 0);
        for pool in pools(w) {
            if biggest.is_none_or(|b| pool.pool_volume() > b.pool_volume()) {
                biggest = Some(pool);
            }
        }
    }
    biggest
}

// ---------------------------------------------------------------------------
// The tables
// ---------------------------------------------------------------------------

fn reach_table() {
    println!("Reach: the scythe at each level of grey");
    println!(
        "  {:>6}{:>10}{:>10}{:>10}",
        "grey", "sweep", "reap", "damage x"
    );
    let sweep = sim::moves::get(Class::BloodMage, b::SWEEP);
    let reap = sim::moves::get(Class::BloodMage, b::REAP);
    for grey in [0, 250, 500, 750, t::max_health() - 1] {
        let mut p = sim::state::Player::new(Class::BloodMage);
        p.health = t::max_health() - grey;
        p.grey = grey;
        let power = crate::lerp(Fx::ONE, t::grey_damage(), p.grey_share());
        println!(
            "  {:>6}{:>10}{:>10}{:>10}",
            grey,
            tenths(sim::state::live_reach(&p, &sweep)),
            tenths(sim::state::live_reach(&p, &reap)),
            hundredths(power),
        );
    }
    println!();
}

fn lerp(a: Fx, b: Fx, at: Fx) -> Fx {
    a.add(b.sub(a).mul(at))
}

fn pool_table() {
    println!("Pools: what each move leaves when it lands on the dummy on bare floor");
    println!(
        "  {:<14}{:>7}{:>8}{:>8}{:>10}{:>8}",
        "move", "cost", "dealt", "volume", "radius m", "lives f"
    );
    for (slot, button) in kit() {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut w = mage();
        in_reach(&mut w);
        let full = w.players[1].health;
        let biggest = cast(&mut w, slot, button);
        let dealt = full - w.players[1].health;
        let Some(pool) = biggest else {
            println!(
                "  {:<14}{:>7}{:>8}{:>8}{:>10}{:>8}",
                m.name, m.cost, dealt, "--", "--", "--"
            );
            continue;
        };
        let volume = pool.pool_volume();
        let radius = pool.pool_radius();
        let mut lives = 0;
        while pools(&w).iter().any(|p| p.pos == pool.pos) && lives < 20_000 {
            run(&mut w, 1, 0, 0);
            lives += 1;
        }
        println!(
            "  {:<14}{:>7}{:>8}{:>8}{:>10}{:>8}",
            m.name,
            m.cost,
            dealt,
            volume,
            hundredths(radius),
            lives
        );
    }
    println!();
}

fn drink_table() {
    let reap = sim::moves::get(Class::BloodMage, b::REAP);
    let volume = reap.damage;
    println!(
        "Drinks: each move landed on the dummy standing in a pool of {volume} (a Reap's), with 500 grey open. \
         A drink takes what is left and the pool is gone"
    );
    println!(
        "  {:<14}{:>7}{:>9}{:>9}{:>12}",
        "move", "cost", "share %", "drank", "spilled"
    );
    for (slot, button) in kit() {
        let m = sim::moves::get(Class::BloodMage, slot);
        let mut w = mage();
        in_reach(&mut w);
        wounded(&mut w, 500);
        let at = w.players[1].pos;
        w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, volume));
        let before = w.players[0].health;
        cast(&mut w, slot, button);
        let drank = w.players[0].health - (before - m.cost);
        let after = w.effects[0].map_or(0, |p| if p.is_a_pool() { p.pool_volume() } else { 0 });
        println!(
            "  {:<14}{:>7}{:>9}{:>9}{:>12}",
            m.name, m.cost, m.drink, drank, after
        );
    }
    println!();
}

fn grey_over_time() {
    println!(
        "Grey over time: a Reap at nothing, two bashes taken, a wait, then a Reap over a pool"
    );
    println!(
        "  {:>6}{:>7}{:>7}{:>7}  event",
        "frame", "red", "grey", "gone"
    );
    let mut w = mage();
    let reap = sim::moves::get(Class::BloodMage, b::REAP);
    let bash = sim::moves::get(Class::Bulwark, sim::state::SLOT_POKE);
    let log = |w: &World, event: &str| {
        let p = w.players[0];
        println!(
            "  {:>6}{:>7}{:>7}{:>7}  {event}",
            w.frame,
            p.health,
            p.grey,
            t::max_health() - p.health - p.grey
        );
    };
    log(&w, "start");
    // Far from the dummy, so the cast opens a wound and nothing else.
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
    run(&mut w, 2, Input::RIGHT, 0);
    log(&w, "Reap cast at nothing: opened by a cost");
    run(&mut w, reap.whiff_cost() as u32, 0, 0);
    log(&w, "recovered");
    // The dummy walks up and bashes her twice.
    w.players[1].pos = w.players[0].pos.add(V3::new(
        // Inside the bash's reach by a body: the dummy at arm's length.
        bash.reach.sub(t::body_radius()),
        Fx::ZERO,
        Fx::ZERO,
    ));
    for _ in 0..2 {
        let was = w.players[0].health;
        for _ in 0..90 {
            run(&mut w, 1, 0, Input::LEFT);
            if w.players[0].health < was {
                break;
            }
        }
        run(&mut w, bash.whiff_cost() as u32 + 4, 0, 0);
        log(&w, "hit by a bash: opened by a hit");
    }
    for _ in 0..3 {
        run(&mut w, 60, 0, 0);
        log(&w, "a second of fading");
    }
    // A pool under the dummy, and a Reap over it.
    in_reach(&mut w);
    let at = w.players[1].pos;
    w.effects[0] = Some(Effect::pool(0, Class::BloodMage, b::SWEEP, at, reap.damage));
    run(&mut w, 2, Input::RIGHT, 0);
    log(&w, "Reap cast over a pool: opened by a cost");
    run(&mut w, reap.startup as u32 + 4, 0, 0);
    log(&w, "Reap landed over the pool: reclaimed");
    println!();
}

fn against_the_creature() {
    println!(
        "Against the creature: the pool under a Ridgeback swept standing and toppled, and Reaped toppled"
    );
    println!(
        "  {:<18}{:>8}{:>8}{:>10}",
        "state", "dealt", "volume", "radius m"
    );
    for (name, toppled, slot, button) in [
        ("swept, standing", false, b::SWEEP, Input::LEFT),
        ("swept, toppled", true, b::SWEEP, Input::LEFT),
        ("Reaped, toppled", true, b::REAP, Input::RIGHT),
    ] {
        let mut w = World::hunt([Class::BloodMage, Class::BloodMage]);
        let mut beast = w.monster.expect("a hunt has a creature");
        beast.pos = V3::ZERO;
        // Half a turn: faced at the hunter, the way a hunt spawns it.
        beast.yaw = Fx::ratio(1, 2);
        beast.doing = if toppled {
            sim::monster::Doing::Toppled { left: 600 }
        } else {
            sim::monster::Doing::Prowl
        };
        beast.brain.think_left = u16::MAX;
        w.monster = Some(beast);
        w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(8));
        let m = sim::moves::get(Class::BloodMage, slot);
        // Walk in from the side until the blade, held level at half its
        // reach, meets a part: a scan over where she stands rather than a
        // typed spot, because a toppled Ridgeback lies lower and elsewhere.
        let mut spot = None;
        'scan: for zi in -30..=30 {
            for step in (-90..0).rev() {
                let x = Fx::ratio(step, 10);
                let z = Fx::ratio(zi, 10);
                let part = w.monster.expect("creature").part_struck(
                    V3::new(x.add(m.reach.mul(Fx::ratio(1, 2))), Fx::ZERO, z),
                    m.radius,
                    t::body_height(),
                );
                if part.is_some() {
                    spot = Some(V3::new(x, Fx::ZERO, z));
                    break 'scan;
                }
            }
        }
        let Some(stand) = spot else {
            println!(
                "  {:<18}{:>8}{:>8}{:>10}",
                name, "--", "--", "no part in reach"
            );
            continue;
        };
        w.players[0].pos = stand;
        let full = w.monster.expect("creature").health;
        run(&mut w, 2, button, 0);
        let mut biggest: Option<Effect> = None;
        for _ in 0..m.whiff_cost() as u32 {
            run(&mut w, 1, 0, 0);
            for pool in pools(&w) {
                if biggest.is_none_or(|b| pool.pool_volume() > b.pool_volume()) {
                    biggest = Some(pool);
                }
            }
        }
        let dealt = full - w.monster.expect("creature").health;
        match biggest {
            Some(pool) => println!(
                "  {:<18}{:>8}{:>8}{:>10}",
                name,
                dealt,
                pool.pool_volume(),
                hundredths(pool.pool_radius())
            ),
            None => println!("  {:<18}{:>8}{:>8}{:>10}", name, dealt, "--", "--"),
        }
    }
    println!();
}

/// One decimal place, without touching floating point.
fn tenths(v: Fx) -> String {
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    format!("{}.{}", t / 10, (t % 10).abs())
}

/// Two decimal places, the same way.
fn hundredths(v: Fx) -> String {
    let h = (v.raw() as i64 * 100 + (1 << 15)) >> 16;
    format!("{}.{:02}", h / 100, (h % 100).abs())
}
