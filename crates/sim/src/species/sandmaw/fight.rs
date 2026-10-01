//! What the Sandmaw does to the fighters, the floor and itself: its body
//! under the sand, what it heard and what it acted on, the sinkhole, the
//! spray and the tail, the swallow, the stand and the beach -- run once a
//! frame from its `FightDecl::frame` hook, and through its body hooks.
//!
//! **Its state** lives in two places, for the reason the Mireback's does:
//! the words the hit path reads (`FightDecl::hide`, `struck`, `buried`) are
//! on the body, [`body`]; everything else is in the hunt's lore, [`word`].
//!
//! **Everything it knows is in the lore and drawn from there** (§6): the
//! noise it attended at its last glance ([`word::ATTEND`]), which noises in
//! the ring reached it ([`word::HEARD`]), the path its head took
//! ([`word::SPINE`]) and what its last move from below acted on
//! ([`word::ACTED`]) -- so the wake, the rings in the sand, the fight report
//! and the tests all read one record.
//!
//! See `docs/design/creatures/sandmaw.md` §2, §4, §5 and §10.

use super::{
    BACK, BREACH, DIVE, GULLET, HEAD_PARTS, HOLD, Knob, LASH, RISE, SOUND, SPECIES, SPIT, SWALLOW,
    TEETH, UNDERTOW, VENTS, bones, mind,
};
use crate::aim;
use crate::arena::{Material, Terrain};
use crate::beast::{self, Pose, Rig};
use crate::fixed::Fx;
use crate::hazard::{self, Hazard, HazardDecl, HazardField, reach};
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Doing, Monster, mount_part, mount_slot};
use crate::noise::{self, NoiseKind};
use crate::perception::Perceiver;
use crate::species::{FightDecl, FightField, Mark, MarkLook, Marks, SpeciesId};
use crate::state::{self, Action, Hit, MAX_PLAYERS, QUARRY, World};
use crate::{DT, Input};

// ---------------------------------------------------------------------------
// The floor's kinds
// ---------------------------------------------------------------------------

/// The undertow's sinkhole: pulls a body standing in it toward its middle,
/// halves the walk and the dodge. The air is not sand: the shared floor pulls
/// and slows feet only, so a fighter who jumps keeps their takeoff.
pub const SINKHOLE: u8 = 0;

pub const HAZARDS: [HazardDecl; 1] = [HazardDecl::disc("sinkhole")
    .reaching(reach::FIGHTERS | reach::CRITTERS)
    .made_of(Material::Sand)];

// ---------------------------------------------------------------------------
// Its state
// ---------------------------------------------------------------------------

/// Its own words in the hunt's lore.
pub mod word {
    /// Bit 0: the round has been set up.
    pub const FLAGS: usize = 0;
    /// **The path its head took**, newest first: a point every
    /// `SegmentLength`, centimetres as `lore::halves`. What the wake is drawn
    /// along: the design document's spine of positions.
    pub const SPINE: usize = 1;
    pub const SPINE_LEN: usize = 10;
    /// How many of the spine's points are filled.
    pub const SPINE_FILL: usize = 11;
    /// **What it attended at its last glance that took anything**: what (low
    /// byte: [`super::fight::HEARD_IT`] or [`super::fight::FELT_IT`]), the
    /// noise's kind (next byte), who (next), and bit 24 if it was on rock.
    pub const ATTEND: usize = 12;
    /// Where, as `lore::halves` of centimetres.
    pub const ATTEND_AT: usize = 13;
    /// The frame it attended it.
    pub const ATTEND_FRAME: usize = 14;
    /// The frame a run of noises on rock began: zero when the last thing it
    /// attended was on sand. What `island_patience` counts from.
    pub const ISLAND_SINCE: usize = 15;
    /// The frame, as of the end of the last one: what the brain's hooks, which
    /// are not handed the world, read "now" from.
    pub const NOW: usize = 16;
    /// **Which noises in the ring reached it**: a bit per noise cell, decided
    /// the frame after each is made. Only these are drawn.
    pub const HEARD: usize = 17;
    /// Where a rise left from, under the sand, for the travel through its tell.
    pub const RISE_FROM: usize = 18;
    /// **What the move from below in progress acted on**: as [`ATTEND`], and
    /// where. The report's "marker covered the noise", and the fairness test.
    pub const ACTED: usize = 19;
    pub const ACTED_AT: usize = 20;
    /// **The swallow**: who plus one (low byte), gulps taken (next), whether
    /// this gulp's press is spent (bit 16).
    pub const HOLD: usize = 21;
    /// Frames held.
    pub const HOLD_T: usize = 22;
    /// The special key, per fighter: bit `i` held last frame (from inside),
    /// bit `8 + i` pressed this frame.
    pub const KEYS: usize = 23;
    /// How the last beach came about: [`super::route`].
    pub const ROUTE: usize = 24;
    /// The fighters the own-hit move in progress has struck: a bit each.
    pub const STRUCK: usize = 25;
    /// Where it circles while it takes the hunters in, as `lore::halves`.
    pub const GRACE_AT: usize = 26;
    /// **How the last swallow ended** (for the report): the gulp it was
    /// broken out on (low byte; zero for none), and why (next byte:
    /// [`super::end`]).
    pub const ENDED: usize = 27;
}

/// What it attended.
pub const HEARD_IT: u32 = 1;
pub const FELT_IT: u32 = 2;

/// Its four words on the body (`Monster::own`), which the hit path reads.
pub mod body {
    /// Under the sand, standing out of it, or beached: [`super::posture`].
    pub const POSTURE: usize = 0;
    /// Frames it has stood since it last came up.
    pub const UP_FOR: usize = 1;
    /// What happened to the body, for the frame hook to act on with the
    /// world: [`super::event`].
    pub const EVENTS: usize = 2;
    /// The route of a beach a body hook decided: [`super::route`].
    pub const ROUTE: usize = 3;
}

/// The values of [`body::POSTURE`].
pub mod posture {
    /// A tag, not a quantity.
    pub type Posture = i32;
    pub const BURIED: Posture = 0;
    pub const STANDING: Posture = 1;
    pub const BEACHED: Posture = 2;
}

/// The bits of [`body::EVENTS`].
pub mod event {
    /// One bit of the word.
    pub type Bit = i32;
    /// Gagged: struck in the open mouth through the swallow's tell.
    pub const GAGGED: Bit = 1;
    /// A teammate's blow on the head: let the swallowed go.
    pub const RESCUE: Bit = 1 << 1;
    /// The tooth ring broke this frame.
    pub const TEETH: Bit = 1 << 2;
    /// A root has kept it up this stand already.
    pub const ROOTED: Bit = 1 << 3;
    /// The rise in progress is the undertow's, at the sinkhole's middle.
    pub const FROM_UNDERTOW: Bit = 1 << 4;
    /// Riders have been thrown by the dive in progress.
    pub const THROWN: Bit = 1 << 5;
}

