---
status: exploration
started: 2026-09-29
---

# 0001 — The control budget

*Where more spells can come from without the hand leaving the keys.*

## The question

Only `Q` and `E` cast abilities from the left hand, because moving takes five fingers there:
pinky on shift, ring on `A`, middle on `W`/`S`, index on `D`, thumb on space. The kits
specify six abilities plus an auto and the mechanic — eight inputs — and the archive's target
is twelve. Immediacy rules out moving the hand. What is left?

## The unit is fingers and frames, not keys

A keyboard has a hundred keys and the player can reach two of them. The right count is of
**fingers**, and of what each finger's key *means*:

| Finger | Resting on | Verb | Keys it can borrow without the hand moving |
| --- | --- | --- | --- |
| L pinky | shift | dodge | ctrl (crouch), tab, caps |
| L ring | `A` | strafe left | `Q`, `Z` |
| L middle | `W` / `S` | forward / back | `E`, `X` |
| L index | `D` | strafe right | `F`, `R`, `G`, `V`, `C` (crouch), `T`, `B` |
| L thumb | space | jump | alt |
| R index | left click | poke | — |
| R middle | right click | attack / guard | scroll click, scroll up, scroll down |
| R thumb | *nothing* | *nothing* | side buttons (`M4`, `M5`) |
| R ring, pinky | grip | — | side buttons on some mice |

Seven fingers are committed. **The right thumb is idle**, on a hand that already means
*where*.

Every input the game can accept falls into one of four prices, and the prices are the whole
argument:

1. **Free.** A finger that has no verb (the right thumb), or the *situation* (where your feet
   are, how deep in a string you are). Nothing is given up.
2. **One frame of one verb.** A committed finger reaches a neighbouring key, and for that
   frame its own key is up. `Q` costs a frame of strafe-left; `E` a frame of forward. This is
   the price the grammar already pays twice, and it is fine: a cast locks facing and hinders
   the walk anyway.
3. **A verb, for as long as it is held.** A modifier. The finger holding it cannot press its
   own key until it lets go.
4. **Every verb at once.** The hand moves. The number row, `T`, `Y`, anything past the index
   finger's reach. Not immediate, by the definition in the question.

There is a fifth, which is not a key at all: **time**. Tap-versus-hold as two different moves,
or a double tap, tells the moves apart by waiting, so the fast one arrives late by exactly the
waiting. The Blood mage's Grasp shows the only shape that keeps immediacy: the press fires the
move, and the hold is a *dimension* of it (range). A hold may set a number on a move that has
already started; it may never pick a different move. That is a rule the kernel can enforce
rather than a taste.

So "immediate" partitions cleanly: prices 1–3 are in, prices 4 and time are out. Everything
below is an inventory of prices 1–3.

## What is bound today, at those prices

Price 2 keys: `Q`, `E`. Price 1 buttons: left, right, middle click. That is **five** immediate
ability inputs on most classes (`L`, `R`, `M`, `Q`, `E`), which is where "two buttons" comes
from once the mouse is set aside. The Reaver and the Elementalist reach a sixth through shift
+ left, a leftover the 2026-09-16 rule says should not exist.

Not read at all: `F`, `R` (price 2, the index finger — the same price as `Q` and `E`), the
mouse side buttons (price 1), scroll up and down (price 1, but see below), alt (price 3).

## Four multipliers

### 1. More primitives — the cheap one

- **`F` and `R`.** Under the index finger, one row from `D`, exactly as `Q` and `E` sit one row
  from `A` and `W`. Same price, and the price was already judged acceptable. Every shooter and
  MMO layout uses them for this reason. Two new slots.
- **The mouse side buttons.** Under the idle thumb: price 1, the only price-1 keys in the
  building. Two new slots — and they are on the hand that aims, which matters below.
- **Scroll up and down.** Price 1, but a wheel *tick* is a poor edge: its timing is whatever
  the detent gives, and the controls document already distrusts the scroll click for exactly
  that. A wheel fits a choice that is not timed — cycling the Elementalist's element loadout
  when that exists, say — and not a cast. I would not count it toward spells.
- **`G`, `V`, `Z`, `X`.** Price 2 but a longer reach, and no shape a player already knows. Kept
  in reserve.

That takes the immediate set from five to **nine** without a modifier and without touching
the grammar's sentences: `L`, `R`, `M`, `M4`, `M5`, `Q`, `E`, `F`, `R`.

### 2. A modifier that has no verb of its own

Shift plus click was retired because shift also meant dodge, so what the key did depended on
what the rest of the hand was doing. Read exactly, the failure was **a modifier that was also
a verb**, not modification itself. A key that does nothing alone has one meaning when held —
"the other version of what you click" — and the 2026-09-16 objection does not apply to it.

