# Design — current state

Everything currently decided, proposed, or parked, in one place. This supersedes
[`../archive/`](../archive/README.md), which is 2016–2019 source material kept for reference.

**Start here:** [combat kernel](combat-kernel.md) → [controls](controls.md) →
[ability spec](ability-spec.md) → a class kit. For implementation, see
[architecture](architecture.md).

---

## 1 · Settled

**Frame.** Peer-to-peer, isolated battle arena. Coop against monsters, or versus. 3D third
person, with ability spectacle as an explicit goal. Smash-like — spatial commitment, whiff
punishment, reads — not Tekken-like. The closed arena is deliberate: far less art than an
open world.

**No cooldowns.** Abilities cost **frames** (primary) and **the class mechanic** (secondary).
There is no universal resource bar.

**Time to kill.** ~60 seconds versus. 1–20 minutes coop, by fight difficulty.

**Defense splits into two verbs.** Dodge is universal and evasive. Block is shield-gated and
positional, covering a facing arc rather than a bubble. Blocking costs **space and a
vulnerable window** — knockback plus stunlock, no chip damage, no guard meter. Parry is the
opening frames of block and rewards with a stagger. Special attacks are the guard breakers.

**Control grammar.** Click = attack. Shift = ability. WASD = move. Space = move more. Shift
beats WASD when both are held.

**Roster.** Six classes. The Gatekeeper is retired and not backfilled — a missing long-range
poke is a design choice in a closed arena, not a gap.

## 2 · The roster

| Class | Mechanic — what abilities spend | Primary buttons | State |
| --- | --- | --- | --- |
| [Shadow Reaver](kits/shadow-reaver.md) | Shadow position | `L`/`R` melee autos | Strong |
| [Elementalist](kits/elementalist.md) | Structure slots (cap 3) | `L` bolt · `R` Raise | Strong |
| [Blood mage](kits/blood-mage.md) | Health | `L` auto · `R` Rend | Decent |
| [Dual mage](kits/dual-mage.md) | Meter position | `L` dark auto · `R` light auto | Reworked |
| [Bellator](kits/bellator.md) | Rush charge, and which form you end in | `L`/`M`/`R` = sword/hammer/spear | Reworked |
| [Bulwark](kits/bulwark.md) | Shield position | `L` auto · `R` Guard · `M` Throw/Recall | New |
| ~~Gatekeeper~~ | — | — | Retired |

*Dual mage was Statera. Bellator was Shifter.*

Each kit is **six abilities plus an auto and the mechanic input** — enough for a real match,
few enough to balance and to read in third person.

## 3 · Documents

| Document | Covers | Status |
| --- | --- | --- |
| [combat-kernel.md](combat-kernel.md) | No cooldowns, TTK, what that breaks | Decided |
| [controls.md](controls.md) | Input grammar, per-class schemes | Proposed |
| [ability-spec.md](ability-spec.md) | The format kits are written in | Proposed |
| [defense.md](defense.md) | Dodge, block, parry, guard breaks | Proposed |
| [dual-mage.md](dual-mage.md) | The two-pole meter and ascension | Decided |
| [bellator.md](bellator.md) | Forms and the mid-animation swap | Decided |
| [bulwark.md](bulwark.md) | Why the class exists; shield as volume | Proposed |
| [elementalist.md](elementalist.md) | Structure interaction in versus | Decided |
| [gatekeeper-retirement.md](gatekeeper-retirement.md) | Why it was cut, what was salvaged | Decided |
| [architecture.md](architecture.md) | Rust workspace, determinism, rollback | Decided |
| [parked.md](parked.md) | Progression and equipment | **Parked** |

## 4 · Open

Nothing here blocks a prototype.

| Item | Question |
| --- | --- |
| **Arena size and shape** | Determines whether a space-denying class can corner anyone, and whether block pushback has teeth |
| **Frame counts and damage** | Absent everywhere on purpose. Needs a prototype, not a guess |
| **`M` and `LR` reliability** | They carry the Dual mage's finishers and are the slowest inputs on most mice |
| **Airborne movesets** | Prototype differentiates directional basics only |
| Dual mage | Naming the two forces. Ascension drain, refund, threshold and stun numbers |
| Bulwark | Possibly a seventh slot for a dedicated ally-cover stance |
| Bellator | Whether the mid-animation swap costs Rush |
| Shadow Reaver | Whether the shadow has collision |
| Elementalist | Structure cap of three is a readability guess, not a balance one |
| Blood mage | Health cost flat or percentage |

## 5 · Parked — not slated for initial implementation

**Progression and equipment.** See [parked.md](parked.md). The classes are the product; a
build system has nothing to modify until they exist.

The leading proposal is **offensive items as auto-attack modifiers** — mild buffs that change
playstyle rather than power, letting a class spec toward offence, defence or utility without
changing identity, with unlocks forming the early ramp. Its open risk is that some modifiers
may be too central to be optional.

Two conclusions there should only be reopened deliberately: vertical character power is
corrosive to a skill-gates-content thesis, and competitive versus is incompatible with
character progression.

## 6 · Implementation

Rust, three crates, simulation as a pure function. See
[architecture.md](architecture.md). The scaffold builds and its determinism and
rollback harnesses pass; combat content is a stub on purpose, so every class
implemented from here is determinism-checked from its first commit.

```
crates/sim    Deterministic simulation. Zero deps, no floating point.
crates/net    Rollback session. Shaped like the GGRS handler.
crates/game   Front end. Headless soak today, Bevy later.
```

## 7 · Next

1. **One real class in `sim`** — where `slow` and `committed` become frame counts.
2. **Capsule collision and hitboxes.**
3. **Bevy front end**, capsules and debug hitboxes only, no art. Where feel gets tuned.
4. **Arena size and shape.**
5. Per-class open items above, most of which want the prototype anyway.
