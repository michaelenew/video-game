//! The browser build and the desktop build are one program.
//!
//! `game` compiles for a window and for a canvas, and the difference between
//! the two is five facts about the world outside the fight: where a run's
//! settings come from, where a player's settings are kept, whether there is a
//! peer, whether there is a checkout to write to, and where a panic can be
//! read. **All five live in the four files in `ALLOWED` below. Nothing else in the
//! crate may reach the host.**
//!
//! The rule needs a test because breaking it is silent in the direction that
//! matters. A `std::fs::write` added to a system compiles perfectly well for
//! wasm32 — `std` is there, the call is there, it returns an error at runtime
//! and the feature quietly does nothing on the web while working fine on the
//! desk. Nobody notices until somebody who was sent a link says the thing did
//! not save.
//!
//! If you are here because this test failed: the answer is almost always to add
//! a function to `platform.rs` with two bodies, one per platform, and call that.
//! See [`docs/design/web.md`](../../../docs/design/web.md).

use std::path::Path;

/// Ways of reaching outside the process, and the browser's answer to each.
const RESERVED: &[(&str, &str)] = &[
    (
        "std::env::",
        "there is no command line and no environment in a page",
    ),
    ("std::fs::", "there is no filesystem in a page"),
    (
        "std::process::",
        "there is nothing to shell out to in a page",
    ),
    ("std::net::", "a page cannot open a socket"),
    (
        "std::thread::",
        "spawning a thread on wasm32 without shared memory does not fail, it panics",
    ),
    (
        "web_sys::",
        "the browser's own API belongs on the browser's side of the seam",
    ),
    ("wasm_bindgen::", "likewise"),
];

/// Files allowed to reach the host, and why.
///
/// An entry without a reason is just a way to silence the test.
const ALLOWED: &[(&str, &str)] = &[
    (
        "platform.rs",
        "The seam itself: the query string against argv, local storage against \
         a settings file, and a panic hook that has somewhere to print. This is \
         the file every other one is being pointed at.",
    ),
    (
        "online.rs",
        "The peer. UDP on the desktop, and on wasm a `Driver` with one variant, \
         so the browser build does not depend on `net` at all.",
    ),
    (
        "bake.rs",
        "Committing a tuning session writes `tuned.rs` and pushes it, which \
         needs a checkout and a git. The browser half of the file says so in a \
         sentence instead, and the palette cannot tell which it got.",
    ),
    (
        "hub.rs",
        "Saving a clip writes its recipe file and re-runs the animation bake in \
         a subprocess. Same shape as bake: the browser half says so, and \
         everything up to saving -- editing, solving, watching it on a fighter \
         -- works on both.",
    ),
];

fn game_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read game/src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if name.ends_with(".rs") {
                out.push((name, std::fs::read_to_string(&path).expect("read source")));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn only_the_platform_seam_reaches_the_host() {
    let allowed: Vec<&str> = ALLOWED.iter().map(|(f, _)| *f).collect();
    let mut loose = Vec::new();
    for (file, source) in game_sources() {
        if allowed.contains(&file.as_str()) {
            continue;
        }
        for (i, line) in source.lines().enumerate() {
            // Prose is allowed to name them; that is how the rule gets read.
            let code = line.split("//").next().unwrap_or(line);
            for (needle, why) in RESERVED {
                if code.contains(needle) {
                    loose.push(format!(
                        "  crates/game/src/{file}:{}  {needle}  ({why})",
                        i + 1
                    ));
                }
            }
        }
    }
    assert!(
        loose.is_empty(),
        "these reach the host from outside the platform seam, so the browser \
         build will compile and then quietly not do them.\nPut the two answers \
         in `platform.rs`, one per platform, and call that. If the file \
         genuinely owns a host-only tool, add it to ALLOWED in this test with \
         the reason:\n{}",
        loose.join("\n")
    );
}

#[test]
fn every_exemption_gives_a_reason() {
    for (file, reason) in ALLOWED {
        assert!(
            reason.len() > 40,
            "{file} is allowed without a real reason -- an exemption nobody had \
             to justify is just a way to silence the test"
        );
    }
}

#[test]
fn every_exemption_has_an_opinion_about_the_browser() {
    // Being allowed to reach the host means having decided what the browser
    // does instead. A file on this list with no `target_arch` in it has not
    // decided anything -- it has been forgotten, and forgetting is the failure
    // this whole test exists to catch.
    let sources = game_sources();
    for (file, _) in ALLOWED {
        let (_, source) = sources
            .iter()
            .find(|(name, _)| name == file)
            .unwrap_or_else(|| panic!("{file} is in ALLOWED and does not exist"));
        assert!(
            source.contains("target_arch = \"wasm32\""),
            "{file} is allowed to reach the host but never mentions wasm32, so \
             nothing in it says what the browser does instead"
        );
    }
}