/// How a beach came about.
pub mod route {
    /// Struck past `interrupt_strain` while standing, or its poise broken.
    pub const BROKEN: u32 = 1;
    /// It rose into a stone.
    pub const STONE: u32 = 2;
    /// A breach that came down on rock, or was broken in the air.
    pub const LANE: u32 = 3;
    /// Gagged while past `cc_strain`.
    pub const GAG: u32 = 4;
    /// A knock-up while past `cc_strain`.
    pub const KNOCKED: u32 = 5;
}

/// How a swallow ended.
pub mod end {
    /// Broken out of from the inside, on a gulp.
    pub const ESCAPED: u32 = 1;
    /// A teammate hit the head.
    pub const RESCUED: u32 = 2;
    /// Spat out when the hold ran out.
    pub const SPAT: u32 = 3;
    /// Its broken teeth could not hold.
    pub const SLIPPED: u32 = 4;
    /// It was beached, or the swallowed went down.
    pub const DROPPED: u32 = 5;
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 2,
        noises: 8,
        objectives: 0,
        own: 7,
    },
    row: true,
    hazards: &HAZARDS,
    perceives,
    hears: true,
    collides: true,
    frame: Some(frame),
    appetite: Some(mind::appetite),
    prowl_to: Some(mind::prowl_to),
    commit: Some(mind::commit),
    hide: Some(hide),
    struck: Some(struck),
    landed: Some(landed),
    marks: Some(marks),
    shown: Some(shown),
    buried: Some(buried),
    clip: Some(clip),
    hearing: Some(hearing),
    from_inside: Some(from_inside),
    radius: Some(radius),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// Reading it
// ---------------------------------------------------------------------------

pub fn posture_of(m: &Monster) -> i32 {
    m.own[body::POSTURE]
}

/// Under the sand: no body, and the wake is what you see.
pub fn under(m: &Monster) -> bool {
    posture_of(m) == posture::BURIED && m.alive()
}

/// Standing out of the sand.
pub fn standing(m: &Monster) -> bool {
    posture_of(m) == posture::STANDING && m.alive()
}

/// On its side on the sand.
pub fn beached(m: &Monster) -> bool {
    matches!(m.doing, Doing::Toppled { .. })
}

/// **Hungry**: below `HungerHealth` it hears a quarter further and loses
/// patience sooner.
pub fn hungry(m: &Monster) -> bool {
    below(m, Knob::HungerHealth.fx())
}

/// **Frantic**: below `FranticHealth` it cuts its own stand short.
pub fn frantic(m: &Monster) -> bool {
    below(m, Knob::FranticHealth.fx())
}

fn below(m: &Monster, share: Fx) -> bool {
    let full = Fx::from_int(m.sp().health());
    Fx::from_int(m.health).raw() < full.mul(share).raw()
}

/// Its head, flat on the floor: where it hears and feels from.
pub fn head_flat(m: &Monster) -> V3 {
    flat(m.head())
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn point(w: u32) -> V3 {
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}

fn word_of(at: V3) -> u32 {
    lore::halves(lore::to_cm(at.x), lore::to_cm(at.z))
}

/// Its mouth's open radius in the rise, now: the knob, or the smaller one
/// once the tooth ring is broken.
pub fn rise_radius(m: &Monster) -> Fx {
    if m.broken(TEETH) {
        Knob::RiseBroken.fx()
    } else {
        m.sp().attack(RISE).hit_radius
    }
}

/// [`FightDecl::radius`]: the broken tooth ring shrinks the rise for good.
fn radius(m: &Monster, kind: u8, r: Fx) -> Fx {
    if kind == RISE && m.broken(TEETH) {
        Knob::RiseBroken.fx()
    } else {
        r
    }
}

/// Is its mouth down where a blow can reach it?
pub fn mouth_down(m: &Monster) -> bool {
    matches!(
        m.doing,
        Doing::Startup { kind: SWALLOW, .. }
            | Doing::Active { kind: SWALLOW, .. }
            | Doing::Active { kind: HOLD, .. }
            | Doing::Toppled { .. }
    ) || (matches!(m.doing, Doing::Flinch { .. }) && m.own[body::EVENTS] & event::GAGGED != 0)
}

/// Are its spiracles shut? Through the whole of a dive, and under the sand.
pub fn vents_shut(m: &Monster) -> bool {
    matches!(m.doing.attacking(), Some(SOUND | DIVE)) || under(m)
}

/// **What it attended**, as its lore remembers it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attended {
    /// [`HEARD_IT`] or [`FELT_IT`].
    pub what: u32,
    pub kind: Option<NoiseKind>,
    pub who: u8,
    pub on_rock: bool,
    pub at: V3,
    pub frame: u32,
}

impl Attended {
    pub fn heard(&self) -> bool {
        self.what == HEARD_IT
    }

    pub fn felt(&self) -> bool {
        self.what == FELT_IT
    }

    fn pack(&self) -> u32 {
        self.what
            | (self.kind.map_or(0, |k| k as u32)) << 8
            | (self.who as u32) << 16
            | (self.on_rock as u32) << 24
    }

    fn unpack(w: u32, at: u32, frame: u32) -> Option<Attended> {
        let what = w & 0xFF;
        (what != 0).then(|| Attended {
            what,
            kind: NoiseKind::from_byte(((w >> 8) & 0xFF) as u8),
            who: ((w >> 16) & 0xFF) as u8,
            on_rock: w & 1 << 24 != 0,
            at: point(at),
            frame,
        })
    }
}

/// What it attended last, if anything.
pub fn attended(lore: &Lore) -> Option<Attended> {
    Attended::unpack(
        lore.word(word::ATTEND),
        lore.word(word::ATTEND_AT),
        lore.word(word::ATTEND_FRAME),
    )
}

/// What its move from below in progress acted on.
pub fn acted_on(lore: &Lore) -> Option<Attended> {
    Attended::unpack(lore.word(word::ACTED), lore.word(word::ACTED_AT), 0)
}

/// The frame, as its brain may know it.
pub fn now(lore: &Lore) -> u32 {
    lore.word(word::NOW).wrapping_add(1)
}

/// Who it is holding in its throat, if anyone.
pub fn holding(lore: &Lore) -> Option<usize> {
    let who = lore.word(word::HOLD) & 0xFF;
    (who != 0).then(|| who as usize - 1)
}

/// Gulps taken in the swallow in progress.
pub fn gulps(lore: &Lore) -> u32 {
    (lore.word(word::HOLD) >> 8) & 0xFF
}

/// The path its head took, newest first.
pub fn spine(lore: &Lore) -> impl Iterator<Item = V3> + '_ {
    let n = (lore.word(word::SPINE_FILL) as usize).min(word::SPINE_LEN);
    (0..n).map(move |i| point(lore.word(word::SPINE + i)))
}

