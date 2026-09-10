---
source_doc: "General information"
source_folder: "Game notes / Combat design"
drive_file_id: 18mK3noDhCV9pKQlyL7_ArWy7vasZMrH3rud8pQgGxRI
drive_url: https://docs.google.com/document/d/18mK3noDhCV9pKQlyL7_ArWy7vasZMrH3rud8pQgGxRI/edit
drive_modified: 2016-02-11T04:45:35Z
exported: 2026-09-09
---

# General information

## Default key setup

*May be subject to change.*

- **Shift** — crouch
- **Space** — jump
  - Jumping while crouched will roll
- **wasd** OR **esdf** — movement (esdf provides easy access to the a key, but harder to press ctrl)
- **qerf1234** OR **wrtg2345** — ability slots
- **ctrl+asdfqwer123** OR **ctrl+sdfgwert234** — more ability slots
  - In total this key arrangement provides 19 ability keys without having to move the hand
  - Number keys may be easier for people with a Naga or similar
- **Left click** — left hand primary weapon action
- **Right click** — right hand primary weapon action
- **Ctrl+click** — secondary weapon action
- **Middle click** — misc / interact with environment

## Combat mechanics

- **Stagger** — basically a stun; staggers consist of a short period in which the recipient of the stagger can recover (players will do this by pressing spacebar), if the recipient does not recover in time they will be knocked down for a duration that depends on their size and character ability levels; staggers will have a certain strength value that determines how easy they are to resist (players will be given a larger window of opportunity to press space bar for easier staggers)
- **Bleed** — bleed meter must be initiated by a bleed inducing move, after that all damage taken fills the meter; bleeding enemies have a bar with a maximum value set, this maximum decreases over time; bleeds last until either the bleed bar is fully depleted or the maximum reaches 0; enemies bleed a percentage of their missing health based on how full the bar is
  - Bleed resist can decrease damage taken by bleed or increase how quickly the maximum of the bar falls
- **Poison** — poisons are applied through taking repeated damage from a poisoned weapon or poison inducing move, and deal a fixed amount of damage over time after the initial application; taking damage from a poisoning move will reapply the poison; poisons have a short duration
- **Autos** — after an auto with one hand the opposite hand will be ready to auto in half the time of the same hand (eg you auto with right hand, you have wait 1s to auto with your right hand again but can auto with your left after 0.5s)
- **Special attacks** — these occur by shift attacking; only certain weapons have these (some weapons may have one naturally and some may obtain them through forging)
- **Knockbacks** — knockbacks will have a certain strength and be able to knock enemies in all directions; enemies that collide with a wall or the ground while being knocked back will take additional damage based on their surface-normal impact velocity (so using a knockback downwards will deal extra damage, etc)
- **Crouch-jumping** — crouching, releasing, then jumping at the apex of the crouch release increases jump height

## Character progression

Characters will have base stats of strength, agility, attunement, toughness, affinity, dexterity, and fortitude.

- **Strength** — mitigates the effect of weapon size on swing speed, mitigates effect of equipment weight on movement speed and jump height, and increases damage per hit
- **Agility** — increases movement speed and jump height
- **Dexterity** — increases attack speed and critical damage as well as damage per hit
- **Affinity** — increases cdr
- **Attunement** — increases magic damage
- **Toughness** — increases health and physical resistance
- **Fortitude** — increases magic resistance and status resistance
