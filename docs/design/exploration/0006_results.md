---
status: exploration — raw results
run: 2026-10-03
---

*What `python3 scripts/spread.py` printed over the full sweep (`target/spread`, about 258,000 hunts), for [0006_differences_in_kind.md](0006_differences_in_kind.md) §8. Knob identifiers are the Oven's; `ridgeback_·_mind.x` is the Ridgeback's mind family.*

# 0006 — Numbers as differences in kind — results

Baseline: 240 seeds x 6 classes. Signature: 264 components (Blood mage 45, Bulwark 45, Champion 40, Dual mage 44, Elementalist 43, Shadow Reaver 47), each in units of one seed's standard deviation.

Noise floor at 12 seeds: median distance 6.55, **N95 = 8.06** (clipped: 8.06). Difficulty noise at 12 seeds a class: sd(D) = 0.089.

## M0 · The contract rate

The Ridgeback as tuned, on the same 12 seeds, breaks: nothing. A genome *breaks the contract* if it breaks any clause the tuned creature keeps on these seeds.

| Regime | Genomes | Break the contract | Most broken clauses | Unresolved hunts |
| --- | --- | --- | --- | --- |
| mutate | 400 | 200 (50%) | threat bands 113, never idle 81, neither free nor hopeless 76, whole move set 39, poise breaks 31 | 1.8% |
| wild | 200 | 200 (100%) | never idle 200, threat bands 199, reactable mix 183, whole move set 174, poise breaks 158 | 50.3% |

## The control: tempers

Tempers change only how the creature thinks (glance, lead, decisiveness, strain), and were built to be the same fight, harder. If the measure calls them a different fight, it is too sensitive.

| Temper | Difficulty change (noise sd) | Distance / N95 | Orthogonal / N95 | Clipped distance / clipped N95 |
| --- | --- | --- | --- | --- |
| 1 | +2.0 | 1.21 | 1.21 | 0.97 |
| 2 | +4.7 | 1.23 | 1.19 | 1.23 |
| 3 | +5.9 | 1.40 | 1.37 | 1.40 |

## M1 · Separability at matched difficulty

Pairs of genomes whose difficulty differs by less than half its own noise (0.045), and how far apart their fights are. Kind rate: the share of those pairs further apart than N95.

*Clipped* uses the clipped distance against its own noise floor. *Beyond temper III* counts pairs whose shift orthogonal to difficulty is larger than the hardest temper's.

| Regime | Spread of D (sd) | Matched pairs | Median distance / N95 | Kind rate | Orthogonal | Clipped | Beyond temper III |
| --- | --- | --- | --- | --- | --- | --- | --- |
| mutate | 2.8 x noise | 8660 | 1.81 | **100%** | 100% | 100% | 90% |
| wild | 9.6 x noise | 902 | 19.93 | **94%** | 94% | 93% | 93% |
| mutate, contract kept | 2.2 x noise | 2535 | 1.68 | **100%** | 100% | 100% | 85% |

## Two lenses a player would recognise

**Move mix**: the share of the creature's moves that differ between two fights (total variation, 0 to 1); noise at 12 seeds a class reaches 0.067 (95th percentile). **Class wins**: how far the six classes' win rates move between them, summed (0 to 6); noise reaches 0.67.

| Control | Move mix differs | Class wins move |
| --- | --- | --- |
| temper 1 | 0.034 | 0.42 |
| temper 2 | 0.076 | 0.92 |
| temper 3 | 0.120 | 1.25 |

| Regime | Matched pairs | Median move-mix difference | Beyond noise | Over a tenth of moves | Median class-win shift | Beyond noise |
| --- | --- | --- | --- | --- | --- | --- |
| mutate | 8660 | 0.115 | 93% | 64% | 0.42 | 20% |
| wild | 902 | 0.561 | 93% | 93% | 1.08 | 60% |
| mutate, contract kept | 2535 | 0.108 | 91% | 58% | 0.33 | 14% |

## M2 · Identifiability from one run

Hold out one seed of a genome; name the genome from that single run (six hunts, one a class) by the nearest genome mean in its difficulty band. Chance is one over the band.

| Regime | Band size | Accuracy | Chance | Times chance |
| --- | --- | --- | --- | --- |
| mutate | 10 | 68% | 10% | **6.8x** |
| wild | 10 | 89% | 10% | **8.9x** |
| mutate, contract kept | 10 | 66% | 10% | **6.6x** |

### The most separated matched pairs (mutate)

- **281 and 54**: distance 136.6 (16.9 x N95; clipped 2.6 x), difficulty -0.36 and -0.38. Four components carry 96% of it: Bulwark:open_mean +106.3, Bulwark:ride_mean -79.5, Bulwark:open_pm -16.3, Bulwark:idle -9.8.
  - wins Bl0 Bu12 Ch0 Du0 El0 Sh5; most thrown move 3 18%, move 2 17%, move 4 17%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->44246, ridgeback.spike_spray.roots_for 120->153, ridgeback_·_hide.nape_(x_damage) 157286->184126, ridgeback.walk_speed 458752->325993, ridgeback.flinch_length 40->49
  - wins Bl0 Bu11 Ch1 Du0 El0 Sh9; most thrown move 1 16%, move 7 16%, move 0 15%; biggest changes: ridgeback.tail_sweep.hitstun 100->120, ridgeback.spike_spray.hit_travels_at 3932160->3051098, ridgeback_·_legs.speed_per_break_(x) 53740->62238, ridgeback.walk_speed 458752->321716, ridgeback.rear_and_slam.knockback 917504->1168810
