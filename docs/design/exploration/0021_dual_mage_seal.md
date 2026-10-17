---
status: open -- two designs built out for comparison; nothing built
started: 2026-10-17
depends: 0015 (the spellbook), 0016 (the hands), 0017 (the imagery), ../dual-mage.md (the two bars, ascension)
---

# 0021 — The Dual mage's seal: what binds the two beings, and two ways to play it

## Where this came from

The thread since 0017: the owner liked "the sword carries the spell" (the Magus's
spellstrike, 0017 §1) but not a spell that is only a button pressed out of combat. Loading
through landed sweet spots was rejected as a lose-more mechanic -- too much to track beside
spacing and two bars, and hard to land against a good opponent. Zones and gates were the
right direction (holy and unholy ground; a gate that moves with her) with one rule over all
of it: **mobility must never feel like losing out**. The Black mage of FFXIV is the opposite
of this class -- an immobile blaster -- and she is a mobile kite-skirmisher.

Then the image that holds it together, from the owner:

> The disk is actually the seal that holds the two beings inside, and its appearance is the
> result of the beings inside pushing against it. Light and dark sections, probably three
> components each; each part of the seal appears as that side grows stronger. The seal is the
> gate you use, so if a light spell sends the seal to an area you need to retrieve that part
> of the seal before you have the light side of your kit again. That solves "why one then the
> other" too. All the spells use one or both parts of the seal, which opens a "spellbind"
> move that binds the seal to an opponent for a short time.

And the brief for this note (2026-10-17):

> I see two ways to play the seal. The first is as an active part of combat: moving around,
> being pushed, retrieved. You are literally playing with the seal that binds the powers
> inside you, so when the seal is gone, using that side is much less controlled -- her autos
> should reflect this with much bigger areas, higher frame data, more damage. Retrieving the
> seal binds the energy and that side gets lighter and more controlled again. The second is
> as the spell focus: mostly imagery, but strong. The seal stays with you and becomes the
> source of spells. It strains then breaks, which is more natural than the other way --
> the other communicates "when it is away the forces are unbound", which is different from
> breaking. Both are strong thematically, so it comes down to gameplay. I lean towards the
> former as cooler and more dynamic, but it might read chaotic. Both need a full design and
> comparison. Also: a left then a right should come out faster than a right then a right.

## The seal itself (common to both)

**What it is.** She is a jailer. Two beings are locked inside her, and the seal is the
lock: a disk of six plates, three dark and three light, hung over her shoulder where the
sun-disk and the storm cloud hang today (`aim::overhead`). The two bars are drawn on the
seal rather than beside the screen: as the dark bar fills, its three plates come out one by
one, pushed from inside by the dark being; the same on the light side. At a third of a bar
one plate is out, at two thirds two, at full all three and the being's hand is visible
through the gap. **Both sides full and the whole disk strains and cracks** -- that is the
threshold of ascension, and ascension is what happens when it gives.

