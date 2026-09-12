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
        "KeyP" => vec!["P"],
        "KeyQ" => vec!["Q"],
        "KeyR" => vec!["R"],
        // The stand-in for middle click, the way J and K stand in for the other
        // two. It exists for the Champion's third weapon.
        "KeyU" => vec!["U or middle click", "Middle click (or U)"],
        "BracketRight" => vec!["]"],
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

#[test]
fn every_environment_variable_is_documented() {
    let source = game_source();
    let text = manual::render();
    let mut missing = Vec::new();
    for marker in ["env::var(\"", "env_num(\"", "env_f32(\""] {
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
    // Only `main.rs`, where argument parsing lives. `bake.rs` shells out to git
    // and rustfmt, and their flags are theirs to document, not ours -- the first
    // version of this test reported `--abbrev-ref` as an undocumented feature.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("game/src/main.rs");
    let source = std::fs::read_to_string(path).expect("main.rs is readable");
    let text = manual::render();
    let mut missing = Vec::new();
    for (i, _) in source.match_indices("\"--") {
        let rest = &source[i + 1..];
        let Some(end) = rest.find('"') else { continue };
        let flag = &rest[..end];
        if !text.contains(flag) {
            missing.push(flag.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "undocumented flags: {missing:?}");
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

#[test]
fn the_on_screen_legend_is_exactly_the_short_forms() {
    // The legend used to be a hand-kept copy of the key handlers and had
    // already drifted once. It is now assembled from the same entries, so what
    // this checks is that assembly: every short form reaches the screen, and
    // the screen shows nothing that is not a short form.
    //
    // Note it cannot check the shorts against the manual *prose* -- they are
    // deliberately compact rephrasings ("Mouse aims" against "Aim. Where you
    // look is where you are pointed."), not substrings of it.
    let legend = manual::legend();
    assert!(!legend.is_empty(), "the legend is empty");

    let shorts: Vec<&str> = manual::SECTIONS
        .iter()
        .flat_map(|s| s.entries.iter())
        .filter_map(|e| e.short)
        .collect();

    for short in &shorts {
        assert!(
            legend.contains(short),
            "{short:?} is marked for the legend but never reaches it"
        );
    }

    // Checked by removal rather than by splitting on the separator: one short
    // ("- / = mouse") contains " / " itself, and splitting counted it twice.
    let mut remainder = legend.clone();
    for short in &shorts {
        let at = remainder
            .find(short)
            .unwrap_or_else(|| panic!("{short:?} missing from the legend"));
        remainder.replace_range(at..at + short.len(), "");
    }
    let leftover: String = remainder.replace(" / ", "").replace('\n', "");
    assert!(
        leftover.is_empty(),
        "the legend shows something no entry declares: {leftover:?}"
    );
}
