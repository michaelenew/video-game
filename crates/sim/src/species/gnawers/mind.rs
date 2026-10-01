//! The Gnawers' mind: what differs from the generic pack (gnawers.md §5).
//!
//! Every method of [`PackMind`] it overrides is a rule from the document:
//!
//! - **appetite**: the bite and the maul at the range and bearing the move
//!   table says, on the target's own level; the hamstring **only from the
//!   target's rear third**; the scramble only beside a platform a fighter is
//!   standing on; the howl only when the pack has just lost one, its target is
//!   set up, or the Big One is below half -- and every attack nothing at all
//!   while the pack is afraid or a pile-on has suspended the tokens.
//! - **frame**: the pile-on, called on the glance that sees a fighter slowed
//!   or staggered -- every gnawer within `PileonRadius` winds up at once, one
//!   `PileonStagger` frames after the last, so a dodge answers it; the howl's
//!   rally; the clocks; who is treed; and, in coop, the extra bodies and the
//!   extra token.
//! - **steer**: the pile-on's ring tightening as it winds up, the hamstring's
//!   scuttle to the heels, the Big One's hang-back and its short scatter, the
//!   diggers' walk to a stone, a ring that widens under a treed fighter and
//!   stops closing, and a gnawer that has scrambled up going straight for
//!   whoever is up there.
//! - **landed**: the hamstring's slow and latch; three of a pile-on is a
//!   knockdown.
//! - **hurt**: the Big One's strain (a stumble, the gnawers' topple), and any
//!   hit cancelling its howl.
//! - **body**: the Big One rears to a fighter's height to howl.
//!
//! Its own state is the pack's memo ([`word`]), so the creature adds nothing
//! to the world.

use super::{DART, GNAW, GNAWER, HAMSTRING, HOWL, Knob, MAUL, PILE_ON, SCRAMBLE, knob, knob_fx};
use crate::critter::{Body, Critter, CritterField, Critters, MAX_CRITTERS, flag, is, stat_fx};
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::{Attack, Herd};
use crate::pack::{
    Look, Pack, PackKnob, PackMind, Steer, default_appetite, give_back, mood, ring_point,
};
use crate::state::{Action, MAX_PLAYERS, Player};

/// The pack's memo, word by word. Twelve words; every one is used.
pub mod word {
    /// Two clocks, sixteen bits each ([`super::Clock`]): frames left in the
    /// pile-on under way (tokens are suspended while it is non-zero), and
    /// frames before another may be called.
    pub const PILE_CLOCKS: usize = 0;
    /// Where the pile-on under way was called on, raw fixed point: the lane
    /// every leaper locks to.
    pub const PILE_X: usize = 1;
    /// Pile-on bites landed this pile-on, per fighter: sixteen bits each.
    pub const PILE_HITS: usize = 2;
    /// Two clocks: frames before the Big One may howl again, and frames since
    /// the pack last lost a body, counting down from `HowlAfterLoss`.
    pub const HOWL_CLOCKS: usize = 3;
    pub const PILE_Z: usize = 4;
    /// The Big One's strain: recent damage, bled each frame.
    pub const STRAIN: usize = 5;
    /// Frames each fighter's feet have been above `LeapReach`: sixteen bits
    /// each.
    pub const TREED: usize = 6;
    /// The stone being gnawed, as its index in the stone field plus one, or
    /// zero.
    pub const GNAW_STONE: usize = 7;
    /// Digger-frames put into it.
    pub const GNAW_WORK: usize = 8;
    /// Where it stands, raw fixed point: the diggers walk there.
    pub const GNAW_X: usize = 9;
    pub const GNAW_Z: usize = 10;
    /// Bits: [`super::COOP_DONE`], [`super::WAS_ROUTED`].
    pub const BITS: usize = 11;
}

/// The pack's four clocks, two to a memo word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clock {
    PileLeft,
    PileRest,
    HowlLock,
    SinceLoss,
}

impl Clock {
    const ALL: [Clock; 4] = [
        Clock::PileLeft,
        Clock::PileRest,
        Clock::HowlLock,
        Clock::SinceLoss,
    ];

    fn slot(self) -> (usize, usize) {
        match self {
            Clock::PileLeft => (word::PILE_CLOCKS, 0),
            Clock::PileRest => (word::PILE_CLOCKS, 1),
            Clock::HowlLock => (word::HOWL_CLOCKS, 0),
            Clock::SinceLoss => (word::HOWL_CLOCKS, 1),
        }
    }
}

fn clock(pack: &Pack, k: Clock) -> i32 {
    let (w, h) = k.slot();
    half(pack.memo[w], h)
}

fn set_clock(pack: &mut Pack, k: Clock, v: i32) {
    let (w, h) = k.slot();
    set_half(&mut pack.memo[w], h, v);
}