**The art, in three components a side.** Dark: obsidian, polished black with a white rim
(the lightning's look). Light: warm gold with the sun-disk's rings. The three components
are the three things a plate is in play -- the **thrown** plate (the auto), the **planted**
plate (the gate), the **given** plate (the major) -- so each plate's shape says what it does
before it does it. Between uses the three sit flush in the disk; out of it, a hole, and a
limb of the being reaching through.

**The rule every effect obeys:** a spell is a plate leaving the seal and coming back.
Nothing is conjured from nowhere; what is on screen is always a piece of the lock. The
HUD bar can go, or shrink to a reminder: the seal over her shoulder is the HUD, and it is
the same picture for both players.

**Spellbind** falls out of the plates: a dark plate planted *on a body* rather than on
the ground. The opponent carries a piece of her seal -- and so a hole in her kit -- until
she goes and takes it back. Thematically it is the being's grip on them; mechanically it
makes the target the place she has to be.

## Design A: the seal in play

The seal is a thing in the fight. Plates leave her, stand in the world or on a body, and
come back when she passes through them.

### The states of a side

Each side is in one of three states, read off the seal:

| State | Seal | That hand's autos | That hand's holds |
| --- | --- | --- | --- |
| **Bound** (all plates home) | The plates flush, turning slowly | The ball as today: 0.6 m, tidy curve, sweet spot at the end, fills the bar | The three-phase hold (below) |
| **Unbound** (a plate out) | A hole, the being's limb through it | **Wild**: the ball half again as big, no sweet spot (it is all sweet spot), slower wind-up (+4), longer recovery (+6), damage ×1.5, knockback ×1.5, and it **drains** the bar rather than filling it | Only the recall; the plate must come home before the hold is hers again |
| **Given** (the major's plate shattered) | Two holes, and the disk lopsided | Wild, as above, until the plate reforms (a long clock, ~8 s) | None |

Unbound is the owner's "much less controlled": bigger, slower, harder, and the energy
runs out of her because nothing is holding it in. **Wild autos are stronger per hit but
cost her the bar.** The bound auto is her economy; the wild auto is her spending it on the
move, and it is louder in every way (sound, size, screenshake) so the state is never in
doubt.

### What leaves the seal, per hold

Every hold walks one plate out, a step further per phase. Letting go picks the phase;
holding past the last fires it, so a held button never strands her.

| Hold | Phase 1 (release early) | Phase 2 (release mid) | Phase 3 (held to the end) |
| --- | --- | --- | --- |
| **Right** (light) | **Sunbeam**: the thrown plate flares and the ray comes down the line to the crosshair (the Disk ray today) | **Sanctum**: the planted plate slams flat onto the ground at the crosshair in a flash and stays as a gold seal, 3 m across, 6 s. *The light side is unbound until she retrieves it.* | **Judgement**: the given plate flies up and comes down as the columns; shattered, reforms in 8 s |
| **Left** (dark) | **Lightning**: the thrown plate becomes the cloud's bolt onto the body nearest the line (Chain lightning today) | **Spellbind**: the planted plate bolts onto the struck body, a black mirror on their chest, 6 s. *The dark side is unbound until she takes it back.* It curses them on the tick while it sits there. | **Abyss**: the given plate lands as the black sun |
| **Middle** (twilight) | **Flurry**: the cascade, as built, to the disk at its end | **Twin gate**: the disk at the end *stays* -- both planted plates, as one gate the width of her, upright, 6 s. *Both sides unbound.* | **Eclipse**: the gate implodes; everything within a reach is pulled in and struck by both forces |

### The gates, and the autos through them

A planted plate is a gate. A ball that **flies through** a gate is **empowered** -- it
does not matter where she threw it from. So the sanctum is a lens she plays around, not a
room she stands in, and the spellbind is a lens *on the target*, so every ball at them goes
through it:

| | Through the sanctum (light gate) | Through the spellbind (dark gate) |
| --- | --- | --- |
| **Light ball** | **Lance**: pierces its line through to what stands behind | **Purge**: sets off every curse on them |
| **Dark ball** | **Scorch**: burns along its path, a trail of embers | **Grasp**: shadow hands root them a moment |

Light through light pierces, dark through dark binds, and each through the other's gate
*reacts* -- the curse and purge relationship she already has, now with a shape. Four
spells, no new inputs. **Remember the hand that threw it is unbound** while its gate is out
(the other side's gate, or none, if she threw the other hand): so a ball through the
sanctum from the light hand is a *wild* light ball becoming a lance. The empowerment stacks
on the wildness; a gate is the one place being unbound pays double.

### Retrieval

She gets a plate back by **touching it**: walking into the sanctum, hopping through the
twin gate, or **reaching the spellbound body** (an auto that lands on them carries it home;
a hop that ends within a reach of them takes it). Retrieval is a moment -- the plate snaps
back, the being's hand slams behind it, the next ball from that hand comes out at sweet-spot
size for free. The loop: **throw a plate to set the stage, fight through the stage, collect
it on the way out.** She is always moving toward her own kit.

**Recall** (`F` light, `R` dark): pull the planted plate home from anywhere. The sanctum
collapses in a flash that wards her briefly; the spellbind rips free with one tick of
damage. It costs the bar (a vent), and it exists so a plate behind a wall, or on a body she
cannot reach, is never a trap. It is the only thing `F` and `R` do.

### The controls

| Input | Bound | Unbound |
| --- | --- | --- |
| Left / right click tap | Thrown plate (the ball), and a hop | **Wild** ball, and a hop |
| Left / right click hold | The three-phase ladder | Nothing more than the tap |
| Middle click tap | Binary / Phase (both thrown plates) | Same, wild if either side is |
| Middle click hold | Flurry → twin gate → eclipse | Nothing more than the tap |
| `F` / `R` | -- | Recall that side's plate |
| `Q` / `E` | The major outright (phase 3 now), spending the plate | -- (the plate is already out) |
| Space | Jump; through a gate it retrieves | Same |

### Ascension

Both bars full is the whole seal straining. It gives: all six plates fly outward in the
nova (Apotheosis) and **both sides are unbound for the duration** -- every auto wild, no
gates possible because there are no plates to plant, and the health drain is the energy
running out of her with nothing holding it. On landing the plates fly home and reassemble,
and the bars are empty: the lock has been remade.

## Design B: the seal as the focus

The seal stays over her shoulder. It is where every spell comes from and the gauge of
everything she has, and it never leaves her. It **strains** as the bars fill and **breaks**
at the top.

### The states of the seal

| Strain | Seal | What it means |
| --- | --- | --- |
| **Calm** (both bars low) | Plates flush, turning slowly | Autos tidy, holds available, nothing special |
| **Straining** (a side at two thirds) | That side's plates stand out of the disk, light leaking between them | That side's spells are **bigger** -- the bar is power, as the depth curve already says -- and the seal visibly shows which |
| **Cracked** (a side full) | A plate hangs by a thread, the being's hand through the gap | That side's next major is **free of its recovery**: the being throws it for her |
| **Broken** (both full) | The disk comes apart | Ascension |

### What the seal does, per hold

Spells are projected *out of* the seal; nothing is planted and nothing is retrieved. The
phases are the seal opening further:

| Hold | Phase 1 | Phase 2 | Phase 3 |
| --- | --- | --- | --- |
| **Right** | **Sunbeam** from the seal's light face | **Projection**: the light face is projected to the ground at the crosshair as a seal of light, 3 m, 6 s -- a *copy*, the plates stay home. Autos through it are lances/purges as in A. | **Judgement** out of the projection |
| **Left** | **Lightning** from the seal's dark face | **Spellbind**: the dark face is projected onto the body: a mirror on their chest, 6 s, cursing on the tick. A copy; autos at them are grasps/scorch as in A. | **Abyss** out of the mirror |
| **Middle** | **Flurry** | **Twin gate** at the end, a projection of the whole seal | **Eclipse** |

So the gates exist here too -- and the autos through them -- but they cost nothing and
make no hole. The cost of a gate is the hold's frames and a vent of the bar.

### The controls

As today, with the ladders: taps are balls, holds are the ladders, `Q`/`E` are the majors
outright, `F`/`R` vent (the Devour and the Smite, or simply a vent that *recalls the
projection* early for a flash). The seal is a picture of the bars and a source for the
spells; it is never a thing to go and get.

## The comparison

| | A: the seal in play | B: the seal as the focus |
| --- | --- | --- |
| **What the player tracks** | The seal, which replaces the bars: which plates are home, which are out, where. Three states a side, all drawn on one object both players see | The seal, which replaces the bars: how full, how strained. Two states a side |
| **Why one hand then the other** | Structural: a planted light plate unbinds the light hand, so she fights dark until she walks through it | Only the alternation bonus below, and the hill |
| **Mobility** | **The design.** Every plate out is a place to go; retrieval pays. A gate is passed through, never stood in. Black mage's opposite | Neutral. Gates are lenses she plays around, as in A, but nothing pulls her anywhere |
| **Risk of reading chaotic** | Real. Wild autos are bigger, louder and cost bar; a player who plants carelessly spends a fight unbound. Mitigations: wild autos *stronger* so unbound is never only a penalty; recall on `F`/`R`; the state drawn on the seal in one glance | Low. Nothing changes under her hands |
| **Decision density** | High: when to plant, where, which side to leave open, when to recall | Medium: when to hold, how far |
| **Spellbind** | Full: a piece of her on them, a hole in her, and a reason to close. Both players feel it | A debuff with her colours on it |
| **Thematic truth** | "When the seal is away the forces are unbound" -- the owner's note that this is different from breaking. It is: the seal **away** is a hole, and the seal **broken** (ascension) is every hole at once. Two readings of one object, and they agree | "It strains, then breaks": the cleanest single arc, and the breaking is more natural because nothing has been away |
| **Art cost** | The six plates in three shapes each, the limb through the gap, wild ball variants, retrieval flash, the recall | The six plates and the strain states; the projections reuse the gate art |
| **Sim cost** | A per-side state (bound/unbound/given) and up to two planted plates as effects with retrieval; wild auto rows (two more move rows or a multiplier); `F`/`R` recall | A strain state read off the bars; the projections as effects |
| **Snapshot** | Two bytes of state, two effects | Nothing new |

**Recommendation.** Build **A**, with two guards against chaos taken from B: the wild
auto must be *better* in some way she wants (damage, knockback, size) so being unbound reads
as a mode she chose rather than a punishment she walked into; and the seal's one picture
must carry everything -- if a tester has to ask "which side am I missing", it has failed and
B is the fallback. A first prototype needs only the right-click ladder's phase 2 (the
sanctum), the light side's unbound autos, retrieval by touch and recall: one side, the
whole loop.

## The alternation rule (both designs)

> A left then a right should come out faster than a right then a right.

**Not two separate cooldowns** -- the owner's worry that it reads awkward is right: two
clocks are two more things to track, and a hand that is simply locked feels like a dropped
input. Instead, **a same-hand repeat pays a tax on its wind-up**: the second ball from the
same hand within a beat starts `same hand, slower by` frames later (+4 of a ~10-frame
wind-up), and a ball from the other hand within the same beat starts `other hand, faster
by` sooner (−2). The beat is short (~30 frames), so it only shapes the flurry, never the
neutral. Nothing is ever refused; the alternating rhythm is simply the fast one, and the
player finds it with their hands rather than reading it off a clock.

**Drawn:** the being whose hand just threw is **recoiling** -- its limb pulled back behind
the plate -- and the other is leaning out, ready. The seal tilts toward the hand that is
fresh. So the tax is visible as posture, which is how a fighter reads it.

**Thematically** it is the seal itself: one being pushed, the other has room. And it pairs
with A's unbinding -- when one side is unbound the alternation bonus still applies, so a
wild dark ball between two light ones is still the fast way.

## Open questions

- **DUAL-17.** A or B. The recommendation is A with B's guards; a prototype of one side
  settles it.
- **DUAL-18.** How wild is wild. ×1.5 size and damage, +4/+6 frames, and the bar drained
  is a first guess; the feel test is whether a player *wants* to be unbound sometimes.
- **DUAL-19.** Retrieval by touch only, or also by time? Touch only is purest; a plate
  that reforms on its own after a long clock (10 s) is the safety net if recall is not
  enough.
- **DUAL-20.** Where the seal hangs. Over the shoulder keeps the overhead line of effect;
  at the chest reads as a lock on a body. The limb through the gap wants to be seen.
- **DUAL-21.** The alternation tax's numbers, and whether the hop after an auto is also
  shorter on a same-hand repeat (the hop is where the rhythm lives).
- **DUAL-22.** Ascension in A: all six plates out and both sides wild for the whole
  duration, or the disk breaking and reforming once, with the wings as today.
