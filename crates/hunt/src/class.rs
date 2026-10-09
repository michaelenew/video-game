//! How each class fights a creature.
//!
//! A creature's plan (`plans/`) knows the animal: where its feet are, which
//! window is worth going in on, what to jump and what to dodge. What it does
//! not know is the class in its hands, and for a long time it did not need
//! to: it pressed the poke, the heavy and the dodge, which is the Champion's
//! whole fight and a fraction of everybody else's. The Elementalist never
//! raised a stone or lit a pillar, the Reaver never sent her shadow, the
//! Blood mage never made a pool to drink, and the Dual mage punched with one
//! hand until her bars burned her -- so four classes lost almost every hunt
//! against every creature, and the report said nothing about why.
//!
//! **The plan says what, this says how.** [`Hands`] is one per hunter, owned
//! by its plan, and the plan calls it at five places it already had:
//!
//! * [`Hands::hit`] -- *hit that, now*: where the plan was about to press its
//!   poke or its heavy at a point on the creature. The class chooses what to
//!   throw and how to aim it, given how long the plan says the opening lasts.
//! * [`Hands::idle`] -- *you have a moment*: holding station between openings,
//!   with nothing visibly coming. The class spends it on its own business: the
//!   shadow out beside the work, a bolt or a pillar from range, a spike on a
//!   pool, the bars goaded up.
//! * [`Hands::close_in`] -- *go in on this window*: the Reaver's dash to a
//!   shadow already standing there, the Blood mage's blink to a pool.
//! * [`Hands::leave`] -- *get out*: where the plan was about to dodge, the same
//!   dodge pointed at a shadow or a pool lying that way.
//! * [`Hands::guard`] -- *this one is blockable*: the Bulwark takes it on the
//!   shield, and answers with Slam.
//!
//! And one the plan does not call: [`Hands::finish`], run on every frame's
//! input by [`crate::Hunter`] after the plan has spoken, which carries a press
//! that lasts more than a frame (the Grasp's hold, a recall after a lotus, a
//! swing out of a dash) and keeps the Dual mage's hands on the side that does
//! not burn her.
//!
//! **The Champion is the plans' own class**, and every one of these returns
//! what the plan asked for unchanged when he is the one holding it: the plans
//! were written for him and the Ridgeback's pin hashes his hunts. The Bulwark
//! changes only where a plan had no guard of its own.
//!
//! **It plays no better than the plan does.** Everything it reads about the
//! creature comes from the plan, which saw it through its fifteen-frame delay
//! line; what it reads for itself is its own body -- where its shadow stands,
//! which pools are its own, how full its bars are, how heavy its shield --
//! which a person knows without looking. It aims the way the sparring bot
//! does, through `sim::aim::look_onto`, and which button throws which move is
//! learned the way the sparring bot learns it (`duel::Kit::learn`), not typed
//! a second time.

use sim::class::Shield;
use sim::effects::EffectKind;
use sim::fixed::Fx;
use sim::math::{atan2_turns, wrap_turns};
use sim::moves;
use sim::state::{Player, SLOT_COMMITTED, SLOT_MECHANIC, SLOT_POKE, SLOT_SPECIAL};
use sim::{Class, Input, V3, World};

use crate::duel::{Kit, dual_after, dual_bias};
use crate::{steer, turns_to_aim};

/// Every button that throws something.
const ATTACKS: u16 = Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::SPECIAL | Input::MECHANIC;

/// Frames between two pieces of the class's own play, at the least. A person
/// does not do their class's trick on every frame they could.
const COOL_MIN: u16 = 24;
/// And the spread on top of that, drawn per use.
const COOL_SPREAD: u32 = 24;
/// Frames of an opening kept back, beyond a move's startup and active, before
/// it is worth starting: the same margin the plans keep for leaving.
const SPARE: i32 = 6;
/// How far from the work the shadow may stand and still be beside it.
const SHADOW_NEAR: Fx = Fx::ratio(3, 1);
/// A shadow further than this from the work, for this long, is called home.
const SHADOW_FAR: Fx = Fx::ratio(45, 10);
const SHADOW_STALE: u16 = 45;
/// Frames after the lotus opens before the recall drags it home.
const LOTUS_DRAG: u16 = 24;
/// The furthest the mouse goes off a plan's camera in one frame: a flick,
/// about eleven degrees, the sparring bot's `ON_TARGET`.
const FLICK: Fx = Fx::ratio(3, 100);
/// Frames past a move's own that make a window long: a creature down.
const LONG_WINDOW: i32 = 45;
/// How far out the crosshair's ray is asked about, checking where a cast
/// would land: further than anything in a kit reaches.
const SIGHT_REACH: Fx = Fx::from_int(30);
/// Further than this from what it hits is thrown from range.
const RANGED: Fx = Fx::from_int(4);
/// Marks on the creature worth cashing with the Executioner.
const CASH_AT: u8 = 2;
/// A pool this near the point being hit is a pool under it.
const POOL_UNDER: Fx = Fx::ratio(16, 10);
/// The nearest and furthest a blink or a dash is worth taking.
const HOP_NEAR: Fx = Fx::ratio(3, 1);
const HOP_FAR: Fx = Fx::from_int(12);
/// How well a shadow or a pool has to lie along the way out to be the way
/// out: the cosine of the angle between them, about sixty degrees.
const ALONG: Fx = Fx::ratio(5, 10);
/// The Dual mage stops goading her bars up at this: three quarters is the
/// second jump, and both full is an ascension nobody asked for.
const GOAD_TO: i32 = 80;
/// The Blood mage's red, in percent, below which she stops paying for things.
const RED_FLOOR: i32 = 25;
const RED_GRASP: i32 = 70;
const RED_CUT: i32 = 70;
/// **The Bulwark's throw and leap**: the nearest and furthest a window is
/// worth crossing that way. Inside the near one he walks; past the far one
/// the shield's nine metres and the leap's four or five leave him walking
/// most of it anyway.
const LEAP_NEAR: Fx = Fx::from_int(5);
const LEAP_FAR: Fx = Fx::from_int(11);
/// Frames the thrown shield is let travel before the leap after it: about
/// two metres of its nineteen a second, so he is thrown toward something
/// rather than at his own hand, and the key has been let up for the edge.
const LEAP_WAIT: u16 = 6;
/// Frames from the leap to the Slam out of it: the leap's rise is six metres
/// a second against forty-two of gravity, about seventeen frames up and down.
const LEAP_AIR: i32 = 18;
/// And how long the whole sequence may take before it is given up -- the
/// shield is recalled by [`Hands::finish`] if it was left out.
const LEAP_GIVE_UP: u16 = 60;
/// Frames before he throws to close again, at the least, and the spread on
/// top: a person throws a shield to cross a gap now and then, not at every
/// window the plan finds -- with only the class's ordinary rest it was
/// thrown about once a second against the Ridgeback.
const LEAP_REST: u16 = 150;
const LEAP_REST_SPREAD: u32 = 90;
/// **A pool worth a spike**: one holding about what the spike costs her and
/// more. The spike drinks the whole pool and comes up half again as hard, but
/// it costs nine in a hundred of her red on the press, and a scythe's pool is
/// twenty-two: spiking those was a hunt of paying ninety for twenty, and the
/// first runs bled to death without being hit.
const SPIKE_POOL: i32 = 80;