/// Did it hear the noise in this cell of the ring?
pub fn heard_cell(lore: &Lore, cell: usize) -> bool {
    cell < 32 && lore.word(word::HEARD) & 1 << cell != 0
}

/// **Does it feel a body here?** Within its feel radius of its head, low
/// enough off the floor under it, and on sand: it feels weight through the
/// sand, and rock is not sand.
pub fn feels(m: &Monster, head: V3, pos: V3, ground: &Terrain) -> bool {
    let sp = m.sp();
    let near = math::wide_flat_dist(pos, head).raw() <= sp.fight_fx(FightField::FeelRadius).raw();
    if !near {
        return false;
    }
    let floor = ground.ground_under(pos);
    let low = pos.y.sub(floor).raw() <= sp.fight_fx(FightField::FeelHeight).raw();
    low && on_sand(ground, pos)
}

/// Is this point over sand -- not on rock, a stone or a shield?
pub fn on_sand(ground: &Terrain, at: V3) -> bool {
    !matches!(
        ground.material_under(at),
        Material::Rock | Material::Stone | Material::Wood
    )
}

/// **Its perception filter**: no bodies, except one it feels.
fn perceives(p: &Perceiver) -> bool {
    p.quarry.alive && feels(p.monster, flat(p.head), p.quarry.pos, p.scene.arena)
}

/// **How far it hears now**, times its row: nothing at all for the
/// breach's deafness, a quarter further hungry.
fn hearing(m: &Monster, _lore: &Lore) -> Fx {
    if let Doing::Recovery { kind: BREACH, left } = m.doing {
        let into = m.sp().attack(BREACH).recovery.saturating_sub(left) as i32;
        if into < Knob::BreachDeafness.raw() {
            return Fx::ZERO;
        }
    }
    if hungry(m) {
        Knob::HungerHearing.fx()
    } else {
        Fx::ONE
    }
}

/// **Where a rise at `at` can come up**: pushed off any rock until its own
/// radius clears it, so it never rises closer than that to rock. The push
/// is at most its radius, which is less than the bite's: the circle still
/// covers the noise. `None` if it cannot fit there at all -- a noise deep on
/// an island.
pub fn fit(m: &Monster, ground: &Terrain, at: V3) -> Option<V3> {
    let body = m.sp().fight_fx(FightField::BodyRadius);
    let mut p = flat(at);
    // A few passes, for a corner between two boxes.
    for _ in 0..3 {
        p = flat(ground.fence(p, body, Fx::ZERO).0);
    }
    let moved = math::wide_flat_dist(p, at);
    let inside = rise_radius(m).sub(m.sp().fight_fx(FightField::StepOver));
    let b = ground.bounds;
    let room = m.sp().margin();
    let in_bounds = p.x.raw() >= b.lo_x.add(room).raw()
        && p.x.raw() <= b.hi_x.sub(room).raw()
        && p.z.raw() >= b.lo_z.add(room).raw()
        && p.z.raw() <= b.hi_z.sub(room).raw();
    (moved.raw() <= inside.raw() && in_bounds && on_sand(ground, p)).then_some(p)
}

/// **Where it surfaces beside an island** to spit at a noise on it: the
/// nearest point to the noise its body fits, along the line from its head.
pub fn island_edge(m: &Monster, ground: &Terrain, at: V3) -> Option<V3> {
    let body = m.sp().fight_fx(FightField::BodyRadius);
    let from = head_flat(m);
    let to = flat(at);
    let span = math::wide_flat_dist(from, to);
    if span.raw() <= 0 {
        return None;
    }
    let dir = math::wide_normalized(to.sub(from));
    // Walk back from the noise toward the head until it fits.
    let step = body;
    let mut back = Fx::ZERO;
    while back.raw() < span.raw() {
        let p = to.sub(dir.scale(back));
        let pushed = flat(ground.fence(p, body, Fx::ZERO).0);
        if pushed == p && on_sand(ground, p) {
            return Some(p);
        }
        back = back.add(step);
    }
    None
}

/// **A trail**: the two newest noises a fighter made within
/// `TrailWindow` of each other, as a velocity, if they were moving faster
/// than `TrailSpeed`. The breach's one prediction.
pub fn trail(lore: &Lore, who: u8) -> Option<(V3, V3)> {
    let mut newest: Option<noise::Noise> = None;
    let mut before: Option<noise::Noise> = None;
    for n in noise::all(lore).filter(|n| n.who == who) {
        if newest.is_none_or(|b| n.born.wrapping_sub(b.born) as i32 > 0) {
            before = newest;
            newest = Some(n);
        } else if before.is_none_or(|b| n.born.wrapping_sub(b.born) as i32 > 0) {
            before = Some(n);
        }
    }
    let (a, b) = (before?, newest?);
    let gap = b.born.wrapping_sub(a.born) as i32;
    if gap <= 0 || gap > Knob::TrailWindow.raw() {
        return None;
    }
    let moved = flat(b.pos()).sub(flat(a.pos()));
    let vel = moved.scale(Fx::ONE.div(Fx::from_int(gap).mul(DT)));
    if math::wide_flat_len(vel).raw() <= Knob::TrailSpeed.fx().raw() {
        return None;
    }
    Some((flat(b.pos()), vel))
}

/// How many discs the spray's cone is drawn and tested as: a count, not a
/// size -- the cone's length and width are the knobs.
pub const SPIT_DISCS: usize = 4;
/// How many discs the tail's half-ring is drawn and tested as.
pub const LASH_DISCS: usize = 6;

/// **The spray's shape**: a run of discs along its facing from its head,
/// widening to `SpitSpread` at `SpitLength`. What the hit test reaches and
/// what is drawn: one function.
pub fn spit_discs(m: &Monster) -> [(V3, Fx); SPIT_DISCS] {
    let from = head_flat(m);
    let along = V3::from_turns(m.yaw);
    let length = Knob::SpitLength.fx();
    let spread = Knob::SpitSpread.fx();
    let n = Fx::from_int(SPIT_DISCS as i32);
    std::array::from_fn(|i| {
        // The middle of the i-th quarter of the cone, and wide enough there
        // to touch its neighbours and the cone's edge.
        let k = Fx::from_int(i as i32).add(math::half(Fx::ONE)).div(n);
        let d = length.mul(k);
        let r = spread.mul(k).add(length.div(n).mul(math::half(Fx::ONE)));
        (
            from.add(along.scale(d.sub(length.div(n).mul(math::half(Fx::ONE))))),
            r,
        )
    })
}

