---
status: proposed
proposed: 2026-09-11
built: 2026-09-12
rebuilt: 2026-09-13
hardened: 2026-09-23
sharpened: 2026-09-25
hunts: 2026-09-25
---

# Monsters — the Ridgeback

The first monster, end to end: a body you can stand on, eight moves, a control
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
**five and a half metres at the back on long legs**. Its head, flanks, haunch
and tail are armoured. There are two soft places, both on top:

- the **ridge**, the strip of unarmoured spine it is named for, standing proud
  of the back between the shoulders and the haunch;
- the **nape**, where the plates part to let the head turn — softer still, and
  another walk forward past the shoulders to reach.

Its height is in its legs rather than in its bulk, and both halves of that are
load-bearing. A standing full hop apexes at 2.7 m on the heaviest class and
6.0 m on the floatiest, and the lowest thing on the standing animal is 5.3 m
up, so its back is out of reach from the floor for **five of the six classes**
(the Dual mage floats onto it, and that is her thing); and its **feet are the
only part of it a fighter on the floor can touch at all**, which is what makes
the ground game a route up rather than a chore.

⚠️ **A metre taller since 2026-09-23.** At four and a half metres the tail's
middle sat at 3.4 m and four classes walked up it from the floor, so the climb
was free for most of the roster and the ground game was a thing you could
decline. The metre went into the legs, and every *earned* route — a stumble, a
topple, the slam's crash — was deepened by the same metre so it lands where it
did. The same pass made the animal gallop, gave it a seventh move for the one
place behind it nothing reached, and fixed a bug that had two of its six moves
landing metres from where they were thrown. See [feel-log.md](feel-log.md).

So the loop is:

1. **Ground phase.** Get behind it, out of the cone every forward move needs,
   and **break a foot**. That is most of what you do down there, and the feet
   are the softest thing you can reach. Behind it is not safe: the tail sweeps
   the whole half-circle behind the hips, toward whichever side you are on, and
   the back kick covers the patch directly astern. Beside a hind leg you have
   one of those to read; directly behind it you have both.
2. **Mount.** The back is above a standing jump. Five ways up, four of them
   earned and one of them a class's — see §2.
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

| Move | Range | What it is | The answer |
| --- | --- | --- | --- |
| Bite | close | Neck coils, head is thrown and the body lunges | 30 frames that **follow you** — a dodge timed to the snap. Then ninety frames of recovery: the walk-up window |
| Stomp | close | Foreleg comes up past the shoulder and down | 11 frames — *positional*: do not stay in front of it once a window closes |
| Rear and slam | close | Both forelegs up, a held pose at the top, then down, five metres wide | 48 frames that follow you — run if you are at the edge of it, dodge if you are not, and get *off* it. A hundred and ten frames of recovery after |
| Tail sweep | mid | A low arc round the hips, **to the side you are on**, out to seven or eight metres | **Jump it**. Caught, you are staggered for a hundred frames — long enough for the bite or the charge |
| Back kick | close, behind | Both hind legs, straight back; the tail lifts and the rump drops | 20 frames — **sidestep**, and do not linger dead astern |
| Charge | long | A straight line at **30 m/s**, twenty metres of it | 26 frames that follow you, then a dodge through it — it is too wide to walk out of and too fast to outrun |
| Spike spray | long | The tail comes up over the back and flings spikes out along its facing, eight to twenty metres | A dodge through them, timed to their arrival. Caught, you are **rooted for two seconds** — and the charge is what it roots you for |
| Shake | aboard | No damage; pure buck | Brace, leave, or jump the whip |

## 1a · Threat modes

**Added 2026-09-25**, from a report from play: the animal could be walked
away from, walked out of, and distracted, and a fighter at range could
backpedal and shoot it for free. So it hunts now, and there is something to
fear at every range a class fights from.

**It chases.** Its gallop is seventeen metres a second against a fighter's
walk of seven, and it matches a target that is leaving: the speed you are
moving away at is added to what the distance alone asks for
(`pursuit_gain`). It used to ask for a stroll at seven metres and a walk at
eight, so a fighter backing off from just outside its reach stayed there for
as long as they liked. Its charge is thirty metres a second, faster than a
dodge, and it is launched rather than accelerated into (see below).

