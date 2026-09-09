---
source_doc: "Weapons"
source_folder: "Game notes / Combat design"
drive_file_id: 1NmdTvIDK8QDLUX7adRnHEtuHC68c5VvnT7bRFkQjzMM
drive_url: https://docs.google.com/document/d/1NmdTvIDK8QDLUX7adRnHEtuHC68c5VvnT7bRFkQjzMM/edit
drive_modified: 2016-01-20T17:07:08Z
exported: 2026-09-09
---

# Weapons

- Weapons each have a "size" rating that corresponds to the speed of their animations (great swords swing slower than daggers). This extends to cast times.
- Weapons have multipliers that are attached to them that augment the user's character abilities (a staff could have a multiplier of 3 and a character magic attack could be 50, so the effective character magic damage would be 3\*50=150). Dual weapons count separately for each auto and stack multiplicatively when casting.

## System

Weapons have stats associated with them. These stats dictate the behavior of the weapon. Some stats can be changed, others are fixed.

- **Experience** — increases based on how much damage the weapon deals. May be hidden from the player, not sure yet
- **Weight**
- **Length (reach)** — fixed
- **Length damage function** — fixed (determines things like sweet spot, do you want to hit with the handle or tip, etc)
- **Strength multiplier** (most weapons have a natural nonunity strength multiplier)
- **Dexterity multiplier** (most weapons have a natural nonunity dexterity multiplier)
- **Affinity multiplier** (most weapons naturally have a unity affinity multiplier)
- **Attunement multiplier** (most weapons naturally have a unity attunement multiplier)

Weapons have multipliers. Multipliers act with a specific stat (eg a strength multiplier) to increase the damage of the weapon. Higher multipliers cause more damage scaling with the specified stat.

Weapons have different multipliers with different offensive stats naturally. Refinement can be used to reallocate weapon stats by removing a chunk of exp. Note that the weapon will NOT grow in absolute power by refining, only have its stats rearranged to suit the character better.

A weapon will "evolve" after receiving a certain amount of exp, depending on the weapon level and player choices. If weapons are refined in certain manners they may gain passives when evolving.

Weapons can be enchanted once per evolution. Refining a weapon opens up different enchantments for that weapon. Letting a weapon evolve without enchanting it will give it a stat boost with higher stats for higher evolutions.

Weapons can be reforged. Reforging a weapon makes it into a class weapon and gives it specific bonuses/effects (eg anything a blood mage reforges becomes a reaping \_\_\_\_\_\_ and has lifesteal).

## Weapon types

- **Dagger** — light weight; naturally high dexterity multiplier
- **Sword** — medium/heavy weight; varies, naturally has dexterity and strength multipliers
- **Shield** — light/medium/heavy weight; equip as secondary to give defense and gain stagger resistance while casting
- **Spear** — medium weight, high attack range; sweet spot at tip; same stat distribution as sword
- **Glaive** — heavy weight, gains increased attack range; sweet spot at tip; more strength than dexterity
- **Hammer** — heavy weight; defensive bonus with high strength multiplier

## Enchantments

### Evolution 1

- Gives an aura that deals damage over time to enemies close to you
- Gives increased movement speed
- Reduces cast animation times
- Taking damage gives you increased damage for 2 seconds
- Gives further increased physical damage
- Gives further increased magic damage
- Increase damage vs slowed enemies
- Increase damage when attacking from behind
- Autos decrease physical armor
- Abilities decrease magic resistance

### Evolution 2

- **Overheal** — reduces base health by 35% but allows for overheal from any healing source, overheal decays over time but can go up to 140% of your base health
- Taking damage reduces all cooldowns based on the percentage of your current health lost
- Each auto summons a fireball that falls in the location of the struck enemy
- Dealing damage with an ability causes the next autoattack within 5 seconds to put a DoT debuff the monster, dealing an additional 10% of the ability's nominal damage
- Dealing damage fills up a bar on the weapon that depletes over time, healing the player while it depletes
- **Flight** — taking damage increases movement speed
- Some of damage dealt is returned as a shield which quickly decays
- After taking a certain amount of damage a minion will spawn which follows the player and attacks the last enemy the player hit; the first attack of these minions will stagger
- Charging attack — uppercut that increases in damage, stagger strength, and height jumped with charge
- Charging attack — drive forwards; reduced movement speed while charging; range and damage increase with charge

### Evolution 3

- **White dervish** — hitting an auto attack grants greatly increased movement speed that falls off quickly; autoing in the air makes the character stay suspended for a moment; autos have an AoE component that deals magic damage based on weapon strength
- **Red dervish** — all abilities on hit will deal an extra 15% of the ability damage over time; taking damage causes an explosion centered on you, dealing damage to nearby enemies, explosion has a 15 second cooldown; -50% incoming healing; -80% resource regen; returns a small portion of the damage done as resource
- While in the air, jumping again will dash in the direction the character is moving
- After filling up a bar based on how much damage the weapon does a seed is dropped a short distance from the player that summons a plant which slows nearby enemies and damages them slightly, healing for some of the damage done
- **Ravenous** — dealing damage unleashes the hunger of your weapon; the weapon must consume a certain amount of health every second until sated; if the player does not deal enough damage to enemies the remaining health will be subtracted from their health; once sated through enough damage being dealt the weapon grants cooldown reduction and increased physical and magic damage
