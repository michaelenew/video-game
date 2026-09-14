---
status: implemented
decided: 2026-09-10
rebuilt: 2026-09-12
chained: 2026-09-14
formerly: Bellator, Shifter
sources: docs/archive/combat-design/shifter-skills.md
depends: ../champion.md
---

# Champion — kit

**Identity.** Range bands and flow, in three dimensions. Three weapons on three mouse
buttons, a three-hit string where every hit is a free choice of all three, a dash that
changes what all three of them do, and an air game that is the point of the class rather
than a place it visits.

## The grid

The whole input scheme, and it is worth reading as a table rather than as a list of
nineteen moves:

| | Left click | Middle click | Right click |
| --- | --- | --- | --- |
| **On foot, hit 1** | Sword — arc across | Hammer — arc down | Spear — a line ahead |
| **On foot, hit 2** | Backcut — back the other way | Uproot — torn up out of the floor | Skewer — a second, shorter |
| **On foot, hit 3** | Crescent — a full turning cut | Earthbreaker — the guard break | Impale — the longest lunge |
| **In the air** | Air sword — the arc rolled vertical | Air hammer — wind up, then spike | Air spear — a fan around the aim |
| **Rushing** | Rush slash — cut as you go past | Rush sweep — dragged along the floor | Rush stab / Pole vault |
| **Leaving the floor** | Rising cut — up and away | Uppercut — launch, and hold on | Pole drive — the floor throws you |

**The button is the weapon and never changes meaning. The row is the situation.**
That is the entire thing a new player has to learn, and it is why nineteen moves fit on
three buttons with one modifier: you never choose a move, you choose a weapon, and the
situation chooses the move.

The nineteenth move shares right click with the Rush stab and is separated by where you
are pointing — a spear put into the ground vaults, a spear levelled at somebody stabs.
That is the only overloaded input in the class and it is the honest one: a pole vault *is*
a spear planted in the floor.

`Q` is unassigned. The class's identity is the three weapons and the dash, and there is
nothing left that wants a key.

## The chain — rebuilt 2026-09-14

**The first three rows are one three-hit string, and every hit is a free choice of all
three weapons.**

Connect with anything in the first row and the same three buttons throw the second row;
connect again and they throw the third. Nothing remembers what the previous hit was made
of, so *sword into spear into hammer* is an ordinary thing to do — and it is the class
fantasy written as a move list. One haft, three heads, chosen a hit at a time.

### Why the weapon being free is the whole point

Each weapon owns a different piece of space — the sword owns width, the hammer owns the
line underneath, the spear owns distance — so a **mixed** string covers three different
volumes in three beats and a **pure** one covers the same volume three times. Against
somebody moving, that is not a flavour choice, it is the difference between the third hit
landing and the third hit whiffing. Nothing has to reward mixing for mixing to be correct;
the shapes do it.

The frame data does one small thing on top of that, and it is deliberately small. **A
connected link cuts its own recovery short so the next one can start, and swapping weapons
cuts it shorter than repeating one** — because the head is re-formed out of the
follow-through rather than re-chambered. On the sword that is three frames against six; on
the hammer six against twelve. Linear play stays completely viable, which is the "strong
when played linearly" half of [the design](../champion.md#identity); mixing pays a little
better, which is the other half. The two knobs are `Chain, recovery owed on a swap (%)`
and `…on a repeat (%)` in the Oven, and setting them equal turns the incentive off without
changing anything else.

### The chain is a hit confirm

**A blocked, parried or whiffed link pays its whole recovery.** The string still continues
— the button does something, which matters — but it continues at the speed the frame table
prints rather than at the cancelled speed.

Three things fall out of that, and all three are load-bearing:

- **Every number in the frame table stays true.** On-block advantage is measured from the
  full recovery, and against a defender who blocked, the full recovery is what you pay. A
  cancel available on block would have quietly made half the kit plus on block.
- **Blocking one hit is worth doing.** It costs the attacker about thirteen frames across
  a three-hit sword string, which is a real window rather than a moral victory.
- **Whiffing is still punishable.** Throwing the opener at nothing and continuing anyway is
  a choice you can make and a choice you can be hit for.

### What a string costs and what it buys

