---
status: exploration — second pass on 0002/0003 after review
started: 2026-09-30
supersedes: the grid in 0002 where they differ
---

# 0004 — The Elementalist converts space into damage

The review of 0002 cut three things and added one idea that turns out to be the kit's
centre. Cut: Grip, Gather, and Lift-at-a-point (the crosshair has nowhere to rest in the air,
and "bring it closer" has no moment where it beats raising a new one). Added: **charge modes
on the two placement buttons**, held at the crawl, that do something heavier on release.

That addition is the reward for spacing the kit lacked, and it is worth saying why before
the moves, because it decides what the rest of the grid is for.

## Distance is the resource the charge spends

A charge is time spent at the committed speed, in full view, unable to dodge. The opponent
walks at 7 m/s and she crawls at 1.4, so **every tenth of a second she holds costs about
seven tenths of a metre of the gap**. A one-second charge is only safe with roughly seven
metres between them, less whatever a stone in the lane, a slow field underfoot or a stagger
buys. That is the class's whole sentence made mechanical: she builds terrain to buy distance,
and a charge is how distance is cashed for damage. A player who has spaced well gets a payoff
nobody else in the roster can reach; one who has not gets hit out of it at a crawl.

The frame table could print it: *safe charge distance* per hold length, against the fastest
walk in the roster.

Both charges are the `channel` the move table already has. Its own comment: *what a channel
chooses is a number the move otherwise takes from the table.* The Grasp chooses reach. These
choose a number each, below, and the rule from 0001 survives sharpened: **a hold deforms the
move along one axis; it never selects a different one.** A pillar and a strike are the ends of
one shape, and a stone and a Fissure are too.

**Immediacy is kept by letting the startup be the tap window.** The press begins the move's
own startup; a release inside it is the ordinary move, and a hold past it is the charge. A
tap costs nothing it did not already cost.

## The two charges

### `Q` — Fire pillar, held: the strike

Tap: the pillar as built. Hold: the fire **gathers in her hands** rather than at the target —
brighter and tighter the longer she holds, visible across the arena — and she crawls. On
release, a column of flame **the pillar's exact shape** lands at the crosshair's point and is
gone in a few frames: the pillar's whole burn concentrated into one hit. The number the channel
chooses is *how much of the fire arrives at once*. A tap is all burn and no strike; a full hold
is all strike and no burn; in between, a flash and then a shorter pillar.

The aim is live through the hold and locks on release, as the Grasp's does. That is what makes
it a spacing reward rather than a telegraph: the opponent sees *that* she is charging and *how
far* it can reach, not where it lands. Their answers are the honest ones — close and hit her
while she crawls, or stay out of range, or put a stone in the lane. A strike that meets a stone
lands on its top and **lights it** (below), so cover against the strike costs them the cover.

### `E` — Raise, held: Fissure

Tap: a stone at the crosshair, instant, as built. Hold: the patch **churns and does not
erupt** — the rise's first half, held — and she crawls. On release the earth *travels*: a
crack races from the churning patch along the direction she is looking, staggering and slowing
along its length, and the stone erupts where it stops — at the first body it meets, or at the
end. The number the channel chooses is **how far the crack runs**, the Grasp's own axis.

Fissure is a Raise that went somewhere. That is the thematic continuity asked for, and it
frees middle click. It also gives the class a stone at range with a telegraph attached, which
Raise's instant never could, and the slow field along the crack is the setup the pillar strike
wants: somebody slowed in a scar is somebody who cannot leave the strike's footprint.

## Air on her, not at a point

Updraft and Downdraft stay, moved on to **her**. A column at a point in the air was unaimable;
a column on her body is not aimed at all, so it moves to the unaimed hand.

### `F` — Updraft, standing

