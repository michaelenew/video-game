//! **The Blood mage's kit, measured against what was promised.**
//!
//!     cargo run --release -p sim --example kitcheck
//!
//! Every line is one sentence of the agreed design
//! (`docs/design/exploration/0038_blood_mage_the_world_bleeds.md`, and the
//! owner's report from play on 2026-11-06), played in the simulation the way a
//! player plays it -- the second press early, the button held -- and measured.
//! It prints what it measured beside what was promised, and `MISSED` where
//! the two disagree, and exits non-zero if any did.
//!
//! It exists because slice 1 was handed over on green tests that the builder
//! wrote from the code rather than from the promise, and on nobody playing it:
//! the zip that went "a few inches", the jump scythe lower than a jump, every
//! move still costing health. Run it before handing a change to her kit over,
//! and paste its table into the report (`docs/design/delivery.md` D3).

use sim::class::{Class, Mechanic};
use sim::effects::{EffectKind, Head};
use sim::moves::blood::{self as b, keys};
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}
fn m(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}
fn at(x: f32, z: f32) -> V3 {
    V3::new(m(x), Fx::ZERO, m(z))
}
fn flat(a: V3, b: V3) -> f32 {
    let (dx, dz) = (f(a.x.sub(b.x)), f(a.z.sub(b.z)));
    (dx * dx + dz * dz).sqrt()
}

/// Her at (-6, 8) facing +x, and a Bulwark `apart` metres in front.
fn with_foe(apart: f32) -> World {
    let mut w = World::with_classes([Class::BloodMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 8.0);
    w.players[1].pos = at(-6.0 + apart, 8.0);
    w
}
fn step(w: &mut World, bits: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}
fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        step(w, bits, pitch);
    }
}
fn head(w: &World) -> Option<sim::effects::Effect> {
    w.effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::ScytheHead)
        .copied()
}
fn plants(w: &World) -> usize {
    match w.players[0].mechanic {
        Mechanic::Blood(g) => g.plants.iter().flatten().count(),
        _ => 0,
    }
}
fn hold() -> u32 {
    t::hold_from() as u32 + 2
}

struct Card {
    missed: usize,
}

impl Card {
    fn line(&mut self, promise: &str, measured: String, kept: bool) {
        if !kept {
            self.missed += 1;
        }
        println!(
            "{:<7} {:<62} {}",
            if kept { "kept" } else { "MISSED" },
            promise,
            measured
        );
    }
}

