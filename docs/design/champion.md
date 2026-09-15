---
status: decided
decided: 2026-09-10
renamed: 2026-09-11
rebuilt: 2026-09-12
chained: 2026-09-14
formerly: Bellator, Shifter
supersedes: docs/archive/combat-design/shifter-skills.md
---

# Champion

Formerly **Shifter**, which did double duty with the weapon-shift verb and read as a plain
word; then **Bellator**, Latin for a warrior or combatant, chosen because the class descends
from the old cities' professional duelling champions.

*Bellator* was accurate and out of place. Every other class on the roster has a plain English
name — Bulwark, Elementalist, Blood mage — and one Latin term among them reads as belonging to
a different game. **Champion** says the same thing in the register the rest of the roster is
written in, and the lore already used the word: these fighters *were* the aristocrats'
champions.

Bellator is kept in the front matter, and the roster's naming is now open across the board —
see [README](README.md#4--open).

## Identity

Range bands. The three weapon forms — hammer (short), sword (medium), spear (long) —
change the auto attack and carry signature moves. The Rush charge governs commitment.

The class should be **one of the strongest classes when played linearly**, with mechanics
that incentivise nonlinear play sometimes. The two sections below are how that gets built.

> **Rebuilt 2026-09-12.** The form is no longer a mode and Rush is no longer unimplemented.
> Each weapon is a **mouse button** with its own moves on the ground, in the air and out of
> a Rush — ten moves on three buttons — and the uppercut has moved from the class special
> onto Rush + hammer, where it is a combo hinge rather than a committed launcher. The
> diagnosis in [The core addition](#the-core-addition--mid-animation-form-swap) below was
> right about *why* the class was linear and the fix went one step further than the
> sentence it is written in: the toggle is gone rather than made mid-animation.

> **Chained 2026-09-14.** Ten became nineteen, and the shape of the class changed with it.
> The grounded row is now a **three-hit string**, and **every hit is a free choice of all
> three weapons** — sword into spear into hammer is an ordinary thing to do. Every weapon
> also gained a **takeoff**, thrown on the same press as jump, and the uppercut moved
> again: off Rush and onto the hammer's takeoff, where the launcher costs a jump rather
> than a charge. Rush + hammer became a low run-through that owns the floor. The whole
> thing is specified in
> [kits/champion.md](kits/champion.md#the-chain--rebuilt-2026-09-14), and what it did to
> the mid-animation swap below is the interesting part: it took most of what the swap was
> reaching for, at a fraction of the animation cost.

## What the mode toggle cost, stated properly

Worth keeping, because it generalises past this class.

A form multiplier **cannot make two moves feel different, because it does not change what
they do — it changes how much.** A spear with 1.55× reach is a sword that reaches further;
a hammer with 1.35× damage is a sword that hits harder. All three were the same swing, the
same shape, at the same height, and the choice between them was arithmetic. So playing the
class was arithmetic.

What actually separates weapons is **shape**: a sword goes corner to corner, a hammer goes
down, a spear goes out. Those are three different questions about where the other player is
standing, and none of them is a number you can multiply. Giving each weapon its own moves
with their own hit volumes is what the nine multipliers were reaching for and could not
reach.

**A second thing separates them, found 2026-09-15, and it is the same argument one step
further on.** Shape answers *where can I reach*; it does not answer *where will I be
afterwards*, and that turns out to be half of what a weapon is. A move may now carry the
body forward on its own — `Move::step` — so the sword closes half a metre on every link
whether it lands or not, the spear's opener leaves you exactly where you were, and its
second hit crosses nearly two metres in one beat behind a telegraph you are meant to see.
Three weapons, three answers to "what does throwing this commit my feet to", and none of
those is a multiplier either.

## The core addition — mid-animation form swap

> **Superseded in the direction it was pointing, 2026-09-14.** The idea below is still
> sound and still unbuilt, but the thing it was *for* — making the choice of weapon a
> decision you take more than once per exchange — is now what the three-hit chain does, and
> the chain does it with nine ordinary animations rather than with nine blended tails. Read
> this section as the argument that got the class here rather than as the next thing to
> build. What is left for the swap afterwards is the **bait**: a silhouette that says one
> weapon and a tail that does another, which the chain cannot do because each of its hits
> is honest about itself from the first frame.

Form swapping is currently a mode toggle: free, instant, and outside the flow of a move.
That is exactly why the class is linear — pick the form for the range, press the button.

**Let the form change during a move's active frames, changing the move's tail.**

- Start a spear thrust — long reach, thin hitbox — swap to hammer mid-thrust, and it ends
  in a downward slam.
- Start a sword slash, swap to spear, and it extends into a lunge.

Why this is the highest-leverage change available to the class:

- **n forms × n forms = n² move endings from 2n animations.** Three forms gives nine
  distinct enders from six animations. That is enormous depth per unit of art, which is
  the project's binding constraint.
- **It resolves the linear/nonlinear tension directly.** Linear play is "right form,
  right range, press the button" — and that can be tuned to be genuinely strong, as
  intended. Nonlinear play is "commit in one form, cash out in another", which is strictly
  harder and riskier because the wrong tail whiffs.
- **It is the right kind of hard.** A decision made inside a committed animation. That is
  Smash, not Tekken — no input strings, just a read and a timing.

## Rush as a universal cancel

Rush is a chargeable dash with a single stored charge. Extend it: **Rush can be spent to
cancel out of any move's recovery frames.**

This is the class's "break my own pattern" tool — the wavedash / air-dodge role. One
charge means spending it is a real decision, and it is what lets a linear class stop being
linear when it needs to, at a cost.

## Resulting skill ceiling

Three questions per exchange:

1. Which form do I start in?
2. Which form do I end in?
3. Do I spend Rush to escape the tail?

Deep without being wide. The class stays legible to a new player — pick the right range
band — while having a real ceiling.

**Restated after the chain, 2026-09-14.** The three questions are still the right three,
and two of them are now asked by a mechanism that exists. "Which form do I end in" is the
third hit of a string rather than the tail of one move, and it is asked twice rather than
once:

1. Which weapon opens, which is a question about range.
2. Which weapon connects, which is a question about where they *went*.
3. Which weapon finishes, which is a question about what you want the exchange to end as —
   a body knocked across the arena, a body launched, or a body on the floor with a broken
   guard.
4. Do I spend Rush to go somewhere else in the middle of it?

Four rather than three, and the extra one is free: it needs no input the player did not
already have.

## Carried forward from the old notes

- Rush as a chargeable dash, one charge, shift-click to charge. **Built**, on `E`, as a
  dash that also cancels any recovery.
- Per-form auto attacks: sword medium melee strike, hammer short blow (staggers while
  rushing), spear long thrust (vault off the ground while moving or rushing). **Built**,
  and the two parenthetical notes turned out to be the good half of the idea: the spear
  planted while rushing is the vault, and the hammer thrown out of a Rush is the low
  run-through that owns the floor.
- Per-form signature moves: Uppercut (sword), Hammerfall (hammer), Spear toss (spear).
  Uppercut is built, on the hammer rather than the sword — a launcher is a heavy weapon's
  move — and since 2026-09-14 it is the hammer's **takeoff**, thrown on the same press as
  jump. Hammerfall arrived under another name: **Earthbreaker**, the hammer's chain
  finisher, which is the whole body over the head and through a shield. Spear toss is still
  open, and is now the only thing on this list that is.
- The reforge idea for combining weapons at the cost of an accessory slot — "quicksilver
  ___" as the reforged weapon title.

## The air is the class — added 2026-09-12

The rebuild settled something the original notes left implicit. **Verticality is where this
kit's depth lives**, and it is a better home for it than the swap matrix because it needs no
extra inputs at all: the same three buttons mean three different moves off the ground.

The loop is: hit them up, get somewhere else fast, and have an attack already waiting when
they arrive. Hammer to stagger, jump-plus-hammer to take them both up, space to take it
higher, air hammer to put them back into the floor. Every step costs something real —
frames, the jump, or committing to the air with them.

**Amended 2026-09-14.** The launcher used to need the Rush charge and now needs the jump,
which is the better price for two reasons. A loop whose first step is a resource check is a
loop you often cannot start; and having three *different* ways off the floor — one per
weapon — makes leaving the ground a choice rather than a step. The Rush charge is free for
the reposition, which is the half of this the class was always better at.

It is the anime pattern, and the reason it works as *play* rather than as spectacle is that
the reposition is the hard part. Launching somebody is easy. Being where they land is not.

## Open questions

- Is the mid-animation swap free, or does it cost Rush? Free is more expressive; costed
  makes it compete with the cancel and forces a choice. Leaning free, with the risk being
  the whiff itself. **Now cheaper to answer**: the weapon is a button, so the swap is
  "press a different button during the active frames" rather than a mode change. **And
  less urgent, 2026-09-14** — the chain already lets you change weapon between hits, so
  what is left for the swap is the bait rather than the mixing.
- Three forms or more? Four gives sixteen enders but the readability cost is steep — and
  it is much steeper now: each weapon carries six moves, and a fourth column would be a
  twenty-fifth move on a class that already asks a player to hold nineteen.
- **Does the chain want a fourth hit for one weapon?** Three is symmetrical and readable.
  A weapon with a fourth would be a real asymmetry, and asymmetry is where a class gets a
  favourite. Nobody has played three yet.
- Does the swap have a window, or can it happen any time during active frames? A window
  is a cleaner mechanical test.
- **Does `Q` want something?** The class special is unassigned: the three weapons are the
  three clicks and Rush is the mechanic. A dead key is not a problem, but it is a free slot
  and the swap matrix may want it.