- **281 and 278**: distance 132.1 (16.4 x N95; clipped 2.5 x), difficulty -0.36 and -0.33. Four components carry 96% of it: Bulwark:open_mean +106.3, Bulwark:ride_mean -71.2, Bulwark:idle -14.8, Bulwark:open_pm -14.1.
  - wins Bl0 Bu12 Ch0 Du0 El0 Sh5; most thrown move 3 18%, move 2 17%, move 4 17%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->44246, ridgeback.spike_spray.roots_for 120->153, ridgeback_·_hide.nape_(x_damage) 157286->184126, ridgeback.walk_speed 458752->325993, ridgeback.flinch_length 40->49
  - wins Bl0 Bu9 Ch2 Du0 El0 Sh7; most thrown move 4 18%, move 0 16%, move 1 15%; biggest changes: ridgeback.size_(x) 65536->51656, ridgeback.charge.ideal_range 851968->1071487, ridgeback.bite.hitstun 30->39, ridgeback.charge.advance_while_active 1966080->1684096, ridgeback_·_mind.lead_on_the_target_(x) 65536->59769
- **249 and 281**: distance 113.9 (14.1 x N95; clipped 2.5 x), difficulty -0.32 and -0.36. Four components carry 94% of it: Bulwark:open_mean -107.5, Bulwark:open_pm +15.6, Bulwark:idle +14.6, Bulwark:ride_mean +14.2.
  - wins Bl1 Bu9 Ch6 Du0 El0 Sh5; most thrown move 2 17%, move 4 16%, move 0 15%; biggest changes: ridgeback_·_legs.speed_per_break_(x) 53740->65536, ridgeback_·_mind.repeat_penalty 1100->883, ridgeback.spike_spray.hit_travels_at 3932160->4567442, ridgeback.walk_speed 458752->324338, ridgeback_·_nerve.slow_it_actually_feels_(x) 29491->35476
  - wins Bl0 Bu12 Ch0 Du0 El0 Sh5; most thrown move 3 18%, move 2 17%, move 4 17%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->44246, ridgeback.spike_spray.roots_for 120->153, ridgeback_·_hide.nape_(x_damage) 157286->184126, ridgeback.walk_speed 458752->325993, ridgeback.flinch_length 40->49
- **260 and 281**: distance 113.8 (14.1 x N95; clipped 2.5 x), difficulty -0.36 and -0.36. Four components carry 95% of it: Bulwark:open_mean -107.5, Bulwark:open_pm +16.3, Bulwark:idle +15.7, Bulwark:ride_mean +15.3.
  - wins Bl0 Bu11 Ch6 Du0 El0 Sh4; most thrown move 4 17%, move 2 17%, move 0 15%; biggest changes: ridgeback_·_mind.decisiveness_(%) 62->44, ridgeback.charge.knockback 1179648->838792, ridgeback_·_mind.repeat_penalty 1100->840, ridgeback.spike_spray.hit_travels_at 3932160->4620296, ridgeback_·_legs.broken_leg,_knee_fold 22282->18715
  - wins Bl0 Bu12 Ch0 Du0 El0 Sh5; most thrown move 3 18%, move 2 17%, move 4 17%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->44246, ridgeback.spike_spray.roots_for 120->153, ridgeback_·_hide.nape_(x_damage) 157286->184126, ridgeback.walk_speed 458752->325993, ridgeback.flinch_length 40->49
- **281 and 190**: distance 113.6 (14.1 x N95; clipped 2.6 x), difficulty -0.36 and -0.32. Four components carry 94% of it: Bulwark:open_mean +106.8, Bulwark:ride_mean -18.5, Bulwark:idle -15.2, Bulwark:open_pm -14.0.
  - wins Bl0 Bu12 Ch0 Du0 El0 Sh5; most thrown move 3 18%, move 2 17%, move 4 17%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->44246, ridgeback.spike_spray.roots_for 120->153, ridgeback_·_hide.nape_(x_damage) 157286->184126, ridgeback.walk_speed 458752->325993, ridgeback.flinch_length 40->49
  - wins Bl0 Bu11 Ch3 Du0 El0 Sh6; most thrown move 4 18%, move 2 16%, move 5 14%; biggest changes: ridgeback_·_mind.switch_targets_when_nearer_than_(x) 39322->49148, ridgeback.size_(x) 65536->48223, ridgeback_·_nerve.root,_per_grab_frame_(x) 39322->50453, ridgeback.matches_a_fleeing_target's_speed_(x) 65536->79631, ridgeback.bite.damage 190->241

### The most separated matched pairs (wild)

