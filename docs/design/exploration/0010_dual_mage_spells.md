---
status: decided 2026-10-09 by the person's brief, designed and built by Claude; unplayed
started: 2026-10-09
supersedes: kits/dual-mage.md §"What is bound", the autos, Lance, Sweep; 0007 §5 where they differ
depends: ../dual-mage.md (the two bars, unchanged), 0007_core_kits.md, 0008, 0009
---

# 0010 — The Dual mage, every move a spell

## The brief (the person, 2026-10-09)

> A complete rework. For now left click stays dark and right click light, but every move needs
> to be changed to be a spell. The autos need to feel more like a short-range spell, since
> they are one of the three attacks we get on each side. `Q` and `E` are probably dark and
> light major attacks. Go through the whole combo decomposition, figure out move sets that are
> cohesive — what genre does each click fit in — craft the moves, then build them. Middle
> click: figure something cool out. The push/pull on the autos goes: it is awkward and feels
> neither mage-y nor melee-y.

**What stays:** the two bars, goading, the hill, the burn, the tiers (blink, second jump, slow
fall, float, wings) and ascension — all of [../dual-mage.md](../dual-mage.md). The rework is
the moves that feed them.

**What goes:** both punches and their wing-shaped blades, the pull and the shove, the step
on each auto, the two-form Lance and its tether, and the Sweep. "The form is the arm you last
punched with" goes with them: every move now *has* a force.

## The method: combos first

As with the Elementalist and the Blood mage: decide what she does *together* first, then break
it into pieces that each stand alone and recombine.

The class's own rule makes the combos for us. **Her two hands are two forces, and she is the
one thing that can hold both.** So the combo engine is a body that has been touched by one
force and is then touched by the other. That is also the bars' rule — alternating hands keeps
her level, and level is the only road to the wings — so **the thing that deals her damage and
the thing that climbs her bars are the same thing: alternating.** Being one-sided still works
and still runs away, exactly as the bars already say.

### The hex, and its two reactions

- **A dark spell hexes what it hits with Umbra; a light spell hexes it with Radiance.** One
  hex at a time, shown on the body in its colour; it fades on its own (`Hex, lasts`).
- **The other force on a hexed body sets it off**, and the hex is spent:
  - **Shatter** — light on an Umbra hex. The *strike*: a burst of bonus damage and a short
    stagger, right now.
  - **Wither** — dark on a Radiance hex. The *hex*: bonus damage drained back to her as
    health, and a slow.
- The same force again only refreshes the hex. So the bread and butter is
  **dark, light, dark, light** on one body: every second spell is a reaction, and her bars
  rise together.

### The six combos

1. **Hex and shatter.** Shade bolt (dark) hexes him → Sunray (light) shatters it → Shade bolt
   hexes again. The poke game, and the climb.
2. **Wither and walk.** Sunray hexes him → Shade bolt withers it: she heals, he is slowed →
   she takes the range she wants.
3. **Gather and judge.** Abyss (`Q`) drags a group together and hexes every one of them with
   Umbra → Judgement (`E`) on the well shatters every hex at once. The two majors are a pair:
   dark gathers, light ends it.
4. **Dawn and the air.** Space + right: Dawn throws him up and her with him → Reel (dark, in
   the air) pulls her onto him → Flare (light, in the air) off him, which kicks her higher →
   down together.
5. **Binary.** Middle click's twin orbs on a hexed body set off *its* reaction; on a clean one
   they set off **both at half** — a Shatter and a Wither in one hit. The balanced mage's
   payoff, and the one spell that goads both bars.
6. **The gate.** Phase (middle, in the air) through a gap with a flash left behind her where
   she was; Equinox (space + middle) is the jump that is highest when she is most level.

## Three genres, one per click

| Click | Genre | What every move in it does |
| --- | --- | --- |
| **Left — Dark: the hex** | Things that **travel and linger** | Projectiles you can see coming and dodge; wells that stay; drains that heal her; slows. Attrition and control |
| **Right — Light: the strike** | Things that **happen now** | No travel time: rays, bursts, pillars. Damage up front, launches, kicks |
| **Middle — Twilight: both at once** | Things only the **balance** can do | Powered by her *lower* bar, goads *both* bars, sets off reactions and moves her through space |

That split is also how an opponent reads her: **dark you dodge, light you block.** A Shade bolt
is a thing in flight you can step out of; a Sunray has already hit you or already missed.

## The grid

| | Left: dark (hex) | Middle: twilight | Right: light (strike) |
| --- | --- | --- | --- |
| **On foot** | **Shade bolt** | **Binary** | **Sunray** |
| **In the air** | **Reel** | **Phase** | **Flare** |
| **Leaving the floor** (space + click) | **Nightfall** | **Equinox** | **Dawn** |