/// **The tail's sweep**: discs round a half-ring behind the hole, between
/// `LashInner` and `LashOuter`. What the hit test reaches and what is drawn.
pub fn lash_discs(m: &Monster) -> [(V3, Fx); LASH_DISCS] {
    let hole = flat(m.pos);
    let mid = math::half(Knob::LashInner.fx().add(Knob::LashOuter.fx()));
    let r = math::half(Knob::LashOuter.fx().sub(Knob::LashInner.fx()));
    let n = Fx::from_int(LASH_DISCS as i32);
    std::array::from_fn(|i| {
        // From its left flank, round behind it, to its right: a half turn
        // centred on straight back.
        let k = Fx::from_int(i as i32).add(math::half(Fx::ONE)).div(n);
        let turn = m
            .yaw
            .add(math::QUARTER_TURN)
            .add(math::half(Fx::ONE).mul(k));
        (hole.add(V3::from_turns(turn).scale(mid)), r)
    })
}

/// Is a body at `pos` inside one of these discs?
fn in_discs(discs: &[(V3, Fx)], pos: V3) -> bool {
    let body = crate::tuning::body_radius();
    discs
        .iter()
        .any(|(c, r)| math::wide_flat_dist(*c, pos).raw() <= r.add(body).raw())
}

// ---------------------------------------------------------------------------
// The body's hooks
// ---------------------------------------------------------------------------

/// **Which parts are under the sand**: every part whose box is wholly below
/// the floor. Buried, that is all of it; standing, the body behind the hole.
fn buried(m: &Monster, rig: &Rig) -> u64 {
    let sp = m.sp();
    let mut mask = 0u64;
    for i in 0..sp.parts.len() {
        if top_of(sp, rig, i).raw() < 0 {
            mask |= 1 << i;
        }
    }
    mask
}

/// The highest point of a part's box, in the world.
fn top_of(sp: &crate::species::Species, rig: &Rig, part: usize) -> Fx {
    let sh = sp.shape(part);
    let frame = rig.of(part);
    let mid = sh.min.add(sh.max).scale(math::half(Fx::ONE));
    let half = sh.max.sub(sh.min).scale(math::half(Fx::ONE));
    let centre = frame.local_to_world(mid);
    let up = frame.rot.r[1];
    centre
        .y
        .add(up.x.abs().mul(half.x))
        .add(up.y.abs().mul(half.y))
        .add(up.z.abs().mul(half.z))
}

/// Drawn only where it has a body.
fn shown(w: &World, slot: usize, part: usize) -> Fx {
    match w.monsters.get(slot).and_then(|m| m.as_ref()) {
        Some(m) if m.rig().there(part) => Fx::ONE,
        _ => Fx::ZERO,
    }
}

/// **Its posture picks its clip**: buried it swims whatever its speed,
/// standing it stands. The moves and the stock states play as anybody's.
fn clip(m: &Monster) -> Option<Pose> {
    if !matches!(m.doing, Doing::Prowl) {
        return None;
    }
    let sp = m.sp();
    Some(match posture_of(m) {
        posture::BURIED => beast::sample(
            sp,
            super::Clip::Swim as usize,
            Fx::from_raw(m.stride as i32),
        ),
        posture::STANDING => {
            beast::sample(sp, super::Clip::Stand as usize, Fx::from_raw(m.beat as i32))
        }
        _ => beast::sample(sp, super::Clip::Beached as usize, Fx::ONE),
    })
}

/// **What its hide is worth on a part this frame**: the spiracles shut
/// before a dive are hide; the tooth ring is out of reach unless its mouth
/// is down, and a blow into the open mouth through the swallow's tell is
/// worth half again.
fn hide(m: &Monster, part: usize) -> Fx {
    let soft = m.sp().vulnerability(part);
    if soft.raw() <= 0 {
        return Fx::ONE;
    }
    if VENTS.contains(&part) && vents_shut(m) {
        return Knob::VulnHide.fx().div(soft);
    }
    if part == TEETH {
        if !mouth_down(m) {
            return Fx::ZERO;
        }
        if matches!(m.doing, Doing::Startup { kind: SWALLOW, .. }) {
            return Knob::GagTeeth.fx();
        }
    }
    Fx::ONE
}

/// Over on its side: the big window.
fn beach(m: &mut Monster, why: u32) {
    m.doing = Doing::Toppled {
        left: m.sp().topple_frames(),
    };
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    m.strain = 0;
    m.poise = 0;
    m.own[body::ROUTE] = why as i32;
}

/// **A hit, after health and strain.** The swallow's rescue and gag, the
/// tooth ring, and a stand broken past `interrupt_strain` -- which beaches
/// it -- take their frame here before the shared ladder.
fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    // **The tooth ring takes every blow that reaches it here**, so the shared
    // ladder never counts one twice: broken, a flinch and the rise smaller
    // for good.
    let mut teeth_broke = false;
    if let Some(slot) = m.sp().break_slot(part) {
        if m.breaks[slot] > 0 {
            m.breaks[slot] = (m.breaks[slot] - dealt).max(0);
            if m.breaks[slot] == 0 {
                m.own[body::EVENTS] |= event::TEETH;
                teeth_broke = true;
            }
        }
    }
    // A teammate's blow on the head lets the swallowed go.
    if let Doing::Active { kind: HOLD, .. } = m.doing {
        if HEAD_PARTS.contains(&part) {
            m.own[body::EVENTS] |= event::RESCUE;
        }
        return true;
    }
    // A blow into the open mouth through the swallow's tell gags it -- and
    // if it was already reeling, puts it over.
    if matches!(m.doing, Doing::Startup { kind: SWALLOW, .. }) && HEAD_PARTS.contains(&part) {
        if m.strain >= m.cc_bar() {
            beach(m, route::GAG);
        } else {
            m.doing = Doing::Flinch {
                left: Knob::GagFrames.raw() as u16,
            };
            m.own[body::EVENTS] |= event::GAGGED;
        }
        return true;
    }
    if teeth_broke && !beached(m) {
        m.doing = Doing::Flinch {
            left: Knob::GagFrames.raw() as u16,
        };
        return true;
    }
    // **Broken while it stands**, or in the air over a breach: beached.
    let aloft = matches!(m.doing, Doing::Active { kind: BREACH, .. });
    if (standing(m) || aloft) && !beached(m) && m.strain >= m.interrupt_bar() {
        beach(m, if aloft { route::LANE } else { route::BROKEN });
        return true;
    }
    beached(m) || part == TEETH
}

// ---------------------------------------------------------------------------
// What its moves do besides hurt
// ---------------------------------------------------------------------------

fn landed(w: &mut World, slot: usize, victim: usize, kind: u8, guarded: bool) {
    if kind != SWALLOW || guarded || holding(&w.lore).is_some() {
        return;
    }
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    let p = &mut w.players[victim];
    if p.health <= 0 {
        return;
    }
    // Taken under. Held in the throat, on the ride's rules.
    state::put_inside(p, slot, &m, GULLET);
    p.action = Action::Free;
    // One frame longer than the hold itself, so the hold's own clock (in
    // `hold`) is what ends it, not the move's.
    m.doing = Doing::Active {
        kind: HOLD,
        left: Knob::HoldFrames.raw() as u16 + 1,
    };
    m.hit_used = true;
    w.lore.set_word(word::HOLD, victim as u32 + 1);
    w.lore.set_word(word::HOLD_T, 0);
    w.lore.set_word(word::ENDED, 0);
    w.monsters[slot] = Some(m);
}

