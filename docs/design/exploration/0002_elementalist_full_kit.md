---
status: exploration — a proposal to prune, not a decision
started: 2026-09-29
builds on: 0001_control_budget.md, ../kits/elementalist.md, ../../archive/combat-design/elementalist-skills.md
---

# 0002 — The Elementalist, filled out

*Nine buttons, two rows, and a rule for what the air is. More than the kit needs, on purpose.*

## Why she feels good, said precisely

She plays best of the six, and the reason is not any one spell. It is that her kit is a small
**algebra**: a few objects, a few verbs, and every verb has an answer for every object.

- **Objects:** a stone (solid, standable, movable, erupts once), a fire pillar (a hazard that
  outlives the cast), a body, and hers.
- **Verbs:** *place* (`E` raises earth, `Q` plants fire), *kick* (the beam sends a stone along
  the line, lights a pillar), *break and transform* (Cataclysm shatters a stone, tears a pillar
  loose as a tornado), and *ride* (the structure jump).

Freedom comes from the verbs being typed by **what they meet**, not by which button was
pressed. The beam does not know it is a stone-kicker; it asks what is on the line and the
stone answers. So the number of things she can do grows as verbs × objects, while the number
of things a player has to learn grows only as verbs. Nine buttons each carrying one verb is a
small kit that plays like a large one, and that is the property to protect as she grows.

The aerials break it. Air bolt and Gale are projectiles with a flight time, and a projectile
is a verb with one answer: *hit*. They compose with nothing she built.

## The two rules this proposal adds

**1. Every button is a verb that reads what it meets.** A new move is written as its
dispatch table — stone, lifted stone, pillar, body, nothing — the way the beam already is in
the kit document. A move with one row in that table is a projectile and should be argued for.

**2. The air row is the ground row with the vertical axis added or flipped.** Standing, her
verbs act *along the floor*: the beam kicks a stone along the line, Fissure races through the
ground, Raise comes up. Airborne, the same button acts *along the vertical*: the bolt lofts,
Downdraft is Lift with its sign flipped, Landfall comes down. Raise and Landfall already obey
this; nobody had said it. It makes the air row cheap to design (often one signed number) and
cheap to learn (the same verb, turned upright), and it is what "now you are doing air magic"
means mechanically: **off the floor, height is the thing she controls.**

The second rule is testable: every air-row move either changes someone's height, changes a
stone's height, or changes hers. A feel test can pin that.

## The grid

Nine buttons: the three clicks, the two mouse side buttons (`M4`, `M5`), `Q`, `E`, `F`, `R`.
Exactly four bits are free in the input word, and exactly four inputs are new.

|  | Ground | Air |
| --- | --- | --- |
| `L` | **Bolt** — the beam · kicks a stone along the line, lights a pillar | **Air bolt** — *now lofts a stone it meets, and a bolt through a pillar leaves it as fire* |
| `R` | **Cataclysm** — breaks a stone into debris, tears a pillar into a tornado | **Gale** — *now lifts what it passes over; through a pillar it becomes a firestorm* |
| `M` | **Fissure** — the stranded committed skillshot | **Fissure from above** — the crack starts where the shot lands and runs onward |
| `M4` | **Lift** — a column of rising air at the crosshair | **Downdraft** — the same column, sign flipped |
| `M5` | **Grip** — take hold of a stone, carry it on the crosshair, throw it on release | the same verb; falling while she holds it is what changes |
| `Q` | **Fire pillar** — tall and narrow · *on a stone, arms it* | **Cinder rain** — wide and shallow: embers falling in a cone |
| `E` | **Raise** — earth up | **Landfall** — earth down |
| `F` | **Gather** — her stones lift and orbit her; again to drop them | the orbit rises with her |
| `R` | **Tremor** — Quake at her own feet | **Hover** — a held hang |

Bold and unstarred are built. Italic is a change to a built move. The rest is new. Seventeen
distinct moves on nine buttons, and a player learns nine verbs.