/// The coop bodies have been mustered.
const COOP_DONE: u32 = 1;
/// The pack was routed last frame: coming back, it howls for free.
const WAS_ROUTED: u32 = 2;

/// Bits of [`Critter::role`], the species' own byte.
pub mod role {
    /// Sent to dig at the stone being gnawed.
    pub const DIGGER: u8 = 1 << 0;
    /// Holding a fighter's calf: see `rules`.
    pub const LATCHED: u8 = 1 << 1;
    /// The Big One, knocked down by strain: its long flinch is a stumble.
    pub const STUMBLED: u8 = 1 << 2;
    /// Handed a token for a dart-bite and closing to its crouch: the tail is
    /// up and the body is coming. The high four bits count how long.
    pub const CLOSING: u8 = 1 << 3;
    /// The low four bits are flags; the high four count a latch's length, or
    /// a close's, in sixteen-frame steps.
    pub const LATCH_STEP: u8 = 1 << 4;
    /// The counter's bits.
    pub const STEPS: u8 = 0xF0;
}

pub struct Mind;

/// A sixteen-bit half of a memo word, per fighter.
fn half(word: i32, who: usize) -> i32 {
    (word as u32 >> (16 * (who & 1)) & 0xFFFF) as i32
}

fn set_half(word: &mut i32, who: usize, value: i32) {
    let shift = 16 * (who & 1);
    let kept = *word as u32 & !(0xFFFF << shift);
    *word = (kept | ((value.clamp(0, 0xFFFF) as u32) << shift)) as i32;
}

/// Is fighter `who` out of the pack's reach -- feet above `LeapReach` for
/// longer than `TreedAfter`?
pub fn treed(pack: &Pack, who: usize) -> bool {
    half(pack.memo[word::TREED], who) > knob(Knob::TreedAfter)
}

/// Is the Big One (or any critter) howling: winding the howl up or out?
pub fn howling(c: &Critter) -> bool {
    c.attacking() && c.act == HOWL
}

/// Is a critter holding a fighter's calf?
pub fn latched(c: &Critter) -> bool {
    c.role & role::LATCHED != 0 && c.alive()
}

/// Is the Big One down in its stumble -- the window?
pub fn stumbling(c: &Critter) -> bool {
    c.state == is::FLINCH && c.role & role::STUMBLED != 0
}

/// The stone being gnawed, if one is: its index in the stone field, and how
/// far through felling it the diggers are, nought to one. For the renderer's
/// cracks and the report.
pub fn gnawed(pack: &Pack) -> Option<(usize, Fx)> {
    let stone = pack.memo[word::GNAW_STONE];
    if stone <= 0 {
        return None;
    }
    let work = knob(Knob::GnawWork).max(1);
    Some((
        stone as usize - 1,
        Fx::ratio(pack.memo[word::GNAW_WORK].clamp(0, work), work),
    ))
}

/// Is a pile-on under way: tokens suspended, the ring closing?
pub fn piling(pack: &Pack) -> bool {
    clock(pack, Clock::PileLeft) > 0
}

/// Is fighter `who` treed, as far as the report and the renderer are
/// concerned. See [`treed`].
pub fn treed_now(pack: &Pack, who: usize) -> bool {
    treed(pack, who)
}

/// The fighter a critter is after, clamped to a real index.
fn target_of(c: &Critter) -> usize {
    (c.target as usize).min(MAX_PLAYERS - 1)
}

/// Is somebody set up for the pile-on: slowed, or on the floor?
fn set_up(pack: &Pack, who: usize) -> bool {
    let s = &pack.seen[who];
    s.alive && (s.slowed || s.staggered)
}

/// The arena solid a fighter at `at` is standing on top of, if one is and it
/// is low enough to scramble up.
fn scramble_top(look: &Look, at: V3) -> Option<crate::arena::Solid> {
    if at.y.raw() < Fx::ratio(1, 2).raw() || at.y.raw() > knob_fx(Knob::ScrambleTop).raw() {
        return None;
    }
    // Standing on it: the resolve holds a body a skin off a top.
    let skin = crate::arena::SKIN.add(crate::arena::SKIN);
    look.arena
        .solids()
        .find(|s| {
            at.x.raw() >= s.min.x.raw()
                && at.x.raw() <= s.max.x.raw()
                && at.z.raw() >= s.min.z.raw()
                && at.z.raw() <= s.max.z.raw()
                && at.y.sub(s.max.y).abs().raw() <= skin.raw()
        })
        .copied()
}

