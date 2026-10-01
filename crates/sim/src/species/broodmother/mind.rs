//! What the Broodmother wants, and what her brood want.
//!
//! **Her own terms** (`docs/design/creatures/broodmother.md` §5), after the
//! Ridgeback's shared scoring:
//!
//! ```text
//! + under(m)   enormous for the bare slam while anyone is under the abdomen
//! + aloft(m)   the web shot against an airborne target: it webs her down
//! + recall(m)  screech only: how far her brood are from her, and how many
//!              fighters are at her sacs
//! ⟂ the slam only from a screech or under(m); no screech once enraged
//! ```
//!
//! (`rooted(m)` is the shared `combo_appetite`, which the lunge and the web
//! shot already read off a stunned target.) What she decides as a move
//! commits is here too: which leg stabs, and where; how many follow it; which
//! anchor a web line goes to.
//!
//! **Her brood are gnawers**: the pack brain is the Gnawers' own mind
//! ([`gnawers::Mind`]), handed every question first. What [`Brood`] adds is
//! hers: the screech calling them home, the Brood guard naming whoever hit a
//! sac, and the rooted target's cap.

use super::fight::{self, body};
use super::{
    ABDOMEN_PART, Knob, LEG_COUNT, LUNGE, SAC_COUNT, SCREECH, SLAM, STAB, WEB_LINE, WEB_SHOT,
    shin_part,
};
use crate::critter::{Body, Critter, CritterKind, CritterMove, Critters, flag, is};
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Attack, Doing, Herd, Mind, Monster};
use crate::pack::{Look, Pack, PackDecl, PackKnob, PackMind, Steer, give_back};
use crate::species::gnawers;
use crate::state::{MAX_PLAYERS, Player};

// ---------------------------------------------------------------------------
// Her brain
// ---------------------------------------------------------------------------

/// Is a point on the floor under her abdomen: inside its box, seen from
/// above?
pub fn under_abdomen(m: &Monster, at: V3) -> bool {
    let rig = m.rig();
    let sh = m.sp().shape(ABDOMEN_PART);
    let frame = rig.of(ABDOMEN_PART);
    let local = frame.world_to_local(V3::new(at.x, frame.at.y, at.z));
    local.x.raw() >= sh.min.x.raw()
        && local.x.raw() <= sh.max.x.raw()
        && local.z.raw() >= sh.min.z.raw()
        && local.z.raw() <= sh.max.z.raw()
}

/// **Her own terms in the scoring** (`FightDecl::appetite`).
pub fn appetite(m: &Monster, kind: u8, score: i32, mind: &Mind) -> i32 {
    let mad = fight::enraged(m);
    match kind {
        // The slam is the window: only from a screech (which chains to it)
        // or for somebody under her.
        SLAM => {
            let under = mind
                .quarry
                .iter()
                .any(|q| q.alive && !q.aboard && under_abdomen(m, q.pos));
            if under {
                score.max(0) + Knob::UnderAppetite.raw()
            } else {
                0
            }
        }
        // The screech calls a brood she no longer has once enraged.
        SCREECH if mad => 0,
        SCREECH => score.max(0) + recall(m),
        WEB_SHOT => {
            let q = mind.quarry.get(m.brain.target as usize);
            let aloft = q.is_some_and(|q| q.pos.y.raw() > crate::tuning::body_radius().raw());
            let up = if aloft { Knob::AloftAppetite.raw() } else { 0 };
            score + rooted(m, score) + up
        }
        LUNGE => score + rooted(m, score),
        WEB_LINE => {
            let far = math::wide_flat_dist(m.brain.seen, m.pos).raw() > Knob::LineFrom.fx().raw();
            if !far || anchor_for(m, mind).is_none() {
                return 0;
            }
            let line = score.max(0) + Knob::LineAppetite.raw();
            if mad {
                Fx::from_int(line).mul(Knob::EnrageLines.fx()).to_int()
            } else {
                line
            }
        }
        // A stab only at somebody standing at a foot, or under her.
        STAB if !stab_reaches(m, m.lead_point(m.sp().attack(STAB).startup)) => 0,
        _ => score,
    }
}

/// **rooted(m)**: the lunge and a second web shot, against a target a glob
/// holds -- the shared `combo_appetite`, which reads a stunned target, and a
/// fighter rooted by the floor is not stunned. Nothing for a move the shared
/// terms gave nothing: one that does not reach them.
fn rooted(m: &Monster, score: i32) -> i32 {
    let held = m.own[body::WEBBED] & 0b11;
    if score > 0 && held & (1 << m.brain.target.min(1)) != 0 {
        m.sp().combo_appetite()
    } else {
        0
    }
}

/// The screech's appetite, as her frame worked it out with the whole world
/// in hand ([`work_out_recall`]).
fn recall(m: &Monster) -> i32 {
    (m.own[body::WEBBED] as u32 >> 8) as i32
}

/// Is a point under her -- the abdomen's footprint, or her middle -- where
/// only the middle legs, driven inward, reach?
pub fn under_her(m: &Monster, at: V3) -> bool {
    under_abdomen(m, at) || math::wide_flat_dist(at, m.pos).raw() <= Knob::StabFar.fx().raw()
}