- **104 and 165**: distance 2630.6 (326.5 x N95; clipped 3.3 x), difficulty -1.87 and -1.88. Four components carry 99% of it: Blood mage:ride_mean +1802.9, Bulwark:ride_mean +1325.0, Dual mage:ride_mean +985.0, Elementalist:ride_mean +949.1.
  - wins Bl1 Bu0 Ch12 Du0 El3 Sh8; most thrown move 7 78%, move 0 14%, move 3 6%; biggest changes: ridgeback.tail_sweep.advance_while_active 0->5834539, ridgeback.rear_and_slam.active 4->1149, ridgeback.spike_spray.appetite_per_rider 0->3796, ridgeback.spike_spray.active 16->1126, ridgeback.damage_that_flinches_it 120->1937
  - wins Bl0 Bu0 Ch12 Du8 El1 Sh11; most thrown move 0 73%, move 5 15%, move 4 12%; biggest changes: ridgeback.tail_sweep.hit,_lowest 0->2581912, ridgeback.charge.roots_for 0->292, ridgeback.shake.hit_travels_at 0->5669083, ridgeback.shake.startup 40->1171, ridgeback.back_kick.appetite_per_rider 0->3744
- **104 and 144**: distance 2606.6 (323.5 x N95; clipped 3.3 x), difficulty -1.87 and -1.91. Four components carry 100% of it: Blood mage:ride_mean +1772.0, Bulwark:ride_mean +1325.5, Dual mage:ride_mean +980.4, Elementalist:ride_mean +953.2.
  - wins Bl1 Bu0 Ch12 Du0 El3 Sh8; most thrown move 7 78%, move 0 14%, move 3 6%; biggest changes: ridgeback.tail_sweep.advance_while_active 0->5834539, ridgeback.rear_and_slam.active 4->1149, ridgeback.spike_spray.appetite_per_rider 0->3796, ridgeback.spike_spray.active 16->1126, ridgeback.damage_that_flinches_it 120->1937
  - wins Bl0 Bu12 Ch10 Du0 El0 Sh12; most thrown move 2 52%, move 4 11%, move 5 10%; biggest changes: ridgeback.shake.damage 0->893, ridgeback.bite.roots_for 0->289, ridgeback.tail_sweep.recovery 30->1181, ridgeback.shake.appetite 0->3794, ridgeback.tail_sweep.hit,_lowest 0->2456016
- **104 and 73**: distance 2576.3 (319.8 x N95; clipped 3.7 x), difficulty -1.87 and -1.84. Four components carry 100% of it: Blood mage:ride_mean +1804.0, Bulwark:ride_mean +1272.3, Dual mage:ride_mean +949.6, Elementalist:ride_mean +916.5.
  - wins Bl1 Bu0 Ch12 Du0 El3 Sh8; most thrown move 7 78%, move 0 14%, move 3 6%; biggest changes: ridgeback.tail_sweep.advance_while_active 0->5834539, ridgeback.rear_and_slam.active 4->1149, ridgeback.spike_spray.appetite_per_rider 0->3796, ridgeback.spike_spray.active 16->1126, ridgeback.damage_that_flinches_it 120->1937
  - wins Bl0 Bu9 Ch12 Du1 El1 Sh12; most thrown move 1 28%, move 2 21%, move 6 18%; biggest changes: ridgeback.rear_and_slam.hit_travels_at 0->5743495, ridgeback.bite.startup 30->1198, ridgeback.spike_spray.hit,_lowest 0->2487598, ridgeback.charge.hit_travels_at 0->5573888, ridgeback.tail_sweep.hit_travels_at 0->5564144
- **104 and 33**: distance 2260.9 (280.6 x N95; clipped 3.1 x), difficulty -1.87 and -1.84. Four components carry 99% of it: Blood mage:ride_mean +1804.0, Bulwark:ride_mean +1325.5, Elementalist:ride_mean -235.2, Bulwark:open_mean +116.0.
  - wins Bl1 Bu0 Ch12 Du0 El3 Sh8; most thrown move 7 78%, move 0 14%, move 3 6%; biggest changes: ridgeback.tail_sweep.advance_while_active 0->5834539, ridgeback.rear_and_slam.active 4->1149, ridgeback.spike_spray.appetite_per_rider 0->3796, ridgeback.spike_spray.active 16->1126, ridgeback.damage_that_flinches_it 120->1937
  - wins Bl0 Bu12 Ch2 Du0 El0 Sh12; most thrown move 1 91%, move 2 8%, move 7 1%; biggest changes: ridgeback.stomp.hit,_lowest 0->2607929, ridgeback.spike_spray.active 16->1196, ridgeback.spike_spray.advance_while_active 0->5795376, ridgeback.shake.launch 0->2570449, ridgeback.charge.active 40->1200