**The windup follows you; the hit does not.** During a forward move's startup
it keeps turning toward where you will be *when the hit arrives* — the startup
left, plus the time a travelling hit takes to cross the gap — at nine tenths
of its free turn rate. The yaw locks on the first active frame, so the hit
still commits and whiff punishment still exists. What this removes is walking
out of a tell: the bite, the slam and the charge are dodged, jumped or read,
never strolled away from. Moves aimed behind it do not track, because turning
its head toward you swings the tail away. `the_bite_is_dodged_not_walked_out_of`
and its siblings in `crates/sim/tests/hunted.rs` pin both halves.

**It meets an approach.** A target closing on it faster than a stroll raises
its appetite for every forward move that fits (`closing_appetite`). Walking
up to it is how you get bitten.

**It presses a set-up.** A target that cannot act — staggered, stunned or
rooted — raises its appetite for every forward move that fits
(`combo_appetite`), and it skips the pause it would otherwise take to turn
round after a rear move. Two set-ups, each pinned by arithmetic in
`a_set_up_lasts_long_enough_for_what_follows_it`:

- the **sweep** staggers for a hundred frames, which outlasts the rest of the
  sweep, the beat and a bite;
- the **spray** roots for a hundred and twenty, which outlasts the rest of the
  spray, the beat, a charge's windup and its run across the ground the spray
  was thrown over.

**It keeps its target.** Somebody else has to be nearer than six tenths of
the current target's distance before its attention moves (`target_switch`).
The report of it "turning round and lumbering away" was two things. One was
nearest-wins targeting; the other was the game leaving the training dummy
standing in a hunt, so a player who backed off further than the dummy stood
was abandoned for it. A hunt in the game now starts without the dummy unless
somebody has its keys, which is what the fight harness always did.

**It comes about.** After a sweep or a kick it takes forty frames to turn to
face you (`rear_pause`) — unless you are staggered. Without it a target
behind the animal could only be scored against by the two rear moves, and it
threw them one after the other for ten minutes and never turned round.

**It takes a moment to notice you.** For the first three seconds of a hunt
(`hunt_grace`) it stands its ground, turns to face you, and throws nothing; a
hit ends the moment at once. It used to open with a spray and a charge that
landed before anybody had moved the camera. The first version walked toward
you through the moment, which only moved the ambush three seconds later: a
fighter getting their bearings was standing under it when the moment ended.

**It plants for a move, launches into a charge, and pulls up short of a
wall.** Committed to a move that does not travel, it brakes at seventy metres
a second squared, so it no longer slides a body length into you through a
stomp's windup. A charge reaches its speed in a few frames at six hundred,
under twice a rider's grip, so a braced rider holds on. And within its braking
distance of the wall it brakes instead: charging into the edge used to stop it
dead in a frame, which the grip test reads as a buck.

### How much of the fight is which

The target from play, as shares of the whole fight: about **four tenths** in
which nothing lands before it answers; about **a fifth** in which anybody can
walk up and swing; and the rest open only to a poke from where you stand or
to a fast way in — a dash, a vault, a structure jump, a dash to the shadow.

`cargo run -p hunt --bin fight` measures it frame by frame from one number,
how long until the creature can start its next move (`frames_until_free`),
against what each kind of approach takes from a standoff four and a half
metres from a foot: a reaction and a swing for a poke; add a dodge's worth of
ground for a way in; add a walk's for walking up.

| Window | Asked for | Across twelve hunts |
| --- | --- | --- |
| Threatening | ~40% | 46% |
| Open to a poke | the rest | 13% |
| Open to a way in | the rest | 19% |
| Safe to walk up | ~20% | 21% |

The walk-up share is where it was asked for. Threatening is six points over,
and the lever is the beat between moves (`think_frames`): each frame of it
moves time from threatening into the bands below, and each also shortens the
fight, because a longer beat is a longer opening for the hunter as well. The
bands are pinned in `it_is_dangerous_most_of_the_time_and_open_the_rest`.

The long openings are the slow, scary moves missing: ninety frames after a
bite, a hundred and ten after a slam, seventy after a charge that has run
twenty metres past you, and the forty-frame flinch that a big enough burst
earns.

## 2 · Getting on it

Five ways up. Four are earned, and the fifth belongs to one class.

```
cargo run -p sim --bin beastcheck
```

prints all of this against the jump the simulation actually produces, because
geometry arguments conducted in prose go wrong:

