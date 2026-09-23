//! The goad: the Dual mage's two bars, driven by named input scripts.
//!
//!     cargo run -p sim --bin goad                  every script, summarised
//!     cargo run -p sim --bin goad alternate        one script, frame by frame
//!
//! Nobody can feel a number, but the relationships the design asks for -- one
//! cast from level stays inside the band and two do not, the drift can never
//! fill a bar, the second tier falls below the first inside a real exchange --
//! are all things this prints in a second. It is how the mechanic is
//! self-evaluated between the human checkpoints in
//! `docs/design/plans/dual-mage-v2.md`, and the first thing a reviewer runs.
//!
//! Every script runs twice: in an **empty arena**, with the other fighter out
//! of reach, and against a **dummy** standing where the autos land, idle, the
//! way the game's own training dummy stands. The bars move on the press either
//! way; what the dummy adds is the hits, and with them the refund while she is
//! ascending and the knockback that walks the dummy out of range.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers all of it.

use sim::class::{Class, Force, Mechanic};
use sim::dual::{self, Tier};
use sim::moves::dual as d;
use sim::tuning as t;
use sim::{Fx, Input, World};

/// How long a script runs: half a versus round.
const RUN_FRAMES: u32 = 1800;

/// One frame in every this many is printed in the frame-by-frame view, on top
/// of every frame a button is pressed on.
const EVERY: u32 = 6;

