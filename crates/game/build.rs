//! Stamp the build with the commit it was built from, and bake the brush.
//!
//! Two players are only matched if they run the same build (`src/online.rs`,
//! `terms`), and a desktop and the published page have to be able to agree on
//! what that means: the commit hash, from either side. The page's build script
//! sets `ARENA_BUILD` itself; anything else asks git, and a machine without a
//! checkout or without git says `dev`.

use std::process::Command;

fn main() {
    bake_strokes();
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

/// **The brush strokes, baked at compile time**: `look::strokes::stack`, the
/// stroke sheets stacked, written as a PNG into `OUT_DIR` for
/// `src/paint.rs` to compile in. Drawn here rather than at start-up (a second
/// or more, and ten in a debug build) or committed (a binary that has to be
/// re-baked by hand when a knob moves). The sheet is a pure function of the
/// `look` crate, so it is drawn again exactly when that crate changes.
fn bake_strokes() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let path = out.join("strokes.png");
    let (w, h) = (
        look::strokes::SIZE as u32,
        (look::strokes::SIZE * look::strokes::LAYERS) as u32,
    );
    let px: Vec<u8> = look::strokes::stack().into_iter().flatten().collect();
    let file = std::fs::File::create(&path).expect("write the strokes");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::High);
    enc.write_header()
        .expect("png header")
        .write_image_data(&px)
        .expect("png data");
}