```text
a full hop reaches 2.681 m off the floor (Bulwark) to 5.968 m (Dual mage)
the arena's platforms are 1.5 m, so from one the best of them reaches 7.468 m

standing, the tops of the surfaces you can stand on:
  shoulders        5.58 m   a standing jump for Dual mage
  barrel          5.569 m   a standing jump for Dual mage
  haunch          5.482 m   a standing jump for Dual mage
  tail            5.374 m   a standing jump for Dual mage
  tail, middle    5.263 m   a standing jump for Dual mage

the weak points, standing:
  ridge           5.514 m at its foot,  6.194 m at its top, x1.75 damage
  nape            5.476 m at its foot,  6.056 m at its top, x2.4 damage

and what opens a way up (lowest the surface gets):
  the shoulders, stumbling            2.665 m   a standing jump, every class
  the haunch, stumbling               3.874 m   a standing jump for Champion, Shadow Reaver, Elementalist, Blood mage, Dual mage
  the shoulders, both forefeet broken  3.807 m   a standing jump for Champion, Shadow Reaver, Elementalist, Blood mage, Dual mage
  the barrel, toppled                 2.217 m   a standing jump, every class
  the shoulders, through a slam       3.222 m   a standing jump for Champion, Shadow Reaver, Elementalist, Blood mage, Dual mage
  the tail, through a sweep           5.268 m   a standing jump for Dual mage

nose to tail: 13.406 m.  clips baked: 14
```

**It names who can get there, since 2026-09-23.** It printed "the floatier
classes" for anything between the shortest hop and the tallest, and once the
animal grew a metre that band held every surface on it while the honest answer
for most of them was one name. A route is a route for the classes that can jump
it, and the climb turns on which those are.

**What the metre did to the climb.** Standing, nothing on the animal is inside a
hop for anybody but the Dual mage, whose 5.97 m float clears the tail with
seventy centimetres to spare — the one class that can decline the ground game,
and she is the class whose whole identity is not touching the floor. For the
other five the way up is one of the four below, every one of which is either
earned on the ground or read off a move. The Bulwark's 2.68 m hop reaches the
two lowest — a stumbling shoulder and a toppled barrel — and nothing else, which
is unchanged from before the metre and is still the open Bulwark-shaped
question.

1. **A broken foot.** It goes down on a knee for nearly two seconds and the
   shoulders come to 2.67 m, inside every class's hop. This is what the ground
   game is *for*, and it is now the baseline route rather than a bonus.
2. **A topple.** The barrel at 2.2 m, and a long window to use it in.
3. **The slam's recovery.** Its shoulders are at 3.2 m at the bottom of the
   crash — which means the answer to the hardest-hitting move in the set is also
   an invitation, if you are quick.
4. **Two broken forefeet.** The shoulders at 3.8 m for the rest of the fight:
   a permanent route for everybody but the Bulwark, bought with the whole ground
   game.
5. **The tail, for the Dual mage.** Carried level at 5.3 m; a hop for her and a
   wall for everyone else. It used to droop to 3.4 m at its middle and was the
   free route for most of the roster. From one of the arena's 1.5 m platforms
   the Reaver and the Elementalist reach it too, if they can fight the animal
   over to one.

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

> **It is folded into the input once, at the top of `World::advance`, and
> nothing downstream adds it again.** A rider's look is *the mouse plus the
> ride*, and everything built from a look has to be built from that same sum:
> the eye, the ray the crosshair draws out of it, the facing, and the direction
> `W` walks. Added at two of the four and not the other two — which is what it
> was until 2026-09-16 — the drawn camera sits at the mouse's angle while the
> fighter stands at the mouse's angle plus the ride, so the character comes out
> looking off to one side of the screen and the crosshair stops meaning what it
> says. It does not heal, either: the carry has to survive coming off (zeroing
> it would whip the view round the moment you landed), and mounting is landing,
> so a knock that drops you onto the animal for a second leaves you crooked for
> the rest of the match. `the_creature_turning_carries_the_camera_too` in
> `crates/sim/tests/monster.rs` is what holds the four together, and
> `ground_that_turns_under_the_fighter_turns_the_camera_with_them` in
> `crates/view/tests/presentation.rs` is what carries it across to the drawn
> camera, which reads `carry_yaw` out of the snapshot like it reads `aloft`.

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
  last part of the tell and not the first.
