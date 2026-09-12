//! Look at a clip without running the game.
//!
//!     cargo run -p anim --bin preview -- walk_forward
//!     cargo run -p anim --bin preview -- --file dodge
//!     cargo run -p anim --bin preview -- --all
//!     cargo run -p anim --bin preview -- --match
//!
//! Writes a PNG contact sheet per clip into `target/anim-preview/` and prints
//! where. Each sheet has three panels: the clip from the side, the clip from
//! the front, and every frame overlaid so the arcs of the hands and feet are
//! visible as arcs.
//!
//! The side view is the important one -- it is what an opponent reads across
//! the arena -- and the overlay is where foot skate shows up, as a planted
//! ankle whose dots drift instead of piling on one spot.

use anim::sheet;
use view::clips::{ALL, Clip};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--match") {
        played_match(&args);
        return;
    }
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

    let wanted: Vec<Clip> = if args.iter().any(|a| a == "--all") || args.is_empty() {
        ALL.to_vec()
    } else if let Some(i) = args.iter().position(|a| a == "--file") {
        let file = args.get(i + 1).cloned().unwrap_or_default();
        ALL.iter().copied().filter(|c| c.file() == file).collect()
    } else {
        args.iter()
            .filter(|a| !a.starts_with("--"))
            .filter_map(|name| match Clip::by_name(name) {
                Some(c) => Some(c),
                None => {
                    eprintln!("no clip called {name}");
                    None
                }
            })
            .collect()
    };

    if wanted.is_empty() {
        eprintln!("nothing to draw. Names:");
        for c in ALL {
            eprintln!("  {}", c.name());
        }
        std::process::exit(1);
    }

    // Bake live rather than reading the checked-in table, so a preview always
    // shows what the recipes currently say -- waiting on a separate bake step
    // is exactly the kind of friction that stops people looking.
    let (baked, _) = anim::bake_all();
    let skeleton = view::skeleton::skeleton_for(class);

    let dir = std::path::Path::new("target/anim-preview");
    std::fs::create_dir_all(dir).expect("create preview directory");

    let report = args.iter().any(|a| a == "--feet");

    for clip in wanted {
        let Some(b) = baked.iter().find(|b| b.clip == clip) else {
            continue;
        };
        if report {
            let stride = if clip.name().starts_with("run") {
                view::play::RUN_STRIDE
            } else if clip.name() == "crouch_walk" {
                view::play::CROUCH_STRIDE
            } else {
                view::play::WALK_STRIDE
            };
            let dir = match clip.name().rsplit('_').next().unwrap_or("") {
                "back" => [0.0, 0.0, -1.0],
                "left" => [-1.0, 0.0, 0.0],
                "right" => [1.0, 0.0, 0.0],
                _ => [0.0, 0.0, 1.0],
            };
            feet_report(b, stride, dir);
        }
        let (canvas, shown) = sheet::contact_sheet(&skeleton, &b.frames);
        let path = dir.join(format!("{}.png", clip.name()));
        canvas.write(&path).expect("write sheet");
        println!(
            "{:<24} {:>3} frames on {:<14} -> {}",
            clip.name(),
            b.frames.len(),
            class.name(),
            path.display()
        );
        if shown.len() < b.frames.len() {
            println!("    showing frames {shown:?}");
        }
    }
}

/// Print what each foot is doing, frame by frame.
///
/// `heel` and `toe` are the two points on the sole, in metres above the floor.
/// `world` is where whichever of them is lower sits in the arena once the
/// body's own travel is added in, and `slide` is how far that point moved since
/// the frame before. **While a foot is down, slide should be near zero.** That
/// is the whole definition of not skating.
pub fn feet_report(baked: &anim::Baked, stride: f32, dir: [f32; 3]) {
    let skeleton = view::pose::reference();
    let n = baked.frames.len();
    println!(
        "\n{}  stride {stride:.2} m over {n} frames",
        baked.clip.name()
    );
    println!(
        "{:>5} | {:>6} {:>6} {:>7} {:>6} | {:>6} {:>6} {:>7} {:>6}",
        "frame", "L heel", "L toe", "L world", "slide", "R heel", "R toe", "R world", "slide"
    );
    let contact = |i: usize, left: bool| {
        let pose = &baked.frames[i % n];
        let (heel, toe) = pose.foot_contact(skeleton, left);
        let on_toe = toe[1] < heel[1];
        let p = if on_toe { toe } else { heel };
        let along = stride * (i as f32) / n as f32;
        (
            heel[1],
            toe[1],
            [p[0] + dir[0] * along, p[1], p[2] + dir[2] * along],
            on_toe,
        )
    };
    for i in 0..n {
        let mut cells = Vec::new();
        for left in [true, false] {
            let (heel, toe, world, on_toe) = contact(i, left);
            let (_, _, next, next_toe) = contact(i + 1, left);
            let slide = if on_toe == next_toe {
                ((next[0] - world[0]).powi(2) + (next[2] - world[2]).powi(2)).sqrt()
            } else {
                f32::NAN
            };
            let down = heel.min(toe) < 0.035;
            cells.push(format!(
                "{heel:>6.3} {toe:>6.3} {:>7.3} {}",
                if dir[0] != 0.0 { world[0] } else { world[2] },
                if down {
                    format!("{slide:>6.3}")
                } else {
                    "     -".to_string()
                }
            ));
        }
        println!("{i:>5} | {} | {}", cells[0], cells[1]);
    }
}

/// Draw a scripted match through the *whole* animation system.
///
/// The per-clip sheets show what a recipe bakes to. This shows what a player
/// actually sees: clip selection, the blends between the directional walks, the
/// cross-fades in and out of attacks, and every transition between them. It is
/// the only way to catch a clip that is fine on its own and wrong the moment it
/// is entered from something else.
fn played_match(args: &[String]) {
    use sim::{Input, World};

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

    let mut w = World::with_classes([class, sim::Class::Bulwark]);
    let mut fade = view::play::Crossfade::default();
    let mut poses = Vec::new();
    let mut labels = Vec::new();

    for i in 0..240u32 {
        let phase = i % 120;
        let bits = match phase {
            0..=22 => Input::W,
            23..=34 => Input::W | Input::A,
            35..=42 => Input::SPACE,
            43..=50 => Input::SHIFT | Input::D,
            51..=62 => Input::LEFT,
            63..=76 => Input::SHIFT | Input::LEFT,
            77..=88 => Input::RIGHT,
            89..=98 => Input::CROUCH,
            99..=106 => Input::S,
            _ => 0,
        };
        let aim = ((i * 420) % 65536) as u16;
        let prev = w.clone();
        w.advance([Input::aimed(bits, aim), Input::new(Input::RIGHT)]);
        let frame = view::interpolate(&prev, &w, 1.0);
        let input =
            view::play::PoseInput::of(&frame.players[0], w.players[0].class, frame.round_left);
        poses.push(fade.pose(input, 1.0 / 60.0));
        labels.push(format!("{:?}", w.players[0].action));
    }

    let skeleton = view::skeleton::skeleton_for(class);
    let (canvas, shown) = sheet::contact_sheet(&skeleton, &poses);
    let dir = std::path::Path::new("target/anim-preview");
    std::fs::create_dir_all(dir).expect("create preview directory");
    let path = dir.join("match.png");
    canvas.write(&path).expect("write sheet");
    println!(
        "{} frames of a scripted match on {} -> {}",
        poses.len(),
        class.name(),
        path.display()
    );
    for f in shown {
        println!("  {f:>4}  {}", labels[f]);
    }
}
