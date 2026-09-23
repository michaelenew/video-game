---
status: built 2026-09-23, unplayed — every milestone's measured criteria pass (see the feel log); the felt questions are open
decided: 2026-09-23
supersedes: the mechanic section of kits/bulwark.md and the "shield as volume" promise in bulwark.md, once built
sources: bulwark.md, kits/bulwark.md, defense.md, docs/archive/combat-design/blood-mage-skills.md (Seal of the unforgiven), this design thread
---

# Bulwark — v2: the shield is a battery

**Identity.** The wall gives back what it took. Every blow that lands on the shield is
stored in it as weight, and the shield's own attacks — the slam, the throw, the wall it
becomes when planted — spend that weight. A Bulwark who has blocked three hits is holding
three hits, and the opponent can see them.

This is the answer to the question the class could not answer: **why do I use a shield when
I am alone?** Because blocking is not the absence of being hit. It is loading the weapon.

**The action plan for building this is [plans/bulwark-v2.md](plans/bulwark-v2.md).**

## What is wrong today

The class was designed as the foil to the Champion and as cover for a party, and the game
as it stands is one player against one, or one player against the creature. In both, a block
produces not-being-hit, which a dodge produces for less commitment and no facing arc. The
parry produces one stagger from a four-frame read. Against the creature the shield produces
nothing at all — the guard test is there, its blows go through it, and nothing comes of a
successful block but the pushback.

The planted shield, which [bulwark.md](bulwark.md) calls the structural trick that makes the
class a class, stops nothing. So the identity is "the wall", and there is no wall.

## The mechanic

### Weight

**Every hit taken on the shield is stored as weight.** A blocked hit stores its damage; a
parried hit stores more. Weight has a cap, and it decays slowly on its own so that it is
about the current exchange rather than the whole round.

Weight is **drawn on the shield**: it thickens, it darkens, it drags. The opponent can read
how loaded the wall is from across the arena, which is what makes a heavy Bulwark something
to respect and a light one something to press.

### Spending it

Three things spend weight, and they are the three things the shield already does.

| Spend | What weight does to it |
| --- | --- |
| **Slam** — middle click | The overhead's damage and the radius of its ground shake grow with weight, on top of the fall-speed scaling already sketched. At full weight the shake is an area stagger. Spends everything |
| **Throw** — `E` from hand | A loaded shield flies slower, hits harder, and **knocks down** what it hits. Spends everything on impact. Empty, it is the poke it is today |
| **Planted** — where a throw lands | The shield is a **structure**: solid, it stops bodies and shots, in the same system the Elementalist's stones already use. Its size is the weight it carried. Recalling it is the ordinary recall |

And one thing weight does without being spent: **a heavy shield resists pushback.** The
trait [defense.md](defense.md) promised the class and never built, made a function of the
thing the class is doing. The more it has taken, the less it moves.

### The loop, alone

- **Versus.** Block, and the shield grows. The opponent now has to choose between attacking
  a wall that is loading and standing off a class with no ranged damage. Throw the loaded
  shield and it is a boulder that plants as a wall; leap to it; slam out of the leap with
  everything you took. Grapple when they stop attacking and start blocking. That is a loop a
  single player runs, and every step of it is legible to the person on the other side.
- **The hunt.** The creature's blows are the heaviest in the game and they go through the
  guard test today. One blocked stomp loads the shield further than any fighter's string
  could. Slam that into a leg and it is the poise break; plant it and it is cover the whole
  party can use, sized by what the creature itself put into it. The shield finally does
  something in the mode where it did nothing.

### Health

**Highest on the roster**, in the per-class health table the Reaver proposal introduces.
Committed, not slow: normal movement, long commitments, and now a reason to stand in them.

## Inputs

| Input | Move |
| --- | --- |
| `L` | Bash, unchanged |
| `M` | **Slam** — the stranded overhead, given the free third click |
| `R` held | Guard, and the parry in its opening frames, unchanged. Both load the shield |
| `Q` | Grapple, unchanged |
| `E` | Throw, recall, leap, by state — unchanged, and the throw now spends weight |