/// A fighter in its throat pressed something. **The special is judged
/// against the gulp, and nothing else they press does anything.** One press
/// is read per gulp: the frame hook decides what it was worth.
fn from_inside(w: &mut World, _slot: usize, who: usize, input: Input) -> Input {
    let keys = w.lore.word(word::KEYS);
    let held = 1u32 << who;
    let down = input.has(Input::SPECIAL);
    let mut next = keys & !held;
    if down {
        next |= held;
        if keys & held == 0 {
            next |= 1 << (8 + who);
        }
    }
    w.lore.set_word(word::KEYS, next);
    Input { bits: 0, ..input }
}

// ---------------------------------------------------------------------------
// Once a frame
// ---------------------------------------------------------------------------

fn frame(w: &mut World) {
    for slot in 0..w.monsters.len() {
        if w.monsters[slot].is_some_and(|m| m.species == SpeciesId::SANDMAW) {
            creature(w, slot);
        }
    }
    w.lore.set_word(word::NOW, w.frame);
}

/// One frame of the creature's own rules.
fn creature(w: &mut World, slot: usize) {
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    if w.lore.word(word::FLAGS) & 1 == 0 {
        w.lore.set_word(word::FLAGS, 1);
        w.lore.set_word(word::GRACE_AT, word_of(m.pos));
        let head = head_flat(&m);
        for i in 0..word::SPINE_LEN {
            w.lore.set_word(word::SPINE + i, word_of(head));
        }
        w.lore.set_word(word::SPINE_FILL, 1);
    }
    let ground = w.terrain();
    attend(w, &m, &ground);
    hear(w, &m);
    trace(w, &m);
    act(w, slot, &mut m, &ground);
    hold(w, slot, &mut m);
    own_hits(w, &mut m, &ground);
    circle_in_grace(w, &mut m, &ground);
    w.monsters[slot] = Some(m);
}

/// **What it attended**, recorded on the frame it glanced: a body it felt,
/// or the noise it heard. A glance that took nothing new leaves the old
/// record, which it goes on circling.
fn attend(w: &mut World, m: &Monster, ground: &Terrain) {
    if m.brain.glance_left != m.glance_frames() {
        return;
    }
    let seen = m.brain.seen;
    let who = m.brain.target as usize;
    let head = head_flat(m);
    let mut got = None;
    if who < MAX_PLAYERS {
        let p = &w.players[who];
        if p.health > 0 && p.pos == seen && feels(m, head, p.pos, ground) {
            got = Some(Attended {
                what: FELT_IT,
                kind: None,
                who: who as u8,
                on_rock: false,
                at: flat(seen),
                frame: w.frame,
            });
        }
    }
    if got.is_none() {
        let window = m.glance_frames() as u32;
        got = noise::all(&w.lore)
            .filter(|n| w.frame.wrapping_sub(n.born) <= window && n.pos() == seen)
            .map(|n| Attended {
                what: HEARD_IT,
                kind: Some(n.kind),
                who: n.who,
                on_rock: !on_sand(ground, n.pos()),
                at: flat(n.pos()),
                frame: w.frame,
            })
            .next();
    }
    let Some(a) = got else { return };
    w.lore.set_word(word::ATTEND, a.pack());
    w.lore.set_word(word::ATTEND_AT, word_of(a.at));
    w.lore.set_word(word::ATTEND_FRAME, a.frame);
    if a.on_rock {
        if w.lore.word(word::ISLAND_SINCE) == 0 {
            w.lore.set_word(word::ISLAND_SINCE, w.frame.max(1));
        }
    } else {
        w.lore.set_word(word::ISLAND_SINCE, 0);
    }
}

/// **Which noises reached it**: each one judged the frame after it was
/// made, against where its head was and how far it hears now. Only these
/// are drawn in the sand.
fn hear(w: &mut World, m: &Monster) {
    let mut bits = w.lore.word(word::HEARD);
    let head = m.head();
    let mul = hearing(m, &w.lore);
    let made = w.frame.wrapping_sub(1);
    let cells = w.lore.noise_cells().len().min(32);
    for i in 0..cells {
        let Some(n) = noise::nth(&w.lore, i) else {
            bits &= !(1 << i);
            continue;
        };
        if n.born != made {
            continue;
        }
        let loud = noise::loudness(m.sp(), &n).mul(mul);
        if loud.raw() > math::wide_len(n.pos().sub(head)).raw() {
            bits |= 1 << i;
        } else {
            bits &= !(1 << i);
        }
    }
    w.lore.set_word(word::HEARD, bits);
}

/// **The path its head took**: a point every `SegmentLength` while it swims.
fn trace(w: &mut World, m: &Monster) {
    if !under(m) {
        return;
    }
    let head = head_flat(m);
    let last = point(w.lore.word(word::SPINE));
    if math::wide_flat_dist(head, last).raw() < Knob::SegmentLength.fx().raw() {
        return;
    }
    for i in (1..word::SPINE_LEN).rev() {
        let older = w.lore.word(word::SPINE + i - 1);
        w.lore.set_word(word::SPINE + i, older);
    }
    w.lore.set_word(word::SPINE, word_of(head));
    let fill = (w.lore.word(word::SPINE_FILL) + 1).min(word::SPINE_LEN as u32);
    w.lore.set_word(word::SPINE_FILL, fill);
}

/// Remember what the move from below in progress acted on: what it
/// attended, as it committed.
fn act_on(w: &mut World) {
    w.lore.set_word(word::ACTED, w.lore.word(word::ATTEND));
    w.lore
        .set_word(word::ACTED_AT, w.lore.word(word::ATTEND_AT));
}

