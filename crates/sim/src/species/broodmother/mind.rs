//! What the Broodmother wants, and what her brood want.
//!
//! **Her brood are gnawers** (`docs/design/creatures/broodmother.md` §5): the
//! pack brain is the Gnawers' own mind ([`gnawers::Mind`]), handed every
//! question first. What [`Brood`] adds is hers: the screech calling them home,
//! the Brood guard naming whoever hit a sac, the rooted target's cap, and the
//! collapse that scatters them.

use crate::critter::{Body, Critter, CritterKind, CritterMove, Critters};
use crate::monster::{Attack, Herd};
use crate::pack::{Look, Pack, PackDecl, PackMind, Steer};
use crate::species::gnawers;
use crate::state::Player;

/// Her pack's one kind: the gnawer, borrowed whole. Index zero, as it is in
/// the Gnawers' own table, so the gnawer's mind reads its kind right.
pub const KINDS: [CritterKind; 1] = [gnawers::GNAWER_KIND];

/// The brood. Nobody musters: they hatch (`fight`), the first few at the
/// start of the hunt.
pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[],
    leader: None,
    mind: &Brood,
};

/// The brood's mind: the gnawers', and hers on top.
pub struct Brood;

const GNAWER: gnawers::Mind = gnawers::Mind;

impl PackMind for Brood {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        GNAWER.appetite(look, i, m, a)
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        GNAWER.frame(pack, critters, herd, frame);
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        GNAWER.steer(look, i, want)
    }

    fn landed(
        &self,
        pack: &mut Pack,
        critters: &mut Critters,
        i: usize,
        who: usize,
        victim: &mut Player,
        blocked: bool,
        parried: bool,
    ) {
        GNAWER.landed(pack, critters, i, who, victim, blocked, parried);
    }

    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        GNAWER.hurt(pack, critters, i, dealt);
    }

    fn died(&self, pack: &mut Pack, critters: &mut Critters, i: usize) {
        GNAWER.died(pack, critters, i);
    }

    fn body(&self, c: &Critter, plain: Body) -> Body {
        GNAWER.body(c, plain)
    }
}
