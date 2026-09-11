//! Committing a tuning session.
//!
//! The palette changes numbers in memory; bake is what makes them survive the
//! process. It writes `crates/sim/src/tuned.rs`, formats it, and commits and
//! pushes on whatever branch is checked out — so an afternoon of tuning ends as
//! a reviewable diff rather than as something you have to remember and retype.
//!
//! Generated Rust rather than a data file, for the same reasons the baked
//! animations are: no loader, no asset path, no runtime parsing, and a diff that
//! shows exactly which numbers moved. Every entry carries its identifier and its
//! value as a comment, which is what makes the diff readable by someone who was
//! not in the session.

use std::path::{Path, PathBuf};
use std::process::Command;

/// What happened, for showing in the palette.
#[derive(Clone, Debug)]
pub enum Outcome {
    Ok(String),
    Failed(String),
}

/// The repository root, found from where this crate was compiled.
///
/// The Oven is a development tool run from a source checkout, so this is sound
/// in the only situation it is ever used in.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new(args[0])
        .args(&args[1..])
        .current_dir(root)
        .output()
        .map_err(|e| format!("{} would not start: {e}", args[0]))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { stdout } else { err })
    }
}

/// Write the live values back to the repository and push them.
///
/// Deliberately not silent about any step: a bake that wrote the file but could
/// not push is a *different* outcome from one that worked, and a tool that
/// blurred the two would cost someone an afternoon of tuning.
pub fn bake(message: &str) -> Outcome {
    let root = repo_root();
    let rel = "crates/sim/src/tuned.rs";
    let path = root.join(rel);

    if let Err(e) = std::fs::write(&path, sim::oven::emit()) {
        return Outcome::Failed(format!("could not write {rel}: {e}"));
    }

    // Formatting is best effort. A correctly-valued file that rustfmt could not
    // reach is still worth committing.
    let _ = run(&root, &["rustfmt", "--edition", "2024", rel]);

    let branch = match run(&root, &["git", "rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(b) => b,
        Err(e) => return Outcome::Failed(format!("no branch: {e}")),
    };

    if let Err(e) = run(&root, &["git", "add", rel]) {
        return Outcome::Failed(format!("git add: {e}"));
    }

    // Nothing staged means nothing changed. Saying so beats an empty commit.
    if run(&root, &["git", "diff", "--cached", "--quiet", "--", rel]).is_ok() {
        return Outcome::Ok("already baked — no changes".into());
    }

    let body = format!("Bake tuning: {message}\n\nBaked from the Oven.");
    if let Err(e) = run(&root, &["git", "commit", "-m", &body, "--only", rel]) {
        return Outcome::Failed(format!("git commit: {e}"));
    }

    match run(&root, &["git", "push", "origin", &branch]) {
        Ok(_) => Outcome::Ok(format!("baked and pushed to {branch}")),
        Err(e) => Outcome::Failed(format!("committed on {branch} but push failed: {e}")),
    }
}
