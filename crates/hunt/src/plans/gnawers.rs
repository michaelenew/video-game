//! The Gnawers' hunter: what a person learns in the first ten minutes against
//! the pack (gnawers.md §9), played the way a person plays it -- seeing
//! everything `REACTION` frames late, and aiming with the crosshair on the
//! body (`sim::aim::look_onto_closely`), so it exercises the small-body aim
//! the creature exists to test rather than going round it.
//!
//! The plan, in the document's order:
//!
//! 1. **Find a back.** In the grace moment, walk to the nearest wall, bank,
//!    boulder or platform side and stand with your back to it.
//! 2. **Face the pack**: the centroid of the gnawers it can see.
//! 3. **Swing first.** At any gnawer that raises its tail within reach --
//!    before it crouches if the class is slow, in the crouch if it is fast.
//! 4. **Slowed or latched: dodge at once**, away from the centroid.
//! 5. **A gnawer dies: go for the Big One** with a dodge and unload for the
//!    rest of the scatter; then back to a wall.
//! 6. **Howl: hit the Big One** with whatever reaches it.
//! 7. **Never cross open floor with the pack behind**: going back to the wall
//!    is walking, facing them.
//!
//! What it does not do is read the pack's mind: it knows a token is held
//! because the tail is up, a bite is coming because the body is crouched,
//! and nothing about which body the pack will hand the next token to.

use sim::critter::{Critter, CritterField, MAX_CRITTERS, flag, is};
use sim::fixed::Fx;
use sim::math::atan2_turns;
use sim::species::gnawers;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Input, V3, World};

use crate::{Hands, Intent, Plan, REACTION, steer, turns_to_aim};

pub const POST: Intent = Intent("ToWall");
pub const HOLD: Intent = Intent("Hold");
pub const SWING: Intent = Intent("Swing");
pub const SHED: Intent = Intent("Shed");
pub const LEADER: Intent = Intent("Leader");
pub const WAIT: Intent = Intent("Wait");
pub const HOP: Intent = Intent("Hop");

/// A maul crouched this near is jumped.
const MAUL_NEAR: Fx = Fx::ratio(45, 10);
/// How long the jump is held: long enough to be over the lunge when it comes.
const HOP_HOLD: u32 = 20;

/// How far off a wall's face it stands: its own body and a little.
const OFF_WALL: Fx = Fx::ratio(7, 10);
/// Close enough to its post to stop walking to it.
const AT_POST: Fx = Fx::ratio(8, 10);
/// A solid counts as a back if it stands at least this high.
const BACK_HEIGHT: Fx = Fx::ratio(14, 10);
/// Frames between presses: a person does not mash.
const SWING_GAP: u32 = 8;
/// How long it keeps going at the Big One after a death it saw, in frames:
/// the scatter's sixty, and the reaction it took to see it.
const WINDOW: u32 = 60;
/// A class whose fastest poke winds up longer than this swings at a raised
/// tail before the crouch; faster, in the crouch (§9 step 3).
const SLOW_POKE: u16 = 9;
/// It dodges toward the Big One when it is further than this.
const DASH_FROM: Fx = Fx::ratio(45, 10);
/// A pile-on crouch this near is one to get out of.
const HEAP: Fx = Fx::from_raw(9 << 16);
/// A body this near a slowed fighter is on its heels: a latch.
const AT_HEEL: Fx = Fx::ratio(16, 10);
/// A raised tail this near is a crouch coming: the swing waits for it.
const WATCH_TAILS: Fx = Fx::from_raw(6 << 16);
/// How far it steps out to meet a raised tail or a crouch.
const STEP_IN: Fx = Fx::ratio(15, 10);
/// A gnawer this near is close enough to matter to where it stands.
const NEAR: Fx = Fx::ratio(35, 10);

