---
status: built 2026-10-18, unplayed
started: 2026-10-18
supersedes: 0016 where they differ
depends: 0016_blood_mage_thralls_and_the_well.md, ../kits/blood-mage.md
---

# 0021 — The Blood mage: territory

The owner's new direction: **she is about controlling territory and building forward.** Her
area spells are empowered ground she uses to her own advantage; the stains build towards it;
she is more mobile inside it. The Elementalist no longer stands in the way -- in practice she
is a combo mage, not an area-control one.

**Sentence:** her blood spreads like a sickness, and she fights from inside it.

This note is the decision, after two rounds of options in conversation. The options that were
offered and not taken are listed at the end, so they are not lost.

## 1 · What was decided

- **Ground is one substance.** No kinds, no thickness, no age (those two are kept for later,
  §8). Three things make it: **her abilities, wherever they touch**; **the bleed**, whose ticks
  spill stains; and **stains**, which are ground while they stand and **bleed into ground as
  they fade**.
- **Patches bridge.** Touch makes short-lived sparks; a stain's ground lasts longer; two
  patches of hers close enough to each other grow a bridge between them, and a patch that
  touches others fades slower. Scattered close together, the sparks join into a field; far
  apart, they stay islands and go out. That is the plague picture.
- **On her ground she is strong.** Every move she throws with her feet on her ground is the
  **strong version** -- the same move, harder. Off it, the move is the **weak version**, and
  it leaves more ground behind it: off-ground she builds, on-ground she hits. The rhythm is
  throw from the edge, step in, hit.
- **She swims in it.** `shift` on her ground sinks her into it: fast, hard to hit, a ripple on
  the surface. A click comes up as that click's move (strong, since she is on her ground by
  definition); letting go comes up; the edge pops her out with a burst. **The teleport to a
  stain under the crosshair works exactly as before** and comes first.
- **Thralls are the risk of entering.** On her ground they hit harder and run faster -- drawn
  bigger and brighter, so it is seen -- and do not drain. They prefer her ground: they go for a
  body near it, and with nothing to bite they go back to it. Command is gone: her territory is
  where they gather, so growing it is how she sends them. Enemies get no other penalty on her
  ground, so the risk is legible -- it is the bodies standing in the red.
- **Hearts.** `E` plants one: sturdy (several hits, not one), it holds a radius of ground
  round itself, and while it stands it sends out a thrall every few seconds, spending its own
  blood to do it. Two at most. She leaves one and protects it by pushing forward so it cannot
  be reached.
- **The three clicks are thorns / spikes / scythe.** Thorns are the area attacks that make
  ground by bleeding; spikes are the damage, **unchanged**; the scythe is fast and reliable up
  close, because she advances to where they just bled.

## 2 · The grid

| | Left: thorns | Middle: spikes (unchanged) | Right: the scythe |
| --- | --- | --- | --- |
| **Tapped** | **Thorn lash**: a bolt of thorns; it cuts, bleeds, and leaves ground where it lands, hit or miss | **Black spike**, one | **Sweep**, and tapped again inside its recovery, a **string** of up to three: the second comes back the other way, the third heavier |
| **Held** | **Bramble**: a thorn field growing for as long as she holds, hurting and slowing; it lays ground across itself | **Black spike**, one to seven (air: **Stake**) | **Reap**: a lunge into a swing, further the longer it was held, and further again on her ground |
| **Held to the end** | **Black vines** (unchanged) | **Crimson cascade** (unchanged) | **Harvest**: one great turning reap round her, drinking every stain in its reach |
| **Space + click** | **Thornfall**: the wings' burst along her aim, landing in a ring of thorns that leaves ground | **Upheaval** | **Rising reap** |

| Key | Move |
| --- | --- |
| `Q` | **Coagulate** (unchanged: dragging stains is dragging ground) |
| `E` | **Heart** |
| `shift` | The teleport to a stain under the crosshair; otherwise, on her ground, **swim**; otherwise an ordinary dodge |

The rows are retuned, not renumbered (`sim::moves::blood`): the Haemorrhage's row is the Thorn
lash, the Hemoplague's the Bramble, Command's the Sweep, the Carpet's the Reap, the Summon's
the Heart, the wings' Thornfall, the golem's Harvest. Cut: **the Carpet, Command, the Summon
and the Blood golem.**

## 3 · The ground

A **patch** is a disc on the floor: where, how wide, whose, and how many frames it has left.
They live in their own small table in the snapshot (`World::ground`), not in the effect pool:
there can be dozens, and they must not crowd out a fight's effects.

**Where it comes from:**

| Source | Patch | Lasts |
| --- | --- | --- |
| A move of hers touching the floor or a body -- a spike or an eruption, the Thorn lash's end, the Stake's landing, the Upheaval, Thornfall's ring, a cascade's eruption, the vines | the radius of what touched | `ground, a touch lasts` (2.5 s) |
| The Bramble, while it stands | its own radius, refreshed every tick | as a touch, from its last tick |
| A stain, while it stands | the stain's column and `ground, a stain spreads` round it | as long as the stain |
| A stain that **fades** (drains out, rather than being drunk or set off) | the same disc | `ground, a faded stain lasts` (6 s) |
| A heart, while it stands | `heart, holds ground` round it | as long as the heart |

**Thrown off her ground, a touch seeds more**: `ground, off her ground seeds (x)` on both the
radius and the life.

**Bridging.** Every few frames, two patches of hers whose edges are within `ground, bridges a
gap of` get a patch laid over the gap between them, and every patch that touches another of
hers -- or a stain, or a heart -- loses its life at a share of the normal rate (`ground,
joined fades at (x)`). So a field holds while it is fed and joined; an island goes out.