| | Frames, start to free | Damage | Character |
| --- | --- | --- | --- |
| Sword ×3 | 80 | 234 | The fast one. Widest coverage, smallest commitment |
| Spear ×3 | 87 | 230 | The same, a metre and a half further out |
| Hammer ×3 | 121 | 212 | The slow one, and it ends with a guard break |

The hammer being both the slowest and the least damaging is the same deliberate choice it
always was: it buys stagger, knockback and — on the finisher — an attack that goes through
a shield. Live numbers: `cargo run -p sim --bin frametable`.

**A string dies** if you stop swinging for about half a second, if you are hit, if you
block, if you dodge, or if you leave the floor. It survives a Rush, which is worth knowing:
one charge buys you a reposition in the middle of a string, and the grace window keeps
running while you dash.

### The rhythm: fast, fast, slow

Each weapon's second hit is **faster** than its opener — the weapon is already moving —
and its third is **slower and much bigger**. That is the shape of the whole thing, and it
is what makes the decision to commit to a finisher a decision:

| | Hit 1 | Hit 2 | Hit 3 |
| --- | --- | --- | --- |
| Sword | 7f, 66 | 6f, 72 | 11f, 96 |
| Hammer | 15f, 48 | 13f, 54 | 20f, 110 |
| Spear | 10f, 58 | 8f, 64 | 13f, 108 |

Each finisher also does something the first two do not. **Crescent** is the widest volume
in the game — most of a half turn, and it catches everything in front of you. **Earthbreaker**
is **unblockable**, which is this class's answer to a turtle. **Impale** reaches four and a
half metres, which is further than anything else on the ground.

## The takeoffs — a weapon on the way off the floor

Press a weapon on the same press as jump and you get that weapon's **takeoff** instead of
its grounded or airborne move. Three moves, one per weapon, and they are three different
reasons to leave the ground:

| | What it is for |
| --- | --- |
| **Rising cut** (sword) | An angled slash up and forward, and the hardest hit of the three. No grab, no boost, nothing else in it — and a long fall if it misses |
| **Uppercut** (hammer) | Launches, and *holds on*: both fighters leave the ground together. Press space again and you both go higher, once |
| **Pole drive** (spear) | The butt of the spear cracked into the floor at your own feet. The least damage in the class and the most height, plus a shove in whatever direction you are holding |

Read from behind they are a **diagonal**, a **column** and a **vault**, which is how you
tell which one somebody threw while all three are in the air.

**The window is a few frames wide**, because "attack as you jump" is one intention and two
buttons and nobody presses two buttons on the same frame. Jump then weapon is the window;
both at once is the same frame. A takeoff spends the window, so one jump buys one of them.

Aboard the creature there are no takeoffs: jumping is how you leave its back, and a move
that spent the jump on an attack would take that away.

## What replaced the form toggle

The form used to be a **mode**: a key cycled hammer → sword → spear, and the form
multiplied the reach, damage and recovery of one shared three-move table. Nine numbers,
three moves, one button to rotate between them.

That is why the class read as linear, and the reason is worth stating because it
generalises. **A multiplier cannot make two moves feel different, because it does not
change what they do** — it changes how much. A spear with 1.55× reach is a sword that
reaches further, and a hammer with 1.35× damage is a sword that hits harder, and all three
of them were the same swing with the same shape at the same height. The choice between
them was arithmetic, so playing the class was arithmetic.

