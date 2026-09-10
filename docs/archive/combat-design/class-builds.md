---
source_doc: "Class builds"
source_folder: "Game notes / Combat design"
drive_file_id: 1Qg1abYcCej8udl2hJhmdDHdzHTxvVMS4PZBbWOcBD20
drive_url: https://docs.google.com/document/d/1Qg1abYcCej8udl2hJhmdDHdzHTxvVMS4PZBbWOcBD20/edit
drive_modified: 2016-02-12T18:04:53Z
exported: 2026-09-09
---

# Class builds

## Current Shadow Reaver build

Highly tactical playstyle, positioning of the reaver and their shadow must be maintained constantly, as this positioning is what gives the Reaver combat mobility. High mobility and close quarters invisibility, coupled with high damage, are a strong combo, but gaining invisibility and damage limits the Reaver's mobility by making him move close to his shadow. Also, by putting some of his damage into his shadow we force the player to make a choice between damage and mobility, and make switching between damage and mobility a skill-based and natural part of the playstyle. This build might be better without invisibility or at least with very short invisibility, since the combat mobility of the reaver is so extreme.

### Shadow mechanics / theory

- All mobility skills require having the shadow out
- Most damaging skills allow you to send your shadow out
- The first AA after collecting your shadow deals extra damage based on dexterity
- MAYBE — collecting your shadow grants invisibility for 1.5 seconds
- Your shadow has a max range it can be away from you, if you exit this range it will dash to you, lightly damaging and slowing enemies along the way

### Skills

- **Reascend** — you slide quickly to your shadow, passing through enemies; you will dash slightly in the direction of your travel once you reach your shadow
  - Can cast shadow swap or otherwise manipulate your shadow during its motion
- **Guillotine lotus** — blades erupt from the Reaver's shadow, dealing damage; the blades return after a short delay or once the Reaver moves his shadow, dealing damage based on missing health
- **Shadow swap** — swap places with shadow if your shadow is away from you, detaches your shadow and goes on ¼ of normal cooldown if you have your shadow
- **Slash** — slashes in an AoE around you; if your shadow is out and you hit your shadow with the slash then the shadow will slash as well, dealing extra damage
- **Black trap** — places a trap on the ground, when triggered the trap will spring open and blades will spin around dealing damage; spins three times and each circle is bigger than the last; the blades cause a stacking slow
- **Executioner** — the Reaver blinks upward, then slashes downward for a ways; if the Reaver has their shadow the shadow slashes downward then dashes forward, dealing extra damage on the slash and bleeding on the dash
- **Razor shadows / Deadly mistake** — passive that causes bleed to be applied on enemies that attack the Reaver's shadow; activating causes the reaver to become invulnerable for a period of about 0.5 seconds; if the reaver has their shadow and is hit during the dodge period, their shadow will appear behind the enemy (obviously the shadow cannot appear outside of its max range)
- **Rifen** — your shadow grabs enemies, rooting them for a brief duration
  - This ability can be cast while your shadow is in motion to drag enemies with it; swapping with your shadow while the root is active switches the enemy's location too
  - Cannot cast a damaging shadow ability (eg executioner) while the root is in effect
  - Long cooldown
- **Night's veil** — causes your shadow to emit a time-delayed massive damage nuke and slow; very low range and about 3-4 seconds of arm time; arm time depletes to 0.5 seconds if your shadow passes through an enemy or if it is hit by an enemy
  - Disables shadow swap for the cast time
  - Shadow can be otherwise manipulated
  - This ability will hurt the reaver if it detonates in range of the reaver (this includes if the reaver has their shadow when it detonates)

## Current Statera build

### Resource system

- Casting abilities grants arcana which decays naturally over time
- At **<25%** resource the statera is considered *human*, casting abilities will restore health based on the resource they give
- At **>25% and <75%** resource the statera is considered *balanced*, giving them a 20% magic damage increase and 20% CDR
- At **>75%** resource the statera is considered *divine*, draining health on cast based on the ability cost and giving them 40% cdr and a 40% magic damage increase
- At **100%** resource the statera is *fully divine*, the statera's resource starts to rapidly deplete until it reaches 0, during which time they gain 95% cdr and casting abilities still costs health

### Skills

- **Tempest** — passively causes all abilities to mark enemies on hit, autoing a marked enemy will deal extra damage, consume the mark, and grant a short burst of movement speed

#### Rebuild

