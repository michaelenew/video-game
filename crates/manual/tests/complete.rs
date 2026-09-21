//! The manual has to describe the game that exists.
//!
//! Help text maintained separately from the thing it describes is wrong within
//! a month, and wrong help is worse than none — it sends you looking for a
//! feature that moved. These read the game's own source and check that nothing
//! it responds to is missing here.

use std::path::Path;

fn game_source() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("game/src");
    let mut all = String::new();
    for entry in std::fs::read_dir(root).expect("game/src is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            all.push_str(&std::fs::read_to_string(&path).expect("source is utf-8"));
        }
    }
    all
}

/// How a `KeyCode` variant is written for a person.
///
/// Several map onto one phrase — `KeyW` through `KeyD` are all "WASD", and left
/// and right modifiers are both just the modifier — because that is how the
/// manual talks about them and how a player thinks about them.
fn spoken(key: &str) -> Vec<&'static str> {
    match key {
        "KeyW" | "KeyA" | "KeyS" | "KeyD" => vec!["W A S D", "WASD"],
        "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => vec!["Arrow keys"],
        "ShiftLeft" => vec!["Shift"],
        "ShiftRight" => vec!["Right Shift"],
        "ControlLeft" => vec!["Ctrl"],
        "ControlRight" => vec!["Right Ctrl"],
        "KeyC" => vec!["Ctrl or C", " C"],
        "Space" => vec!["Space"],
        "Escape" => vec!["Esc"],
        "Tab" => vec!["Tab"],
        "KeyE" => vec!["E"],
        "KeyH" => vec!["H"],
        "KeyJ" => vec!["J"],
        "KeyK" => vec!["K"],
        "KeyL" => vec!["L"],
        // Player two's third attack button. They have no scroll wheel on their
        // half of the keyboard, the way they have no mouse at all.
        "KeyM" => vec!["M"],
        "KeyP" => vec!["P"],
        "KeyQ" => vec!["Q"],
        "KeyR" => vec!["R"],
        // The stand-in for middle click, the way J and K stand in for the other
        // two. It exists for the Champion's third weapon.
        "KeyU" => vec!["U or middle click", "Middle click (or U)"],
        "BracketRight" => vec!["]"],
        "BracketLeft" => vec!["["],
        // The rehearsed double structure jump, `--dev` only.
        "KeyG" => vec!["G"],
        "Minus" | "NumpadSubtract" => vec!["-"],
        "Equal" | "NumpadAdd" => vec!["="],
        "Semicolon" | "Quote" => vec!["; and '"],
        "Period" | "Comma" | "Slash" => vec![". , /"],
        "Digit1" => vec!["1"],
        "Digit2" => vec!["2"],
        "Digit3" => vec!["3"],
        "Digit4" => vec!["4"],
        "F1" => vec!["F1"],
        "F2" => vec!["F2"],
        "F3" | "F4" => vec!["F3 and F4"],
        "F5" | "F6" => vec!["F5 and F6"],
        "F7" => vec!["F7"],
        // Every `F<n>` the game might grow needs a spelling here, so that
        // `a_documented_function_key_does_something` below can ask the question
        // in both directions.
        "F8" => vec!["F8"],
        "F9" => vec!["F9"],
        _ => vec![],
    }
}

#[test]
fn every_key_the_game_handles_is_documented() {
    let source = game_source();
    let text = manual::render();

    let mut undocumented = Vec::new();
    let mut unmapped = Vec::new();
    for (i, _) in source.match_indices("KeyCode::") {
        let rest = &source[i + "KeyCode::".len()..];
        let end = rest
            .find(|c: char| !c.is_alphanumeric())
            .unwrap_or(rest.len());
        let key = &rest[..end];

        let names = spoken(key);
        if names.is_empty() {
            unmapped.push(key.to_string());
            continue;
        }
        if !names.iter().any(|n| text.contains(n)) {
            undocumented.push(key.to_string());
        }
    }
    unmapped.sort();
    unmapped.dedup();
    undocumented.sort();
    undocumented.dedup();

    assert!(
        unmapped.is_empty(),
        "the game handles keys this test does not know how to spell: {unmapped:?} \
         -- add them to `spoken`, and to the manual"
    );
    assert!(
        undocumented.is_empty(),
        "the game handles these keys and the manual never mentions them: {undocumented:?}"
    );
}

/// And the other way round: the manual must not offer a key the game ignores.
///
/// The test above asks whether everything the game answers to is written down.
/// That is only half of "the manual has to describe the game that exists", and
/// the half it leaves out is the one that rots quietly: a binding is deleted,
/// its entry is not, and the help goes on naming a key for the rest of its
/// life. Nothing fails, because nothing was added.
///
/// **Function keys only**, and that is the honest scope rather than a
/// simplification. `F3` means one thing, so finding it in the game's source is
/// a real answer. A manual entry reading "Shift" or "1-4" is a phrase the
/// source spells several ways and a scan for it would be a guess dressed as a
/// check. The function keys are where the settings, the overlays and the tools
/// live, which is exactly where a binding gets retired without its paperwork --
/// camera distance did precisely this, on `F5` and `F6`.
#[test]
fn a_documented_function_key_does_something() {
    let source = game_source();
    let text = manual::render();

    let mut dead = Vec::new();
    for n in 1..=12u8 {
        let key = format!("F{n}");
        // The manual names them singly ("F7") and in pairs ("F3 and F4"), so
        // look for the entry either way round.
        let named = text.contains(&format!("{key} and "))
            || text.contains(&format!(" and {key}"))
            || text.contains(&format!("**{key}**"))
            || text.contains(&format!("{key}."))
            || text.contains(&format!("{key} "));
        if named && !source.contains(&format!("KeyCode::{key}")) {
            dead.push(key);
        }
    }

    assert!(
        dead.is_empty(),
        "the manual documents these keys and the game never reads them: {dead:?} \
         -- either the binding went and the entry did not, or the entry is \
         describing something that was never built"
    );
}

