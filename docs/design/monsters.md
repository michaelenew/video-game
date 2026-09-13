---
status: proposed
proposed: 2026-09-11
built: 2026-09-12
rebuilt: 2026-09-13
---

# Monsters — the Ridgeback

The first monster, end to end: a body you can stand on, six moves, a control
algorithm that decides between them, and a way to measure whether the fight is
any good.

Everything here is one creature. The plural in the filename is a promise, not a
claim — a second monster should reuse the body, ride and control machinery and
supply only its own parts and moves.

---

## 1 · What the fight is

**You cannot reach the thing that kills it from the ground, and you cannot get
off the ground without earning it.**

The Ridgeback is a quadruped about thirteen metres nose to tail that stands
**four and a half metres at the back on long legs**. Its head, flanks, haunch
and tail are armoured. There are two soft places, both on top:

- the **ridge**, the strip of unarmoured spine it is named for, standing proud
  of the back between the shoulders and the haunch;
- the **nape**, where the plates part to let the head turn — softer still, and
  another walk forward past the shoulders to reach.

Its height is in its legs rather than in its bulk, and both halves of that are
load-bearing. A standing full hop apexes at 4.14 m, so its back is out of reach;
and its **feet are the only part of it a fighter on the floor can touch at all**,
which is what makes the ground game a route up rather than a chore.

So the loop is:

1. **Ground phase.** Get behind it, out of the cone every forward move needs and
   out of the tail's, and **break a foot**. That is most of what you do down
   there, and the feet are the softest thing you can reach.
2. **Mount.** The back is above a standing jump. Five ways up, one of them free
   and four of them earned — see §2.
3. **Ride.** Walk up the back to the ridge, and forward again to the nape. Your
   controls are relative to the surface under your feet: the creature turning
   does not turn your movement.
4. **Buck.** It notices, and the moves change. **Nothing it throws can reach its
   own back** — a rider is threatened by the *buck* and by nothing else, which
   is what makes riding a phase with its own vocabulary. Brace, leave, or jump
   it.
5. **Topple.** Enough damage to either weak point breaks its poise and it falls
   over. That is the big window, and it is the reward the climb is for.

Each of its moves has a *different* answer. That is the whole test of a
monster's move set, and it is the one the fight report measures directly.

| Move | What it is | The answer |
| --- | --- | --- |
| Bite | Neck coils, head is thrown | Sidestep, or dodge through it |
| Stomp | Foreleg comes up past the shoulder and down | 13 frames — *positional*: do not stand in front |
| Tail sweep | Wide, low arc behind and to the sides | **Jump it** — the hitbox is 1.6 m, and a tapped hop is not enough |
| Charge | Commits to a straight line at 15 m/s | Step laterally late — it cannot turn |
| Rear and slam | Both forelegs up, a held pose at the top, then down | 52-frame telegraph; get out from under it, and *off* it |
| Shake | No damage; pure buck | Brace, leave, or jump the whip |

## 2 · Getting on it

Five ways up, and only one of them is available whenever you like.

```
cargo run -p sim --bin beastcheck
```

prints all of this against the jump the simulation actually produces, because
geometry arguments conducted in prose go wrong:

```text
a full hop reaches 4.14 m off the floor
the arena's platforms are 1.5 m, so from one it reaches 5.642 m

standing, the tops of the surfaces you can stand on:
  shoulders        4.581 m   only from a platform
  barrel            4.57 m   only from a platform
  haunch           4.482 m   only from a platform
  tail             3.915 m   a standing jump
  tail, middle     3.345 m   a standing jump

the weak points, standing:
  ridge            4.514 m at its foot,  5.194 m at its top, x1.75 damage
  nape             4.477 m at its foot,  5.057 m at its top, x2.4 damage

and what opens a way up (lowest the surface gets):
  the shoulders, stumbling             2.353 m   a standing jump
  the haunch, stumbling                3.566 m   a standing jump
  the shoulders, both forefeet broken  3.407 m   a standing jump
  the barrel, toppled                  2.289 m   a standing jump
  the shoulders, through a slam        2.344 m   a standing jump
  the tail, through a sweep            3.811 m   a standing jump
```