/// How far a point is from a box's footprint, in the floor plane.
pub(super) fn off_footprint(s: &crate::arena::Solid, at: V3) -> Fx {
    let dx = s.min.x.sub(at.x).max(at.x.sub(s.max.x)).max(Fx::ZERO);
    let dz = s.min.z.sub(at.z).max(at.z.sub(s.max.z)).max(Fx::ZERO);
    V3::new(dx, Fx::ZERO, dz).flat_len()
}

/// **Is the critter in front of its target**, inside `FrontArc` of the
/// facing the glance saw? The dart-bite and the maul commit only from there:
/// they are the moves you face, and it keeps their windups on the screen of
/// the fighter they are for (the report's hidden commits). Facing is
/// something the glance saw, so this reads nothing it could not see.
fn in_front(c: &Critter, seen: &crate::pack::Seen) -> bool {
    let from = V3::new(c.pos.x.sub(seen.pos.x), Fx::ZERO, c.pos.z.sub(seen.pos.z));
    let facing = V3::from_turns(Fx::from_raw(seen.facing as i32));
    from.flat_len().raw() > 0
        && facing.dot(from.normalized()).raw() >= knob_fx(Knob::FrontArc).raw()
}

/// Is the critter on the level its target stands on -- both on the floor, or
/// both on one top? A bite from below a platform at somebody on it is a bite
/// at the platform's side.
fn same_level(c: &Critter, at: V3) -> bool {
    c.pos.y.sub(at.y).abs().raw() <= Fx::ratio(1, 2).raw()
}