- **21 and 167**: distance 1644.7 (204.1 x N95; clipped 3.7 x), difficulty -1.66 and -1.62. Four components carry 95% of it: Elementalist:ride_mean +1301.6, Dual mage:ride_mean +803.9, Bulwark:ride_mean +457.9, Champion:open_mean +181.2.
  - wins Bl0 Bu9 Ch0 Du12 El0 Sh0; most thrown move 1 49%, move 5 48%, move 6 3%; biggest changes: ridgeback.back_kick.roots_for 0->299, ridgeback.stomp.appetite_per_rider 0->3804, ridgeback.shake.hit,_lowest 0->2463789, ridgeback.tail_sweep.roots_for 0->262, ridgeback.shake.startup 40->1087
  - wins Bl0 Bu0 Ch0 Du0 El0 Sh0; most thrown move 0 0%, move 1 0%, move 2 0%; biggest changes: ridgeback.spike_spray.advance_while_active 0->5871566, ridgeback.back_kick.roots_for 0->289, ridgeback.tail_sweep.hit_travels_at 0->5654602, ridgeback.back_kick.active 4->1137, ridgeback.tail_sweep.active 10->1129

### The most separated matched pairs (mutate, contract kept)

- **97 and 54**: distance 83.2 (10.3 x N95; clipped 2.6 x), difficulty -0.41 and -0.38. Four components carry 92% of it: Bulwark:ride_mean -78.0, Bulwark:open_pm -9.5, Bulwark:thr0 +9.0, Bulwark:thr1 -8.7.
  - wins Bl0 Bu9 Ch4 Du0 El0 Sh10; most thrown move 5 19%, move 4 18%, move 2 16%; biggest changes: ridgeback_·_mind.windup_follows_you_(x_turn_rate) 58982->46650, ridgeback_·_mind.switch_targets_when_nearer_than_(x) 39322->27959, ridgeback_·_pose.shake_force_(x) 91750->67845, ridgeback.turn_bleed-off,_committed 56361->49155, ridgeback.rear_and_slam.knockback 917504->1153843
  - wins Bl0 Bu11 Ch1 Du0 El0 Sh9; most thrown move 1 16%, move 7 16%, move 0 15%; biggest changes: ridgeback.tail_sweep.hitstun 100->120, ridgeback.spike_spray.hit_travels_at 3932160->3051098, ridgeback_·_legs.speed_per_break_(x) 53740->62238, ridgeback.walk_speed 458752->321716, ridgeback.rear_and_slam.knockback 917504->1168810
- **54 and 311**: distance 81.8 (10.2 x N95; clipped 2.5 x), difficulty -0.38 and -0.37. Four components carry 93% of it: Bulwark:ride_mean +77.7, Bulwark:thr0 -8.6, Bulwark:open_pm +7.2, Bulwark:thr1 +6.7.
  - wins Bl0 Bu11 Ch1 Du0 El0 Sh9; most thrown move 1 16%, move 7 16%, move 0 15%; biggest changes: ridgeback.tail_sweep.hitstun 100->120, ridgeback.spike_spray.hit_travels_at 3932160->3051098, ridgeback_·_legs.speed_per_break_(x) 53740->62238, ridgeback.walk_speed 458752->321716, ridgeback.rear_and_slam.knockback 917504->1168810
  - wins Bl0 Bu0 Ch8 Du0 El0 Sh10; most thrown move 4 18%, move 2 16%, move 0 15%; biggest changes: ridgeback_·_nerve.slow_it_actually_feels_(x) 29491->23070, ridgeback.walk_acceleration 1048576->808985, ridgeback.shake.appetite_per_rider 1400->1743, ridgeback.approach_gain 131072->105399, ridgeback.preferred_distance 458752->388153
- **278 and 311**: distance 72.8 (9.0 x N95; clipped 2.2 x), difficulty -0.33 and -0.37. Four components carry 93% of it: Bulwark:ride_mean +69.4, Elementalist:topple_pm +8.7, Bulwark:thr1 +5.1, Bulwark:thr0 -5.0.
  - wins Bl0 Bu9 Ch2 Du0 El0 Sh7; most thrown move 4 18%, move 0 16%, move 1 15%; biggest changes: ridgeback.size_(x) 65536->51656, ridgeback.charge.ideal_range 851968->1071487, ridgeback.bite.hitstun 30->39, ridgeback.charge.advance_while_active 1966080->1684096, ridgeback_·_mind.lead_on_the_target_(x) 65536->59769
  - wins Bl0 Bu0 Ch8 Du0 El0 Sh10; most thrown move 4 18%, move 2 16%, move 0 15%; biggest changes: ridgeback_·_nerve.slow_it_actually_feels_(x) 29491->23070, ridgeback.walk_acceleration 1048576->808985, ridgeback.shake.appetite_per_rider 1400->1743, ridgeback.approach_gain 131072->105399, ridgeback.preferred_distance 458752->388153
