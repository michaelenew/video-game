---
status: built
proposed: 2026-09-30
built: 2026-10-01
tier: 5
---

# Mantis — the Duellist

A three-metre mantis with two scythe arms that fights the way a player does. It
guards its front, it parries what it sees coming, it waits for you to commit and
then it goes, and it answers the move you have shown it three times. It is the
cast's one lesson in **versus fundamentals**: spacing, whiff punishing, mixing
up, and breaking a guard. Nothing else in the cast asks for any of them.

Part of the cast planned in [../bestiary.md](../bestiary.md). The body, the
scoring, the strain thresholds, the telegraph markers and the fight report are
the Ridgeback's, from [../monsters.md](../monsters.md). The guard, the parry and
the guard breaker are the fighters' own, from [../defense.md](../defense.md),
given to a creature for the first time. Its eyes are the sparring bot's
([../sparring.md](../sparring.md)): it sees through a delay line, never the
present.

---

## 1 · What the fight is

**Its front is a wall and its eyes are late: beat it by hitting what it cannot
see coming, from where its blades are not.**

It stands **3.0 m** at the head, upright on four walking legs, with the thorax
from 0.7 m to 2.4 m and the abdomen carried level behind it at 0.6–1.4 m. The
scythes hinge at the shoulder, 2.2 m up, and reach **4.0 m** from its centre —
twice the Champion's sword and a little more than the spear's 3.4 m, so in any
straight exchange of pokes it is the one outside your reach. It is the first
creature a fighter's level swing hits squarely: the chest-height swing (1.25 m)
meets the thorax, and a skillshot aimed at the floor under it is raised to 0.9 m
and meets the thorax too. Nothing about aiming changes (§10).

It walks at **6 m/s** (slower than you), **3.5 m/s with its guard up**, skitters
at **12 m/s** to close a gap, and its lunge crosses **9 m in 8 frames** — about
67 m/s, the fastest thing in the cast and more than twice the Ridgeback's
charge. Its Leap goes **6 m straight up**, level with the Dual mage's apex, so
there is no hop in the roster that is above it.

Health **4,000** solo (first guess; the seed's 3,500 was raised for the fight
length in §9). A hit from it is 90–220, so five to eight mistakes are a dead
hunter.

The loop:

1. **Neutral.** It holds you at the edge of its reach with its guard up, and
   anything you throw into its front is blocked — or, if it was slow enough for
   its late eyes to see, **parried and countered**. Your job here is spacing:
   stay outside four metres, and do not give it anything to see.
2. **The coil.** From five to nine metres it crouches into the coil and waits
   for you to commit — a swing, a dodge, a jump, a cast. It sees late, but its
   lunge is faster than your recovery. **Commit to nothing**; walk, and change
   direction when the coil runs long.
3. **The whiff.** Everything it throws has a recovery, and the recoveries are the
   fight: 50 frames after a lunge that missed, 55 after a Leap, 30 after the
   scythes. Punish those, not its guard.
4. **The break.** Its guard has a minimum hold. An **unblockable** hit landing
   inside its guard arc while the guard is up breaks it: blades flung wide,
   **120 frames** of stagger. That is the window, and the approach is baiting
   the guard up and breaking it before it can drop.
5. **The blade.** Hits on a scythe arm wear it down. A broken blade stays broken,
   and its guard covers **one side** for the rest of the fight. Go round the
   broken side.
6. **The mixup.** It remembers the last four moves that struck its guard. Show it
   the same one three times and it takes the **Ready** stance against that move:
   throw it again and you are parried; throw anything else and it is hit clean.

## 2 · The moves

Frame numbers are first guesses. "Tell" is the frames from the first visible
change to the first active frame.

| Move | Range | What it is | Tell (frames) | The answer |
| --- | --- | --- | --- | --- |
| **Scythe pair** | close, 1.5–4 m | Right blade across, a step in, then the left — after **12** frames, or held cocked for **26** | 18, then the cocked arm | **After the first, walk straight out, and press nothing in the gap.** The fast second catches whoever stayed; the slow one catches whoever pressed |
| **Lunge** (from the coil) | mid–long, 5–9 m | Crouched, blades back, it holds for 20–60 frames; it releases when it *sees* you commit, or when the coil runs out | 20–60 (the coil) | **Commit to nothing while it is coiled.** Walk, and change direction as the chitter peaks. A dodge is a commitment |
| **Guard** | close–mid, its front | Both blades crossed; a 160° arc, the first 8 frames a parry, held at least 24 | a stance, 6 to raise | **Go round it.** It turns at 120°/s while guarding; at three metres a walk circles faster than that |
| **Counter** | close, along the parry's line | A straight thrust out of a parry, and only out of a parry | 12 | **Do not give it the parry.** At its front, throw only what is faster than its eyes — that is blocked, not parried |
| **Leap** | anyone aloft within 10 m, or mid 5–10 m | Drops, springs 6 m up cutting on the way, hangs, and dives where it last saw you | 20 (the drop) + 18 rising | **Stay on the floor while it is near.** If it goes up anyway, leave the circle its dive draws |
| **Wing flare** | close, all round, 0–3 m | Wings snap open, a shove of 5 m, and it hops 4 m back — the spacing reset into the coil | 16 | **Walk after it at once.** Its landing is 24 frames and the coil wants you far; do not dash, that is a commitment |
| **Pivot cut** | behind, 1–3.5 m | Plants its hind legs and whirls one blade round at 1.3–2.0 m, 200° from dead astern to its flank | 16 | **Crouch.** The blade passes over a crouching body |
| **Prayer** | any, prefers 9 m+ | Arms folded, head bowed, a hum for 90 frames; a parry-ready guard **all round** the whole time | 90 | **Break it or leave it.** Only an unblockable lands on it; anything else is parried from any side |
| **Ready** | close, its front | The habit stance (§5): one blade cocked at the side the remembered move arrives from | 8 to take | **Throw something else**, or go round, or wait out its 30 frames |

