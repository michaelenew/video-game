//! What each class does with the thing that makes it that class.
//!
//! The rest of the bot fights with buttons that throw moves, and would fight
//! the same way on any kit. This is the part that would not: the Champion's
//! Rush, the Bulwark's shield, the Reaver's second body, the Elementalist's
//! stones, the Blood mage's pools and the Dual mage's two bars. Each class gets
//! two hooks:
//!
//! * **[`Duelist::class_play`]**, every frame: read the mechanic, and decide
//!   whether to start a *gesture* -- an aimed press of `E`, of right click, of
//!   `Q` or of a dodge -- or to hold a button down. A gesture is carried out by
//!   the same hands as an attack: the mouse still has to come round, and the
//!   aim is still off by a little.
//! * **[`Duelist::tool_bias`]**: how much it favours or refuses each of its
//!   moves right now. A Slam is worth more with a full shield; the Dual mage's
//!   auto on her higher side is worth nothing, because it burns her.
//!
//! What they know is what a player knows about their own class: where the
//! shield is, how full it is, where the shadow stands, which stones are up and
//! which pools are theirs. About the other fighter they know only what the
//! delay line shows them.

use super::{After, Duelist, Gesture, Seen, Tool, back, by_the_wall, flat};
use sim::class::{Force, Mechanic, Shield};
use sim::effects::EffectKind;
use sim::fixed::Fx;
use sim::moves::{self, elementalist};
use sim::state::{self, Action, Player, SLOT_MECHANIC, SLOT_SPECIAL};
use sim::{Class, Input, V3, World};

/// What `class_play` wants this frame on top of the plan.
#[derive(Clone, Copy, Default)]
pub(super) struct ClassOut {
    /// Buttons held down this frame.
    pub hold: u16,
    /// Where to walk instead of where the plan said.
    pub want: Option<V3>,
}

/// Things about the fight each class's weighing wants, read once a frame.
#[derive(Clone, Copy, Default)]
pub(super) struct Cues {
    /// Marks on the other fighter, from the Reaver's shadow. On their body
    /// for anybody to see.
    pub marks: u8,
    /// One of the Blood mage's pools is at the other fighter's feet.
    pub pool_by_them: bool,
    /// One of the Elementalist's standing stones is between her and them.
    pub stone_between: bool,
    /// Her fire -- a pillar, a cloud of embers, a carpet -- is on the line
    /// to them, or round them: a Bolt shot through it comes out a fire bolt.
    pub fire_between: bool,
    /// They are guarding.
    pub guarding: bool,
    /// How full the Bulwark's shield is, in percent.
    pub fullness: i32,
}

/// Frames the Champion holds the next weapon through a recovery, waiting for
/// its chain window to open.
const CHAIN_HOLD: u16 = 30;
/// How close to the opponent the shadow has to be for her to cash in on it.
const SHADOW_CLOSE: Fx = Fx::ratio(25, 10);
/// How close to the opponent a pool has to be to be worth blinking to.
const POOL_CLOSE: Fx = Fx::ratio(25, 10);
/// The furthest a blink is worth taking.
const BLINK_FAR: Fx = Fx::from_int(12);
/// Frames a stone takes to come up, for leading someone walking.
const STONE_RISE: i32 = 14;
/// How far a Rush carries him, near enough: twenty frames at nineteen metres a
/// second.
const RUSH_REACH: Fx = Fx::ratio(63, 10);
/// Frames of jump held after raising a stone to ride it up.
const STONE_RIDE: u16 = 30;
/// How far inside the walls a stone is ridden from. A jump off a stone is
/// half as high again as one off the floor, and her retreat carries on
/// through it: from a jump's clearance it took her over the proving ground's
/// wall, once the bot raised the stone with the right button (2026-10-20).
const STONE_RIDE_CLEAR: Fx = Fx::from_int(8);

/// How close to the line her fire has to be for a Bolt to pass through it:
/// a pillar's girth and a little.
const FIRE_NEAR: Fx = Fx::ratio(15, 10);

