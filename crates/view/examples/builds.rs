//! `cargo run -p view --example builds` -- how well the roster separates by shape.
//!
//! Colour is spoken for as the channel that says which *player*, so silhouette
//! is what has to say which *class*. This prints the pairwise distinctness so
//! that claim can be checked rather than assumed.

use view::build::{self, Build, Silhouette, distinctness};

const NAMES: [&str; 6] = [
    "Bulwark",
    "Champion",
    "Reaver",
    "Elementalist",
    "Blood mage",
    "Dual mage",
];

fn main() {
    let sils: Vec<Silhouette> = build::CLASS_BUILDS.iter().map(Silhouette::of).collect();

    println!("{:>13}  {:>5}  {:>6}", "class", "cells", "height");
    for (i, b) in build::CLASS_BUILDS.iter().enumerate() {
        println!(
            "{:>13}  {:>5}  {:>5.2}m",
            NAMES[i],
            build::area(&sils[i]),
            b.height()
        );
    }

    println!("\npairwise distinctness (0 identical, 1 no overlap)");
    let mut worst = (1.0f32, "", "");
    for i in 0..sils.len() {
        for j in i + 1..sils.len() {
            let d = distinctness(&sils[i], &sils[j]);
            println!("  {:>13} / {:<13} {d:.3}", NAMES[i], NAMES[j]);
            if d < worst.0 {
                worst = (d, NAMES[i], NAMES[j]);
            }
        }
    }
    println!("\nworst pair: {} / {} at {:.3}", worst.1, worst.2, worst.0);

    // And against the body the prototype used for everyone, as a reference for
    // how much any of this actually moved.
    let even = Silhouette::of(&Build::EVEN);
    println!("\nagainst the old one-size body:");
    for (i, s) in sils.iter().enumerate() {
        println!("  {:>13} {:.3}", NAMES[i], distinctness(s, &even));
    }
}
