//! Draw the Ridgeback, without running the game.
//!
//!     cargo run -p anim --bin preview_beast -- shake
//!     cargo run -p anim --bin preview_beast -- --all
//!     cargo run -p anim --bin preview_beast -- --states
//!
//! Writes PNGs into `target/beast-preview`. A clip sheet is three panels: the
//! creature from the side, from above, and every frame of the clip overlaid so
//! the arcs are visible as arcs. `--states` is not a clip at all -- it is the
//! poses the *simulation* produces, layers and all, which is what a player sees
//! and is not the same thing as a baked sample.
//!
//! Every sheet draws a dashed line at 4.14 m, which is what a full hop reaches.
//! The climb is a geometry problem and this is the geometry, as a picture.
//!
//! Colours: green is a surface you can stand on, red is a weak point, sand is a
//! breakable foot, grey is armour.

use sim::beast::Clip;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = std::path::Path::new("target/beast-preview");
    std::fs::create_dir_all(dir).expect("could not make target/beast-preview");

    let want: Vec<Clip> = if args.iter().any(|a| a == "--all") || args.is_empty() {
        Clip::ALL.to_vec()
    } else {
        args.iter()
            .filter(|a| !a.starts_with("--"))
            .filter_map(|name| Clip::ALL.into_iter().find(|c| c.name() == name))
            .collect()
    };

    if args.iter().any(|a| a == "--states") || args.is_empty() {
        let path = dir.join("states.png");
        anim::beast::sheet::states()
            .write(&path)
            .expect("could not write the sheet");
        println!(
            "{}  standing, walking, galloping, bite, slam, sweep, stumbling, toppled, lamed",
            path.display()
        );
    }

    for clip in want {
        let path = dir.join(format!("{}.png", clip.name()));
        anim::beast::sheet::contact_sheet(clip)
            .write(&path)
            .expect("could not write the sheet");
        println!("{}", path.display());
    }
}