/// What a hunter did with its class, counted. The report's **THE CLASS**
/// section prints the lines that belong to the class that was played, so a
/// creature's document can say what each class did against it -- and the
/// harness can say whether it did anything at all.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Uses {
    /// The Reaver: shadows sent, swings thrown at nothing for the shadow's
    /// copy, dashes to one, lotuses opened, recalls, and Executioners thrown
    /// with marks on the creature.
    pub sent: u32,
    pub copies: u32,
    pub dashes: u32,
    pub lotuses: u32,
    pub recalls: u32,
    pub cashed: u32,
    /// The Elementalist: pillars lit, heavy and light shots from range.
    pub pillars: u32,
    pub cataclysms: u32,
    pub shots: u32,
    /// The Blood mage: spikes, spikes on a pool, Grasps, cuts, blinks.
    pub spikes: u32,
    pub erupted: u32,
    pub grasps: u32,
    pub cuts: u32,
    pub blinks: u32,
    /// The Dual mage: autos thrown with the hand the plan did not ask for so
    /// as to keep the bars level, finishers and sweeps, second jumps, and
    /// autos thrown at nothing to goad the bars up.
    pub turned: u32,
    pub finishers: u32,
    pub second_jumps: u32,
    pub goads: u32,
    /// The Bulwark: guards raised on a blockable blow, and Slams answering
    /// one; shields thrown to close on a window, leaps after them, Slams out
    /// of a leap, and recalls of a shield left planted.
    pub guards: u32,
    pub slams: u32,
    pub throws: u32,
    pub leaps: u32,
    pub leap_slams: u32,
    pub shield_recalls: u32,
}

impl Uses {
    /// Add another hunter's.
    pub fn add(&mut self, o: &Uses) {
        self.sent += o.sent;
        self.copies += o.copies;
        self.dashes += o.dashes;
        self.lotuses += o.lotuses;
        self.recalls += o.recalls;
        self.cashed += o.cashed;
        self.pillars += o.pillars;
        self.cataclysms += o.cataclysms;
        self.shots += o.shots;
        self.spikes += o.spikes;
        self.erupted += o.erupted;
        self.grasps += o.grasps;
        self.cuts += o.cuts;
        self.blinks += o.blinks;
        self.turned += o.turned;
        self.finishers += o.finishers;
        self.second_jumps += o.second_jumps;
        self.goads += o.goads;
        self.guards += o.guards;
        self.slams += o.slams;
        self.throws += o.throws;
        self.leaps += o.leaps;
        self.leap_slams += o.leap_slams;
        self.shield_recalls += o.shield_recalls;
    }

    /// The lines for one class: (what, how many, what it means).
    pub fn lines(&self, class: Class) -> Vec<(&'static str, u32, &'static str)> {
        match class {
            Class::Champion => Vec::new(),
            Class::Bulwark => vec![
                (
                    "guards raised",
                    self.guards,
                    "a blockable blow taken on the shield",
                ),
                (
                    "slams answering",
                    self.slams,
                    "the weight it stored, given back",
                ),
                ("throws", self.throws, "the shield thrown at a window"),
                ("leaps", self.leaps, "to the shield in flight"),
                ("leap slams", self.leap_slams, "Slams out of the leap"),
                (
                    "recalls",
                    self.shield_recalls,
                    "a planted shield called home",
                ),
            ],
            Class::ShadowReaver => vec![
                ("shadows sent", self.sent, "out beside the work"),
                (
                    "copies",
                    self.copies,
                    "swings at nothing, for the shadow's copy",
                ),
                ("lotuses", self.lotuses, "opened on the shadow"),
                ("recalls", self.recalls, "dragged home through it"),
                ("dashes", self.dashes, "to the shadow, in or out"),
                ("cashed", self.cashed, "Executioners on a marked creature"),
            ],
            Class::Elementalist => vec![
                ("pillars", self.pillars, "lit where it stands"),
                ("cataclysms", self.cataclysms, "the heavy, in a long window"),
                ("shots", self.shots, "bolts thrown from range"),
            ],
            Class::BloodMage => vec![
                ("spikes", self.spikes, "Black spikes thrown"),
                ("on a pool", self.erupted, "of those, at a pool under it"),
                ("grasps", self.grasps, "held and let go at it"),
                ("cuts", self.cuts, "Haemorrhages from range"),
                ("blinks", self.blinks, "to a pool, in or out"),
            ],
            Class::DualMage => vec![
                (
                    "hands turned",
                    self.turned,
                    "autos thrown with the low side's hand",
                ),
                (
                    "goads",
                    self.goads,
                    "autos thrown at nothing, to climb the bars",
                ),
                (
                    "finishers",
                    self.finishers,
                    "Judgements and sweeps in a long window",
                ),
                (
                    "second jumps",
                    self.second_jumps,
                    "off the three-quarter tier",
                ),
            ],
        }
    }
}

/// A press that lasts more than one frame, carried by [`Hands::finish`].
#[derive(Clone, Copy, Debug)]
enum Seq {
    /// Hold `bits` with the crosshair on `at` for `left` more frames, then let
    /// go. The Grasp's channel: how long it is held is how far it reaches.
    Hold { bits: u16, at: V3, left: u16 },
    /// Press `bits` in `left` frames, wherever the plan is looking. The recall
    /// that drags a lotus home.
    Later { bits: u16, left: u16 },
    /// A dash or a blink under way: press `then` at `at` the first frame the
    /// body is free of it (or inside the Reaver's carry), giving up after
    /// `left` frames.
    Then { then: u16, at: V3, left: u16 },
    /// The Bulwark's shield thrown at `at`: `wait` frames of its flight, then
    /// the leap to it, then -- shield in hand again, in the air -- the Slam
    /// out of the leap at `at`. `leapt` once the leap is pressed. Given up
    /// after `left` frames.
    Leap {
        at: V3,
        wait: u16,
        leapt: bool,
        left: u16,
    },
}

/// One hunter's class, in its hands. See the module.
#[derive(Clone)]
pub struct Hands {
    who: usize,
    rng: u32,
    kit: Option<Kit>,
    /// Frames before its own play starts anything again.
    cool: u16,
    seq: Option<Seq>,
    /// The sequence was started this frame, by a call the plan made: the
    /// frame's input already has it in, and [`Hands::finish`] leaves it be.
    fresh: bool,
    /// Buttons down on the last frame, so a press that needs an edge gets one.
    last_bits: u16,
    /// The Dual mage's second jump: 2 lets the button up, 1 presses it again.
    second: u8,
    /// And whether this airtime has had one.
    second_used: bool,
    /// Frames the Reaver's shadow has stood away from the work.
    far_for: u16,
    /// The Bulwark: frames a blow just taken on the shield is still answered
    /// with Slam, and the weight last frame, to feel one land.
    answer_left: u16,
    weight_was: Fx,
    /// The plan's camera, if it keeps one: see [`Hands::camera`].
    camera: Option<Fx>,
    /// What it did with its class.
    pub uses: Uses,
}

impl Hands {
    pub fn new(who: usize, seed: u32) -> Hands {
        Hands {
            who,
            rng: (0x2C1B_3C6D ^ seed.wrapping_mul(0x9E37_79B9) ^ (who as u32) << 9) | 1,
            kit: None,
            cool: 0,
            seq: None,
            fresh: false,
            last_bits: 0,
            second: 0,
            second_used: false,
            far_for: 0,
            answer_left: 0,
            weight_was: Fx::ZERO,
            camera: None,
            uses: Uses::default(),
        }
    }

