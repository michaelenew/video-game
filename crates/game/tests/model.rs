//! **Every class's model is skinned to the game's skeleton** (`models.rs`,
//! `art/<class>/build.py`).
//!
//! The game sets the model's joints by name, from the skin `apply_poses`
//! solves, in `view::skeleton::JOINTS` order. A model built against another
//! skeleton -- a joint renamed, one added to the middle, the export run with
//! an old `anim.json` -- would load, draw, and bend the wrong limbs, with
//! nothing to say why. So this reads the file's own table of contents.
//!
//! If you are here because this failed: rebuild the model
//! (`art/blender.sh art/<class>/build.py`, after
//! `cargo run -p anim --bin export -- target/art/anim.json`), never edit it.

use std::path::Path;

/// The JSON chunk of a binary glTF.
fn contents(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert_eq!(&bytes[0..4], b"glTF", "not a binary glTF");
    let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    assert_eq!(&bytes[16..20], b"JSON", "the first chunk is not the JSON");
    String::from_utf8(bytes[20..20 + len].to_vec()).expect("the JSON is text")
}

/// The `"name"`s of the nodes, in order: a hand-rolled read, because the
/// game has no JSON parser and one is not worth a dependency for a test.
fn node_names(json: &str) -> Vec<String> {
    // The node table, not a scene's list of node numbers before it.
    let nodes = &json[json.find("\"nodes\":[{").expect("a node table") + 9..];
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut rest = nodes;
    while let Some(c) = rest.chars().next() {
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => {
                depth -= 1;
                if depth < 0 {
                    break;
                }
            }
            '"' if depth == 1 && rest.starts_with("\"name\":\"") => {
                let start = 8;
                let end = rest[start..].find('"').expect("a closed name");
                names.push(rest[start..start + end].to_string());
            }
            _ => {}
        }
        rest = &rest[c.len_utf8()..];
    }
    names
}

/// The models the game draws (`models::MODELS`, which the test cannot see:
/// the game is a binary). A class added there is added here.
const FILES: [&str; 5] = [
    "reaver.glb",
    "champion.glb",
    "elementalist.glb",
    "blood_mage.glb",
    "dual_mage.glb",
];

#[test]
fn every_model_is_skinned_to_the_games_skeleton() {
    for file in FILES {
        skinned(file);
    }
}

fn skinned(file: &str) {
    let json = contents(file);
    let names = node_names(&json);
    let want: Vec<&str> = view::skeleton::JOINTS.iter().map(|j| j.name()).collect();
    assert!(
        names.len() >= want.len(),
        "the model has {} nodes; the skeleton has {} joints",
        names.len(),
        want.len()
    );
    assert_eq!(
        &names[..want.len()],
        &want[..],
        "{file}: the model's first nodes are not the skeleton's joints, in order"
    );
    // The coat's joints the springs look for (`coat::TAIL_NAMES`), and the
    // blades' points the trail reads (`slash`).
    for extra in [
        "tail.0.a", "tail.0.b", "tail.1.a", "tail.1.b", "tail.2.a", "tail.2.b", "tail.3.a",
        "tail.3.b",
    ] {
        assert!(names.iter().any(|n| n == extra), "{file} has no {extra}");
    }
    if file == "reaver.glb" {
        for extra in ["blade.l.tip", "blade.l.base", "blade.r.tip", "blade.r.base"] {
            assert!(names.iter().any(|n| n == extra), "{file} has no {extra}");
        }
    }
    // The sash's material is the one the game dyes.
    assert!(
        json.contains("\"name\":\"team\""),
        "{file}: no material called team"
    );
}