/// **Its postures and its moves, frame by frame**: the rise travelling under
/// its tell and standing at its hit, the sinkhole, the beach, the dive.
fn act(w: &mut World, slot: usize, m: &mut Monster, ground: &Terrain) {
    let sp = m.sp();
    match m.doing {
        // ---- the rise: under the sand to the circle, then up through it ----
        Doing::Startup { kind: RISE, left } => {
            let a = sp.attack(RISE);
            if left == a.startup {
                w.lore.set_word(word::RISE_FROM, word_of(m.pos));
                if m.own[body::EVENTS] & event::FROM_UNDERTOW == 0 {
                    act_on(w);
                }
            }
            let from = point(w.lore.word(word::RISE_FROM));
            let to = m.aimed_at();
            let t = Fx::from_int((a.startup - left.min(a.startup)) as i32)
                .div(Fx::from_int(a.startup.max(1) as i32));
            // Under the sand to the circle, round any rock between.
            let body = sp.fight_fx(FightField::BodyRadius);
            m.pos = flat(ground.fence(math::lerp3(from, to, t), body, Fx::ZERO).0);
            m.speed = Fx::ZERO;
        }
        Doing::Active { kind: RISE, left } if left == sp.attack(RISE).active => {
            m.pos = flat(m.aimed_at());
            m.own[body::POSTURE] = posture::STANDING;
            m.own[body::UP_FOR] = 0;
            m.own[body::EVENTS] &= !(event::ROOTED | event::FROM_UNDERTOW | event::THROWN);
            rise_through(w, m);
        }
        Doing::Recovery { kind: RISE, left } if left == sp.attack(RISE).recovery && frantic(m) => {
            m.doing = Doing::Recovery {
                kind: RISE,
                left: left.min(Knob::FranticStand.raw() as u16),
            };
        }

        // ---- the breach: what it came for, and where it comes down ----
        Doing::Startup { kind: BREACH, left } if left == sp.attack(BREACH).startup => act_on(w),
        // **The arc is in the air**: over whatever lies in the lane, rock
        // included, at its own speed -- the lane drawn is the lane flown.
        Doing::Active { kind: BREACH, left } => {
            let a = sp.attack(BREACH);
            if left == a.active {
                w.lore.set_word(word::RISE_FROM, word_of(m.pos));
            }
            let from = point(w.lore.word(word::RISE_FROM));
            let gone = a
                .advance
                .mul(Fx::from_int((a.active - left.min(a.active) + 1) as i32))
                .mul(DT);
            m.pos = flat(from.add(V3::from_turns(m.yaw).scale(gone)));
            m.speed = a.advance;
        }
        Doing::Recovery { kind: BREACH, left } if left == sp.attack(BREACH).recovery => {
            // Down on rock it did not hear: beached across its own lane.
            let body = sp.fight_fx(FightField::BodyRadius);
            let (_, push) = ground.fence(m.pos, body, Fx::ZERO);
            if push != V3::ZERO || stone_near(w, m.pos, body) {
                beach(m, route::LANE);
            }
        }

        // ---- the undertow: the sinkhole, then a rise at its middle ----
        Doing::Startup {
            kind: UNDERTOW,
            left,
        } if left == sp.attack(UNDERTOW).startup => act_on(w),
        Doing::Active {
            kind: UNDERTOW,
            left,
        } if left == sp.attack(UNDERTOW).active => {
            let r = hazard::stat_fx(&SPECIES, SINKHOLE, HazardField::Radius);
            hazard::place(&mut w.lore, Hazard::disc(SINKHOLE, m.aimed_at(), r));
        }
        Doing::Recovery { kind: UNDERTOW, .. } => {
            m.doing = Doing::Startup {
                kind: RISE,
                left: sp.attack(RISE).startup,
            };
            m.hit_used = false;
            m.own[body::EVENTS] |= event::FROM_UNDERTOW;
            w.lore.set_word(word::RISE_FROM, word_of(m.pos));
        }

        // ---- the dive: riders thrown, and it is under ----
        Doing::Active { kind, left }
            if matches!(kind, SOUND | DIVE) && left == sp.attack(kind).active =>
        {
            throw_riders(w, slot, m);
            m.own[body::POSTURE] = posture::BURIED;
            m.own[body::UP_FOR] = 0;
        }

        // ---- the beach, however it came, and the dive that ends it ----
        Doing::Toppled { .. } if posture_of(m) != posture::BEACHED => {
            m.own[body::POSTURE] = posture::BEACHED;
            let why = match m.own[body::ROUTE] {
                0 => route::BROKEN,
                r => r as u32,
            };
            w.lore.set_word(word::ROUTE, why);
            m.own[body::ROUTE] = 0;
        }
        // A knock-up while it is reeling: over it goes.
        Doing::Stumble { .. } => beach(m, route::KNOCKED),
        Doing::Prowl if posture_of(m) == posture::BEACHED => {
            // Off the rock it came down on, if it did, before it dives.
            let body = sp.fight_fx(FightField::BodyRadius);
            m.pos = flat(ground.fence(m.pos, body, Fx::ZERO).0);
            m.doing = Doing::Startup {
                kind: DIVE,
                left: sp.attack(DIVE).startup,
            };
            m.hit_used = false;
            m.own[body::EVENTS] &= !event::THROWN;
        }
        _ => {}
    }
    if standing(m) {
        m.own[body::UP_FOR] += 1;
        // A root keeps it up: the dive put back, once a stand.
        if m.rooted > 0 && m.own[body::EVENTS] & event::ROOTED == 0 {
            m.own[body::EVENTS] |= event::ROOTED;
            m.own[body::UP_FOR] = (m.own[body::UP_FOR] - Knob::RootPush.raw()).max(0);
        }
    }
    if !matches!(m.doing, Doing::Flinch { .. }) {
        m.own[body::EVENTS] &= !event::GAGGED;
    }
}

/// Is a stone standing within `reach` of this point?
fn stone_near(w: &World, at: V3, reach: Fx) -> bool {
    crate::stones::gather(&w.players)
        .iter()
        .enumerate()
        .filter(|(slot, _)| !is_shield(w, *slot))
        .filter_map(|(_, s)| *s)
        .any(|s| {
            s.standing_height().raw() > 0
                && math::wide_flat_dist(flat(s.at), at).raw() <= s.radius().add(reach).raw()
        })
}

/// Is this slot of the field somebody's planted shield?
fn is_shield(w: &World, slot: usize) -> bool {
    let owner = slot / crate::class::MAX_STRUCTURES;
    w.players
        .get(owner)
        .is_some_and(|p| matches!(p.mechanic, crate::class::Mechanic::Shield(_)))
}

/// **Up through the circle**: a stone in it beaches the worm; a planted
/// shield in it is thrown clear, still planted.
fn rise_through(w: &mut World, m: &mut Monster) {
    let at = flat(m.aimed_at());
    let r = rise_radius(m);
    if stone_near(w, at, r.add(Knob::StoneReach.fx())) {
        beach(m, route::STONE);
        return;
    }
    for p in w.players.iter_mut() {
        if let crate::class::Mechanic::Shield(crate::class::Shield::Planted { pos, weight }) =
            p.mechanic
        {
            let gap = flat(pos).sub(at);
            if math::wide_flat_len(gap).raw() > r.raw() {
                continue;
            }
            let away = if math::wide_flat_len(gap).raw() > 0 {
                math::wide_normalized(gap)
            } else {
                V3::from_turns(m.yaw)
            };
            let to = pos.add(away.scale(Knob::ShieldThrow.fx()));
            p.mechanic =
                crate::class::Mechanic::Shield(crate::class::Shield::Planted { pos: to, weight });
        }
    }
}