| Move | Startup / active / recovery | Damage | Notes |
| --- | --- | --- | --- |
| Scythe pair | 18 / 4 / gap 12 or 26 / 4 / 30 | 90 + 120 | first steps 0.8 m; second does not step |
| Lunge | coil 20–60, travel 8, active 8, recovery 50 | 170, knockdown 40 f | a line; stops at a solid, then staggers 40 f |
| Guard | 0 to raise; parry 8; min hold 24; lowering 16 | — | blocked hits push it 0.6 m, blockstun 8 f |
| Counter | 12 / 3 / 36 | 220 | only from a parry; 4.5 m along the parried line |
| Leap | 20 drop / 18 rise (active) / 6 hang / 10 dive (active) / 55 landing | 130 rising, 110 dive + knockdown | dive circle 3 m |
| Wing flare | 16 / 3 / 24 landing | 30, shove 5 m | braced (crouched) fighters slide 1 m |
| Pivot cut | 16 / 4 / 28 | 120 | does not hit a crouching body |
| Prayer | 90, then haste on the next three moves | — | haste: startups ×0.8, never under 15; recoveries ×0.5; no beat between |
| Ready | 8 to take / 30 held / 16 lowering | — | parries the remembered move only; anything else is a clean hit ×1.25 |

**Why the second slash has no fill.** Every other marker in the cast fills to its
edge on the frame the hit comes out. The second slash's footprint is drawn when
the first slash ends, and it is the true footprint — *where* is honest — but it
has no fill, because *when* is the mixup. This is the one exception to contract
item 4 in the cast, and it is bounded: the first slash is a full 18-frame tell,
and the answer to the pair is positional (be outside four metres), so a hunter
who answers the first never has to guess the second. The report counts second-
slash hits separately and asks whether each one landed on somebody who pressed
in the gap or stayed inside four metres (§9). If a person finds it unreadable,
the fix is two cocked-arm poses rather than a fill.