impl PackMind for Mind {
    fn appetite(&self, look: &Look, i: usize, m: crate::critter::CritterMove, a: &Attack) -> i32 {
        let pack = look.pack;
        let c = &look.critters[i];
        let who = target_of(c);
        let seen = &pack.seen[who];
        // Afraid -- scattered, routed, broken -- nothing attacks at all.
        if pack.mood != mood::HUNTING || !seen.alive {
            return 0;
        }
        // The pile-on suspends the tokens.
        if m.token && clock(pack, Clock::PileLeft) > 0 {
            return 0;
        }
        match m.kind {
            // The dart is handed out by the pack, in `frame`: a token first,
            // and the crouch once the body has closed to it (`close_in`).
            DART => 0,
            MAUL => {
                if !same_level(c, seen.pos) || treed(pack, who) || !in_front(c, seen) {
                    return 0;
                }
                default_appetite(look, i, m, a)
            }
            // Handed out by the pack, as the dart is: see `close_in`.
            HAMSTRING => 0,
            SCRAMBLE => {
                let Some(top) = scramble_top(look, seen.pos) else {
                    return 0;
                };
                if c.pos.y.raw() > Fx::ratio(1, 2).raw()
                    || off_footprint(&top, c.pos).raw() > knob_fx(Knob::ScrambleFrom).raw()
                {
                    return 0;
                }
                a.weight.max(1)
            }
            HOWL => {
                if !c.has(flag::LEADER) || clock(pack, Clock::HowlLock) > 0 {
                    return 0;
                }
                let health = crate::critter::stat(look.sp, c.kind, CritterField::Health);
                let desperate = (c.health as i32) * 2 < health;
                let worth = clock(pack, Clock::SinceLoss) > 0 || set_up(pack, who) || desperate;
                // From behind the ring, never at somebody's feet: the
                // move's own range says where.
                if worth {
                    default_appetite(look, i, m, a)
                } else {
                    0
                }
            }
            // The pile-on and the gnaw are the pack's to call, in `frame`
            // and in `rules`; no body decides on either.
            PILE_ON | GNAW => 0,
            _ => default_appetite(look, i, m, a),
        }
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        let sp = pack.sp();
        for k in Clock::ALL {
            let left = clock(pack, k);
            set_clock(pack, k, left - 1);
        }
        let bleed = knob(Knob::StrainBleed);
        pack.memo[word::STRAIN] = (pack.memo[word::STRAIN] - bleed).max(0);
        if clock(pack, Clock::PileLeft) == 0 {
            pack.memo[word::PILE_HITS] = 0;
        }

        // Treed: feet above what a leap reaches, counted while it lasts.
        let reach = knob_fx(Knob::LeapReach);
        for who in 0..MAX_PLAYERS {
            let s = pack.seen[who];
            let up = s.alive && s.pos.y.raw() > reach.raw();
            let was = half(pack.memo[word::TREED], who);
            set_half(
                &mut pack.memo[word::TREED],
                who,
                if up { was + 1 } else { 0 },
            );
        }

        // Coop: more bodies, and a token more, while two are hunting. The
        // bodies once, from the den; the token held on as a standing rally
        // underneath any howl's.
        let two = pack.seen.iter().filter(|s| s.alive).count() >= 2;
        if two && pack.memo[word::BITS] as u32 & COOP_DONE == 0 && pack.mood != mood::BROKEN {
            pack.memo[word::BITS] |= COOP_DONE as i32;
            for k in 0..knob(Knob::CoopGnawers).max(0) {
                let side = Fx::from_int(k - 1);
                let at = pack.home.add(V3::new(side, Fx::ZERO, Fx::ZERO));
                crate::pack::spawn(pack, critters, GNAWER, at, 0);
            }
        }
        let coop = if two {
            knob(Knob::CoopTokens).max(0)
        } else {
            0
        };
        if coop > 0 && pack.boost_left == 0 {
            pack.rally(coop as u8, u16::MAX);
        }

        // **Cornered at the den, a rout turns.** A fighter who follows it to
        // the mouth has stopped the regroup clock (`RegroupClear`), and one
        // who walks into it is fought: the survivors come out of the rout
        // where they stand, counted again against who is left, without the
        // free howl a regroup brings. Decided while building: the harness
        // found a routed pack that never fought was a free win at the den.
        if pack.mood == mood::ROUTED {
            let bay = knob_fx(Knob::CorneredAt);
            let cornered = pack.seen.iter().any(|s| {
                s.alive
                    && V3::new(s.pos.x.sub(pack.home.x), Fx::ZERO, s.pos.z.sub(pack.home.z))
                        .flat_len()
                        .raw()
                        <= bay.raw()
            });
            if cornered {
                pack.mood = mood::HUNTING;
                pack.mood_left = 0;
                pack.lost = 0;
                pack.mustered = critters.iter().filter(|c| c.alive()).count() as u8;
                pack.memo[word::BITS] &= !(WAS_ROUTED as i32);
            }
        }

        // Coming back from a rout, behind a howl, for free.
        let routed = pack.mood == mood::ROUTED;
        if pack.memo[word::BITS] as u32 & WAS_ROUTED != 0 && pack.mood == mood::HUNTING {
            set_clock(pack, Clock::HowlLock, 0);
            set_clock(pack, Clock::SinceLoss, knob(Knob::HowlAfterLoss));
        }
        if routed {
            pack.memo[word::BITS] |= WAS_ROUTED as i32;
        } else {
            pack.memo[word::BITS] &= !(WAS_ROUTED as i32);
        }

        // The howl, on the frame it comes out: the rally, and the scatter is
        // over.
        for c in critters.iter_mut() {
            if c.alive() && c.state == is::ACTIVE && c.act == HOWL && !c.has(flag::HIT_USED) {
                c.set(flag::HIT_USED, true);
                set_clock(pack, Clock::HowlLock, knob(Knob::HowlLockout));
                let base = sp.pack_raw(PackKnob::Tokens).max(0);
                let wanted = knob(Knob::HowlTokens).max(base) + coop;
                pack.rally(
                    (wanted - base).max(0) as u8,
                    knob(Knob::HowlFrames).max(0) as u16,
                );
                if pack.mood == mood::SCATTERED {
                    pack.mood = mood::HUNTING;
                    pack.mood_left = 0;
                }
            }
        }

        close_in(pack, critters, herd, frame);

        // **The pile-on**, on what the glance saw: somebody slowed or on the
        // floor, and the pack hunting and rested.
        if pack.mood != mood::HUNTING
            || clock(pack, Clock::PileLeft) > 0
            || clock(pack, Clock::PileRest) > 0
            || pack.grace > 0
        {
            return;
        }
        let Some(who) = (0..MAX_PLAYERS).find(|w| set_up(pack, *w) && !treed(pack, *w)) else {
            return;
        };
        pile_on(pack, critters, who);
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let sp = look.sp;
        let pack = look.pack;
        let c = &look.critters[i];
        if !c.alive() {
            return want;
        }
        let who = target_of(c);
        let seen = &pack.seen[who];
        let lead = look.lead(who);
        let run = stat_fx(sp, c.kind, CritterField::Run);
        let walk = stat_fx(sp, c.kind, CritterField::Walk);
        let hold = Steer {
            to: c.pos,
            speed: Fx::ZERO,
            face: want.face,
        };
        if latched(c) {
            return hold;
        }
        let away = |from: V3, dist: Fx| {
            let d = c.pos.sub(from);
            let d = V3::new(d.x, Fx::ZERO, d.z);
            let d = if d.flat_len().raw() == 0 {
                c.facing().scale(Fx::ONE.neg())
            } else {
                d.normalized()
            };
            from.add(d.scale(dist))
        };

        // Winding up: the pile-on closes to its ring as it crouches, and the
        // hamstring scuttles to the heels. Everything else holds still.
        if c.state == is::STARTUP {
            let tracking = c.timer > crate::critter::stat(sp, c.kind, CritterField::Lock) as u16;
            let face = tracking.then_some(lead).or(want.face);
            return match c.act {
                PILE_ON => {
                    let lane = V3::new(
                        Fx::from_raw(pack.memo[word::PILE_X]),
                        lead.y,
                        Fx::from_raw(pack.memo[word::PILE_Z]),
                    );
                    Steer {
                        to: away(lane, knob_fx(Knob::PileonRing)),
                        speed: run,
                        face: Some(lane),
                    }
                }
                // The scuttle: in at the heels.
                HAMSTRING => {
                    let back = V3::from_turns(Fx::from_raw(seen.facing as i32));
                    Steer {
                        to: lead.sub(back.scale(sim_body_radius())),
                        speed: walk,
                        face,
                    }
                }
                _ => Steer { face, ..want },
            };
        }
        if c.state != is::PROWL {
            return want;
        }

        // Closing to its windup, tail up: to the crouch in front, or round to
        // the heels.
        if c.role & role::CLOSING != 0 {
            let to = if c.act == HAMSTRING {
                let facing = V3::from_turns(Fx::from_raw(seen.facing as i32));
                lead.sub(facing.scale(knob_fx(Knob::HamstringFrom)))
            } else {
                dart_spot(c, seen, lead)
            };
            return Steer {
                to,
                speed: run,
                face: Some(lead),
            };
        }

        // The diggers go to the stone.
        if c.role & role::DIGGER != 0 && pack.memo[word::GNAW_STONE] > 0 {
            let stone = V3::new(
                Fx::from_raw(pack.memo[word::GNAW_X]),
                Fx::ZERO,
                Fx::from_raw(pack.memo[word::GNAW_Z]),
            );
            return Steer {
                to: away(stone, Fx::ONE),
                speed: run,
                face: Some(stone),
            };
        }

        // Up on a platform with its target: straight for it.
        if pack.mood == mood::HUNTING
            && seen.alive
            && c.pos.y.raw() > Fx::ratio(1, 2).raw()
            && same_level(c, seen.pos)
        {
            return Steer {
                to: away(lead, sp.attack(DART).ideal_range),
                speed: run,
                face: Some(lead),
            };
        }

        if c.has(flag::LEADER) {
            match pack.mood {
                // The Big One flinches back only a little: close, alone, and
                // a second from its pack coming back. The window.
                mood::SCATTERED => {
                    // A startled step back, `LeaderScatter` over the
                    // scatter's length, and no further.
                    let frames = sp.pack_raw(PackKnob::ScatterFrames).max(1);
                    let speed = knob_fx(Knob::LeaderScatter)
                        .mul(Fx::from_int(crate::TICK_HZ as i32))
                        .div(Fx::from_int(frames));
                    let now = c.pos.sub(lead);
                    let now = V3::new(now.x, Fx::ZERO, now.z).flat_len();
                    return Steer {
                        to: away(lead, now.add(knob_fx(Knob::LeaderScatter))),
                        speed: speed.min(walk),
                        face: Some(lead),
                    };
                }
                mood::HUNTING if seen.alive => {
                    // It comes in for somebody set up, or when the pack is
                    // too few to hide behind; otherwise it keeps its distance
                    // behind the ring (the generic answer).
                    let members = look
                        .critters
                        .iter()
                        .filter(|o| o.alive() && !o.has(flag::LEADER))
                        .count();
                    let few = members as i32 <= sp.pack_raw(PackKnob::LineBelow);
                    let near =
                        seen.pos.sub(c.pos).flat_len().raw() <= knob_fx(Knob::LeaderComesIn).raw();
                    if (set_up(pack, who) && near) || few {
                        // In from the front, where the maul is thrown from.
                        let front = V3::from_turns(Fx::from_raw(seen.facing as i32));
                        let spot = lead.add(front.scale(sp.attack(MAUL).ideal_range));
                        return Steer {
                            to: spot,
                            speed: run,
                            face: Some(lead),
                        };
                    }
                }
                _ => {}
            }
            return want;
        }

        // **Treed**: a wider ring round where it will come down, and no
        // closing in. They wait; waiting costs them nothing.
        if pack.mood == mood::HUNTING && treed(pack, who) {
            let ring = c.slot.min(crate::pack::MAX_RING as u8 - 1);
            let place = if c.slot == crate::critter::NO_SLOT {
                away(lead, Fx::ONE)
            } else {
                ring_point(pack, seen, ring)
            };
            return Steer {
                to: away_from(lead, place, knob_fx(Knob::TreedRadius)),
                speed: walk,
                face: Some(lead),
            };
        }
        want
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
        if blocked || parried {
            return;
        }
        let c = &mut critters[i];
        match c.act {
            HAMSTRING => {
                victim.slow(
                    knob(Knob::HamstringFrames).max(0) as u16,
                    knob_fx(Knob::HamstringSlow),
                );
                c.role |= role::LATCHED;
                c.role &= !(0xF * role::LATCH_STEP);
                c.target = who as u8;
            }
            PILE_ON => {
                let mut hits = pack.memo[word::PILE_HITS];
                let n = half(hits, who) + 1;
                set_half(&mut hits, who, n);
                pack.memo[word::PILE_HITS] = hits;
                let down = knob(Knob::PileonKnockdown);
                if down > 0 && n == down {
                    let frames = knob(Knob::KnockdownFrames).max(1) as u16;
                    victim.action = Action::Stagger { left: frames };
                    victim.stun_total = frames;
                }
            }
            _ => {}
        }
    }

    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        let sp = pack.sp();
        let c = &mut critters[i];
        // Any hit shakes a latch loose, and the scramble is knocked off by
        // the generic flinch.
        if c.state == is::FLINCH {
            c.role &= !role::LATCHED;
        }
        if !c.has(flag::LEADER) || !c.alive() {
            return;
        }
        // **Anything that lands cancels the howl**, and flinches it.
        if c.state == is::STARTUP && c.act == HOWL {
            give_back(pack, c);
            c.state = is::FLINCH;
            c.timer = knob(Knob::HowlFlinch).max(1) as u16;
            set_clock(pack, Clock::HowlLock, knob(Knob::HowlLockout));
        }
        // **Strain**: recent damage past the threshold knocks the Big One
        // down. Lower when it is hurt.
        pack.memo[word::STRAIN] += dealt.max(0);
        let health = crate::critter::stat(sp, c.kind, CritterField::Health);
        let line = if (c.health as i32) * 2 < health {
            knob(Knob::StrainDesperate)
        } else {
            knob(Knob::Strain)
        };
        if line > 0 && pack.memo[word::STRAIN] >= line {
            pack.memo[word::STRAIN] = 0;
            give_back(pack, c);
            c.state = is::FLINCH;
            c.timer = knob(Knob::StumbleFrames).max(1) as u16;
            c.role |= role::STUMBLED;
        }
    }

    fn died(&self, pack: &mut Pack, critters: &mut Critters, i: usize) {
        critters[i].role = 0;
        set_clock(pack, Clock::SinceLoss, knob(Knob::HowlAfterLoss));
    }

    fn body(&self, c: &Critter, plain: Body) -> Body {
        if c.has(flag::LEADER) && howling(c) {
            // Up on its hind legs: a fighter's height, and half as long.
            return Body {
                height: knob_fx(Knob::HowlHeight).max(plain.height),
                half_len: plain.half_len.mul(Fx::ratio(1, 2)),
                ..plain
            };
        }
        plain
    }
}