/// **The dive throws everyone off its back.**
fn throw_riders(w: &mut World, slot: usize, m: &mut Monster) {
    if m.own[body::EVENTS] & event::THROWN != 0 {
        return;
    }
    m.own[body::EVENTS] |= event::THROWN;
    let a = m.sp().attack(SOUND);
    for p in w.players.iter_mut() {
        if !p.aboard() || mount_slot(p.mount) != slot {
            continue;
        }
        let part = mount_part(p.mount);
        if m.sp().parts.get(part).is_some_and(|pt| pt.shape.hollow) {
            continue;
        }
        let at = m.world_of(mount_part(p.mount), p.local);
        let out = flat(at.sub(m.pos));
        let out = if math::wide_flat_len(out).raw() > 0 {
            math::wide_normalized(out)
        } else {
            V3::from_turns(m.yaw.add(math::QUARTER_TURN))
        };
        state::let_out(p, at, V3::ZERO);
        state::apply_hit(
            p,
            Hit {
                damage: Knob::RiderThrow.raw(),
                hitstun: a.hitstun,
                blockstun: 0,
                knockback: a.knockback,
                launch: a.launch,
                grabs: 0,
                by: QUARRY,
                dir: out,
                blocked: false,
                parried: false,
                interrupts: true,
            },
        );
    }
}

/// **The swallow, frame by frame**: a gulp every `GulpEvery`, each opening
/// a window to break out; a press outside it spends that gulp; spat out when
/// the hold runs out, at once if a teammate hits the head, at the first gulp
/// if the teeth are broken.
fn hold(w: &mut World, slot: usize, m: &mut Monster) {
    let Some(who) = holding(&w.lore) else {
        w.lore.set_word(word::KEYS, w.lore.word(word::KEYS) & 0xFF);
        return;
    };
    let who = who.min(MAX_PLAYERS - 1);
    let keys = w.lore.word(word::KEYS);
    let pressed = keys & 1 << (8 + who) != 0;
    w.lore.set_word(word::KEYS, keys & 0xFF);
    let still = matches!(m.doing, Doing::Active { kind: HOLD, .. });
    if w.players[who].health <= 0 || !still {
        release(w, slot, m, end::DROPPED, 0);
        return;
    }
    if m.own[body::EVENTS] & event::RESCUE != 0 {
        m.own[body::EVENTS] &= !event::RESCUE;
        release(w, slot, m, end::RESCUED, gulps(&w.lore));
        m.doing = Doing::Flinch {
            left: Knob::GagFrames.raw() as u16,
        };
        return;
    }
    let t = w.lore.word(word::HOLD_T) + 1;
    w.lore.set_word(word::HOLD_T, t);
    let every = Knob::GulpEvery.raw().max(1) as u32;
    let window = Knob::GulpWindow.raw().max(0) as u32;
    let mut hold = w.lore.word(word::HOLD);
    let mut gulped = (hold >> 8) & 0xFF;
    // A gulp: the contraction, its damage, and its window opening.
    if t % every == 0 {
        gulped += 1;
        hold = (hold & !(0xFF << 8)) | gulped.min(0xFF) << 8;
        w.players[who].wound(Knob::GulpDamage.raw());
        if m.broken(TEETH) {
            w.lore.set_word(word::HOLD, hold);
            release(w, slot, m, end::SLIPPED, gulped);
            m.doing = Doing::Recovery {
                kind: HOLD,
                left: m.sp().attack(HOLD).recovery,
            };
            return;
        }
    }
    // A new period begins as the last window closes: the press is fresh.
    if t % every == window {
        hold &= !(1 << 16);
    }
    let open = t >= every && t % every < window;
    if pressed && hold & 1 << 16 == 0 {
        if open {
            w.lore.set_word(word::HOLD, hold);
            release(w, slot, m, end::ESCAPED, gulped);
            m.doing = Doing::Flinch {
                left: Knob::GagFrames.raw() as u16,
            };
            return;
        }
        // Pressed outside the window: that gulp is gone.
        hold |= 1 << 16;
    }
    w.lore.set_word(word::HOLD, hold);
    if t >= Knob::HoldFrames.raw().max(1) as u32 {
        w.players[who].wound(Knob::SpitOutDamage.raw());
        release(w, slot, m, end::SPAT, 0);
        m.doing = Doing::Recovery {
            kind: HOLD,
            left: m.sp().attack(HOLD).recovery,
        };
    }
}

/// Let the swallowed out beside the hole.
fn release(w: &mut World, _slot: usize, m: &mut Monster, why: u32, on_gulp: u32) {
    let Some(who) = holding(&w.lore) else { return };
    w.lore.set_word(word::HOLD, 0);
    w.lore.set_word(word::HOLD_T, 0);
    w.lore.set_word(word::ENDED, on_gulp.min(0xFF) | why << 8);
    let side = V3::from_turns(m.yaw.add(math::QUARTER_TURN));
    let at = flat(m.pos).add(side.scale(Knob::SpitOutDistance.fx()));
    let p = &mut w.players[who.min(MAX_PLAYERS - 1)];
    if p.aboard() {
        state::let_out(p, at, V3::ZERO);
    }
    p.action = Action::Free;
}

/// **The spray and the tail hit what their discs reach**: the spray if
/// nothing solid is between its mouth and you, the tail if you are not
/// under it.
fn own_hits(w: &mut World, m: &mut Monster, ground: &Terrain) {
    let (kind, left) = match m.doing {
        Doing::Active { kind, left } if kind == SPIT || kind == LASH => (kind, left),
        Doing::Startup { kind, .. } if kind == SPIT || kind == LASH => {
            w.lore.set_word(word::STRUCK, 0);
            return;
        }
        _ => return,
    };
    let _ = left;
    let a = m.sp().attack(kind);
    let field = crate::stones::gather(&w.players);
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let scene = aim::Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &[None; crate::monster::MAX_MONSTERS],
        critters: &critters,
        arena: ground,
    };
    let mut struck = w.lore.word(word::STRUCK);
    // The spray comes down off the mouth onto the sand and skids along it:
    // what stops it is what stands on the sand in its way.
    let mouth = head_flat(m);
    let spray_from = V3::new(mouth.x, Knob::SpitFrom.fx(), mouth.z);
    for i in 0..MAX_PLAYERS {
        let p = fighters[i];
        if struck & 1 << i != 0 || p.health <= 0 || p.action.invulnerable() || p.aboard() {
            continue;
        }
        let reached = match kind {
            SPIT => {
                in_discs(&spit_discs(m), p.pos)
                    && p.pos.y.sub(ground.ground_under(p.pos)).raw() <= a.hit_high.raw()
                    && aim::line_clear(
                        spray_from,
                        aim::standing_middle(p.pos, p.hurt_height()),
                        &scene,
                    )
            }
            _ => {
                in_discs(&lash_discs(m), p.pos)
                    && p.pos.y.add(p.hurt_height()).raw() >= Knob::LashLow.fx().raw()
                    && p.pos.y.raw() <= Knob::LashHigh.fx().raw()
            }
        };
        if !reached {
            continue;
        }
        struck |= 1 << i;
        let away = flat(p.pos.sub(m.pos));
        let away = if math::wide_flat_len(away).raw() > 0 {
            math::wide_normalized(away)
        } else {
            V3::from_turns(m.yaw)
        };
        let facing_it =
            p.facing.dot(away.scale(Fx::ONE.neg())).raw() >= crate::tuning::guard_arc_cos().raw();
        let guarding = !a.unblockable && p.action.guarding() && facing_it;
        let victim = &mut w.players[i];
        state::apply_hit(
            victim,
            Hit {
                damage: a.damage,
                hitstun: if kind == LASH {
                    Knob::LashStagger.raw() as u16
                } else {
                    a.hitstun
                },
                blockstun: a.blockstun,
                knockback: a.knockback,
                launch: a.launch,
                grabs: 0,
                by: QUARRY,
                dir: away,
                blocked: guarding,
                parried: false,
                interrupts: true,
            },
        );
        if kind == SPIT && !guarding {
            victim.slow(Knob::SpitSlowFrames.raw() as u16, Knob::SpitSlow.fx());
        }
        m.hit_used = true;
    }
    w.lore.set_word(word::STRUCK, struck);
}

