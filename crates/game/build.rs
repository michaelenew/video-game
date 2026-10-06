//! Stamp the build with the commit it was built from.
//!
//! Two players are only matched if they run the same build (`src/online.rs`,
//! `terms`), and a desktop and the published page have to be able to agree on
//! what that means: the commit hash, from either side. The page's build script
//! sets `ARENA_BUILD` itself; anything else asks git, and a machine without a
//! checkout or without git says `dev`.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=ARENA_BUILD");
    let stamp = std::env::var("ARENA_BUILD").ok().unwrap_or_else(|| {
        Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "dev".into())
    });
    // A new commit is a new build. HEAD names the branch; the branch's ref
    // file, or packed-refs, holds the commit.
    if let Ok(git) = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        && let Ok(dir) = String::from_utf8(git.stdout)
    {
        let dir = dir.trim();
        println!("cargo:rerun-if-changed={dir}/HEAD");
        println!("cargo:rerun-if-changed={dir}/packed-refs");
        if let Ok(head) = std::fs::read_to_string(format!("{dir}/HEAD"))
            && let Some(branch) = head.trim().strip_prefix("ref: ")
        {
            println!("cargo:rerun-if-changed={dir}/{branch}");
        }
    }
    println!("cargo:rustc-env=ARENA_BUILD={stamp}");
}