- Its **whip is 36 frames**, which is shorter than a full hop by enough that a
  hop begun in the last quarter of the startup lands after it. That is what
  makes the timing a *read*: committing late in the tell clears the whole
  thing, committing early lands you in the middle of it, and committing once it
  has begun is a jump that never leaves. Measured frame by frame, a jump taken
  in frames 25–39 earns 80–130 frames back on the animal, and frames 40 onward
  earn nothing. ⚠️ **It was 56 frames until 2026-09-23**, and the document said
  any frame of the startup would do. It would not: a hop is about 55 frames and
  the whip began at 40, so every jump landed inside it and was thrown — and
  then, because a thrown rider was kicked along the surface's own velocity,
  which at a whip's reversal is nothing, went straight up, came straight back
  down onto the same back, and stayed. The read "worked" through a fall taken
  every time, at the fall's cost.
- Its **lockout is 230 frames**. Without one the answer to every buck is another
  buck, and a rider who read the shake and jumped it has earned nothing, because
  the next shake starts on the frame the last one ended.

Two more rules under the buck, both from the same day:

- **A thrown rider comes off the side**, at the full kick, whichever way the
  back was moving. The kick is taken in the creature's *heading* rather than in
  the frame of the part underfoot, because at the top of a shake that part is
  fifty degrees over and "sideways" read there is a throw that goes up. And a
  body in the stun of a throw does not land back on: the back that threw it is
  still whipping underneath, and a rolled top face scooped falling riders up a
  dozen frames after throwing them, for the fall's damage a second time.
- **A cut is not a buck.** The pose is a lookup, so on the frame the creature
  changes clip — a shake interrupted into a flinch, say — its back jumps from
  one pose to another with no motion in between, and the grip test read that as
  an acceleration. The rider who had just landed the hit that caused the flinch
  was the one thrown, every time, so every ride ended with its first good hit on
  the ridge. The rider's feet are planted again on a cut, exactly as on landing,
  and the motion that follows is what is judged.

Pinned by `you_can_jump_the_shake_if_you_commit_before_the_whip` and
`a_move_it_has_just_thrown_cannot_come_straight_back`. The whip is six
reversals now rather than nine, for the same six frames a swing it had before:
nine over thirty-six frames threw braced riders off the hips, which are meant
to be the calm place.

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
| **Stumble** | a **foot breaking**, or control landing while susceptible | 110f | the mount window: the shoulders come to 2.70 m |
| **Topple** | poise broken by damage to the ridge or the nape | 150f | the big punish, and the reward the climb is for |

A broken foot is not only the moment. The corner **stays** down for the rest of
the fight: the body pitches toward the missing end, rolls toward the missing
side, the useless limb folds rather than punching through the floor, and the
creature turns worse and moves slower on that side. Two broken forefeet bring
the shoulders to 3.59 m — inside a standing jump, permanently.

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
narrowed, there was a station **directly behind it** outside every move's cone
but the shake's — which the first version of this document called the spot the
ground game is played from, and a player called a safe spot, because it was
one: you stood at the tail root and hit a hind foot and nothing reached you.
Two things fixed that. The **back kick** is aimed at exactly that patch, its
tell is twenty frames and its answer is a sidestep, which is not the
sweep's answer. And the sweep's cone now runs to dead astern and the whip goes
to whichever side you are on — the clip is baked one way and mirrored on the
side the target was on when it committed, so the left flank is no longer the
side the tail never came to. There is still a place the ground game is played
from: **beside a hind leg**, about a hundred and ten degrees round, where the
kick does not reach and the sweep's thirty-frame tell is long enough to
see over a poke you have already thrown — and not over one you throw into it,
which is the rhythm the flank teaches. Directly behind, both threats are live
and the kick's tell is the length of a poke, which is the point. The scripted
hunter stands where a person learns to.

⚠️ **The kick's radius is the station's definition.** At 2.4 m the flank is
outside it with a body's width to spare; at 2.6 m, tried on 2026-09-25, a
hunter that lags a hundred and thirty degrees round behind a turning animal is
inside it, and the scripted hunter lost every hunt to kicks it was standing
in. Every other volume on the animal grew that day. This one did not, because
a station that a person can find is worth more than a bigger kick.