1. **The tail, any time.** 3.9 m against a 4.14 m apex — two hundred
   centimetres of margin, from beside an animal that is turning. Doable, and
   not casually. This is the baseline route and it is deliberately the tightest
   one.
2. **A platform.** The arena's two 1.5 m platforms put the whole back inside a
   hop. The cost is that you have to fight the creature over to one.
3. **A broken foot.** It goes down on a knee for nearly two seconds and the
   shoulders come to 2.35 m. This is what the ground game is *for*.
4. **A topple.** The barrel at 2.3 m, and a long window to use it in.
5. **The slam's recovery.** Its shoulders are at 2.34 m at the bottom of the
   crash — which means the answer to the hardest-hitting move in the set is also
   an invitation, if you are quick.

Once aboard, the climb is a **staircase**: tail → haunch → barrel → shoulders,
with the ridge over the barrel and the nape over the shoulders. Consecutive
mountable parts **overlap** rather than meeting, and a tread you could step onto
resolves as a floor rather than as a wall — otherwise a body standing in the
seam is nearer one part's side than its top by a hair, and least penetration
shoves them backwards every frame forever. That bug cost an afternoon; both
halves of the fix are in `monster::Rig::resolve` and `Rig::surface_within`.

## 3 · The ride

**While mounted, the player's position in the part's own frame is the
authoritative one and the world position is derived from it.** Unmounted, it is
the other way round. That is the whole rule, and it is why the creature can spin
under you without sliding you off.

Four consequences, each load-bearing:

**Movement is relative to the surface.** The creature's yaw change each tick is
added to the rider's own look angle, as a `carry_yaw` that lives in the
snapshot. Movement is camera-relative already, so carrying the look angle makes
movement body-relative for free, and the camera turns with the creature rather
than the creature turning away underneath the camera. One mechanism, both
effects. The raw aim on the wire is never rewritten — `carry_yaw` is simulation
state, recomputed from the snapshot, so rollback reproduces it.

**Mounting is landing, not a button.** Fall onto a mountable part and you are on
it, the same way falling onto a platform puts you on the platform.

**Jumping leaves — and jumping *comes back*.** A jump converts the local
position to a world one, adds the surface's own velocity, and hands you to
ordinary air movement. What the leap carries is **capped** (`leap_carry`), and
that cap is the whole of why the ride has three answers rather than two:
uncapped, a jump taken during a shake is a jump taken off a back whipping
sideways at forty metres a second, so the only thing leaving the ground buys you
is being flung further away by choice.

**Nothing it throws can reach its own back.** Every attack it has is aimed at
the floor around it, and its back is four and a half metres up. Pinned by
`nothing_it_throws_can_reach_its_own_back`.

**So the buck is what the ride costs.** Being thrown takes health -- you came
off four and a half metres of animal, helpless, and you land. Without that the
back is a room you sit in, which is what it became the first time it was made
safe from hitboxes: the scripted hunter climbed thirty-five times a fight, was
bucked thirty-four of them, and never took a scratch for it. It is also what
makes the two answers to a buck worth knowing, because bracing and jumping both
cost something and neither is worth paying unless the alternative has a price.

### The buck is acceleration, not a flag

A rider comes off when **the surface under their feet accelerates harder than
they can hold on through**.

That is the honest rule, and it is worth the small cost of computing it, because
the alternative — tagging each move "this one throws riders" — authors the same
fact twice and lets the tag drift from the animation. With acceleration as the
rule, a move that whips the tail throws whoever is out on the tail, and the same
move does nothing to someone standing on the shoulder, without anyone writing
that down.

Concretely: the world position of a rider's mount point is known this frame and
was known last frame, so its velocity is a subtraction and its acceleration is
one more. The acceleration is taken **in the frame of the part underfoot**,
where that surface's normal is `+y`, and the throwing component is everything
except acceleration pressing the rider *into* the surface:

```text
throw = |(a_x, a_z, max(a_y, 0))|
```

The part's frame rather than the body's, because the creature is a skeleton now:
a tail that whips throws whoever is on the tail and does nothing to somebody on
the shoulder, and reading both in one shared frame is how that distinction gets
lost.