| Key | Move | Force |
| --- | --- | --- |
| `Q` — the dark major | **Abyss** | Dark |
| `E` — the light major | **Judgement** | Light |

The majors are the same on the floor and in the air.

### Left — dark, the hex

- **Shade bolt** (the dark auto). A small dark dart from her left hand along the crosshair,
  fast but in flight — about nine metres of reach. Light damage, hexes Umbra, withers a
  Radiance hex. It is a projectile on purpose: the dark spells are the ones you can see coming.
- **Reel** (air). The same dart, and **on a body it pulls her to them** — the dark pulls; in
  the air she is the lighter thing. A hit is what pays for the distance (0007's refuel rule:
  paid in aim). It hexes and withers like the bolt.
- **Nightfall** (space + left). She jumps, and a small **dark well** blooms where she left the
  floor: for a moment it drags whoever is near it to its middle and hexes them. The setup
  takeoff — *leave something behind*.

### Right — light, the strike

- **Sunray** (the light auto). An instant short beam from her right hand to the first body on
  the crosshair's line, about seven metres. Light damage, hexes Radiance, shatters an Umbra
  hex. Instant on purpose: you block light, you do not outrun it.
- **Flare** (air). A burst of light a few metres along her aim. Whoever is in it is hit and
  hexed; and **if it met anything solid or a body she is kicked back the other way** — light
  pushes, and in the air she is the lighter thing. Aimed down at a floor or a body it is a lift;
  aimed at a wall it is a wall-kick. Paid in being near something.
- **Dawn** (space + right). A column of light from her feet as she rises: whoever is beside
  her is **launched up with her** and hexed. The *take them with you* takeoff.

### Middle — twilight, both at once

- **Binary** (floor). Two orbs, one of each force, wound round each other along the crosshair
  — mid range, in flight. A hexed body goes off with its own reaction at full; a clean body
  with **both at half**. Leaves no hex. Its power is her *lower* bar, and it goads **both** bars.
- **Phase** (air). She is **through**: a short blink along the crosshair, stopped by what she
  would hit, and a flash of light hurts whoever was beside her where she left. The air's
  answer to being caught, and a way across a gap.
- **Equinox** (space + middle). A takeoff whose height is her **lower bar**: the most level
  mage jumps the highest. The class mechanic, as a jump.

### The majors

- **Abyss** (`Q`, dark). A well where the crosshair meets the floor. For a second and a half
  it drags everything in it toward its middle, ticks damage, drains a share back to her and
  hexes Umbra. The setup for everything light.
- **Judgement** (`E`, light). As built: a delayed strike where the crosshair meets the floor,
  then a burning field that makes her fast. Now it hexes Radiance — and shatters every Umbra
  hex it hits, which is what makes it the end of the Abyss.

## The bars

- Every left move goads **Dark**, every right move **Light** — an auto by
  `meter_auto_push`, any other spell by `meter_cast_push`, and the majors by
  `meter_finisher_push`.
- **Middle goads both**, each by `meter_auto_push`. It never widens the gap.
- **Power** (`dual::depth_at`) reads the move's own bar; a middle move reads the **lower** one.
- **Which force she carries** — the HUD border, and what a question between moves reads — is
  the last force she threw; middle leaves it as it was.

## What the build touches

- Her table grows from six rows to eleven. The first six keep their slots and are retuned as
  the new moves (Dual mage is the last class in the Oven's move store, so the five new rows are
  appended at its very end).
- `Player::hex` / `hex_left` and `Monster::hex` / `hex_left`: one hex at a time. Hashed only
  when set. A creature is hexed and reacts like a fighter.
- New effects: the Shade bolt (and Reel's), Binary's orbs, the Abyss well (and Nightfall's),
  Flare's burst. Sunray, Dawn and Phase need no effect of their own.
- The takeoff window is the shared one (`Player::rise`); in the air **space is still her second
  jump and her wing beat**, so her takeoffs are from the floor only.
- Gone: the pull and shove (`knockback` is plain again on her row), the steps on the autos, the
  autos' wing-shaped blades, the tether, the Lance burst, the Sweep's two forms.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **DUAL-1** | Is the reaction too strong for a poke game? Every second spell is one | Small numbers on the autos; the reaction is most of the damage |
| **DUAL-2** | Does "dark you dodge, light you block" read from across the arena? | Bolt and orbs in flight are drawn; the ray is a line for a few frames |
| **DUAL-3** | Is Phase too good an escape? | Short, stopped by walls, one per airtime with the airdodge's cost |
| **DUAL-4** | Should Equinox be a jump at all from an empty pair of bars? | A floor height everybody gets, and the bars add to it |
