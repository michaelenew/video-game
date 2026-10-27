# 0034 — Bred as the game breeds

*2026-10-27.* The owner, after 0033: *"Good enough. Now it's time to go back
and fix the crossbreeding and evolution algorithms"* -- so that what breeding
makes is as plausible as the four lines 0033 walked.

## The lines were four walks somebody chose

0033's checks (`tests/strides.rs`) held the cast's genomes and four lines of
feeding: chosen, and fixed until they passed. A player's eggs are whatever
the dice give: a cross with any of the cast (`cross::breed`), a trophy
grafted on (`cross::graft`), a generation's drift (`cross::mutate`), in any
order.

`bred()` in `tests/strides.rs` breeds them as the game does: forty families of
six generations, each from a parent of the cast, each generation one of the
three with a mate of the cast. `bred_bodies_are_plausible` puts every one
through all of 0033's checks -- it moves, it is carried like an animal, its
attacks are ones its body could make and are read off it -- and asks that no
more than a fifth of their damaging moves are told by a cry.

When it was written, **53 of the 240 failed** (22%). Most were low-slung:
none of the four lines has a low-slung body, and the cast's toad does. The
check also had a hole: a held windup that lifted every foot off the floor
was not counted, and the cast's own Ridgeback's bite did that (its feet a
metre up). All 240 pass now.

## What was wrong, and what changed

**Standing.**

- *A trunk's ends in the floor.* The height kept the belly off the floor at
  the middle only; tipped, a toad's head-front or a haunch went into it.
  The height keeps both ends clear, and a thorax by its own depth
  (`Solver::trunk_low`).
- *Legs too short to hold it up at all*: the body rests on its belly, a
  newt's carriage, on its deepest part -- a leg that cannot reach the floor
  from there held up as a paddle (`Solver::trunk_held`).
- *A grazer's head through the floor*: the head is held a tenth of the
  height clear (`Solver::head`).
- *Nose down past a hyena's back* on short forelegs: the level pitch limit
  holds both ways.
- *A sprawling leg thicker than its hip is high* lay along the floor: the
  hip goes up the flank, no further than the leg still reaches.
- *A spider's legs*: the middle pairs fan twice as wide, the gait and the
  balance reckon a spider's foot where it lands (`build::spider_ahead`),
  and a spider's leg is not an obstacle the legs behind step round.
- And the reckoning checked (`Solver::clear_the_floor`): the standing body
  built and measured, and anything but a foot or a shin under a tenth of
  its height lifted -- a head by its neck, the rest by standing taller.

**Attacking.**

- *Lifted off its feet.* A move that put anything into the floor was lifted
  out whole -- windup and strike together, again each time it re-aimed --
  with no regard for whether the legs reached. The windup is lifted only for
  what goes under in the windup, and no higher than its planted legs reach
  (`strike::leg_room`, measured off the built feet); what is left comes out
  of the windup itself, made a quarter smaller at a time. A windup on spread
  wings, or one that leaves the floor anyway, is borne by more than its legs.
- *A swelling body* grows up from its belly, as a toad puffs up, not down
  through the floor.
- *Aimed by the middle of the head*: a big body's rush drove its jaw three
  metres under the floor to put its head's middle at a man's height. A head
  comes down no lower than its own depth allows; a head that is the front of
  the body reaches its mark with its lower jaw; and nothing sinks the body
  past a deep crouch on its legs (`strike::sink_room`).
- *A spider stabbing with both legs of its only front pair* stood on its
  hind pair with its weight three metres ahead: on two pairs it stabs with
  one leg and stands on three.
- *The weight over the feet through the windup's middle*, not only at its
  end: a windup stood on is held a tenth of the torso in from the feet's
  edge.

**Read.**

- *A big body with a small head*: the trunk moved a sixteenth of the body,
  as asked, but nothing in the silhouette moved an eighth, and its bite was
  read by nobody. The whole body gathers until both hold (`strike::gather`).
- *A cry is heard from the windup's first frame*: a move that cries is read,
  whatever its silhouette does -- the owner's edge case. A move cries when,
  measured as the verdict measures it, it would otherwise be unanswerable;
  one answered by where you stand needs none. 28 of the cast's and lines'
  379 damaging moves cry (36 before), and 138 of the bred bodies' 1353.

**One check was wrong.** "A fore foot ahead of the next pair's" was asked of
feet a leg's thickness apart sideways, which pass without touching; it is
asked of feet in line, and segments through each other are still caught.

## How far it holds

The 240 are the bodies the changes were made against. The same breeding,
families 160 to 399 -- bodies none of the changes saw -- fails 43 of 1440
(3%): legs crossing on upright bodies, a lizard's haunch on the floor, a
tongue or a swallow sunk past what short legs allow. To measure it again:

    FAMILY_FROM=160 FAMILIES=240 cargo test -p sim --test strides bred_bodies

`FAMILIES` alone grows the set from the start. The tail is the next work: it
is rarer chimeras of the same faults, and each fix above was made general
rather than for its family, which is why the unseen rate fell from about one
in five to one in thirty.