**Crouch is brace.** Crouching multiplies grip — the answer to a shake that is
neither "dodge" nor "leave", and a real decision because bracing costs you the
attack you were about to throw. The slam exceeds even a braced grip. It is not
meant to be survived on the creature's back; it is meant to be *left*.

Where you stand matters, and it is a gradient rather than a flag:

| Standing on | Shake | Sweep | Slam |
| --- | --- | --- | --- |
| Barrel (under the ridge) | throws loose, brace holds | nothing | throws anyway |
| Shoulders (under the nape) | throws loose, brace holds | nothing | throws anyway |
| Haunch | calm | nothing | throws anyway |
| Tail base | calm | nothing | throws anyway |
| Tail, out along it | throws loose | **throws loose** | — |

The hips and the root of the tail are the calm places, and both are a long walk
from anything worth hitting. That is the trade, and it is why *where* you stand
on the animal is a decision.

### Baiting the shake, and jumping it

**The strategy the ride phase is built around.** Be up there, which makes the
shake the thing it most wants to do. Read its startup. Leave the ground over the
whole whip. Land back on. Spend the window before it can shake again.

Three numbers make it work, and all three are the design rather than tuning
noise:

- The shake's **startup is 40 frames** and is emphatically not violent — it
  plants its feet, hunches, and leans. A rider has to be able to tell the
  telegraph from the move, because the whole strategy is to leave over the
  second half and not the first.
- Its **whip is 56 frames**, which is longer than a full hop. That is what makes
  the timing a *read*: committing during the startup clears the whole thing, and
  committing once it has begun is a jump that never leaves. The transition is
  sharp — measured frame by frame, jumping anywhere in frames 0–37 earns 80–108
  frames back on the animal, and frames 38 onward earn nothing.
- Its **lockout is 230 frames**. Without one the answer to every buck is another
  buck, and a rider who read the shake and jumped it has earned nothing, because
  the next shake starts on the frame the last one ended.

Pinned by `you_can_jump_the_shake_if_you_commit_before_the_whip` and
`a_move_it_has_just_thrown_cannot_come_straight_back`.

## 4 · Crowd control, and what a threshold is for

A monster that took full crowd control would be a monster you lock down. A
monster that took none would make half of every kit dead weight in a hunt, and
the two halves of the game would stop teaching each other anything.

So: **a piece of control is offered to the creature and it answers with what it
is willing to take**, and what it is willing to take depends on how hard it has
been hit *recently*.

`strain` is damage taken lately. Every hit adds to it and a few per cent of it
bleeds away every frame, so a burst fills it and a trickle does not — which is
the right shape, because what should move something this size is a concentrated
answer rather than chip damage.

| Above | What happens |
| --- | --- |
| `cc_strain` | It is **susceptible**: a knock-up trips it, a grab roots it, a slow lands at a fraction of its strength |
| `interrupt_strain` | A hit **breaks it out of what it is doing** — startup, recovery, *and a live hitbox* |

Both are spent when used, so control has to be bought again rather than held.

**Both thresholds fall as it is worn down** (`strain_desperation`), and that one
number is the arc of a hunt. At full health nothing short of a real burst moves
it and the fight is methodical: bait, punish, chip a foot. By the end an
ordinary combo is enough, the creature is committing harder because it is hurt,
and the fight goes frantic. The same buttons mean something in both halves of
the game without meaning the *same* thing, which is the point.

The interrupt is the one thing besides a topple that may cancel an active frame.
An ordinary flinch may not: a creature whose hit can be cancelled by being hit
is a creature you never have to trade with, and trading is most of the ground
game.

## 5 · The ladder of ways to be in trouble

| State | Cause | Length | What it gives |
| --- | --- | --- | --- |
| **Flinch** | one hit past `flinch_threshold`, or a burst past `interrupt_strain` | 18f | a small punish; the interrupt version cancels a live hitbox |
| **Stumble** | a **foot breaking**, or control landing while susceptible | 110f | the mount window: the shoulders come to 2.35 m |
| **Topple** | poise broken by damage to the ridge or the nape | 150f | the big punish, and the reward the climb is for |

