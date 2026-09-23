---
status: built — v1, unplayed
decided: 2026-09-09
revised: 2026-09-23
sources: ../blood-mage.md, ../plans/blood-mage-v1.md, docs/archive/combat-design/blood-mage-skills.md
---

# Blood mage — kit

**Identity.** Her blood goes out, and theirs comes back. Every cast is a cut she makes in
herself; every hit spills the other fighter onto the floor; and the only way to close her own
wounds is to put an ability through the blood she spilled. A war scythe, a reaper's reach that
grows with how open she has left herself, and a floor she wants to fight on.

> **Rebuilt 2026-09-23.** This is the v1 kit of [../blood-mage.md](../blood-mage.md), built
> along [../plans/blood-mage-v1.md](../plans/blood-mage-v1.md): grey health, essence pools,
> the scythe on the clicks, the blink on the dodge. Everything below describes what is in the
> game. **None of it has been played by a person.** The four human checkpoints in the plan
> are open, and the feel-log entries for the four milestones say so. The economy it replaced
> — cost on the press, leech on the hit, a field that drains — is recorded under
> [Was](#was) at the end.

The specification is [../blood-mage.md](../blood-mage.md). This document is what the game
does, where it differs from the proposal, and why. `cargo run -p sim --bin essence` prints
every number named here, measured; `cargo run -p sim --bin frametable` prints the frames.

## Mechanic — grey health and essence

### Grey health

Health she loses — spent on a cast, or taken from a hit — turns **grey** rather than going:
a segment of the bar beside the red that is no longer hers but can still be reclaimed. The
HUD draws it as a faded tail on the red. Only a drink from a pool turns grey back to red.
Grey **fades** on its own, at a fixed rate rather than a fixed time (`Grey fades`, 12 a
second), so a large wound stays open longer than a small one and there is always a clock.

`red + grey + gone == max` on every frame, and `gone` only ever grows by the fade —
`crates/sim/tests/grey.rs` holds it. A cut can never kill her: self-damage clamps red at one,
and what was actually paid is what turns grey. Enemy damage lands on red and turns grey by
what it dealt; nothing erases grey but the fade.

**The more grey she carries, the longer the scythe.** The two scythe moves multiply their
reach by a straight line from one at no grey to `Scythe reach at full grey` (×1.5) at a full
bar, and their damage by a second line to `Scythe damage at full grey` (×1.3). This is the
one reach in the game that scales with a bar — [../aiming.md](../aiming.md) refuses that
everywhere else — and it is allowed on one condition: **the blade is drawn at the length it
hits at.** `view::scythe` reads the same function the hit test lengthens the sweep with, the
game hangs the blade off her grip at that length whether or not she is swinging, and
`view/tests/kinematics.rs` checks the drawn blade against the hit volume at three levels of
grey. Both players read a weapon, not a number.

| Grey | Reach | Damage |
| --- | --- | --- |
| 0 | 2.4 m | ×1.00 |
| 250 | 2.7 m | ×1.07 |
| 500 | 3.0 m | ×1.15 |
| 750 | 3.3 m | ×1.22 |
| 999 | 3.6 m | ×1.30 |

**How it is drawn.** A war scythe is a haft with a blade set at the head, and it is drawn
as two pieces: the haft from her grip to the neck, and the blade — flat, as wide as the arc it
sweeps — from the neck to the tip. The tip is what the hit test reaches with, and it sits
exactly at the far end of the hit volume while she swings; the neck bows a little off that
line so the blade reads as a blade. At rest the weapon stands upright beside her, leaning
forward, with the tip the live reach from her grip. `view::scythe` is the geometry.

> **Was**, for a day: one bar from the hand straight out along the facing at the full reach,
> which read as a five-metre pole pointed at whoever she was looking at.

So grey is risk and power in one segment. A cast at full health opens a wound and lengthens
the blade; being hit does the same; drinking gives the power back and shortens it. The edge
she rides is *how much of my bar am I willing to leave open.*

The fade's first number was chosen so that a Reap's worth of grey (its cost, 55) survives one
whole exchange — the Reap's own startup to the end of its recovery plus a dodge, 72 frames —
with more than half of it left. `grey.rs` pins that relationship, not the number.

### Essence pools

**Every hit she lands spills the target.** What is left on the floor is not a puddle but a
**figure**: a shadowy column the size of the body it came out of, standing where their feet
were — under the fighter on a direct hit, under the struck part's floor projection on the
creature. Its **essence** is the damage dealt, and the essence decays at a constant rate
(`Pool drains`, 10 a second) until it is gone.

- **Size follows essence.** A full body's width and height at `Pool, full-sized at volume`
  (110, a Reap's worth), shrinking as it drains and never below `Pool, smallest share of a
  body` (×0.35). It never stands taller or wider than the mage, and it does not spread over
  the ground.
- **Solidity follows essence too.** The renderer draws the figure faint when little is left
  and nearly solid when it is full, in six steps, so the figure on the floor *is* the heal it
  is worth.
- **They merge.** A hit onto a spot within a body of a pool of hers adds to it rather than
  standing a second figure beside it.
- **Cap of four** per Blood mage (`Pools at once`); a fifth merges into the newest.
- **They are effects**, in the fixed effect array — which grew from eight slots to twelve to
  make room for them beside a blade, a Grasp and a spike. Drawn at exactly the column they are
  tested at, in the arena and in the overlay. Nothing collides with them.
- **Her own blood never pools.** A cost is paid into the ability, not onto the floor. A hit
  on somebody in the air spills nowhere — the simpler of the two answers in the proposal's
  open question, kept until somebody plays it.
- **The creature bleeds too**, and a pool under a toppled Ridgeback is a door onto its back:
  see [Blink](#blink--the-mechanics-movement-shift--a-direction-with-the-crosshair-on-a-pool).

Measured, one cast landed on the dummy on bare floor:

| Move | Cost | Dealt | Essence | Radius | Lives |
| --- | --- | --- | --- | --- | --- |
| Reaping sweep | 3 | 22 | 22 | 0.18 m | 64 f |
| Reap | 55 | 111 | 111 | 0.50 m | 586 f |
| Bloodletter | 8 | 40 | 34 | 0.18 m | drunk by its own return |
| Grasp | 60 | 192 | 192 | 0.50 m | 1102 f |
| Black spike | 80 | 100 | 100 | 0.45 m | 524 f |

> **Was**, for a day: a disc on the floor with a radius of 0.24 m per root of the essence,
> which put a Reap's pool across two and a half metres of arena and a Grasp's across three.
> Too big, and it spread; the figure replaced it on 2026-09-23.

### Double duty — the drink

An ability that **lands over a pool** does its ordinary job and *also* drinks — and **a
drink is one and done.** The pool is spent the moment a move is put through it, whatever
came back: what she gets is the move's share (`Drinks of a pool (%)`, a column in the move
table) of the essence *remaining* in the pool, converted out of grey and never past it, and
anything the share or the grey did not take is lost with the pool. A pool counts if the
victim is standing against it or the hit volume passes over it; the fullest one is drunk.
**The drink comes before the spill**, so a hit on bare floor returns nothing, and a hit that
drinks leaves only its own fresh, smaller figure behind.

| Move | Share of what is left | From a Reap's pool (110), with grey to fill |
| --- | --- | --- |
| Reaping sweep | 35% | 38 |
| Reap | 100% | 107 |
| Bloodletter | 30%, on the way home | 42 |
| Grasp | — | — |
| Black spike | 100%, and the pool erupts | 105 |

The shares are what make the sweep a bad thing to put through a big pool: it takes a third
and wastes the rest, where a Reap takes all of it. Grey is the ceiling: at full health a Reap
over a pool gets back exactly its own cost and the pool is gone all the same.
`crates/sim/tests/essence.rs` pins the two halves of the feel relationship — landed over a
pool of its own making, each of the sweep, the Reap and the Bloodletter returns more than it
cost; landed on bare floor it returns nothing on the frame it lands — and that a drunk pool
is gone.

> **Was**, for a day: a drink took its share and the pool kept the rest, so a pool could be
> hit over and over for health. One and done since 2026-09-23.

### Naturally deals increased damage to disabled enemies

Kept exactly as built: ×1.4 to anything staggered, held, or a creature on its side. It is
what makes a Grasp a setup rather than a tax, and it multiplies the pool a Reap leaves.

## What is bound

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Reaping sweep** | The auto. A war scythe drawn across the front. Reach grows with grey |
| `R` | **Reap** | The committed heavy. The scythe over and down, unblockable, an overhead. Over a pool the biggest hit and the biggest drink she has |
| `M` | **Bloodletter** | A blade thrown a fixed distance and back, cutting on both passes |
| `Q` | **Grasp** | Hold to choose a depth; four arms converge. All four catch, bind, and haul the victim to her feet |
| `E` | **Black spike** | A spike out of the floor after a delay. On a pool, the pool erupts |
| `shift` + direction, crosshair on a pool | **Blink** | She is at the pool. Consumes it |

Five abilities, an auto, and the mechanic's own movement on the dodge. Right click is free on
this class (no shield) and takes the committed melee; middle click is the third click and
takes the throw.

**The move table is five rows in storage order, not button order.** The Oven packs the move
store in class order, so a slot appended to this class shifts every Dual mage index in
`tuned.rs` and a slot renumbered silently rewrites every number on the class. The four rows
that existed before the scythe kept their knobs — the Bloodletter is still slot 0, Reap took
Rend's row and was retuned rather than replaced — and the sweep was appended as the fifth. See
`moves::blood`.

## Abilities

### Reaping sweep — auto, `L`
**Startup** 8 · **Active** 5 · **Recovery** 14 · **Damage** 22 · **Reach** 2.4 m, growing
with grey · **Cost** 3 · **Drinks** 35% of what is left · −10 on block, 0 on hit ·
**Repeat lockout** 100%

A flat swing across the front, 0.45 of a turn from the body's left to its right — the Dual
mage's wing reasoning with the shape turned into a weapon: a blade on a long haft covers width,
and a body that has closed inside the haft is inside the sweep. The **tip** hits harder:
anybody caught further from her feet than `Sweep, tip as a share of the reach` (65%) of the
blade's live length takes `Sweep, tip damage` (×1.5), decided per defender rather than per
frame, so one sweep can catch one body on the haft and another on the point.

Over a pool it is the trickle heal, thrown constantly — never the reason to make a pool, but
why standing in one while she trades is the right place for her and the wrong place for them.

Its clip is authored low to high, hands gathered off the left hip and swept across, which is
what tells it from the Dual mage's level Sweep at a glance; the hit volume itself is level, at
`sweep height`, because `Plane::Flat` is the one that owns the width of the front.

### Reap — committed, `R`
**Startup** 20 · **Active** 4 · **Recovery** 26 · **Damage** 110 · **Reach** 2.4 m, growing
with grey · **Cost** 55 · **Drinks** all of what is left · −17 on block, +1 on hit ·
unblockable · overhead

The scythe raised over the right shoulder and brought over and down: an upright swing of 0.4
of a turn. Unblockable — the class's guard breaker, on the special's neighbour — and an
overhead, so it passes over a crouch. Slow enough to read and punish, and the reward is the
biggest number in the kit, the biggest pool a single hit can leave, and, landed on somebody
standing in essence, a drink of most of it.

This is the *double duty* ability: a Reap on bare floor is a heavy; a Reap on a pool is a
heavy and a heal. Landing it there is the thing the rest of the kit exists to arrange.

> Took Rend's row. Rend was a one-handed claw at chest range with no button since shift
> stopped modifying clicks; the row was retuned and its clip re-authored as the overhead, so
> its knobs kept their index in the Oven.

### Bloodletter — throw, `M`
**Startup** 7 · **Active** 3 · **Recovery** 14 · **Damage** 20 a pass · **Reach** 7 m,
skillshot · **Cost** 8 · **Drinks** 30%, on the way home

A blade thrown along the crosshair to a fixed distance and back to her, cutting on the way out
and on the way home. Unchanged from the built kit except in two ways: it moved off the auto to
middle click, and **it brings back a cut, not health.** The leech on the catch is gone. Its
return leg crossing pools on the floor is a small drink on the way back — each pool once per
blade, and only the crossing: the version where its cuts also drank took three shares of one
pool in one throw and out-healed the Reap, and was reverted (feel log, 2026-09-23).

### Grasp — special, `Q`
**Startup** 16 · **Active** 4 · **Recovery** 20 · **Damage** 48 an arm · **Reach** chosen by
the hold, 1.5–10 m · **Cost** 60 · **Hold** 26 frames, 10 of them bound

Kept as built, because the built version is the best-argued ability in the class: hold to
choose a depth on the crosshair's line, four arms converge there, each arm damages and spills,
and only all four catch. The catch binds and then hauls at forty metres a second to **her
feet** — the victim arrives touching her, held, disabled, at ×1.4, on whatever she is standing
in, in reach of the Reap that is already winding up. Everything about the channel and the catch
is as it was: see [Holding `Q` chooses the depth](#holding-q-chooses-the-depth) below, kept
from the previous revision.

Against something that cannot be hauled — the creature — the catch **hauls her to it**: all
four arms landed on a Ridgeback's flank pull her across the gap to the contact point at the
same reel speed, with the arena and the creature solid under her, so she arrives against its
side. A second route up.

### Black spike — mechanic, `E`
**Startup** 30 · **Active** 4 · **Recovery** 22 · **Damage** 100 · **Reach** 9 m, grounded ·
**Cost** 80 · launches 9 m/s · slows

A spike out of the floor where the crosshair is, after the same long delay as before: the cast
is the telegraph. What erupts depends on the floor, decided the frame it comes up:

- **Bare ground:** a spike. The move's own disc does the damage, the launch and a short slow
  (`Black spike slow`), and it spills what it hits — how she seeds a pool at range where no
  scythe reaches. The launch and the spill are ordered so the pool lands under where the
  victim *stood*, not under where the launch has since put them.
- **A pool of hers:** the whole pool erupts, launching and slowing everything standing in it
  at the move's damage times `Black spike, eruption damage` (×1.5), and **drinks the entire
  pool** in one go. The eruption is sized by the pool's essence — `Black spike, eruption
  radius per root of volume` (0.3) times the root of it, so a Reap's worth of blood comes up
  across three metres against the bare spike's 1.6 — and stands `Black spike, eruption
  height` (3.4 m) tall against the bare spike's 2.2. The pool is spent whether or not she
  had grey to fill. The bigger the pool, the bigger the eruption, the harder it hits and the
  bigger the heal.

  > **Was**, for a day: the eruption was the pool's own radius and the spike's own damage,
  > which on a sweep's pool was *smaller* than the bare spike and looked identical to it.

**The drain field is gone**, and with it its three knobs (radius, lifetime, drain). Nothing
ticks. The spike stands for `Black spike, eruption lasts` (20 frames) as the thing you can see
from across the arena, and is drawn at the disc it hit.

### Blink — the mechanic's movement, `shift` + a direction with the crosshair on a pool
**Startup** instant · **Recovery** the dodge's own · **Range** wherever a pool of hers is ·
**Cost** the pool

A dodge thrown with the reticle on one of her pools puts her in it, instantly, and the pool is
spent. It is the Reaver's forward dodge pointed at the class's object: `aim::pointing_at_disc`
— the ray from the eye through the crosshair against a short cylinder standing on the pool's
own disc, `Crosshair lock on a pool` tall — and `aim::clear_between` for a straight line from
her to it that nothing solid crosses. The nearest pool the crosshair is on wins. On the ground
and in the air; the airborne one spends the airdodge, and a pool is on the floor so it also
lands her. It is invulnerable for the dodge's own opening frames, the way every dodge is.

Refused with no pool under the crosshair (an ordinary dodge happens instead), with the line
blocked by a stone, and in the air after the airdodge is spent — `crates/sim/tests/blink.rs`.
Nobody else blinks to her pools.

**A pool is a heal or a door.** A blink eats the heal, and a player who blinks to every pool
never heals, while one who drinks every pool never moves. A pool draining is the other limit:
a door that is closing is a door you have to decide about now.

## Playing it

Cast to open a wound and lengthen the blade. Sweep to spill them. Read where the blood fell and
bend the fight back onto it: Grasp them onto the pool you are standing in, or Bloodletter them
where they stand to start one there, or blink to the pool they left behind and Reap them as
they come back for you. Cash grey in when it is about to fade or when you are about to die,
and not before — every point of grey you are carrying is reach.

In a hunt the same sentence reads: hit the creature until the floor under it is red, ride the
topple for the ×1.4, and when it is on its side the pool under it is a door onto its back.
The scripted hunter (`cargo run -p hunt --bin fight -- --class blood`) Reaps when the
creature is open and drinks from the pools its sweeps leave; the report's **THE BLOOD**
section counts pools made, health drunk, and health drunk while the creature was on its side.

**Counterplay.** Do not stand in blood. Keep moving, so the pools she makes are small and far
apart and gone before she can use them. Punish her while she is grey: a Blood mage with a
long blade is a Blood mage with a short bar, and she cannot heal without going somewhere you
can see.

## Holding `Q` chooses the depth

The only move in the game you aim with *time*. Press and hold and a small marker leaves the
caster's chest and travels outward over half a second, from melee range out to ten metres; let
go and the arms converge on wherever it had got to. Hold past the end and it throws itself.

**The marker is not a projectile.** It is the far end of `Player::aim_path`, solved every
frame of the wind-up by the same `sim::aim` call the finished move uses. **The line is solved
at the move's full reach every frame, and the hold only picks a point along it**, which is what
keeps the marker from jumping about while the mouse is still; **the line is read for its
direction only**, so a full hold at a wall two metres away still reaches ten, through the
wall. Aiming stays live through the wind-up and locks on the frame the button comes up. The
health is paid on the press, because there is no cancelling out of a channel.

**The catch: bound, then hauled.** About a sixth of a second held exactly where the arms
closed, then dragged to her feet at forty metres a second, feet handed back the same frame the
haul ends. The whole catch is shorter than her most expensive cast takes to come out, on
purpose: whatever is supposed to meet them at the end of the trip has to have been committed
to before the Grasp was known to have landed.
`preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution` and
`a_grasp_always_finishes_hauling_before_it_lets_go` in `feel.rs` pin it.

## Open questions

Carried from the proposal, with what the build found beside each.

- **How fast does grey fade?** 12 a second. A Reap's cost survives one exchange with more than
  half left; the first thing C1 will say is whether the blade ever gets long enough to
  matter.
- **How much reach does a full bar buy?** ×1.5, the design's first number. Read across the
  arena is the whole justification; nobody has looked at it from across the arena.
- **Blink consumes the pool.** As written. If the class turns out to have too few pools to
  spend one on moving, the middle answer is a blink that takes a share, the way the sweep
  does.
- **Should the blink cost health?** It does not. One more cost on an already expensive class.
- **Grasp onto her feet.** The haul's endpoint is touching distance, which for any pool
  wider than a body is inside it. Whether half a second of channel is worth that is C3's.
- **Pools on slopes and platforms.** A pool is spilled where the victim stands; a victim in
  the air spills nowhere, and a creature's blood falls to the arena floor whatever it stood
  on.
- **The figure.** The proposal's idea is that the essence starts as a shadow of the
  target's own model and coalesces into the figure. It is a column today, body-sized and
  shrinking; a copy of the target's skeleton fading into the floor is the next step and is
  presentation only.
- **Costs.** 3 / 55 / 8 / 60 / 80 against a thousand-point bar, raised from the built kit's
  because they are recoverable. The two relationships that bound them — one cast is never a
  third of the match, and a cast over its own pool returns more than it cost — both hold with
  the sweep's cost at three; at four the second one is a coin toss on the drain.
- **The `leech` column stays in the move table.** It was to be removed, but the Dual mage's
  dark auto and tether heal through it and her test pins that; it is zero on every move of
  hers and `feel.rs` keeps it there.
- **The name of the essence.** Still "pool" and "essence" both.

## Was

The kit this replaced, 2026-09-12 to 2026-09-23. Four abilities, four buttons, one economy:

| Key | Ability | What it did |
| --- | --- | --- |
| `LMB` | Bloodletter | A blade out and back; the blood it cut was banked and paid on the catch (40%) |
| — | Rend | A raking claw at chest range, 30 health for half back; no button since 2026-09-16 |
| `Q` | Grasp | As now, but the catch hauled to arm's length and each arm leeched 55% |
| `E` | Black spike | A spike in a draining, slowing field: 11 a tick for four seconds, 59% back |

**Cost on the press, leech on the hit.** Every ability cost health the moment the button went
down and returned a percentage of the damage it dealt, wherever the damage happened — a direct
hit immediately, a field on every tick, the blade when caught. The auto was the only one that
reliably profited; every committed ability was close to break even at its best and a straight
loss at its worst.

It turned out to be four abilities with a red colour scheme and a percentage. A number that
pays out wherever the hit lands gives the player nothing to *go to*, so a class whose
sentence was "sustain through aggression" shipped with three tools that rewarded standing
back. The rebuild's argument is in [../blood-mage.md](../blood-mage.md) §"Why this shape".
What survived: the Grasp entire, the spike's thirty-frame telegraph, the thrown blade, the
×1.4, and the rule that a cut can never kill her.