## The verbs, one at a time

Each entry gives the dispatch table, what it composes with, what it costs, and how sure I am.
Costs are in the currencies the kernel has: frames, the cap of three, and commitment.

### `M` — Fissure, and Fissure from above

The committed ground skillshot has had no button since 2026-09-16, and the Bulwark's Slam
went to the third click for the same reason this should: it is the committed move and the
third click is the committed click. It is aimed, so it belongs on the mouse.

| Meets | Does |
| --- | --- |
| a body | stops there, staggers, leaves a stone at the point of impact (kit) |
| a stone | runs *under* it and heaves it — the pop a stone already gets when one is raised beneath it. Free, from `stones` |
| a pillar | carries the fire: the slow field along the crack burns as well |
| nothing | runs to full reach and leaves the slow field |

**From above:** the same crack, started where the shot lands rather than at her feet, running
onward along the ground projection of the aim. Rule 2 says the air version adds the vertical,
and here the vertical is where the crack *begins*. Coming down on a stone from above sinks it
a step rather than heaving it. Confidence high on the ground, medium in the air.

### `M4` — Lift, and Downdraft

A column of air at the point the crosshair picks (grounded path), telegraphed the way a stone's
rise is: the air visibly gathers, then goes. The cheapest new move in the kit, because the air
version is one signed number.

| Meets | Lift (ground) | Downdraft (air) |
| --- | --- | --- |
| a stone | lofts it: it rises, hangs a few frames, falls. Lofted, it is a stone with height — the beam kicks it *down on to* somebody, Cataclysm shatters it from above, and she can stand on it | slams it down: a lofted stone becomes a meteor where it is; a grounded one sinks and leaves the slow field |
| a body | lifts them, **by their class's gravity** — the Bulwark barely leaves the floor, the Dual mage goes up a storey. Weight becomes legible again, which the movement document already prizes | spikes them, the Champion's air hammer as a spell |
| a pillar | stretches it: taller, narrower, longer-lived | flattens it into a **ring of fire** along the floor — the archive's Ring of fire, arrived at by a verb rather than a slot |
| herself | *proposed no.* A free lift under her own feet is a second jump that undercuts the structure jump, which is her movement. Excluding herself is one clause |

This is the verb the aerials were missing: a lofted stone is earth *in the air*, and every
ground verb then applies to it up there. It also makes the double structure jump a family
rather than a trick — raise, jump, lift the stone to catch her again. Confidence high; the
juggling risk (see below) is the thing to watch.

### `M5` — Grip

Press on a stone and she has hold of it. While held it hovers and follows the crosshair at a
fixed distance; release throws it along the line. **Hold is a dimension, not a second move**
(0001): the press does the whole thing, the hold moves a position, the release ends it, which
is exactly the Grasp's shape. Aim locks on release as everywhere else.

| Meets | Does |
| --- | --- |
| a stone | carries it. She is committed while holding — crawl, no jump, no dodge — so a floating stone and a slowed mage is the telegraph |
| a lofted stone | the same, and it is already at height |
| a body | nothing. Grabbing bodies is the Blood mage's Grasp and should stay hers |
| a pillar | nothing, deliberately. A carried tornado is a different class |

Composition is the point: the beam kicks a held stone early (a fast throw), Cataclysm
shatters it wherever it hovers (the shotgun, from over their head), Downdraft drops it as a
meteor, and a stone pulled under her own falling feet is footing. A hit on her drops it where
it is, and a falling stone hits by relative speed, which the stones already compute. It is the
one verb here with no air *variant*: the same move, and the situation supplies the difference.
Confidence medium-high. The readability question is whether a stone that follows the mouse is
too fluid for third person; a fixed carry distance and a visible tether are the answer if so.

**Lift and Grip overlap** — both put a stone in the air. If one goes, keep Lift for legibility
(a telegraphed column) or Grip for expression (clay in the hand). They are on the two thumb
buttons because both are verbs on her objects and both are aimed.

