---
status: implemented
decided: 2026-09-10
rebuilt: 2026-09-12
formerly: Bellator, Shifter
sources: docs/archive/combat-design/shifter-skills.md
depends: ../champion.md
---

# Champion — kit

**Identity.** Range bands and flow, in three dimensions. Three weapons on three mouse
buttons, a dash that changes what all three of them do, and an air game that is the point
of the class rather than a place it visits.

## The grid

The whole input scheme, and it is worth reading as a table rather than as a list of ten
moves:

| | Left click | Middle click | Right click |
| --- | --- | --- | --- |
| **On foot** | Sword — arc across | Hammer — arc down | Spear — a line ahead |
| **In the air** | Air sword — the arc rolled vertical | Air hammer — wind up, then spike | Air spear — a fan around the aim |
| **Rushing** | Rush slash — cut as you go past | Uppercut — launch, and hold on | Rush stab / Pole vault |

**The button is the weapon and never changes meaning. The row is where your feet are.**
That is the entire thing a new player has to learn, and it is why ten moves fit on three
buttons with no modifier: you never choose a move, you choose a weapon, and the situation
chooses the move.

The tenth move shares right click with the ninth and is separated by where you are
pointing — a spear put into the ground vaults, a spear levelled at somebody stabs. That is
the only overloaded input in the class and it is the honest one: a pole vault *is* a spear
planted in the floor.

`Q` is unassigned. The class's identity is the three weapons and the dash, and there is
nothing left that wants a key.

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

| | Reach | Startup | Recovery | Damage | On hit | Character |
| --- | --- | --- | --- | --- | --- | --- |
| **Sword** | 2.0 m | 7f | 11f | most | +0 | The combo tool. Small knockback, small hitstun, and you can walk while you swing it. |
| **Hammer** | 1.6 m | 15f | 22f | least | +7 | The crowd-control tool and the combo *starter*. Big knockback, long stagger, roots you. |
| **Spear** | 3.4 m | 10f | 14f | middling | +0 | The spacing tool. Longest reach in the game, thin, and it goes over a crouching opponent. |

The hammer doing the least damage is deliberate and it is what "heavy" means here: it buys
stagger and knockback, not numbers. Its +7 on hit is the whole of its job — it is the move
that starts the exchange the sword finishes.

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
Rush moves rather than the standing ones, and rushing beats airborne — an uppercut that has
already left the floor is still a Rush move.

Getting hit ends the dash. One charge, on a recharge, so spending it is always a decision.

### Rush slash — left click

Does not stop the dash. One long active window that re-arms every few frames and
alternates direction, so you cut two or three times as you run past. The samurai walks
through a line of people and they fall over behind him.

This is the only move in the game that hits more than once, and it is the only one for
which "on hit" means nothing — the exchange is not over when it connects, it is still
swinging.

### Uppercut — middle click

**The combo hinge, and the reason the class has an air game.** It launches, and it *holds
on*: both fighters leave the ground together. It does not slow the dash at all — it is an
attack that starts while you are already travelling.

Then **press space and you both go higher, once.** That is the "we are settling this in the
air" button, and it is the only thing space does while airborne.

The move used to be the class special on `Q`, a slow committed launcher thrown from
standing. It did not feel good, and the reason is that it was a *commitment to the air*
rather than a *continuation*: sixteen frames of startup from a standstill, telegraphed, and
if it missed you were airborne and helpless. As a Rush move it is fast, it comes out of
movement you already committed to, and its cost is the charge rather than the frames.

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

The class is the loop, not the moves:

> **Hammer** (+7 on hit, big stagger) → **Rush** to cancel the recovery → **Uppercut**,
> which launches and carries them up → **space**, taking the exchange higher → **air
> hammer**, which spikes them into the floor → they land staggered, and you are on top of
> them.

Every step of that is a decision with a cost. The hammer is fifteen frames of telegraph.
Rush is one charge. The uppercut commits you to the air along with them. The air hammer is
twenty-two frames of startup that they can see coming, and missing it leaves you falling
with nothing.

The other half is the reverse: hit them *up*, then get somewhere else fast, so they come
down into an attack that is already waiting. That is what the vault, the fan's on-hit
shove, and the Rush slash's reposition are for. It is the anime pattern — you do not chase
them, you arrive first.

Linear play — pick the weapon for the range, throw it — is still meant to be strong. The
sword is the best plain auto in the game. Nonlinear play is the loop above, and it is where
the ceiling is.

## Still to build

- **The mid-animation form swap** — starting a move with one weapon and finishing it with
  another, for n² endings out of 2n animations. This is the class's headline idea in
  [../champion.md](../champion.md) and it is untouched: the swap needs a weapon *in* the
  move to change, and until this rebuild the weapon was a mode rather than a button. Now
  that the button is the weapon, the obvious shape for it is "press a different button
  during the active frames".
- **The six-ability kit** — Drive, Sweep, Throw, Brace. Shift plus a click is unused on this
  class now.
- **Air-to-air reads.** Two Champions both airborne with hammers wound up is a game of
  chicken nobody has played yet.
