//! Legibility is a computation, so it is a test rather than an opinion.
//!
//! This is the point of doing colour in a perceptual space. "Can you tell the
//! green player from the monster" is a question somebody would otherwise have
//! to re-form a view on every time a colour moved, which in practice means
//! nobody checks and it drifts. Here it fails the build.

use art::color::{self, Lch};
use art::materials::{self, Role};
use art::palette::{self, ARENA_SEPARATION, IDENTITY_SEPARATION};

/// Every identity, including the one reserved for things that hurt you.
fn identities() -> Vec<(String, Lch)> {
    let mut v: Vec<(String, Lch)> = palette::PLAYERS
        .iter()
        .enumerate()
        .map(|(i, c)| (format!("player {i}"), *c))
        .collect();
    v.push(("hostile".into(), palette::HOSTILE));
    v
}

#[test]
fn identities_stay_apart_including_for_a_colour_blind_player() {
    for (i, (na, a)) in identities().iter().enumerate() {
        for (nb, b) in identities().iter().skip(i + 1) {
            let (ra, rb) = (a.to_linear(), b.to_linear());
            let normal = color::difference(ra, rb);
            let deuter = palette::deuteranope_separation(ra, rb);
            assert!(
                normal >= IDENTITY_SEPARATION,
                "{na} and {nb} are {normal:.3} apart, which is under {IDENTITY_SEPARATION}"
            );
            assert!(
                deuter >= IDENTITY_SEPARATION,
                "{na} and {nb} collapse to {deuter:.3} apart for a red-green colour-blind \
                 player. Lightness is what survives colour blindness -- move one of them in \
                 lightness, not in hue. `cargo run --release -p art --example hunt` searches it."
            );
        }
    }
}

#[test]
fn no_player_wears_the_colour_of_a_threat() {
    for (i, c) in palette::PLAYERS.iter().enumerate() {
        assert!(
            !palette::is_hostile(*c),
            "player {i} is inside the reserved hostile region -- \"that will hurt me\" and \
             \"that is me\" would be the same signal"
        );
    }
    assert!(
        palette::is_hostile(palette::HOSTILE),
        "the hostile colour is not hostile"
    );
}

#[test]
fn the_world_is_almost_colourless() {
    // The rule the whole palette rests on. A mossy green rock is a perfectly
    // nice rock and it costs you the green player.
    for m in materials::FIXED {
        if m.role != Role::World {
            continue;
        }
        for k in 0..=16 {
            let rgb = m.surface.ramp.at(k as f32 / 16.0);
            let c = Lch::from_oklab(color::linear_to_oklab(rgb)).c;
            assert!(
                c <= palette::WORLD_CHROMA_CEILING,
                "{} reaches chroma {c:.3} at {k}/16, over the ceiling of {}. The arena being \
                 nearly grey is what makes everything with colour in it read.",
                m.name,
                palette::WORLD_CHROMA_CEILING
            );
        }
    }
}

#[test]
fn every_identity_reads_against_the_arena() {
    let mut world = Vec::new();
    for m in materials::FIXED {
        if m.role == Role::World {
            for k in 0..=8 {
                world.push(m.surface.ramp.at(k as f32 / 8.0));
            }
        }
    }
    assert!(!world.is_empty(), "no world materials to check against");

    for (name, c) in identities() {
        for bg in &world {
            let normal = color::difference(c.to_linear(), *bg);
            let deuter = palette::deuteranope_separation(c.to_linear(), *bg);
            let worst = normal.min(deuter);
            assert!(
                worst >= ARENA_SEPARATION,
                "{name} is only {worst:.3} from something the arena is made of"
            );
        }
    }
}

#[test]
fn shading_an_identity_keeps_it_the_same_identity() {
    // What OkLCh is for. Scaling an sRGB triple to darken it desaturates as it
    // goes, so a "darker blue" made that way is a different colour; holding
    // hue and chroma and moving only lightness is the operation that was
    // actually meant.
    for i in 0..palette::PLAYERS.len() {
        let base = palette::PLAYERS[i];
        for l in [0.25, 0.45, 0.65, 0.85] {
            let shaded = Lch::from_oklab(color::linear_to_oklab(palette::player_shade(i, l)));
            let mut dh = (shaded.h - base.h).abs();
            if dh > 0.5 {
                dh = 1.0 - dh;
            }
            assert!(
                dh < 0.02,
                "player {i} shaded to lightness {l} drifted {dh:.3} turns in hue"
            );
        }
    }
}