### `Q` — Fire pillar on a stone, and Cinder rain

Built, and one dispatch row short. **A pillar cast on a stone arms it**: the stone burns, and
the next thing that moves it hard — the beam's kick, a throw, a fall — makes it burst on
impact, burning debris in place of plain. The archive had this ("the next time the structure
is hit or violently displaced it will explode") and it is the earth-plus-fire composition the
loadout keeps asking for. One row, no new button.

**Cinder rain, airborne:** the pillar's opposite shape. Where a pillar is tall, narrow and
stands a long time, embers fall in a cone under the aim and leave a wide, shallow, short-lived
burn. Area denial from above at the cost of the pillar's column, which is the part that stops
a jump. Meets a stone: lights it (arms it). Meets a Gale below her: the firestorm. Confidence
medium; it is the least verb-like thing here.

### `E` — as built

Raise up, Landfall down. This pair is where rule 2 came from.

### `L` and `R` — the built aerials, given tables

The Air bolt stays the long poke and gains one row: **a stone it meets is lofted** — the
vertical version of the beam's kick, by rule 2 — and a bolt through a pillar comes out as
fire, which is the row the kit already lists as open. The Gale gains two: **it lifts what it
passes over**, bodies by their gravity and stones outright, so a disc thrown low across the
floor is a wave that takes the field up with it; and **through a pillar it is a firestorm**, a
burning disc. Both are the open questions in the kit document answered the same way: yes, and
upward.

### `F` — Gather

Unaimed, so it lives on the left hand. All her stones lift and orbit her at chest height.
Press again and they drop where they are, hitting by relative speed. While orbiting they block
shots and bodies as they do now, so it is cover that walks with her; the beam kicks one out of
the orbit along the line, Cataclysm shatters one at point blank, Lift is meaningless on them.

It is a mode: **terrain author or armoured**, and the cost is every stone leaving the field at
once — nothing to jump off, nothing to hide behind, the cap spent on herself. Airborne, the
orbit rises with her and a stone under her feet is footing, which is a hover she paid three
slots for. Confidence medium. Three orbiting stones read fine; the question is whether the
mode is a decision or a panic button.

**Sink** is the smaller alternative for `F`: her oldest stone goes back into the floor, no
frames, leaving the slow field. It frees the cap on purpose and drops whoever was standing on
it. Cheap, honest, less fun.

### `R` — Tremor, and Hover

**Tremor** is Quake cast at her own feet, which is why it is unaimed and on a key: the floor
around her shakes, staggering anything *moving* through it, then erupts and leaves a stone
**under her** — a telegraphed self-launch. It is the structure jump with a read attached, and
a way to make space when somebody has closed. Meets a stone she is standing on: pops it, and
her with it.

**Hover, airborne:** press and the fall is arrested for a few frames; hold and the hang
extends toward a cap, steering slowly. Hold is a dimension (how long), the press is the move.
It is how she lines up a Gale or a Downdraft, and it is committed — no dodge while she hangs,
so a hovering Elementalist is the most readable target in the game. The airdodge is spent by
it, as the Reaver's dash spends hers. Confidence medium-high; the cap on the hang is the knob.

`R` is also where a **second element** would toggle once ice or lightning exist. Nothing here
needs that, and the toggle is the one thing on a key that is not a verb.

## The table, all at once

How each verb answers each object. A dense table is the design goal; a sparse row is a
projectile.

| Verb | Body | Stone on the ground | Stone in the air | Fire pillar | Her |
| --- | --- | --- | --- | --- | --- |
| Bolt / Air bolt | interrupt | kick along · **loft** | kick down | fire bolt | — |
| Cataclysm / Gale | heavy · **lift** | debris | debris from above | tornado · **firestorm** | — |
| Fissure | stagger, stone | heave | sink | burning crack | — |
| Lift / Downdraft | lift · spike | loft · sink | — · meteor | stretch · ring | no |
| Grip | — | carry, throw | carry | — | footing |
| Pillar / Cinder | burn | **arm** | arm | — | — |
| Raise / Landfall | erupt · slab | pop | — | — | ride |
| Gather | — | orbit | orbit | — | cover, footing |
| Tremor / Hover | stagger moving | pop | — | — | launch · hang |

Bold is a change to something built. The two blank columns worth filling later are Grip on a
pillar and Gather on fire, and both are deliberately blank.

## What to watch

- **Juggling.** Lift, Gale and Tremor all put bodies in the air. In the air a fighter has one
  dodge and no guard, so a class that lifts freely owns the neutral. The gravity scaling helps
  (heavy classes barely leave the floor) and lifted bodies should get the Gale's rule: the stun
  is frame data and only the height scales. If it is still too much, Gale stops lifting bodies
  and Lift alone does, once per body per airtime.
- **The cap under more pressure.** Landfall already spends it; Tremor and Fissure do too. Three
  slots with five ways to fill them is the right pressure or too much, and only play says.
  The kit document's guess that three is a readability number rather than a balance one still
  holds.
- **Snapshot size.** A held stone is a position the stones already carry; an orbit is one
  angle; a hover is a frame count. Nothing here threatens the 4 KiB cap, but Gather's orbit
  should be a phase on the class rather than three positions.
- **Readability per button.** Nine verbs is more than six abilities. The argument that it is
  not more to learn rests on rule 1 holding — a player who knows *Lift raises what I point at*
  knows all four of its rows. If a verb needs its rows taught one by one, it is two verbs.

## If I had to prune

Keep every button, cut inside them. Drop **Gather** for **Sink** (a mode is a lot to read; a
free cap is quietly good). Keep **Lift** over **Grip** if only one thumb verb survives, and
give the freed thumb button to **Blast**: a short cone of air that kicks every stone in it and
shoves *her* the other way — aimed down in the air it is a rocket jump, which composes with
Landfall and is the most emergent thing on the list. Keep **Hover**; **Tremor** can wait on
whether a self-launch that costs a slot is wanted once Lift exists.

That leaves: Bolt, Cataclysm, Fissure, Lift, Grip-or-Blast, Pillar, Raise, Sink, Tremor-or-
nothing on the ground, and their air rows. Eight verbs. The kit spec's "six abilities plus an
auto and the mechanic" is eight inputs, so this is the spec's count, read as a grid.

## What this locks in for the control scheme

Written so the other five classes can be read against it.

1. **Three clicks, no chords, no modifiers.** Poke, heavy, committed. The committed click is
   `M` on the two classes that have one placed, and it should be `M` everywhere.
2. **The thumb buttons act on the class's objects, and are aimed.** Stones here; pools, the
   shadow, the shield elsewhere. Two per class, and a class with one object may leave one empty.
3. **`F` and `R` are about her, not a place.** State, stance, self-cast. Unaimed by
   construction, which is why they can live on the hand that does not aim.
4. **Every button has a ground row and an air row**, and the air row is the ground row with
   the vertical added or flipped. A class may leave an air cell as *the same move*; it may not
   leave it silent.
5. **Hold is a dimension of a move already thrown, never a different move.**
6. **A new move is written as its dispatch table.** One row is a projectile and needs a
   reason.

## Next

1. Read `M4`, `M5`, `F`, `R` in `read_input` with keyboard stand-ins, and give them the four
   free bits. Test alt in the browser while there (0001), though nothing here uses it.
2. Fissure on `M` and Lift on `M4` first: one is stranded and the other is one signed number
   with the largest composition payoff. Play those before anything else here is believed.
3. The two dispatch rows on the built aerials (loft, fire), which are the smallest change with
   the biggest effect on "dumb projectiles".
4. Then Grip, Hover, and the `Q`-on-stone arming, in whatever order play suggests.
