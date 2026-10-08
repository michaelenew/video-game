//! Does the animation match the hit volume?
//!
//!     cargo run -p anim --bin audit                      # every Champion move
//!     cargo run -p anim --bin audit -- --class reaver    # another class
//!     cargo run -p anim --bin audit -- champion_sword    # one clip
//!
//! For every attack clip it throws the move in a real simulation, reads the
//! hit volume the simulation has out on each active frame
//! (`sim::state::hitbox`, the one description of an attack's shape), and
//! draws that capsule over the clip's own frames on a contact sheet in
//! `target/anim-audit/`. Beside the picture it prints, per active frame, how
//! far each hand is off the volume's axis, how far the line through the two
//! hands is turned away from that axis, and where along the reach the grip
//! sits -- and the worst of each over the move.
//!
//! The standard is in `docs/design/animation.md`: the hands are *on* the hit
//! line while the volume is out. A hand a hand's width off it is a weapon the
//! renderer has to drag out of the fist to put it where the hit test is
//! (`view::arms` does exactly that for the Champion), which is what reads as
//! rough.

use anim::sheet::{self, Mark};
use sim::state::Action;
use view::clips::{ALL, Clip};
use view::math::V3;
use view::skeleton;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let class = args
        .iter()
        .position(|a| a == "--class")
        .and_then(|i| args.get(i + 1))
        .and_then(|name| {
            sim::class::ALL_CLASSES.iter().copied().find(|c| {
                c.name()
                    .to_lowercase()
                    .replace(' ', "")
                    .contains(&name.to_lowercase())
            })
        })
        .unwrap_or(sim::Class::Champion);
    let class_arg = args.iter().position(|a| a == "--class").map(|i| i + 1);
    let named: Vec<&String> = args
        .iter()
        .enumerate()
        .filter(|(i, a)| !a.starts_with("--") && Some(*i) != class_arg)
        .map(|(_, a)| a)
        .collect();
    let wanted: Vec<Clip> = ALL
        .iter()
        .copied()
        .filter(|c| match c.move_slot() {
            Some((cl, _)) => {
                cl == class && (named.is_empty() || named.iter().any(|n| *n == c.name()))
            }
            None => false,
        })
        .collect();
    if wanted.is_empty() {
        eprintln!("no attack clips for {}", class.name());
        std::process::exit(1);
    }
    let dir = std::path::Path::new("target/anim-audit");
    std::fs::create_dir_all(dir).expect("create audit directory");
    let skeleton = skeleton::skeleton_for(class);
    let mut worst: Vec<(String, f32, f32)> = Vec::new();
    for clip in wanted {
        let (class, slot) = clip.move_slot().unwrap();
        let Some(run) = throw(class, slot, clip) else {
            println!(
                "{:<24} could not be thrown from a standing start (needs a state)",
                clip.name()
            );
            continue;
        };
        let frames: Vec<_> = (0..clip.length() as u32).map(|f| clip.at(f)).collect();
        let (canvas, _) = sheet::contact_sheet_marked(&skeleton, &frames, 0.0, &run.marks);
        let path = dir.join(format!("{}.png", clip.name()));
        canvas.write(&path).expect("write sheet");
        let (startup, active, recovery) = sim::moves::frames(class, slot);
        println!(
            "\n{}  slot {slot}  {startup}+{active}+{recovery} frames  hand {}  -> {}",
            clip.name(),
            sim::moves::get(class, slot).hand.name(),
            path.display()
        );
        println!(
            "  {:>5} | {:>8} {:>8} | {:>7} | {:>7} | {:>6} | {:>7}",
            "frame", "L off", "R off", "axis", "grip", "reach", "in disc"
        );
        let (mut off_max, mut axis_max) = (0.0f32, 0.0f32);
        for (elapsed, m) in &run.rows {
            let pose = clip.at(*elapsed as u32);
            let skin = skeleton::solve(&skeleton, &pose);
            let l = skin.origin[view::hand_joint(true).index()];
            let r = skin.origin[view::hand_joint(false).index()];
            let axis = sub(m.to, m.from);
            let reach = len(axis);
            let off_l = off_line(l, m.from, m.to);
            let off_r = off_line(r, m.from, m.to);
            let hands = sub(r, l);
            let axis_err = if len(hands) > 0.05 && reach > 0.05 {
                let c = dot(hands, axis) / (len(hands) * reach);
                c.abs().clamp(0.0, 1.0).acos().to_degrees()
            } else {
                f32::NAN
            };
            let mid = [
                (l[0] + r[0]) * 0.5,
                (l[1] + r[1]) * 0.5,
                (l[2] + r[2]) * 0.5,
            ];
            let grip = if reach > 0.05 {
                dot(sub(mid, m.from), axis) / (reach * reach)
            } else {
                f32::NAN
            };
            // A disc has no axis: what matters is whether the striking hand
            // is inside it, which this is as metres past its edge (negative
            // is inside).
            let in_disc = if reach <= 0.05 {
                off_l.min(off_r) - m.radius
            } else {
                f32::NAN
            };
            if reach > 0.05 {
                off_max = off_max.max(off_l.min(off_r));
            } else {
                off_max = off_max.max(in_disc.max(0.0));
            }
            if axis_err.is_finite() {
                axis_max = axis_max.max(axis_err);
            }
            println!(
                "  {:>5} | {:>6.0}cm {:>6.0}cm | {:>6.1}° | {:>7.2} | {:>5.2}m | {:>6.2}m",
                elapsed,
                off_l * 100.0,
                off_r * 100.0,
                axis_err,
                grip,
                reach,
                in_disc
            );
        }
        println!(
            "  worst: nearer hand {:.0} cm off the axis (or past a disc's edge), hands turned {:.0}° from it",
            off_max * 100.0,
            axis_max
        );
        worst.push((clip.name().to_string(), off_max, axis_max));
    }
    println!("\n{:<26} {:>8} {:>8}", "summary", "off cm", "axis °");
    worst.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (name, off, axis) in worst {
        println!("{name:<26} {:>8.0} {:>8.0}", off * 100.0, axis);
    }
}