A broken foot is not only the moment. The corner **stays** down for the rest of
the fight: the body pitches toward the missing end, rolls toward the missing
side, the useless limb folds rather than punching through the floor, and the
creature turns worse and moves slower on that side. Two broken forefeet bring
the shoulders to 3.4 m — inside a standing jump, permanently.

## 6 · The body is a skeleton

It used to be ten boxes welded to one another, carried by a body that could yaw
and pitch and nothing else; five scalars articulated it. That proved the ride
worked and it was never going to look like an animal — the legs did not move,
the neck did not bend, and a tail sweep was the whole creature rotating because
the tail could not swing by itself.

It has a skeleton now, on the same terms the fighters' one is built on: **bones
hang off their parents at fixed offsets, and a pose is joint angles only.**
Eighteen bones, and every one of its eighteen part boxes is authored in the
frame of the bone that carries it.

```text
                                       Neck ── Neck2 ── Head
                                      /
  Tail4 ─ Tail3 ─ Tail2 ─ Tail1 ── Root ── Spine ── Chest
                                    |                 |
                           Thigh ───┴─── Thigh   Shoulder ─┴─ Shoulder
                             |            |          |          |
                           Shin         Shin      Forearm    Forearm
```

**Why it is in the simulation rather than the renderer.** A fighter's skeleton
lives in `crates/view` because nothing about a fighter's hitboxes depends on it.
The creature is the other way round: its parts **are** its geometry — what you
collide with, what you can stand on, what your attacks land on — so the boxes
have to be where the simulation can see them, and therefore so does the skeleton
that moves them. It is also what keeps the overlay honest: the renderer draws
the boxes the simulation places, and does not rebuild them.

**It is still a pure function of the snapshot.** Nothing about the pose is
stored. It is sampled out of a baked table by phase, layered with a gait indexed
by ground covered and whatever the broken legs are doing, and every input comes
from the snapshot — so a rollback reproduces it bit for bit. The gait's phase
accumulator *is* in the snapshot, and has to be, because the legs are collision
geometry: where a foot is decides what a player walks into.

The rotations are fixed-point 3×3 matrices (`math::Mat3`). Composition down a
six-bone chain accumulates a little error, and it accumulates *identically* on
every machine, which is the only property determinism asks for.

### The animation

Authored through the same factory the fighters go through — sparse keys, an ease
per gap, a spring per channel — in `crates/anim/src/beast/`, and baked to
`crates/sim/src/beast_baked.rs` by `cargo run -p anim --bin bake_beast`. The
house style is Monster Hunter's, and it is three rules:

1. **The windup is the move.** Most of the frames are before the hit and the
   silhouette changes early and hugely. The slam rises for half its startup,
   reaches the top early, and then *holds* there — a creature standing on its
   hind legs and not moving is the loudest thing in the fight.
2. **The hit is short and enormous**, travelling further than looks survivable,
   with the whole body behind it.
3. **The recovery is the fight.** It is long, it is where you are allowed to be,
   and what the animal is doing during it is the answer to where to stand.

Two things about the bake are gameplay rather than presentation, and both were
learned the hard way:

- **The table is in phase, not in frames.** An attack bakes as three runs of
  thirty-two samples — startup, active, recovery — each read on its own. Retune
  a move in the Oven and its animation stretches with it, instead of leaving the
  contact pose on the wrong frame.
- **The sample count is a gameplay number.** Samples are read back with a
  straight lerp, so each join is a corner in position and therefore a *spike in
  acceleration* — which is exactly what the grip test measures. At twelve
  samples a 34-frame shake put three frames between corners and threw braced
  riders on single frames that had nothing to do with how hard the animal was
  moving. Thirty-two is about one sample per frame of the longest phase.

The same lesson in a different place: **two keys at the same instant with
different poses are a step, not a motion.** The shake had its last startup key
and its first whip key both at the phase boundary, and the arbitrarily large
acceleration that produced threw braced riders off the hips before the shake had
started.

## 7 · The control algorithm

Three properties decide whether a monster is hard in a way anyone enjoys:

1. It must **commit**, so that whiff punishment exists.
2. It must **telegraph**, so that the decision the player is being asked for can
   actually be made.
3. It must not **read your inputs**, so that a good read stays good.