/// One body as the hunter remembers seeing it.
#[derive(Clone, Copy, Default)]
struct Seen {
    at: V3,
    middle: V3,
    alive: bool,
    leader: bool,
    /// Tail up: it holds a token.
    tail: bool,
    /// Crouched: a windup it can see.
    crouching: bool,
    /// Which move the windup is, as its silhouette says: the howl's rear is
    /// unmistakable, and so is a crouch.
    act: u8,
    /// Down: flinched or stumbling.
    down: bool,
}

pub struct Gnawers {
    who: usize,
    memory: Vec<[Seen; MAX_CRITTERS]>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u32,
    dodge_left: u32,
    /// The point it stands at, back to something solid, and where that back
    /// faces out.
    post: Option<V3>,
    /// Frames of going at the Big One left.
    window: u32,
    /// How many it saw alive last frame, to see a death.
    was_alive: usize,
    /// Frames it has felt slowed: a person notices on a delay too.
    slowed_for: u32,
    /// Frames left holding the jump.
    hop_left: u32,
    rng: u32,
    /// Its class (`crate::class`).
    hands: Hands,
}

impl Gnawers {
    pub fn new(who: usize, seed: u32) -> Gnawers {
        Gnawers {
            who,
            memory: vec![[Seen::default(); MAX_CRITTERS]; REACTION + 1],
            at: 0,
            filled: 0,
            intent: WAIT,
            cooldown: 0,
            dodge_left: 0,
            post: None,
            window: 0,
            was_alive: 0,
            slowed_for: 0,
            hop_left: 0,
            rng: seed | 1,
            hands: Hands::new(who, seed),
        }
    }

    fn recall(&self) -> [Seen; MAX_CRITTERS] {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }

    fn roll(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }
}

/// The nearest place with a back to it: a point just off the face of the
/// nearest solid at least `BACK_HEIGHT` tall that stands on the floor, inside
/// the arena's bounds.
fn find_post(w: &World, from: V3) -> Option<V3> {
    let arena = w.arena();
    let b = arena.bounds;
    let mut best: Option<(i32, V3)> = None;
    for s in arena.solids() {
        if s.max.y.raw() < BACK_HEIGHT.raw() || s.min.y.raw() > Fx::ratio(1, 10).raw() {
            continue;
        }
        // The nearest point of its footprint, and out from it along the face.
        let x = from.x.clamp(s.min.x, s.max.x);
        let z = from.z.clamp(s.min.z, s.max.z);
        let out = V3::new(from.x.sub(x), Fx::ZERO, from.z.sub(z));
        if out.flat_len().raw() == 0 {
            continue;
        }
        let spot = V3::new(x, Fx::ZERO, z).add(out.normalized().scale(OFF_WALL));
        let inside = spot.x.raw() > b.lo_x.add(OFF_WALL).raw()
            && spot.x.raw() < b.hi_x.sub(OFF_WALL).raw()
            && spot.z.raw() > b.lo_z.add(OFF_WALL).raw()
            && spot.z.raw() < b.hi_z.sub(OFF_WALL).raw();
        let spot = if inside {
            spot
        } else {
            V3::new(
                spot.x.clamp(b.lo_x.add(OFF_WALL), b.hi_x.sub(OFF_WALL)),
                Fx::ZERO,
                spot.z.clamp(b.lo_z.add(OFF_WALL), b.hi_z.sub(OFF_WALL)),
            )
        };
        let d = spot.sub(from).flat_len().raw();
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, spot));
        }
    }
    best.map(|(_, s)| s)
}

/// The button for the auto. The Dual mage's two hands are two bars, and a
/// hand thrown alone runs one of them away until she burns (`sim::dual`), so
/// she throws the hand whose bar is lower, the way a person playing her
/// learns to. (Her dark auto is a lunge that carries her through a knee-high
/// body inside two metres -- `critcheck`'s "runs through it at 1 2 m" -- and
/// choosing hands by range instead burned her to death: the bars win.)
fn auto_button(me: &sim::state::Player) -> u16 {
    match sim::dual::bars(me) {
        Some((dark, light)) if dark.raw() < light.raw() => Input::LEFT,
        Some(_) => Input::RIGHT,
        None => Input::LEFT,
    }
}