fn main() {
    let mut card = Card { missed: 0 };
    println!("The Blood mage, against what was promised\n");

    // -- What costs -------------------------------------------------------
    for (name, bits, frames) in [
        ("Sweep", keys::SCYTHE, 3u32),
        ("Cast", keys::SCYTHE, hold() + 4),
        ("Rend", keys::HAND, 3),
        ("Red claw", keys::HAND, hold() + 4),
        ("Rising reap", Input::SPACE | keys::SCYTHE, 3),
        ("Rising rend", Input::SPACE | keys::HAND, 3),
    ] {
        let mut w = with_foe(30.0);
        let hp = w.players[0].health;
        for fr in 0..60 {
            step(&mut w, if fr < frames { bits } else { 0 }, -2000);
        }
        let paid = hp - w.players[0].health;
        card.line(
            &format!("{name} costs her nothing"),
            format!("paid {paid}"),
            paid == 0,
        );
    }

    // -- The spike and the plant -------------------------------------------
    {
        // 2026-11-10, from play: middle click only where there is a pool.
        let mut w = with_foe(30.0);
        let hp = w.players[0].health;
        run(&mut w, 2, keys::SPIKES, -3000);
        run(&mut w, 40, 0, -3000);
        card.line(
            "a Black spike on bare ground throws nothing and costs nothing",
            format!("{} plant(s), paid {}", plants(&w), hp - w.players[0].health),
            plants(&w) == 0 && w.players[0].health == hp,
        );
    }
    {
        let mut w = with_foe(30.0);
        let hp = w.players[0].health;
        step(&mut w, Input::SPACE, -3000);
        step(&mut w, Input::SPACE | keys::SPIKES, -3000);
        run(&mut w, 40, 0, -3000);
        let price = hp - w.players[0].health;
        card.line(
            "a Rising spike on bare ground raises a plant, paid in her blood",
            format!("{} plant(s), paid {price}", plants(&w)),
            plants(&w) == 1 && price > 0,
        );
        run(&mut w, 540, 0, -3000);
        let standing = plants(&w) == 1;
        card.line(
            "a plant stands at least ten seconds",
            format!("stands {} s", t::plant_stands() as f32 / 60.0),
            standing && t::plant_stands() >= 600,
        );
        card.line(
            "what a plant holds stays grey on her bar while it stands",
            format!("grey {} of {price}", w.players[0].grey),
            w.players[0].grey == price,
        );
    }
    {
        // Damage of a spike under a foe 8 m off, at the pitch that lands it.
        let mut best = 0;
        for pitch in (2000..7000).step_by(250).map(|p| -(p as i16)) {
            let mut w = with_foe(8.0);
            // A stain of hers under him: the spike raises only a pool
            // (2026-11-10).
            let mut stain =
                sim::effects::Effect::pool(0, Class::BloodMage, b::CAST, w.players[1].pos, 30);
            stain.age = t::stain_settles() + 1;
            if let Some(slot) = w.effects.iter_mut().find(|s| s.is_none()) {
                *slot = Some(stain);
            }
            let his = w.players[1].health;
            step(&mut w, keys::SPIKES, pitch);
            run(&mut w, 25, 0, pitch);
            best = best.max(his - w.players[1].health);
        }
        let row = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).damage;
        card.line(
            "a spike under them does less than the old 100",
            format!("{best} (row {row})"),
            best > 0 && best < 100,
        );
    }

    // -- Up ------------------------------------------------------------------
    let top = |bits: u16| {
        let mut w = with_foe(30.0);
        let mut top = Fx::ZERO;
        for fr in 0..90 {
            step(&mut w, if fr < 12 { bits } else { 0 }, 0);
            top = top.max(w.players[0].pos.y);
        }
        f(top)
    };
    let jump = top(Input::SPACE);
    let reap = top(Input::SPACE | keys::SCYTHE);
    let spike = top(Input::SPACE | keys::SPIKES);
    card.line(
        "the Rising reap takes her far higher than a jump (2x)",
        format!("{reap:.1} m against a jump's {jump:.1} m"),
        reap > jump * 2.0,
    );
    card.line(
        "the spike under her own feet launches her (2x a jump)",
        format!("{spike:.1} m"),
        spike > jump * 2.0,
    );

    // -- The scythe's head ---------------------------------------------------
    {
        let mut w = with_foe(30.0);
        let mut out_at = None;
        for fr in 0..60u32 {
            step(&mut w, keys::SCYTHE, -3000);
            if head(&w).is_some() {
                out_at = Some(fr + 1);
                break;
            }
        }
        let frames = out_at.unwrap_or(999);
        card.line(
            "held, the head leaves her hand within a third of a second",
            format!("{frames} frames from the press"),
            frames <= 20,
        );
    }
    for (when, pressed) in [("before it leaves", 0u32), ("while it flies", 6)] {
        let mut w = with_foe(30.0);
        let from = w.players[0].pos;
        run(&mut w, hold(), keys::SCYTHE, -3000);
        step(&mut w, 0, -3000);
        run(&mut w, pressed, 0, -3000);
        // The zip is a tap of right since 2026-11-10 (left from 2026-11-07).
        step(&mut w, keys::SCYTHE, -3000);
        let mut bit = None;
        for _ in 0..90 {
            step(&mut w, 0, -3000);
            if let Some(h) = head(&w).filter(|h| h.head_state() != Head::Flying) {
                bit.get_or_insert(h.home);
            }
        }
        let out = bit.map_or(0.0, |b| flat(from, b));
        let gone = flat(from, w.players[0].pos);
        card.line(
            &format!("a tap of right {when} zips her all the way"),
            format!("{gone:.1} m of {out:.1} m"),
            out > 3.0 && gone >= out - 2.0,
        );
    }
    {
        let mut w = with_foe(8.0);
        run(&mut w, hold(), keys::SCYTHE, 0);
        step(&mut w, 0, 0);
        step(&mut w, keys::SCYTHE, 0);
        run(&mut w, 60, 0, 0);
        let gap = flat(w.players[0].pos, w.players[1].pos);
        let reach = f(sim::moves::get(Class::BloodMage, b::REND).reach);
        card.line(
            "a zip onto a fighter leaves her in the Red Hand's reach",
            format!("{gap:.1} m apart, reach {reach:.1} m"),
            gap < reach,
        );
    }
    // -- The head is a weight on a chain (round three, 2026-11-07) ---------
    {
        // Thrown level at nothing: it arcs and comes down on the floor.
        let mut w = with_foe(60.0);
        run(&mut w, hold(), keys::SCYTHE, 0);
        let (mut top, mut landed, mut was_y, mut fell) = (0.0f32, None, 0.0f32, false);
        for _ in 0..120 {
            step(&mut w, 0, 0);
            if let Some(h) = head(&w) {
                let y = f(h.home.y);
                top = top.max(y);
                if h.head_state() == Head::Flying && y < was_y {
                    fell = true;
                }
                was_y = y;
                if h.head_state() == Head::World {
                    landed.get_or_insert(y);
                }
            }
        }
        card.line(
            "thrown at nothing, the head arcs, falls and lands on the floor",
            format!(
                "top {top:.1} m, fell {fell}, landed at {:.1} m",
                landed.unwrap_or(-1.0)
            ),
            fell && landed.is_some_and(|y| y < 0.3),
        );
    }
    {
        // Aimed at a fighter 10 m off: the arc still meets the crosshair.
        let mut bit = false;
        for pitch in (-1500..1500).step_by(150) {
            let mut w = with_foe(10.0);
            run(&mut w, hold(), keys::SCYTHE, pitch as i16);
            for _ in 0..60 {
                step(&mut w, 0, pitch as i16);
                bit |= head(&w).is_some_and(|h| matches!(h.head_state(), Head::Fighter(_)));
            }
            if bit {
                break;
            }
        }
        card.line(
            "aimed at a fighter 10 m off, the throw still bites him",
            format!("bitten {bit}"),
            bit,
        );
    }
    {
        // Called home: it starts slow and speeds up.
        let mut w = with_foe(60.0);
        run(&mut w, hold(), keys::SCYTHE, -2000);
        let mut speeds = Vec::new();
        for _ in 0..400 {
            step(&mut w, 0, -2000);
            if let Some(h) = head(&w).filter(|h| h.head_state() == Head::Home) {
                speeds.push(f(h.dir.len()));
            }
        }
        let (first, most) = (
            speeds.first().copied().unwrap_or(0.0),
            speeds.iter().copied().fold(0.0, f32::max),
        );
        card.line(
            "reeled home, the head starts slow and speeds up",
            format!("{first:.1} m/s rising to {most:.1} m/s"),
            !speeds.is_empty() && most > first * 2.0,
        );
    }
    {
        // The zip is a pull: her speed rises along it.
        let mut w = with_foe(30.0);
        run(&mut w, hold(), keys::SCYTHE, -2500);
        step(&mut w, 0, -2500);
        step(&mut w, keys::SCYTHE, -2500);
        let mut speeds = Vec::new();
        for _ in 0..90 {
            step(&mut w, 0, -2500);
            if matches!(w.players[0].mechanic, Mechanic::Blood(g) if g.zipping) {
                speeds.push(f(w.players[0].vel.len()));
            }
        }
        let (first, most) = (
            speeds.first().copied().unwrap_or(0.0),
            speeds.iter().copied().fold(0.0, f32::max),
        );
        card.line(
            "the zip is a pull: she speeds up along the chain",
            format!("{first:.1} m/s rising to {most:.1} m/s"),
            !speeds.is_empty() && most > first * 1.5,
        );
    }

    // -- The Red Hand ---------------------------------------------------------
    {
        let mut w = with_foe(2.2);
        let his = w.players[1].health;
        for fr in 0..70u32 {
            let press = matches!(fr, 0 | 16 | 32);
            step(&mut w, if press { keys::HAND } else { 0 }, 0);
        }
        let dealt = his - w.players[1].health;
        let reach = f(sim::moves::get(Class::BloodMage, b::REND).reach);
        card.line(
            "Rend is big: three claws from 2 m away, 150 or more",
            format!("{dealt} dealt, reach {reach:.1} m"),
            dealt >= 150,
        );
    }
    // -- The chain on the right click (round three; right since 2026-11-10) ----
    // The head in a fighter 10 m off, thrown level.
    let hooked = || {
        for pitch in (-1500..1500).step_by(150) {
            let mut w = with_foe(10.0);
            run(&mut w, hold(), keys::SCYTHE, pitch as i16);
            for _ in 0..40 {
                step(&mut w, 0, pitch as i16);
                if head(&w).is_some_and(|h| matches!(h.head_state(), Head::Fighter(_))) {
                    return w;
                }
            }
        }
        panic!("no throw bit the fighter");
    };
    {
        let mut w = hooked();
        step(&mut w, keys::SCYTHE, 0);
        step(&mut w, 0, 0);
        run(&mut w, 40, 0, 0);
        let gap = flat(w.players[0].pos, w.players[1].pos);
        card.line(
            "a tap of right pulls her to what the head bit",
            format!("{gap:.1} m from him after"),
            gap < 2.6,
        );
    }
    {
        let mut w = hooked();
        run(&mut w, t::reel_full() as u32 + 2, keys::SCYTHE, 0);
        run(&mut w, 90, 0, 0);
        let gap = flat(w.players[0].pos, w.players[1].pos);
        card.line(
            "right held to its end reels him in to her",
            format!("{gap:.1} m from her after, from 10 m"),
            gap < 2.5,
        );
    }
    {
        let mut w = hooked();
        let his = w.players[1].health;
        run(&mut w, t::hold_from() as u32 + 4, keys::SCYTHE, 0);
        step(&mut w, 0, 0);
        let staggered = matches!(
            w.players[1].action,
            Action::Stagger { .. } | Action::HitStun { .. }
        );
        run(&mut w, 3, 0, 0);
        let home = head(&w).is_none_or(|h| h.head_state() == Head::Home);
        let dealt = his - w.players[1].health;
        card.line(
            "right let go early, the head rips out: hit, bleed, stagger",
            format!(
                "{dealt} dealt, staggered {staggered}, bleeding {}, head home {home}",
                w.players[1].bleeding > 0
            ),
            dealt > 0 && staggered && w.players[1].bleeding > 0 && home,
        );
    }
    {
        // The head in the floor, and right held: it is reeled home.
        let mut w = with_foe(60.0);
        run(&mut w, hold(), keys::SCYTHE, 0);
        for _ in 0..120 {
            step(&mut w, 0, 0);
            if head(&w).is_some_and(|h| h.head_state() == Head::World) {
                break;
            }
        }
        run(&mut w, t::reel_full() as u32 + 2, keys::SCYTHE, 0);
        let home = head(&w).is_some_and(|h| h.head_state() == Head::Home);
        card.line(
            "right held with the head in the floor reels it home",
            format!("home {home}"),
            home,
        );
    }
    {
        // The shrike: a plant between them, and he is reeled into it.
        let mut w = hooked();
        if let Mechanic::Blood(g) = &mut w.players[0].mechanic {
            g.plants[0] = Some(sim::class::Structure::planted(at(-1.0, 8.0)));
        }
        let mut pinned = false;
        for f in 0..120u32 {
            step(
                &mut w,
                if f < t::reel_full() as u32 + 2 {
                    keys::SCYTHE
                } else {
                    0
                },
                0,
            );
            pinned |= matches!(w.players[1].action, Action::Held { .. })
                && w.players[1].held_by == sim::state::NAILED;
        }
        card.line(
            "reeled into one of her plants, he is impaled on it",
            format!("pinned {pinned}"),
            pinned,
        );
    }

    // -- The Red Hand (round three) ---------------------------------------------
    {
        let mut w = with_foe(2.2);
        let mut last = w.players[1].health;
        let mut hits = Vec::new();
        for fr in 0..80u32 {
            step(
                &mut w,
                if matches!(fr, 0 | 16 | 32) {
                    keys::HAND
                } else {
                    0
                },
                0,
            );
            if w.players[1].health != last {
                hits.push(last - w.players[1].health);
                last = w.players[1].health;
            }
        }
        let rising = hits.len() == 3 && hits[0] < hits[1] && hits[1] < hits[2];
        card.line(
            "three taps are three blows, each bigger than the last",
            format!("{hits:?}"),
            rising,
        );
    }
    let claw_at = |held: u32| {
        let mut w = with_foe(40.0);
        let from = w.players[0].pos;
        run(&mut w, held, keys::HAND, -2000);
        for _ in 0..30 {
            step(&mut w, 0, -2000);
            if let Some(c) = w
                .effects
                .iter()
                .flatten()
                .find(|e| e.kind == EffectKind::Claw)
            {
                return (flat(from, c.pos), f(c.reach));
            }
        }
        (0.0, 0.0)
    };
    {
        let (near, small) = claw_at(t::hold_from() as u32 + 8);
        let (far, big) = claw_at(t::hold_from() as u32 + 45);
        card.line(
            "held, the claw lands further out and bigger the longer the hold",
            format!("{near:.1} m wide {small:.1} -> {far:.1} m wide {big:.1}"),
            far > near + 3.0 && big > small,
        );
    }
    {
        let mut w = with_foe(3.0);
        let his = w.players[1].health;
        run(&mut w, t::hold_from() as u32 + 70, keys::HAND, -2000);
        run(&mut w, 30, 0, -2000);
        card.line(
            "held too long, the claw lands past a fighter 3 m off",
            format!("dealt {}", his - w.players[1].health),
            w.players[1].health == his,
        );
    }
    {
        let mut slowed = false;
        for held in 0..40u32 {
            let mut w = with_foe(5.0);
            let his = w.players[1].health;
            run(&mut w, t::hold_from() as u32 + held, keys::HAND, -2500);
            for _ in 0..20 {
                step(&mut w, 0, -2500);
            }
            if w.players[1].health < his {
                slowed = w.players[1].slowed > 0;
                break;
            }
        }
        card.line(
            "the charged claw slows what it hits",
            format!("slowed {slowed}"),
            slowed,
        );
    }
    {
        // A plant raked by the hand is fed.
        let mut w = with_foe(40.0);
        if let Mechanic::Blood(g) = &mut w.players[0].mechanic {
            g.plants[0] = Some(sim::class::Structure::planted(at(-4.4, 8.0)));
        }
        let before = match w.players[0].mechanic {
            Mechanic::Blood(g) => f(g.plants[0].map_or(Fx::ZERO, |s| s.scale)),
            _ => 0.0,
        };
        step(&mut w, keys::HAND, 0);
        run(&mut w, 20, 0, 0);
        let after = match w.players[0].mechanic {
            Mechanic::Blood(g) => f(g.plants[0].map_or(Fx::ZERO, |s| s.scale)),
            _ => 0.0,
        };
        card.line(
            "raking one of her plants feeds it",
            format!("size {before:.2} -> {after:.2} of a stone"),
            after > before,
        );
    }
    {
        let leap = |held: u32| {
            let mut w = with_foe(40.0);
            let from = w.players[0].pos;
            let mut top = 0.0f32;
            for fr in 0..120u32 {
                let bits = if fr < 2 {
                    Input::SPACE | keys::HAND
                } else if fr < 2 + held {
                    keys::HAND
                } else {
                    0
                };
                step(&mut w, bits, 0);
                top = top.max(f(w.players[0].pos.y));
            }
            (flat(from, w.players[0].pos), top)
        };
        let (tap_far, tap_high) = leap(0);
        let (full_far, full_high) = leap(t::leap_hold() as u32 + 4);
        card.line(
            "space and left held leaps much further than tapped, and low",
            format!("tapped {tap_far:.1} m on, {tap_high:.1} m up; held {full_far:.1} m on, {full_high:.1} m up"),
            full_far > tap_far + 2.0 && full_high < tap_high + 1.0,
        );
        let mut w = with_foe(5.0);
        let his = w.players[1].health;
        for fr in 0..80u32 {
            let bits = if fr < 2 {
                Input::SPACE | keys::HAND
            } else if fr < 2 + t::leap_hold() as u32 {
                keys::HAND
            } else {
                0
            };
            step(&mut w, bits, 0);
        }
        card.line(
            "the claw rides the leap and hits a fighter in her path",
            format!("dealt {}", his - w.players[1].health),
            w.players[1].health < his,
        );
    }

    println!(
        "\n{} promise(s) missed",
        if card.missed == 0 {
            "no".to_string()
        } else {
            card.missed.to_string()
        }
    );
    if card.missed > 0 {
        std::process::exit(1);
    }
}
