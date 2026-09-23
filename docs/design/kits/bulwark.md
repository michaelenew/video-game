---
status: proposed
decided: 2026-09-09
depends: ../bulwark.md, ../defense.md
---

# Bulwark — kit

> ⚠️ **A v2 of the mechanic is built, 2026-09-23, and unplayed** — see [../bulwark-v2.md](../bulwark-v2.md)
> and [../plans/bulwark-v2.md](../plans/bulwark-v2.md). Every hit taken on the shield is stored as
> **weight**; Slam (on middle click) and Throw spend it, and a planted shield becomes a real
> structure sized by it. **Built so far (M1):** weight itself — loading, the drain, the
> pushback it resists, drawn on the shield — and the Bulwark's health, highest on the roster.
> See [Weight](#weight--built-m1) below. **M2:** Slam is on middle click and spends it — see
> [Slam](#slam--m-and-it-spends-the-weight). **M3:** a loaded throw is slower, harder and
> knocks down, and a planted shield is a real wall sized by its weight — see
> [Throw / Recall](#throw--recall--e-the-mechanic).

**Identity.** The wall. Wins by denying space and funnelling the opponent into where you are
already aimed. Committed, not slow.

Greenfield — no archive material. Read [../bulwark.md](../bulwark.md) for the class rationale
and [../defense.md](../defense.md) for the block system it is built on.

## Mechanic — the shield has its own position

Three states, and every ability reads differently in each:

| State | You | Shield |
| --- | --- | --- |
| **Held** | Protected in a facing arc, slow to turn | With you |
| **Planted** | Mobile, exposed, faster | Holding space where you left it |
| **Returning** | Can leap to it | In flight, damaging along its path |

**Traits:** reduced block stunlock, and a shield wide enough to cover an ally. Carrying it
costs movement speed — which you get back when it is planted.

**Two systems are meant to share the object.** As a *world volume* it stops projectiles by
collision, with no timing or state involved — which is why a planted shield would keep
blocking while you are elsewhere. As a *guard state* it handles melee, with stunlock and
parry timing. Only the second requires you to be holding it.

> **Both are built, since 2026-09-23 (v2, M3).** A planted shield is a solid in the *same*
> field as the Elementalist's stones — `stones::gather` puts it in its owner's first slot, a
> Bulwark having no stones of his own — so everything that walks the stones walks it: bodies
> stop against it and stand on it, the crosshair's ray and every shot meet it, the Reaver's
> dash is refused across it. It is sized by the weight it landed with, from 0.8 of a stone
> empty (0.56 m across, 1.4 m tall — shorter than a fighter) to 1.6 of one full (1.1 m, 2.8 m
> — taller than a full hop), and it holds that weight until it is recalled. Nothing moves it
> and Cataclysm does not break it: it is bypass-only. Drawn by the same code that draws the
> stones, from the same field, at the size it is tested at.

## Weight — built (M1)

**Every blow taken on the shield is stored in it**, in health: a blocked hit stores its damage,
a parried one `Weight, a parry loads` times that, up to `Weight, the most the shield holds`.
It drains on its own, a full shield emptying in `Weight, a full shield empties in`, so it is
about the current exchange — in hand and in flight, but **not planted**: a wall keeps the
weight it landed with until it is recalled (M3). It rides on every shield state, so a shield
thrown with three hits in it lands with them. All of it is `crate::bulwark`, and the
only place that loads it is the one place a blocked blow is resolved (`state::apply_hit`, plus
the Elementalist's poke), which is why the creature's blows load it with no code of their own.

**What it does today, unspent:** a heavy guard is pushed back less — blocked knockback times a
line from one at empty to `Weight, pushback at the cap` at full. That is the trait
[../defense.md](../defense.md) promised and never built.

**Drawn**: the shield thickens and darkens as it fills, and the HUD's mechanic line reads
`shield held, weight: N`. `cargo run -p sim --bin weight` prints what each class's opener and
each creature move deposits, the drain, and the shove at five weights; the frame table prints
the four knobs under the Bulwark.

**Health**: 1250 against everyone else's 1000, from the `Health` family in the Oven — one
multiplier per class, shared with the Reaver's v2.

**What spends it**: Slam (M2), and the throw on impact (M3). A throw that misses plants as a
wall sized by what it carried, and the wall keeps that weight until it is recalled.

## What is bound today

Three moves and the mechanic. The shared grammar puts the poke on the bare click, so Bash sits
one rung lower than the six-ability sketch below implies — and Slam sat on the modifier until
shift stopped modifying clicks on 2026-09-16, which left it with no input at all until it took
the free middle click on 2026-09-23.

| Input | Move | Bound |
| --- | --- | --- |
| `L` | **Bash** — the shield strike | yes |
| `M` | **Slam** — the shield into the ground, spending its weight | yes, since 2026-09-23 |
| `Q` | **Grapple** — the command grab | yes |
| `R` (hold) | **Guard**, with the parry in its opening frames | yes |
| `E` | **Throw** / **Recall** / **leap to it**, by shield state | yes |

`cargo run -p sim --bin frametable` prints the live numbers. There is **no separate off-hand
auto**: `L` is Bash, which is the shield itself.

Input map in [../controls.md](../controls.md).

## Auto attack

Intended as a short melee with the off hand — unremarkable on purpose, because the class does
not win by autoing. **Not built as its own move:** left click is Bash, so the poke and the
shield strike are one thing today. Whether the class wants both is open.

## Abilities

### Guard — right click, held
**Startup** instant · **Recovery** short · **Range** facing arc · **Mechanic** requires the
shield held

Hold to block. Blocked hits knock you back and stunlock you briefly; the Bulwark's stunlock
is short. **The opening frames are a parry** — timed correctly it staggers the attacker
instead. One input, two outcomes.

### Bash — `L`
**Startup** fast · **Recovery** short · **Range** melee · **Mechanic** requires the shield
held

A short forward shield strike. The safe poke, and the natural follow-up to a parry stagger.

> **Implemented**, on the bare click rather than on `shift` + click. 4/3/10, −4 on block and
> +2 on hit, and it is the one move in the game the repeat lockout actually charges: it costs
> 17 frames and the lockout is 30, so throwing it twice in a row waits 13. See
> [../combat-kernel.md](../combat-kernel.md) §"The repeat lockout".

### Slam — `M`, and it spends the weight
**Startup** slow · **Recovery** committed · **Range** short area · **Mechanic** requires the
shield held, and spends everything it holds

Drive the shield into the ground. Shakes the ground and staggers everything close.

> **Built, 2026-09-23 (v2, M2).** On middle click — it had no input from 2026-09-16, when shift
> stopped modifying clicks, until the proposal gave it the free third button. 14/4/24, and the
> frames never change with weight: a blocked full Slam is exactly as punishable as an empty one.
>
> - **Weight.** Damage is the move's 170 plus `Slam, damage per weight` of what the shield
>   holds; the shake's radius is the move's 1.4 m plus `Slam, shake radius added when full` in
>   proportion to how full it is. 170 over 1.4 m empty, 369 over 3.0 m full.
> - **The stagger.** From `Slam, staggers from` of the cap (90 %), whatever the shake catches
>   unguarded is staggered for `Slam, stagger at the cap` frames rather than put in hitstun.
>   Blocked, it is an ordinary blocked Slam.
> - **Spent** on its last active frame, landed or not. The shield does not drain while Slam is
>   out, so what the shake is worth and how wide it is drawn is what he had when he committed.
> - **The fall.** `Slam, damage per m/s fallen`, for the fastest he fell during the wind-up.
>   **Thrown in the air it lands with the feet**: once its wind-up is spent it waits for the
>   floor, the way Landfall does, so the shake is where the ground is and the fall is counted to
>   the end. A Slam pressed a frame before landing still owes its whole wind-up. Out of a full
>   jump, 260 against 170 standing; out of the leap, which is low, 194.
> - **A crouch does not duck it.** It was marked as an overhead (`hits_crouching` off) while
>   this document said otherwise; the shake goes through the floor, so the document won.
>
> `cargo run -p sim --bin weight slam` prints all of it.

### Throw / Recall — `E`, the mechanic
**Startup** instant · **Recovery** none · **Range** the throw's own reach · **Mechanic**
cycles the shield through its three states

One key, and which of three things it does depends on where the shield is.

- **Held → thrown.** It flies at what the crosshair is on — a skillshot, not a ray from the
  chest — damaging along its path, and plants where it lands, **standing on the floor** under
  where it stopped. You are now faster, exposed, and unable to block. **Loaded, it is a
  boulder** (M3): slower (`Throw, speed at the cap` ×0.6), harder (`Throw, damage per weight`
  ×0.5 on top of its own 85), and from half the cap it knocks down what it hits for
  `Throw, knockdown` frames. Striking somebody spends everything it carried, so it plants
  there empty; missing, it plants as a wall sized by what it still holds.
- **Planted → recalled.** It flies back, damaging everything along the return path — **once
  each, and on through them**. It used to plant where it struck on the way home too, which did
  not matter while a planted shield sat at hand height and passed over heads; on the floor, a
  recall through anybody never came back.
- **In flight → leap to it.** Pressed while it travels, you are thrown toward it **and it turns
  to meet you**, homing like a recall, so you arrive with it in hand and still in the air — which
  is what lets a Slam come out of the leap. Since 2026-09-23: before, the leap was slower than the
  throw and fell short of it every time, and every Bulwark move needs the shield in hand, so the
  slam out of a leap this document promised could not happen.

> **Implemented**, and **with no frames at all**. The kit above specified a startup and a
> recovery for Throw and Recall as though they were abilities; they are the class mechanic,
> and the mechanic fires on the press like every other class's. That is a real difference from
> the Reaver's Send shadow, which *is* a move in the table precisely because throwing a second
> body across the arena has a wind-up somebody can punish — see
> [../controls.md](../controls.md#where-e-is-an-ability). **Whether the shield should pay the
> same price is open**, and it is the one place this class gets something for nothing.
>
> The planting is built: see the note under *Two systems are meant to share the object*.

Recall is the class's mobility and its approach tool — throw to commit, leap to follow. It
sets up Slam: the leap brings the shield back in the air, and a Slam thrown there lands with
the feet and is paid for the fall. The leap is low, so the fall is worth little (about 4 m/s,
+24); a Slam out of a full jump is worth much more.

### Grapple — `Q`
**Startup** slow · **Recovery** committed · **Range** melee · **Mechanic** requires the
shield held

A command grab. Beats blocking outright, loses badly to dodge. Long startup and a punishing
tail — this is the payoff for a read on a turtling opponent, not something to throw out.

> **Implemented** (`Q`). It grabs: the victim is pinned at the Bulwark's arm's length and goes
> where he goes until the hold ends. Not hitstun with a longer timer — hitstun is something you
> recover from where you stand, and a grab is something that takes you somewhere.

## Playing it

Hold space with Guard and Bash, aimed so that the only open approach is the one you want
them to take. Throw and leap to close, land Slam out of the leap, Grapple when they start
blocking instead of moving.

In coop, plant the shield in front of the party as cover, or hold it wide to body-block for
someone who cannot take the hit. The planted volume is built (M3): a shield planted heavy is
cover taller than anybody, and one planted empty is a waist-high post. A blow taken on the
guard, then a throw that misses on purpose, is how the cover gets big.

## Open questions

- **Should the mechanic cost frames?** It costs none, which makes the Bulwark the one class
  whose mechanic press is free. The Reaver's is a move in the table for exactly the opposite
  reason.
- **Can the planted shield be destroyed, or only bypassed?** Bypassed, as built (M3):
  Cataclysm stops against it and does not break it. Revisit if a broken wall ever wants to
  spend the Bulwark's weight for him.
- Does the shield block allied projectiles? A real coordination mechanic in coop and
  potentially maddening.
- Does holding the shield wide for an ally need its own input, or is it just the Guard arc
  being large? Probably the latter — fewer inputs is better.
- Is six enough here? This class has the most state and may need a seventh for a dedicated
  ally-cover stance.