### It glances, rather than watching

The creature does not see the player continuously. Every `glance` frames it
takes one sample — **position and velocity** — and acts on that sample until the
next one. In between, it is working from stale information.

Aiming at where you were is how a predator misses, so the sample is
extrapolated: it aims at `seen_pos + seen_vel · lead`. Two knobs, and between
them they are the whole difficulty model:

- **`glance`** — how fresh its information is.
- **`lead`** — how well it extrapolates from it.

A short glance with a long lead is frightening. A long glance with no lead is an
animal you can walk around. And the skill the pair rewards is specific and
learnable: **change direction between its glances**. That is a thing a player
can notice, get better at, and explain to someone else, which is the test of
whether a difficulty knob is a difficulty knob or just a number.

It is also why the creature never reads a button. It cannot: it is working from
a position and a velocity that are already several frames old.

### It scores its moves and then does not always pick the best one

At a decision point — free, and the think timer expired — every move is scored:

```text
U(m) =  range(m)        how well the current distance suits the move
      + arc(m)          whether the target is in the move's cone
      + rider(m)        enormous for the bucking moves when someone is aboard
      + variety(m)      a decaying penalty on the move it just used
      + hurt(m)         wounded animals commit harder
      ⟂ cooldown(m)     a move on its lockout scores nothing at all
```

`range` is a tent function peaking at the move's ideal distance and reaching
zero at the edges of its window. `arc` is the dot product against the move's own
cone, which is what makes standing at the creature's flank *mean* something.

The cones matter more than they look. The tail sweep's was once wide enough to
cover almost the whole circle, and it became 70% of everything the creature did;
narrowed, there is now a station **directly behind it** that is outside every
move's cone but the shake's — which is the spot the ground game is played from,
and is a thing a player can find.

Selection is **not** the maximum. Everything scoring at least
`decisiveness × best` goes into a weighted draw, seeded from the snapshot.
`decisiveness` at one is a script you can memorise; at zero it is noise. In
between, the *distribution* is learnable while the next move is not, which is the
only definition of "hard but fair" that survives contact with a player who has
fought the thing fifty times.

### Turning is the fight

A creature this size cannot snap around, and that limit is where the player's
advantage lives. Yaw is driven by a proportional controller behind two limits:

```text
error          = wrap(desired_yaw − yaw)          into ±half a turn
desired_rate   = clamp(error · turn_gain, ±turn_rate_max)
yaw_rate      += clamp(desired_rate − yaw_rate, ±turn_accel · dt)
yaw           += yaw_rate · dt
```

The rate limit is how fast it can turn. The **acceleration** limit is what gives
it mass: it cannot reverse a turn instantly, so it overshoots slightly when it
has been swinging hard, and a player who cuts back across its nose gets a window
that a rate limit alone would not have given them. Committed moves lock the yaw
entirely, for exactly the reason the players' moves do.

Each broken foot multiplies `turn_rate_max` toward that side, applied once per
break — so an animal with both legs gone on one side barely comes round at all.
The player can see the consequence of a thing they did, which is the only reason
to have breakable legs at all.

### Poise, and the topple

Damage to either weak point fills a poise pool. Full, the creature **topples**: a
long, threat-free recovery. Poise regenerates while it is standing, so the window
has to be earned again rather than accumulated over a fight.

The topple is what the climb is *for*. Without it, riding is a damage-per-second
choice and the fight is arithmetic; with it, riding is a wager on reaching a weak
point before the next buck, and that is a decision.

## 8 · Judging whether the fight is any good

"Feels good" is not a test. What can be tested is whether the fight has the
properties a good fight needs, and `cargo run -p hunt --bin fight` plays a
scripted hunter against the creature and reports them.

