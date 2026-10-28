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

`bred()` in `tests/strides.rs` breeds them as the game does: forty families (a hundred now) of
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

## The last 3%

The owner: *"fix the remaining 3% too."* Fixing one family at a time did
not converge -- each round turned up rarer ones elsewhere -- so the rest
were fixed as rules, on 6,000 bodies (families 0 to 999), and held out
against 6,000 none of the rules saw.

- **A move it could make, or none** (`strike::makeable`): every move is put
  through the same physical checks after it is settled -- nothing but what
  strikes, the feet and the tail into the floor, the weight over the feet
  through a held windup -- and a body is not given one it cannot make. The
  same rule already gave no kick back to a body with no other legs. Wings
  bear a windup only on a body that can fly. The whole-body moves (a leap, a
  rush, a stoop) are one list in the simulation (`strike::whole_body`) that
  the test reads.
- **A head strikes with its jaw** (`strike::struck_by`): counted as a body
  part, a jaw under the floor at the strike lifted the whole body metres.
- **A rear that drags is stood on**: a seal-like body on one pair of legs
  and its rump balances over both.
- **Nothing on the floor, finally** (`Solver::clear_the_floor`): stood taller,
  then tipped to lift the low end, then -- a trunk its legs cannot hold clear
  -- rested on its belly; a head that is the body's front is lifted as the
  trunk is. A belly barely clear of the floor does not bob onto it walking.
- **A body on its belly** carries its legs up its flank, knees out to the
  side as a lizard's, a crowded middle pair of paddles held up; its feet
  reckoned from the hips it really has.
- **A pair that holds the body down is held up** -- an end pair too, where
  two are left to stand on -- and **the weight is put back over the pairs
  left standing** once any is held up: a biped whose forelegs came up stood
  with its weight ahead of its feet.
- **Legs out of each other's way**: hips kept apart across an empty
  station (a fore and a mid-hind pair grew from one spot); a knee leads
  forward only by the floor's measure, not the body's (an upright body
  stacks its pairs); the way a knee bends is the standing pose's, kept
  through the stride (decided frame by frame it flipped forward mid-step);
  a middle knee with no room either way turns out as an insect's
  (`build::knee_out`) and its foot sprawls; a swinging foot comes no higher
  than two thirds of its hip (lifted past a low hip, the leg turned inside
  out); a back kick throws the weight forward before the hind feet leave.
- **A middle walking pair is no longer than the longer of its neighbours**
  (`plan::repair`, `Fix::MiddleOutgrown`): a longer one between two that
  stand the body crouched deep and stepped round them. A spider's legs fan
  instead and are let be.
- **Three checks were wrong**: feet a leg's thickness apart on the floor do
  not meet whatever their order; a walked-on wing is as thick as it is
  drawn; and a biped is what a body stands on, not what its genome calls
  legs.

## How far it holds

`bred_bodies_are_plausible` breeds a hundred families (600 bodies) and
holds all of them. Bred as the game breeds, families 0 to 1999 -- 12,000
bodies, half of which no rule was written against -- all pass, and 12% of
their damaging moves cry. To measure it again on bodies nobody has seen:

    FAMILY_FROM=2000 FAMILIES=500 cargo test -p sim --test strides bred_bodies
