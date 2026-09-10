---
source_doc: "Statera skills"
source_folder: "Game notes / Combat design"
drive_file_id: 1KGLeSLQ7J-DG6onznm8SM9LDoct8zMgRlF94ydSVmUA
drive_url: https://docs.google.com/document/d/1KGLeSLQ7J-DG6onznm8SM9LDoct8zMgRlF94ydSVmUA/edit
drive_modified: 2016-04-27T20:56:27Z
exported: 2026-09-09
---

# Statera skills

## Theory

The statera is a melee mage with high sustained damage and the potential to burst. The playstyle is about balance. A focus on balance should be encouraged by the penalties of unbalanced (human, divine, devolvat) states, but the benefits of each of these states should encourage players to constantly be changing their playstyle to fit the situation. Positional balance should also be necessary; this is encouraged through ability cast times and ranges. Individual abilities should avoid being dependent on other abilities for impact (such as buff abilities, pure CC abilities, etc); in other words, abilities should feel impactful when used alone but still allow for combo potential. Abilities should NOT be individually impactful to the point that spamming a single ability can decimate an entire area/target. Focus on making low range, medium damage, low-medium utility moves, as well as some mobility skills.

## Auto attacks

- IDEA: Every third auto attack grants increased damage on the next ability cast
- IDEA: Abilities mark enemies, marks can be detonated for extra damage by auto attacks
- IDEA: Auto attacks remove resource, gaining extra range and scaling with magic damage

## Resource system

- Casting abilities grants arcana which decays naturally over time
- At **<25%** resource the statera is considered *human*, casting abilities will restore health based on the resource they give
- At **>25% and <75%** resource the statera is considered *balanced*, giving them 20% cdr
- At **>75%** resource the statera is considered *divine*, draining health on cast increasingly with number of casts in the divine state and giving them 40% cdr and a 25% magic damage increase
- If the statera reaches **100%** resource, the being inside them takes over
  - Possible limiting factors
    - Form has a cooldown
    - Abilities used in this form are set on cooldown after the form ends
    - Abilities used in this form continue to cost health to cast even after the form ends based on how much you used them
      - I like this, we can give each ability a counter for how much damage it will deal to the player when cast; the counter decays over time and only goes up when the ability is cast from a divine form
  - Abilities have 95% cdr; abilities already on cooldown have their cooldowns refunded
  - High health cost per cast
  - Double attack speed
  - Basic attacks that land cause a small dash in the direction the player is moving

## Skills

- **Tempest** — passively causes all abilities to mark enemies on hit, autoing a marked enemy will deal extra damage, consume the mark, and grant a short burst of movement speed
- Line skillshot on a low cooldown, velocity decreases as the skillshot moves further and detonates once the casting key is released; deals low damage when it passes through an enemy and again when it detonates; if detonated in the vicinity of the statera it will hurt the statera and push them away from the explosion
- Large AoE with medium range, slows enemies; detonating a mark of the merciful on an enemy in the AoE will root the enemy
- Time delayed AoE skillshot, massive instantaneous (thin) damaging pillar comes down in the center, then slowly expands around it; enemies hit by the initial pillar are staggered
- Line skillshot that moves rapidly forward with a small hitbox until it hits something or reaches its max range, then moves more slowly; damages first target hit; the statera can teleport to the projectile once it is in the second stage of its movement, dealing high AoE damage
- Snares all marked enemies in a small radius
- Four prongs that start out at the statera's location (like two pairs of open scissors); holding the ability button down moves the prongs outward radially and releasing causes them to swing inward; each prong deals independent damage
- Short range dash in any direction; the next autoattack casts a mid range trace that locks on if it hits a tempest marked enemy; the statera will dash to a target if the trace hits
- Very short range, fast moving line skillshot; travels forward a short distance damaging enemies and pushing them back then stops; the projectile then begins to move back to the statera, gaining speed; explodes when it reaches the statera or an enemy afflicted by mark of the merciful, damaging in an AoE and pushing away whoever it hits
- **Divide** — medium range dome that rotates into existence, dealing damage on the way up and blocking enemy projectiles once it is up; if the statera hits the dome with any other ability they will dash to where the ability hit the dome and keep momentum; the shield is only up for 2-3 seconds
- Cone that starts at the statera and extends outwards; staggers enemies that are afflicted by a mark
- Spawns three damaging motes that fly in the direction of the statera's next three auto attacks; these motes can detonate or apply tempest marks; charging an auto attack will charge all of the remaining motes, sending them all out and letting the statera dash a short distance upon release
  - REVISION 1: Spawns five projectiles that swirl around the statera
- Very short range attack, two arms sweep down diagonally in a V formation; if an enemy has a tempest mark on it then they will be slowed; hitting with both arms will knock back
- AoE that deals damage when auto attacks are cast
  - BORING
- Thick arrow shaped skillshot that fires three time in the target direction, the third hit is a heavy knockback; ability channels for about 0.5 seconds, then the three projectiles fire in rapid succession
- The statera uppercuts into the air with a large blade-like aoe in front of them after winding up, knocking enemies up if they were hit while the statera was on the ground
- Wide disc projectile (hollow disc); projectile travels a fixed distance then contracts; the projectile will track towards the cursor location; deals damage while traveling and while contracting; reactivating the ability will cause the statera to teleport to the disc and make it contract early
- Instantaneous short range slash around the statera that deals damage; reduces damage taken for the duration of the animation
  - REVISION 1: instantaneous short range slash around the statera that deals damage; taking damage during the cast causes a shockwave around the statera that staggers
- Short range projectile that detonates when it hits the ground, an enemy, or is hit by any of the statera's moves; knocks away the statera and enemies hit
- Cone in front of the statera that fires for 3 seconds at 4 times per second, every 4 hits an enemy receives makes them take extra damage
- Medium sized projectile that quickly extends forward and then comes back
- Strikes instantaneously in an area around the statera, boosting them upwards
- Medium ring at target location that quickly collapses inward, dealing low damage; after the ring collapses to nothing a vertical bolt strikes at the center, dealing medium damage
- Charged skillshot; line of bolts coming down from above extending forward; slow while charging; number of bolts, forward extension, and height of bolts increases with charge duration; charge rate of effect increases with increasing resource (ie at higher resource you don't have to charge for as long)
- Triple line projectile firing in a cone; each projectile detonates on the first enemy hit, leaving behind a slowing area which detonates after 2 seconds
- Ball skillshot that grows slowly over time up to a cap size; moves at slightly faster than the statera's base move speed; ball velocity slowly changes to try to match the direction the statera is moving; while in the ball, the statera gains increased movement speed and is not affected by gravity; can be detonated by reactivating or by reaching max range; ball velocity and movement speed buff increase with increasing resource
- Short range skillshot that detonates multiple times over its path; the projectile can be stopped by enemies in the way, but will continue to detonate

## Proposed build 1

*(Empty in the source document.)*