**The cap.** A patch laid mostly on top of one of hers merges into it (it grows to cover
both, up to `ground, widest patch`, and keeps the longer life). Past the table's size, the
patch with least life left -- hers first -- makes way.

**On her ground** means her feet on the floor (or swimming) inside one of her patches, a stain
of hers, or a heart's radius. It is worked out once a frame and kept on the fighter
(`Player::on_blood`), because the input code, the renderer and the thralls all ask it.

## 4 · Strong and weak

**The strong version is the move's damage times `ground, her moves hit (x)` (×1.3).** It is
decided when the move starts -- where she stood when she committed -- the way the Dual mage's
depth is (`Player::thrown_at`), so a move that carries her off her ground mid-swing keeps what
it started as. Sizes do not change: the spikes and fields are exactly as big either way.

**Drawn:** while she stands on her ground an aura of essence rises off her, the same violet
flame her stains carry, so a player can see from across the arena whether she is on it.

## 5 · Swim

- **In:** `shift` on her ground, on the floor, with no stain under the crosshair (the teleport
  comes first), with or without a direction.
- **Under:** she moves at `swim, speed (x walk)`; her hurt volume is `swim, height` tall, so
  most blows thrown at a standing body pass over her; nothing she does is invulnerable.
- **Out:** let go of `shift` and she stands. A **click** comes up as that click's move. A
  **jump** comes up as a jump. A **hit** brings her up. **The edge** of her ground pops her out
  with `swim, pops out at (m/s)` along the way she was going.
- She cannot swim in the air. Walking into stains while swimming drinks them, as walking does.

## 6 · Thralls and hearts

**A thrall on her ground** bites for `thrall, on her ground bites (x)` and runs at `thrall, on
her ground runs (x)`, and does not drain; off it, it drains as its stain would. It goes for the
nearest enemy within its seek **who is on her ground or within `thrall, leaves her ground by`
of it**; with none, a thrall off her ground walks back to the nearest of it. `Effect::power`
carries how strong it is this frame, which is what the renderer draws it by.

**A heart** is planted at the crosshair's floor (grounded, within the move's reach). It holds
`heart, from what she paid (x)` times what she paid for it (never less than `heart, least`).
Blows come off that; at nothing it is gone. Every `heart, sends a thrall every` frames, while
she has room for another thrall, it spends `heart, a thrall costs` of its blood into a thrall
at its foot. Two at a time: a third replaces the oldest. It is drawn as a pulsing knot of
essence standing out of its ground.

## 7 · What it does not change

The grey and the finishers, the Well, the stain economy (every cost a stain, walking always
drinks, eruptions spill nothing), Coagulate, the spikes and their chains, the teleport, the
Stake and the Upheaval, the vines and the cascade.

## 8 · Later, not now

- **Depth.** Ground laid on her ground deepens it: thin is speed, deeper slows, deepest has
  thorns. If the life of a patch stays a single number, it can become a depth.
- **Age.** Fresh ground slick for her, settled ground dragging at them, old ground crusted and
  hurting.
- **Supply lines.** Ground cut off from every heart fades fast; a heart as what the territory
  hangs from. Only if hearts turn out to be good.
- **A slow for enemies** on her ground, if play shows opponents walking through her territory
  because the thralls are not threat enough.
- **Spike homing** on her ground was offered and **rejected**: the spikes are the best thing
  she has and stay a skill shot.

## 9 · Numbers (first guesses)

| Knob | Value |
| --- | --- |
| a touch lasts | 150 frames |
| a faded stain lasts | 360 frames |
| a stain spreads | 0.8 m |
| off her ground seeds | ×1.5 |
| bridges a gap of | 2 m |
| joined fades at | ×0.33 |
| widest patch | 4 m |
| her moves hit | ×1.3 |
| swim speed | ×1.8 walk |
| swim height | 0.6 m |
| pops out at | 9 m/s |
| thrall on her ground bites / runs | ×1.5 / ×1.4 |
| thrall leaves her ground by | 3 m |
| heart, from what she paid | ×3, at least 150 |
| heart sends a thrall every | 240 frames, costing 60 |
| heart holds ground | 3 m |
| Reap lunge | 3.5 m, ×1.6 on her ground |
| Sweep string | three; the third ×1.5 |
| Harvest reach | 5 m all round |

## 10 · As built, 2026-10-18

Everything above is in the game ([../kits/blood-mage.md](../kits/blood-mage.md) §"Territory",
`tests/territory.rs`). Where the build had to decide something the note did not:

- **The table is forty-eight patches of sixteen bytes.** The snapshot was 1 KiB under its
  20 KiB cap; a patch's radius is a byte in thirty-seconds of a metre.
- **A heart bleeds away as a stain does**, as well as spending blood on thralls: one that
  stood forever would be ground nobody could take back, and its frame counter would overflow.
- **A heart's blows** are the ones a thrall feels, and one swing counts once across its
  active frames.
- **A swing's cost lands at the blade's end.** Every cost is a stain where the cast lands; a
  swing's used to land at her feet, where walking drinks it back, so the Sweep was free.
- **Thornfall's ring is where she lands**, and the teleport to a stain arrives in the same
  ring.
- **The numbers `feel.rs` caught**: the Reap is the table's first row, which the feel
  properties read as the class's poke, so it hits for 55 rather than 70 and keeps 60% of a
  walk; the Sweep reaches 2.6 m (at 2.4 a knee-high critter at 3 m was missed where a
  fighter was caught) and is 0 on hit; the Harvest shoves 12 m/s and costs 1%.
- **Her movement envelope grew** (`tests/fixtures/envelope.txt`, re-searched): the Reap's
  lunge from her own ground carries her further than anything in the old kit.
- **The creature turns on a heart** in its way, as it does on a thrall nearer it than she is.
