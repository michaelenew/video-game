//! Who is who, and what will hurt you.
//!
//! **Hue is a gameplay channel, so materials may not spend it.**
//!
//! That single rule does more for how the game reads than any texture will. In
//! a third-person fight with four players and several monsters, the questions
//! that have to be answered in a glance are *which one is me* and *which of
//! that is dangerous*. Colour is the fastest channel the eye has for that, and
//! it only works if nothing else is competing for it. A mossy green rock is a
//! perfectly nice rock and it costs you the green player.
//!
//! So the allocation is:
//!
//! | Band | Belongs to |
//! | --- | --- |
//! | Saturated red | **Hostile.** Monsters, active hitboxes, incoming danger. No player is ever assigned it |
//! | Four spaced hues | **Players.** One each |
//! | Near zero chroma | **The world.** Stone, floor, walls, sky |
//!
//! The arena being almost colourless is the load-bearing part. It is not a
//! stylistic preference, it is what makes everything else legible: on a grey
//! stage, anything with colour in it is by definition a thing that matters.
//! This is also the cheapest art direction available, which is not a
//! coincidence — it is the same reason the closed arena was chosen.
//!
//! **Class identity is carried by silhouette, material and effect shape, not
//! by hue.** Which class the opponent is playing is something you learn once
//! at the start of a round; which fighter is yours is something you need every
//! frame. The urgent question gets the fast channel.
//!
//! ## Identity is never hue alone
//!
//! Around one man in twelve has a shifted green response, and for them two
//! colours that differ only in hue can land on top of each other. The fix is
//! not to avoid green — it is to make every identity differ in **lightness as
//! well**, because lightness survives every form of colour blindness. The four
//! player colours below are spaced in both, and
//! [`separations_hold`](../../art/tests/palette.rs) fails the build if they
//! ever stop being.

use crate::color::{self, Lch, Rgb};

/// How colourful a player colour is. One value for all four: equal chroma at
/// equal lightness is what "equally vivid" means in a perceptual space, and it
/// stops one player's colour shouting over another's.
pub const IDENTITY_CHROMA: f32 = 0.14;

/// The hostile band, in turns. Red through orange-red.
pub const HOSTILE_BAND: (f32, f32) = (0.015, 0.145);

/// ...and how colourful something has to be before that band means anything.
///
/// This qualifier was not in the first draft of the rule and the first
/// material built against it found the hole: **human skin is red-orange.** So
/// is leather, so is rust, so is firelight on a wall. A hue band alone would
/// have banned half of what a fighter is made of.
///
/// The fix is to say what was actually meant. Danger is not *reddish*, it is
/// **saturated red** — the eye separates a vivid red from a tan long before it
/// separates two hues at the same chroma. So the reserved region is a hue band
/// **and** a chroma floor, and a desaturated warm walks underneath it without
/// ever competing.
pub const HOSTILE_CHROMA_FLOOR: f32 = 0.10;

/// Anything the world is made of stays below this chroma. Enforced by a test
/// over every material in [`crate::materials`].
pub const WORLD_CHROMA_CEILING: f32 = 0.045;

/// The four player identities.
///
/// **These lightnesses were searched for, not chosen.** Hue alone is not
/// enough — see the module header — so the free parameter is where each
/// identity sits in lightness, and the objective is the *worst* pair, because
/// a palette is only as readable as its most confusable two colours.
///
/// The search that produced them is in `examples/hunt.rs`. Two things it
/// taught, both of which cost an attempt each:
///
/// - **Unconstrained, it has no taste.** Maximising separation alone puts the
///   green at lightness 0.92, which separates beautifully and looks like pale
///   mint. Lightness is bounded to the range where a colour still reads as its
///   own name.
/// - **Telling two players apart and telling a player from a wall are not the
///   same task.** Folding both into one objective collapsed every identity
///   onto the same lightness, chasing an arena term that was never in danger:
///   the arena has almost no chroma, so anything coloured is far from all of
///   it by construction. The arena is a floor to clear, not a quantity to
///   maximise.
///
/// What came out: every pair at least **0.136** apart under red-green colour
/// blindness, and every identity at least **0.122** from anything the arena is
/// made of. The stubborn pair is green against hostile red, which is the one
/// the condition is named for, and it is carried entirely by lightness.
pub const PLAYERS: [Lch; 4] = [
    Lch::new(0.46, IDENTITY_CHROMA, 0.700), // blue
    Lch::new(0.82, IDENTITY_CHROMA, 0.190), // amber
    Lch::new(0.74, IDENTITY_CHROMA, 0.470), // green
    Lch::new(0.66, IDENTITY_CHROMA, 0.830), // violet
];

/// Hostile red. Monsters wear it, hitboxes flash it, and no player ever has it.
pub const HOSTILE: Lch = Lch::new(0.62, 0.19, 0.080);

/// How far apart two identities have to stay, as a distance in Oklab, under
/// ordinary *and* colour-blind vision. Below roughly 0.10 two things meant to
/// be told apart at a glance cannot be.
pub const IDENTITY_SEPARATION: f32 = 0.13;

/// How far an identity has to stay from anything the arena is made of. Lower
/// than [`IDENTITY_SEPARATION`] on purpose: the arena is colourless, so this
/// is a sanity floor rather than a hard task.
pub const ARENA_SEPARATION: f32 = 0.12;

/// The colour of a player, in linear RGB.
pub fn player(index: usize) -> Rgb {
    PLAYERS[index % PLAYERS.len()].to_linear()
}

/// A player's colour, lightened or darkened without changing which player it
/// is.
///
/// For the parts that need to read as the same identity at a different value —
/// armour against cloth, a trail against its source. Hue and chroma are held,
/// which is precisely the operation OkLCh exists to make possible and which
/// scaling an RGB triple does not do: scaling sRGB channels desaturates as it
/// darkens and shifts hue as it brightens.
pub fn player_shade(index: usize, lightness: f32) -> Rgb {
    let base = PLAYERS[index % PLAYERS.len()];
    Lch::new(lightness.clamp(0.0, 1.0), base.c, base.h).to_linear()
}

/// Whether a colour lands in the region reserved for things that want to hurt
/// you: in the red band **and** colourful enough to read as a warning.
pub fn is_hostile(c: Lch) -> bool {
    if c.c < HOSTILE_CHROMA_FLOOR {
        return false;
    }
    let h = c.h.rem_euclid(1.0);
    if HOSTILE_BAND.0 <= HOSTILE_BAND.1 {
        h >= HOSTILE_BAND.0 && h <= HOSTILE_BAND.1
    } else {
        h >= HOSTILE_BAND.0 || h <= HOSTILE_BAND.1
    }
}

/// How far apart two colours stay for a viewer with the common form of
/// red-green colour blindness.
///
/// The number to compare against is roughly 0.10: below it, two things that
/// are meant to be told apart at a glance cannot be.
pub fn deuteranope_separation(a: Rgb, b: Rgb) -> f32 {
    color::difference(color::deuteranope(a), color::deuteranope(b))
}