/// **Where leg `leg` stabs**, for a target at `at`. **Outward**, at its own
/// foot: the disc a step in from where the foot stands -- a leg drives down
/// where it is, so the answer is positional: do not stand at a foot. **Inward**,
/// a middle leg driven under her, for a target under her: at the target, kept
/// off her middle.
pub fn stab_disc(m: &Monster, leg: usize, at: V3) -> V3 {
    let middle = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
    if super::middle_leg(leg) && under_her(m, at) {
        let from = V3::new(at.x.sub(m.pos.x), Fx::ZERO, at.z.sub(m.pos.z));
        let far = math::wide_flat_len(from);
        let keep = Knob::StabFar.fx();
        if far.raw() >= keep.raw() {
            return V3::new(at.x, Fx::ZERO, at.z);
        }
        let dir = if far.raw() > 0 {
            math::wide_normalized(from)
        } else {
            V3::from_turns(m.yaw.add(crate::math::QUARTER_TURN))
        };
        return middle.add(dir.scale(keep));
    }
    let home = super::legs::home(m, leg);
    let out = V3::new(home.x.sub(m.pos.x), Fx::ZERO, home.z.sub(m.pos.z));
    let far = math::wide_flat_len(out);
    if far.raw() <= 0 {
        return home;
    }
    middle.add(math::wide_normalized(out).scale(far.sub(Knob::StabNear.fx()).max(Fx::ZERO)))
}

/// **Which of her legs stabs at `at`**: the sound leg whose disc is nearest
/// it -- a middle leg driven inward if `at` is under her -- and not one
/// already used in this flurry. A broken leg does not stab. With its disc.
pub fn stab_leg(m: &Monster, at: V3) -> Option<(usize, V3)> {
    let used = (m.own[body::STAB] as u32 >> 16) & 0xFF;
    (0..LEG_COUNT)
        .filter(|leg| !m.broken(shin_part(*leg)) && used & 1 << leg == 0)
        .map(|leg| (leg, stab_disc(m, leg, at)))
        .min_by_key(|(_, disc)| math::wide_flat_dist(*disc, at).raw())
}

/// Is a stab worth throwing at `at`: some sound leg's disc within reach of
/// it, `StabSlack` to spare?
pub fn stab_reaches(m: &Monster, at: V3) -> bool {
    let a = m.sp().attack(STAB);
    let reach = a
        .hit_radius
        .add(crate::tuning::body_radius())
        .add(Knob::StabSlack.fx());
    stab_leg(m, at).is_some_and(|(_, disc)| math::wide_flat_dist(disc, at).raw() <= reach.raw())
}

/// The web anchor a line would go to: the one that puts her target in the
/// lane on the way, furthest along it -- she crosses the cave *through* you.
/// Read off the arena's sites named `anchor`.
pub fn anchor_for(m: &Monster, mind: &Mind) -> Option<V3> {
    let lane = Knob::LineLane.fx().add(crate::tuning::body_radius());
    let target = m.brain.seen;
    let near = math::wide_flat_dist(target, m.pos);
    // Which anchors her frame found a clear line to (`fight::anchors_clear`).
    let clear = (m.own[body::STAB] as u32) >> 24;
    mind.ground
        .sites
        .iter()
        .filter(|s| s.name == "anchor")
        .take(8)
        .enumerate()
        .filter(|(k, _)| clear & 1 << k != 0)
        .map(|(_, s)| s.at(Fx::ZERO).0)
        .filter(|a| {
            let len = math::wide_flat_dist(*a, m.pos);
            let gap = math::flat_segment_gap(target, m.pos, *a);
            gap.raw() <= lane.raw() && near.raw() < len.raw()
        })
        .max_by_key(|a| math::wide_flat_dist(*a, m.pos).raw())
}

/// **As a move commits** (`FightDecl::commit`): which leg stabs and where,
/// how many stabs follow; which anchor a line goes to; an enraged lunge's
/// shorter windup.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    match kind {
        STAB => {
            m.own[body::STAB] &= !0x00FF_FFFF;
            let lead = m.lead_point(m.sp().attack(STAB).startup);
            let (leg, disc) = stab_leg(m, lead).unwrap_or((0, lead));
            let most = if fight::enraged(m) {
                Knob::EnrageFlurry.raw()
            } else {
                Knob::Flurry.raw()
            }
            .max(1);
            // How many follow: a draw on the brain's own dice, which the brain
            // advanced choosing this move.
            let follow = (m.brain.rng % most as u32) as i32;
            m.own[body::STAB] = (m.own[body::STAB] & !0x00FF_FFFF)
                | (leg as i32 + 1)
                | follow << 8
                | 1 << (16 + leg);
            m.aim_at(disc);
        }
        LUNGE if fight::enraged(m) => {
            m.doing = Doing::Startup {
                kind,
                left: Knob::EnrageLunge.raw().max(1) as u16,
            };
        }
        WEB_LINE => {
            if let Some(a) = anchor_for(m, mind) {
                m.aim_at(a);
            }
        }
        _ => {}
    }
}