/// Is this one of the Elementalist's two pushes?
fn pushes(bits: u16) -> bool {
    bits == elementalist::keys::WEAK_PUSH || bits == elementalist::keys::STRONG_PUSH
}

impl Duelist {
    /// Read what the class-specific weighing needs.
    pub(super) fn read_cues(&mut self, me: &Player, seen: &Seen, w: &World) {
        let them = &seen.them;
        let mut cues = Cues {
            marks: them.marks,
            guarding: them.action.guarding(),
            fullness: (sim::bulwark::fullness(me).mul(Fx::from_int(100))).to_int(),
            ..Cues::default()
        };
        if me.class == Class::BloodMage {
            cues.pool_by_them = self
                .pools(w)
                .any(|(at, _)| flat(at.sub(them.pos)).flat_len().raw() < POOL_CLOSE.raw());
        }
        if let Mechanic::Structures(stones) = me.mechanic {
            let line = flat(them.pos.sub(me.pos));
            let dist = line.flat_len();
            let dir = line.normalized();
            let on_the_line = |at: V3, within: Fx| {
                let rel = flat(at.sub(me.pos));
                let along = rel.dot(dir);
                let off = rel.sub(dir.scale(along)).flat_len();
                along.raw() > 0 && along.raw() < dist.add(within).raw() && off.raw() < within.raw()
            };
            cues.stone_between = stones.iter().flatten().any(|s| on_the_line(s.at, Fx::ONE));
            let who = self.who as u8;
            cues.fire_between = w.effects.iter().flatten().any(|e| {
                e.owner == who
                    && matches!(
                        e.kind,
                        EffectKind::FirePillar | EffectKind::Embers | EffectKind::FireCarpet
                    )
                    && on_the_line(e.pos, FIRE_NEAR)
            });
        }
        self.cues = cues;
    }