#[test]
fn every_environment_variable_is_documented() {
    let source = game_source();
    let text = manual::render();
    let mut missing = Vec::new();
    // `platform::env` is how the game asks now; the other three are the
    // spellings that reach it. All four have to be watched or a knob can be
    // added by using the one this test forgot.
    for marker in [
        "env::var(\"",
        "env_num(\"",
        "env_f32(\"",
        "platform::env(\"",
        "env_parsed(\"",
    ] {
        for (i, _) in source.match_indices(marker) {
            let rest = &source[i + marker.len()..];
            let Some(end) = rest.find('"') else { continue };
            let name = &rest[..end];
            // HOME is the operating system's, not ours to document.
            if name == "HOME" || text.contains(name) {
                continue;
            }
            missing.push(name.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "the game reads these and the manual never mentions them: {missing:?}"
    );
}

#[test]
fn every_command_line_flag_is_documented() {
    // Asking through `platform::flag` and `platform::value` is the only way the
    // game reads a flag, so those two call sites are what this looks for rather
    // than every `"--"` in the source. That used to mean scanning `main.rs`
    // alone and excluding `bake.rs`, which shells out to git and rustfmt: the
    // first version of this test reported `--abbrev-ref` as an undocumented
    // feature. Now the whole crate can be read, subprocess arguments and the
    // parser's own test cases included, because neither of those is a call.
    let source = game_source();
    let text = manual::render();
    let mut missing = Vec::new();
    for marker in ["platform::flag(\"", "platform::value(\""] {
        for (i, _) in source.match_indices(marker) {
            let rest = &source[i + marker.len()..];
            let Some(end) = rest.find('"') else { continue };
            let flag = &rest[..end];
            if !text.contains(flag) {
                missing.push(flag.to_string());
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "undocumented flags: {missing:?}");
}

#[test]
fn the_browser_panel_is_exactly_the_sections_marked_for_it() {
    // The page a shared link leads to gets its controls from `browser_help`.
    // Everything declared for it arrives, and nothing arrives that was not
    // declared. A section quietly appearing
    // there would be offering a browser something it cannot do; one quietly
    // missing would leave a player without a key they need.
    let html = manual::browser_help();
    let mut declared = 0;
    for section in manual::SECTIONS {
        let heading = format!("<h3>{}</h3>", section.title);
        assert_eq!(
            html.contains(&heading),
            section.in_browser,
            "{:?} is marked in_browser = {} and the panel disagrees",
            section.title,
            section.in_browser
        );
        if !section.in_browser {
            continue;
        }
        declared += section.entries.len();

        // Inside this section's own block, not just somewhere in the page: two
        // sections can name the same thing, and `./crates/web/build-game.sh` is
        // in both "In a browser" and "Scripts" the way `./scripts/dev.sh`
        // already is. Checking the whole page would let a shared name stand in
        // for a missing one.
        let from = html.find(&heading).expect("the heading is there");
        let block = &html[from..];
        let block = &block[..block.find("</section>").expect("the block closes")];
        for entry in section.entries {
            let term = format!("<dt>{}</dt>", escaped(entry.invocation));
            assert!(
                block.contains(&term),
                "{:?} is declared for the browser panel and never reaches it",
                entry.invocation
            );
        }
    }

    // And nothing else got in. Counting closes the other half: the loop above
    // proves every declared entry arrived, this proves nothing arrived that was
    // not declared.
    assert_eq!(
        html.matches("<dt>").count(),
        declared,
        "the browser panel shows a number of entries nothing declared"
    );
}

/// The same escaping `browser_help` does, so a placeholder like `<class>` can be
/// looked for as it actually reaches the page.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[test]
fn every_binary_and_script_in_the_repository_is_listed() {
    // Catches a tool added without a way to find out it exists, which is the
    // same as it not existing.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let text = manual::render();
    let mut missing = Vec::new();

    for crate_dir in std::fs::read_dir(root).expect("crates is readable") {
        let bin = crate_dir.expect("dir entry").path().join("src/bin");
        if !bin.is_dir() {
            continue;
        }
        for f in std::fs::read_dir(&bin).expect("bin dir") {
            let p = f.expect("dir entry").path();
            let stem = p.file_stem().unwrap().to_string_lossy().to_string();
            if !text.contains(&stem) {
                missing.push(stem);
            }
        }
    }

    let scripts = root.parent().unwrap().join("scripts");
    for f in std::fs::read_dir(&scripts).expect("scripts is readable") {
        let p = f.expect("dir entry").path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if !text.contains(&name) {
            missing.push(name);
        }
    }

    missing.sort();
    assert!(
        missing.is_empty(),
        "these exist but the manual never mentions them: {missing:?}"
    );
}
