---
source_doc: "Shadow Reaver skills"
source_folder: "Game notes / Combat design"
drive_file_id: 1g8DiH0eDF0VbPJj-dLQ3UfB3HXX606HYSXAQq3vleg8
drive_url: https://docs.google.com/document/d/1g8DiH0eDF0VbPJj-dLQ3UfB3HXX606HYSXAQq3vleg8/edit
drive_modified: 2016-04-21T03:14:48Z
exported: 2026-09-09
---

# Shadow Reaver skills

- **Shadow dash** — short range dash in any direction, hitting an enemy will cancel the rest of your dash; at the end of your dash a shadow continues forward for a short distance dealing damage; reactivate to switch with shadow
  - REVISION 1: Shadow dash — short range dash in any direction, hitting an enemy will cancel the rest of your dash; if you have your shadow then it will continue forward at the end of your dash
- Causes you to spawn a shadow on top of yourself and grants a burst of movement speed
- Passively marks enemies when you hit them, max 6 marks per enemy; activating deals damage to marked enemies based on the number of marks they have
  - REVISION 1: Passively marks enemies when you hit them, max 6 marks per enemy; passing your shadow through the enemy detonates the marks and deals damage
  - I like this type of move better on the statera, it's a more reactive type of move
- **Razor shadows / Deadly mistake** — passive that causes bleed to be applied on enemies that attack the Reaver's shadow; when the Reaver has their shadow, activation causes the Reaver to enter a counter stance for a short time, taking damage during this time will cause the Reaver to appear on the other side of the enemy hit, leaving their shadow behind
- **Reascend** — returns all shadows to the Reaver slowing and damaging all targets hit
  - REVISION 1: Reascend — returns shadow to the Reaver, slowing and damaging all targets hit; the reaver will dash in the direction the shadow comes from for a short distance based on how many enemies it hit (the more enemies you hit with your shadow the shorter the dash)
  - REVISION 2: Reascend — you slide quickly to your shadow, passing through enemies; you will dash slightly in the direction of your travel once you reach your shadow
    - Can cast shadow swap or otherwise manipulate your shadow during the cast
- **Executioner** — spawns a shadow a bit in front of the reaver that slashes downward then dashes forward, dealing high damage on the slash and low on the dash; reactivate to swap places with shadow
  - REVISE TO: Executioner — the Reaver blinks upward, then slashes downward for a ways; if the Reaver has their shadow the shadow slashes downward then dashes forward, dealing extra damage on the slash and bleeding on the dash
- **Black lotus** — spawns a shadow at cursor location that has daggers explode from it radially, then pause in midair; daggers return to the shadow after 2 seconds, dealing damage again; reactivate to swap places and cause the daggers to return early
  - REVISION 1: Guillotine lotus — blades erupt from the Reaver's shadow, dealing damage; the blades return after a short delay or once the Reaver moves his shadow, dealing damage based on missing health
- **Black trap** — places a trap on the ground, when triggered the trap will spring open and blades will spin around dealing damage; spins three times and each circle is bigger than the last; the blades cause a stacking slow
- **Shadow swap** — swap places with shadow if your shadow is away from you, detaches your shadow and goes on ¼ of normal cooldown if you have your shadow
- **Slash** — slashes in an AoE around you; if your shadow is out and you hit your shadow with the slash then the shadow will slash as well, dealing extra damage
- **Rifen** — your shadow grabs enemies, rooting them for a brief duration
  - This ability can be cast while your shadow is in motion to drag enemies with it; swapping with your shadow while the root is active switches the enemy's location too
  - Cannot cast a damaging shadow ability (eg executioner) while the root is in effect
  - Long cooldown
- **Night's veil** — causes your shadow to emit a time-delayed massive damage nuke and slow; very low range and about 3-4 seconds of arm time; arm time depletes to 0.5 seconds if your shadow passes through an enemy or if it is hit by an enemy
  - Disables shadow swap for the cast time
  - Shadow can be otherwise manipulated
  - This ability will hurt the reaver if it detonates in range of the reaver (this includes if the reaver has their shadow when it detonates)
- **Razor wire** — places a trap that when activated slows enemies before dealing damage in an AoE based on how long the trap has been set (longer = more damage); slow is also stronger with time; upon casting, this ability has an instantaneous circular "tripwire" around it that slowly shrinks, dealing damage and pulling enemies with it, if an enemy is within the area for too long they are rooted
- Sends the shadow forward slowly with a ring of damaging daggers around it; after the shadow comes to rest, the daggers remain suspended for a few seconds, and will collapse inwards if an enemy steps into the ring

## Proposed SR rebuild 1

Highly tactical playstyle, positioning of the reaver and their shadow must be maintained constantly, as this positioning is what gives the Reaver combat mobility. High mobility and close quarters invisibility, coupled with high damage, are a strong combo, but gaining invisibility limits the Reaver's mobility by making him move close to his shadow. This build might be better without invisibility or with very short invisibility, since the combat mobility of the reaver is so extreme.

### Shadow mechanics / theory

- All mobility skills require having the shadow out
- Most damaging skills allow you to send your shadow out
- The first AA after collecting your shadow deals extra damage based on dexterity
- Collecting your shadow grants invisibility for 1.5 seconds
- Your shadow has a max range it can be away from you, if you exit this range it will dash to you, lightly damaging and slowing enemies along the way

### Kit

- Reascend revision 2
- Guillotine lotus
- Shadow swap
- Slash
- Black trap
- Executioner revision 1
- Razor shadows / Deadly mistake
  - Deadly mistake requires having your shadow in order to appear behind the enemy, otherwise it is just a dodge
  - The farthest you can teleport is to half your shadow's max range
- Rifen
- Night's veil
