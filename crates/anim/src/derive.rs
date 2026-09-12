//! A clip for every move, from frame data the game already has.
//!
//! Every move in `sim::moves` carries a **startup**, an **active** and a
//! **recovery** count -- three numbers that already decide the entire neutral
//! game. They also completely determine the *shape* of the animation: wind up
//! for the startup, be extended for the active window, return during recovery.
//! So the clip does not need authoring. It needs deriving.
//!
//! ## What this fixes, concretely
//!
//! Before it, every attack in the game played one of two hand-authored
//! clips -- a 17-frame poke or a 42-frame overhead -- indexed by the move's own
//! elapsed frames. When the two lengths happened to match, that worked. When
//! they did not:
//!
//! - The Bulwark's **Grapple** is 20/3/30, so 53 frames. It played the
//!   17-frame poke and then **stood frozen for 36 frames**, which is most of
//!   the move.
//! - The Champion's **Sweep** is 6/3/12 against a clip that strikes on frame
//!   7, so the arm arrived a frame after the hitbox did.
//!
//! The second is the one that matters. **A telegraph that does not line up
//! with the frame data is a telegraph that lies**, and the whole design rests
//! on moves being readable: `combat-kernel.md` makes startup length the thing
//! an opponent reacts to, and the debug overlay exists because an overlay that
//! can drift from the rules is worse than none. An animation is the same kind
//! of claim, made to the player instead of to the developer.
//!
//! Deriving the keys from the move's own numbers means the extended pose
//! lands on the first active frame **by construction**, for every move, and
//! stays lined up when the frame data is retuned in the Oven.
//!
//! ## What it does not do
//!
//! Make a move look like itself. A derived clip knows how long a move takes
//! and roughly what shape it is; it does not know that the Reaver's
//! Guillotine is a downward chop with a shadow behind it. This is a floor, not
//! a ceiling -- every move gets motion that is correctly timed and
//! recognisably an attack, and the ones that carry a class's identity still
//! deserve hand-authored keys.

use view::pose::Pose;

use crate::bake::{Key, Looseness, Recipe};

/// The broad shape of a move, which is as much as its frame data can say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A level swing that a crouching opponent cannot duck. The ordinary poke.
    Level,
    /// Comes down from above. A crouch goes under it, which is what makes it
    /// worth its startup.
    Overhead,
    /// Reaches out and takes hold. No arc to read, which is the point -- a
    /// grab beats a block and is answered by not being there.
    Seize,
}

impl Shape {
    /// Which shape a move's own flags imply.
    ///
    /// Derived rather than declared, so a move that changes from an overhead
    /// to a level swing in the Oven changes its animation at the same moment
    /// rather than the next time somebody remembers.
    pub fn of(hits_crouching: bool, grabs: u16) -> Shape {
        if grabs > 0 {
            Shape::Seize
        } else if hits_crouching {
            Shape::Level
        } else {
            Shape::Overhead
        }
    }
}

/// The four poses a derived clip is built from.
///
/// Supplied by the caller rather than defined here, because they are the one
/// genuinely authored thing in this file and they live with the other poses in
/// the bake binary.
pub struct Vocabulary {
    pub neutral: Pose,
    /// Coiled back, ready. Where the move goes during startup.
    pub coil: Pose,
    /// Fully extended. Where the move is while the hitbox is live.
    pub strike: Pose,
    /// An optional sharp beat on frame two, before the coil.
    ///
    /// This is not decoration. A heavy clip lags by design, and on a long
    /// startup that lag eats the first third of the wind-up -- the silhouette
    /// barely moves while the opponent is supposed to be reading it and
    /// deciding whether to block. `moves.rs` puts the reaction window at
    /// fifteen frames, so a move with more startup than that is one the design
    /// *intends* to be answered on sight, and it has to have something to see.
    pub anticipate: Option<Pose>,
}

/// Build a clip for a move from its frame data.
///
/// The key placement is the whole idea:
///
/// | Frame | Pose | Why |
/// | --- | --- | --- |
/// | 0 | neutral | Where the fighter was |
/// | a quarter into startup | coil | The anticipation, early enough to read |
/// | **startup** | **strike** | **The hitbox appears on this frame** |
/// | startup + active − 1 | strike | Held while it is live |
/// | end | neutral | Recovery |
///
/// The third row is the load-bearing one. Everything else is timing; that one
/// is the promise the animation makes to the opponent.
pub fn from_frame_data(
    name: &'static str,
    startup: u16,
    active: u16,
    recovery: u16,
    shape: Shape,
    vocab: &Vocabulary,
) -> Recipe {
    let length = startup + active + recovery;

    // Halfway through the wind-up, capped so a long startup does not spend ten
    // frames drifting before anything happens. Under about fifteen frames of
    // startup a move is unreactable and must be *anticipated* -- `moves.rs`
    // says so in as many words -- so on short moves the coil has to land early
    // enough to be worth anticipating, and the clamp at one frame is what
    // guarantees a four-frame poke still shows a wind-up at all.
    let coil_at = (startup / 2).clamp(1, 6).min(startup.saturating_sub(1));

    let mut keys = vec![Key {
        frame: 0,
        pose: vocab.neutral,
    }];

    // The early beat only fits, and is only needed, on a startup long enough
    // to be reacted to. On a four-frame poke there is no room for it and no
    // decision for it to inform.
    if let Some(anticipate) = vocab.anticipate {
        if startup >= 10 {
            keys.push(Key {
                frame: 2,
                pose: anticipate,
            });
        }
    }

    keys.push(Key {
        frame: coil_at,
        pose: vocab.coil,
    });
    keys.push(Key {
        frame: startup,
        pose: vocab.strike,
    });

    // Hold the extension across the active window. Without this the solver
    // starts returning to neutral the instant the hitbox appears, so the
    // fighter is already withdrawing during the frames the move can hit --
    // which reads as the attack missing on purpose.
    if active > 1 {
        keys.push(Key {
            frame: startup + active - 1,
            pose: vocab.strike,
        });
    }

    keys.push(Key {
        frame: length.saturating_sub(1),
        pose: vocab.neutral,
    });

    Recipe {
        name,
        length,
        keys,
        // Heavier moves get looser limbs, and the threshold is the reaction
        // window rather than a round number: a move you can see coming is a
        // move whose weight you have time to read, and one you cannot is a
        // move that has to be crisp or it is unreadable as well as unreactable.
        looseness: match shape {
            Shape::Level if startup < 15 => Looseness::MARTIAL,
            Shape::Seize => Looseness::MARTIAL,
            _ => Looseness::HEAVY,
        },
    }
}
