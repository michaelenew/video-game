//! Every material the game draws with has its pipelines compiled before they
//! are needed (`game::warm`).
//!
//! A pipeline is compiled the first frame something on screen needs it, and
//! in a browser and on macOS the frame waits for it: tens of milliseconds,
//! hundreds on the web. The warm-up asks for every combination of mesh and
//! material as the mesh is made, so the wait lands while the game starts
//! instead of in the middle of a fight -- but only for the material types it
//! was told about. A new material type left off its list puts the seizing
//! back, at random and only the first time each thing is seen, which is the
//! kind of failure that takes a replay and an afternoon to find (2026-10-16).
//!
//! If you are here because this test failed: add `prime::<YourMaterial>` to
//! `WarmPlugin` and give the material an `impl Warm` saying what picks its
//! pipeline.

use std::path::Path;

fn source(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {file}: {e}"))
}

fn game_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read game/src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = path.display().to_string();
                out.push((name, std::fs::read_to_string(&path).expect("read source")));
            }
        }
    }
    out
}

#[test]
fn every_material_is_warmed() {
    let warm = source("warm.rs");
    // The standard material is Bevy's own, registered by its default
    // plugins rather than here, and it is most of what moves.
    let mut materials = vec![String::from("StandardMaterial")];
    for (_, text) in game_sources() {
        for line in text.lines() {
            let code = line.split("//").next().unwrap_or(line);
            let Some(at) = code.find("MaterialPlugin::<") else {
                continue;
            };
            let rest = &code[at + "MaterialPlugin::<".len()..];
            let name = rest.split('>').next().unwrap_or_default();
            let name = name.rsplit("::").next().unwrap_or(name).trim();
            if !name.is_empty() && !materials.iter().any(|m| m == name) {
                materials.push(name.to_string());
            }
        }
    }
    let missing: Vec<&String> = materials
        .iter()
        .filter(|m| !warm.contains(&format!("prime::<{m}>")))
        .collect();
    assert!(
        missing.is_empty(),
        "these materials are drawn but never warmed, so the first frame each \
         combination is seen compiles a pipeline in the middle of play: {missing:?}"
    );
}