/// The move a left click throws on the floor: the class's auto. Slot zero
/// everywhere but the Blood mage, whose auto is the fifth row of her table
/// (`sim::moves::blood`).
fn auto_slot(class: sim::Class) -> u8 {
    match class {
        sim::Class::BloodMage => sim::moves::blood::SWEEP,
        _ => 0,
    }
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

impl Plan for Gnawers {
    fn watch(&mut self, w: &World) {
        let sp = w.critters.sp();
        let seen = std::array::from_fn(|i| {
            let c: &Critter = &w.critters[i];
            Seen {
                at: c.pos,
                middle: c.body(sp).middle(),
                alive: c.alive(),
                leader: c.has(flag::LEADER),
                tail: c.has(flag::TOKEN),
                crouching: c.state == is::STARTUP,
                act: c.act,
                down: c.state == is::FLINCH,
            }
        });
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.window = self.window.saturating_sub(1);
        let me = w.players[self.who];
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let seen = self.recall();
        let alive = seen.iter().filter(|s| s.alive).count();
        // A death, seen: the scatter is the window.
        if alive < self.was_alive && self.filled > REACTION {
            self.window = WINDOW;
        }
        self.was_alive = alive;
        if self.post.is_none() {
            self.post = find_post(w, me.pos);
        }

        let dist = |s: &Seen| flat(s.at.sub(me.pos)).flat_len();
        let members: Vec<&Seen> = seen.iter().filter(|s| s.alive && !s.leader).collect();
        let leader = seen.iter().find(|s| s.alive && s.leader).copied();
        let centroid = if members.is_empty() {
            leader.map(|l| l.at)
        } else {
            let n = Fx::from_int(members.len() as i32);
            let sum = members.iter().fold(V3::ZERO, |acc, s| acc.add(flat(s.at)));
            Some(V3::new(sum.x.div(n), Fx::ZERO, sum.z.div(n)))
        };

        let poke = sim::moves::get(me.class, auto_slot(me.class));
        let reach = poke.reach.max(Fx::ONE);
        let free = me.action.actionable();

        // Where it looks, and the pitch that puts the crosshair on a body.
        let look_at = |at: V3, middle: V3| {
            let to = at.sub(me.pos);
            let yaw = atan2_turns(to.z, to.x);
            let aim = turns_to_aim(yaw.sub(me.carry_yaw));
            let pitch = sim::aim::look_onto_closely(me.pos, aim, me.aloft, middle);
            (yaw, aim, pitch)
        };
        let facing_pack = centroid.unwrap_or(me.pos.add(me.facing));
        let (pack_yaw, pack_aim, _) = look_at(facing_pack, facing_pack);

        // 4. **Slowed: dodge out of the ring** -- when the pile-on comes, and
        // at once if one is hanging off a calf. The leapers are locked to
        // where they were called on, so the dodge is timed at the crouches
        // rather than at the slow: thrown at the slow, it is spent before the
        // pile-on is called, and they leap at where it ends. (§9 says "at
        // once"; the harness found the timing, and the document says so.)
        if me.slowed > 0 {
            self.slowed_for += 1;
        } else {
            self.slowed_for = 0;
        }
        let piling = members
            .iter()
            .any(|s| s.crouching && s.act == gnawers::PILE_ON && dist(s).raw() < HEAP.raw());
        let at_heel = members.iter().any(|s| dist(s).raw() < AT_HEEL.raw());
        let shed = self.slowed_for as usize >= REACTION && at_heel;
        if (piling || shed) && self.dodge_left == 0 && free {
            self.intent = SHED;
            self.dodge_left = sim::tuning::dodge_frames() as u32 + 20;
            let away = centroid.map_or(me.facing.scale(Fx::ONE.neg()), |c| {
                let d = flat(me.pos.sub(c));
                if d.flat_len().raw() == 0 {
                    me.facing.scale(Fx::ONE.neg())
                } else {
                    d.normalized()
                }
            });
            // Out along the wall rather than into it, when it has one.
            let dodge = Input::aimed(steer(pack_yaw, away) | Input::SHIFT, pack_aim);
            return self.hands.leave(w, &me, away, dodge);
        }

        // **Jump the maul** (§2): a crouched Big One close by, winding the
        // heavy lunge at the legs. Every class's hop clears a metre; held,
        // so the feet are up when it arrives.
        if self.hop_left > 0 {
            self.hop_left -= 1;
            return Input::aimed(Input::SPACE, pack_aim);
        }
        if let Some(big) = leader {
            let mauling = big.crouching && big.act == gnawers::MAUL;
            if mauling && dist(&big).raw() < MAUL_NEAR.raw() && me.grounded && free {
                self.intent = HOP;
                self.hop_left = HOP_HOLD;
                return Input::aimed(Input::SPACE, pack_aim);
            }
        }

        // 5 and 6. **The Big One**, when it is the thing to hit: a death's
        // scatter, a howl winding up, a stumble, or a pack too thin to hide it.
        // A routed pack is running for its den, and it is seen to: after it,
        // before it comes back (§4).
        let running = w
            .pack
            .is_some_and(|p| matches!(p.mood, sim::pack::mood::ROUTED | sim::pack::mood::BROKEN));
        if let Some(big) = leader {
            let howling = big.crouching && big.act == gnawers::HOWL;
            let alone = members.is_empty();
            if self.window > 0 || howling || big.down || running || alone {
                self.intent = LEADER;
                let (yaw, aim, pitch) = look_at(big.at, big.middle);
                let gap = dist(&big);
                let body =
                    sim::critter::stat_fx(w.critters.sp(), gnawers::BIG_ONE, CritterField::Length)
                        .mul(Fx::ratio(1, 2));
                let strike = reach.add(body);
                if gap.raw() > strike.raw() {
                    let dash = if gap.raw() > DASH_FROM.raw() && self.dodge_left == 0 && free {
                        self.dodge_left = sim::tuning::dodge_frames() as u32 + 6;
                        Input::SHIFT
                    } else {
                        0
                    };
                    return Input::looking_at(
                        steer(yaw, flat(big.at.sub(me.pos))) | dash,
                        aim,
                        pitch,
                    );
                }
                if self.cooldown == 0 && free {
                    self.cooldown = SWING_GAP + self.roll() % 4;
                    let swing = Input::looking_at(auto_button(&me), aim, pitch);
                    // Down, or scattered and alone: the window is long.
                    let window = (big.down || self.window > 0).then_some(self.window as i32);
                    return self.hands.hit(w, &me, big.middle, swing, window);
                }
                return Input::looking_at(0, aim, pitch);
            }
        }

        // 3. **Swing first**: a raised tail in reach -- before the crouch for
        // a slow class, in it for a fast one -- and failing that, any body in
        // reach.
        let slow = poke.startup > SLOW_POKE;
        let in_reach = |s: &Seen| dist(s).raw() <= reach.add(Fx::ratio(1, 2)).raw();
        let step_in = |s: &Seen| dist(s).raw() <= reach.add(STEP_IN).raw();
        let threat = members
            .iter()
            .filter(|s| step_in(s) && s.tail && (slow || s.crouching))
            .min_by_key(|s| dist(s).raw())
            .copied()
            .or_else(|| {
                // A raised tail coming in is a crouch about to be thrown: keep
                // the swing for it rather than spend it on whatever is near.
                if members
                    .iter()
                    .any(|s| s.tail && dist(s).raw() < WATCH_TAILS.raw())
                {
                    return None;
                }
                // Any body in reach, the Big One included -- out of its maul,
                // which the hop just cleared, it is sixty frames of recovery.
                seen.iter()
                    .filter(|s| s.alive && in_reach(s) && !(s.leader && s.crouching))
                    .min_by_key(|s| dist(s).raw())
            });
        if let Some(t) = threat {
            let (yaw, aim, pitch) = look_at(t.at, t.middle);
            if !in_reach(t) {
                // A step in to bring the crouch into reach.
                self.intent = SWING;
                return Input::looking_at(steer(yaw, flat(t.at.sub(me.pos))), aim, pitch);
            }
            if self.cooldown == 0 && free {
                self.intent = SWING;
                self.cooldown = SWING_GAP + self.roll() % 4;
                let swing = Input::looking_at(auto_button(&me), aim, pitch);
                return self.hands.hit(w, &me, t.middle, swing, None);
            }
            return Input::looking_at(0, aim, pitch);
        }

        // 1 and 7. **Back to the wall**, walking, facing the pack.
        if let Some(post) = self.post {
            let to = flat(post.sub(me.pos));
            if to.flat_len().raw() > AT_POST.raw() {
                self.intent = POST;
                return Input::aimed(steer(pack_yaw, to), pack_aim);
            }
        }
        // 2. **Face the pack**, and wait for it to come to you; a gnawer
        // circling in close is stepped toward to bring it into reach.
        self.intent = HOLD;
        let nearest = members
            .iter()
            .filter(|s| dist(s).raw() < NEAR.raw())
            .min_by_key(|s| dist(s).raw());
        // Nothing coming at it: the class's own business, at the nearest.
        let coming = members
            .iter()
            .any(|s| (s.tail || s.crouching) && dist(s).raw() < WATCH_TAILS.raw());
        if !coming
            && let Some(s) = nearest.or(members.first())
            && let Some(input) = self.hands.idle(w, &me, facing_pack, s.middle, i32::MAX)
        {
            return input;
        }
        let bits = match nearest {
            Some(s) if dist(s).raw() > reach.raw() => {
                let (yaw, _, _) = look_at(s.at, s.middle);
                steer(yaw, flat(s.at.sub(me.pos)))
            }
            _ => 0,
        };
        let _ = MAX_PLAYERS;
        Input::aimed(bits, pack_aim)
    }

    fn intent(&self) -> Intent {
        self.intent
    }

    fn hands(&mut self) -> Option<&mut Hands> {
        Some(&mut self.hands)
    }

    fn hands_ref(&self) -> Option<&Hands> {
        Some(&self.hands)
    }
}

/// **The Gnawers' own report lines** (§9): whether the back lesson and the
/// slow lesson are being taught, target priority, and the arc of the fight.
#[derive(Default)]
pub struct Lines {
    hamstrings: u32,
    hamstrings_landed: u32,
    shed: u32,
    pileons: u32,
    pileon_bites: u32,
    escaped: u32,
    howls: u32,
    howls_cancelled: u32,
    stumbles: u32,
    windows: u32,
    windows_used: u32,
    window_hit: bool,
    leader_dead: Option<u32>,
    felled: u32,
}

impl crate::report::Tally for Lines {
    fn observe(&mut self, before: &World, after: &World) {
        let (Some(was), Some(now)) = (before.pack, after.pack) else {
            return;
        };
        for (b, a) in before.critters.iter().zip(after.critters.iter()) {
            let began = a.state == is::STARTUP && b.state != is::STARTUP;
            let landed = a.state == is::ACTIVE
                && a.has(flag::HIT_USED)
                && !(b.state == is::ACTIVE && b.has(flag::HIT_USED));
            if began && a.act == gnawers::HAMSTRING {
                self.hamstrings += 1;
            }
            if landed && a.act == gnawers::HAMSTRING {
                self.hamstrings_landed += 1;
            }
            if landed && a.act == gnawers::PILE_ON {
                self.pileon_bites += 1;
            }
            if gnawers::latched(b) && !gnawers::latched(a) {
                let p = after.players[(b.target as usize).min(MAX_PLAYERS - 1)];
                if matches!(p.action, sim::state::Action::Dodge { .. }) {
                    self.shed += 1;
                }
            }
            if b.has(flag::LEADER) {
                if a.state == is::ACTIVE && a.act == gnawers::HOWL && b.state == is::STARTUP {
                    self.howls += 1;
                }
                if b.state == is::STARTUP && b.act == gnawers::HOWL && a.state == is::FLINCH {
                    self.howls_cancelled += 1;
                }
                if gnawers::stumbling(a) && !gnawers::stumbling(b) {
                    self.stumbles += 1;
                }
                if b.alive() && !a.alive() && self.leader_dead.is_none() {
                    self.leader_dead = Some(after.frame);
                }
                if a.health < b.health && now.mood == sim::pack::mood::SCATTERED {
                    self.window_hit = true;
                }
            }
        }
        // A pile-on beginning, and ending with nothing landed.
        if gnawers::piling(&now) && !gnawers::piling(&was) {
            self.pileons += 1;
            self.pileon_bites = 0;
        }
        if !gnawers::piling(&now) && gnawers::piling(&was) && self.pileon_bites == 0 {
            self.escaped += 1;
        }
        // A scatter: the window, and whether the leader was hit in it.
        let scattered = |m: u8| m == sim::pack::mood::SCATTERED;
        if scattered(now.mood) && !scattered(was.mood) {
            self.windows += 1;
            self.window_hit = false;
        }
        if !scattered(now.mood) && scattered(was.mood) && self.window_hit {
            self.windows_used += 1;
        }
        if gnawers::gnawed(&was).is_some()
            && gnawers::gnawed(&now).is_none()
            && sim::stones::gather(&after.players).iter().flatten().count()
                < sim::stones::gather(&before.players)
                    .iter()
                    .flatten()
                    .count()
        {
            self.felled += 1;
        }
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let row = |name: &str, value: String, why: &str| (name.to_string(), value, why.to_string());
        vec![
            row(
                "hamstrings landed",
                format!("{} / {}", self.hamstrings_landed, self.hamstrings),
                "of those thrown: the back lesson",
            ),
            row(
                "latches shed by a dodge",
                format!("{}", self.shed),
                "the answer to a latch",
            ),
            row(
                "pile-ons escaped",
                format!("{} / {}", self.escaped, self.pileons),
                "nothing landed, of those started: the slow lesson",
            ),
            row(
                "howls cancelled",
                format!(
                    "{} / {}",
                    self.howls_cancelled,
                    self.howls_cancelled + self.howls
                ),
                "target priority",
            ),
            row(
                "stumbles",
                format!("{}", self.stumbles),
                "the Big One knocked down: the big window",
            ),
            row(
                "scatter windows used",
                format!("{} / {}", self.windows_used, self.windows),
                "windows in which the Big One was hit",
            ),
            row(
                "leader dead at",
                self.leader_dead
                    .map_or("--".to_string(), |f| format!("{:.0}s", f as f32 / 60.0)),
                "the arc of the fight",
            ),
            row("stones felled", format!("{}", self.felled), "by the gnaw"),
        ]
    }
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::GNAWERS,
    plan: |who, seed, _hop| Box::new(Gnawers::new(who, seed)),
    bucks: |_| false,
    words: crate::plans::Words {
        weak_hits: "weak-point hits",
        broken: "parts broken",
        into_breakables: "damage into parts",
        into_breakables_why: "nothing to break on a pack",
        worst: "worst part",
        ride_for: "nothing to ride",
        toppled_pool: "off a pool under a fallen body",
    },
    tally: Some(|| Box::new(Lines::default())),
    gamble: None,
};