/// **The screech's appetite, worked out with the world in hand**: the sum of
/// her brood's distances from her over the cap, times `RecallAppetite`, and
/// `ThreatAppetite` for every fighter near a living sac. Kept on her body for
/// the brain to read ([`appetite`] has no critters to look at).
pub fn work_out_recall(m: &mut Monster, critters: &Critters, fighters: &[Player; MAX_PLAYERS]) {
    let cap = Knob::BroodCap.raw().max(1);
    let spread: i32 = critters
        .iter()
        .filter(|c| c.alive())
        .map(|c| math::wide_flat_dist(c.pos, m.pos).to_int())
        .sum();
    // Near a sac: within a few bodies of the floor under it.
    let near = crate::tuning::body_radius().mul(Fx::from_int(SAC_COUNT as i32));
    let threat = fighters
        .iter()
        .filter(|p| p.health > 0)
        .filter(|p| {
            (0..SAC_COUNT).any(|i| {
                let at = fight::sac_middle(m, i);
                m.own[body::SACS] & (1 << i) == 0
                    && math::wide_flat_dist(at, p.pos).raw() <= near.raw()
            })
        })
        .count() as i32;
    let score = spread * Knob::RecallAppetite.raw() / cap + threat * Knob::ThreatAppetite.raw();
    m.own[body::WEBBED] = (m.own[body::WEBBED] & 0xFF) | score.clamp(0, 0xFF_FFFF) << 8;
}

// ---------------------------------------------------------------------------
// Her brood
// ---------------------------------------------------------------------------

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

/// The mother, if she owns this pack and stands.
fn mother<'a>(pack: &Pack, herd: &'a Herd) -> Option<&'a Monster> {
    herd.get(pack.owner as usize)
        .and_then(|m| m.as_ref())
        .filter(|m| m.alive())
}

/// Is she screaming: the screech's windup or scream, before the slam?
fn screeching(m: &Monster) -> bool {
    matches!(
        m.doing,
        Doing::Startup { kind: SCREECH, .. } | Doing::Active { kind: SCREECH, .. }
    )
}

impl PackMind for Brood {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        // Called home: nobody bites.
        if mother(look.pack, look.herd).is_some_and(screeching) {
            return 0;
        }
        GNAWER.appetite(look, i, m, a)
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        GNAWER.frame(pack, critters, herd, frame);
        let Some(mom) = mother(pack, herd).copied() else {
            return;
        };
        // The screech: every broodling breaks off what it was winding up.
        if screeching(&mom) {
            for c in critters.iter_mut() {
                if c.alive() && c.state == is::STARTUP {
                    c.state = is::PROWL;
                    c.timer = 0;
                    give_back(pack, c);
                }
            }
        }
        // The Brood guard: every broodling near her turns on whoever hit a
        // sac.
        let named = fight::guarded(&mom);
        if let Some(who) = named {
            let near = Knob::GuardRadius.fx();
            for c in critters.iter_mut() {
                if c.alive() && math::wide_flat_dist(c.pos, mom.pos).raw() <= near.raw() {
                    c.target = who as u8;
                }
            }
        }
        // Tokens: the pack's own, a token more on whoever the guard names, as
        // many as `RootedTokens` on a webbed target, and the gnawers' own
        // extra for two hunters.
        let base = pack.sp().pack_raw(PackKnob::Tokens).max(0);
        let webbed = mom.own[body::WEBBED] & 0b11 != 0;
        let guard = if named.is_some() {
            Knob::GuardTokens.raw().max(0)
        } else {
            0
        };
        let rooted = if webbed {
            (Knob::RootedTokens.raw() - base).max(0)
        } else {
            0
        };
        let two = pack.seen.iter().filter(|s| s.alive).count() >= 2;
        let coop = if two {
            gnawers::knob(gnawers::Knob::CoopTokens).max(0)
        } else {
            0
        };
        let extra = (guard.max(rooted) + coop).clamp(0, crate::pack::MAX_TOKENS as i32);
        pack.boost = extra as u8;
        pack.boost_left = if extra > 0 { 2 } else { 0 };
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let plain = GNAWER.steer(look, i, want);
        let c = &look.critters[i];
        let Some(mom) = mother(look.pack, look.herd) else {
            return plain;
        };
        if !screeching(mom) || !c.alive() || c.has(flag::TOKEN) {
            return plain;
        }
        // Called home: to a ring round her, each on the bearing it is on.
        let ring = Knob::RecallRadius.fx();
        let from = V3::new(c.pos.x.sub(mom.pos.x), Fx::ZERO, c.pos.z.sub(mom.pos.z));
        let dir = if math::wide_flat_len(from).raw() > 0 {
            math::wide_normalized(from)
        } else {
            V3::from_turns(mom.yaw)
        };
        let to = V3::new(mom.pos.x, Fx::ZERO, mom.pos.z).add(dir.scale(ring));
        Steer {
            to,
            speed: crate::critter::stat_fx(look.sp, c.kind, crate::critter::CritterField::Run),
            face: Some(mom.pos),
        }
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