- **278 and 306**: distance 70.4 (8.7 x N95; clipped 2.0 x), difficulty -0.33 and -0.29. Four components carry 95% of it: Bulwark:ride_mean +67.8, Elementalist:topple_pm +8.7, Shadow Reaver:ride_mean +4.5, Shadow Reaver:mix6 +4.4.
  - wins Bl0 Bu9 Ch2 Du0 El0 Sh7; most thrown move 4 18%, move 0 16%, move 1 15%; biggest changes: ridgeback.size_(x) 65536->51656, ridgeback.charge.ideal_range 851968->1071487, ridgeback.bite.hitstun 30->39, ridgeback.charge.advance_while_active 1966080->1684096, ridgeback_·_mind.lead_on_the_target_(x) 65536->59769
  - wins Bl0 Bu4 Ch5 Du0 El0 Sh9; most thrown move 0 17%, move 4 16%, move 5 14%; biggest changes: ridgeback_·_legs.broken_leg,_knee_fold 22282->28308, ridgeback_·_legs.speed_per_break_(x) 53740->65536, ridgeback_·_mind.repeat_penalty 1100->776, ridgeback_·_legs.corner_drop_per_break 78643->99350, ridgeback.back_kick.knockback 983040->728942
- **5 and 278**: distance 65.0 (8.1 x N95; clipped 2.3 x), difficulty -0.29 and -0.33. Four components carry 91% of it: Bulwark:ride_mean -61.1, Elementalist:topple_pm -8.7, Elementalist:hit6 -6.5, Bulwark:thr1 -4.2.
  - wins Bl0 Bu4 Ch1 Du0 El1 Sh11; most thrown move 5 17%, move 4 17%, move 3 13%; biggest changes: ridgeback_·_legs.broken_leg,_knee_fold 22282->17782, ridgeback.shake.bearing_tolerance 262144->200826, ridgeback.size_(x) 65536->47566, ridgeback.charge.ideal_range 851968->603097, ridgeback_·_legs.broken_leg,_hip_share_(x) 45875->58219
  - wins Bl0 Bu9 Ch2 Du0 El0 Sh7; most thrown move 4 18%, move 0 16%, move 1 15%; biggest changes: ridgeback.size_(x) 65536->51656, ridgeback.charge.ideal_range 851968->1071487, ridgeback.bite.hitstun 30->39, ridgeback.charge.advance_while_active 1966080->1684096, ridgeback_·_mind.lead_on_the_target_(x) 65536->59769

## M3 · Which knobs carry kind

Every knob alone, at seven levels across its range and five across +-30% of tuned. *Kind* is the largest shift of the fight orthogonal to the difficulty direction, in N95 units; *degree* the largest change of difficulty in noise sds; *kind at matched D* the largest orthogonal shift among levels within one noise sd of the tuned difficulty. A knob is **kind** if its orthogonal shift passes the bar -- the larger of the noise a best-of-eleven reaches 95% of the time (1.13 x N95) and the hardest temper's own orthogonal shift (1.37 x N95) -- **degree** if not but its difficulty moves by more than three sds, **dead** otherwise; **pure kind** if it passes the bar at a level within one sd of the tuned difficulty. A **cliff** is one step between adjacent range levels carrying over half the sweep's whole movement, and more than the bar.

Of 247 knobs: 94 kind (28 of them pure), 11 degree, 142 dead; 6 with a cliff.

| Group | Knobs | Kind | Pure kind | Degree | Dead |
| --- | --- | --- | --- | --- | --- |
| move: hit volume and travel | 56 | 8 | 2 | 4 | 44 |
| move: what a hit does | 48 | 8 | 3 | 2 | 38 |
| move: when it is chosen | 32 | 16 | 4 | 2 | 14 |
| move: frame data | 24 | 23 | 11 | 0 | 1 |
| move: appetite | 16 | 9 | 2 | 0 | 7 |
| movement | 14 | 7 | 0 | 0 | 7 |
| mind | 14 | 6 | 2 | 1 | 7 |
| hide | 12 | 5 | 2 | 0 | 7 |
| health, poise, stagger | 8 | 7 | 1 | 0 | 1 |
| move: lockout | 8 | 2 | 1 | 0 | 6 |
| nerve | 7 | 1 | 0 | 0 | 6 |
| lame layer | 6 | 1 | 0 | 1 | 4 |
| size | 1 | 1 | 0 | 0 | 0 |
| shake force | 1 | 0 | 0 | 1 | 0 |

Strongest kind knobs, by their shift at matched difficulty:

