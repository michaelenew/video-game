---
status: parked
parked: 2026-09-09
---

# Parked — progression and equipment

Deferred until the core classes are built. Nothing here is decided. It is recorded so the
reasoning does not have to be re-derived, and because two conclusions below are
load-bearing enough that reopening them later should be deliberate.

## Why parked

The classes are the product. Until the six of them exist and play well, a build system has
nothing to modify and no way to be evaluated. The reference games — Dark Souls, Monster
Hunter, League, Dota — share a shape worth patterning after: **a short early ramp to bring
new players in, then an effectively infinite skill ceiling.** That shape is compatible with
several different equipment systems, so choosing one now would be premature.

## The thesis it has to serve

**Player skill is the progression.** The long-term ambition is that skill is what unlocks
world — the better you are, the more of the game you can reach. Ability accumulates in the
player, not the character.

Two consequences follow, and both are firm enough to treat as constraints on whatever
build system eventually gets designed:

1. **Vertical character power is corrosive to the thesis.** Any character power that can
   overcome a wall means the wall is no longer a skill check, and grinding becomes the
   answer to every gate.
2. **Competitive versus and character progression are incompatible.** In a 1v1 arena, gear
   power *is* the equipment check the design is trying to avoid — new players would lose to
   loadout rather than skill, and matchmaking would have to account for gear. Dark Souls
   tolerates vertical progression partly because it is single-player with asymmetric PvP.
   This game has a real competitive mode.

Monster Hunter is the closest existing test of the thesis, with one asterisk: its gear
progression is genuinely vertical, but the ramp is *finite* and monsters do not scale, so
endgame builds converge and everything past that point is skill. A bounded ramp, then no
roof.

## Ideas worth keeping

### Equipment modifies the class mechanic, not individual abilities

The blocker on a TF2-style sidegrade system is combinatorial: TF2 works because a class has
one primary weapon and a few variants of it, whereas these classes have a dozen abilities
each, and authoring meaningful per-weapon variants of all of them is untenable.

The way out is to leave abilities alone and have equipment modify **the one central
mechanic** each class is built on:

| Class | Mechanic | What gear would tune |
| --- | --- | --- |
| Shadow Reaver | The shadow | Leash length, return speed, persistence |
| Elementalist | Structures | Size, count per cast, durability, lifetime |
| Statera | The meter | Safe-band width, auto pull strength, ability push force |
| Shifter | Weapon forms | Which forms are available, swap window length |
| Bulwark | The shield | Width, stunlock duration |
| Blood mage | Health cost | Health-to-power conversion, lifesteal efficiency |

One or two numbers each, propagating through the entire kit with no per-ability authoring.
A Reaver with a longer leash and a slower shadow return plays completely differently across
all twelve abilities.

It is also the axis where **sidegrades are easy to write.** "Longer leash, slower return"
is self-evidently a trade; making an ability variant that is different-but-equal takes
careful tuning every single time.

### A second layer of common-mechanic modifiers

Alongside the class-mechanic layer, modifiers on shared mechanics: dash startup and
duration, auto speed, movement speed, jump height, stun resistance (distinct from shield
knockback), incoming damage reduction.

Monster Hunter runs exactly this pair — universal armour skills plus weapon-specific ones —
which is evidence both layers are worth having rather than one crowding out the other.

### The slot budget is the mechanism, not the stats

Monster Hunter's decoration slots are scarce, so every build is an opportunity-cost puzzle.
This matters more than any individual stat: **without a tight budget, "which modifiers do I
want" collapses into "all the good ones" and the system contains no decisions.**

### Enabling stats over raw-power stats

The distinction between an interesting build stat and a boring one:

- **Enabling** — changes what you can *do*. Dash duration changes which attacks you can
  dodge through. Auto speed changes what fits in a punish window. Stun resistance changes
  whether you can trade through pressure. Jump height matters given the game's verticality.
- **Raw power** — changes only how much you can absorb or output. Incoming damage reduction
  is the clearest example: it does not change your options, it lets you be wrong more often.

Movement speed sits in between — enabling, but among the most powerful stats in any
fighting game, so small ranges only.

If damage reduction is wanted, make it **conditional or directional** — only from behind,
only while blocking, only above a health threshold — so it becomes a build shape that
rewards a playstyle rather than a slider that makes you tankier.

### Structural notes

- **Stats as class constants.** If there is no vertical progression, the seven stats
  (strength, agility, dexterity, affinity, attunement, toughness, fortitude) stop being a
  player-allocated build and become how classes differ in weight and speed — which is how
  fighting games do it. Equipment could *shift* them without raising the total.
- **The old weapon system is mostly the MMO treadmill.** Experience, evolution tiers,
  refinement, the three enchantment tiers, and stat multipliers are all vertical. What
  survives regardless is the shield as an equipped secondary, and weapon "size" affecting
  animation speed — though that reads as a class or form property, and the Shifter already
  uses it that way.
- **Versus and coop may want different gear rules.** Matchup-driven swapping is good in
  coop and bad in versus; raw-power stats are exactly what a competitive mode would ban.
  Probably one system with a competitive ruleset rather than two systems.
- **Six classes is a smaller variety problem than it looks.** Monster Hunter's variety comes
  mostly from having 14 weapon types, which map onto *classes* here, not equipment. But
  several classes contain internal modes that function like separate weapon types — fire
  vs ice vs earth Elementalist, the Shifter's three forms, the Statera's two poles, Reaver
  with and without the shadow. Counted that way the effective roster is nearer 12–15, and
  intra-class specialisation is a variety lever that costs nothing extra, since the
  ability-slotting system already decides which sub-identity is being played.

### Gating structure, if skill gates world

The risk of skill gates with no grind valve is that a player who cannot clear a gate is
simply stopped. Two mitigations, both already available:

- **Breadth at each tier.** Make the gate "beat N of these M fights" rather than one
  specific fight. Arenas are the cheapest content in the game.
- **Coop as the accessibility path.** Already in the design. This pairs well with
  horizontal equipment: when gear is a sidegrade the only assist is another player, which
  is social; when gear is vertical the assist is grinding alone, which undermines the
  premise.

## The bow

Proposed earlier as a universal secondary to fill the ranged hole left by retiring the
Gatekeeper. **Parked with the rest of the equipment question, and leaning against.**

The hole was a roster artifact rather than a design need — in a closed arena fighter,
"nobody has a long-range poke" is a legitimate choice, and the Elementalist already covers
mid-range. A universal weak poke also flattens class differentiation, which is the opposite
of what a fighting game wants, and it drags in an aiming system the game does not otherwise
need.

The secondary slot itself is not parked: the shield occupies it, and shield-or-nothing is
already a real choice. Adding a bow later is easy; removing one after six classes are
balanced around it is not.
