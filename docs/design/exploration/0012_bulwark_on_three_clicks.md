---
status: decided 2026-10-10 by the person's brief ("do the same"), designed and built by Claude; unplayed
started: 2026-10-10
supersedes: kits/bulwark.md §"What is bound today" where they differ; 0007 §7 where they differ
depends: 0007_core_kits.md §7, ../kits/bulwark.md, ../bulwark-v2.md
---

# 0012 — The Bulwark on three clicks

## The brief

The same treatment the Elementalist, the Blood mage, the Dual mage and the Shadow Reaver got:
the Champion's grid — three clicks, each with a move on the floor, an aerial and a jump attack
— designed **combos first**, each click a genre, the keys carrying the majors. What he keeps is
everything that makes him the class: the shield and its three states, the weight it stores,
the Slam that spends it, the throw, the recall and the leap, the Grapple.

**What he wants from the air is different** from the four before him (0007 §7). He is the
heaviest class and stays the heaviest: his air game is short and violent and about **being
thrown** rather than floating. The shield is his springboard, his sail and his battery.

## The combos first

His loop is already written down: **take it, store it, spend it.** A blow taken on the
shield is weight; Slam and the throw spend the weight. Every combo below puts the weight
somewhere new.

1. **Load and unload.** Guard through a string and load the shield → space and middle click,
   **Unload**: the shield driven into the floor at his feet, and the weight throws him upward,
   higher the more it held → **Slam** from that height, paid for the fall. One battery, spent
   twice: on height now, and on the speed the Slam lands with.
2. **The ram.** Space and left click, **Battering ram**: a low, flat leap forward, shield first,
   carrying whoever it meets → into the wall behind them → Bash, or the Grapple.
3. **Rebound.** In the air, left click is a Bash that meets **anything solid** — a body, a
   wall, a stone, his own planted shield — and throws him back off it, up and away. His wall
   jump, and his only way to stay up: a heavy man bounces off things rather than floating over
   them.
4. **Step and recall.** Space and right click, **Shield step**: the shield planted where he
   stands, and he springs off its top — higher the heavier it is. It stays behind as a wall → `E`
   in the air calls it home, hitting whoever is in the way, into his hand → Slam on the way down.
5. **Sail.** Guard held in the air: he falls slowly behind the shield, and a blow he blocks up
   there is still weight. Over a gap, under a volley, and down onto whoever threw it.
6. **Throw, leap, Slam.** As built: `E` throws, `E` again leaps to it in flight, and a Slam out of
   the leap lands with his feet.

## Three genres

| Click | Genre | What every move in it does |
| --- | --- | --- |
| **Left — the strike** | The shield as a weapon | Fast, short, off the front of the shield. Its aerial bounces him off what it meets; its takeoff takes them with him |
| **Middle — the weight** | Where the battery goes | Slow and heavy, and every one of them is the stored weight spent: on a shake, on a fall, on height |
| **Right — the guard** | The shield as a thing | Hold it, sail on it, stand on it. Nothing in this column is an attack |

## The grid

| | Left: the strike | Middle: the weight | Right: the guard |
| --- | --- | --- | --- |
| **On foot** | Bash | Slam | Guard / parry |
| **In the air** | **Rebound** | Slam (as built: lands with the feet) | **Sail** (guard held in the air) |
| **Leaving the floor** (space + click) | **Battering ram** | **Unload** | **Shield step** |

| Key | Move |
| --- | --- |
| `Q` | Grapple, as built |
| `E` | Throw / recall / leap to it, as built — and in the air it is the second half of the Shield step |

### Left — the strike

- **Bash.** As built.
- **Rebound** (air). The Bash thrown in the air. If the shield's front meets anything solid
  within its reach — a body, a wall, a stone, his planted shield, the floor if he points down —
  he is **thrown back off it**, away from what he struck and up. It hits whoever it met. Paid
  for by being next to something.
- **Battering ram** (space + left). A low, flat leap forward with the shield in front: he goes
  along the floor rather than up, and whoever the shield meets is **carried with him** — the
  knockback is his own speed, so they travel together.

### Middle — the weight

- **Slam.** As built.
- **Slam in the air.** As built: once its wind-up is spent it waits for the floor, and the fall
  is counted.
- **Unload** (space + middle). The shield driven into the floor at his feet: a small shake
  round him, and **the stored weight throws him upward** — `Unload, lift empty` with nothing in
  the shield, plus `Unload, lift added when full` in proportion to how full it is. It spends
  everything. Empty, it is a stomp and barely a hop.

### Right — the guard

- **Guard / parry.** As built: right click held.
- **Sail** (guard held in the air). Not a move: the guard, which he could already hold in the
  air. Held there, he falls no faster than `Sail, falls at most` — the shield is a sail. Every
  block is still weight.
- **Shield step** (space + right). The shield is **planted where he stands**, at the size its
  weight makes it, and he springs off its top — `Shield step, lift empty`, plus `Shield step,
  lift added when full` in proportion — and on forward. (Planted a step in front, as first
  written, he sprang into its side and never got over it.) He has no shield in hand
  until he calls it home with `E`. That is the price, and the recall that answers it is a hit.

## What the build touches

- His table grows from three rows to seven: Rebound, Battering ram, Unload, Shield step,
  appended after the Grapple.
- The shared takeoff window (`Player::rise`). For him, space in the air is only a jump.
- Rebound's bounce, asked of the same line the Dual mage's Flare asks (`first_on_the_line`).
- The Sail is a fall cap on the guard in the air; the Shield step plants the shield through the
  same `Shield::Planted` a throw that missed lands as.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **BULWARK-1** | Does Rebound off his own planted shield make a loop that never comes down? | Each Rebound has its startup and recovery, and the bounce is a fixed speed; gravity wins over three |
| **BULWARK-2** | Is Unload from a full shield too much height for the heaviest class? | Full is about three of his jumps; `Unload, lift added when full` is the knob |
| **BULWARK-3** | Should the Shield step's wall be smaller than a missed throw's? | The same size: what the weight makes it |
| **BULWARK-4** (0007's CORE-7) | Shield step leaves him with no moves until he calls it home. A price or a trap? | A price; the recall is a hit |