/// **The bites, as the pack hands them out.** At a body's decision point,
/// with a token free and the pack hunting, the pack hands it a token for the
/// bite its place calls for: **in front of its target, the dart-bite**;
/// **in its rear third, the hamstring**; at its side, nothing yet. Its tail
/// goes up and it closes from the ring -- to `DartFrom` in front, or to
/// `HamstringFrom` behind the heels -- and there the windup begins: the
/// dart's crouch, tracking until `Lock`, or the hamstring's scuttle. A body
/// that cannot get there in `DartGiveUp` frames, or whose target turns to
/// face it before a hamstring begins, gives the token back.
///
/// Why the close is its own step: the ring is at five metres, and the answer
/// to the crouch is a hit, which only means something if the crouch happens
/// where a hit can reach it. The tail up while it comes is the warning a slow
/// class answers by swinging before the crouch, and turning to face it is
/// the answer to a hamstring before it is thrown.
fn close_in(pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
    let sp = pack.sp();
    let give_up = (knob(Knob::DartGiveUp).max(16) / 16).min(15) as u8;
    let hunting = pack.mood == mood::HUNTING && pack.grace == 0;
    for i in 0..MAX_CRITTERS {
        let c = critters[i];
        if c.role & role::CLOSING == 0 {
            continue;
        }
        let who = target_of(&c);
        let seen = pack.seen[who];
        // Arrived is measured to where the glance saw the fighter, not to
        // the lead it steers by: a dodge projects the lead metres ahead, and
        // a crouch begun there is a crouch at nobody.
        let gap = crate::math::big_len(V3::new(
            seen.pos.x.sub(c.pos.x),
            Fx::ZERO,
            seen.pos.z.sub(c.pos.z),
        ));
        let steps = (c.role & role::STEPS) / role::LATCH_STEP;
        let behind = in_the_rear(&c, &seen);
        let ahead = in_front(&c, &seen);
        let body = &mut critters[i];
        let stop = |body: &mut Critter, pack: &mut Pack| {
            body.role &= !(role::CLOSING | role::STEPS);
            give_back(pack, body);
        };
        if !body.alive() || body.state != is::PROWL || !hunting || !seen.alive || piling(pack) {
            if body.state == is::PROWL || !body.alive() {
                stop(body, pack);
            } else {
                // Taken into something else (a pile-on): it keeps the token
                // until that move ends, and is no longer closing.
                body.role &= !(role::CLOSING | role::STEPS);
            }
            continue;
        }
        // Faced before it could begin, the back it was going for is gone: the
        // token goes back.
        if body.act == HAMSTRING && !behind {
            stop(body, pack);
            continue;
        }
        let from = if body.act == HAMSTRING {
            knob_fx(Knob::HamstringFrom)
        } else {
            knob_fx(Knob::DartFrom)
        };
        let there = gap.raw() <= from.add(crate::arena::SKIN).raw();
        // A dart crouches only in front: arrived anywhere else -- the fighter
        // turned while it came round -- it goes on coming round.
        if there && (body.act == HAMSTRING || ahead) {
            // There: the windup. Belly to the floor for the crouch -- it
            // stops where it is rather than sliding the last metre in on its
            // momentum, so the body you see crouch is the body still there to
            // be hit.
            body.role &= !(role::CLOSING | role::STEPS);
            if body.act == DART {
                body.vel = V3::new(Fx::ZERO, body.vel.y, Fx::ZERO);
            }
            body.state = is::STARTUP;
            body.timer = sp.attack(body.act).startup.max(1);
            body.set(flag::HIT_USED, false);
            continue;
        }
        if steps >= give_up {
            stop(body, pack);
            continue;
        }
        if body.clock % 16 == 0 {
            body.role += role::LATCH_STEP;
        }
    }
    if !hunting || piling(pack) {
        return;
    }
    // Handing out: one body at a time, on its own decision frame, while a
    // token is free.
    let every = pack.think_every().max(1);
    for i in 0..MAX_CRITTERS {
        if crate::pack::tokens_out(pack, critters, herd) >= pack.token_cap() {
            return;
        }
        let c = critters[i];
        let can = |m: u8| sp.kind(c.kind).moves.iter().any(|k| k.kind == m && k.token);
        if frame % every != i as u32 % every
            || !c.alive()
            || c.state != is::PROWL
            || c.has(flag::LEADER)
            || c.has(flag::TOKEN)
            || c.role & (role::DIGGER | role::LATCHED) != 0
        {
            continue;
        }
        let who = target_of(&c);
        let seen = pack.seen[who];
        if !seen.alive || treed(pack, who) || !same_level(&c, seen.pos) {
            continue;
        }
        let lead = crate::pack::lead_of(pack, who);
        let gap = crate::math::big_len(V3::new(lead.x.sub(c.pos.x), Fx::ZERO, lead.z.sub(c.pos.z)));
        let within = |m: u8| {
            let a = sp.attack(m);
            gap.sub(a.ideal_range).abs().raw() <= a.range_span.raw()
        };
        // **From in front, the dart; from behind, the hamstring.** Facing is
        // something the glance saw, so this reads nothing it could not see --
        // and it keeps a crouch on the screen of the fighter it is for (the
        // report's hidden commits).
        let act = if can(HAMSTRING) && in_the_rear(&c, &seen) && within(HAMSTRING) {
            HAMSTRING
        } else if can(DART) && !in_the_rear(&c, &seen) && within(DART) {
            // From the side as well: it comes round into the front to crouch
            // (`dart_spot`), so a ring pushed to a fighter's flanks by a wall
            // behind them still bites.
            DART
        } else {
            continue;
        };
        // Individuals are noisy: the same coin the generic pack throws.
        if pack.roll() % 4 == 0 {
            continue;
        }
        let body = &mut critters[i];
        body.set(flag::TOKEN, true);
        body.act = act;
        body.role = (body.role & !role::STEPS) | role::CLOSING;
    }
}

