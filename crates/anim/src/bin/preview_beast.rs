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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = std::path::Path::new("target/beast-preview");
    std::fs::create_dir_all(dir).expect("could not make target/beast-preview");

    let named = args
        .iter()
        .position(|a| a == "--species")
        .and_then(|i| args.get(i + 1));
    let sp = match named {
        Some(name) => sim::species::named(name).unwrap_or_else(|| {
            eprintln!("no species is called {name}");
            std::process::exit(2);
        }),
        None => &sim::species::ridgeback::SPECIES,
    };
    let rest: Vec<&String> = args
        .iter()
        .filter(|a| Some(*a) != named && *a != "--species")
        .collect();

    let all: Vec<usize> = (0..sp.clips.len()).collect();
    let want: Vec<usize> = if rest.iter().any(|a| *a == "--all") || rest.is_empty() {
        all
    } else {
        rest.iter()
            .filter(|a| !a.starts_with("--"))
            .filter_map(|name| sp.clips.iter().position(|c| c.name == name.as_str()))
            .collect()
    };

    if rest.iter().any(|a| *a == "--states") || rest.is_empty() {
        let path = dir.join(format!("{}-states.png", sp.slug()));
        anim::beast::sheet::states(sp)
            .write(&path)
            .expect("could not write the sheet");
        println!(
            "{}  standing, walking, galloping, every move out, stumbling, toppled, lamed",
            path.display()
        );
    }

    for clip in want {
        let path = dir.join(format!("{}-{}.png", sp.slug(), sp.clips[clip].name));
        anim::beast::sheet::contact_sheet(sp, clip)
            .write(&path)
            .expect("could not write the sheet");
        println!("{}", path.display());
    }
}
