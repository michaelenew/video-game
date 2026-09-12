//! Bake the animation tables.
//!
//!     cargo run -p anim --bin bake
//!
//! Writes `crates/view/src/baked.rs`. **Edit the recipes in
//! `crates/anim/src/clips/`, or in the animation hub (F9), not the generated
//! file.**
//!
//! Each recipe is a handful of poses, an easing per gap, and a looseness
//! setting. Everything between the keys -- the lag, the overshoot, the settle
//! -- comes from the solver. That is the point of the factory: the parts that
//! make motion read as alive are the parts that are miserable to key by hand.

fn main() {
    let (baked, missing) = anim::bake_all();
    let duplicated = anim::clips::duplicated();

    let path = "crates/view/src/baked.rs";
    std::fs::write(path, anim::bake::emit(&baked)).expect("write baked table");

    let mut by_family: Vec<(&str, usize, usize)> = Vec::new();
    for b in &baked {
        let family = b.clip.family().name();
        match by_family.iter_mut().find(|(f, _, _)| *f == family) {
            Some(entry) => {
                entry.1 += 1;
                entry.2 += b.frames.len();
            }
            None => by_family.push((family, 1, b.frames.len())),
        }
    }
    for (family, clips, frames) in &by_family {
        println!("{family:<18} {clips:>3} clips  {frames:>5} frames");
    }

    let total: usize = baked.iter().map(|b| b.frames.len()).sum();
    println!("\n{path}  ({} clips, {total} frames)", baked.len());

    if !duplicated.is_empty() {
        println!("\nauthored twice -- one of each pair is being thrown away:");
        for c in duplicated {
            println!("  {} ({})", c.name(), c.file());
        }
    }
    if !missing.is_empty() {
        println!(
            "\n{} clips have no recipe and bake as a held rest pose:",
            missing.len()
        );
        for c in missing {
            println!("  {:<24} {}.rs", c.name(), c.file());
        }
    }
}