/// **Where a dart-bite crouches from**: `DartFrom` off the fighter, inside
/// the front arc, on the side the body is coming from -- straight in if it is
/// already in front, the arc's edge if it is out at a flank.
fn dart_spot(c: &Critter, seen: &crate::pack::Seen, lead: V3) -> V3 {
    let facing = V3::from_turns(Fx::from_raw(seen.facing as i32));
    let from = V3::new(c.pos.x.sub(seen.pos.x), Fx::ZERO, c.pos.z.sub(seen.pos.z));
    let dist = knob_fx(Knob::DartFrom);
    if from.flat_len().raw() == 0 {
        return lead.add(facing.scale(dist));
    }
    let from = from.normalized();
    let arc = knob_fx(Knob::FrontArc);
    if facing.dot(from).raw() >= arc.raw() {
        return lead.add(from.scale(dist));
    }
    // The edge of the arc on the body's side: the facing turned by the arc's
    // half-angle toward it. cos is the knob; sin is what squares with it.
    let side = V3::new(facing.z.neg(), Fx::ZERO, facing.x);
    let toward = if side.dot(from).raw() >= 0 {
        side
    } else {
        side.scale(Fx::ONE.neg())
    };
    // Halfway in from the edge to dead ahead, so that arriving there is
    // arriving in front.
    let cos = arc.add(Fx::ONE).mul(Fx::ratio(1, 2));
    let sin = Fx::ONE.sub(cos.mul(cos)).max(Fx::ZERO).sqrt();
    let edge = facing.scale(cos).add(toward.scale(sin));
    lead.add(edge.scale(dist))
}