A cylinder a couple of body widths across, either centred on her or with one edge at her feet
and its centre a fixed distance along her flat facing (`aim::planted_ahead`, the Landfall
slab's placement, so nothing new is aimed). The air gathers for a medium wind-up — dust rising
in the footprint — then one hard gust. **Everything in it goes up, her included**, each by
their own gravity: she rises a full jump's worth, the Bulwark barely leaves the floor, a stone
lifts a body height and drops. It is her "jump higher" that costs a cast rather than a slot,
and a body lifted beside her is a body with one dodge and no guard.

### `F` — Downdraft, airborne

The same cylinder drawn falling, under her. She and everything in it are driven down. **If she
lands while it is still blowing, the air breaks outward** — a ring gust from her feet that
shoves everyone nearby away, a body width or two, no damage. That is the fast defensive tool
the kit did not have: caught in the air and about to be juggled, Downdraft puts her on the floor
now and opens space on arrival, and the space is exactly what the next charge needs. A lofted
stone in the column comes down as a meteor; a grounded one is pressed into the floor and leaves
the slow field.

## Fire in the air — the flare

The aerials were flat because they met nothing. Rather than have them lift things, which read
as odd, give them something to fly through.

### `M` — Flare, both rows

A thrown ember that **bursts at a fixed distance along the crosshair**, or on the first thing it
meets, into a hanging cloud of sparks a couple of metres across that lasts a couple of seconds.
It needs no rest point, because the range sphere *is* the burst point — a firework, not a shot.
Burst on the floor it is a low burning patch instead. It is one move, and the row only changes
where it pops.

What it is for:

- **Air shots through it ignite.** An Air bolt comes out as fire — more damage and a small burst
  on hit. A Gale comes out burning, a firestorm disc that leaves embers along its path. This is
  how the air row gets teeth without a lifting mechanic: fly the shot through the fire you put
  there.
- **A stone in the cloud is lit** (below).
- **Backup fire.** The pillar is far away or burnt out; the flare is fire wherever she is
  looking, in a second.
- **The fighter-jet flare.** Fleeing in the air, pop one behind you. Whoever follows flies
  through embers, and your next shot back through them is fire.

This absorbs Cinder rain, which no longer needs a shape of its own.

## The built aerials, adjusted

Air bolt: **more knockback**, and it lights through a pillar or a flare. Gale: **more
knockback**, it **pushes stones** it passes — kicked along its travel, the beam's shove made
wide — and it lights through fire. Nothing lifts. The Gale pushing a lit stone into somebody
is the class's air combo, and it is built from three things she placed.

## Fire on earth has a job now

Fire on a stone — a pillar strike landing on its top, a flare bursting beside it, a Gale that
was already burning — **lights it**: it glows at the seams and smoulders for a while. A lit
stone does three things, and each is a moment where you would want it:

1. **It takes the cover back.** The versus document's one ruling is that her stones are cover
   for the opponent too, and nothing has ever answered that. A lit stone burns whoever stands
   on it and, when *anyone* shoves or breaks it, bursts into burning debris — so the opponent
   hiding behind her wall is hiding behind a bomb, and standing on her platform is standing on
   a stove.
2. **It is a delivered explosion.** Kick it with the beam or a Blast and it bursts on impact,
   Cataclysm's payoff at range with a fast wind-up, paid for by having lit it first.
3. **It makes the swap a trap** — next section.

That is the niche fire-plus-earth was missing: **fire is how she denies her own terrain to the
other side.** Earth builds the field; fire decides who may use it.

## The dodge, pointed at what she built

Shift plus a direction is the universal dodge, and on three classes it is also the class's
mobility when pointed at the class's object: the Reaver's shadow, the Blood mage's pool, the
Dual mage's bar. She has two objects.

### Dodge at a stone — **swap**

The dodge carries her *into* the stone and she comes out where it was; the stone comes up where
she left. The substitution: whoever was chasing her now has a wall in their face and her behind
it. Fast, costs only the dodge and the stone's position, and its telegraph is that the stone
exists. It is the one defensive tool that beats a combo rather than a hit, which the review
asked for, and it is about *her* moving rather than about bringing anything to her. Dodge at a
**lit** stone and it bursts on arrival at her old spot — burning debris into the pursuer.

The alternative is **mantle**: the dodge puts her on top of the stone, instant footing, the
Reaver's dash on to a dais. Simpler, less defensive, and it competes with the structure jump for
the same job. If the swap reads as a blink too many (there are two already), mantle is the
fallback.

### Dodge through fire — **the trail**

Her own fire cannot hurt her. Dodge through a pillar or a flare cloud and the dodge **drags
the fire with it**: a burning line along the dodge's path, on the floor or hanging in the air,
lasting a few seconds. A wall of fire drawn with her body, between her and the thing she
dodged. In the air it is a hanging line her next shot can fly through.

## `R`, and the two thumb buttons

**`R` — Tremor**, standing: Quake at her own feet. The floor in a ring around her shakes through
a slow, visible wind-up; anyone *moving* through it staggers and anyone standing still is fine,
which is what makes it a read. Then it erupts and a stone comes up under her, taking her with
it. It costs a slot and a wind-up they can see, and it makes space and height when somebody has
closed. Standing on a stone already, it pops that one instead. Airborne, `R` is open; **Hover**
(a held hang, spending the airdodge, committed) is the candidate and nothing here depends on
it.

**`M4` — Blast**, both rows: a short, fast cone of air from both hands along the crosshair, a
swing's aim so it needs no rest point. It kicks **every stone** in the cone forward — the beam
kicks one, this kicks the field — shoves bodies back a step, and shoves *her* the other way.
Aimed down in the air the recoil is a **rocket jump**. It is the most "make her move" thing on
the list and the one most likely to become the class if its numbers are generous; the recoil
is the knob.

**`M5` — open.** Quake at range (the kit's listed ability, the aimed sibling of Tremor) is the
candidate if the class wants a third way to place a stone; the second element's slot is the
other. A thumb button left empty costs nothing.

## The grid, revised

|  | Ground | Air |
| --- | --- | --- |
| `L` | Bolt | Air bolt — more knockback, lights through fire |
| `R` | Cataclysm | Gale — more knockback, pushes stones, lights through fire |
| `M` | Flare | Flare |
| `M4` | Blast | Blast — recoil is a rocket jump |
| `M5` | *open* (Quake) | — |
| `Q` | Fire pillar · **hold: the strike** | Pillar; a charge continues while she falls |
| `E` | Raise · **hold: Fissure** | Landfall |
| `F` | Updraft — her and the cylinder | Downdraft — her and the cylinder; landing gust |
| `R` | Tremor | *open* (Hover) |
| shift → stone | **Swap** | Swap |
| shift → fire | **Trail** | Trail |

Nine buttons, two of them holding a charge, two dodge variants, no captures.

## How a round reads

Raise a stone, hold `E` a moment so it runs the crack toward them instead; they are slowed in the
scar. Charge `Q` — they see the fire in her hands and start to close, but the scar costs them the
half second she needs — and the strike lands on the spot they could not leave. Or they get there
first: Downdraft to the floor, the landing gust opens two metres, swap through the stone behind
her, and she is charging again on the other side of a wall. Every move in that paragraph is
either building space or spending it, which is what she should be.

## Open, after this pass

- **The charge lengths** and what a full hold is worth. The strike's full-hold damage against
  the pillar's total burn is the one ratio the feel harness should pin: a strike worth *more*
  than the burn it replaced is a free upgrade, worth *less* is a trap.
- **Live aim on the strike** versus locking the target on press. Live is the spacing reward;
  locked is the fairer telegraph. Play both.
- **Swap versus mantle**, and whether the swap's arrival needs a frame of vulnerability so it
  is not a free reset.
- **Blast's recoil**, which is the number that decides whether she is a mage or a rocket.
- **Whether Updraft lifting her competes with the structure jump** or complements it — it is
  cheaper (no slot) and slower (a wind-up), which may be exactly the right pair.