The weapons have their own moves now, with their own frame data and — the part that
actually carries it — **their own shapes**. See [Hitboxes](#hitboxes).

## The three weapons

The openers, which are what a weapon is before you have committed to a string:

| | Reach | Startup | Recovery | Damage | On hit | Character |
| --- | --- | --- | --- | --- | --- | --- |
| **Sword** | 2.0 m | 7f | 11f | most | +0 | The combo tool. Small knockback, small hitstun, and you can walk while you swing it. |
| **Hammer** | 1.6 m | 15f | 22f | least | +7 | The crowd-control tool and the string *starter*. Long stagger, and it slows you to a crawl. |
| **Spear** | 3.4 m | 10f | 14f | middling | +0 | The spacing tool. Longest reach in the game, thin, and it goes over a crouching opponent. |

The hammer doing the least damage is deliberate and it is what "heavy" means here: it buys
stagger, not numbers. Its +7 on hit is the whole of its job — it is the move that starts
the exchange the sword finishes.

**The openers and the second hits are the softest things the class throws, on purpose.** A
hit that shoves somebody out of range of the next one has ended the string whether or not
the game says so, so the knockback lives on the finishers: four on the hammer's opener,
eleven on Earthbreaker. That rule is pinned in `crates/sim/tests/feel.rs` as
`the_first_two_hits_leave_somebody_standing_where_the_third_can_reach_them`, and it is the
thing most likely to be broken by an innocent-looking retune.

Live numbers: `cargo run -p sim --bin frametable`.

## Hitboxes

Every other class in the game swings a **disc at arm's length**: a flat circle at a fixed
distance in front, live for the whole active window, with no top and no bottom. The
Champion's are **capsules** — a line from the hand to the head of the weapon, thickened by
the move's radius, which moves over the active frames the way the weapon does.

That is the change that makes the three weapons different to *play* rather than different
to read:

- **The sword sweeps across the front**, low, from right to left. It owns width. A fighter
  standing inside the arc is caught by the haft, and one standing beyond the tip is not
  caught at all.
- **The hammer sweeps down**, from over the head to floor level, in the vertical plane you
  are aiming along. It owns the line underneath it — including things low enough that a
  sword passes over them.
- **The spear is a line**, extending along the aim over its active frames. It owns
  distance, and it is thin enough to miss with.

**The shape belongs to the weapon rather than to the move.** Every sword move in the class
cuts across, every hammer move travels up or down the vertical plane you are aiming along,
every spear move is a line — through all three links of the chain, mid-Rush, and on the way
off the floor. So a player who has learnt that the hammer owns the ground under it has
learnt something true of every hammer move there is, which is what makes nineteen moves
learnable. The two exceptions are both the air and both the same exception: off the ground
there is no floor to cut across, so the sword rolls its arc into the vertical and the spear
sweeps its fan flat around the aim.

Because a capsule has a top and a bottom, **height is now real** for this class: an attack
thrown from the air can miss somebody standing under it, and one aimed at the floor can
reach somebody below you. That is what makes the air game below possible at all.

## Rush — `E`

A dash on a single stored charge. Six metres in twenty frames, in whatever direction you
are holding — not where you are looking, so it retreats and sidesteps as well as it
approaches.

Two things it does:

**It is the universal cancel.** Spent from a recovery it ends the recovery. This is the
class's "break my own pattern" tool, and it is what turns two moves into a combo: the
hammer's twenty-two frames of recovery are the price of its stagger, and Rush is how you
decline to pay it, once.

**It changes what the three buttons do.** While the dash runs, the mouse buttons throw the
Rush moves rather than the standing ones, and rushing beats every other row — a Rush move
is the one you spent a charge to reach, and a jump or a set of feet in the air should not
take it away from you.

Getting hit ends the dash. One charge, on a recharge, so spending it is always a decision.

**A Rush does not end a string.** The grace window keeps running while you dash, so one
charge buys a reposition in the middle of a chain: open, cancel the recovery, cross four
metres, and finish from somewhere they were not expecting the third hit to come from.
Throwing a *Rush move* does end it, because that is a different weapon in a different
situation.

### Rush slash — left click

Does not stop the dash. One long active window that re-arms every few frames and
alternates direction, so you cut two or three times as you run past. The samurai walks
through a line of people and they fall over behind him.

This is the only move in the game that hits more than once, and it is the only one for
which "on hit" means nothing — the exchange is not over when it connects, it is still
swinging.

### Rush sweep — middle click

The hammer's answer to somebody you are running at, and it is the one that goes
**underneath**. The head comes down outside the lead foot, scrapes the floor across the
whole front, and comes back up behind the hip without the run ever stopping. It does not
slow the dash.

The Rush row is three answers to the same question, and they are three heights: the sword
cuts across at chest level, the spear stops and puts everything into the point, and this
one owns the floor. Somebody crouching under a run-through, or standing on a ridge along an
animal's back, is what it is for.

It took this slot from the uppercut on 2026-09-14, which moved onto the hammer's **takeoff**
— see [The takeoffs](#the-takeoffs--a-weapon-on-the-way-off-the-floor). The reason is worth
keeping: a launcher behind a charge is a launcher you often do not have, and the loop the
whole class is built around started with a resource check. It starts with a button now, and
the charge is free for the reposition it was always better at.

### Rush stab — right click, level

The one Rush move that stops. All of the dash goes into the point: the longest reach and
the biggest single hit in the class, and the longest recovery to pay for it.

### Pole vault — right click, aimed at the floor

Not an attack. It has no hit volume at all. The spear goes into the ground, the run becomes
height, and you come off it higher than a jump reaches with most of your forward speed
intact.

It is a way *into* the air on a class whose best moves are up there.

## Aerials

The three buttons again, and the three moves are what the air game is made of.

**Air sword.** The same arc, rolled into the vertical — down across the opponent rather
than across the front of your own body. Fast, cheap, and it shoves you the way you are
holding, so it is part of air movement rather than a pause in it.

**Air hammer.** Twenty-two frames of startup, which is the slowest thing the class has and
demands a read rather than a reaction. Against somebody **already off the ground** it does
two things at once: a much larger knockback, because a body in the air has nothing to brace
against, and a **spike** straight down. Then the floor charges them for the landing —
damage proportional to the speed they arrive at, and a stagger while they get up.

**Air spear.** A wide fan swept around wherever the mouse is pointing, left to right. Mid
range, small knockback, and **if it hits anything it kicks you the way you are holding.**
On contact rather than on the throw, which is the difference between a movement option and
a reward: you get the reposition for catching somebody.

## Playing it

The class is three loops, and they share a first beat.

**The string.** Three hits, and the decision is which weapon each one is:

> **Hammer** (fifteen frames of telegraph, the longest stagger in the kit) → **Skewer**,
> because they backed off and the spear is what reaches them there → **Crescent**, because
> they are moving sideways and nothing else catches somebody moving sideways.

That is the whole class in one sentence: you are choosing a *shape* per beat against where
they actually are, and the string is what gives you three beats to be right about.

**The air.** The string is how you get the first hit; the air is what the third one buys:

> **Hammer** (+7 on hit) → **jump + hammer** = **Uppercut**, which launches and carries
> them up → **space**, taking the exchange higher → **air hammer**, which spikes them into
> the floor → they land staggered, and you are on top of them.

Every step is a decision with a cost. The hammer is fifteen frames of telegraph. The
uppercut commits you to the air along with them, and it costs the jump. The air hammer is
twenty-two frames of startup they can see coming, and missing it leaves you falling with
nothing.

**Arriving first.** The reverse of the same idea: hit them *up*, then get somewhere else
fast, so they come down into an attack that is already waiting. That is what the pole
drive, the vault, the fan's on-hit shove and the Rush slash's reposition are for. You do
not chase them, you arrive first.

Linear play — pick the weapon for the range, hold the button, take the whole string — is
still meant to be strong, and it is: a held sword chain is 234 damage in eighty frames and
needs one button. Nonlinear play is the three loops above, and it is where the ceiling is.

## Still to build

- **The mid-animation form swap** — starting a move with one weapon and finishing it with
  another, for n² endings out of 2n animations. Still the class's headline idea in
  [../champion.md](../champion.md), and the chain has changed what it would be *for*:
  swapping between hits is already the class's texture, so a swap inside one hit now has to
  justify itself as a different thing rather than as the only way to mix weapons. The
  honest reading is that the chain took most of what the swap was reaching for, at a
  fraction of the animation cost, and what is left for the swap is the *bait* — a
  silhouette that says one thing and a tail that does another. Worth building for that, and
  no longer urgent.
- **The six-ability kit** — Drive, Sweep, Throw, Brace. Shift plus a click is unused on this
  class now.
- **Air-to-air reads.** Two Champions both airborne with hammers wound up is a game of
  chicken nobody has played yet.
- **Does the chain want a fourth hit for one weapon?** Three is symmetrical and readable.
  A weapon with a fourth would be a real asymmetry, and asymmetry is where a class gets a
  favourite. Nobody has played three yet, so this is a note rather than a proposal.
