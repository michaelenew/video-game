---
status: built — v1, then on three clicks (2026-10-09), the blaster (2026-10-12), thralls and the Well (2026-10-13), then her territory (2026-10-18); unplayed
decided: 2026-09-09
revised: 2026-09-23
revised: 2026-10-12
revised: 2026-10-13
revised: 2026-10-18
sources: ../exploration/0021_blood_mage_territory.md, ../exploration/0016_blood_mage_thralls_and_the_well.md, ../exploration/0015_blood_mage_blaster.md, ../blood-mage.md, ../plans/blood-mage-v1.md, docs/archive/combat-design/blood-mage-skills.md
---

# Blood mage — kit

**Identity.** Her blood spreads like a sickness, and she fights from inside it. Every cast is a
cut she makes in herself; every hit spills the other fighter onto the floor; everything she
throws claims the floor it touches; and the only way to close her own wounds is to put an
ability through the blood she spilled. A war scythe, a reaper's reach that grows with how open
she has left herself, and a floor that is hers.

> **Rebuilt 2026-09-23.** This is the v1 kit of [../blood-mage.md](../blood-mage.md), built
> along [../plans/blood-mage-v1.md](../plans/blood-mage-v1.md): grey health, essence pools,
> the scythe on the clicks, the blink on the dodge. Everything below describes what is in the
> game. **None of it has been played by a person.** The four human checkpoints in the plan
> are open, and the feel-log entries for the four milestones say so. The economy it replaced
> — cost on the press, leech on the hit, a field that drains — is recorded under
> [Was](#was) at the end.

> **On three clicks, 2026-10-09 — built, unplayed.** The Champion's grid, as the Elementalist
> got it: her three clicks are *my blood*, *your blood* and *the scythe*, each with a move on the
> floor, one in the air and a jump attack. The sections below describe the moves as they were
> built before it; **where they name a key, §"On three clicks" wins.** The direction, and the
> six combos it was built towards, are
> [../exploration/0009_blood_mage_on_three_clicks.md](../exploration/0009_blood_mage_on_three_clicks.md).

> **The blaster, 2026-10-12 — built, unplayed.** The owner's verdict on three clicks: short,
> awkward, pools too small, and "spam Black spike". §"The blaster" below is what is in the game
> now; **where anything after it disagrees, §"The blaster" wins.** The direction, the six combos
> and the questions still open are
> [../exploration/0015_blood_mage_blaster.md](../exploration/0015_blood_mage_blaster.md).

> **Thralls and the Well, 2026-10-13 — built, unplayed.** The owner's verdict on the blaster:
> the stains and the detonation chain are the class, but the snowball feeds itself, the start
> costs most of her bar, and the moves are not synergistic. §"Thralls and grey" below is
> what is in the game now; **where anything after it disagrees, it wins.** The direction, the
> nine combos and the answers are
> [../exploration/0016_blood_mage_thralls_and_the_well.md](../exploration/0016_blood_mage_thralls_and_the_well.md).

> **Her territory, 2026-10-18 — built, unplayed.** The owner's new direction: controlling
> territory and building forward. §"Territory" below is what is in the game now; **where
> anything after it disagrees, it wins.** The direction and the options weighed are
> [../exploration/0021_blood_mage_territory.md](../exploration/0021_blood_mage_territory.md).

## Territory — built 2026-10-18

| | Left: thorns | Middle: the spikes | Right: the scythe |
| --- | --- | --- | --- |
| **Tapped** | **Thorn lash** (floor and air) | **Black spike**, one | **Sweep**, strung to three |
| **Held** | **Bramble**, growing | **Black spike**, one to seven (air: **Stake**) | **Reap** |
| **Held to the end** | **Black vines** | **Crimson cascade** | **Harvest** |
| **Space + click**, floor or once in the air | **Thornfall** | **Upheaval** | **Rising reap** |

| Key | Move |
| --- | --- |
| `Q` | **Coagulate** |
| `E` | **Heart** |
| `shift` | The **teleport** to a stain under the crosshair (12 m); otherwise, on her ground, **swim**; otherwise an ordinary dodge |

**The rows were retuned, not renumbered** (`sim::moves::blood`): the Haemorrhage's row is the
Thorn lash, the Hemoplague's the Bramble, Command's the Sweep, the Carpet's the Reap, the
Summon's the Heart, the wings' Thornfall, the golem's the Harvest. **Cut: the Carpet, Command,
the Summon and the Blood golem.** `tests/territory.rs` holds the ground, the strong version, the
swim, hearts and her thralls on her ground; `tests/blood_mage.rs` the clicks.

### Her ground

**One substance, in patches** (`sim::ground`, `World::ground`): discs on the floor with an owner
and the frames each has left -- forty-eight of sixteen bytes, a table of its own in the snapshot
so a fight's effects do not make room for it. It comes from three places:

- **Her moves, where they touch** (`World::lay_touch`): every spike and eruption, the Stake's
  landing, the Bramble on every tick, the vines, Thornfall's ring, the Upheaval and the Thorn
  lash where it lands, hit or miss -- a patch the size of what touched, never under `Ground, a
  touch lays at least` (1 m), for `Ground, a touch lasts` (150 frames).
- **Stains**: a stain standing is her ground, its column and `Ground, a stain spreads round it`
  (0.8 m); one that **drains away** -- not drunk, not set off -- leaves a patch where it stood
  for `Ground, a faded stain lasts` (360). The bleed's stains make the trail of a bleeding body
  hers.
- **Hearts**: `Heart, holds ground round it` (3 m) while it stands.

**Patches bridge** (`ground::step`): every `Ground, bridges are laid every` (8) frames, two of
hers whose edges are within `Ground, bridges a gap of` (2 m) get a patch laid over the gap, and a
patch touching another of hers -- or a stain or a heart -- loses a frame of life only every
`Ground, joined loses a frame of life every` (3). Scattered close, sparks join into a field;
far apart they go out. A patch laid mostly on one of hers merges into it, up to `Ground, widest
patch` (4 m); past the table, the patch with least life left makes way, hers first.

**Drawn as one surface** (`game::claimed`): the union of every patch, stain and heart disc,
rebuilt every frame on a 0.3 m grid with its edge found by marching squares, so overlaps are
never darker, and every vertex draped 6 cm over the floor the simulation stands on, so slopes and
a town's flagstones never cover it.

**On her ground** is worked out once a frame before anybody moves (`World::mark_the_blood`,
`Player::on_blood`): her feet on the floor, or swimming, inside a patch, a stain or a heart's
reach, within `Ground, counts this far above or below it` (0.6 m).

### Strong and weak

**A move she starts on her ground hits `Ground, her moves hit (x)` (×1.3) harder** -- decided on
the press (`Player::thrown_at`, the slot the Dual mage's depth uses), so a swing that carries her
off it keeps what it started as. **Never wider**: `Effect::field_radius` does not grow for her.
**Off her ground a move seeds more**: what it lays is `Ground, thrown off her ground seeds (x)`
(×1.5) wider and longer. Drawn: a ring of essence at her feet and a faint flame off her body
while she stands on it.

### The swim

`shift` on her ground, on the floor, with no stain under the crosshair: she sinks
(`Player::swim`). Under, she walks at `Swim, speed (x walk)` (×1.8) and stands `Swim, stands this
tall` (0.6 m) -- most blows thrown at a standing body pass over. Letting go, a jump, a hit, or any
move brings her up; a click comes up as its move, the strong one. **Off the edge she pops out**,
hauled along the way she was going at `Swim, pops out at the edge at` (9 m/s), and a `shift`
still held from it is not a dodge until it is let go. Drawn sunk, a hump of her showing, with a
ripple over her.

### The scythe

- **Sweep** (right, tapped): the scythe across her front, 2.6 m, 45. Right click again inside its
  recovery strings the next (`Player::string`), up to three: the second comes back the other way,
  the third hits `Sweep, the third of a string hits` (×1.5).
- **Reap** (right, held past the Champion's window): a lunge into a heavier swing. It steps
  half its row's 3.5 m for no hold and all of it for the longest, and `Reap, lunges further on her
  ground` (×1.6) from her ground.
- **Harvest** (the Reap held to its end, a finisher): one great turning reap, a whole turn,
  worth the row and `finisher_per_grey` for every point of grey and Well it spent, paying back
  what it burned as every finisher does -- and every stain of hers within `Harvest, reaches all
  round` (5 m) is drunk as it comes out.
- A swing's cost lands as a stain **at the blade's end**, not at her feet, where she would walk
  it straight back up.

### Thornfall

The wings' burst along her aim, stopped at the first wall or stone, and where it ends she lands
in **a ring of thorns** (`World::land_the_thornfall`): Thornfall's row, 60 in 2.5 m, and the
floor under it is hers. The teleport to a stain arrives in the same ring.

### Thralls and hearts

**A thrall on her ground** (`state::ON_GROUND` on its `struck`) runs at `Thrall, on her ground
runs (x)` (×1.4), bites `Thrall, on her ground bites (x)` (×1.5) harder and does not drain; off
it, it drains as its stain would. It goes for the nearest enemy within its seek **that is on her
ground or within `Thrall, leaves her ground by` (3 m) of it**; with none, off her ground, it walks
back to the nearest of it. Drawn a fifth bigger and burning brighter on it. Command is gone.

**A heart is earned** (`Player::heart_meter`): every point of damage she deals -- a blow, an
eruption, a bite, a bleed's tick; counted where each hit spills (`World::spill`), not what a
cast cost her -- fills it, up to `Heart, earned by dealing` (400). Full, `E` plants a heart and
empties it; empty, `E` throws nothing. The HUD shows it as a thin bar under the Well, burning
bright when full.

**A heart** (`E`) is planted at the crosshair's floor (grounded, 15 m). It holds `Heart, holds
what she paid (x)` (×3) what she paid for it (10% of red), never less than `Heart, holds at
least` (150) -- what she paid goes into it, not onto the floor. Blows come off its blood (a
swing's frames count once), it bleeds away as a stain does, and every `Heart, sends a thrall
every` (240) frames, while she has room for a thrall, it spends `Heart, a thrall costs it` (60)
on one at its foot. `Hearts at once` (2): a third replaces the oldest. A creature turns on a heart
or a thrall nearer it than she is. Drawn as a heart's shape, not its anatomy -- two lobes over a
point, two short vessels out of its top -- in a crimson that burns from inside, hung at head
height, beating (a heavy beat and a light one), turning, with a ring of essence round it, a
faint pillar of light through it and roots into the floor; smaller the less blood is left in
it.

### What it did not change

The grey and the Well, every stain one kind, Coagulate, the spikes and their chains, the
teleport, the Stake and the Upheaval, the vines and the cascade.

## Thralls and grey — built 2026-10-13, revised from play the same day (superseded 2026-10-18)

| | Left: the area | Middle: the spikes | Right: the thralls |
| --- | --- | --- | --- |
| **Tapped** | **Haemorrhage** (floor and air) | **Black spike**, one | **Command** |
| **Held** | **Hemoplague**, growing | **Black spike**, one to seven (air: **Stake**) | **Carpet of blood** |
| **Held to the end** | **Black vines** | **Crimson cascade** | **Blood golem** |
| **Space + click**, floor or once in the air | **Blood wings** | **Upheaval** | **Rising reap** |

| Key | Move |
| --- | --- |
| `Q` | **Coagulate** |
| `E` | **Summon** |
| `shift` + direction | The **teleport** to a stain under the crosshair (12 m); otherwise an ordinary dodge |

**Tapped or held is the Champion's window** (2026-10-14): a click is held once it has been
down `Hold, a finisher after` (12 frames) from the press, or for the tap's own startup if that
is longer. Command winds up in one frame, and with the tap decided there a right click had to
come up inside a frame or two to be Command at all; it now waits at the top of its wind-up for
the press to decide. The Haemorrhage's startup is already twelve, so the left click is as it was.

**The rows were retuned, not renumbered**: the Transfusion's row is the Carpet, the sweep's
Command, Tides' the Summon, the chain's Coagulate, the leap's the Rising reap, Frenzy's the
Black vines; the cascade and the golem were appended (`sim::moves::blood`).
`tests/blood_mage.rs` holds one test per move and rule.

### Grey, and the finishers

**Grey holds for `grey_grace` (two seconds) after it last opened, then fades at `grey_fade`
(60 a second).** It is the window to spend it in. **A finisher is a hold kept to its end, at
any time**, and it spends all her grey and the Well (`Player::fuel`): its blow is the row's
damage and `finisher_per_grey` for every point spent, so it always beats the row and is bigger
the more there was. **What it burns it owes back** (2026-10-14): the grey it spent is held as
`Player::owed`, and every point of damage the finisher deals turns `finisher_repays` (100%) of
a point of it back into red -- by way of `Player::heal`, the one leech she has. A finisher that
lands is no longer health given up for good; one that whiffs still is. The vines, the golem's
punches and the cascade's eruptions all pay.

- **Black vines** (left): barbed vines erupt out of the Hemoplague's middle to `vines_size` of
  its radius over `vines_grow`, hitting and bleeding everything the front passes, then stand for
  `vines_lasts` (five seconds), bleeding and slowing whatever is in them.
- **Crimson cascade** (middle, since 2026-10-14): the impact erupts for the finisher's blow
  and spills `cascade_first` pools round it, holding all the grey she spent between them. Each
  goes up `cascade_fuse` frames later and spills `cascade_branch` smaller ones
  (`cascade_share` of it) `cascade_step` further out, for `cascade_generations`, while a pool
  holds at least `cascade_least`; each eruption hits for `cascade_power` of the spike's row per
  full stain it held. **Every pool it spills reaches for her nearest stain** within
  `cascade_track`, however small, stepping toward it and lighting it when it gets there -- so a
  bleed's thin trail is followed to its end. Cascade pools are her spent grey: not drunk, not
  walked up.
- **Blood golem** (right): everything the Carpet covers -- stains, thralls, what the carpet held
  -- and all her grey, fused into one thrall with that many hit points. It takes blows off its
  health, loses `golem_self_hurt` of every punch it lands, and drains as a stain does.

### Every stain is one kind

**A stain is a pool of essence, and it stands** (2026-10-14): it is tested as a column a body's
height at most (`Effect::pool_slab`) and is drawn so -- a splash on the floor with a swaying
flame of essence out of it, crimson gone to violet and lit from inside, so it reads as what she
draws out of a body rather than what a wound leaves. **A spike aimed through that column goes
to the stain** (`aim::column_on_sight`, with the blink's `Crosshair lock on a pool` slack above
it): the Black spike, the Stake, the Upheaval and the cascade (`moves::blood::seeks_pools`).
Before, the crosshair had to be on the stain's disc on the floor, which past a few metres was a
pixel-hunt.

**The floor spike and the Stake** (2026-10-14): from play, the best thing she could do was jump
in circles throwing the Stake -- 10 frames out against the floor spike's 30, a 2.5 m area
against 1.6, 4% of her health against 6%. The floor spike now winds up in 16 (just past
reaction time, which `the_spike_takes_long_enough_to_be_seen_coming` holds), stands 2.2 m
wide and costs 5%; the Stake lands in 1.8 m and costs 5%. The Stake keeps what it is for:
throwing from the air and pinning a body there.

- **Every cost is a stain**, where it was aimed. **Every hit is a stain**, hers or a thrall's.
- Collected by walking, teleporting, hitting through it, or the spike erupting it; it settles
  for `stain_settles` first. **Eruptions spill nothing.** What her red cannot hold fills the
  **Well**, the bar under her health.
- **A chain runs link by link**: a stain set off by an eruption beside it waits `chain_delay`
  (three frames) -- fast enough not to react to, slow enough to see where it started.
- **The dodge** is an ordinary dodge unless the crosshair is on a stain within 12 m; it throws
  nothing.

### The fields

- **Hemoplague** (left held): laid at the crosshair's floor when the hold opens, growing from
  `plague_from` to `plague_to` over the hold, hurting and slowing to `plague_slow`; let go, it
  stands at that size for `plague_lasts`.
- **Carpet of blood** (right held): laid the same way, growing from `carpet_from` to
  `carpet_to`, paying 1% of her red every `carpet_pays_every` into itself and hurting a little;
  let go, it is **one big stain** of what it held, and every stain of hers it covered stands up
  as a thrall. A longer hold is a bigger stain and a wider area.
- **The Black spike** (middle held): one spike for a tap, and up to `spikes_most` (seven) on a
  ring as wide as `spike_spread_most` at a full hold. **The swarm goes for her stains first**
  (2026-10-14): the spikes past the first land on stains of hers inside the ring, nearest first,
  however small; only what is left stands on the ring.

### Thralls

A thrall is **a stain on legs**, up to `thrall_cap`: stood up by the Carpet or by `E`. **Her
click comes first**: Command sends every thrall and the golem to the crosshair's floor, and
they bite nothing until they are there. Otherwise a thrall goes for **the nearest enemy** within
`thrall_seek` -- a fighter, the creature, a pack's critter -- **walking round walls** (it
follows a wall on the hand it turned to). **A bite spends what it deals** and spills it under
the prey. One blow kills a thrall back into a stain of what it had less the blow. A creature
turns on the golem, and on a thrall nearer it than she is. It is drawn as a fighter's body gone
brutish -- the same rounded torso, head and limbs, crouched, horned, long-armed -- in
translucent blood.

## The blaster — built 2026-10-12 (superseded 2026-10-13)

| | Left: my blood | Middle: your blood | Right: the scythe |
| --- | --- | --- | --- |
| **On foot** (held: three tiers) | **Transfusion** | **Black spike** | Reaping sweep (tap) / **Tides** (held) |
| **In the air** | Haemorrhage | **Stake** | **Reaper's chain** |
| **Leaving the floor** (space + click) | **Blood wings** | **Upheaval** | **Reaper's leap** |

| Key | Move |
| --- | --- |
| `Q` | **Frenzy** |
| `E` | **Hemoplague** |
| `shift` + direction | The **teleport** to a stain under the crosshair (20 m); otherwise the **Sanguine flow** |

**The rows were retuned, not renumbered**: the Bloodletter's row is the Transfusion, the
Grasp's the Hemoplague, the nova's Tides, the jet's the wings, Marionette's the Upheaval, the
Nail's the Stake, the Hook's the chain, Harvest's the leap, and Frenzy was appended
(`sim::moves::blood`). The Bloodletter and Grasp effects are still in the simulation and
nothing casts them. Its tests were folded into `tests/blood_mage.rs` in 2026-10-13.

### The charge

Each click on the floor is **held through three tiers** (`moves::Charge::Tier`): a tap is the
first; the second at `Charge, tier two at` (30 frames), the third at `tier three at` (60),
sooner by up to `Charge, faster at full grey` (×1.5) with a full bar of grey. Each tier is
paid in red as it is reached (`tier two costs` 3%, `tier three costs` 4%, on top of the
row's own cost on the press). Holding, she crawls at `Charge, walk while holding` (35% of a
walk). **A jump or a dodge out of the hold stores it** in her vessel (`Player::vessel`, the
move and its tier) and she jumps or dodges that frame; a **fresh** press of the same click lets
it go from wherever she is, in the air too, at the stored tier, with nothing paid again. The
vessel bleeds 1% of red every `Vessel, 1% of red every` (60 frames). A hit while still holding
loses the charge. Right click is the sweep on a tap: held past the sweep's startup it becomes
Tides (`countdown`), paying Tides' own cost there.

### Every stain is collected

A stain is **drunk** -- its essence turned from grey back into red, grey still the ceiling --
by any of three things: **walking** into it (`Stain, walked into drinks` 100%,
`World::drink_underfoot` -- only as much as her grey can hold, the stain keeping the rest, so
at full health she walks through her stains and leaves them for the spike), the **teleport**
to it (the same share), or **an attack** whose
volume, bolt or effect lands over it (every move's `Drinks of a pool` is 100%). **An attack
through a stain is harder for it**: `stain_bonus` adds `Stain, a hit through one adds` (50%) of
the stain's essence to the blow, on a fighter, a creature or a small body alike. A spike on a
stain still **erupts** it instead, and every stain within the eruption and `Black spike, an
eruption sets off stains this far past its edge` (2.5 m) erupts with it.

Grey fades at 3 a second (was 12) and stains drain at 5 (was 10). **Every move deals damage**,
the movement included, and **every hit on a creature spills** by the blow's own worth rather
than what its hide let through. The bleed ticks 25 every half second for four seconds. Her own
blood -- the wings' takeoff -- is still only a door.

### The moves

| Move | Reach | Damage | Notes |
| --- | --- | --- | --- |
| Transfusion | 20 m bolt | 45 / ×1.7 / ×2.6 | tier two pierces everybody on the line; tier three throws back at 14 m/s |
| Black spike | 22 m, grounded | 100 / 3 × 80 / 3 waves × 80 | tier two: three spikes 1.6 m out from the point; tier three: a 3 m field, a wave every 30 frames |
| Reaping sweep | 2.4 m, grows with grey | 45 | the tap |
| Tides | a ring 4 / 6 / 8 m | 60 / ×1.7 / ×2.6 | the hold; throws out |
| Haemorrhage | 18 m bolt | 30, then the bleed | |
| Stake | to the crosshair, 18 m | 90 | pins anybody in the air (`NAILED`); lands in a 1.8 m area that slows (2.5 until 2026-10-14) |
| Reaper's chain | to the crosshair, 18 m | 40 | a fighter is yanked to her (30 m/s); a creature, a critter, the floor or a wall pulls her (40 m/s); the sky, 5 m along the aim |
| Blood wings | 8 m along the aim | 40 in 2 m | tapped; any way, up included; her own stain at the takeoff |
| Upheaval | 15 m, grounded | 80, launches 18 m/s | a hop for her (12 m/s); on a stain it erupts |
| Reaper's leap | an arc to the crosshair's floor, 10 m | 90 in 2.5 m | lands as the slam; drinks every stain within 3 m |
| Hemoplague | 22 m, grounded | 30 every half second for 5 s | a 2.5 m-radius, 4 m-tall cloud; not a hit -- no stun, no shove; all it spills goes to one stain at its centre; one at a time |
| Frenzy | 6 m round her | 15 every 10 frames | see below |

### Frenzy

`Q`. **Its own blows spill nothing** -- the area and the explosions -- or each would be a
stain inside the area exploding for twice itself (the loop that killed a Mireback in twenty
frames). She is **planted** for `Frenzy, lasts` (180 frames): no walking and no jumping; her
only movement is the teleport. Within the row's reach (6 m) everyone takes the row's damage
every `Frenzy, hurts every` (10 frames) and starts to bleed. Every stain of hers within
`Frenzy, pulls stains within` (15 m) **rushes** to her at 12 m/s and **explodes** at the edge
of the area for `Stain, explodes for` (×2 its essence) in 2.5 m. **The price**: no stain heals
her while it lasts -- whatever would have been drunk explodes -- grey drains at 60 a second,
and when it ends whatever grey is left is gone. A **teleport to a stain** explodes it where she
arrives and adds `Frenzy, a teleport adds` (60 frames), to `Frenzy, lasts at most` (480).
`Frenzy, needs grey of at least` is 0: it is BLAST-7 in 0015.

### Send and zip: the teleport and the flow

`shift` with a stain of hers under the crosshair, within `Teleport, to a stain within` (22 m)
and with a clear line: she is there, drinks it, and arrives in a burst of blood (the wings'
row: 40 in 2 m) -- in the air too, even with the airdodge spent, since the stain is on the
floor. With none, the dodge is the **Sanguine flow**: `Dodge is a blink` is on, a flat 6 m
slide in `Blink, frames to cross`, invulnerable for the dodge's frames, drinking every stain
she crosses on the way -- **and it sends her blood**: a glob thrown to the crosshair's spot on
the floor, up to `Dodge, sends her blood up to` (20 m), that lands as a stain of her own blood
(60) -- a door, never a heal, and the only one: a new one replaces the last
(`World::send_the_blood`, `land_the_sent`). An airdodge sends it too. **The traversal cycle is
send, zip, send**: about 55 m in 2.5 s on flat ground, against 17.5 m walking.

**Not drawn yet:** the vessel's glow, and the Frenzy area. The Hemoplague is a faint disc with
wisps inside its column, the spike field a ring of spikes rising with each wave, and the moves
borrow her five clips (`view::play`).

## On three clicks — built 2026-10-09 (superseded 2026-10-12)

| | Left: my blood | Middle: your blood | Right: the scythe |
| --- | --- | --- | --- |
| **On foot** | **Blood nova** (held) | Grasp (held) | Reaping sweep |
| **In the air** | Haemorrhage | **Nail** | **Hook** |
| **Leaving the floor** (space + click) | **Blood jet** (held) | **Marionette** | **Harvest** |

| Key | Move |
| --- | --- |
| `Q` | Bloodletter — in no combo; the candidate to cut once she has been played |
| `E` | Black spike |
| `shift` + direction, crosshair on a pool | Blink |

The buttons are named once, in `sim::moves::blood::keys`; the dispatch is `state::blood_move`,
which reads where her feet are and the takeoff window (`Player::rise`, shared with the
Elementalist). The six new rows were appended to her table, so the five before them kept their
slots and their knobs.

**Her own pools are doors, not heals.** A pool of her own blood can be blinked to, erupts under
the Black spike, and is what the jet leaves behind; drinking skips it (`drink_over`, the
scythe's collection), and `drink_from` on one of hers spends it for nothing. Only another
body's blood heals her. Without this she spends health and drinks it straight back.

**A hit on somebody in the air spills under them**, on the floor below (`spill_under`, through
`Terrain::ground_under`) — reversing v1's "an airborne hit spills nothing", since Marionette
and the Nail exist to put people in the air.

**Creatures bleed.** Her hits on a creature already spilled under the part struck; now the
Haemorrhage's bleed runs on a creature too (`Monster::bleeding`, ticked beside the fighters'
bleeds, `bleed_on_creatures` per tick), so the trail-and-chain combo and the Hanging work on a
hunt. This is the answer to `CLASS-2`.

The new moves, each in one paragraph:

- **Blood nova** (`EffectKind::Nova`). Hold to charge, paid **in red health as she holds** —
  one point every `blood_pays_every` frames, kept on the fighter (`Player::self_spent`) — and
  release to burst: a sphere from `channel_from` to the move's full reach by the share of the
  hold, which hurts and throws back whoever is inside, and leaves a pool of her own blood at her
  feet holding what she paid.
- **Blood jet**. A held takeoff: the jet drives her along the line `aim::skillshot_path` solved
  at `jet_speed`, never flatter than `jet_least_rise`, gravity off while it runs. On release
  everybody on the line from where she left the floor (`Player::launched_from`) to where she
  is is hurt — more by `jet_damage_per_frame` for each frame held — and so is any creature at
  either end or critter on the line, and her own pool is left on the floor where she took off.
- **Marionette**. A takeoff swing that **only lifts** (`launch`), and lifts her after them
  (`self_lift`), so she arrives in the air beside her victim. It drinks like the Grasp.
- **Nail** (`EffectKind::Nail`). A long black spike thrown down the crosshair. On a body **in
  the air** it **pins** them there for `nail_pin` frames (`seized` with `NAILED`: no velocity,
  no gravity), and their spill drops to the floor below. On a body on the floor it is a hit
  and nothing more. The pin takes two hits — Marionette, then the Nail — by design.
- **Hook**. Thrown along the aim, it catches the first fighter, creature, stone or wall
  (`aim::first_along`) and hauls her to it at `hook_speed`, stopping a body's width short. A
  miss is the cost. It is the grappling hook in every combo.
- **Harvest**. A big jump inside a wide flat spin (`self_lift`, `arc`), which drinks the pools
  under its blade. Since she is off the floor before the blade comes round it passes over a
  knee-high critter (`docs/design/critters.md` §7).

**Not drawn yet:** a pinned body hangs in the air with nothing showing the nail through it;
the Nail in flight and the nova's burst are drawn, the pin is not. The jet's wake is not drawn
either — only her body moving and the pool left behind.

`tests/blood_mage_clicks.rs` holds one test per move and the Hanging end to end.

The specification is [../blood-mage.md](../blood-mage.md). This document is what the game
does, where it differs from the proposal, and why. `cargo run -p sim --bin essence` prints
every number named here, measured; `cargo run -p sim --bin frametable` prints the frames.

## Mechanic — grey health and essence

### Grey health

Health she loses — spent on a cast, or taken from a hit — turns **grey** rather than going:
a segment of the bar beside the red that is no longer hers but can still be reclaimed. The
HUD draws it as a faded tail on the red. Only a drink from a pool turns grey back to red.
Grey **fades** on its own, at a fixed rate rather than a fixed time (`Grey fades`, 12 a
second), so a large wound stays open longer than a small one and there is always a clock.

`red + grey + gone == max` on every frame, and `gone` only ever grows by the fade —
`crates/sim/tests/grey.rs` holds it. A cut can never kill her: self-damage clamps red at one,
and what was actually paid is what turns grey. Enemy damage lands on red and turns grey by
what it dealt; nothing erases grey but the fade.

**A cost is a share of what she has.** Every `Cost` in her move table is a percentage of her
*current* red, not of the bar: a spike at full health opens ninety, a spike at two hundred opens
eighteen. Casting at high health is how she gets power quickly — the wound is the reach — and
casting at low health opens a small wound, so she is never burning herself to death trying
to get back into a fight. `Player::cost_of` is the one place it is worked out, and `grey.rs`
pins that a full-health cast costs more than three times a one-fifth-health cast and that a
cast at one health costs nothing.

**The more grey she carries, the bigger and harder the swing.** The sweep's hit volume
multiplies its reach by a straight line from one at no grey to `Scythe reach at full grey`
(×1.5) at a full bar, its **radius** by a second line to `Scythe width at full grey` (×2),
and its damage by a third to `Scythe damage at full grey` (×1.6). A wider volume is an
easier collection of the pools it passes near, which is what pays the constant, measured
aggression the class is for. This is the one hit volume in the game that scales with a bar
— [../aiming.md](../aiming.md) refuses that everywhere else — and it is allowed on one
condition: **the volume is drawn at the size it hits at.** The weapon itself is iron and one
size, drawn at the row's reach and riding her hands; what grows is drawn as **essence**, the
same stuff as her pools, in the exact capsule the hit test reads — because it is the life
force doing the swinging, and the more of it she has let out, the more of it there is around
the blade. It lingers and fades through the first frames of the recovery so the arc can be
read. `view::scythe::swing_volume` is the shape, `state::live_reach` and `live_radius` are
the numbers, and `view/tests/kinematics.rs` checks the drawn capsule against the hit volume
at three levels of grey, and that the iron did not grow with it. Both players read a thing in
the world, not a number.

| Grey | Reach | Damage |
| --- | --- | --- |
| 0 | 2.4 m | ×1.00 |
| 250 | 2.7 m | ×1.07 |
| 500 | 3.0 m | ×1.15 |
| 750 | 3.3 m | ×1.22 |
| 999 | 3.6 m | ×1.30 |

**How it is drawn.** A war scythe is a haft with a curved blade set at the head, and it is
drawn as three pieces: the haft from the butt through her hands to the neck, and the blade
from the neck through a mid point to the tip, so its curve is visible. **It rides her
hands.** Through a swing both hands are on the haft — the right leads, the left trails, the
way the clips grip it — and the pole runs from behind the trailing hand through the leading
one to the tip; while the hit volume is out the tip sits exactly at the volume's far end, and
through the wind-up and the recovery it lies along the line of the two hands at the live
reach. So the weapon moves with the arms rather than jumping to a line the arms are not on.
At rest it **stands**: the haft upright with its butt on the floor under her left hand, its
head level with hers, and the blade leaving the head curving forward so its tip ends a little
over her head — the way a scythe is stood when it is not being swung. The standing weapon
grows with grey by the same factor as the reach, and the blade's breadth grows with it.
`view::scythe` is the geometry and `view/tests/kinematics.rs` checks the tip against the hit
volume, the leading hand against the haft, and the standing pose's butt on the floor and tip
over the head.

> **Was**: one bar from the hand straight out along the facing at the full reach, which read
> as a five-metre pole pointed at whoever she was looking at; then a haft and flat blade
> pinned to the world's axes, which read as a picket sign; then, for a day, carried low and
> horizontal, out of the crosshair but no longer reading as a scythe at all.

So grey is risk and power in one segment. A cast at full health opens a wound and lengthens
the blade; being hit does the same; drinking gives the power back and shortens it. The edge
she rides is *how much of my bar am I willing to leave open.*

The fade's first number was chosen so that a spike's worth of grey (its cost at full health,
90) survives one whole exchange — the spike's own startup to the end of its recovery plus a
dodge, 82 frames — with more than half of it left. `grey.rs` pins that relationship, not the number.

### Essence pools

**Every hit she lands spills the target.** What is left on the floor is not a puddle but a
**figure**: a shadowy column the size of the body it came out of, standing where their feet
were — under the fighter on a direct hit, under the struck part's floor projection on the
creature. Its **essence** is the damage dealt, and the essence decays at a constant rate
(`Pool drains`, 10 a second) until it is gone.

- **Size follows essence.** A full body's width and height at `Pool, full-sized at volume`
  (110, a spike's worth), shrinking as it drains and never below `Pool, smallest share of a
  body` (×0.35). It never stands taller or wider than the mage, and it does not spread over
  the ground.
- **Solidity follows essence too.** The renderer draws the figure faint when little is left
  and nearly solid when it is full, in six steps, so the figure on the floor *is* the heal it
  is worth.
- **They merge.** A hit onto a spot within a body of a pool of hers adds to it rather than
  standing a second figure beside it.
- **Cap of eight** per Blood mage (`Pools at once`); a ninth merges into the newest. Raised
  from four for the Haemorrhage's trail, which is several small pools by design.
- **They are effects**, in the fixed effect array — which grew from eight slots to twelve to
  make room for them beside a blade, a Grasp and a spike. Drawn at exactly the column they are
  tested at, in the arena and in the overlay. Nothing collides with them.
- **Her own blood never pools.** A cost is paid into the ability, not onto the floor. A hit
  on somebody in the air spills nowhere — the simpler of the two answers in the proposal's
  open question, kept until somebody plays it.
- **The creature bleeds too**, and a pool under a toppled Ridgeback is a door onto its back:
  see [Blink](#blink--the-mechanics-movement-shift--a-direction-with-the-crosshair-on-a-pool).

Measured, one cast landed on the dummy on bare floor:

| Move | Cost (% of red) | Dealt | Essence | Radius | Lives |
| --- | --- | --- | --- | --- | --- |
| Reaping sweep | 1 | 22 | 22 | 0.18 m | 64 f |
| Haemorrhage | 4 | 30, then 8 a tick | 30, then 8 a tick | 0.18 m | 180 f, then 48 f each |
| Bloodletter | 1 | 40 | 34 | 0.18 m | drunk by its own return |
| Grasp | 7 | 192 | 192 | 0.50 m | 1102 f |
| Black spike | 9 | 100 | 100 | 0.45 m | 524 f |

> **Was**, for a day: a disc on the floor with a radius of 0.24 m per root of the essence,
> which put a spike's pool across two and a half metres of arena and a Grasp's across three.
> Too big, and it spread; the figure replaced it on 2026-09-23.

### Double duty — the drink

An ability that **lands over a pool** does its ordinary job and *also* drinks — and **a
drink is one and done.** The pool is spent the moment a move is put through it, whatever
came back: what she gets is the move's share (`Drinks of a pool (%)`, a column in the move
table) of the essence *remaining* in the pool, converted out of grey and never past it, and
anything the share or the grey did not take is lost with the pool. A pool counts if the
victim is standing against it or the hit volume passes over it; the fullest one is drunk.
**The drink comes before the spill**, so a hit on bare floor returns nothing, and a hit that
drinks leaves only its own fresh, smaller figure behind. A hit an effect delivers -- an arm,
the bolt -- drinks only pools older than the effect, so the Grasp's four arms closing on one
spot take one share of the pool that was there and nothing of what they spill themselves.

**The scythe collects without a hit.** On every active frame of a sweep, every pool
of hers the blade's volume passes over is drunk, whether or not anybody was in the way — the
blade is the thing she puts through the blood. Only pools older than the swing count, so a
hit does not refund itself through the pool it just spilled; that one is left on the floor
for the next swing, or the spike. `World::collect_with_the_scythe`; `blink.rs` pins a sweep
through an empty pool.

| Move | Share of what is left | From a full pool (110), with grey to fill |
| --- | --- | --- |
| Reaping sweep | 50% | 54 |
| Haemorrhage | 40%, on the bolt's cut | 44 |
| Bloodletter | 30%, on the way home | 42 |
| Grasp | 60%, once at the catch, and again from the pool the victim is hauled onto | 66 |
| Black spike | 100%, and the pool erupts | 105 |

The shares are what make the sweep a bad thing to put through a big pool: it takes half and
wastes the rest, where the spike takes all of it. Grey is the ceiling: at full health a sweep
over a pool gets back exactly its own cost and the pool is gone all the same.
`crates/sim/tests/essence.rs` pins the two halves of the feel relationship — landed over a
pool of its own making, each of the sweep and the Bloodletter returns more than it cost;
landed on bare floor it returns nothing on the frame it lands — and that a drunk pool is
gone.

> **Was**, for a day: a drink took its share and the pool kept the rest, so a pool could be
> hit over and over for health. One and done since 2026-09-23.

### Naturally deals increased damage to disabled enemies

Kept exactly as built: ×1.4 to anything staggered, held, or a creature on its side. It is
what makes a Grasp a setup rather than a tax, and it multiplies the pool a spike leaves.

## What is bound

| Input | Move | What it is |
| --- | --- | --- |
| `L` | **Reaping sweep** | The auto. A war scythe drawn across the front. Reach grows with grey |
| `R` | **Haemorrhage** | A bolt that opens a bleed. Every tick of the bleed spills a pool under the victim, wherever they have got to |
| `M` | **Bloodletter** | A blade thrown a fixed distance and back, cutting on both passes |
| `Q` | **Grasp** | Hold to choose a depth; four arms converge. All four catch, bind, and haul the victim to her feet |
| `E` | **Black spike** | A spike out of the floor after a delay. On a pool, the pool erupts |
| `shift` + direction, crosshair on a pool | **Blink** | She is at the pool. Consumes it |

Five abilities, an auto, and the mechanic's own movement on the dodge. Right click is free on
this class (no shield) and takes the spell; middle click is the third click and takes the
throw.

**The move table is five rows in storage order, not button order.** The Oven packs the move
store in class order, so a slot appended to this class shifts every Dual mage index in
`tuned.rs` and a slot renumbered silently rewrites every number on the class. The four rows
that existed before the scythe kept their knobs — the Bloodletter is still slot 0, the
Haemorrhage took Rend's row (by way of the Reap) and was retuned rather than replaced — and
the sweep was appended as the fifth. See
`moves::blood`.

## Abilities

### Reaping sweep — auto, `L`
**Startup** 8 · **Active** 5 · **Recovery** 14 · **Damage** 22, growing with grey ·
**Reach** 2.4 m and **radius** 0.45 m, both growing with grey · **Cost** 1% of red ·
**Drinks** 50% of what is left · −10 on block, 0 on hit · **Repeat lockout** 100%

A flat swing across the front, 0.45 of a turn from the body's left to its right — the Dual
mage's wing reasoning with the shape turned into a weapon: a blade on a long haft covers width,
and a body that has closed inside the haft is inside the sweep. The **tip** hits harder:
anybody caught further from her feet than `Sweep, tip as a share of the reach` (65%) of the
blade's live length takes `Sweep, tip damage` (×1.5), decided per defender rather than per
frame, so one sweep can catch one body on the haft and another on the point.

Over a pool it is the trickle heal, thrown constantly — never the reason to make a pool, but
why standing in one while she trades is the right place for her and the wrong place for them.

Its clip is authored low to high, hands gathered off the left hip and swept across, which is
what tells it from the Dual mage's level Sweep at a glance; the hit volume itself is level, at
`sweep height`, because `Plane::Flat` is the one that owns the width of the front.

### Haemorrhage — spell, `R`
**Startup** 12 · **Active** 3 · **Recovery** 18 · **Damage** 30 on the bolt · **Reach** 9 m,
skillshot · **Cost** 4% of red · **Drinks** 40%, on the bolt's cut · **Bleed** 180 frames, 8 every 12

A bolt straight out along the crosshair, spent on the first body it reaches: it cuts, it
spills like any hit, and it opens a **bleed**. For three seconds the victim takes a tick every
fifth of a second, and **every tick spills a pool under them wherever they are standing**, so
a bleeding fighter who keeps moving walks a trail of her pools behind them, and one who stands
and fights grows one pool under their feet. Not a hit: a tick has no stun, no block and no
knockback, and it goes through a guard, because it was the bolt that had to land. A guarded
bolt is spent and opens nothing. The bleed cannot kill.

**It is the easier thing to land** — `Haemorrhage, bolt radius` (0.7 m) against the blade's
0.45, fast, and thrown at a body rather than led onto a spot the way a spike is — in a kit
where everything else can miss. Not easy: it still has to connect. And it is what **marks
somebody as the target**: any spike that lands on a bleeding fighter lands on a pool, so it
erupts, hits harder and heals more; and a spike on an *old* pool of the trail chains along it
— the ticks land `Bleed ticks every` (12 frames) apart, which at a walk is under a bare
spike's own radius, so each eruption covers the next pool — toward wherever the bleeding
fighter has got to. `crates/sim/tests/haemorrhage.rs` pins the bolt, the ticks, the trail's
spacing, the chain reaching them, and that a guard stops it.

The scythe stays standing under her other hand throughout: this is a spell, not a swing,
and its clip is one hand drawn in to the chest and thrust out flat, palm forward.

> Took the Reap's row, which took Rend's. The Reap was a heavier, unblockable overhead of the
> same scythe: a second swing on a class whose one swing already grows with grey, and the
> kit's two hard-to-land tools -- the spike at mid range, the Grasp up close -- had nothing
> easier beside them. The creature does not bleed; on a hunt the bolt is a cut and a gore.

### Bloodletter — throw, `M`
**Startup** 7 · **Active** 3 · **Recovery** 14 · **Damage** 20 a pass · **Reach** 7 m,
skillshot · **Cost** 1% of red · **Drinks** 30%, on the way home

A blade thrown along the crosshair to a fixed distance and back to her, cutting on the way out
and on the way home. Unchanged from the built kit except in two ways: it moved off the auto to
middle click, and **it brings back a cut, not health.** The leech on the catch is gone. Its
return leg crossing pools on the floor is a small drink on the way back — each pool once per
blade, and only the crossing: the version where its cuts also drank took three shares of one
pool in one throw and out-healed the rest of the kit, and was reverted (feel log, 2026-09-23).

### Grasp — special, `Q`
**Startup** 16 · **Active** 4 · **Recovery** 20 · **Damage** 40 an arm · **Reach** chosen by
the hold, 1.5–12 m · **Cost** 7% of red · **Drinks** 60%, at the catch and where the haul
ends · **Hold** 26 frames, 10 of them bound

Kept as built, because the built version is the best-argued ability in the class: hold to
choose a depth on the crosshair's line, four arms converge there, each arm damages and spills,
and only all four catch. The catch binds and then hauls at forty metres a second to **her
feet** — the victim arrives touching her, held, disabled, at ×1.4, on whatever she is standing
in, in reach of the sweep that is already winding up — **and the Grasp drinks the pool they
land on**. Two drinks, then: the pool under the victim where the arms closed, and the pool
under her feet when the haul ends, each only from pools older than the hold, so the arms'
own spill is never drunk back. "Grasp them onto the pool you are standing in" is the play,
and it pays. Everything about the channel and the catch
is as it was: see [Holding `Q` chooses the depth](#holding-q-chooses-the-depth) below, kept
from the previous revision.

Against something that cannot be hauled — a wall, a structure, the creature — the catch
**hauls her to it** instead: the arms find nobody to close on, so they pull her to the anchor
they met, behind `Grasp hauls her to an anchor` (on). On a Ridgeback that is a second route
up. See [Movement](#movement--two-answers-both-behind-a-flag-2026-09-17) below.

### Black spike — mechanic, `E`
**Startup** 30 · **Active** 4 · **Recovery** 22 · **Damage** 100 · **Reach** 9 m, grounded ·
**Cost** 9% of red · launches 9 m/s · slows

A spike out of the floor where the crosshair is, after the same long delay as before: the cast
is the telegraph. What erupts depends on the floor, decided the frame it comes up:

- **Bare ground:** a spike, and only a spike — drawn without a skirt, so a bare spike and an
  eruption are told apart from across the arena. The move's own disc does the damage, the
  launch and a short slow (`Black spike slow`), and it spills what it hits — how she seeds a
  pool at range where no scythe reaches. The launch and the spill are ordered so the pool
  lands under where the victim *stood*, not under where the launch has since put them.
- **A pool of hers:** the whole pool erupts, launching and slowing everything standing in it
  at the move's damage times `Black spike, eruption damage` (×1.5), and **drinks the entire
  pool** in one go. **And the eruption chains:** every other pool of hers inside its radius
  erupts too, at its own size, and each of those sets off whatever *it* covers. A spike on
  one pool is a bigger spike; a spike on one pool among several is the floor coming up
  across the whole fight, and arranging the pools for it is the skill curve the mechanic
  wants. Each eruption drinks its own pool first, so no blood is counted twice and the chain
  ends when the pools do. `World::erupt_from`; `blink.rs` pins a three-pool chain. The eruption is sized by the pool's essence — `Black spike, eruption
  radius per root of volume` (0.3) times the root of it, so a spike's worth of blood comes up
  across three metres against the bare spike's 1.6 — and stands `Black spike, eruption
  height` (3.4 m) tall against the bare spike's 2.2. The pool is spent whether or not she
  had grey to fill. The bigger the pool, the bigger the eruption, the harder it hits and the
  bigger the heal.

  > **Was**, for a day: the eruption was the pool's own radius and the spike's own damage,
  > which on a sweep's pool was *smaller* than the bare spike and looked identical to it.

**The drain field is gone**, and with it its three knobs (radius, lifetime, drain). Nothing
ticks. The spike stands for `Black spike, eruption lasts` (20 frames) as the thing you can see
from across the arena; an eruption is drawn with the disc it hit across, a bare spike without.

### Blink — the mechanic's movement, `shift` + a direction with the crosshair on a pool
**Startup** instant · **Recovery** the dodge's own · **Range** wherever a pool of hers is ·
**Cost** the pool

A dodge thrown with the reticle on one of her pools puts her in it, instantly, and the pool is
spent. It is the Reaver's forward dodge pointed at the class's object: `aim::pointing_at_disc`
— the ray from the eye through the crosshair against a short cylinder standing on the pool's
own disc, `Crosshair lock on a pool` tall — and `aim::clear_between` for a straight line from
her to it that nothing solid crosses. The nearest pool the crosshair is on wins. On the ground
and in the air; the airborne one spends the airdodge, and a pool is on the floor so it also
lands her. It is invulnerable for the dodge's own opening frames, the way every dodge is.

Refused with no pool under the crosshair (an ordinary dodge happens instead), with the line
blocked by a stone, and in the air after the airdodge is spent — `crates/sim/tests/blink.rs`.
Nobody else blinks to her pools.

**A pool is a heal or a door.** A blink eats the heal, and a player who blinks to every pool
never heals, while one who drinks every pool never moves. A pool draining is the other limit:
a door that is closing is a door you have to decide about now.

## Playing it

Cast to open a wound and grow the swing. Sweep to spill them. Read where the blood fell and
bend the fight back onto it: Grasp them onto the pool you are standing in, or Bloodletter them
where they stand to start one there, or Haemorrhage them and let their own feet lay the trail
the spike will run along. Blink to the pool they left behind and sweep them as they come back
for you. Cash grey in when it is about to fade or when you are about to die, and not before —
every point of grey you are carrying is reach, and width, and damage.

In a hunt the same sentence reads: hit the creature until the floor under it is red, ride the
topple for the ×1.4, and when it is on its side the pool under it is a door onto its back.
The scripted hunter (`cargo run -p hunt --bin fight -- --class blood`) throws the bolt when
the creature is open and drinks from the pools its sweeps leave; the report's **THE BLOOD**
section counts pools made, health drunk, and health drunk while the creature was on its side.

**Counterplay.** Do not stand in blood. Keep moving, so the pools she makes are small and far
apart and gone before she can use them. Punish her while she is grey: a Blood mage with a
long blade is a Blood mage with a short bar, and she cannot heal without going somewhere you
can see.

## Holding `Q` chooses the depth

The only move in the game you aim with *time*. Press and hold and a small marker leaves the
caster's chest and travels outward over half a second, from melee range out to ten metres; let
go and the arms converge on wherever it had got to. Hold past the end and it throws itself.

**The marker is not a projectile.** It is the far end of `Player::aim_path`, solved every
frame of the wind-up by the same `sim::aim` call the finished move uses. **The line is solved
at the move's full reach every frame, and the hold only picks a point along it**, which is what
keeps the marker from jumping about while the mouse is still; **the line is read for its
direction only**, so a full hold at a wall two metres away still reaches ten, through the
wall. Aiming stays live through the wind-up and locks on the frame the button comes up. The
health is paid on the press, because there is no cancelling out of a channel.

**The catch: bound, then hauled.** About a sixth of a second held exactly where the arms
closed, then dragged to her feet at forty metres a second, feet handed back the same frame the
haul ends. The whole catch is shorter than her most expensive cast takes to come out, on
purpose: whatever is supposed to meet them at the end of the trip has to have been committed
to before the Grasp was known to have landed.
`preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution` and
`a_grasp_always_finishes_hauling_before_it_lets_go` in `feel.rs` pin it.

## Open questions

Carried from the proposal, with what the build found beside each.

- **How fast does grey fade?** 12 a second. A spike's cost survives one exchange with more than
  half left; the first thing C1 will say is whether the blade ever gets long enough to
  matter.
- **How much reach does a full bar buy?** ×1.5, the design's first number, and now ×2 in
  width and ×1.6 in damage beside it. Read across the arena is the whole justification;
  nobody has looked at it from across the arena.
- **The bleed's trail is short-lived.** A tick's pool is eight essence and drains in under a
  second, so the live trail is the last four strides, not the whole walk. That is enough for
  a chain to cross, and a longer one would be a floor full of figures; if it turns out too
  short to read, the answer is a slower drain on a tick's pool, not a bigger tick.
- **The creature does not bleed.** It has no status to carry one; the bolt cuts and gores it
  and nothing more. Worth doing if hunts want the trail.
- **Blink consumes the pool.** As written. If the class turns out to have too few pools to
  spend one on moving, the middle answer is a blink that takes a share, the way the sweep
  does.
- **Should the blink cost health?** It does not. One more cost on an already expensive class.
- **Grasp onto her feet.** The haul's endpoint is touching distance, which for any pool
  wider than a body is inside it. Whether half a second of channel is worth that is C3's.
- **Pools on slopes and platforms.** A pool is spilled where the victim stands; a victim in
  the air spills nowhere, and a creature's blood falls to the arena floor whatever it stood
  on.
- **The figure.** The proposal's idea is that the essence starts as a shadow of the
  target's own model and coalesces into the figure. It is a column today, body-sized and
  shrinking; a copy of the target's skeleton fading into the floor is the next step and is
  presentation only.
- **Costs.** 1 / 4 / 1 / 7 / 9 percent of current red — at full health 10 / 40 / 10 / 70 /
  90 against a thousand-point bar, and a quarter of that at a quarter bar. The two
  relationships that bound them — one cast is never a third of the match, and a cast over
  its own pool returns more than it cost — both hold; the sweep's drink went to half so the
  second holds at full health, where its cost is the largest it can be.
- **The `leech` column stays in the move table.** It was to be removed, but the Dual mage's
  dark auto and tether heal through it and her test pins that; it is zero on every move of
  hers and `feel.rs` keeps it there.
- **The name of the essence.** Still "pool" and "essence" both.

## Was

The kit this replaced, 2026-09-12 to 2026-09-23. Four abilities, four buttons, one economy:

| Key | Ability | What it did |
| --- | --- | --- |
| `LMB` | Bloodletter | A blade out and back; the blood it cut was banked and paid on the catch (40%) |
| — | Rend | A raking claw at chest range, 30 health for half back; no button since 2026-09-16 |
| `RMB` | Reap | 2026-09-23 only: the scythe over and down, unblockable, 110 and the biggest drink. Replaced by the Haemorrhage the same day |
| `Q` | Grasp | As now, but the catch hauled to arm's length and each arm leeched 55% |
| `E` | Black spike | A spike in a draining, slowing field: 11 a tick for four seconds, 59% back |

**Cost on the press, leech on the hit.** Every ability cost health the moment the button went
down and returned a percentage of the damage it dealt, wherever the damage happened — a direct
hit immediately, a field on every tick, the blade when caught. The auto was the only one that
reliably profited; every committed ability was close to break even at its best and a straight
loss at its worst.

It turned out to be four abilities with a red colour scheme and a percentage. A number that
pays out wherever the hit lands gives the player nothing to *go to*, so a class whose
sentence was "sustain through aggression" shipped with three tools that rewarded standing
back. The rebuild's argument is in [../blood-mage.md](../blood-mage.md) §"Why this shape".
What survived: the Grasp entire, the spike's thirty-frame telegraph, the thrown blade, the
×1.4, and the rule that a cut can never kill her.

## Movement — two answers, both behind a flag, 2026-09-17

**This class had no movement at all, and it could not stay that way.** Every other class has
one thing it does with the ground: the Reaver crosses to her shadow, the Elementalist rides a
structure up, the Champion vaults, the Dual mage floats, the Bulwark leaps to its shield. The
Blood mage walks. That was survivable while a full hop reached seven and a half metres and
mobility was something everybody had for free; the jump was cut back across the cast on the
same day precisely so that the class techniques matter more, and it left her as the one
character with nothing to be good at.

Two answers are built. **Neither is chosen**, and both are live flags in the Oven under
**Blood mage** rather than compile-time features, so all four combinations are two sliders
away in a running match:

| Knob | Default | What it does |
| --- | --- | --- |
| `Grasp hauls her to an anchor` | **on** | A Grasp whose four arms converge on a wall, a structure or the creature — rather than on a person — pulls *her* to it |
| `Dodge is a blink` | off | Her dodge covers `Blink, how far` in `Blink, frames to cross`, flat, instead of a shove that decays |

### The generator, stated once

The Grasp haul is the better of the two, and the reason is worth naming because it is the test
every future candidate should be put to:

> **Movement should be a re-reading of an ability she already has, and it should be paid for in
> health.**

The Grasp already exists to close a gap — four arms converge on a point and whatever they
catch is hauled back to her. The haul is that same sentence with the subject swapped: catch a
person and it works as it always has, catch a wall and the arms have something to pull against
and nothing to pull. Nothing new was invented, and the price is already right — the health
went on the press and none of it comes back, because nothing was cut, so using the Grasp as a
grappling hook costs a whiffed cast every single time.

The blink fails the test on both halves. There is nothing in the kit it is a re-reading of,
and it leans entirely on the animation to make it feel like it belongs — which is a way of
saying the design is hoping to be rescued later. It is built anyway, because "does she just
need the distance?" is a real question and the cheapest way to answer it is to try it.

> **The v1 blink passes the test where this fails it, and it is built.** `shift` plus a
> direction with the crosshair on an **essence pool** puts her at the pool and consumes it —
> see [Blink](#blink--the-mechanics-movement-shift--a-direction-with-the-crosshair-on-a-pool)
> above. That is a re-reading of the mechanic, and it is paid for in the currency the class is
> made of, because a blink eats a heal. It takes precedence over this flag: with the crosshair
> on a pool the dodge is the pool blink, and only otherwise does the flag decide what a dodge
> is. What the flag can still settle is the *distance*.

### What changed with the haul

The Grasp's throw went from 10 m to 12 m and its damage from 48 to 40, which is the balance
payment for the second job. The v1 build had its own haul to the creature; it was folded into
this one when the two branches met, since an anchor here already includes the creature. `feel.rs` then caught the consequence: the hold is a fixed number
of frames and the haul has to cover the whole reach inside it, so a longer throw was dropping
a full-range *catch* halfway home — mid-air, mid-drag, suddenly able to walk. `Grab haul
speed` went up with it.

The haul stops at the **anchor**, not at the convergence point. The Grasp deliberately does
not shorten itself against the scenery — point at a near wall, wind to full, and the arms
close metres behind it — so without that she would arrive inside the arena.

### Other ideas, none of them built

Kept here because the two above are a first pass and the generator above produces more. Rough
order of how well each satisfies it.

1. **Ride the Bloodletter home.** The blade goes out a fixed distance and comes back, and the
   blood it took is paid **on the catch**. Let her take the ride instead: press again while it
   is outbound and she goes to the blade rather than the blade coming to her, forfeiting the
   payout. The cost is *the thing the ability is for*, which is the purest statement of this
   class there is, and it is on the auto — so it is movement she has every couple of seconds
   rather than movement she saves up for. Its own risk is that the auto is already the one
   reliably profitable thing in the kit, and making it the movement as well puts most of the
   class on one button.
2. **The leech pulls.** Give the blood that comes back a positional component: every hit that
   pays out drags her a fraction of the way toward what it came from. No input, no ability —
   a property of her damage, always on, and it only ever helps when she is connecting, which
   is exactly when the class is supposed to be winning. Its risk is the opposite of the
   blink's: a movement with no button is one nobody notices, and it may simply read as jank.
3. **The Grasp as a swing rather than a reel.** The ceiling version of what is built. The arms
   hold and she travels an arc, preserving and redirecting the speed she came in with instead
   of being placed at the far end. Same thematic tie, far more skill in it — and it is the one
   idea here that is not a re-reading, because a rope constraint is a primitive the simulation
   does not have.
4. **Recall to the spike.** Black spike already plants something at long range; a reactivation
   (`Move::reactivate`, zero everywhere and an open question in its own right) yanks her to
   it, cutting whatever is on the line. Proven shape — it is the Reaver's shadow — which is
   both the recommendation and the objection.
5. **Follow her own blood back.** She leaves a trail; the input snaps her back along it to
   where she stood a second or two ago. The most thematic idea on this list and the most
   novel, and the reason it is fifth is that it makes positional commitment *free*, which is
   the opposite of what a game built on whiff punishment wants.
6. **Buy the height.** A takeoff that costs health, with the rise scaling in what she spends.
   The only idea here that addresses **height at all** — every other one, including both of
   the built ones, is purely horizontal. If she turns out to need the vertical game rather
   than the ground one, this is the only candidate on the list.
7. **Spent blood makes her light.** Extra speed and lower gravity in proportion to how much
   health she has spent recently, decaying back. No input at all, and it turns her existing
   loop into her movement — but it is a stat rather than a technique, and a technique is what
   the jump nerf was making room for.
8. **Standable spikes.** Black spike's spike becomes a solid you can get on top of, like a
   structure. Reuses `stones::resolve_body` almost unchanged and gives her vertical reach for
   a telegraph and some health. Last because the thematic tie is thin: nothing about a blood
   spike says platform.

**What the pair that is built does not do is height.** Both answers are flat, deliberately —
what she is short of is ground, and a blink or a haul that also went up would be the second
jump the one class with no movement has not earned. Whether that is the right read is the
first thing to find out by playing it.
