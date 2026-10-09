---
status: decided 2026-10-09 by the person, combos first; built 2026-10-09, unplayed
started: 2026-10-09
supersedes: 0007 §4 (the Blood mage), where they differ
depends: 0007_core_kits.md, ../kits/blood-mage.md, ../blood-mage.md
---

# 0009 — The Blood mage on three clicks

The same treatment the Elementalist got in 0008: the class's core on left, middle and right
click, each with a move on the floor, an aerial and a jump attack (space and the click), in the
Champion's grid. Built 2026-10-09; what was built is [../kits/blood-mage.md](../kits/blood-mage.md) §"On three clicks".

## Decided, 2026-10-09: the combos come first

The person's method, from what worked on the Champion and the Elementalist: **decide the combos,
decompose them into their pieces, and let each piece stand on its own**, so a player can
recombine them freely. Her combos run on four shared states: **pools on the floor** (hers or
theirs), **grey health** (spent and not yet gone; it grows the scythe and fades), **a body in
the air** (lifted or pinned), and **a bleed** (a target who drips pools as they go).

1. **The Hanging.** Marionette lifts him → Hook pulls her up to him → Nail pins him in the air →
   his blood drips pools on to the floor below → she drops among them and Black-spikes the pool
   under him, or Harvests through them.
2. **Spend to swing.** A charged Blood nova shoves him off and costs red, so her grey climbs →
   the Reaping sweep at full size at the range he was shoved to.
3. **The return ticket.** The Blood jet leaves her pool where she took off → air game → blink
   back to her own pool.
4. **The trail and the chain.** Haemorrhage from the air → his bleed lays a trail → the Black
   spike on the nearest pool chains along it to him.
5. **Reel and raise.** Grasp hauls him to her feet → Marionette straight off the haul.
6. **The harvest run.** Harvest's spinning jump along the line that crosses the most pools.

**Hook is a grappling hook** (the person, 2026-10-09): thrown along the aim, it catches the first
thing it meets — a fighter, a creature, a stone or a wall — and **pulls her to it**. That puts it
in every combo: to a lifted or pinned body, to a pool's victim, out of trouble.

**Creatures bleed** (the person, 2026-10-09). Her hits on a creature spill a pool on the floor
under the part she struck, and the Haemorrhage's bleed runs on a creature as on a fighter. This
is the fix for `CLASS-2`: combos 1, 4 and 6 run on *their* blood, and "the creature does not
bleed" was the sentence that cost her most.

**The Bloodletter** is in no combo. It moves to `Q` for now, as the candidate to cut once she
has been played; the Black spike stays on `E`.

## The direction, from the person

**The three clicks are *my blood*, *your blood* and *utility*.**

- **Left — my blood.** Things she can do with no enemy at all, and that **make pools** from her
  own blood. This is the change that matters most: today her own blood never pools and a hit on
  somebody in the air spills nothing, so against a creature that does not bleed she has nothing
  to drink — the main reason she wins almost nothing in the hunts (`CLASS-2`).
- **Middle — your blood.** Things done to somebody else's blood.
- **Right — utility.** The scythe, and how she moves.

## The grid

| | Left: my blood | Middle: your blood | Right: utility |
| --- | --- | --- | --- |
| **On foot** | **Blood nova**: hold to charge a sphere of her blood that bursts and repels whoever is close | **Grasp** (as built) | **Reaping sweep** (as built) — *assumed, see the first question* |
| **In the air** | **Haemorrhage** (the built bolt and bleed) | **Nail** | **Hook** (proposed) |
| **Leaving the floor** | **Blood jet**: a held launch that propels her and hurts, more the longer she holds, to a cap | **Marionette**: lifts the target up into the air | **Harvest**: a big jump inside a wide scythe spin that drinks the pools it passes over |

### Left — my blood