    /// Her pools, where they are and how much is in them.
    fn pools<'a>(&self, w: &'a World) -> impl Iterator<Item = (V3, i32)> + 'a {
        let who = self.who as u8;
        w.effects
            .iter()
            .flatten()
            .filter(move |e| e.is_a_pool() && e.owner == who)
            .map(|e| (e.pos, e.pool_volume()))
    }

    /// How much it wants this move right now, in percent. Zero refuses it.
    ///
    /// **A move that hurt nobody when it was tried is refused** -- the kit
    /// measured it ([`super::Kit::measure`]) -- unless the class has a reason
    /// for it here: the Elementalist's pushes with a stone in the line.
    pub(super) fn tool_bias(&self, me: &Player, t: Tool) -> i32 {
        let bias = self.class_bias(me, t);
        if t.hurt > 0 || self.wants_harmless(me, t) {
            bias
        } else {
            0
        }
    }

    /// The Elementalist's two pushes go through a body and act on what she
    /// built: worth throwing at somebody only with one of her stones between
    /// her and them, to kick into them (the Bolt) or break into debris on
    /// them (Cataclysm) -- or, for the Bolt, her fire on the line, which it
    /// comes out of as a fire bolt. That is what the creature hunter does
    /// with it, a pillar under the work and the Bolt through it, and without
    /// it her hunts do half the damage (2026-10-20).
    fn wants_harmless(&self, me: &Player, t: Tool) -> bool {
        me.class == Class::Elementalist
            && pushes(t.bits)
            && (self.cues.stone_between
                || self.cues.fire_between && t.bits == elementalist::keys::WEAK_PUSH)
    }

    fn class_bias(&self, me: &Player, t: Tool) -> i32 {
        let cues = self.cues;
        match me.class {
            Class::Bulwark => {
                let m = moves::get(me.class, t.kind);
                if t.bits == Input::MIDDLE {
                    // Slam: what the weight is for.
                    return match cues.fullness {
                        f if f >= 90 => 700,
                        f if f >= 50 => 300,
                        f if f >= 20 => 100,
                        _ => 40,
                    };
                }
                if m.grabs > 0 && cues.guarding {
                    return 500;
                }
                100
            }
            // The shadow is moved by `class_play`, not thrown as a poke, and
            // a mark on the board is a reason to hit them with her own body.
            Class::ShadowReaver => {
                if t.kind == SLOT_MECHANIC {
                    0
                } else if cues.marks >= 3 {
                    250
                } else {
                    100
                }
            }
            Class::Elementalist => {
                if cues.stone_between && pushes(t.bits) {
                    // A push kicks a stone along the line it was shot on.
                    400
                } else if cues.fire_between && t.bits == elementalist::keys::WEAK_PUSH {
                    // The Bolt, through her fire: a fire bolt.
                    300
                } else if t.bits == elementalist::keys::FIRE {
                    // The pillar: her damage, and the field's fire.
                    150
                } else {
                    100
                }
            }
            Class::BloodMage => {
                let m = moves::get(me.class, t.kind);
                let red = me.health * 100 / me.full_health().max(1);
                let mut bias = 100;
                // What she pays for, she pays in health she is short of.
                if m.cost > 2 && red < 30 {
                    bias = 30;
                }
                if t.kind == moves::blood::BLACK_SPIKE {
                    // The spike, on a pool at their feet, erupts.
                    bias = if cues.pool_by_them {
                        bias * 4
                    } else {
                        bias / 3
                    };
                }
                if moves::blood::scythe(t.kind) && me.grey_share().raw() > Fx::ratio(3, 10).raw() {
                    // The scythe is longer with grey on the bar.
                    bias *= 2;
                }
                bias
            }
            Class::DualMage => dual_bias(me, t.kind),
            Class::Champion => 100,
        }
    }

    /// One frame of the class's own play.
    pub(super) fn class_play(&mut self, me: &Player, seen: &Seen, w: &World) -> ClassOut {
        self.mech_cool = self.mech_cool.saturating_sub(1);
        let mut out = ClassOut::default();
        // A chain being held through a recovery.
        if let Some((bits, left, from)) = self.chain_hold {
            let moved_on = !matches!(me.action, Action::Recovery { kind, .. } if kind == from);
            if left == 0 || moved_on {
                self.chain_hold = None;
            } else {
                out.hold |= bits;
                self.chain_hold = Some((bits, left - 1, from));
            }
        }
        if self.gesture.is_some() {
            return out;
        }
        let them = &seen.them;
        let to = flat(them.pos.sub(me.pos));
        let dist = to.flat_len();
        let toward = to.normalized();
        match me.mechanic {
            Mechanic::Forms { .. } => self.champion(me, seen, dist, toward),
            Mechanic::Shield(shield) => self.bulwark(me, seen, shield, dist, &mut out),
            Mechanic::Shadow(_) => self.reaver(me, seen, dist),
            Mechanic::Structures(_) => self.elementalist(me, seen, dist, toward),
            Mechanic::Blood => self.blood(me, seen, w, dist),
            Mechanic::Meter { .. } => self.dual(me, seen, dist, toward, &mut out),
        }
        out
    }

    fn ready(&self, me: &Player) -> bool {
        self.mech_cool == 0 && self.attempt.is_none() && me.action.actionable()
    }

    fn cool(&mut self, lo: i32, hi: i32) {
        self.mech_cool = self.between(lo, hi) as u16;
        self.mechanic_uses += 1;
    }

    /// Rush, and the chain.
    fn champion(&mut self, me: &Player, seen: &Seen, dist: Fx, toward: V3) {
        let Mechanic::Forms {
            recharge,
            chain,
            chain_hit,
            ..
        } = me.mechanic
        else {
            return;
        };
        let rush_ready = recharge == 0;
        if let Action::Recovery { kind, .. } = me.action {
            if chain_hit && chain < moves::champion::DEPTH && self.chain_hold.is_none() {
                // **Hit-confirmed, by feel.** The hit landed on this body's
                // own swing, so there is no delay on knowing it; what is left
                // is whether it goes for the next link, and with which weapon.
                let go = 55 + self.style.aggression / 2;
                if self.chance(go) {
                    let weapons = [Input::LEFT, Input::MIDDLE, Input::RIGHT];
                    let pick = weapons[self.roll(3) as usize];
                    self.chain_hold = Some((pick, CHAIN_HOLD, kind));
                }
            } else if !chain_hit && rush_ready && self.mech_cool == 0 {
                // Whiffed or blocked, and close: Rush out of the recovery --
                // away if it is careful, straight past them if it is not.
                let close = dist.raw() < Fx::from_int(4).raw();
                let bail = (self.style.caution + self.style.aggression) / 3;
                if close && self.chance(bail) {
                    let side = V3::new(toward.z.neg(), Fx::ZERO, toward.x);
                    let mut away = if self.chance(self.style.caution) {
                        back(toward)
                    } else {
                        side.scale(Fx::from_int(self.strafe))
                    };
                    // Not into a wall: a Rush carries six metres.
                    let lands = me.pos.add(away.scale(RUSH_REACH));
                    if by_the_wall(lands, super::EDGE, &self.bounds) {
                        away = side.scale(Fx::from_int(-self.strafe));
                    }
                    let lands = me.pos.add(away.scale(RUSH_REACH));
                    if !by_the_wall(lands, super::EDGE, &self.bounds) {
                        self.gesture = Some(
                            Gesture::press(Input::MECHANIC, seen.them.pos)
                                .walking(away)
                                .level(),
                        );
                        self.cool(40, 90);
                    }
                }
            }
            return;
        }
        // Rush in from mid range and swing out of it.
        let mid = dist.raw() > Fx::from_int(4).raw() && dist.raw() < Fx::from_int(9).raw();
        let eager = matches!(self.plan, super::Plan::Press | super::Plan::DashIn);
        let walled = by_the_wall(me.pos, super::JUMP_CLEAR, &self.bounds)
            || by_the_wall(seen.them.pos, super::JUMP_CLEAR, &self.bounds);
        if rush_ready
            && mid
            && eager
            && !walled
            && self.ready(me)
            && self.chance(self.style.aggression / 2)
        {
            // The sword or the hammer out of it: the spear out of a Rush,
            // looking down, is the Pole vault.
            let pick = if self.chance(50) {
                Input::LEFT
            } else {
                Input::MIDDLE
            };
            let follow = self.mine.iter().find(|t| t.bits == pick);
            self.gesture = Some(
                Gesture::press(Input::MECHANIC, seen.them.pos)
                    .walking(toward)
                    .level()
                    .then(follow.map_or(After::Nothing, |t| After::Swing(t, 6))),
            );
            self.cool(60, 140);
        }
    }

    /// The shield: guard to load it, Slam to spend it, throw, leap and recall.
    fn bulwark(&mut self, me: &Player, seen: &Seen, shield: Shield, dist: Fx, out: &mut ClassOut) {
        let them = &seen.them;
        match shield {
            Shield::Held { .. } => {
                self.shield_out = 0;
                // Turtling at the edge of their reach is how the shield gets
                // heavy. Not always: a guard is a vulnerable window too.
                let near = dist.raw() < self.theirs.threat().add(Fx::ONE).raw();
                if near
                    && self.holding.is_none()
                    && me.action.actionable()
                    && matches!(
                        self.plan,
                        super::Plan::Footsies | super::Plan::Wait | super::Plan::Bait
                    )
                    && self.mech_cool == 0
                    && self.chance(self.style.caution / 3)
                {
                    let frames = self.between(15, 45) as u16;
                    self.holding = Some((super::Answer::Guard, frames));
                    self.mech_cool = frames + self.between(10, 40) as u16;
                }
                // Throw it from range, heavier the better.
                let range = sim::tuning::shield_range();
                let far = dist.raw() > Fx::from_int(5).raw() && dist.raw() < range.raw();
                let want =
                    self.style.zoning / 2 + self.style.aggression / 4 + self.cues.fullness / 4;
                if far && self.ready(me) && self.chance(want / 3) {
                    let at = self.lead(seen, 20);
                    self.gesture = Some(Gesture::press(Input::MECHANIC, at).free());
                    self.cool(20, 40);
                }
            }
            Shield::Flying { outbound, .. } => {
                self.shield_out = self.shield_out.saturating_add(1);
                // Leap after it -- throw, leap, slam.
                if outbound
                    && self.shield_out > 6
                    && dist.raw() > Fx::from_int(4).raw()
                    && self.mech_cool == 0
                    && self.chance(self.style.aggression / 3)
                {
                    self.gesture = Some(Gesture::press(Input::MECHANIC, them.pos));
                    self.cool(40, 80);
                }
            }
            Shield::Planted { pos, .. } => {
                self.shield_out = self.shield_out.saturating_add(1);
                // Recall it: now if they are standing on the way home,
                // otherwise before long, because he cannot guard without it.
                let home = flat(me.pos.sub(pos));
                let rel = flat(them.pos.sub(pos));
                let along = rel.dot(home.normalized());
                let off = rel.sub(home.normalized().scale(along)).flat_len();
                let in_path = along.raw() > 0
                    && along.raw() < home.flat_len().raw()
                    && off.raw() < Fx::ratio(15, 10).raw();
                let stale = self.shield_out > 40 + self.style.patience as u16;
                if (in_path || stale) && me.action.actionable() && self.chance(40) {
                    self.gesture = Some(Gesture::press(Input::MECHANIC, them.pos).free());
                    self.mechanic_uses += 1;
                }
            }
        }
        let _ = out;
    }

    /// The shadow: send it at them, cut with it, dash to it, cash the marks.
    fn reaver(&mut self, me: &Player, seen: &Seen, dist: Fx) {
        let Some(shadow) = sim::shadow::of(me) else {
            return;
        };
        let them = &seen.them;
        let from_them = flat(shadow.pos.sub(them.pos)).flat_len();
        if let Some(t) = self.recall_in {
            if t == 0 {
                self.recall_in = None;
                if shadow.is_out() {
                    self.gesture = Some(Gesture::press(Input::RIGHT, them.pos));
                }
            } else {
                self.recall_in = Some(t - 1);
            }
            return;
        }
        if !shadow.is_out() {
            self.shadow_out = 0;
            // Send it where they are, or a little past them.
            let reach = moves::get(me.class, SLOT_MECHANIC).reach;
            let range = dist.raw() > Fx::from_int(2).raw() && dist.raw() < reach.raw();
            let want = 30 + self.style.zoning / 3 + self.style.aggression / 4;
            if range && self.ready(me) && !me.locked_out(SLOT_MECHANIC) && self.chance(want / 2) {
                let at = self.lead(seen, 12);
                self.gesture = Some(Gesture::press(Input::RIGHT, at).free());
                self.cool(15, 40);
            }
            return;
        }
        self.shadow_out = self.shadow_out.saturating_add(1);
        let close = from_them.raw() < SHADOW_CLOSE.raw();
        // The lotus, where the shadow is, and dragged home through them.
        if close
            && self.ready(me)
            && !me.locked_out(SLOT_SPECIAL)
            && self.chance(25 + self.style.aggression / 3)
        {
            self.gesture = Some(Gesture::press(Input::SPECIAL, them.pos).free());
            self.recall_in = Some(self.between(20, 35) as u16);
            self.cool(30, 60);
            return;
        }
        // Dash to it and cash in: the marks are on them and the shadow is
        // beside them.
        let far_enough = dist.raw() > Fx::from_int(3).raw();
        if close
            && far_enough
            && self.cues.marks >= 2
            && self.ready(me)
            && self.chance(20 + self.style.aggression / 3)
        {
            let slash = self.mine.iter().find(|t| t.bits == Input::LEFT);
            self.gesture = Some(
                Gesture::press(Input::SHIFT, shadow.pos)
                    .walking(flat(shadow.pos.sub(me.pos)).normalized())
                    .free()
                    .then(slash.map_or(After::Nothing, |t| After::Swing(t, 2))),
            );
            self.cool(40, 90);
            return;
        }
        // Out too long, somewhere useless: bring it back, through them if
        // they are on the way.
        if self.shadow_out > 90 + self.style.patience as u16 && !close && self.mech_cool == 0 {
            self.gesture = Some(Gesture::press(Input::RIGHT, them.pos));
            self.cool(10, 30);
        }
    }

    /// Stones: up under them, up in front of her, under her own feet, and
    /// up between them to push into them.
    ///
    /// **Her buttons since 2026-10-09**: left click is earth (Raise on the
    /// floor, Landfall in the air), and `Q` and `E` are the two pushes, which
    /// go through a body and hurt nobody on their own -- they are for the
    /// stones. The bot pressed `E` for Raise until 2026-10-20, so all of this
    /// threw Cataclysm at air.
    fn elementalist(&mut self, me: &Player, seen: &Seen, dist: Fx, toward: V3) {
        let earth = elementalist::keys::EARTH;
        let them = &seen.them;
        if !me.grounded {
            // Landfall, from above them.
            if me.vel.y.raw() < 0
                && dist.raw() < Fx::ratio(35, 10).raw()
                && self.ready(me)
                && self.chance(30)
            {
                self.gesture = Some(Gesture::press(earth, them.pos).free());
                self.cool(40, 80);
            }
            return;
        }
        if !self.ready(me) {
            return;
        }
        let reach = sim::tuning::raise_reach();
        let red = me.health * 100 / me.full_health().max(1);
        let pressed = dist.raw() < Fx::ratio(25, 10).raw();
        // Ride a stone up and out of trouble.
        if (red < 40 || pressed)
            && !by_the_wall(me.pos, STONE_RIDE_CLEAR, &self.bounds)
            && self.chance(self.style.caution / 6 + self.style.air / 8)
        {
            self.gesture = Some(
                Gesture::press(earth, me.pos)
                    .free()
                    .then(After::Jump(STONE_RIDE)),
            );
            self.cool(120, 240);
            return;
        }
        // A wall between them and her, when they are coming.
        if pressed && matches!(self.plan, super::Plan::Retreat | super::Plan::Zone) {
            if self.chance(35) {
                let at = me.pos.add(toward.scale(Fx::ratio(18, 10)));
                self.gesture = Some(Gesture::press(earth, at).free());
                self.cool(60, 150);
            }
            return;
        }
        // **A stone in the lane, then the push.** Raised a third of the way
        // to them and kicked along the line into them by the Bolt, or broken
        // over them by Cataclysm: the two pushes are what the stones are for.
        let lane = dist.raw() > Fx::from_int(4).raw() && dist.raw() < Fx::from_int(10).raw();
        let push = self.pick_push(me);
        if lane
            && let Some(push) = push
            && self.chance(20 + self.style.zoning / 4)
        {
            let at = me
                .pos
                .add(toward.scale(dist.div(Fx::from_int(3)).min(reach)));
            self.gesture = Some(
                Gesture::press(earth, at)
                    .free()
                    .then(After::Swing(push, STONE_RISE as u16)),
            );
            self.cool(70, 160);
            return;
        }
        // Up under their feet, where they will be when it rises.
        if dist.raw() < reach.sub(Fx::ratio(3, 10)).raw()
            && dist.raw() > Fx::ratio(15, 10).raw()
            && self.chance(15 + self.style.aggression / 4)
        {
            let at = self.lead(seen, STONE_RISE);
            self.gesture = Some(Gesture::press(earth, at).free());
            self.cool(70, 180);
        }
    }

    /// One of her two pushes, ready: the Bolt more often, since it is quick
    /// and only kicks the stone, and Cataclysm sometimes, since it breaks it.
    fn pick_push(&mut self, me: &Player) -> Option<Tool> {
        let weak = elementalist::keys::WEAK_PUSH;
        let strong = elementalist::keys::STRONG_PUSH;
        let first = if self.chance(65) { weak } else { strong };
        let ready = |t: &Tool| me.mechanic_ready(t.kind) && !me.locked_out(t.kind);
        let mine = self.mine;
        mine.iter()
            .find(|t| t.bits == first && ready(t))
            .or_else(|| mine.iter().find(|t| pushes(t.bits) && ready(t)))
    }

    /// Pools: blink to one by them to swing, or one far from them to get out.
    fn blood(&mut self, me: &Player, seen: &Seen, w: &World, dist: Fx) {
        if !self.ready(me) {
            return;
        }
        let them = &seen.them;
        let red = me.health * 100 / me.full_health().max(1);
        let mut near_them: Option<V3> = None;
        let mut away: Option<(V3, Fx)> = None;
        for (at, _) in self.pools(w) {
            let from_me = flat(at.sub(me.pos)).flat_len();
            if from_me.raw() > BLINK_FAR.raw() || from_me.raw() < Fx::from_int(2).raw() {
                continue;
            }
            let from_them = flat(at.sub(them.pos)).flat_len();
            if from_them.raw() < POOL_CLOSE.raw() {
                near_them = Some(at);
            }
            if away.is_none_or(|(_, d)| from_them.raw() > d.raw()) {
                away = Some((at, from_them));
            }
        }
        let in_trouble = red < 35 || (dist.raw() < Fx::from_int(2).raw() && self.mood < -10);
        if in_trouble
            && let Some((at, d)) = away
            && d.raw() > Fx::from_int(6).raw()
            && self.chance(self.style.caution / 3)
        {
            self.gesture = Some(self.blink(me, at, None));
            self.cool(60, 120);
            return;
        }
        if let Some(at) = near_them
            && dist.raw() > Fx::from_int(5).raw()
            && self.chance(self.style.aggression / 3)
        {
            let sweep = self
                .mine
                .iter()
                .find(|t| t.bits == moves::blood::keys::SCYTHE);
            let blink = self.blink(me, at, sweep.map(|t| After::Swing(t, 3)));
            self.gesture = Some(blink);
            self.cool(60, 140);
        }
    }

    fn blink(&self, me: &Player, at: V3, then: Option<After>) -> Gesture {
        Gesture::press(Input::SHIFT, at)
            .walking(flat(at.sub(me.pos)).normalized())
            .free()
            .then(then.unwrap_or(After::Nothing))
    }

    /// The bars are kept level by `dual_bias`. This is the body they unlock:
    /// the second jump, going all in once the wings are out, and both majors
    /// back to back once the bars are deep.
    fn dual(&mut self, me: &Player, seen: &Seen, dist: Fx, toward: V3, out: &mut ClassOut) {
        // **Off the floor near a wall, back toward the middle** (2026-10-13):
        // with the second jump and the slow fall most of a fight, her drift
        // carried her over the proving ground's low wall, and a bot that only
        // let go of the stick kept the momentum it had.
        let margin = sim::tuning::auto_hop().add(Fx::from_int(3));
        if !me.grounded && by_the_wall(me.pos, margin, &self.bounds) {
            out.want = Some(flat(back(me.pos)).normalized());
        }
        if sim::dual::ascending(me) {
            self.plan = super::Plan::Press;
            self.plan_left = self.plan_left.max(30);
            return;
        }
        // Lopsided, and only an auto on the low side mends it: those are
        // thrown up close, so whatever the plan says, walk in.
        let gap = sim::dual::gap(me);
        // A push on the low side -- the mend, which is the whole of how she
        // uses the mechanic -- shows as the gap closing by a push at once.
        let push = Fx::from_int(sim::tuning::meter_auto_push());
        // And since 2026-10-12 a bar spent -- a major, or a vent -- shows as
        // the lower bar or the gap dropping by more than a push at once.
        let lower = sim::dual::lower(me);
        if self.gap_was.sub(gap).raw() >= push.raw() || self.lower_was.sub(lower).raw() > push.raw()
        {
            self.mechanic_uses += 1;
        }
        self.gap_was = gap;
        self.lower_was = lower;
        let near_the_line = Fx::from_int(sim::tuning::meter_band() * 3 / 4);
        if gap.raw() > near_the_line.raw() && dist.raw() > Fx::ratio(25, 10).raw() {
            out.want = Some(toward);
        }
        // **Both majors, one after the other** (2026-10-20). Each spends the
        // whole of its bar, so one alone leaves the other bar running away
        // and `dual_bias` refuses it; the Abyss and then Judgement spend both
        // and leave her level -- the turn the kit is written around, and the
        // biggest numbers she has, grown by how full the bars were.
        let deep = Fx::from_int(sim::tuning::tier_jump());
        let judgement = self
            .mine
            .iter()
            .find(|t| t.bits == moves::dual::keys::LIGHT_MAJOR);
        let abyss = moves::get(me.class, moves::dual::ABYSS);
        if let Some(judgement) = judgement
            && lower.raw() >= deep.raw()
            && me.grounded
            && dist.raw() < self.mine.reach_of(judgement, Some(me)).raw()
            && self.ready(me)
            && me.mechanic_ready(moves::dual::ABYSS)
            && !me.locked_out(moves::dual::ABYSS)
            && !me.locked_out(judgement.kind)
            && self.chance(25 + self.style.aggression / 3)
        {
            let after = abyss.startup + abyss.active + abyss.recovery;
            self.gesture = Some(
                Gesture::press(
                    moves::dual::keys::DARK_MAJOR,
                    self.lead(seen, abyss.startup as i32),
                )
                .free()
                .then(After::Swing(judgement, after)),
            );
            self.cool(90, 180);
            return;
        }
        let tier = sim::dual::tier(me);
        if tier >= sim::dual::Tier::Jump
            && !me.grounded
            && me.vel.y.raw() < 0
            && self.jump_left == 0
            && self.second_jump == 0
            && dist.raw() > Fx::ratio(15, 10).raw()
            && self.chance(20 + self.style.air / 3)
        {
            // A fresh press: the button has to come up first.
            self.second_jump = 2;
            self.mechanic_uses += 1;
        }
    }

    /// Where they will be in `frames`, as it saw them, if it thinks to lead.
    fn lead(&mut self, seen: &Seen, frames: i32) -> V3 {
        let lead = self.skill.lead as i32;
        if !self.chance(lead) {
            return seen.them.pos;
        }
        let ahead = Fx::from_int(self.lag as i32 + frames).mul(sim::DT);
        seen.them.pos.add(flat(seen.them.vel).scale(ahead))
    }
}