    fn roll(&mut self, below: u32) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        if below == 0 { 0 } else { self.rng % below }
    }

    fn rest(&mut self) {
        self.cool = COOL_MIN + self.roll(COOL_SPREAD) as u16;
    }

    /// The button that throws `kind` for this class, learned by pressing.
    fn button(&mut self, me: &Player, kind: u8) -> Option<u16> {
        if self.kit.is_none_or(|k| k.class != me.class) {
            self.kit = Some(Kit::learn(me.class));
        }
        self.kit.and_then(|k| k.button(kind))
    }

    /// Can it throw `kind` now, as far as its own body says? The repeat
    /// lockout, the mechanic's gate, and -- for the two moves that stand in
    /// the world -- whether the last one is still standing.
    fn ready(&self, w: &World, me: &Player, kind: u8) -> bool {
        if !me.mechanic_ready(kind) || me.locked_out(kind) {
            return false;
        }
        let standing = match (me.class, kind) {
            (Class::Elementalist, SLOT_SPECIAL) => Some(EffectKind::FirePillar),
            (Class::ShadowReaver, SLOT_SPECIAL) => Some(EffectKind::GuillotineLotus),
            _ => None,
        };
        standing.is_none_or(|k| {
            !w.effects
                .iter()
                .flatten()
                .any(|e| e.kind == k && e.owner == self.who as u8)
        })
    }

    /// Her pools: where each stands.
    fn pools<'a>(&self, w: &'a World) -> impl Iterator<Item = V3> + 'a {
        self.pools_of(w, 0)
    }

    /// Her pools holding at least `least` essence.
    fn pools_of<'a>(&self, w: &'a World, least: i32) -> impl Iterator<Item = V3> + 'a {
        let who = self.who as u8;
        w.effects
            .iter()
            .flatten()
            .filter(move |e| e.is_a_pool() && e.owner == who && e.pool_volume() >= least)
            .map(|e| e.pos)
    }

    /// Per frame, before anything else: the clocks, and the shield's feel.
    fn sync(&mut self, me: &Player) {
        if me.class == Class::Bulwark {
            let weight = sim::bulwark::weight(me);
            if weight.raw() > self.weight_was.raw() {
                self.answer_left = 30;
            }
            self.weight_was = weight;
        }
    }

    /// **How far from what it is hitting this class hits it from.** The poke's
    /// reach -- the bolt's nine metres, for the Elementalist -- and for the
    /// Blood mage the scythe's as it is now, grey making it longer: her poke
    /// slot is the Bloodletter, a thrown blade, and a plan that walked to its
    /// seven metres swung her scythe at the air. A plan that walks to "in
    /// reach" walks this far.
    pub fn reach(&self, me: &Player) -> Fx {
        let poke = moves::get(me.class, SLOT_POKE);
        match me.class {
            Class::BloodMage => {
                sim::state::live_reach(me, &moves::get(me.class, moves::blood::SWEEP))
            }
            _ => sim::state::live_reach(me, &poke),
        }
    }

    // -----------------------------------------------------------------------
    // Hit that, now
    // -----------------------------------------------------------------------

    /// **Hit `at`, now.** `plan` is the input the plan built for this frame,
    /// with its own press in it, looking at `at`; `window` is how many frames
    /// it believes the opening has left, in the present, if it knows. The
    /// class answers with what to press instead, or the plan's own input when
    /// that is already the right thing.
    pub fn hit(
        &mut self,
        w: &World,
        me: &Player,
        at: V3,
        plan: Input,
        window: Option<i32>,
    ) -> Input {
        if plan.bits & ATTACKS == 0 || self.seq.is_some() || !me.action.actionable() {
            return plan;
        }
        let window = window.unwrap_or(0);
        match me.class {
            Class::Champion => plan,
            Class::Bulwark => {
                if self.answer_left > 0
                    && sim::bulwark::weight(me).raw() > 0
                    && plan.bits & Input::MIDDLE == 0
                {
                    self.answer_left = 0;
                    self.uses.slams += 1;
                    return Input::looking_at(
                        (plan.bits & !ATTACKS & !Input::SHIFT) | Input::MIDDLE,
                        plan.aim,
                        plan.pitch,
                    );
                }
                plan
            }
            Class::Elementalist => self.cast(w, me, at, plan, window),
            Class::ShadowReaver => self.reave(w, me, at, plan, window),
            Class::BloodMage => self.bleed(w, me, at, plan, window),
            Class::DualMage => self.punch(w, me, at, plan, window),
        }
    }

    /// The Elementalist: **the pillar where it stands** in a window long enough
    /// to plant it, **Cataclysm** in one long enough for that, and otherwise
    /// the bolt -- aimed, every one of them, through the crosshair onto the
    /// point, not levelled at the horizon.
    fn cast(&mut self, w: &World, me: &Player, at: V3, plan: Input, window: i32) -> Input {
        let far = flat(at.sub(me.pos)).flat_len();
        if me.grounded {
            let pillar = moves::get(me.class, SLOT_SPECIAL);
            if window >= fits(&pillar, far)
                && far.raw() < pillar.reach.sub(Fx::ratio(5, 10)).raw()
                && self.ready(w, me, SLOT_SPECIAL)
                && let Some(b) = self.button(me, SLOT_SPECIAL)
            {
                self.uses.pillars += 1;
                return self.look_keeping(me, plan, self.spot(w, me, floor_under(me, at), true), b);
            }
            let heavy = moves::get(me.class, SLOT_COMMITTED);
            if window >= fits(&heavy, far)
                && far.raw() < heavy.reach.sub(Fx::ONE).raw()
                && self.ready(w, me, SLOT_COMMITTED)
                && let Some(b) = self.button(me, SLOT_COMMITTED)
            {
                self.uses.cataclysms += 1;
                return self.look_keeping(me, plan, self.spot(w, me, at, false), b);
            }
        }
        self.uses.shots += 1;
        let bolt = self.button(me, SLOT_POKE).unwrap_or(Input::LEFT);
        self.look_keeping(me, plan, self.spot(w, me, at, false), bolt)
    }

    /// The Reaver: **the lotus on a shadow standing at the work**, dragged home
    /// through it; **the Executioner on a marked creature**, which cashes the
    /// marks; and otherwise her own swing, which the shadow copies from where
    /// it stands. The shadow is sent between openings ([`Hands::idle`]), not
    /// in one: the send is nineteen frames the window was for, and sending it
    /// into the Hornback's stun -- a long window -- halved her wins there.
    fn reave(&mut self, w: &World, me: &Player, at: V3, plan: Input, window: i32) -> Input {
        let Some(shadow) = sim::shadow::of(me) else {
            return plan;
        };
        let near = flat(shadow.pos.sub(at)).flat_len().raw() < SHADOW_NEAR.raw();
        let lotus = moves::get(me.class, SLOT_SPECIAL);
        if shadow.is_out()
            && near
            && window >= fits(&lotus, flat(at.sub(me.pos)).flat_len())
            && self.ready(w, me, SLOT_SPECIAL)
            && let Some(b) = self.button(me, SLOT_SPECIAL)
        {
            self.uses.lotuses += 1;
            self.seq = Some(Seq::Later {
                bits: Input::RIGHT,
                left: LOTUS_DRAG + self.roll(8) as u16,
            });
            self.fresh = true;
            return self.look_keeping(me, plan, at, b);
        }
        let far = flat(at.sub(me.pos)).flat_len();
        let exe = moves::get(me.class, SLOT_COMMITTED);
        if marks_near(w, at) >= CASH_AT
            && window >= busy(&exe)
            && far.raw() < exe.reach.add(exe.step).add(Fx::ratio(8, 10)).raw()
            && self.ready(w, me, SLOT_COMMITTED)
            && let Some(b) = self.button(me, SLOT_COMMITTED)
        {
            self.uses.cashed += 1;
            return self.look_keeping(me, plan, at, b);
        }
        plan
    }

    /// The Blood mage: **the spike on a pool under it**, which drinks the pool
    /// whole and comes up half again as hard; **the Grasp** from range in a
    /// long window, which hauls her to it; **the cut** from beyond the
    /// scythe; and the scythe, which drinks what it passes over.
    fn bleed(&mut self, w: &World, me: &Player, at: V3, plan: Input, window: i32) -> Input {
        let red = me.health * 100 / me.full_health().max(1);
        let far = flat(at.sub(me.pos)).flat_len();
        let spike = moves::get(me.class, moves::blood::BLACK_SPIKE);
        if red >= RED_FLOOR
            && window >= (spike.startup + spike.active) as i32 + SPARE
            && let Some(pool) = self.pool_under(w, at)
            && flat(pool.sub(me.pos)).flat_len().raw() < spike.reach.sub(Fx::ratio(5, 10)).raw()
            && !self.on_top(w, me, pool)
            && self.ready(w, me, moves::blood::BLACK_SPIKE)
            && let Some(b) = self.button(me, moves::blood::BLACK_SPIKE)
        {
            self.uses.spikes += 1;
            self.uses.erupted += 1;
            return self.look_keeping(me, plan, pool, b);
        }
        let sweep = sim::state::live_reach(me, &moves::get(me.class, moves::blood::SWEEP));
        if let Some(input) = self.grasp(w, me, at, plan, window, red) {
            return input;
        }
        let cut = moves::get(me.class, moves::blood::HAEMORRHAGE);
        if red >= RED_CUT
            && far.raw() > sweep.add(Fx::ratio(5, 10)).raw()
            && far.raw() < cut.reach.raw()
            && window >= (cut.startup + cut.active) as i32 + SPARE
            && self.ready(w, me, moves::blood::HAEMORRHAGE)
            && let Some(b) = self.button(me, moves::blood::HAEMORRHAGE)
        {
            self.uses.cuts += 1;
            return self.look_keeping(me, plan, self.spot(w, me, at, false), b);
        }
        // **The scythe, not the plan's heavy**, in its reach: a plan's heavy
        // is her Haemorrhage, four in a hundred of her red for thirty against
        // something that does not bleed, where the scythe costs one and
        // drinks the pools under it.
        // The scythe is her right click since 2026-10-09; asked of the kit
        // rather than written down, as every other button here is.
        let scythe = self.button(me, moves::blood::SWEEP).unwrap_or(Input::LEFT);
        if plan.bits & ATTACKS != scythe {
            if far.raw() < sweep.add(Fx::ratio(5, 10)).raw() {
                return Input::looking_at((plan.bits & !ATTACKS) | scythe, plan.aim, plan.pitch);
            }
            if red < RED_CUT {
                return Input::looking_at(plan.bits & !ATTACKS, plan.aim, plan.pitch);
            }
        }
        plan
    }

    /// The Grasp, if it is worth holding: from beyond the scythe, in a long
    /// window, with red to spare. **Against a creature it is a way in and
    /// little else**: the arms cannot haul it, so they haul her to it, and
    /// what they deal is one arm's worth -- forty or fifty, measured on the
    /// Pair, for seven in a hundred of her red. The hold sets the depth, so
    /// it is held for the frames that put the arms' meeting point on `at`.
    fn grasp(
        &mut self,
        w: &World,
        me: &Player,
        at: V3,
        plan: Input,
        window: i32,
        red: i32,
    ) -> Option<Input> {
        let grasp = moves::get(me.class, moves::blood::GRASP);
        let far = flat(at.sub(me.pos)).flat_len();
        let far_end = grasp.reach;
        // The hold whose reach is nearest the point.
        let frames = (1..=grasp.channel)
            .min_by_key(|h| grasp.reach_after(*h).sub(far).abs().raw())
            .unwrap_or(1);
        if red < RED_GRASP
            || far.raw() < Fx::from_int(3).raw()
            || far.raw() > far_end.sub(Fx::ratio(5, 10)).raw()
            || window < frames as i32 + (grasp.startup + grasp.active) as i32 + LONG_WINDOW
            || self.cool > 0
            || !self.ready(w, me, moves::blood::GRASP)
        {
            return None;
        }
        let b = self.button(me, moves::blood::GRASP)?;
        self.uses.grasps += 1;
        self.rest();
        self.seq = Some(Seq::Hold {
            bits: b,
            at,
            left: frames,
        });
        self.fresh = true;
        Some(self.look_keeping(me, plan, at, b))
    }

    /// The nearest of her pools to `at`, if one lies under it.
    fn pool_under(&self, w: &World, at: V3) -> Option<V3> {
        self.pools_of(w, SPIKE_POOL)
            .map(|p| (p, flat(p.sub(at)).flat_len()))
            .filter(|(_, d)| d.raw() < POOL_UNDER.raw())
            .min_by_key(|(_, d)| d.raw())
            .map(|(p, _)| p)
    }

    /// The Dual mage: **a finisher or a sweep in a long window**, if it does
    /// not push her bars past the band; otherwise the auto, which `finish`
    /// puts in the hand that keeps the bars level -- and turned so the punch
    /// leaving that shoulder meets the point rather than passing beside it.
    fn punch(&mut self, w: &World, me: &Player, at: V3, plan: Input, window: i32) -> Input {
        let far = flat(at.sub(me.pos)).flat_len();
        for kind in [moves::dual::JUDGEMENT, moves::dual::SWEEP] {
            let m = moves::get(me.class, kind);
            let reach = sim::state::live_reach(me, &m);
            if window >= (m.startup + m.active) as i32 + SPARE
                && far.raw() < reach.raw()
                && keeps(me, kind)
                && self.ready(w, me, kind)
                && let Some(b) = self.button(me, kind)
            {
                self.uses.finishers += 1;
                let point = if m.aim().is_a_skillshot() || kind == moves::dual::JUDGEMENT {
                    floor_under(me, at)
                } else {
                    at
                };
                return self.look_keeping(me, plan, self.spot(w, me, point, true), b);
            }
        }
        if plan.bits & (Input::LEFT | Input::RIGHT) == 0 {
            return plan;
        }
        let (bits, kind) = self.auto_for(me);
        if bits == 0 {
            return Input::looking_at(plan.bits & !ATTACKS, plan.aim, plan.pitch);
        }
        if plan.bits & (Input::LEFT | Input::RIGHT) != bits {
            self.uses.turned += 1;
        }
        // The punch leaves the shoulder: turn so its line meets the point.
        let hand = moves::get(me.class, kind).hand;
        let yaw = yaw_of(at.sub(me.pos));
        let shoulder = sim::aim::hand_origin(me.pos, V3::from_turns(yaw), hand);
        let yaw = yaw_of(at.sub(shoulder));
        let wire = turns_to_aim(yaw.sub(me.carry_yaw));
        let held = plan.bits & !ATTACKS & !(Input::W | Input::A | Input::S | Input::D);
        Input::looking_at(held | bits | rekey(me, plan, wire), wire, plan.pitch)
    }

    /// The auto that keeps her bars level: the low side's, unless that would
    /// burn her, then the other, and neither if both would. With its move.
    fn auto_for(&self, me: &Player) -> (u16, u8) {
        let low_dark = sim::dual::bars(me).is_none_or(|(d, l)| d.raw() <= l.raw());
        let order = if low_dark {
            [
                (Input::LEFT, moves::dual::DARK_AUTO),
                (Input::RIGHT, moves::dual::LIGHT_AUTO),
            ]
        } else {
            [
                (Input::RIGHT, moves::dual::LIGHT_AUTO),
                (Input::LEFT, moves::dual::DARK_AUTO),
            ]
        };
        order
            .into_iter()
            .find(|(_, k)| keeps(me, *k))
            .unwrap_or((0, moves::dual::DARK_AUTO))
    }

    // -----------------------------------------------------------------------
    // You have a moment
    // -----------------------------------------------------------------------

    /// **Between openings**, holding station: `beast` is where it last saw the
    /// creature, `at` the point it will go for next, and `safe` how many
    /// frames it believes it has before it must answer anything (`i32::MAX`
    /// when nothing is coming). The class may take the frame, and the plan
    /// carries on next frame from wherever it is.
    pub fn idle(&mut self, w: &World, me: &Player, beast: V3, at: V3, safe: i32) -> Option<Input> {
        if self.seq.is_some()
            || !me.action.actionable()
            || !me.grounded
            || me.aboard()
            || !self.sees(me, at)
        {
            return None;
        }
        let far = flat(at.sub(me.pos)).flat_len();
        match me.class {
            Class::Champion | Class::Bulwark => None,
            Class::ShadowReaver => self.idle_reaver(w, me, at, far, safe),
            Class::Elementalist => self.idle_caster(w, me, beast, at, far, safe),
            Class::BloodMage => self.idle_blood(w, me, beast, at, far, safe),
            Class::DualMage => self.idle_dual(w, me, at, safe),
        }
    }

    /// The shadow out beside the work and kept there; called home when the
    /// work has moved on; a swing at nothing now and then, so its copy marks
    /// the creature from where it stands.
    fn idle_reaver(&mut self, w: &World, me: &Player, at: V3, far: Fx, safe: i32) -> Option<Input> {
        let shadow = sim::shadow::of(me)?;
        let send = moves::get(me.class, SLOT_MECHANIC);
        let b = self.button(me, SLOT_MECHANIC)?;
        if !shadow.is_out() {
            self.far_for = 0;
            if far.raw() < send.reach.sub(Fx::ratio(5, 10)).raw()
                && safe > (send.startup + send.active + send.recovery) as i32
                && self.cool == 0
                && self.ready(w, me, SLOT_MECHANIC)
                && self.last_bits & b == 0
            {
                self.uses.sent += 1;
                self.rest();
                return Some(self.look(me, self.spot(w, me, floor_under(me, at), true), b));
            }
            return None;
        }
        let from = flat(shadow.pos.sub(at)).flat_len();
        if from.raw() > SHADOW_FAR.raw() {
            self.far_for = self.far_for.saturating_add(1);
        } else {
            self.far_for = 0;
        }
        if self.far_for > SHADOW_STALE && self.last_bits & b == 0 && shadow.is_waiting() {
            // Home, through whatever is in the way.
            self.far_for = 0;
            self.uses.recalls += 1;
            self.rest();
            return Some(self.look(me, at, b));
        }
        let slash = moves::get(me.class, SLOT_POKE);
        if from.raw() < SHADOW_NEAR.raw()
            && shadow.is_waiting()
            && far.raw() > slash.reach.add(Fx::ONE).raw()
            && safe > (slash.startup + slash.active + slash.recovery) as i32 + SPARE
            && self.cool == 0
        {
            // At nothing, for the copy: it turns to the creature beside it.
            self.uses.copies += 1;
            self.rest();
            return Some(self.look(me, at, Input::LEFT));
        }
        None
    }

    /// From range: **the pillar where it stands**, and the bolt at the point.
    fn idle_caster(
        &mut self,
        w: &World,
        me: &Player,
        beast: V3,
        at: V3,
        far: Fx,
        safe: i32,
    ) -> Option<Input> {
        if self.cool > 0 {
            return None;
        }
        let pillar = moves::get(me.class, SLOT_SPECIAL);
        let under = flat(beast.sub(me.pos)).flat_len();
        if safe > (pillar.startup + pillar.active + pillar.recovery) as i32
            && under.raw() < pillar.reach.sub(Fx::ratio(5, 10)).raw()
            && self.ready(w, me, SLOT_SPECIAL)
            && let Some(b) = self.button(me, SLOT_SPECIAL)
        {
            self.uses.pillars += 1;
            self.rest();
            return Some(self.look(me, self.spot(w, me, floor_under(me, at), true), b));
        }
        let bolt = moves::get(me.class, SLOT_POKE);
        if safe > (bolt.startup + bolt.active + bolt.recovery) as i32
            && far.raw() < bolt.reach.sub(Fx::ratio(5, 10)).raw()
            && self.ready(w, me, SLOT_POKE)
        {
            self.uses.shots += 1;
            self.rest();
            let b = self.button(me, SLOT_POKE).unwrap_or(Input::LEFT);
            return Some(self.look(me, self.spot(w, me, at, false), b));
        }
        None
    }

    /// From range: **the spike on a pool under it** -- which heals her -- and,
    /// with red to spare, the cut that makes the next pool.
    fn idle_blood(
        &mut self,
        w: &World,
        me: &Player,
        beast: V3,
        at: V3,
        far: Fx,
        safe: i32,
    ) -> Option<Input> {
        if self.cool > 0 {
            return None;
        }
        let red = me.health * 100 / me.full_health().max(1);
        let spike = moves::get(me.class, moves::blood::BLACK_SPIKE);
        let pool = self.pool_under(w, at).or_else(|| {
            self.pools_of(w, SPIKE_POOL)
                .find(|p| flat(p.sub(beast)).flat_len().raw() < Fx::ratio(25, 10).raw())
        });
        if red >= RED_FLOOR
            && safe > (spike.startup + spike.active + spike.recovery) as i32
            && let Some(pool) = pool
            && flat(pool.sub(me.pos)).flat_len().raw() < spike.reach.sub(Fx::ratio(5, 10)).raw()
            && !self.on_top(w, me, pool)
            && self.ready(w, me, moves::blood::BLACK_SPIKE)
            && let Some(b) = self.button(me, moves::blood::BLACK_SPIKE)
        {
            self.uses.spikes += 1;
            self.uses.erupted += 1;
            self.rest();
            return Some(self.look(me, pool, b));
        }
        let cut = moves::get(me.class, moves::blood::HAEMORRHAGE);
        if red >= RED_CUT + 10
            && safe > (cut.startup + cut.active + cut.recovery) as i32
            && far.raw() < cut.reach.sub(Fx::ONE).raw()
            && self.ready(w, me, moves::blood::HAEMORRHAGE)
            && let Some(b) = self.button(me, moves::blood::HAEMORRHAGE)
        {
            self.uses.cuts += 1;
            self.rest();
            return Some(self.look(me, self.spot(w, me, at, false), b));
        }
        None
    }

    /// **The bars goaded up**, a hand at a time, while there is nothing to
    /// hit: half is the blink, three quarters the second jump.
    fn idle_dual(&mut self, w: &World, me: &Player, at: V3, safe: i32) -> Option<Input> {
        let _ = w;
        let auto = moves::get(me.class, moves::dual::DARK_AUTO);
        let lower = sim::dual::lower(me).to_int();
        if self.cool > 0
            || lower >= GOAD_TO
            || safe <= (auto.startup + auto.active + auto.recovery) as i32 + SPARE
            || sim::dual::ascending(me)
        {
            return None;
        }
        let (bits, _) = self.auto_for(me);
        if bits == 0 || self.last_bits & bits != 0 {
            return None;
        }
        self.uses.goads += 1;
        self.cool = (auto.startup + auto.active + auto.recovery) + 4 + self.roll(8) as u16;
        Some(self.look(me, at, bits))
    }

    // -----------------------------------------------------------------------
    // Go in, get out, take it on the shield
    // -----------------------------------------------------------------------

    /// **Go in on this window**, the class's way, if it has one from here: the
    /// Reaver dashes to a shadow already standing at `at` and swings out of
    /// the carry -- the Executioner, if the creature carries her marks; the
    /// Blood mage blinks to a pool there. `None` and the plan walks.
    pub fn close_in(&mut self, w: &World, me: &Player, at: V3, window: i32) -> Option<Input> {
        if self.seq.is_some() || !me.action.actionable() || !me.grounded || me.aboard() {
            return None;
        }
        let far = flat(at.sub(me.pos)).flat_len();
        if far.raw() < HOP_NEAR.add(Fx::ONE).raw() {
            return None;
        }
        let dodge = sim::tuning::dodge_frames() as i32;
        match me.class {
            Class::ShadowReaver => {
                let shadow = sim::shadow::of(me)?;
                if !shadow.is_waiting()
                    || !self.sees(me, shadow.pos)
                    || flat(shadow.pos.sub(at)).flat_len().raw() > Fx::ratio(25, 10).raw()
                    || flat(shadow.pos.sub(me.pos)).flat_len().raw() > HOP_FAR.raw()
                {
                    return None;
                }
                let exe = moves::get(me.class, SLOT_COMMITTED);
                let cash = marks_near(w, at) >= CASH_AT
                    && window > dodge / 2 + (exe.startup + exe.active) as i32 + SPARE
                    && self.ready(w, me, SLOT_COMMITTED);
                let slash = moves::get(me.class, SLOT_POKE);
                if !cash && window <= dodge / 2 + (slash.startup + slash.active) as i32 + SPARE {
                    return None;
                }
                let then = if cash {
                    self.uses.cashed += 1;
                    self.button(me, SLOT_COMMITTED).unwrap_or(Input::LEFT)
                } else {
                    Input::LEFT
                };
                self.uses.dashes += 1;
                self.seq = Some(Seq::Then {
                    then,
                    at,
                    left: dodge as u16 + 6,
                });
                self.fresh = true;
                Some(self.look(me, chest(shadow.pos), Input::SHIFT | Input::W))
            }
            Class::BloodMage => {
                // **The Grasp is her way in**: the arms close on something
                // that cannot be hauled and haul her to it instead, and the
                // pool they spill is a spike's worth for the next window.
                let red = me.health * 100 / me.full_health().max(1);
                if let Some(input) = self.grasp(w, me, at, self.look(me, at, 0), window, red) {
                    return Some(input);
                }
                if window <= dodge / 2 + 14 {
                    return None;
                }
                let pool = self
                    .pools(w)
                    .filter(|p| flat(p.sub(at)).flat_len().raw() < Fx::ratio(25, 10).raw())
                    .filter(|p| self.sees(me, *p))
                    .filter(|p| {
                        let d = flat(p.sub(me.pos)).flat_len();
                        d.raw() > HOP_NEAR.raw() && d.raw() < HOP_FAR.raw()
                    })
                    .min_by_key(|p| flat(p.sub(at)).flat_len().raw())?;
                self.uses.blinks += 1;
                self.seq = Some(Seq::Then {
                    then: Input::LEFT,
                    at,
                    left: dodge as u16,
                });
                self.fresh = true;
                Some(self.look(me, pool, Input::SHIFT | Input::W))
            }
            Class::Bulwark => self.throw_in(me, at, far, window),
            _ => None,
        }
    }

    /// **The Bulwark's way in from range** (`kits/bulwark.md`, Throw /
    /// Recall: *throw to commit, leap to follow*): the shield thrown at the
    /// work, the leap to it while it flies -- it turns and meets him, so he
    /// arrives in the air with it in hand -- and a Slam out of the leap,
    /// which lands with the feet and is paid for the fall. On a window long
    /// enough for all of it, from five to eleven metres, with the shield in
    /// his hand and the work on his screen.
    ///
    /// The thrown shield does not strike a creature (only a fighter, in
    /// versus), so this is a way across the floor and not a ranged blow.
    fn throw_in(&mut self, me: &Player, at: V3, far: Fx, window: i32) -> Option<Input> {
        if self.cool > 0
            || !me.shield().is_some_and(|s| s.in_hand())
            || far.raw() < LEAP_NEAR.raw()
            || far.raw() > LEAP_FAR.raw()
            || self.last_bits & Input::MECHANIC != 0
            || !self.sees(me, at)
        {
            return None;
        }
        let slam = moves::get(me.class, SLOT_COMMITTED);
        let need = LEAP_WAIT as i32 + LEAP_AIR + (slam.startup + slam.active) as i32 + SPARE;
        if window < need {
            return None;
        }
        self.uses.throws += 1;
        self.cool = LEAP_REST + self.roll(LEAP_REST_SPREAD) as u16;
        self.seq = Some(Seq::Leap {
            at,
            wait: LEAP_WAIT,
            leapt: false,
            left: LEAP_GIVE_UP,
        });
        self.fresh = true;
        Some(self.look(me, at, Input::MECHANIC))
    }

    /// **Get out**, along `out`. `plan` is the dodge the plan was about to
    /// throw. The Reaver dashes to her shadow instead if it stands that way --
    /// invulnerable across the gap, and further than a dodge goes; the Blood
    /// mage blinks to a pool lying that way. Anything else dodges.
    pub fn leave(&mut self, w: &World, me: &Player, out: V3, plan: Input) -> Input {
        if !me.action.actionable() || me.aboard() {
            return plan;
        }
        let out = flat(out).normalized();
        let lies = |to: V3| {
            let d = flat(to.sub(me.pos));
            let len = d.flat_len();
            len.raw() > HOP_NEAR.raw()
                && len.raw() < HOP_FAR.raw()
                && d.normalized().dot(out).raw() > ALONG.raw()
                && self.sees(me, to)
        };
        match me.class {
            Class::ShadowReaver => {
                if let Some(shadow) = sim::shadow::of(me)
                    && shadow.is_waiting()
                    && lies(shadow.pos)
                {
                    self.uses.dashes += 1;
                    self.seq = None;
                    return self.look(me, chest(shadow.pos), Input::SHIFT | Input::W);
                }
                plan
            }
            Class::BloodMage => {
                if let Some(pool) = self.pools(w).find(|p| lies(*p)) {
                    self.uses.blinks += 1;
                    self.seq = None;
                    return self.look(me, pool, Input::SHIFT | Input::W);
                }
                plan
            }
            _ => plan,
        }
    }

    /// **A blockable blow is coming** from `toward`, and lands within `frames`:
    /// the Bulwark takes it on the shield, if the shield is in his hand,
    /// raising it now and holding it through the hit -- where a plan was about
    /// to dodge, the guard goes up on the same frame, so its first frames are
    /// the parry. `None` for everybody else, and the plan dodges. The blow
    /// loads the shield, and the next [`Hands::hit`] answers it with Slam.
    pub fn guard(
        &mut self,
        me: &Player,
        blockable: bool,
        toward: V3,
        frames: u16,
    ) -> Option<Input> {
        if me.class != Class::Bulwark
            || !blockable
            || !me.shield().is_some_and(|s| s.in_hand())
            || !(me.action.actionable() || me.action.guarding())
        {
            return None;
        }
        if !me.action.guarding() {
            self.uses.guards += 1;
        }
        let at = chest(toward);
        self.seq = Some(Seq::Hold {
            bits: Input::RIGHT,
            at,
            left: frames.max(1),
        });
        self.fresh = true;
        Some(self.look(me, at, Input::RIGHT))
    }

    // -----------------------------------------------------------------------
    // Every frame
    // -----------------------------------------------------------------------

    /// **The last word on every frame's input**, after the plan: carry a press
    /// that lasts longer than a frame, keep the Dual mage's hands on the side
    /// that does not burn her, and give her the second jump when the plan is
    /// holding the button for height. The Champion's input comes out as it
    /// went in.
    pub fn finish(&mut self, w: &World, me: &Player, input: Input) -> Input {
        self.sync(me);
        self.cool = self.cool.saturating_sub(1);
        self.answer_left = self.answer_left.saturating_sub(1);
        let mut out = input;
        if me.class == Class::Champion {
            self.last_bits = out.bits;
            return out;
        }
        if self.fresh {
            self.fresh = false;
        } else if let Some(seq) = self.seq {
            out = self.carry(w, me, seq, out);
        }
        if me.class == Class::DualMage {
            out = self.hands_level(me, out);
            out = self.second_jump(me, out);
        }
        if me.class == Class::Bulwark {
            out = self.shield_home(me, out);
        }
        self.last_bits = out.bits;
        out
    }

    /// **A shield left planted comes home.** Every Bulwark move but the throw
    /// needs it in his hand -- the guard, the Bash, the Slam -- so a shield
    /// out of a leap given up, or knocked out of it, is recalled on the next
    /// frame he is free and not dodging. Nothing he does plants it on purpose.
    fn shield_home(&mut self, me: &Player, out: Input) -> Input {
        if self.seq.is_none()
            && matches!(me.shield(), Some(Shield::Planted { .. }))
            && me.action.actionable()
            && out.bits & (ATTACKS | Input::SHIFT) == 0
            && self.last_bits & Input::MECHANIC == 0
        {
            self.uses.shield_recalls += 1;
            return Input::looking_at(out.bits | Input::MECHANIC, out.aim, out.pitch);
        }
        out
    }

    /// One frame of a press that lasts.
    fn carry(&mut self, w: &World, me: &Player, seq: Seq, plan: Input) -> Input {
        let _ = w;
        match seq {
            Seq::Hold { bits, at, left } => {
                if left == 0 {
                    self.seq = None;
                    return self.look_keeping(me, plan, at, 0);
                }
                self.seq = Some(Seq::Hold {
                    bits,
                    at,
                    left: left - 1,
                });
                self.look_keeping(me, plan, at, bits)
            }
            Seq::Later { bits, left } => {
                if left == 0 {
                    self.seq = None;
                    if sim::shadow::of(me).is_some_and(|s| s.is_out()) {
                        self.uses.recalls += 1;
                        return Input::looking_at(plan.bits | bits, plan.aim, plan.pitch);
                    }
                    return plan;
                }
                self.seq = Some(Seq::Later {
                    bits,
                    left: left - 1,
                });
                plan
            }
            Seq::Then { then, at, left } => {
                let carrying = sim::shadow::carrying_a_dash(me);
                if carrying || (me.action.actionable() && left + 4 < sim::tuning::dodge_frames()) {
                    self.seq = None;
                    if me.action.actionable() || carrying {
                        return self.look_keeping(me, plan, at, then);
                    }
                    return plan;
                }
                if left == 0 {
                    self.seq = None;
                    return plan;
                }
                self.seq = Some(Seq::Then {
                    then,
                    at,
                    left: left - 1,
                });
                // Riding the dodge: nothing to press but the way it is going.
                Input::looking_at(0, plan.aim, plan.pitch)
            }
            Seq::Leap {
                at,
                wait,
                leapt,
                left,
            } => {
                // A dodge the plan asks for comes first: the leap is given
                // up, and the shield comes home after (`shield_home`).
                if left == 0 || plan.bits & Input::SHIFT != 0 {
                    self.seq = None;
                    return plan;
                }
                let next = |wait, leapt| Seq::Leap {
                    at,
                    wait,
                    leapt,
                    left: left - 1,
                };
                match me.shield() {
                    Some(Shield::Flying { outbound: true, .. }) if !leapt => {
                        if wait > 0 {
                            self.seq = Some(next(wait - 1, false));
                            return self.look_keeping(me, plan, at, 0);
                        }
                        self.uses.leaps += 1;
                        self.seq = Some(next(0, true));
                        self.look_keeping(me, plan, at, Input::MECHANIC)
                    }
                    Some(Shield::Held { .. }) if leapt => {
                        if !me.grounded && me.action.actionable() {
                            self.seq = None;
                            self.uses.leap_slams += 1;
                            return self.look_keeping(me, plan, at, Input::MIDDLE);
                        }
                        if me.grounded {
                            // Landed with it before a Slam could start.
                            self.seq = None;
                            return plan;
                        }
                        self.seq = Some(next(0, true));
                        self.look_keeping(me, plan, at, 0)
                    }
                    Some(Shield::Flying { .. }) => {
                        // Coming home to meet him.
                        self.seq = Some(next(wait, leapt));
                        self.look_keeping(me, plan, at, 0)
                    }
                    _ => {
                        // Planted already, or in hand without a leap: the
                        // throw is over, and `shield_home` has it.
                        self.seq = None;
                        plan
                    }
                }
            }
        }
    }

    /// The Dual mage's autos into the hand that keeps her bars level, and any
    /// cast that would burn her into an auto that does not.
    fn hands_level(&mut self, me: &Player, input: Input) -> Input {
        let autos = Input::LEFT | Input::RIGHT;
        let casts = Input::MIDDLE | Input::SPECIAL | Input::MECHANIC;
        let mut bits = input.bits;
        // A cast the bars cannot take is an auto instead.
        if bits & casts != 0 {
            let mut keep = 0;
            for (b, kind) in [
                (Input::MIDDLE, moves::dual::LIGHT_LANCE),
                (Input::SPECIAL, moves::dual::JUDGEMENT),
                (Input::MECHANIC, moves::dual::SWEEP),
            ] {
                if bits & b != 0 && keeps(me, kind) {
                    keep |= b;
                }
            }
            if keep != bits & casts {
                bits = (bits & !casts) | keep;
                if keep == 0 {
                    bits |= Input::LEFT;
                }
            }
        }
        if bits & autos == 0 {
            return Input { bits, ..input };
        }
        let (want, _) = self.auto_for(me);
        if bits & autos != want {
            // Counted once a press, not once a held frame.
            if self.last_bits & autos == 0 {
                self.uses.turned += 1;
            }
            bits = (bits & !autos) | want;
        }
        Input { bits, ..input }
    }

    /// The second jump, off the three-quarter tier: a plan holding jump in the
    /// air on the way down wants height, and this is where she has more.
    fn second_jump(&mut self, me: &Player, input: Input) -> Input {
        if me.grounded {
            self.second_used = false;
            self.second = 0;
            return input;
        }
        match self.second {
            2 => {
                self.second = 1;
                return Input {
                    bits: input.bits & !Input::SPACE,
                    ..input
                };
            }
            1 => {
                self.second = 0;
                return Input {
                    bits: input.bits | Input::SPACE,
                    ..input
                };
            }
            _ => {}
        }
        if !self.second_used
            && input.bits & Input::SPACE != 0
            && me.vel.y.raw() < 0
            && !me.aboard()
            && sim::dual::tier(me) >= sim::dual::Tier::Jump
            && !sim::dual::ascending(me)
        {
            self.second_used = true;
            self.second = 1;
            self.uses.second_jumps += 1;
            return Input {
                bits: input.bits & !Input::SPACE,
                ..input
            };
        }
        input
    }

    /// Look at `at`, crosshair on it, pressing `bits`. **With a camera**
    /// (see [`Hands::camera`]) the mouse goes from where the camera is by
    /// a flick at most, the way a hand does, and the crosshair lands where
    /// that leaves it.
    fn look(&self, me: &Player, at: V3, bits: u16) -> Input {
        let d = flat(at.sub(me.pos));
        let mut yaw = if d.flat_len().raw() > Fx::ratio(2, 10).raw() {
            yaw_of(d)
        } else {
            yaw_of(me.facing)
        };
        if let Some(camera) = self.camera {
            let off = wrap_turns(yaw.sub(camera)).clamp(FLICK.neg(), FLICK);
            yaw = camera.add(off);
        }
        let wire = turns_to_aim(yaw.sub(me.carry_yaw));
        let pitch = sim::aim::look_onto_closely(me.pos, wire, me.aloft, at);
        Input::looking_at(bits, wire, pitch)
    }

    /// [`Hands::look`], keeping the plan's walk and the rest of its held keys.
    fn look_keeping(&self, me: &Player, plan: Input, at: V3, bits: u16) -> Input {
        let looked = self.look(me, at, 0);
        let held =
            plan.bits & !ATTACKS & !Input::SHIFT & !(Input::W | Input::A | Input::S | Input::D);
        Input::looking_at(
            bits | held | rekey(me, plan, looked.aim),
            looked.aim,
            looked.pitch,
        )
    }

    /// Would the crosshair, put on `at`, land on the top of a creature's
    /// part? Since aim A3 (`aiming.md`, the Siegeshell) the top of a part
    /// you could stand on, seen from above, is a place: a pillar aimed
    /// through a beached worm at the floor under it is planted on its back,
    /// over nothing, and a bolt aimed at it flies to a point above it. A
    /// player sees where the marker would go and moves the mouse; this asks
    /// the same ray (`aim::sight`).
    fn on_top(&self, w: &World, me: &Player, at: V3) -> bool {
        let stones = sim::stones::gather(&w.players);
        let ground = w.terrain();
        let scene = sim::aim::Scene {
            stones: &stones,
            players: &w.players,
            effects: &w.effects,
            quarry: &w.monsters,
            critters: &w.critters,
            arena: &ground,
        };
        let look = self.look(me, at, 0);
        sim::aim::sight(self.who, look, SIGHT_REACH, &scene).aboard
    }

    /// **Where to put the crosshair** for something aimed at `at`: there, unless
    /// that lands on top of the creature (see [`Hands::on_top`]). Then, for a
    /// thing planted on the floor, the floor nearer her, half a metre at a
    /// time -- the pillar's base spreads into what stands beside it; for a
    /// shot, lower on its near side, so the ray goes through the body to
    /// the ground behind it as it always did.
    fn spot(&self, w: &World, me: &Player, at: V3, grounded: bool) -> V3 {
        if !self.on_top(w, me, at) {
            return at;
        }
        if me.aboard() {
            // **Riding it**, the part under her feet is where she stands: a
            // shot at it goes to a body's height above it and over it. A
            // rider shooting down into her mount aims past its edge, at the
            // floor beyond, and the line down to there goes through it.
            // Planting something there is planting it on the back, which is
            // what she meant.
            if grounded {
                return at;
            }
            let out = flat(at.sub(me.pos)).normalized();
            let out = if out.flat_len().raw() == 0 {
                V3::from_turns(me.carry_yaw.add(yaw_of(me.facing)))
            } else {
                out
            };
            for k in 1..=12 {
                let p = V3::new(at.x, Fx::ZERO.min(at.y), at.z).add(out.scale(Fx::from_int(k)));
                if !self.on_top(w, me, p) {
                    return p;
                }
            }
            return at;
        }

        let back = flat(me.pos.sub(at)).normalized();
        let floor = me.pos.y.min(at.y);
        for k in 1..=8 {
            let step = Fx::ratio(k, 2);
            let p = if grounded {
                at.add(back.scale(step))
            } else {
                V3::new(
                    at.x,
                    at.y.sub(step.mul(Fx::ratio(8, 10)))
                        .max(floor.add(Fx::ratio(3, 10))),
                    at.z,
                )
                .add(back.scale(step.mul(Fx::ratio(4, 10))))
            };
            if !self.on_top(w, me, p) {
                return p;
            }
        }
        at
    }

    /// Is `at` on its screen, for a plan that keeps a camera? Always, for
    /// one that does not.
    fn sees(&self, me: &Player, at: V3) -> bool {
        let Some(camera) = self.camera else {
            return true;
        };
        let d = flat(at.sub(me.pos));
        d.flat_len().raw() < Fx::ratio(5, 10).raw()
            || wrap_turns(yaw_of(d).sub(camera)).abs().raw() < crate::report::HALF_VIEW.raw()
    }

    /// **Where the plan's camera is pointing**, in world turns, for a plan
    /// that keeps one -- the Pair's, the Veilstalker's, the Mantis's and the
    /// Galewing's hunters turn theirs at a person's rate and know only what
    /// is on it. The class layer then acts only on what is on that screen,
    /// and moves the mouse off it by a flick at most. Set it every frame;
    /// `None` is a plan that looks wherever it likes.
    pub fn camera(&mut self, yaw: Option<Fx>) {
        self.camera = yaw;
    }
}