const SCRIPTS: &[&str] = &["alternate", "one-sided", "finisher", "idle", "ascend"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let chosen: Vec<&str> = if args.is_empty() {
        SCRIPTS.to_vec()
    } else {
        args.iter().map(|a| a.as_str()).collect()
    };
    let verbose = args.len() == 1;

    println!(
        "bars 0..{}  |  band {}  |  goads {}/{}/{}  |  tiers {}/{}/{}  |  calm {}/s  |  drift {}/s per unit, cap {}/s  |  burn {}/s per unit, cap {}/s",
        t::meter_max(),
        t::meter_band(),
        t::meter_auto_push(),
        t::meter_cast_push(),
        t::meter_finisher_push(),
        t::tier_blink(),
        t::tier_jump(),
        t::tier_wings(),
        tenths(t::meter_calm()),
        tenths(t::drift_gain()),
        tenths(t::drift_cap()),
        tenths(t::burn_gain()),
        tenths(t::burn_cap()),
    );
    println!(
        "one exchange = a Judgement's startup to the end of its recovery, twice = {}f\n",
        exchange()
    );

    for name in chosen {
        let Some(script) = Script::named(name) else {
            eprintln!("no script called {name:?}; there are {SCRIPTS:?}");
            std::process::exit(2);
        };
        for dummy in [false, true] {
            let report = run(script, dummy, verbose);
            report.print(name, dummy);
        }
    }
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Script {
    /// Dark auto, dark Lance, light auto, light Sweep, repeated. The climb.
    Alternate,
    /// One dark auto, then dark casts only. The runaway.
    OneSided,
    /// Balanced, then one bar walked to the band's edge, then one Judgement.
    Finisher,
    /// Both bars at the second tier, then nothing. The calm.
    Idle,
    /// The fastest sequence the greedy chooser below finds to both bars full.
    Ascend,
}

impl Script {
    fn named(name: &str) -> Option<Script> {
        Some(match name {
            "alternate" => Script::Alternate,
            "one-sided" | "one_sided" | "onesided" => Script::OneSided,
            "finisher" => Script::Finisher,
            "idle" => Script::Idle,
            "ascend" => Script::Ascend,
            _ => return None,
        })
    }

    /// What the bars are when the script starts. `None` leaves them empty.
    fn opening(self) -> Option<Mechanic> {
        let bars = |dark: i32, light: i32, colour: Force| Mechanic::Meter {
            dark: Fx::from_int(dark),
            light: Fx::from_int(light),
            colour,
            ascending: 0,
            jumped: false,
            landed: 0,
            singe: Fx::ZERO,
        };
        match self {
            // Level at the first tier, with the carried bar at the far edge
            // of the band: the last place a finisher is a question rather than
            // a certainty.
            Script::Finisher => Some(bars(
                t::tier_blink() + t::meter_band(),
                t::tier_blink(),
                Force::Dark,
            )),
            Script::Idle => Some(bars(t::tier_jump(), t::tier_jump(), Force::Dark)),
            _ => None,
        }
    }

    /// The button to press on a frame she is free to act, given how many
    /// presses have gone before and where the bars stand. `0` presses nothing.
    fn press(self, count: u32, w: &World) -> u16 {
        match self {
            Script::Alternate => {
                [Input::LEFT, Input::MIDDLE, Input::RIGHT, Input::MECHANIC][(count % 4) as usize]
            }
            Script::OneSided => {
                if count == 0 {
                    Input::LEFT
                } else {
                    [Input::MIDDLE, Input::MECHANIC, Input::SPECIAL][((count - 1) % 3) as usize]
                }
            }
            Script::Finisher => {
                if count == 0 {
                    Input::SPECIAL
                } else {
                    0
                }
            }
            Script::Idle => 0,
            Script::Ascend => greedy(w),
        }
    }
}

/// Feed the **lower** bar: the biggest goad on it that keeps her inside the
/// band and inside the top, and the other hand's auto whenever she is
/// carrying the higher one -- an auto is the only thing that changes which
/// bar the casts feed.
///
/// A heuristic rather than a search, and good enough for the question the
/// script asks -- whether both bars can be goaded to the top at all, and how
/// long an honest attempt takes. A search would find something a few frames
/// faster and prove nothing more.
fn greedy(w: &World) -> u16 {
    let p = &w.players[0];
    let Some((dark, light)) = dual::bars(p) else {
        return 0;
    };
    let colour = sim::state::carrying(p);
    let (mine, other) = match colour {
        Force::Dark => (dark, light),
        Force::Light => (light, dark),
    };
    if mine.raw() > other.raw() {
        // Carrying the higher: switch hands.
        return match colour.other() {
            Force::Dark => Input::LEFT,
            Force::Light => Input::RIGHT,
        };
    }
    let band = Fx::from_int(t::meter_band());
    let top = Fx::from_int(t::meter_max());
    let fits = |push: i32| {
        let after = mine.add(Fx::from_int(push));
        after.sub(other).raw() <= band.raw() && after.raw() <= top.add(Fx::from_int(push)).raw()
    };
    if fits(t::meter_finisher_push()) && p.repeat_lock[d::JUDGEMENT as usize] == 0 {
        Input::SPECIAL
    } else if fits(t::meter_cast_push()) {
        // Sweep and Lance alternate, so neither runs into its own lockout.
        let sweep_locked = p.repeat_lock[d::SWEEP as usize] > 0;
        let lance = d::lance_for(colour) as usize;
        if !sweep_locked || p.repeat_lock[lance] > 0 {
            Input::MECHANIC
        } else {
            Input::MIDDLE
        }
    } else {
        match colour {
            Force::Dark => Input::LEFT,
            Force::Light => Input::RIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// Running one
// ---------------------------------------------------------------------------

struct Report {
    to_blink: Option<u32>,
    to_jump: Option<u32>,
    to_wings: Option<u32>,
    left_band: Option<u32>,
    /// Frames from leaving the band to the lower bar reaching zero.
    to_empty: Option<u32>,
    back_in_band: Option<u32>,
    /// `idle` only: when the lower bar fell below the first tier.
    lost_blink: Option<u32>,
    health_lost: i32,
    /// What she took off the dummy over the whole run, and the most any one
    /// press took: the two numbers the damage benchmarks are stated in.
    dealt: i32,
    biggest_hit: i32,
    presses: u32,
    peak_gap: Fx,
    end: (Fx, Fx),
}

impl Report {
    fn print(&self, name: &str, dummy: bool) {
        let where_ = if dummy {
            "dummy in reach"
        } else {
            "empty arena"
        };
        println!(
            "== {name}  ({where_}, {RUN_FRAMES}f, {} presses)",
            self.presses
        );
        let when = |f: Option<u32>| match f {
            Some(f) => format!("{f}f ({} exchanges)", hundredths_ratio(f, exchange())),
            None => "never".to_string(),
        };
        println!(
            "  tiers: blink {}  |  second jump {}  |  wings {}",
            when(self.to_blink),
            when(self.to_jump),
            when(self.to_wings)
        );
        let empty = match (self.left_band, self.to_empty) {
            (Some(_), Some(f)) => format!("{f}f after leaving"),
            (Some(_), None) => "never".to_string(),
            (None, _) => "n/a".to_string(),
        };
        println!(
            "  band: left {}  |  lower bar empty {}  |  back inside {}  |  widest gap {}",
            when(self.left_band),
            empty,
            when(self.back_in_band),
            tenths(self.peak_gap)
        );
        if let Some(f) = self.lost_blink {
            println!("  fell below the first tier at {}", when(Some(f)));
        }
        println!(
            "  health lost {} of {}  |  ends dark {} / light {}",
            self.health_lost,
            t::health_of(Class::DualMage),
            tenths(self.end.0),
            tenths(self.end.1)
        );
        if dummy {
            println!(
                "  dealt {} to the dummy ({}% of a health bar), {} a press, biggest single frame {}\n",
                self.dealt,
                self.dealt * 100 / sim::state::max_health().max(1),
                self.dealt / self.presses.max(1) as i32,
                self.biggest_hit
            );
        } else {
            println!();
        }
    }
}

fn run(script: Script, dummy: bool, verbose: bool) -> Report {
    let mut w = World::with_classes([Class::DualMage, Class::Bulwark]);
    if let Some(opening) = script.opening() {
        w.players[0].mechanic = opening;
    }
    // Where the autos land: just inside the wing's reach, dead ahead.
    let reach = sim::moves::get(Class::DualMage, d::DARK_AUTO).reach;
    let stand = w.players[0]
        .pos
        .add(w.players[0].facing.scale(reach.sub(t::body_radius())));
    if dummy {
        w.players[1].pos = stand;
    } else {
        // Out of everything's reach, in the corner behind her. Where the two
        // spawn is inside a Judgement's range, and a Judgement at the top of
        // the curve thrown four times ends the round -- which resets the bars
        // and the health and makes the numbers lie about a fight that never
        // happened.
        let corner = sim::arena::ARENA_HALF.sub(t::body_radius().add(t::body_radius()));
        w.players[1].pos = sim::V3::new(corner.neg(), Fx::ZERO, corner.neg());
    }
    let start_health = w.players[0].health;
    let band = Fx::from_int(t::meter_band());
    let mut r = Report {
        to_blink: None,
        to_jump: None,
        to_wings: None,
        left_band: None,
        to_empty: None,
        back_in_band: None,
        lost_blink: None,
        health_lost: 0,
        dealt: 0,
        biggest_hit: 0,
        presses: 0,
        peak_gap: Fx::ZERO,
        end: (Fx::ZERO, Fx::ZERO),
    };
    if verbose {
        println!(
            "-- {script:?}, {}",
            if dummy {
                "dummy in reach"
            } else {
                "empty arena"
            }
        );
        println!("  frame   dark  light    gap  health  tier          doing");
    }
    for frame in 0..RUN_FRAMES {
        let p = &w.players[0];
        let bits = if p.action.actionable() && p.grounded {
            script.press(r.presses, &w)
        } else {
            0
        };
        if bits != 0 {
            r.presses += 1;
        }
        w.advance([Input::aimed(bits, 0), Input::new(0)]);
        // A training dummy that does not die: a round that ended would reset
        // the bars and the clock mid-script. What it lost this frame is what
        // she dealt.
        let took = w.players[1].full_health() - w.players[1].health;
        r.dealt += took;
        r.biggest_hit = r.biggest_hit.max(took);
        w.players[1].health = w.players[1].full_health();
        // And one that does not move: the benchmarks are stated against a
        // target that stands in everything, and the light hand's shove would
        // otherwise walk it out of range on the second press.
        if dummy {
            w.players[1].pos = stand;
        }

        let p = &w.players[0];
        let (dark, light) = dual::bars(p).unwrap_or((Fx::ZERO, Fx::ZERO));
        let lower = dark.min(light);
        let gap = dark.sub(light).abs();
        let tier = dual::tier(p);
        let f = frame + 1;
        if r.to_blink.is_none() && tier >= Tier::Blink {
            r.to_blink = Some(f);
        }
        if r.to_jump.is_none() && tier >= Tier::Jump {
            r.to_jump = Some(f);
        }
        if r.to_wings.is_none() && tier == Tier::Wings {
            r.to_wings = Some(f);
        }
        if gap.raw() > r.peak_gap.raw() {
            r.peak_gap = gap;
        }
        match r.left_band {
            None if gap.raw() > band.raw() => r.left_band = Some(f),
            Some(left) => {
                if r.to_empty.is_none() && r.back_in_band.is_none() && lower.raw() == 0 {
                    r.to_empty = Some(f - left);
                }
                if r.back_in_band.is_none() && gap.raw() <= band.raw() {
                    r.back_in_band = Some(f);
                }
            }
            None => {}
        }
        if script == Script::Idle && r.lost_blink.is_none() && tier < Tier::Blink {
            r.lost_blink = Some(f);
        }
        if verbose && (bits != 0 || f % EVERY == 0) {
            println!(
                "  {f:>5} {:>6} {:>6} {:>6} {:>7}  {:<12}  {}{}",
                tenths(dark),
                tenths(light),
                tenths(gap),
                p.health,
                tier.name(),
                doing(p),
                if bits != 0 {
                    format!("   <- {}", button(bits))
                } else {
                    String::new()
                }
            );
        }
    }
    let p = &w.players[0];
    r.health_lost = start_health - p.health;
    r.end = dual::bars(p).unwrap_or((Fx::ZERO, Fx::ZERO));
    r
}

/// The frames a real exchange takes: a Judgement from its first startup frame
/// to the end of its recovery, twice. The unit the plan measures the calm and
/// the climb against.
fn exchange() -> u32 {
    let m = sim::moves::get(Class::DualMage, d::JUDGEMENT);
    2 * (m.startup as u32 + m.active as u32 + m.recovery as u32)
}

fn doing(p: &sim::state::Player) -> String {
    match p.action.attack_kind() {
        Some(kind) => sim::moves::get(p.class, kind).name.to_string(),
        None if p.action.stunned() => "staggered".to_string(),
        None => String::new(),
    }
}

fn button(bits: u16) -> &'static str {
    match bits {
        Input::LEFT => "L",
        Input::RIGHT => "R",
        Input::MIDDLE => "M",
        Input::SPECIAL => "Q",
        Input::MECHANIC => "E",
        _ => "?",
    }
}

/// One decimal place, without touching floating point.
fn tenths(v: Fx) -> String {
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    format!("{}.{}", t / 10, (t % 10).abs())
}

/// `num / den` to two decimal places, in integers.
fn hundredths_ratio(num: u32, den: u32) -> String {
    let h = (num as u64 * 100 + den as u64 / 2) / den.max(1) as u64;
    format!("{}.{:02}", h / 100, h % 100)
}
