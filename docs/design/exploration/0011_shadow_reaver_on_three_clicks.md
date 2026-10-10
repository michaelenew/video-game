---
status: decided 2026-10-09 by the person's brief ("do the same"), designed and built by Claude; unplayed
started: 2026-10-09
supersedes: kits/shadow-reaver.md §"What is bound" where they differ; 0007 §6 where they differ
depends: 0007_core_kits.md §6, ../kits/shadow-reaver.md, ../shadow-reaver-v2.md
---

# 0011 — The Shadow Reaver on three clicks

## The brief

The same treatment the Elementalist, the Blood mage and the Dual mage got: the Champion's grid
— three clicks, each with a move on the floor, an aerial and a jump attack — designed **combos
first**, each click a genre, the keys carrying the majors. What she keeps is everything that
makes her the class: the shadow, its copies, the tally and the cash-in, the dash to the shadow
and the jump out of it, the lotus and its drag.

**What changes:** middle click, empty since the class was built, takes the **Executioner**,
which leaves `E` free at last for **Deadly mistake** — the counter stance her kit has listed
without a button since 2026-09-09. And six new moves fill the air and the takeoffs.

## The combos first

Her loop is already written down: **send, mark, cross, cash.** The shadow's copies put marks
on a body, her own hit spends them, and the dash to the shadow is how she gets from where she
marked to where she cashes. Every combo below is that loop with a new piece in it, and each
piece stands alone.

1. **The cut from both sides.** Send the shadow beside him → Slash: the copy marks him from
   over there → dash to the shadow → the Executioner on arrival cashes the tally.
2. **The kite.** Moonsault (space + left) launches him as she flips up and back → Kite cut on
   the **marked** body in the air gives her airdodge back → the airdodge aimed at the shadow is
   the dash → dash, and cash on arrival. A hit on a marked body is what pays for the next dash.
3. **The gallows.** Gallows (space + middle): she is gone upward, hangs a beat, and comes down
   blade first where she was looking. Out of the air, the Guillotine drop: straight down,
   spiking whoever is in the air under her into the floor.
4. **Hang and fly.** Hang the shadow (space + right) up in the sky where the crosshair is →
   the dash goes up to it → the dash jump leaves from the sky. Her biggest movement, made
   vertical, and paid in aim: the shadow has to be placed, and the jump pressed in the dash's
   last frames, as it already is.
5. **The swap.** In the air with the shadow waiting on the floor, right click **trades places**:
   she is where the shadow was and it is up where she was. Whoever jumped to catch her is
   swinging at a shadow.
6. **Deadly mistake.** `E`: a short counter stance. Struck in it, she is behind the attacker and
   the shadow is left where she stood — in front of him. The next Slash lands from behind, and
   its copy from in front.
7. **The drag.** As built: `Q` opens the lotus at the shadow, right click on the floor drags it
   home through everything between.

## Three genres

| Click | Genre | What every move in it does |
| --- | --- | --- |
| **Left — the blade** | Quick cuts off her own body | Fast, copied by the shadow, the move the tally is built on. Its aerial refuels the dash |
| **Middle — the execution** | Committed cuts from above | Slow, overhead, the ones that cash a tally for the most; they come *down* onto a body |
| **Right — the shadow** | Where the other body is | Placement and movement: send, recall, trade, hang. Nothing in this column is a swing |

## The grid

| | Left: the blade | Middle: the execution | Right: the shadow |
| --- | --- | --- | --- |
| **On foot** | Slash | **Executioner** (moved from `E`) | Send / recall |
| **In the air** | **Kite cut** | **Guillotine drop** | **Swap** (send, if the shadow is with her) |
| **Leaving the floor** (space + click) | **Moonsault** | **Gallows** | **Hang the shadow** |

| Key | Move |
| --- | --- |
| `Q` — the shadow's major | Guillotine lotus, as built |
| `E` — her own | **Deadly mistake**, the counter stance |

`shift` + forward with the crosshair on the shadow is still the dash, and space in its last
frames is still the dash jump.

### Left — the blade

- **Slash.** As built.
- **Kite cut** (air). The Slash rolled into a vertical arc. On a hit against a body that was
  carrying marks, **her airdodge comes back** — and her airdodge pointed at the shadow is the
  dash. The refuel is her own loop: send, mark, cut.
- **Moonsault** (space + left). A back flip: she goes up and back, and the blade comes up
  through the space in front of her, **launching** whoever it catches. An escape that punishes
  whoever was pressing her, and the opener of the kite.

### Middle — the execution

- **Executioner.** As built, on middle click.
- **Guillotine drop** (air). She drops straight down blade first; a body in the air under her
  is **spiked** into the floor.
- **Gallows** (space + middle). Her source material's "blink upward, then slash down": she is
  gone upward a few metres, hangs a beat, then plunges forward and down and the blade lands
  where her feet do. Which way she comes down is the defender's problem.

### Right — the shadow

- **Send / recall.** As built.
- **Swap** (air). With the shadow out and waiting, she and it trade places. With the shadow at
  her shoulder, it is the send, as before.
- **Hang the shadow** (space + right). The send, to the crosshair's point **in the air**: it
  waits there a moment, then sinks to the floor under it. She jumps as she throws it.

### `E` — Deadly mistake

A brief counter stance. **Struck during it** by a fighter's blow, she takes nothing: she is
behind the attacker, facing him, and her shadow is left waiting where she stood. Whiffed, it is
a long recovery. The archive's passive (attacking her shadow makes you bleed) is not built.

## What the build touches

- Her table grows from four rows to eleven, the new seven appended after the four that exist.
- The shared takeoff window (`Player::rise`) — space in the air is only a jump on this class.
- A shadow that can hang in the air (`Shadow::hang`, `Shadow::rest`), and sink when its time is
  up.
- Gallows' blink and the drops' dive; the counter in the hit loop; the airdodge refund in the
  cash-in.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **REAVER-1** | Is leaving from a hung shadow too much reach? | The dash jump keeps all of the dash, as on the floor; `Hang the shadow, waits for` is the knob |
| **REAVER-2** | Is the Kite cut refuel a loop that never ends? | Once per airdodge: the refund needs a marked body, and the hit spends the marks |
| **REAVER-3** | Does Deadly mistake want to work against a creature's blow? | Fighters only, for now |
| **REAVER-4** | Should Swap be the recall in the air rather than a third verb? | Swap; the recall is on the floor |