/// **Does this move keep the Dual mage's bars where she wants them?** The
/// sparring bot's rule (`duel::dual_bias`: not past the band, where she
/// burns) and one more for a hunt: **not into ascension.** Both bars full is
/// six seconds of wings that drain her two a frame and pay back only in hits,
/// and against a creature the hits come a window at a time -- the first
/// hunts that let her climb there lost seven hundred health to it. With the
/// lower bar about to reach the top, she pushes the higher one.
fn keeps(me: &Player, kind: u8) -> bool {
    if dual_bias(me, kind) == 0 {
        return false;
    }
    if sim::dual::ascending(me) {
        return true;
    }
    let wings = Fx::from_int(sim::tuning::tier_wings());
    dual_after(me, kind).is_none_or(|(d, l, _)| d.min(l).raw() < wings.raw())
}

/// Marks the Reaver has on whatever creature stands nearest `at`: drawn over
/// it as pips, so something anybody can see.
fn marks_near(w: &World, at: V3) -> u8 {
    w.monsters
        .iter()
        .flatten()
        .filter(|m| m.alive())
        .min_by_key(|m| flat(m.pos.sub(at)).flat_len().raw())
        .map_or(0, |m| m.marks)
}

/// Every frame a move keeps its thrower busy: what an opening has to hold
/// for the move to be over before the creature can act again -- the rule
/// the plans keep for their own heavy (`crate::heavy_commitment`).
fn busy(m: &sim::Move) -> i32 {
    (m.startup + m.active + m.recovery) as i32
}