| Measure | Why it says something | Where it should land |
| --- | --- | --- |
| **Reactable share** | Fraction of attacks with startup at or above human reaction — about 15 frames | a majority, not all |
| **Openings per minute** and their **mean length** | An opening shorter than the player's fastest meaningful attack is not an opening | openings long enough to punish |
| **Idle share** | Frames the creature is doing nothing at all | low, but not zero |
| **Move entropy** and **longest repeat** | A one-note monster scores near zero however hard it hits | high enough that no move dominates |
| **Move coverage** | Moves it never once used are design fiction | every move used |
| **Ride share** and **mean ride length** | Long enough to reach a weak point; short enough that the back is not a safe room | bounded, and not trivial |
| **Damage into feet**, **worst foot** | Whether the ground game is being played at all | a foot should go, in a fight it wins |
| **Time to kill** | The design document asks for one to twenty minutes coop | inside the band |
| **Unanswerable hits** | Hits from an attack below reaction *and* with no positional warning | as close to none as the design allows |

The last one is the important one and the easiest to get wrong. A monster can
score well on every other line and still feel cheap, and it will be because of
damage the player had no way to avoid.

`damage into feet` was added during this rebuild for a specific reason: `legs
broken: 0` is two completely different findings — the hunter never swung at a
leg, or it swung at them constantly and they are too tough — and nothing else in
the report told them apart. It turned out to be the first, and it stayed the
first through three changes to the hunter's station before the real cause showed
up: **a level shot from a fighter on the floor goes under the belly.** The
creature is on stilts, and the bot was standing where nothing could be hit.

## 9 · Where it landed

Numbers from `cargo run -p hunt --bin fight`, against a Champion. **The scripted
hunter is a mediocre player** — a fixed fifteen-frame reaction delay, one plan,
no adaptation — so these are the numbers for someone who has just learned the
fight, not for someone who is good at it.

```text
  killed at frame 8003  --  133.4 s

  reactable moves            4/5   answerable on sight, not from memory
  moves per minute          33.8   the rhythm
  openings per minute       33.4   how often you get a turn
  mean opening               46f   long enough to punish?
  idle share                 12%   doing nothing at all
  move coverage              6/6   moves it ever used
  favourite move share       30%   one-note?
  move entropy              0.93   1.00 is an even mix

  rides                       33   times anyone got on
  thrown off                  28   ended by a buck, not a jump
  ridge hits                  36   damage on a weak point
  topples                      3   poise broken
  legs broken                  1
  damage into feet           596   the ground game

  unanswerable hits            0   too fast to read, from outside its range
```

Across six seeds the hunter wins three, at a mean of seventy seconds. That is
where the first version sat and where a first monster wants to sit: a bot this
crude losing every time would mean nobody could learn against it, and winning
every time would mean it is not a monster.

Ride share fell from around 70% to under a fifth, which is the change this
rebuild was most meant to produce: the back used to be where the fight happened,
and it is now the reward for a phase that happens on the floor.

### Still open

- **The Elementalist cannot win.** Zero of three, with four hundred rides in a
  twelve-minute hunt. The scripted hunter's plan is a melee plan and a ranged
  class plays a different fight; whether that is the class, the creature, or the
  bot is not a question the harness can answer as written.
- **Is the buck's damage the right lever?** Forty-five a throw is what took the
  bot from winning six of six to winning three, which is the right *shape* --
  but it is a blunt number and the bot gets thrown far more often than a person
  would. A player who rides well should barely feel it, and whether that is true
  is the sort of thing only a player can say.
- **Is the tail hop too tight?** Two hundred centimetres of margin on a full hop,
  beside an animal that is turning. It is meant to be the hardest of the five
  routes; it may be the *only* one anyone finds.
- **The nape may be a step too far.** It is worth 2.4× damage and it is another
  walk forward past the shoulders, on the part of the back the shake is most
  violent on. Whether anyone chooses it over the ridge is a question for a
  person.
- **Coop.** Both players can fight it and both can be on it at once, but the
  numbers are set for one, and a monster tuned for two is a different monster.

## 10 · Deliberately not yet

- **A second monster.** The machinery is built to be shared — bones, parts,
  ride, control, clips — but a second one is what proves it, and it should be
  built when there is something to learn from it.
- **Roll on the ground.** The creature topples onto its side, which is the only
  roll in the set. A body roll it *chooses* would be a fine seventh move and
  nothing in the rig would have to change.
- **A gait that turns.** The walk and gallop cycles are straight-line. A
  quadruped leaning into a turn is a real piece of readability and the rig can
  express it; nothing drives it yet.