/// Is the critter in its target's rear third, by the facing the glance saw?
fn in_the_rear(c: &Critter, seen: &crate::pack::Seen) -> bool {
    let from = V3::new(c.pos.x.sub(seen.pos.x), Fx::ZERO, c.pos.z.sub(seen.pos.z));
    let facing = V3::from_turns(Fx::from_raw(seen.facing as i32));
    from.flat_len().raw() > 0 && facing.dot(from.normalized()).raw() <= knob_fx(Knob::RearCos).raw()
}

/// **Call a pile-on at fighter `who`**: every gnawer within `PileonRadius`
/// of where the glance last saw it that is not already committed winds up,
/// nearest first, `PileonStagger` frames apart, all locked to one lane -- the
/// glance's lead on the fighter. What the pack does on seeing a fighter slowed
/// or on the floor (in `frame`); public so a screenshot or a test can call one.
pub fn pile_on(pack: &mut Pack, critters: &mut Critters, who: usize) {
    let sp = pack.sp();
    let at = pack.seen[who].pos;
    let radius = knob_fx(Knob::PileonRadius);
    // Every gnawer within reach of it that is not already committed, nearest
    // first. Ten at most; a fixed order for ties.
    let mut order = [(i32::MAX, 0usize); MAX_CRITTERS];
    let mut n = 0;
    for (i, c) in critters.iter().enumerate() {
        if !c.alive()
            || c.state != is::PROWL
            || c.has(flag::LEADER)
            || c.mounted()
            || !sp.kind(c.kind).moves.iter().any(|m| m.kind == PILE_ON)
        {
            continue;
        }
        let d = crate::math::big_len(V3::new(c.pos.x.sub(at.x), Fx::ZERO, c.pos.z.sub(at.z)));
        if d.raw() <= radius.raw() {
            order[n] = (d.raw(), i);
            n += 1;
        }
    }
    if n == 0 {
        return;
    }
    order[..n].sort_unstable();
    let a = sp.attack(PILE_ON);
    let step = knob(Knob::PileonStagger).max(0) as u16;
    for (k, (_, i)) in order[..n].iter().enumerate() {
        let c = &mut critters[*i];
        c.state = is::STARTUP;
        c.act = PILE_ON;
        c.timer = a.startup.max(1) + step * k as u16;
        c.target = who as u8;
        c.set(flag::HIT_USED, false);
    }
    let last = a.startup as i32 + step as i32 * (n as i32 - 1);
    let left = last + a.active as i32 + a.recovery as i32;
    set_clock(pack, Clock::PileLeft, left);
    set_clock(pack, Clock::PileRest, left + knob(Knob::PileonRest));
    // **One lane for all of them**: where the glance put the fighter
    // when it was called. They close on it and leap at it, and do not
    // follow: the heap lands where you were, so a dodge out of it is the
    // answer and the heap is the sweep target.
    let lane = crate::pack::lead_of(pack, who);
    pack.memo[word::PILE_X] = lane.x.raw();
    pack.memo[word::PILE_Z] = lane.z.raw();
    pack.memo[word::PILE_HITS] = 0;
}

/// A fighter's radius: how close behind the heels the scuttle aims.
fn sim_body_radius() -> Fx {
    crate::tuning::body_radius()
}

/// A point `dist` from `from`, on the side of it `toward` is.
fn away_from(from: V3, toward: V3, dist: Fx) -> V3 {
    let d = toward.sub(from);
    let d = V3::new(d.x, Fx::ZERO, d.z);
    if d.flat_len().raw() == 0 {
        return from.add(V3::new(dist, Fx::ZERO, Fx::ZERO));
    }
    from.add(d.normalized().scale(dist))
}

/// The kinds a stumble clears on recovery: called by `rules` every frame.
pub(super) fn settle(c: &mut Critter) {
    if c.state != is::FLINCH {
        c.role &= !role::STUMBLED;
    }
}