/// The Dual mage's two bars as they would be after `kind` is thrown, and the
/// force she is left carrying: `dual::steer` itself, on a copy of her, so the
/// bot cannot disagree with the rule (2026-10-12: it kept its own copy, which
/// went on filling the bar with a major after the majors began to spend it).
pub(crate) fn dual_after(me: &Player, kind: u8) -> Option<(Fx, Fx, Force)> {
    sim::dual::bars(me)?;
    let mut after = *me;
    sim::dual::steer(&mut after, kind);
    let (dark, light) = sim::dual::bars(&after)?;
    Some((dark, light, state::carrying(&after)))
}

/// The Dual mage's two bars, weighed for one move: never widen the gap past
/// the band, prefer what closes it, and do not stumble into ascension with
/// nobody to hit.
pub(crate) fn dual_bias(me: &Player, kind: u8) -> i32 {
    let (Some((dark, light)), Some((d, l, force))) = (sim::dual::bars(me), dual_after(me, kind))
    else {
        return 100;
    };
    let band = Fx::from_int(sim::tuning::meter_band());
    let gap_now = dark.sub(light).abs();
    let gap = d.sub(l).abs();
    // **A major spends its whole bar** (2026-10-12), so from level it leaves
    // the other ahead: the bot throws one only when that is the bar running
    // away -- the rule below, which prefers what closes the gap -- and leaves
    // the Abyss-then-Judgement turn to a person.
    if gap.raw() > band.raw() && gap.raw() > gap_now.raw() {
        // It burns her. Never.
        return 0;
    }
    let wings = Fx::from_int(sim::tuning::tier_wings());
    let lower = d.min(l);
    let red = me.health * 100 / me.full_health().max(1);
    if lower.raw() >= wings.raw() && !sim::dual::ascending(me) && red < 60 {
        // Ascension costs a great deal of health, and pays it back only
        // in hits. Not from behind.
        return 0;
    }
    let lower_side = if dark.raw() <= light.raw() {
        Force::Dark
    } else {
        Force::Light
    };
    if force == lower_side { 300 } else { 60 }
}