/// **It opens circling**, while it takes the hunters in: round where it
/// started, at its swimming speed.
fn circle_in_grace(w: &mut World, m: &mut Monster, ground: &Terrain) {
    if m.brain.grace == 0 || !under(m) {
        return;
    }
    let centre = point(w.lore.word(word::GRACE_AT));
    let r = Knob::CircleRadius.fx();
    if r.raw() <= 0 {
        return;
    }
    let from = flat(m.pos).sub(centre);
    let at = if math::wide_flat_len(from).raw() > 0 {
        math::atan2_turns(from.z, from.x)
    } else {
        m.yaw
    };
    // Swim at a point a little way round the circle, at its swimming speed,
    // round any rock in the way as its own swim is.
    let speed = m.sp().gallop();
    let ahead = centre.add(V3::from_turns(at.add(Knob::SearchLook.fx())).scale(r));
    let to = flat(ahead.sub(m.pos));
    let dir = if math::wide_flat_len(to).raw() > 0 {
        math::wide_normalized(to)
    } else {
        V3::from_turns(m.yaw)
    };
    let body = m.sp().fight_fx(FightField::BodyRadius);
    let want = flat(m.pos).add(dir.scale(speed.mul(DT)));
    m.pos = flat(ground.fence(want, body, Fx::ZERO).0);
    m.yaw = math::atan2_turns(dir.z, dir.x);
    m.speed = speed;
    m.stride = m.stride.wrapping_add(
        speed
            .mul(DT)
            .div(m.sp().gait_stride())
            .raw()
            .clamp(0, 65535) as u16,
    );
}

// ---------------------------------------------------------------------------
// What it draws
// ---------------------------------------------------------------------------

/// **Everything it knows is drawn** (§6): the wake along the path its head
/// took and the fin over its head, the disc it feels within, a ring where
/// each noise it heard was made, and the shapes of the moves it tests itself
/// -- the sinkhole's warning, the spray, the tail.
fn marks(w: &World, out: &mut Marks) {
    let herd = w.monsters;
    for m in herd.iter().flatten() {
        if m.species != SpeciesId::SANDMAW {
            continue;
        }
        let sp = m.sp();
        if under(m) {
            let fill = (w.lore.word(word::SPINE_FILL) as usize).min(word::SPINE_LEN);
            let tall = Knob::WakeHeight.fx();
            for (i, at) in spine(&w.lore).enumerate() {
                let fade = Fx::from_int((fill - i) as i32).div(Fx::from_int(fill.max(1) as i32));
                out.push(Mark {
                    at,
                    radius: Knob::WakeWidth.fx(),
                    height: tall.mul(fade),
                    look: MarkLook::Sand,
                    progress: Fx::ONE,
                });
            }
            out.push(Mark {
                at: head_flat(m),
                radius: math::half(math::half(Knob::WakeWidth.fx())),
                height: Knob::FinHeight.fx(),
                look: MarkLook::Fin,
                progress: Fx::ONE,
            });
            out.push(Mark {
                at: head_flat(m),
                radius: sp.fight_fx(FightField::FeelRadius),
                height: Fx::ZERO,
                look: MarkLook::Feel,
                progress: Fx::ONE,
            });
        }
        let shown = Knob::HeardShown.raw().max(1);
        for i in 0..w.lore.noise_cells().len() {
            let Some(n) = noise::nth(&w.lore, i) else {
                continue;
            };
            let age = w.frame.wrapping_sub(n.born) as i32;
            if !heard_cell(&w.lore, i) || age >= shown || age < 0 {
                continue;
            }
            out.push(Mark {
                at: flat(n.pos()),
                radius: Knob::WakeWidth.fx(),
                height: Fx::ZERO,
                look: MarkLook::Heard,
                progress: Fx::from_int(age).div(Fx::from_int(shown)),
            });
        }
        let progress = |kind: u8, left: u16| {
            let a = sp.attack(kind);
            Fx::from_int((a.startup - left.min(a.startup)) as i32)
                .div(Fx::from_int(a.startup.max(1) as i32))
        };
        match m.doing {
            Doing::Startup {
                kind: UNDERTOW,
                left,
            } => out.push(Mark {
                at: flat(m.aimed_at()),
                radius: hazard::stat_fx(&SPECIES, SINKHOLE, HazardField::Radius),
                height: Fx::ZERO,
                look: MarkLook::Warning,
                progress: progress(UNDERTOW, left),
            }),
            Doing::Startup { kind, left } | Doing::Active { kind, left }
                if kind == SPIT || kind == LASH =>
            {
                let p = if matches!(m.doing, Doing::Active { .. }) {
                    Fx::ONE
                } else {
                    progress(kind, left)
                };
                let mut discs = [(V3::ZERO, Fx::ZERO); LASH_DISCS];
                let n = if kind == SPIT {
                    discs[..SPIT_DISCS].copy_from_slice(&spit_discs(m));
                    SPIT_DISCS
                } else {
                    discs = lash_discs(m);
                    LASH_DISCS
                };
                for (at, r) in discs.iter().take(n) {
                    out.push(Mark {
                        at: *at,
                        radius: *r,
                        height: Fx::ZERO,
                        look: MarkLook::Warning,
                        progress: p,
                    });
                }
            }
            _ => {}
        }
    }
}

/// The parts a rider stands on, for the tools that put somebody on its back.
pub const fn back() -> &'static [usize] {
    &BACK
}

/// The bone the column stands on, for tools.
pub const HOLE_BONE: usize = bones::ROOT;