| Knob | Kind (x N95) | Kind at matched D (x N95) | Degree (sd) | Cliff |
| --- | --- | --- | --- | --- |
| ridgeback.shake.recovery | 17.18 | 9.40 | 3.3 |  |
| ridgeback.charge.recovery | 7.15 | 7.15 | 2.7 |  |
| ridgeback.rear_and_slam.recovery | 10.94 | 5.90 | 6.7 |  |
| ridgeback.spike_spray.active | 6.90 | 5.69 | 3.7 |  |
| ridgeback.tail_sweep.active | 5.62 | 5.31 | 3.0 |  |
| ridgeback.bite.recovery | 5.05 | 5.05 | 2.8 |  |
| ridgeback.tail_sweep.startup | 3.35 | 3.35 | 2.3 |  |
| ridgeback.shake.appetite | 3.21 | 3.21 | 2.0 |  |
| ridgeback_·_mind.turns_to_face_you_after_a_rear_move | 2.91 | 2.91 | 2.5 |  |
| ridgeback.spike_spray.startup | 4.07 | 2.83 | 6.0 |  |
| ridgeback.bite.appetite_per_rider | 128.58 | 1.97 | 8.7 | 2000 -> 2666 |
| ridgeback.back_kick.active | 1.99 | 1.97 | 1.2 |  |
| ridgeback_·_hide.shoulders_(x_damage) | 1.94 | 1.94 | 1.2 |  |
| ridgeback.spike_spray.hit_radius | 1.83 | 1.81 | 4.7 |  |
| ridgeback.back_kick.damage | 2.08 | 1.73 | 1.1 |  |
| ridgeback.rear_and_slam.damage | 1.66 | 1.66 | 1.6 |  |
| ridgeback.rear_and_slam.lockout_after_use | 1.64 | 1.64 | 1.0 |  |
| ridgeback.tail_sweep.ideal_range | 1.64 | 1.64 | 3.1 |  |
| ridgeback.bite.startup | 2.20 | 1.61 | 3.5 |  |
| ridgeback.stumble_length | 3.01 | 1.60 | 4.2 |  |
| ridgeback_·_hide.foot_(x_damage) | 2.93 | 1.54 | 9.3 |  |
| ridgeback_·_mind.frames_between_glances | 1.62 | 1.49 | 4.0 |  |
| ridgeback.tail_sweep.bearing_tolerance | 1.57 | 1.43 | 2.2 |  |
| ridgeback.rear_and_slam.range_tolerance | 2.24 | 1.42 | 2.3 |  |
| ridgeback.back_kick.recovery | 1.38 | 1.38 | 1.2 |  |

Strongest degree knobs (by difficulty moved):

| Knob | Degree (sd) | Kind (x N95) | Label |
| --- | --- | --- | --- |
| ridgeback.health | 25.9 | 2.94 | kind |
| ridgeback.damage_that_flinches_it | 16.8 | 16.64 | kind |
| ridgeback.back_kick.appetite_per_rider | 11.2 | 128.70 | kind |
| ridgeback_·_mind.pause_between_moves | 10.2 | 10.43 | kind |
| ridgeback_·_hide.foot_(x_damage) | 9.3 | 2.93 | kind |
| ridgeback.tail_sweep.appetite_per_rider | 9.2 | 134.78 | kind |
| ridgeback.bite.appetite_per_rider | 8.7 | 128.58 | kind |
| ridgeback.size_(x) | 7.1 | 3.40 | kind |
| ridgeback.rear_and_slam.recovery | 6.7 | 10.94 | kind |
| ridgeback.turn_bleed-off,_committed | 6.3 | 1.47 | kind |
| ridgeback.poise | 6.1 | 6.85 | kind |
| ridgeback.spike_spray.startup | 6.0 | 4.07 | kind |
| ridgeback.breakable_part_health | 5.9 | 2.41 | kind |
| ridgeback.spike_spray.advance_while_active | 5.4 | 1.56 | kind |
| ridgeback_·_hide.upper_leg_(x_damage) | 5.4 | 1.63 | kind |

Pure kind (28) -- a different fight at the same difficulty: ridgeback.back_kick.active, ridgeback.back_kick.damage, ridgeback.back_kick.recovery, ridgeback.bite.appetite_per_rider, ridgeback.bite.hit,_sideways, ridgeback.bite.recovery, ridgeback.bite.startup, ridgeback.bite.wants_the_target_at_(cos), ridgeback.charge.recovery, ridgeback.rear_and_slam.damage, ridgeback.rear_and_slam.lockout_after_use, ridgeback.rear_and_slam.range_tolerance, ridgeback.rear_and_slam.recovery, ridgeback.shake.appetite, ridgeback.shake.recovery, ridgeback.spike_spray.active, ridgeback.spike_spray.hit_radius, ridgeback.spike_spray.startup, ridgeback.stumble_length, ridgeback.tail_sweep.active, ridgeback.tail_sweep.bearing_tolerance, ridgeback.tail_sweep.damage, ridgeback.tail_sweep.ideal_range, ridgeback.tail_sweep.startup, ridgeback_·_hide.foot_(x_damage), ridgeback_·_hide.shoulders_(x_damage), ridgeback_·_mind.frames_between_glances, ridgeback_·_mind.turns_to_face_you_after_a_rear_move

