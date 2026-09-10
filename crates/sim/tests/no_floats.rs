//! Guard test: the simulation must contain no floating point.
//!
//! This is the single cheapest defence against cross-platform desync. Floats
//! are permitted only on lines explicitly marked `RENDER-ONLY`, which are
//! conversions for the renderer and are never called from simulation code.

use std::fs;
use std::path::Path;

#[test]
fn simulation_contains_no_floating_point() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offences = Vec::new();
    walk(&src, &mut offences);
    assert!(
        offences.is_empty(),
        "floating point found in the simulation:\n{}",
        offences.join("\n")
    );
}

fn walk(dir: &Path, out: &mut Vec<String>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("sim/src is readable")
        .map(|e| e.expect("dir entry").path())
        .collect();
    entries.sort(); // deterministic even in the test harness

    for path in entries {
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).expect("source is utf-8");
            let mut render_only_scope = false;
            for (n, line) in text.lines().enumerate() {
                // A doc comment marking RENDER-ONLY opens a scope covering the
                // item it documents; the marker on a code line covers that line.
                if line.contains("RENDER-ONLY") {
                    render_only_scope = true;
                    continue;
                }
                let code = line.split("//").next().unwrap_or("");
                let hit = ["f32", "f64", "as f", "0.0", "1.0"]
                    .iter()
                    .any(|pat| code.contains(pat));
                if hit && !render_only_scope {
                    out.push(format!("{}:{}: {}", path.display(), n + 1, line.trim()));
                }
                // Scope ends at the closing brace of the marked item.
                if render_only_scope && line.starts_with("    }") {
                    render_only_scope = false;
                }
            }
        }
    }
}