/// One move thrown by a standing fighter, as the simulation played it.
struct Run {
    /// The hit volume on each clip frame, in the body's own space.
    marks: Vec<Vec<Mark>>,
    /// The active frames, with the volume on each.
    rows: Vec<(u16, Mark)>,
}

fn throw(class: sim::Class, slot: u8, clip: Clip) -> Option<Run> {
    let mut w = sim::World::with_classes([class, sim::Class::Bulwark]);
    // The other fighter well out of reach, so nothing lands and nothing
    // reacts; this is geometry, not a fight.
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    // A chain link the Champion can only throw mid-chain is given the chain.
    if let sim::class::Mechanic::Forms {
        chain, chain_left, ..
    } = &mut w.players[0].mechanic
    {
        if let Some(link) = sim::moves::champion::link_of(slot) {
            *chain = link;
            *chain_left = 400;
        }
    }
    let look = sim::Input::looking_at(0, 0, 0);
    let idle = sim::Input::default();
    for _ in 0..2 {
        w.advance([look, idle]);
    }
    w.press(0, slot, look);
    let (startup, active, recovery) = sim::moves::frames(class, slot);
    let length = clip.length().max(1) as usize;
    let mut marks = vec![Vec::new(); length];
    let mut rows = Vec::new();
    let mut seen = false;
    for _ in 0..600 {
        let p = w.players[0];
        let elapsed = match p.action {
            Action::Channel { kind, .. } if kind == slot => 0,
            Action::Startup { kind, left } if kind == slot => startup.saturating_sub(left),
            Action::Active { kind, left } if kind == slot => startup + active.saturating_sub(left),
            Action::Recovery { kind, left } if kind == slot => {
                startup + active + recovery.saturating_sub(left)
            }
            _ => break,
        };
        seen = true;
        if let Some(h) = sim::state::hitbox(&p) {
            let m = Mark {
                from: local(h.from, p.pos, p.facing),
                to: local(h.to, p.pos, p.facing),
                radius: h.radius.to_f32_for_render(),
            };
            marks[(elapsed as usize).min(length - 1)].push(m);
            rows.push((elapsed, m));
        }
        w.advance([look, idle]);
    }
    seen.then_some(Run { marks, rows })
}

/// A point in the arena, in the body's own space: `view::into_world` run
/// backwards.
fn local(v: sim::V3, pos: sim::V3, facing: sim::V3) -> V3 {
    let d = [
        (v.x.sub(pos.x)).to_f32_for_render(),
        (v.y.sub(pos.y)).to_f32_for_render(),
        (v.z.sub(pos.z)).to_f32_for_render(),
    ];
    let yaw = facing
        .x
        .to_f32_for_render()
        .atan2(facing.z.to_f32_for_render());
    let (s, c) = (-yaw).sin_cos();
    [d[0] * c + d[2] * s, d[1], -d[0] * s + d[2] * c]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}

/// How far a point is from the segment `a..b`.
fn off_line(p: V3, a: V3, b: V3) -> f32 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 > 1e-6 {
        (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    len(sub(
        p,
        [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t],
    ))
}