Selection is **not** the maximum. Everything scoring at least
`decisiveness × best` goes into a weighted draw, seeded from the snapshot.
`decisiveness` at one is a script you can memorise; at zero it is noise. In
between, the *distribution* is learnable while the next move is not, which is the
only definition of "hard but fair" that survives contact with a player who has
fought the thing fifty times.

### It gallops when you run, and turns before it runs

Its walk is slower than a fighter's on purpose — inside striking distance it
catches you by cornering, not by outrunning. But the same was true at any
distance until 2026-09-23, and a fight you could walk away from at leisure is
not a hunt. Wanting to close a long gap now runs it up to the **gallop**, seventeen
metres a second against a walk of seven, and the gait blends to match — and
since 2026-09-25 a target that is leaving is matched as well as closed on. See
§"Threat modes". Forward
speed is scaled by how squarely it faces you, so a creature that has been got
behind comes about on the spot rather than galloping off in the wrong direction
and swinging round in an arc — which is what the first version did, and is the
difference between an animal and a car.

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
that a rate limit alone would not have given them. A hit that is out locks the
yaw entirely, for exactly the reason the players' moves do; a forward move's
windup keeps turning at a fraction of the rate, since 2026-09-25 — see
§"Threat modes".

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
| **Landed, per move** | Beside how often each was thrown: a move's hit rate on the hunter | a telegraph that never lands is decoration; one that always does is unreadable |
| **Swings and connected** | Attacks the hunter started, and frames one of them touched the animal | whether "damage dealt" is a cautious hunter or one swinging at air |
| **Threatening / poke / way in / walk up** | The fight divided by how long until it can move again, against what each approach takes | about four tenths, the rest, the rest, a fifth — see §"Threat modes" |

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

Numbers from `cargo run -p hunt --bin fight`, against a Champion, after the
second 2026-09-25 pass — the one that made it hunt. **The scripted hunter is a
decent player, and no better**: a fixed fifteen-frame reaction, dodges and
jumps a few frames early or late at random (`SLOP_EARLY`, `SLOP_LATE`), and one
plan. That plan is the one this fight is meant to teach: wait just outside the
reach of the close moves, dodge at the hit rather than at the tell, jump the
sweep, go in only on a window it can see will still be open when it arrives,
and be gone before the recovery ends.

Across twelve hunts:

```text
  won 4 of 12, the wins in 56 to 71 s and the losses in 25 to 68 s
  hits taken per minute   7.4
  threatening / poke / way in / walk up   46 / 13 / 19 / 21 %
  unanswerable hits         0

  landed / thrown, all twelve
    Bite       20/38    Stomp   16/45    Tail sweep   2/41    Charge  6/35
    Rear+slam   8/36    Kick     0/3     Spike spray  3/25
```

**Every threat mode lands.** Before this pass, across twelve hunts, the bite
had landed 1 time in 192, the charge 0 in 230 and the slam 2 in 375. Now each
one does at a rate a decent player feels. The charge had never landed because
it never charged: it accelerated at its walking rate and crawled a metre and a
half in its whole active window.

**It is lethal, and the fights are short.** Five to ten hits is a dead
hunter, and a decent player's hunt is over, one way or the other, in about a
minute. That is at the floor of the one-to-twenty-minute band the design
document asks of coop. The bite is the move that does it: it lands half the
time, often as the follow-up to a stomp or a sweep, which is the set-up system
working. Its damage came back down to 190 from the previous pass's 240 because
a bite that tracks, lunges and follows a stagger is a far more dangerous move
per throw than the one that never landed.

**What the harness caught.** A perfect-timing bot dodged every reactable move
there is, so "landed" said nothing about whether a tell could be read; the
timing error is what made it a measure again. The dodge windows came out at
four frames for the bite and the slam — a coin toss — and are six to nine now,
because their hits are out for four frames rather than six. The charge's
width is on a knife edge between "too wide to dodge" (2.4 m, where a dodge's
ten invulnerable frames end inside it) and "narrow enough to walk out of"
(1.8 m); it is 2.0, and `the_charge_is_dodged_not_walked_out_of` holds both
sides. A dead fighter was scooped onto its back by a galloping animal and
carried as a rider; the dead do not mount any more. And the report's
"unanswerable" blamed the creature for a hunter walking into a stomp at a full
run; it measures from where the animal stood when it committed now, which is
what the rule was always meant to catch.