/// What an opening has to hold for a move thrown from `far`: all of it up
/// close, where whatever the creature starts next reaches; from range, its
/// startup and its hit and a few frames -- the recovery is spent where
/// nothing the creature starts in the time can arrive. (Half the recovery
/// as well was tried: the Elementalist won two Ridgeback hunts in 24 for
/// eight, its windows too short for a pillar.)
fn fits(m: &sim::Move, far: Fx) -> i32 {
    if far.raw() < RANGED.raw() {
        busy(m)
    } else {
        (m.startup + m.active) as i32 + SPARE
    }
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn yaw_of(v: V3) -> Fx {
    atan2_turns(v.z, v.x)
}

/// Somewhere about the middle of a standing body at `feet`: what the
/// crosshair goes on to point at one.
fn chest(feet: V3) -> V3 {
    sim::aim::standing_middle(feet, sim::tuning::body_height())
}

/// The floor under a point, at the height of the floor the hunter stands on
/// or the point's own, whichever is lower: where a grounded cast is aimed.
fn floor_under(me: &Player, at: V3) -> V3 {
    V3::new(at.x, me.pos.y.min(at.y), at.z)
}

/// The walking keys of `plan`, which were pressed against the plan's own
/// look, pressed again against `wire`: the same way across the floor.
fn rekey(me: &Player, plan: Input, wire: u16) -> u16 {
    const WALK: u16 = Input::W | Input::A | Input::S | Input::D;
    let walk = plan.bits & WALK;
    if walk == 0 || wire == plan.aim {
        return walk;
    }
    let yaw = me.carry_yaw.add(plan.aim_turns());
    let forward = V3::from_turns(yaw);
    let right = V3::from_turns(yaw.add(crate::QUARTER));
    let mut dir = V3::ZERO;
    if walk & Input::W != 0 {
        dir = dir.add(forward);
    }
    if walk & Input::S != 0 {
        dir = dir.sub(forward);
    }
    if walk & Input::D != 0 {
        dir = dir.add(right);
    }
    if walk & Input::A != 0 {
        dir = dir.sub(right);
    }
    let new = me.carry_yaw.add(Fx::from_raw(wire as i32));
    steer(new, dir)
}