- **Blood nova (on foot).** Hold to charge, release to burst: a sphere of blood around her that
  damages and **repels** — her first answer to being pressed. The charge is paid **in red health
  each frame held**, not up front, so the hold is paid for in blood as it goes. It leaves a pool
  of her own blood where she stood.
- **Blood jet (jump attack).** She leaves the floor on a jet of her own blood. The longer she
  holds, the further it propels her and the more it hurts whoever it passes, to a cap — and,
  like the nova, it costs red health per frame held. It leaves a pool of her blood where she
  took off. *Launch, do something, blink back to your own launch pad* is a loop no other class
  has.
- **Haemorrhage (aerial).** The built bolt and its bleed, from the air. An option to try: thrown
  steeply down, it splashes into a short rain of bleeding over an area.

### Middle — your blood

- **Grasp (on foot).** As built: hold for reach, four arms, all four to catch, hauled to her.
- **Marionette (jump attack).** On a hit, the victim's blood hauls them **up into the air**.
  **It only lifts — it does not pin.** (Decided 2026-10-09: the pin is Nail's.)
- **Nail (aerial).** A long black spike driven down onto somebody below. **Hitting someone in
  the air with Nail is what pins them** — held there at the point of the nail for a moment, on
  long black pins. So the pin takes two hits, Marionette and then Nail in the air, and she
  chooses when to spend each: Nail can also be thrown on its own, at somebody on the floor or in
  the air. *Proposal to check in play: while pinned in the air, the victim's bleed drips pools
  on to the floor below them, so she has somewhere to land.*

### Right — utility

- **Reaping sweep (on foot).** The scythe, as built (it grows with grey health).
- **Harvest (jump attack).** A big jump inside a wide scythe spin, which **drinks the pools it
  passes over** — picking up bloodstains becomes a movement skill. Proposal: only what the
  blade's arc actually covers is drunk, so the line she jumps along matters.
- **Hook (aerial).** A grappling hook: thrown along the aim, it catches the first fighter,
  creature, stone or wall and pulls her to it. Decided 2026-10-09.

## The rule her own blood needs

**Her own pools are doors, not heals.** A pool of her own blood can be blinked to, can make the
Black spike erupt, and can be used by the takeoffs; **only another body's blood heals her.**
Without this she can spend health, drink it straight back and loop. With it, *your blood*
remains the button that feeds her, and *my blood* is how she builds the field she moves on.
Proposed by Claude, accepted with the rest ("everything else sounds good").

**A hit on somebody in the air keeps its spill** and drops it where they land (0007's CORE-6).
Marionette and Nail both put people in the air, and without this every one of those hits would
spill nothing.

## Questions for the person

| ID | Question | Lean |
| --- | --- | --- |
| **BLOOD-1** | The note that set this direction put "the current grasp" on both middle and utility. This reads utility's floor move as the **scythe** and middle's as the Grasp. Right? | Yes |
| **BLOOD-2** | The Bloodletter and the Black spike lose their clicks. Where do they go — `Q` and `E`? Or does the Bloodletter go? | Bloodletter on `Q`, spike on `E` |
| **BLOOD-3** | How long does Nail's pin hold a body in the air, and can they airdodge out of it? | Short (well under a second), not breakable — it took two hits |
| **BLOOD-4** | The right-click aerial: Hook, or something else? | Hook |
| **BLOOD-5** | Her own pools are doors, not heals — or do they heal at a fraction? | Doors only |

## What building it touches (to be costed when it starts)

- Her move table grows from five to eleven or so; her dispatch (`state::blood_move`) learns
  where her feet are and the takeoff window (`Player::rise`, as the Elementalist's did).
- The pool rules: whose blood a pool is, that her own does not heal, and spills from airborne
  hits dropping where the victim lands. `tests/essence.rs` and `feel.rs` hold the economy.
- New effects: the nova, the jet, the pins. A body held in the air (Nail's pin) is new.
- Two held charges paid per frame in red health — new, since every cost today is paid on the
  press.