Kind, but only with a change of difficulty (66): ridgeback.back_kick.appetite_per_rider, ridgeback.bite.active, ridgeback.bite.appetite, ridgeback.bite.bearing_tolerance, ridgeback.bite.damage, ridgeback.bite.ideal_range, ridgeback.bite.range_tolerance, ridgeback.braking,_committed_(m/s2), ridgeback.breakable_part_health, ridgeback.charge.active, ridgeback.charge.advance_while_active, ridgeback.charge.appetite_per_rider, ridgeback.charge.bearing_tolerance, ridgeback.charge.damage, ridgeback.charge.hit_radius, ridgeback.charge.ideal_range, ridgeback.charge.range_tolerance, ridgeback.charge.startup, ridgeback.charge.wants_the_target_at_(cos), ridgeback.damage_that_flinches_it, ridgeback.flinch_length, ridgeback.health, ridgeback.launching_into_a_charge_(m/s2), ridgeback.poise, ridgeback.rear_and_slam.active, ridgeback.rear_and_slam.advance_while_active, ridgeback.rear_and_slam.appetite_per_rider, ridgeback.rear_and_slam.startup, ridgeback.shake.active, ridgeback.shake.advance_while_active, ridgeback.shake.appetite_per_rider, ridgeback.shake.damage, ridgeback.shake.lockout_after_use, ridgeback.shake.startup, ridgeback.size_(x), ridgeback.spike_spray.advance_while_active, ridgeback.spike_spray.damage, ridgeback.spike_spray.recovery, ridgeback.stomp.active, ridgeback.stomp.advance_while_active, ridgeback.stomp.appetite, ridgeback.stomp.bearing_tolerance, ridgeback.stomp.damage, ridgeback.stomp.ideal_range, ridgeback.stomp.range_tolerance, ridgeback.stomp.recovery, ridgeback.stomp.startup, ridgeback.stomp.wants_the_target_at_(cos), ridgeback.tail_sweep.appetite_per_rider, ridgeback.tail_sweep.range_tolerance, ridgeback.tail_sweep.recovery, ridgeback.topple_length, ridgeback.turn_acceleration, ridgeback.turn_bleed-off,_committed, ridgeback.turn_gain, ridgeback.turn_per_broken_leg_(x), ridgeback.turn_rate_cap_(turns/s), ridgeback_·_hide.barrel_(x_damage), ridgeback_·_hide.ridge_(x_damage), ridgeback_·_hide.upper_leg_(x_damage), ridgeback_·_legs.pitch_per_break, ridgeback_·_mind.decisiveness_(%), ridgeback_·_mind.lead_on_the_target_(x), ridgeback_·_mind.pause_between_moves, ridgeback_·_mind.windup_follows_you_(x_turn_rate), ridgeback_·_nerve.stumble_per_launch_(frames/mps)

Degree (11): ridgeback.bite.advance_while_active, ridgeback.charge.hit,_forward, ridgeback.charge.hit_travels_at, ridgeback.spike_spray.wants_the_target_at_(cos), ridgeback.stomp.hitstun, ridgeback.stomp.roots_for, ridgeback.tail_sweep.hit,_forward, ridgeback.tail_sweep.wants_the_target_at_(cos), ridgeback_·_legs.broken_leg,_hip_share_(x), ridgeback_·_mind.lead_horizon,_prowling, ridgeback_·_pose.shake_force_(x)

The four knobs a temper turns: frames_between_glances kind (pure), kind 1.62, degree 4.0; lead_on_the_target_(x) kind, kind 1.46, degree 4.2; decisiveness_(%) kind, kind 1.53, degree 2.7; thresholds_fall_by_(%) dead, kind 0.08, degree 0.4