**What the 2026-09-23 pass changed.** The bite lands where it is thrown (it is
chosen for a target at three to eight metres and its volume used to start at
nine); the sweep covers the tail root and goes to the side you are on; the kick
exists and lands on a hunter that lingers astern. Nothing on the standing
animal is inside a hop for five of six classes, and the earned routes are where
they were. The hunter stopped jumping at a tail it cannot reach and learned
three things a person learns in the first ten minutes: poke, then watch, then
poke — a poke is twenty frames of not being able to jump and a chained one is
forty; unload into a recovery you can see is long enough, and nowhere else; and
stand beside the hind leg rather than behind it.

**What the harness caught, and what it did not.** Two of the day's four real
bugs were invisible in the old report and obvious in the new one. `landed`
beside `thrown` is what showed that the bite never touched anybody and the kick
touched a hunter standing exactly where it was aimed; `swings` beside
`connected` is what showed a hunter swinging at air from a metre past its own
reach. And the per-hit trace, which names what the hunter was in the middle of
when each hit landed, is what found the five-throws-in-a-second re-landing and
the flinch that threw the rider who caused it — both of which a person would
have reported as "the ride feels random".

### Still open

- **Is it too lethal?** A decent player's hunt is over in a minute, won or
  lost, which is the floor of the band the design asks for. The levers, in
  the order they cost the least of what was asked for: the beat between moves
  (`think_frames`), the follow-up appetite (`combo_appetite`), and the bite's
  damage.
- **The threatening share is six points over.** The same beat is the lever,
  and it trades against the length of the fight.
- **The windup's tracking is a feel number.** At nine tenths nobody walks out
  of a tell; whether it reads as an animal following you or as a homing
  missile is a person's question, and the knob is `startup_tracking`.
- **It still often opens with the spray**, now after the three-second moment
  rather than on the first frame. Whether three seconds is enough to get your
  bearings is the knob `hunt_grace`.

- **The Dual mage can decline the ground game.** Her float clears the tail from
  the floor by seventy centimetres. Whether that is her identity or a hole in
  the premise is a decision, not a tuning pass; making the animal tall enough
  to stop her would put its back seven metres up.
- **The Bulwark's routes are the two lowest and nothing else.** A stumbling
  shoulder and a toppled barrel. Unchanged by the metre, and still the sharpest
  form of the question §2 asks.
- **Is the kick's tell long enough?** Twenty frames is readable from neutral
  and not from inside a poke, which is the design — but the scripted hunter
  answers it by standing where it does not reach, and whether a person finds
  that spot or eats kicks until they stop playing is a person's question.
- **Is one win in twelve a floor or a wall?** The scripted hunter cannot learn,
  so its losing says the fight is dangerous for somebody with a quarter-second
  reaction and one plan, which is what was asked for. Whether a person who has
  played it ten times finds it dangerous or cheap is the question the pass was
  made to ask, and the answer decides whether the damage (a quarter of a
  fighter a hit) or the tells (bite 24, kick 20, sweep 32) is the next knob.
- **Is the buck's damage the right lever?** Forty-five a throw is now paid
  once per throw rather than five times, and the shake is jumpable as
  documented. A player who rides well should barely feel it, and whether that
  is true is the sort of thing only a player can say.
- **The nape may be a step too far.** It is worth 2.4× damage and it is another
  walk forward past the shoulders, on the part of the back the shake is most
  violent on. Whether anyone chooses it over the ridge is a question for a
  person.
- **The Elementalist cannot win**, as before: the scripted hunter's plan is a
  melee plan.
- **Coop.** Both players can fight it and both can be on it at once, but the
  numbers are set for one, and a monster tuned for two is a different monster.

## 10 · Deliberately not yet

- **A second monster.** The machinery is built to be shared — bones, parts,
  ride, control, clips — but a second one is what proves it, and it should be
  built when there is something to learn from it.
- **Roll on the ground.** The creature topples onto its side, which is the only
  roll in the set. A body roll it *chooses* would be a fine eighth move and
  nothing in the rig would have to change.
- **A gait that turns.** The walk and gallop cycles are straight-line. A
  quadruped leaning into a turn is a real piece of readability and the rig can
  express it; nothing drives it yet.