**The parry is the fighters' rule, given to a creature.** "The first frames of
raising the guard parry" ([../defense.md](../defense.md)), with a wider window
(8 against a fighter's 4) because the creature is not guessing the way a player
is: it raises its guard on what it saw. And what it saw is at least fifteen
frames old. So it can only parry on sight a move whose **startup is longer than
its eyes are late**. Its eyes wander between 15 and 21 frames (§5):

| Your move | Startup | Into its guard |
| --- | --- | --- |
| Champion sword, spear hit 1 | 6, 10 | always blocked, never parried on sight |
| Bulwark Bash | 4 | blocked |
| Champion hammer | 20 | parried when its eyes are 15–19 late, blocked when later |
| Blood mage Reaping sweep, Dual mage autos | per kit | the same rule, the same arithmetic |

That table is the whole lesson of its front, and the fight report prints it per
class. **The counter is a consequence, not a reaction:** a parry is a thing it
did, as a Bulwark's parry staggers the attacker without the Bulwark reacting.
Guard, parry and counter are one committed move with branches decided by what
touches it. The counter's twelve frames are below reaction on purpose; its
answer is the choice before it, and every counter is traceable to one of three
things the hunter could see (§9).

**The coil sees late, and still wins.** It releases on the first commitment it
sees, D frames after you made it. The lunge then takes eight frames, so it
arrives D + 8 = 23–29 frames after you pressed. A dodge is over at 22 and its
invulnerability at 10; a sword swing is over at 19 and has stepped you 0.55 m.
The lunge is aimed at where it saw you plus what it saw your velocity to be,
carried forward over D + 8, so a dodge that was seen at its first frame is
extrapolated along its own line — and a lunge is a line, so the lane passes
through where you went. Walking is not a commitment. A walk is extrapolated too,
which is why the answer is to **change direction**, which it only sees D frames
later: the Ridgeback's "change direction between its glances", made into a
duel.

## 3 · A threat at every range

| Range | What it does | What it asks of you |
| --- | --- | --- |
| **Hugging, under 1.5 m** | Wing flare all round, the first slash's step | Do not live there: you are inside everything and outside nothing |
| **Close, 1.5–4 m** | Guard, scythe pair, counter, Ready; the pivot cut behind | Spacing: hit only recoveries; go round the guard |
| **Mid, 5–9 m** | The coil and lunge; the Leap | Patience: commit to nothing while it is coiled |
| **Long, 9 m+** | It skitters in at 12 m/s — or it **prays**, and comes back faster | Range is not free: standing off gives it the haste |
| **Aloft, anywhere within 10 m** | The Leap's rising cut reaches 7.5 m | Stay on the floor |
| **Aboard** | Nothing: it has no mountable part | — |

**The tempting safe spots, and what covers them.**

- **Behind it.** The guard is frontal, so the back is where the damage is — and
  where the pivot cut and the flare live. A hunter who has stood outside its
  guard arc within four metres for more than about a second raises both (the
  `back(m)` term, §5). The flank, not the back, is where the ground game is
  played from: beside it, a step outside the arc, the pivot cut's crouch answer
  already practised.
- **Above it.** The guard does not cover anything more than 40° above its chest
  (§4), so the top is open — and the Leap is there. This is the one creature
  where jumping is wrong, and the reason is the one thing its guard cannot stop.
- **Far away.** A hunter at twelve metres throwing bolts into its guard is doing
  nothing, and the prayer punishes the nothing: the three hasted moves arrive
  when you come back in.
- **In front, poking.** A sword into its guard is blocked, free and safe — once.
  The third time it is Ready for the sword. See §5.

## 4 · The approach and the window

**The guard, precisely.** A cone from its chest at 2.0 m: 160° across the front,
and up to **40°** above the horizontal. The test is the one `guard_against`
already runs for fighters — the point the hit comes from, against the facing —
with one more comparison for the elevation. So a Reaver copy thrown from a
shadow behind it lands; a light Lance's burst detonating behind it lands; a hit
from high above lands; a pillar coming up under its feet comes from below the
cone and lands. Blocked hits do nothing but push it 0.6 m and hold it 8 frames:
**no chip, no guard meter, and no blade damage from a block**, because the
kernel rules out chip and a blade that broke from blocked hits is a guard meter
under another name — and then the answer to the Duellist would be pressing
buttons into it, which is the opposite of the fight.

**The window: the guard break.** An **unblockable** move — the existing
`unblockable` flag, and nothing else — arriving inside the cone while the guard
or the prayer is up. Guard break: **120 frames**, blades flung wide and low, arms
at 1.0–1.5 m and taking ×1.5. A break during prayer is **180 frames**. An
unblockable that meets an unguarded Mantis is only a hit. The guard breakers in
the roster today are Grapple (Bulwark), Earthbreaker (Champion, the hammer's
third link), Updraft and Downdraft (Elementalist) and Grasp (Blood mage). The
seed said "specials break its guard"; the flag is the honest version of that,
because two classes' `Q` is not a guard breaker
([../ability-spec.md](../ability-spec.md)) and a creature that made up its own
list would be the second description of a rule.

**How the break is earned.** Its guard, once raised, is committed for 24 frames.
After that it may drop the guard (16 frames of lowering) if it *sees* an
unblockable coming — it knows the flag the way a player knows which moves break
guard. So the guard breaker has to land **inside the minimum hold**, or be faster
than its eyes: Grasp at 16 frames usually is; Earthbreaker at 26 never is, and
has to be thrown as the third link of a chain whose first two links raised the
guard. That is the whole approach: **show it something to guard, and break the
guard it raised.**

**What you break for good: a blade.** Each scythe has **700** health, taken only
by hits on the arm itself — in the recovery of its own swings, where the blade
lies extended on the floor for 20 frames, and in the guard-break stagger at ×1.5.
So every whiff punish is a choice: the body (health) or the arm (the guard). A
broken blade is a 90-frame stagger, and then:

- **its guard covers one side**: 80°, from 10° across the broken side to 70° on
  the whole one. The broken side is open for the rest of the fight;
- the scythe pair becomes one slash and the stump; the counter and the pivot cut
  come off the whole arm only;
- it guards by turning its good side to you, at the same 120°/s, so circling
  toward the broken side beats it at any range under four metres.

Both blades broken and it cannot guard, parry or pray; it goes all in (below).

**The arc of the fight.** Strain works as for the Ridgeback, at about half the
Ridgeback's thresholds (it is human-sized; contract item 8), and control is
blocked like any blockable hit. A hit big enough past `interrupt_strain` stuffs
a startup — the fundamental "hit its windup with something heavy" — which is
why the hammer and Grasp matter even when they are not breaking a guard.

| Stage | When | What changes |
| --- | --- | --- |
| Fresh | above 60% | Patient: guards most of neutral, prays when you stand off, coils a lot |
| Hurt | 60–30% | `hurt(m)`: attacks more, guards less; Ready taken sooner (two copies, not three) |
| Desperate | under 30%, or both blades gone | No guard, no Ready, no prayer; the beat halves; the pair is always the delayed variant; strain thresholds at their floor. Loud, fast, and open |

## 5 · The brain

The Ridgeback's control algorithm — scoring, `decisiveness`, variety, cooldowns,
the windup that follows you and the hit that does not — with three changes.

**It sees through a delay line, not a glance.** The Ridgeback samples position
and velocity every `glance` frames. The Mantis needs to see *moves*, so it reads
the hunter as they were D frames ago, where D wanders between `sight_min` 15 and
`sight_max` 21, a frame at a time every `sight_wander` frames — the sparring
bot's hard level, exactly, because that is the fastest that is still fair.

```text
seen_pos, seen_vel   from a ring of samples, one every 4 frames, read at the
                     sample at or older than D (never fresher)
seen_action          the hunter's action if it began at least D frames ago,
                     else the last action it saw
commitment           any seen action but stand, walk, crouch, crouch-walk, guard
```

The action half costs nothing to store: a fighter's action and how long it has
run are in the snapshot, so "it began at least D frames ago" is a subtraction. A
move that starts and ends inside D frames is never seen at all, which is correct.

**It feels at once and sees late.** A blow meeting its guard is known the frame
it lands — proprioception, as the sparring bot's Champion feels its own hit
connect. That is what drives the parry, the counter and the habit memory, and
none of the three is a reaction to you: the parry and counter are branches of a
move already committed, and the memory is only written from what struck it.

**It may guard inside the beat.** Every other move waits for the think timer.
Guard may start on the frame a blockable commitment is seen, because a duellist
that saw your hammer coming and waited out its beat would be the Ridgeback with
blades. Nothing else skips the beat except prayer's haste.

```text
U(m) =  range(m) + arc(m) + variety(m) + hurt(m)      as the Ridgeback
      + seen(m)     a seen commitment: guard if it is blockable and reaches;
                    the coil's release; the Leap if the hunter is aloft
      + punish(m)   a seen recovery longer than m's startup plus travel
      + back(m)     pivot and flare: frames a hunter has stood outside the arc
                    within 4 m, over `back_patience`
      + far(m)      prayer: a hunter past 9 m, or no commitment seen for 90 f
      + ready(m)    Ready only: `habit_weight` × (copies of one move in memory
                    − 1), zero under `habit_anticipate` copies, and zero unless
                    that move's thrower is inside its reach + 1 m
      ⟂ cooldown(m) prayer_lockout 900 f; Ready 120 f; flare 180 f
```

`decisiveness` is **0.8** — higher than the Ridgeback's. A duellist commits to
the right answer more often than an animal; the draw is still there so the next
move is not a script.

### The habit memory — the most contentious rule in the cast

**What it is.** A ring of the last `habit_depth` = 4 moves that struck its guard
(blocked, parried, or broke it), by move identity — class and move, which is
what it saw. Shared across hunters: it recognises the swing, not the swinger.
An entry becomes usable `habit_settle` = 30 frames after it was written. When one
move holds `habit_anticipate` = 3 of the 4 (2 when hurt), `ready(m)` scores, and
at a decision point it may take **Ready** against that move: one blade cocked on
the side that move arrives from, held 30 frames. Throw that move into it and it
is parried and countered. Throw anything else, hit it from outside the arc, or
wait out the 30 frames and it lowers for 16 — and **the entry is forgotten**. A
guess it gets wrong costs it the guess.

That is what "counters a repeated one sooner" means, precisely: it does not see
the move sooner — its eyes never get faster than 15 frames. It *has already
committed* to the answer when you throw it.

**Against contract item 5**, point by point:

1. *It does not read your buttons.* Inputs never touch the memory. It is written
   by the hit resolution, from a hitbox meeting a guard — a simulation event both
   peers compute, which a swing that whiffs never produces.
   `nothing_it_remembers_came_from_an_input` holds that.
2. *Anything that looks like a reaction is to something seen at least fifteen
   frames old.* Ready is not a reaction; it is a prediction chosen at a decision
   point from entries at least thirty frames old. Prediction from what it saw is
   what the Ridgeback's `lead` already is — an extrapolation of old position and
   velocity. Habit is the same extrapolation applied to your choices instead of
   your motion. `habit_never_makes_its_eyes_faster` holds that D is untouched.
3. *It commits, and a good read stays good.* Ready is a stance with an
   eight-frame tell, a fixed length, and one fixed effect, and it is wrong-footed
   by anything else. The player always knows what is in its memory, because they
   put it there — and §6 draws it. So the player can predict its prediction,
   which is the purpose of the rule. And the kernel already makes the same
   argument from the other side: the repeat lockout exists because throwing one
   move forever is not a kit. The Mantis is the creature that enforces it.

**What could still go wrong.** A small kit is read fast: the Bulwark has one
fast poke. Shared memory means one player's habit is the other's problem in
coop. And if the marks are not seen, it will feel like the game cheating — which
is a failure of §6, not of the rule, and is the thing a person must test.

**The fallback, with no memory.** Behind the Oven flag `Mantis · habit` (on).
Off, nothing is written and `ready(m)` is replaced by `guess(m)`: at the same
decision points, at a lower fixed weight, it takes Ready against a move **drawn
from those that reach it from where the hunter stands** — a guess from geometry,
which it reads by glance and nothing else. The stance, the tell, the parry and
the wrong-guess punish are unchanged; what is lost is the lesson "do not repeat
yourself". Its front becomes a wall that one fast poke tests safely forever, and
the tier then rests on the coil, the Leap and the counter. If the fallback ships,
`think_frames` shortens by about a fifth to keep the threatening share (§9).

**Knobs** (a family per species, P1): `sight_min`, `sight_max`, `sight_wander`,
`mantis_parry_window`, `guard_min_hold`, `guard_arc`, `guard_elevation`,
`guard_turn`, `coil_min`, `coil_max`, `lunge_lead`, `back_patience`,
`habit_depth`, `habit_anticipate`, `habit_settle`, `habit_weight`, `ready_hold`,
`prayer_frames`, `prayer_haste`, `guard_break_stagger`, `blade_health`,
`flare_brace`.

## 6 · Reading it

**Four stances, four silhouettes**, because the fight is read from stances
rather than swings:

| Stance | Silhouette | Sound |
| --- | --- | --- |
| Guard | both blades crossed before the head; tall | a dry click on raise; a block is a clack, a **parry a high ring** |
| Coil | body dropped to 1.6 m, blades swept back, abdomen raised | a chitter that **rises in pitch** toward the coil's end |
| Ready | one blade cocked out wide on one side, head tilted to it | a single tick |
| Prayer | arms folded, head bowed, wings half out and still | a low hum; haste is three **glowing pips** on the thorax, one spent per move |

**Floor markers** (`Monster::telegraph`, the hit test's own answer): the lunge's
lane through the whole coil, with the fill reaching the edge at `coil_max`, not
at the release, because the release is yours to cause; the Leap's dive circle,
from the frame it leaves the floor; the pivot cut's arc, low-lit to say "over a
crouch"; the scythes' footprints (the second without a fill, §2). The guard's
arc is drawn faintly on the floor while it is up, with the elevation as a tint on
the blades — the guard is not a hit, but it is the thing a player must walk
round, and an arc that is only in the code is a guard nobody can go round.

**The memory must be drawn.** Four notches along each blade's outer edge, one
per remembered move, marked in the thrower's class colour with a glyph for the
move. When Ready is taken the notches of that move glow. A notch fades when it
is forgotten. That is diegetic and it is the whole of the habit's fairness: a
player who looks at its blades knows what it is waiting for.

**Unanswerable, for this creature.** Four moves have hits under fifteen frames
from their last visible change: the lunge's travel, the counter, the Ready
parry, and the second slash. Each is counted as *answered by position or
choice* only if:

- the lunge came after a coil of at least 20 frames with its lane drawn;
- the counter followed a parry of a move it saw (startup over its current D),
  or of a move thrown into a visible Ready or prayer;
- the second slash hit somebody inside four metres or in a recovery begun
  during the gap.

Anything else is unanswerable and must be zero.

## 7 · The classes

**Bulwark — the mirror.** Both of them guard, both parry, and both know it. His
guard blocks the scythes (its moves carry blockstun like the Ridgeback's), his
blocked scythes load the shield's weight, and his parry of a scythe should
stagger it the fighters' 34 frames — the first creature a parry is meant to
work on, since it is the size of a fighter (§12). His Bash is too fast to be
parried on sight; his Slam is not, and is parried; his Grapple is his guard
breaker and is slow, so it has to land in the minimum hold. His jump is the
lowest in the roster, so the Leap barely matters to him. His hole is the habit:
four moves into a guard, one of them Bash, is read in three exchanges. Hard, and
identity: this is the fight the class was written for.

**Champion — chains against the parry.** A blocked link continues at full
recovery ([../kits/champion.md](../kits/champion.md)), so sword, sword into its
guard raises it, and **Earthbreaker** as the third link breaks it inside the
minimum hold — that is the Champion's route, and it is the class fantasy exactly.
The hammer as an opener is parried; Rush out of a blocked link to its flank is
the other route. Mixed strings feed the memory three different moves, so the
Champion who mixes is almost never read and the one who opens with sword every
time meets Ready by the third string. The takeoffs (Rising cut, Uppercut, Pole
drive) put him in the air in front of the Leap. Medium; identity.

**Shadow Reaver — the shadow gets behind the guard.** Her copies are tested from
the shadow's position, so a shadow waiting at its back turns every body swing
into its guard into a clean hit behind it, and the lotus from there cuts its
back. The dash to the shadow beats the coil, because the lunge is aimed where it
saw her. It answers with the pivot cut and the flare when a shadow sits behind
it within four metres (`back(m)` counts the shadow as a body), but this is her
fight. Easiest of the six, and deliberately — two bodies is the answer to a
frontal guard — unless play says it is free (§12).

**Elementalist — stones as spacing.** A stone in the lunge's lane stops it dead
and staggers it 40 frames, and a raised stone takes 14 frames — faster than its
eyes — so raising one between you and the coil is her answer to it. A stone top
is floor, not air, so standing on one does not call the Leap. Her beam into its
guard is nothing; a fire pillar under its feet comes from below the cone and
lands; Updraft and Downdraft are unblockable and are her guard breakers. After
the Ridgeback, which she cannot win, she has the most answers here of anyone.
Medium; identity.

**Blood mage — Grasp through the guard.** Grasp is unblockable at 16 frames,
usually faster than its eyes, and it cannot be hauled — so the catch hauls her
to it (the anchor rule she already has), arriving touching it, inside a
guard-break stagger, at ×1.4 against the disabled, with a sweep winding up.
It is the best single window any class gets in the cast. Around it she is
out-spaced (her sweep reaches 2.4–3.6 m against its 4.0), and her blink to a
pool is her escape from the coil for the same reason as the Reaver's dash. Hard
to get in; the biggest payoff when she does. Identity.

**Dual mage — the aerial, punished.** Her class is the air and the Leap is the
air's answer; her float at 6 m is inside its rising cut. Her Judgement is not
unblockable, so she has **no guard breaker** here: her ways through the guard are
from above (where the Leap lives), from behind, and the light Lance's burst
aimed *past* it, which detonates behind the guard and lands. Her half-bar blink
beats the coil. Hardest of the six. The lesson — hold the floor — is identity;
having no guard breaker is a hole, and it is the ability-spec's open question
about `Q` showing up in a fight (§12).

## 8 · Coop

Tier 5 is built for two, and two changes the Mantis more than any creature so
far, because a frontal guard against two hunters is a guard with a back.

- **It lines them up.** A new term, `pair(m)`, scores walking so that both
  hunters fall inside its arc — backing toward a wall or a column until the
  angle between them, from it, is under about 100°. That is the duellist's
  answer to being flanked, and it is readable: it backs off to face you both.
- **The back is watched harder.** `back_patience` halves with two hunters; the
  pivot cut and the flare come sooner, and the flare hits everyone within three
  metres, which is often both.
- **The memory is shared.** Two Champions opening with the sword feed it
  together. Coop is also a mixup between two players.
- **One raises the guard, the other breaks it** — or one holds its front while
  the other goes round. The guard break is where two hunters cash in together:
  120 frames, both swinging.
- **Numbers:** health **6,800** (×1.7); `think_frames` −20%; everything about its
  eyes unchanged — two hunters never make it see faster than fifteen frames.

## 9 · Measuring it

**The scripted hunter's plan — the duellist's plan.** What a person learns in the
first ten minutes, in the order they learn it:

1. *Its front is a wall.* Stand at 5–6 m in neutral; inside 4 m only on a
   window.
2. *Don't blink under the coil.* While coiled, walk across the lane and reverse
   when the chitter peaks; never dodge, swing or jump.
3. *Hit the recoveries.* After a missed lunge, a Leap's landing, the pair's
   recovery, a guard lowering — and choose arm or body (the plan: arm until one
   blade is broken).
4. *Break the guard it just raised.* Throw the guard breaker only within 20
   frames of the guard going up.
5. *Don't repeat yourself.* Never the same move into its guard three times in
   four; on Ready, go round or wait.
6. *Stay on the floor* within 10 m. On Prayer: break it if in reach, else step
   out past 10 m and wait for the pips to be spent.
7. After a broken blade, circle to the broken side.

Three ablations prove the plan's parts matter: **the repeater** (steps 1–4 and
6, one opening move always), **the jumper** (jumps in from 6 m), **the dodger**
(dodges the coil). The duellist's plan must beat all three; if the repeater
does as well, the habit is not biting, and if it does much worse with the flag
off than on, the habit is doing more than the lesson.

**Report lines it adds** (P8, from the species table):

```text
into its guard, per hunter move     blocked / parried / broke
parried on sight                    by startup against its D at the time
ready                               taken / right / wrong / timed out
longest run of one move into guard
coil                                released on a commitment / at its end, landed after each
hits taken aloft                    and by which move
guard breaks                        in the minimum hold / in prayer / missed (it dropped it)
blades                              broken at (s), damage taken from the broken side
counters                            after a seen move / after Ready / after prayer / unanswered
second slash                        on a presser / on a stayer / other
```

**Targets.**

| Measure | Target |
| --- | --- |
| Scripted hunter wins (solo) | about **1 in 20** (tier 5) |
| Two scripted hunters win | about **1 in 2** |
| Fight length, won | **5–8 min** solo, 6–10 duo |
| Unanswerable hits | **0**, by §6's definition |
| Ready taken, and right | right about half the time against the repeater; rarely taken against the plan |

The four windows get a fifth band, because a Mantis with its guard up is not
threatening you and is not open either: **guarded** — its guard or prayer is up
and you are inside the cone, so a frontal hit is wasted.

| Window | Asked for |
| --- | --- |
| Threatening | ~40% |
| Guarded | ~20% |
| Open to a poke | ~15% |
| Open to a way in | ~10% |
| Safe to walk up | ~15% |

**The fight length departs from the tier table.** Bestiary §2 asks 8–20 minutes
for tier 5. A duel against something that kills in five to eight hits cannot
last twenty minutes solo without being a war of attrition, which is not this
fight; the table's upper end fits the Siegeshell. 4,000 health at the damage
rate the plan should reach — a guard break every forty seconds or so, whiff
punishes between — is five to seven minutes. The designer should confirm (§12).

## 10 · What it needs built

**Depends on:** P1 (species, and the move flags), P2 (the Shrine), P8 (the plan
and the report). No P3–P7. The bestiary calls it "no new machinery; all new
brain", and that is nearly true.

**New and its own:**

- **A creature guard.** `guard_against` generalised to a monster: arc, elevation,
  parry window, minimum hold, lowering, blocked pushback and blockstun on the
  creature, and the guard break. One function for both kinds of defender, for
  the reason the existing one is shared: a guard that stopped some kinds of
  attack and not others would be worse than none.
- **The delay-line sight** (§5): the sample ring and D's wander, beside the
  glance as a P5-style perception filter for this species.
- **Branching moves.** Guard → parry → counter, and the pair's two variants,
  declared in the move table (`follows`, as the Broodmother also needs).
- **The habit ring and Ready**, behind `Mantis · habit`, with the geometric guess
  as the flag's other side.
- **Breakable arms that change the guard**: the Ridgeback's broken foot,
  generalised from "turns worse on that side" to "guards less on that side".
- **Two small move flags:** `hits_crouching` on creature moves (the pivot cut),
  mirroring the fighters'; and `flare_brace`, crouch scaling a creature's shove
  as it already scales grip.
- **A lunge that stops at a solid:** one segment query against the arena's and
  the stones' solids at release, through `math.rs`.

**Aiming.** No change to `aim.rs`. It is the first creature at a fighter's scale:
the thorax spans 0.7–2.4 m, so a level swing from the chest meets it and a
ground-aimed skillshot, raised to 0.9 m, meets it. That is a claim, so it is a
test (`a_skillshot_at_its_feet_meets_its_thorax`). Aloft in the Leap it is a
body in the air: the crosshair's ray goes through it to the range sphere and the
shot's own line meets it by `aim::first_along`, as for any body. The guard reads
the point a hit comes from, as `guard_against` already does — including a
Reaver copy from the shadow — and never recomputes an aim.

**Snapshot.** Monster core with a 14-bone rig ~180 B; sight ring (2 hunters × 6
samples × 6 B, plus last-seen action and D) ~80 B; guard, Ready and coil state
~6 B; habit ring 4 × 2 B; blades 4 B; prayer pips 1 B. **About 280 B**, against
the bestiary's 240 — the sight ring is the difference.

**Per-frame cost.** Dominated by the rig pose and hit resolution, as for the
Ridgeback, with fewer bones. The guard test is a dot product and a compare per
incoming hit. The sight ring writes two samples every four frames. One segment
query per lunge release. Nothing scales with anything; pinned under `budget.rs`
with a Mantis scene.

**Tests** (`crates/sim/tests/mantis.rs`):

- `it_never_sees_the_present`
- `a_move_faster_than_its_eyes_is_blocked_not_parried`
- `the_counter_follows_only_a_parry`
- `an_unblockable_into_a_raised_guard_breaks_it_and_into_an_open_one_only_hits`
- `a_hit_from_above_the_guard_or_behind_it_lands`
- `a_walk_is_not_a_commitment`
- `the_coil_releases_on_a_commitment_it_saw_or_at_its_end`
- `a_lunge_into_a_solid_stops_at_it`
- `the_pivot_cut_goes_over_a_crouch`
- `a_broken_blade_leaves_its_side_open_for_good`
- `nothing_it_remembers_came_from_an_input`
- `habit_never_makes_its_eyes_faster`
- `a_guess_it_gets_wrong_is_forgotten`
- `prayer_never_makes_a_tell_shorter_than_reaction`
- `a_skillshot_at_its_feet_meets_its_thorax`
- `what_is_drawn_through_the_windup_is_where_the_hit_lands` (extended to it)
- `the_mantis_fight_fits_the_snapshot_and_does_not_allocate`

**Milestones** (a few hours of AI time in all):

- **M1 · Body, Shrine, guard.** Species entry, rig, the arena; the creature
  guard with arc, elevation, parry, hold, lowering and break. *Check:* the guard
  tests; `beastcheck --species mantis` prints its heights and the guard cone's
  top against every class's hop.
- **M2 · Its eyes and the coil.** Sight ring, D's wander, commitments; coil and
  lunge with the lane marker and the stop at a solid. *Check:* the sight and coil
  tests; a `SHOT_MOVE=lunge` screenshot.
- **M3 · The rest of the set.** Pair, counter, Leap, flare, pivot, prayer and
  haste, each with its marker. *Check:* per-move tests; screenshots of each.
- **M4 · Blades and desperation.** Arm health, the one-sided guard, the
  desperate stage. *Check:* the blade test; a scripted run that breaks a blade
  and wins from the broken side.
- **M5 · The habit.** Ring, Ready, the notches, the flag and the geometric
  guess. *Check:* the three habit tests; the report run with the flag both ways.
- **M6 · Measured.** The duellist's plan and its three ablations, twelve seeds
  each, solo and duo; the numbers go into this document. *Check:* the plan beats
  every ablation, or this document says why not.

## 11 · In the world

**The Shrine**, at the end of the world's fifth tier ([../world.md](../world.md)).
A round stone court thirty metres across, walled at 1.5 m, open to the sky, with
**four columns** (1.2 m wide, 8 m tall) standing ten metres from the centre.
No platforms: a platform is a place to be in the air from. The columns are the
arena's only cover, and they matter three ways — a column in the lunge's lane
stops it, a column is what it backs toward to line two hunters up, and a column
between you and it is somewhere to stand while it prays.

**Its idle life is the prayer.** From the trail you see it in the middle of the
court, arms folded, humming. Walking in starts the hunt, and its moment of
noticing you is the prayer's ninety frames: break it with a guard breaker and the
fight starts with a 180-frame window; wait, and it starts with three hasted
moves. The first decision of the fight is the one the rest of it is about.

**Trophy: a Mantis blade** — the one you broke, notches and all, or the right
blade if you won without breaking one.

**Sidegrade family: the guard** — parry windows and counters. The one proposed
here is the Bulwark's, the class it is most obviously for:

> **Duellist's guard** (Bulwark). The parry window is 7 frames instead of 4 —
> and a plain block pushes him 50% further and his guard turns a third slower.
> A shield for the player who parries, and worse for the one who holds it up.

## 12 · Open questions

1. **Keep the habit memory?** (Bestiary §6 q3.) The recommendation is keep, with
   the notches drawn, because it is the only thing in the cast that teaches "do
   not repeat yourself" and it passes item 5 on all three counts. The fallback
   is one flag away. What decides it is whether a person who reads the notches
   calls it fair.
2. **The second slash's marker shows where and not when.** Is that an
   acceptable exception to the rule that the marker fills to the hit, or should
   the delayed variant have its own pose instead?
3. **Should the counter be reactable?** At 12 frames it is answered by the
   choice before it, which the report can prove; at 16 it would be answerable
   on sight and much weaker. Which feels like a duel?
4. **Should a fighter's parry stagger a creature?** The Bulwark mirror wants it.
   The Ridgeback never allowed it, and a rule that one creature is parriable is
   a rule the next one will be asked to follow.
5. **Is the Reaver too comfortable?** Her copies behind the guard and her dash
   beating the coil are two answers where others have one. Identity, or a hole?
6. **The Dual mage has no guard breaker.** Her answers are the Lance burst
   behind it and the air. Is that her fight being hard on purpose — or is it
   the ability-spec's `Q` question, which this creature makes urgent: is `Q`
   the guard breaker on every class or not?
7. **Fight length.** 5–8 minutes solo when won, against the tier table's 8–20.
   Change the table for duellists, or raise the health and accept attrition?
8. **Does the coil read as fair?** The release is caused by the player and comes
   at 23–29 frames, after every dodge's invulnerability. Whether a person learns
   "don't blink" from it, or only learns that dodging is broken, is the question
   the whole fight rests on.

## 13 · Where it landed

Built 2026-10-01: `--hunt mantis` (`?hunt=mantis` in the browser), the Shrine.
The species is `crates/sim/src/species/mantis/` (`fight.rs` the guard, the
reflexes and the body's moves; `sight.rs` the eyes; `habit.rs` the memory and
the notches; `mind.rs` its terms), the court `crates/sim/src/arena/mantis.rs`,
the clips `crates/anim/src/beast/mantis/`, the plan and its ablations
`crates/hunt/src/plans/mantis.rs`, the tests `crates/sim/tests/mantis.rs`.
Every magnitude is a knob in the Oven's `Mantis · …` families.

**What the rest of the cast got from it.** A blow reaching any creature now goes
through `Monster::take_blow` with a `monster::Blow` (where it came from, whether
it is unblockable, who threw what), which asks the species' `FightDecl::guard`;
every hit site does (swings, echoes, recalls, effects, beams, bolts, gusts,
debris), and every other species answers `None`, bit for bit what it did before.
The cone is the fighters' own (`state::in_guard_arc`, shared with
`guard_against`). `FightDecl::sight` lets a species replace the glance's present
with what its eyes saw; `MarkLook::Notch` draws a remembered move; the report
has a fifth band, **guarded**, printed only where it is not zero.

**Numbers**, from `cargo run -p hunt --bin fight -- --species mantis --class <c>
--repeats 24` (duo `--hunters 2 --repeats 12`), the scripted duellist of §9,
after the passes in [feel-log.md](../feel-log.md) of 2026-10-01:

```text
                solo won   mean    duo won   mean   threat / guarded / poke / way in / walk up   unanswerable
  Champion       0/24      --       4/12    115 s      38 / 26 /  9 / 17 / 10 %                   0
  Bulwark       18/24     246 s    11/12    178 s      47 / 18 /  7 / 15 / 13 %                   0
  Reaver         0/24      --       0/12     --        39 / 29 / 11 / 16 /  5 %                   0
  Elementalist   0/24      --       7/12    133 s      41 / 24 /  9 / 17 /  9 %                   0
  Blood mage     0/24      --       0/12     --        38 / 29 / 10 / 17 /  7 %                   0
  Dual mage      0/24      --       0/12     --        33 / 37 / 11 / 16 /  3 %                   0

  Champion solo, 216 hunts on four seed sets: 14 won (about 1 in 15), 3-4 min
  Champion duo, 44 hunts on two seed sets: 22 won (half), about 2 min

  the plan and its ablations, Champion solo, 48 seeds each
                won    dealt (of 7000)   Ready taken / right
    duellist    5/48       5507               213 / 0
    repeater    0/48       4593               366 / 148
    jumper      0/48       1914                 0 / 0
    dodger      0/48       5207               243 / 0
    habit off   0/48       5359               148 / 0   (the geometric guess)

  its own lines, 24 Champion hunts (§9)
    parried on sight 18, otherwise 1          counters: after a seen move 18,
    coil: released on a commitment 60           after Ready 0, after prayer 1,
      (42 landed), at its end 356 (11)          unanswered 0
    guard breaks: in the hold 50,             second slash: on a presser 0,
      in prayer 0, dropped before it 324        on a stayer 20, other 1
    Ready 110: right 0, wrong 7, out 103      a blade broken in all 24
```

**Against the targets.** Zero unanswerable hits for every class, solo and duo.
The Champion wins about one in fifteen (§9 asks one in twenty), and two of them half the time. The
plan beats every ablation on wins and on damage; the jumper is the one the Leap
punishes hardest (a third of the damage), the repeater the one the habit reads
(Ready right 148 of 366, two in five, against "about half"), and against the
plan Ready is taken about four times a hunt and is never right -- the duellist
never throws the remembered move into it. With the flag off it wins nothing,
because a guessed Ready is a stance it is not hit in. The windows sit on §9's
(threatening 38 against ~40, guarded 26 against ~20).

**What departs from the document.**

- **Health 7,000 solo, 8,400 with two** (not 4,000 and 6,800), raised pass by
  pass until the Champion won about one in fifteen: the plan finds the guard
  break and the whiff punishes faster than §9 guessed. A won fight is three to four minutes solo and two duo
  -- under §9's five to eight, and further under the tier table (§12, 7).
- **Damage ×0.83** on every scythe (75 / 100 / lunge 140 / counter 180 / leap
  110 / dive 90 / pivot 100), so a hunt is lost in eight to twelve mistakes, not
  five to eight, which is what the longer health asks for.
- **The guard has a longest hold** (`holds it at most`, 120 frames from when it
  went up) **and a rest after** (40 frames before it is raised again). Without
  them, against two Blood mages, it was up for nearly the whole hunt; a blocked or
  parried hit still recommits it (`a blocked hit holds it`), which is what
  makes the minimum hold (44) the window for the breaker.
- **A move seen too late to parry** -- one whose hit is already out when its
  eyes see the startup -- raises the guard past its parry, against the next link,
  on `late guard` (35 %) of such commitments, chosen once each. Always raising it
  made every string a wall; never raising it made the guard decorative.
- **The habit remembers a Champion's weapon, not his link**: his nine chain
  moves are three weapons swung three ways (§5's "the swing, not the swinger").
  Remembered by link, a sword string read as three different moves and the
  repeater out-earned the duellist.
- **The court is square with filled corners**, walled at 15 m, not round: the
  arena kernel's walls are boxes. The four columns stand ten metres out on the
  diagonals.
- **Bulwark wins three in four.** His Slam on a whiff is 180 a hit, he has the
  most health, and the plan's Bulwark does not need the guard break to win. A
  creature-side answer would be a rule about one class; the question is a
  person's (§12, 4, and a new 9).
- **The Reaver, Blood mage and Dual mage win nothing, Elementalist only in a
  pair.** The plan does not play the shadow behind its guard (§7's "this is her
  fight"), the pools or the bars; the numbers are the plan's reach, not the
  class's.

**Open questions added.**

9. **The Bulwark is the easiest class, not the hardest.** §7 calls the mirror
   hard. Measured, his whiff punishes are worth twice anybody's. Lower the
   creature's damage taken from a Slam (a rule for one class), accept it, or
   give the plan's other classes their tools before deciding?

