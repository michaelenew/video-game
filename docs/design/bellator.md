---
status: decided
decided: 2026-09-10
formerly: Shifter
supersedes: docs/combat-design/shifter-skills.md
---

# Bellator

Formerly **Shifter**, which did double duty with the weapon-shift verb and read as a plain
word. *Bellator* is Latin for a warrior or combatant, which suits a class descended from the
old cities' professional duelling champions.

## Identity

Range bands. The three weapon forms — hammer (short), sword (medium), spear (long) —
change the auto attack and carry signature moves. The Rush charge governs commitment.

The class should be **one of the strongest classes when played linearly**, with mechanics
that incentivise nonlinear play sometimes. The two sections below are how that gets built.

## The core addition — mid-animation form swap

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

## Carried forward from the old notes

- Rush as a chargeable dash, one charge, shift-click to charge.
- Per-form auto attacks: sword medium melee strike, hammer short blow (staggers while
  rushing), spear long thrust (vault off the ground while moving or rushing).
- Per-form signature moves: Uppercut (sword), Hammerfall (hammer), Spear toss (spear).
- The reforge idea for combining weapons at the cost of an accessory slot — "quicksilver
  ___" as the reforged weapon title.

## Open questions

- Is the mid-animation swap free, or does it cost Rush? Free is more expressive; costed
  makes it compete with the cancel and forces a choice. Leaning free, with the risk being
  the whiff itself.
- Three forms or more? Four gives sixteen enders but the readability cost is steep.
- Does the swap have a window, or can it happen any time during active frames? A window
  is a cleaner mechanical test.
