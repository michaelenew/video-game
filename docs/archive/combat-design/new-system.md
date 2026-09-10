---
source_doc: "New System?"
source_folder: "Game notes / Combat design"
drive_file_id: 1h5-5Mq1JmWaMwb2OKlHhbzyAyn2xha-tV6-hdKxPIX0
drive_url: https://docs.google.com/document/d/1h5-5Mq1JmWaMwb2OKlHhbzyAyn2xha-tV6-hdKxPIX0/edit
drive_modified: 2016-10-24T21:46:08Z
exported: 2026-09-09
---

# New System?

## Basics

### Key mapping

- Movement controlled
- Each character has 12 ability slots (ctrl/shft + q/w/e/a/s/d)
  - Can switch to 6, 8, etc. ability slots if need be
- Auto attacks and possibly grab controlled by left and right click
- Crouching is controlled by holding shift
- Jumping is controlled with space bar
- Shift+jump+w/a/s/d to dodge
  - Possibly just double tap jump+w/a/s/d

### Abilities

- **NO COOLDOWNS**
  - Abilities will be balanced by minimizing the impact each one can have on its own, through frame data, and resource costs
- Each character has a set of basic abilities, these are basic striking moves for all classes
  - Mage classes may have these moves gain extra effects
- Each character also has a set of moves that can be equipped in empty slots
  - Generally more powerful but more situational
  - These give the character their combat personality

## New statera / new class (Blade dancer?)

- Double blades that float
  - Can also be 4 blades (this would be cool, but may make the class unbalanced since it will be able to do so much stuff so quickly with blades constantly swirling around)
    - Possible way to balance this play style: blades can be hit by enemy hurt boxes and will be momentarily knocked down
  - I was thinking blade designs something like this
- Each blade is independent
- If the blades touch each other the player will teleport to the place of contact
- Scales with magic stats
- Auto attacks control each blade individually
- Basic abilities control both blades at once and have some mobility attached to them
  - Only use blades that are currently under the player's control (i.e. sending out a blade with an auto attack then sending out a basic ability will only do the ability with the one blade in the player's control)
  - To make maximum use of the class's firepower requires careful chaining and planning of when to use each ability
- Special abilities have other effects, and control the blades no matter where they are

### Special abilities

- Delayed cylindrical magic spell that deals damage and pushes things away; blades that are hit are shot forward
- Dashes towards the blade furthest from the player at 1.5x running speed
- Blades instantly begin moving outward, twirling around one another in a circle for three revolutions

### Basic abilities

- Downward diving slash with a large magical hitbox around blades; if the character hits the ground quickly after casting, the blade(s) used in the attack will be shot forward with their hitbox
- Upward slash and dash that sends blades forward

## Elementalist

- Emphasis on moves that are strong from range with high end lag

### Basic abilities (earth effects)

- No more possessing structures
- Raise slabs of stone
- Disrupt the ground at the target location to stagger enemies
- Shoot an earth spike
  - Casting on a structure will instead shoot many spikes in a cone that deal less damage
- Push a structure
  - Stronger push the closer you are
- Uppercut to launch structures into the air

### Fire abilities

Typically higher damage and higher mana cost.

- Make an AoE at the target location that damages over time
- Shoot a tongue of flame forward
- Hard hitting flame strike at target location (originates by shooting upward from player, requires to be controlled while in motion, accelerates toward where the player's cursor is pointing)
  - High mana cost
- Flame washes over an area, then the area explodes after a delay
  - Casting on a structure will cause the structure to explode as well, sending out shrapnel in a larger area
  - Higher mana cost
- Narrow fire pillar that remains for a short while after casting
- Casts a wall of flame short to mid range in front of the caster in a semicircle

### Air abilities

Little damage.

- Push things away from you in a circle
- Push things away from you more forcefully in a narrow cone
- Fires a wind blade that pushes you back; catches things it touches and carries them for a small ways with it
- **Razor wind** — long wind up, creates a spinning disk of air oriented vertically in front of the player, can be held and will fire on release, hold up to a structure to shoot shrapnel in a cone, will cause structures to explode if it contacts them, medium damage and range
- Back flips backwards on a blast of air, knocking back anything directly in front of the caster with the blast

## Reaver

- Invulnerable fast dash to your shadow that deals damage; high recovery time; slashes with a shadow blade as you dash; casting with your shadow will cause you instead to become invulnerable for a quarter second and slash directly in front of you

## Blood mage

- Invulnerable medium speed dash to target location; medium wind up time and recovery; steals health from enemies hit
- Sends out five tendrils radially in a cone that stop at the first enemy hit; slows and deals initial small damage then steals health over 3 seconds (the tendrils have a base area that the blood mage must remain inside of to regain health)
- Delayed hand-like thing that comes up from the ground and smashes the ground in a medium AoE