No new button. The one input that changes is Slam gaining one, which has been open since
shift stopped modifying clicks.

## What stays

Bash, Guard, Grapple, the three shield states and the leap. The facing arc. Pushback rather
than chip. The parry as the opening frames of block. All of [defense.md](defense.md) as
built.

## Why this shape

**Blocking produces something.** The whole reason the archetype fails alone is that its
identity verb is passive. Weight makes every block a deposit and every committed move a
withdrawal, so the passive verb is the first half of an active one.

**The wall becomes real by joining a system that exists.** Structures already stop bodies
and shots, the docs already say the shield should share that implementation, and a planted
shield sized by weight is one more source for the same solid query. The class's central
promise stops being fiction without a new system.

**It gives the stagger system a class.** [bulwark.md](bulwark.md) says the stagger is the
most fighting-game-like mechanic in the notes and nobody is about it. A full-weight slam is
an earned area stagger, a loaded throw is an earned knockdown, and both are earned by taking
hits on purpose — which is a thing only this class does.

**It is the same trick as the Blood mage's, mirrored.** She converts what she spends into a
thing on the floor she must go to. He converts what he *takes* into a thing in his hand he
gets to give back. Two classes about absorbing cost, one paying forward and one paying
back.

## What it costs to build

- **`Shield` gains a weight** on every variant — held, flying, planted — so it travels with
  the object. One integer.
- **Loading** is two lines in `apply_hit` where `blocked` and `parried` are already decided:
  add the damage, or the parry multiple of it, capped. **Decay** is one line in
  `step_mechanic`.
- **Slam** is bound to middle click in `clicked_move` for the Bulwark and reads weight for its
  damage and its shake radius; the shake is the flat overhead's volume with a radius knob.
- **Throw** reads weight for speed and damage, and its impact applies a knockdown — a launch
  with no height and a floor stagger, the shape the Champion's spike already uses.
- **The planted shield is a structure.** The solid query that walks every player's stones
  also walks a planted shield, sized by weight. The Elementalist's cap does not apply; there
  is one shield.
- **The health table**, shared with the Reaver proposal.
- **Drawing**: the shield mesh scales and darkens with weight, and the planted wall is drawn
  at the size it is tested at, per the overlay rule.

**Tests that change.** `combat.rs` gains: *a blocked hit loads the shield and a parried hit
loads it more*; *weight decays*; *a slam at weight shakes wider and hits harder and spends
it*; *a loaded throw knocks down*; *a planted shield stops a body and a shot, at a size that
follows its weight*; *a heavy shield is pushed back less*; *a blocked creature blow loads the
shield*. `feel.rs`: `hindrance_is_proportional_to_commitment` should hold with weight as the
commitment, and `every_class_can_beat_a_turtle` still passes on Grapple.

## Open questions

- **Does weight decay at all, or only spend?** Decay keeps it about the current exchange and
  stops a first-minute block paying out at the end. No decay makes the shield a battery in
  the fullest sense. Written as slow decay; a knob.
- **Should the mechanic press cost frames?** Still open from the kit. A throw that spends
  weight is a bigger event than a throw that does not, and may want the wind-up the Reaver's
  send has.
- **Can the planted wall be destroyed?** A stone can, by Cataclysm. A wall the opponent can
  break spends the Bulwark's weight for him, which is a real counterplay and a real loss.
  Written as bypass-only; revisit once anybody else can move a stone.
- **The parry multiple.** Parry already pays a stagger. If it also loads double, the four-
  frame read may be worth too much; if it loads the same as a block, the read is only the
  stagger. Play it.
- **Weight against the creature.** One stomp may fill the cap on its own, which makes the
  hunt loop "block once, slam once". If that is too easy, the creature's blows load at a
  fraction; if it is the fantasy, leave it.
- **Whether this is enough to keep the shield.** The fallback discussed in the thread is a
  grappler on a chain or a hook — command grab, leap, funnelling, no block layer. This
  document is the case for keeping the shield; if the loop above does not feel like a reason
  to block alone once it is played, that is the next document.