Dead (142): ridgeback.approach_gain, ridgeback.back_kick.advance_while_active, ridgeback.back_kick.appetite, ridgeback.back_kick.bearing_tolerance, ridgeback.back_kick.blockstun, ridgeback.back_kick.hit,_forward, ridgeback.back_kick.hit,_highest, ridgeback.back_kick.hit,_lowest, ridgeback.back_kick.hit,_sideways, ridgeback.back_kick.hit_radius, ridgeback.back_kick.hit_travels_at, ridgeback.back_kick.hitstun, ridgeback.back_kick.ideal_range, ridgeback.back_kick.knockback, ridgeback.back_kick.launch, ridgeback.back_kick.lockout_after_use, ridgeback.back_kick.range_tolerance, ridgeback.back_kick.roots_for, ridgeback.back_kick.startup, ridgeback.back_kick.wants_the_target_at_(cos), ridgeback.backing-off_speed, ridgeback.bite.blockstun, ridgeback.bite.hit,_forward, ridgeback.bite.hit,_highest, ridgeback.bite.hit,_lowest, ridgeback.bite.hit_radius, ridgeback.bite.hit_travels_at, ridgeback.bite.hitstun, ridgeback.bite.knockback, ridgeback.bite.launch, ridgeback.bite.lockout_after_use, ridgeback.bite.roots_for, ridgeback.charge.appetite, ridgeback.charge.blockstun, ridgeback.charge.hit,_highest, ridgeback.charge.hit,_lowest, ridgeback.charge.hit,_sideways, ridgeback.charge.hitstun, ridgeback.charge.knockback, ridgeback.charge.launch, ridgeback.charge.lockout_after_use, ridgeback.charge.roots_for, ridgeback.gallop_speed, ridgeback.matches_a_fleeing_target's_speed_(x), ridgeback.poise_regained_per_frame, ridgeback.preferred_distance, ridgeback.rear_and_slam.appetite, ridgeback.rear_and_slam.bearing_tolerance, ridgeback.rear_and_slam.blockstun, ridgeback.rear_and_slam.hit,_forward, ridgeback.rear_and_slam.hit,_highest, ridgeback.rear_and_slam.hit,_lowest, ridgeback.rear_and_slam.hit,_sideways, ridgeback.rear_and_slam.hit_radius, ridgeback.rear_and_slam.hit_travels_at, ridgeback.rear_and_slam.hitstun, ridgeback.rear_and_slam.ideal_range, ridgeback.rear_and_slam.knockback, ridgeback.rear_and_slam.launch, ridgeback.rear_and_slam.roots_for, ridgeback.rear_and_slam.wants_the_target_at_(cos), ridgeback.shake.bearing_tolerance, ridgeback.shake.blockstun, ridgeback.shake.hit,_forward, ridgeback.shake.hit,_highest, ridgeback.shake.hit,_lowest, ridgeback.shake.hit,_sideways, ridgeback.shake.hit_radius, ridgeback.shake.hit_travels_at, ridgeback.shake.hitstun, ridgeback.shake.ideal_range, ridgeback.shake.knockback, ridgeback.shake.launch, ridgeback.shake.range_tolerance, ridgeback.shake.roots_for, ridgeback.shake.wants_the_target_at_(cos), ridgeback.spike_spray.appetite, ridgeback.spike_spray.appetite_per_rider, ridgeback.spike_spray.bearing_tolerance, ridgeback.spike_spray.blockstun, ridgeback.spike_spray.hit,_forward, ridgeback.spike_spray.hit,_highest, ridgeback.spike_spray.hit,_lowest, ridgeback.spike_spray.hit,_sideways, ridgeback.spike_spray.hit_travels_at, ridgeback.spike_spray.hitstun, ridgeback.spike_spray.ideal_range, ridgeback.spike_spray.knockback, ridgeback.spike_spray.launch, ridgeback.spike_spray.lockout_after_use, ridgeback.spike_spray.range_tolerance, ridgeback.spike_spray.roots_for, ridgeback.stomp.appetite_per_rider, ridgeback.stomp.blockstun, ridgeback.stomp.hit,_forward, ridgeback.stomp.hit,_highest, ridgeback.stomp.hit,_lowest, ridgeback.stomp.hit,_sideways, ridgeback.stomp.hit_radius, ridgeback.stomp.hit_travels_at, ridgeback.stomp.knockback, ridgeback.stomp.launch, ridgeback.stomp.lockout_after_use, ridgeback.tail_sweep.advance_while_active, ridgeback.tail_sweep.appetite, ridgeback.tail_sweep.blockstun, ridgeback.tail_sweep.hit,_highest, ridgeback.tail_sweep.hit,_lowest, ridgeback.tail_sweep.hit,_sideways, ridgeback.tail_sweep.hit_radius, ridgeback.tail_sweep.hit_travels_at, ridgeback.tail_sweep.hitstun, ridgeback.tail_sweep.knockback, ridgeback.tail_sweep.launch, ridgeback.tail_sweep.lockout_after_use, ridgeback.tail_sweep.roots_for, ridgeback.walk_acceleration, ridgeback.walk_speed, ridgeback_·_hide.haunch_(x_damage), ridgeback_·_hide.head_(x_damage), ridgeback_·_hide.nape_(x_damage), ridgeback_·_hide.neck_(x_damage), ridgeback_·_hide.tail,_middle_(x_damage), ridgeback_·_hide.tail_(x_damage), ridgeback_·_hide.tail_tip_(x_damage), ridgeback_·_legs.broken_leg,_knee_fold, ridgeback_·_legs.corner_drop_per_break, ridgeback_·_legs.roll_per_break, ridgeback_·_legs.speed_per_break_(x), ridgeback_·_mind.aggression_when_wounded, ridgeback_·_mind.appetite_for_a_closing_target, ridgeback_·_mind.appetite_for_a_stunned_target, ridgeback_·_mind.closing_speed_that_provokes_it, ridgeback_·_mind.repeat_penalty, ridgeback_·_mind.repeat_penalty_decay, ridgeback_·_mind.switch_targets_when_nearer_than_(x), ridgeback_·_nerve.root,_per_grab_frame_(x), ridgeback_·_nerve.slow_it_actually_feels_(x), ridgeback_·_nerve.strain_bled_per_frame_(%), ridgeback_·_nerve.strain_to_feel_control, ridgeback_·_nerve.strain_to_interrupt, ridgeback_·_nerve.thresholds_fall_by_(%)

Cliffs:

- ridgeback.turn_rate_cap_(turns/s): between 655 and 15597 (tuned 22282, range 655 to 131072)
- ridgeback.poise: between 50 and 875 (tuned 1500, range 50 to 5000)
- ridgeback.damage_that_flinches_it: between 1 and 84 (tuned 120, range 1 to 2000)
- ridgeback.bite.appetite_per_rider: between 2000 and 2666 (tuned 0, range 0 to 4000)
- ridgeback.tail_sweep.appetite_per_rider: between 2666 and 3333 (tuned 340, range 0 to 4000)
- ridgeback.back_kick.appetite_per_rider: between 2666 and 3333 (tuned 0, range 0 to 4000)