The candidate is **alt**, under the left thumb. Its price is the thumb's verb, the jump, held
for as long as alt is. And a move forbids the jump for its whole duration anyway. So the cost
of holding the modifier is a verb the cast was about to take away regardless. That is the
elegant part: the modifier is free *because* commitment already costs the jump. What it does
cost is the few frames between pressing alt and clicking, in which you could have jumped and
chose not to. That is a commitment made a beat early, and it is invisible to the opponent.

Ctrl would work by the same argument (the pinky's verb is dodge, also forbidden during a
move), except that ctrl is crouch today. Alt is the one thumb key with no verb.

Alt over the five clicks is **five more** slots — fourteen — with the old "committed version
of this attack" row restored on a key that can carry it honestly.

Two cautions I have not tested. Alt is a browser and window-manager key: Firefox raises the
menu bar on an alt *release*, and the web build has to be checked with the pointer locked
before this is relied on. And a chord is a chord; the grammar's other precedent, space plus a
weapon on the Champion, was accepted because both keys were already under fingers. Alt is.

### 3. Context — the grid

The Champion's three buttons throw nineteen moves because the **situation** picks the row:
on foot, deep in a string, airborne, rushing, jumping. The Elementalist does it one step
smaller with an air row. This is price 1: no finger, no frame, no chord.

States that already exist in the snapshot and could be rows: on the ground or in the air;
crouching; chain depth; the mechanic's own state (shield in hand, planted, in flight; a
shadow out or home; a pool or a stone under the crosshair — `aim::pointing_at` already asks
this for the Reaver's dash). Each one doubles what a button can mean for free.

The catch is the one the ability spec flags: the count stops being the budget and
**readability per button** becomes it. A row the player did not choose is a surprise; a row
they did choose (they jumped, they planted the shield) is the move they meant. So the rule for
a row is that the state must be one the player put themselves in on purpose and can see.

### 4. Direction plus click

Already in the option table, already price 1. The unsolved part is not physical: while
walking forward there is no neutral click, so `W` + click must either be the forward version
or the plain one. Platform fighters on keyboards live with this (Brawlhalla: release the
direction for a frame to get neutral). It is a choice to make, not a slot to find, and this
note leaves it where the controls document has it.

## The count

| Step | Immediate ability inputs |
| --- | --- |
| Today (`L` `R` `M` `Q` `E`) | 5 |
| + `F`, `R`, and the two side buttons | 9 |
| + alt over the five clicks | 14 |
| × an air row on the buttons that want one | up to 28 |

The kit target is eight, and the eventual twelve. **Nine unmodified immediate slots already
cover eight**, with one spare. Twelve needs the modifier *or* the air row, not both.

## What actually binds

Not the hand. The controls document's "do not fill all twenty" was right for a reason the
counting makes exact: the physical ceiling is well above the readable one. A spell has to be
read in third person from across an arena and learned from a table, and neither of those
costs scales with keys.

So "a lot more spells" should be bought as **depth** — the same buttons meaning more by
situation — before it is bought as breadth, because depth is the only multiplier that costs
nothing at the hand and keeps the vocabulary small. That is the input-side answer to the
ability spec's open question, "a kit is six abilities or a kit is a grid": from the hand,
it is a grid.

## Where the new primitives should mean something

The grammar's strength is that each region has one job on every class. The two new pairs
divide naturally:

- **The side buttons are on the aiming hand**, so they should carry *aimed* things, by the
  same sentence that put the Reaver's shadow on right click. The Elementalist's Fissure — a
  ground skillshot with no button since 2026-09-16 — is the obvious first tenant.
- **`F` and `R` are on the moving hand**, so they should carry *unaimed* things: swings off the
  body, state changes, seals. The Blood mage's unplaced seal and the Reaver's Deadly mistake
  want checking against this, and I have not read those kits closely enough to place them.

A keyboard stand-in per new mouse button, as `J`, `K` and `U` stand in for the clicks, keeps
trackpads and two-button mice playable.

## A controller reads the same way

The same partition holds with different fingers: both thumbs are committed (move, camera), so
the four shoulder buttons and triggers are the price-1 set, the face buttons cost the camera
for a frame (price 2), a held trigger is the verbless modifier (price 3), and context is still
free. Four plus a modifier plus rows is the same shape as nine plus alt plus rows. That is
mild evidence the partition is about hands rather than about one keyboard.

## Next

1. Read the side buttons and `F`/`R` in `read_input`, with stand-ins, and give each a bit.
   Four bits are free in the input word (`u16`, twelve used; the quarter-turn constant beside
   them is an aim unit, not a button).
2. Test alt under pointer lock in the browser build before designing on it.
3. Decide the region meanings above and rehome Fissure first, since it is the only stranded
   move whose home the rule picks unambiguously.
4. Then the ability spec's grid question, from this side.
