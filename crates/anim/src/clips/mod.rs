//! The recipes: every animation in the game, as poses and timing.
//!
//! One file per family, and **one owner per file**. That is deliberate: it is
//! what lets a dozen unrelated animations be worked on at once without anybody
//! colliding, and it is why the clip *names* are fixed in `view::clips` rather
//! than here. The contract is set first; the content fills in.
//!
//! A clip with no recipe bakes as a held rest pose and is listed by name when
//! the bake runs, so nothing silently ships as a T-pose.

use crate::bake::Recipe;
use view::clips::{ALL, Clip};

pub mod air;
pub mod blood;
pub mod bulwark;
pub mod champion;
pub mod defence;
pub mod dodge;
pub mod dual;
pub mod elementalist;
pub mod locomotion;
pub mod reactions;
pub mod reaver;
pub mod turns;

/// Every recipe in the game, in no particular order.
pub fn all() -> Vec<Recipe> {
    let mut out = Vec::new();
    out.extend(locomotion::clips());
    out.extend(turns::clips());
    out.extend(air::clips());
    out.extend(dodge::clips());
    out.extend(defence::clips());
    out.extend(reactions::clips());
    out.extend(bulwark::clips());
    out.extend(champion::clips());
    out.extend(reaver::clips());
    out.extend(elementalist::clips());
    out.extend(blood::clips());
    out.extend(dual::clips());
    out
}

/// Recipes for one file, so the hub can regenerate exactly that file.
pub fn in_file(file: &str) -> Vec<Recipe> {
    all()
        .into_iter()
        .filter(|r| r.clip.file() == file)
        .collect()
}

/// Clips the contract names that nobody has authored yet.
pub fn missing() -> Vec<Clip> {
    let have: Vec<Clip> = all().iter().map(|r| r.clip).collect();
    ALL.iter().copied().filter(|c| !have.contains(c)).collect()
}

/// Clips authored more than once, which would otherwise silently take whichever
/// the iteration order happened to reach last.
pub fn duplicated() -> Vec<Clip> {
    let mut seen: Vec<Clip> = Vec::new();
    let mut twice = Vec::new();
    for r in all() {
        if seen.contains(&r.clip) {
            twice.push(r.clip);
        }
        seen.push(r.clip);
    }
    twice
}