- Line skillshot on a low cooldown, velocity increases as the skillshot moves further and detonates once the casting key is released; deals low damage when it passes through an enemy and again when it detonates; if detonated in the vicinity of the statera it will hurt the statera and push them backwards
- Large AoE with medium range, slows enemies; detonating a mark of the merciful on an enemy in the AoE will root the enemy
- **Judgement** — time delayed AoE skillshot, massive instantaneous damage at the center after the delay and then a large AoE that deals low damage; high cost; the statera gains a speed boost and extra damage and lifesteal when moving through the AoE while the spell is in effect; the spell can only apply one tempest mark
- Line skillshot that moves rapidly forward with a small hitbox until it hits something or reaches its max range, then moves more slowly; damages first target hit; the statera can teleport to the projectile once it is in the second stage of its movement, dealing high AoE damage
- Snares all marked enemies in a small radius
- Four prongs that start out at the statera's location (like two pairs of open scissors); holding the ability button down moves the prongs outward radially and releasing causes them to swing inward; each prong deals independent damage
- Short range dash in any direction; the next autoattack casts a mid range trace that locks on if it hits a tempest marked enemy; the statera will dash to a target if the trace hits
- Very short range, fast moving line skillshot; travels forward a short distance damaging enemies and pushing them back then stops; the projectile then begins to move back to the statera, gaining speed; explodes when it reaches the statera or an enemy afflicted by mark of the merciful, damaging in an AoE and pushing away whoever it hits
- **Divide** — medium range dome that rotates into existence, dealing damage on the way up and blocking enemy projectiles once it is up; if the statera hits the dome with any other ability they will dash to where the ability hit the dome and keep momentum; the shield is only up for 2-3 seconds
- Cone that starts at the statera and extends outwards; staggers enemies that are afflicted by a mark
- Spawns three damaging motes that fly in the direction of the statera's next three auto attacks; these motes can detonate or apply tempest marks; charging an auto attack will charge all of the remaining motes, sending them all out and letting the statera dash a short distance upon release
- Very short range attack, two arms sweep down in a V formation; if an enemy has a tempest mark on it then they will be slowed; if the enemy is slowed they will be stunned; both arms deal damage independent of one another
- AoE that deals damage when auto attacks are cast

#### Other skills

- **Dark pulse** — sends out a pulse that hits enemies in range, dealing more damage to closer targets
- **Eclipse** — massive beam that casts for a fixed amount of time, dealing damage to targets in front of the statera; gives a stacking slow to those it hits
- **Light rift / Light prison** — first cast: marks an area in front of the statera's location that consistently deals AoE damage over time; second cast: roots all enemies in place within the area, ceasing all damage dealing
- **Judgement** — once cast marks an area which slows enemies, charge by holding for longer (charging increases slow amount and damage), releasing causes a slight delay then high damage in the selected area
- **Culling** — medium spherical AoE centering on the caster; caster cannot move for duration of cast; deals large damage over time and slows enemies within the AoE (slow greater closer to statera); lasts 4 seconds and will cancel if the statera takes damage
- **Chain of disruption** — fires a bolt that stops at the first enemy hit, detonating all Marks of the Merciful for AoE damage; all enemies hit by the AoE also have their marks detonated, and so on
- **Gathering power** — fires a single mote downwards in front of the statera, dealing damage; if the first cast hits then a counter is granted, giving increased damage and cost to subsequent casts of the ability; on the third hit the statera deals a massive AoE blow that staggers and deals damage
- **Raining power** — calls down motes from above in a line in front of the statera, first summoning a mote immediately in front then farther away then farther again; motes start out at a medium height and drop vertically, slowing those hit

## Blood mage skills

*Using a seal incurs a universal cooldown on all seals. Naturally deals increased damage on disabled enemies.*

- Low CD ability — projectile that goes out a fixed distance then comes back in, dealing damage on both passes; returns a portion of the damage dealt as health upon returning
- Short range skillshot, fires 4 arms (top left, bottom left, top right, bottom right) that all deal damage, initially fires in a cone outward but arcs inward to converge; enemies hit by all 4 arms are rooted for a brief duration
- **Rend** — fires a small, fast projectile that ignores enemies until reactivated; once reactivated the projectile slows down and grows rapidly, slowing and dealing low damage; the projectile grows slowly after the initial burst; the ability will, after 3 seconds, shrink back to its original size; if the blood mage is in the area when the projectile shrinks they will be healed
- **Black spike** — raises a spike at target area after a short delay; enemies hit by the spike are slowed and take damage, and all enemies in the vicinity are affected by a tether to the spike that drains health per second for 5 seconds and can be broken; the spike will return health to the caster after 5 seconds or once all bonds are broken based on damage dealt and grants movement speed on return
- **Cripple** — enemies in a small-medium range circle around the blood mage are slowed immensely, decaying over 2 seconds; does no damage but increases the mage's speed based on the enemies hit
- **Affliction** — slow moving, high cost projectile that can be reactivated to detonate or will detonate at the end of its range; detonating will mark enemies in the area of the projectile; marked enemies take increasing damage over time and after 4 seconds the mark will detonate, dealing a fixed amount plus 40% of the damage they took while marked; enemies that die spread the affliction to nearby foes
- **Reaper's debt** — starts channeling in a fixed direction, duration of channel increases width; cone in front of character that deals high damage to those inside when the channel ends
- **Murmur of the forgotten** — creates an AoE for 10 seconds that heals allies for a portion of the damage you do to yourself during the duration
- **Demon hunger** — initiates a chain of 3 detonations in a line in front of the blood mage, each one dealing increasing damage; the first three detonations will always happen, but if no enemy is hit on the first three the detonations start returning to the blood mage, if no enemy is hit by the fifth detonation the sixth occurs on the blood mage, dealing partial damage to the mage and full damage to all surrounding enemies
- **Seal of the unforgiven** — begins a channel that immobilizes the player for the duration; take 60% reduced damage, storing all of the blocked damage as energy; upon ending releases all stored energy on all enemies in a small-medium radius; max 3 seconds of channeling
- **Seal of the voracious** — increases damage but drains health per second
- **Seal of the forsaken** — 3 second prevention from taking damage (including self damage) (99% current health)

## Elementalist skills

### Fire

*Casting fire abilities gives stacking counters on the elementalist that increase the cost of all abilities. Fire abilities cause structures to explode.*

- Base mechanics
  - **Fire** → ranged fireball that deals damage
    - With structure → raises the stone into the air, fires meteor on click release
  - **Alt fire** → attempts to capture structures near mouse location at short range
    - With structure → slam the structure into the ground, staggering in melee range
- Lays a circular area that burns for a few seconds, dealing damage to nearby enemies
  - Drops the structure as a charged area that explodes after 5 seconds or on enemy contact
- **Fire pillar** — creates a focused pillar of fire at target location; small central pillar and moderate range AoE around it
  - High damage, central pillar staggers
  - Cast on whirlwind to create a firestorm
- **Flame spitter** — channels fire towards cursor location with high range
  - With structure → melts the structure into a magma field that damages and slows
  - Damage increases further out
- **Conflagrate** — hits enemies in a small circular area
  - Applies burn that deals damage based on current health

### Ice

*Icy areas provide a speed boost for the elementalist while in ice form and slow enemies.*

- Base mechanics
  - **Fire** → cast a single line skillshot that deals low damage
    - Powers up ice abilities it hits
  - **Alt fire** → attempts to capture structures near mouse location at short range
    - With structure → slam the structure into the ground, staggering in melee range
- **Frost sprite** — summons a frost sprite that puffs a mild healing AoE at the target area then disappears
  - **Frost sprite's domain** — cast the frost sprite while holding a structure and he takes up shop, creating a sentinel stone that heals allies and reflects projectiles; this will also take effect if a structure is in the area of effect of the frost sprite
  - Cast on an icy area to send the frost sprite into a territorial frenzy, dealing high damage over time and lowering enemy resistances
- **Crystallize** — short range icy stream from caster, leaves icy area and cost increases with duration
- **Whirlwind** — powerful suction AoE at target location, leaves an icy area, medium damage stationary tornado
- **Ice blast** — medium range, fast casting, cone skillshot; knocks back structures it hits, dealing extra damage and staggering enemies they hit
  - With structure → shatters the structure and sends out shards in a cone that damage and bleed

### Earth

- Base mechanics
  - **Alt fire** → attempts to capture structures near mouse location at short range
    - With structure → slam the structure into the ground, staggering in melee range
  - **Fire** → spawns a structure in low range at cursor location or max range (2s cooldown)
    - Cast beneath yourself to launch into the air
    - Cast on uncaptured structure to kick it forward through the ground, dealing low damage
- **Fissure** — skillshot that moves quickly forward through the ground to medium/long range and stops at the first enemy hit, staggering them; leaves an AoE slow where the skillshot traverses that lasts for 10 seconds and a structure at the location of the hit
- Spawns three structures in a line in front of the elementalist, dealing damage with each one
- **Quake** — small AoE (3m diameter); the area will immediately shake, causing any moving enemies to stagger and after a short delay the ground will erupt, dealing moderate damage to all in the AoE and leaving a structure in the center
- **Rock spike** — spike that raises from the ground, dealing damage before retreating
  - Cast on structure or with captured structure to send out a higher damage spike from the structure
