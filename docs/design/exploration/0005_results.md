---
status: exploration — raw results
run: 2026-10-03
---

# 0005 — Topology lab, raw results

The tables [0005_body_plans.md](0005_body_plans.md) §7 reads from, as `crates/lab`'s `body` binary printed them; E7 and E8 were run by hand in a scratch checkout and are written up here. Rerun with:

```text
CARGO_TARGET_DIR=target/lab cargo run --release \
    --manifest-path crates/lab/Cargo.toml --bin body > target/lab/body.md
```

Sole heights are metres above the floor (negative is in it). Skate is how far a planted foot slides as a share of how far the body moves (0 planted, 1 dragged). "Body blows off the body" lists every move whose hit volume, on its first active frame, is more than 0.5 m from every part of the body, with the gap in metres; projectiles, lobs and moves with their own hit test are left out. Class hops: Bulwark 2.7 m, Champion 4.0 m, Shadow Reaver 5.1 m, Elementalist 5.0 m, Blood mage 4.0 m, Dual mage 6.0 m.


## E0 · Every body as built

| Body | Idle sole, low / high (m) | Gait sole | Lowest sole in a move | Belly | Skate | Back (m) | Classes up from the floor | Body blows off the body (m) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ridgeback | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| Mireback | -0.24 / -0.07 | -0.56 | -3.23 (Wallow) | -0.24 | 0.70 | 5.1 | 1 | Backwash 0.5 |
| Sandmaw | +0.00 / +0.00 | +0.00 | +0.00 (no legs) | +0.00 | 0.00 | -- | 0 | Breach-dive 1.4 |
| Pair | -0.02 / +0.01 | -0.10 | -0.19 (Ambush) | -0.02 | 0.66 | -- | 0 | none |
| Broodmother | -0.06 / -0.06 | -0.07 | -0.26 (Fang lunge) | -0.06 | 0.31 | -- | 0 | none |
| Veilstalker | -0.05 / -0.01 | -0.16 | -0.45 (Pounce) | -0.05 | 0.47 | -- | 0 | none |
| Mantis | -0.04 / -0.04 | -0.08 | -0.21 (Leap) | -0.04 | 0.43 | -- | 0 | none |
| Galewing | -0.07 / -0.01 | -0.16 | -0.73 (Stoop) | -0.07 | 0.44 | 3.4 | 5 | none |
| Siegeshell | +0.51 / +0.51 | +0.49 | +0.36 (Plough) | +0.00 | 0.00 | 13.9 | 0 | none |

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| Ridgeback | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| Mireback | 23 / 36 | Bu4 Ch6 Sh6 El6 Bl1 Du0 | 0 | 3 | 173 | reactable mix, never idle, threat bands, back reachable, not safe, neither free nor hopeless |
| Sandmaw | 16 / 36 | Bu6 Ch6 Sh3 El0 Bl0 Du1 | 0 | 1 | 220 | reactable mix, threat bands, neither free nor hopeless |
| Pair | 20 / 36 | Bu6 Ch3 Sh3 El6 Bl0 Du2 | 0 | 0 | 121 | no single move, openings punishable, threat bands, back reachable, not safe, ride long enough |
| Broodmother | 7 / 36 | Bu0 Ch3 Sh1 El0 Bl0 Du3 | 0 | 0 | 100 | whole move set, back reachable, not safe, ride long enough, poise breaks |
| Veilstalker | 9 / 36 | Bu3 Ch4 Sh1 El1 Bl0 Du0 | 0 | 1 | 250 | openings punishable, never idle, threat bands, back reachable, not safe, ride long enough |
| Mantis | 7 / 36 | Bu4 Ch1 Sh0 El2 Bl0 Du0 | 0 | 0 | 154 | threat bands, back reachable, not safe, ride long enough, poise breaks |
| Galewing | 6 / 36 | Bu0 Ch2 Sh0 El4 Bl0 Du0 | 0 | 14 | 422 | reactable mix, never idle, back reachable, not safe, ride long enough |
| Siegeshell | 3 / 36 | Bu0 Ch1 Sh2 El0 Bl0 Du0 | 0 | 3 | 319 | reactable mix, whole move set, no single move, never idle, threat bands, back reachable, not safe |

## E1 · Proportions (the Ridgeback)

A segment stretched by `k`: its children sit `k` times further out and its boxes stretch with it. *Naive* leaves the hips where they were; *kept* raises or lowers them so the standing sole is where it was.

| Body | Idle sole, low / high (m) | Gait sole | Lowest sole in a move | Belly | Skate | Back (m) | Classes up from the floor | Body blows off the body (m) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| as built | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| legs x0.7 naive | +1.25 / +1.35 | +1.06 | +0.53 (Rear and slam) | +1.25 | 0.64 | 5.3 | 1 | none |
| legs x0.7 kept | -0.07 / +0.02 | -0.26 | -0.80 (Rear and slam) | -0.07 | 0.64 | 4.0 | 5 | none |
| legs x0.85 naive | +0.59 / +0.68 | +0.40 | +0.03 (Rear and slam) | +0.59 | 0.55 | 5.3 | 1 | none |
| legs x0.85 kept | -0.07 / +0.02 | -0.27 | -0.64 (Rear and slam) | -0.07 | 0.55 | 4.6 | 3 | none |
| legs x1.15 naive | -0.75 / -0.64 | -0.93 | -1.12 (Rear and slam) | -0.75 | 0.38 | 5.3 | 1 | none |
| legs x1.15 kept | -0.08 / +0.03 | -0.26 | -0.45 (Rear and slam) | -0.08 | 0.38 | 6.0 | 1 | none |
| legs x1.3 naive | -1.42 / -1.30 | -1.60 | -1.72 (Rear and slam) | -1.42 | 0.29 | 5.3 | 1 | none |
| legs x1.3 kept | -0.08 / +0.04 | -0.25 | -0.37 (Rear and slam) | -0.08 | 0.29 | 6.6 | 0 | none |
| legs x1.6 naive | -2.77 / -2.63 | -2.93 | -2.97 (Rear and slam) | -2.77 | 0.12 | 5.3 | 1 | none |
| legs x1.6 kept | -0.08 / +0.07 | -0.24 | -0.28 (Rear and slam) | -0.08 | 0.12 | 8.0 | 0 | none |
| neck x0.6 | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| neck x1.5 | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| head x1.5 | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| tail x0.5 | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| tail x1.6 | -0.08 / +0.02 | -0.27 | -0.53 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| trunk x0.8 (length only) | -0.08 / +0.02 | -0.27 | -0.44 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| trunk x1.3 (length only) | -0.08 / +0.03 | -0.26 | -0.68 (Rear and slam) | -0.08 | 0.47 | 5.3 | 1 | none |
| Size knob x0.6 | -0.05 / +0.01 | -0.16 | -0.32 (Rear and slam) | -0.05 | 0.68 | 3.2 | 5 | none |
| Size knob x1.5 | -0.11 / +0.03 | -0.40 | -0.80 (Rear and slam) | -0.11 | 0.20 | 7.9 | 0 | none |

**Drift** -- how far the bone a move rides moves away from where its blow lands, on the frame it comes out, flat metres; the worst move:

| Body | Drift (m) | Move |
| --- | --- | --- |
| legs x0.7 naive | 0.00 | Bite |
| legs x0.7 kept | 0.00 | Bite |
| legs x0.85 naive | 0.00 | Bite |
| legs x0.85 kept | 0.00 | Bite |
| legs x1.15 naive | 0.00 | Bite |
| legs x1.15 kept | 0.00 | Bite |
| legs x1.3 naive | 0.00 | Bite |
| legs x1.3 kept | 0.00 | Bite |
| legs x1.6 naive | 0.00 | Bite |
| legs x1.6 kept | 0.00 | Bite |
| neck x0.6 | 0.84 | Bite |
| neck x1.5 | 1.05 | Bite |
| head x1.5 | 0.00 | Bite |
| tail x0.5 | 1.27 | Tail sweep |
| tail x1.6 | 1.53 | Tail sweep |
| trunk x0.8 (length only) | 0.76 | Bite |
| trunk x1.3 (length only) | 1.14 | Bite |
| Size knob x0.6 | 2.16 | Tail sweep |
| Size knob x1.5 | 2.70 | Tail sweep |

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| as built | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| legs x0.7 kept | 5 / 36 | Bu1 Ch1 Sh2 El1 Bl0 Du0 | 0 | 2 | 80 | whole move set |
| legs x1.3 kept | 2 / 36 | Bu0 Ch1 Sh1 El0 Bl0 Du0 | 0 | 0 | 48 | none |
| legs x1.6 kept | 1 / 36 | Bu0 Ch0 Sh1 El0 Bl0 Du0 | 0 | 0 | 42 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |
| neck x1.5 | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 0 | 54 | none |
| head x1.5 | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 0 | 52 | none |
| tail x1.6 | 7 / 36 | Bu0 Ch3 Sh4 El0 Bl0 Du0 | 0 | 0 | 50 | none |
| trunk x1.3 (length only) | 1 / 36 | Bu0 Ch1 Sh0 El0 Bl0 Du0 | 0 | 0 | 62 | none |
| Size knob x0.6 | 13 / 36 | Bu5 Ch2 Sh6 El0 Bl0 Du0 | 0 | 1 | 77 | none |
| Size knob x1.5 | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 0 | 31 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |

## E2 · Part flags (the Ridgeback)

Parts as built: head (); neck (); nape (weak,soft); shoulders (mount); ridge (weak,soft); barrel (mount); haunch (mount); tail (mount); tail, middle (mount); tail tip (); left foreleg (); right foreleg (); left hindleg (); right hindleg (); left forefoot (breaks); right forefoot (breaks); left hindfoot (breaks); right hindfoot (breaks).

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| as built | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| no weak points | 9 / 36 | Bu2 Ch3 Sh4 El0 Bl0 Du0 | 0 | 1 | 66 | back reachable, not safe, poise breaks |
| head weak, back not | 9 / 36 | Bu2 Ch3 Sh4 El0 Bl0 Du0 | 0 | 1 | 66 | back reachable, not safe, poise breaks |
| nothing mountable | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 0 | 38 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |
| everything mountable | 7 / 36 | Bu1 Ch3 Sh3 El0 Bl0 Du0 | 0 | 2 | 83 | none |
| feet do not break | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 0 | 38 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |
| everything breaks (first twelve) | 1 / 36 | Bu0 Ch0 Sh1 El0 Bl0 Du0 | 0 | 0 | 38 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |

## E3 · A cousin's move, same skeleton

The Pair and the Veilstalker are built on the Ridgeback's eighteen bones in its order, so their baked rows play on it unchanged. Each row is one donor move added to the Ridgeback as a ninth move, its distances scaled by the ratio of hip heights. *Off the body* is the gap between the move's volume and the Ridgeback's body on its first active frame.

| Move | Kind | Rides | Off the body (m) | Lowest sole (m) | Thrown / landed | Hunts won | Unanswerable | Contract broken | Donor appetite, ideal range |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| (as built) | | | | | | 6 / 36 | 0 | none |
| Pair · Pounce | lobbed | root rides root | 0.00 | -0.44 | 0 / 0 | 6 / 36 | 0 | whole move set | 1000, 7.0 m |
| Pair · Rake | body | root rides root | 0.00 | -0.09 | 98 / 71 | 3 / 36 | 0 | whole move set | 1000, 2.5 m |
| Pair · Swat | body | root rides root | 0.00 | -0.18 | 52 / 38 | 2 / 36 | 0 | none | 900, 1.8 m |
| Pair · Ambush | travels | root rides root | 0.00 | -0.38 | 0 / 0 | 6 / 36 | 0 | whole move set | 900, 7.0 m |
| Pair · Tail trip | body | tail4 rides tail4 | 0.00 | -0.21 | 69 / 0 | 4 / 36 | 0 | whole move set | 900, 2.5 m |
| Pair · Perch | harmless | root rides root | -- | -0.35 | 0 / 0 | 6 / 36 | 0 | whole move set | 0, 9.0 m |
| Pair · Dive | lobbed | root rides root | 0.00 | -0.26 | 0 / 0 | 6 / 36 | 0 | whole move set | 0, 9.0 m |
| Pair · Interpose | body | root rides root | 0.00 | -0.46 | 0 / 0 | 6 / 36 | 0 | whole move set | 0, 6.0 m |
| Veilstalker · Ambush lunge | body | root rides root | 0.00 | -0.11 | 31 / 2 | 8 / 36 | 0 | whole move set | 1000, 7.5 m |
| Veilstalker · Tail spear | travels | root rides root | 2.45 | -0.17 | 105 / 29 | 7 / 36 | 0 | none | 1000, 4.5 m |
| Veilstalker · Rake | body | root rides root | 0.00 | -0.42 | 138 / 108 | 1 / 36 | 0 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless | 1000, 1.5 m |
| Veilstalker · Pounce | lobbed | root rides root | 0.00 | -0.64 | 17 / 0 | 8 / 36 | 0 | whole move set | 1000, 9.0 m |
| Veilstalker · Quill fling | own | root rides root | -- | -0.11 | 0 / 0 | 6 / 36 | 0 | whole move set | 700, 17.0 m |
| Veilstalker · Smoke | harmless | root rides root | -- | -0.14 | 76 / 0 | 3 / 36 | 0 | threat bands | 300, 6.0 m |
| Veilstalker · Mimic | harmless | root rides root | -- | -0.19 | 116 / 0 | 5 / 36 | 0 | whole move set, threat bands | 400, 10.0 m |
| Veilstalker · Climb | harmless | root rides root | -- | -0.21 | 0 / 0 | 6 / 36 | 0 | whole move set | 0, 0.0 m |
| Ridgeback · Bite on the Pair | body | head rides head | 0.00 | | 1 / 0 | 17 / 36 | 0 | openings punishable, threat bands, back reachable, not safe, ride long enough |
| Ridgeback · Tail sweep on the Pair | body | tail3 rides tail3 | 0.00 | | 1 / 0 | 16 / 36 | 0 | openings punishable, threat bands, back reachable, not safe, ride long enough |
| Ridgeback · Back kick on the Pair | body | root rides root | 0.00 | | 1 / 0 | 17 / 36 | 0 | openings punishable, threat bands, back reachable, not safe, ride long enough |

## E4 · A move across skeletons, by role

The Ridgeback's moves read onto other skeletons by the role map in `crates/lab/src/roles.rs`: each recipient bone takes the angles of the donor bone that plays its role, and holds its own standing angles where none does.

| Recipient | Recipient bones driven | Donor bones with nowhere to go | Move | Rides | Off the body (m) | Lowest sole in the move (m) | Its own worst (m) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Mireback | 11 of 13 | 7 of 18 | Bite | head rides head | 0.00 | -0.33 | -3.23 |
| Mireback | 11 of 13 | 7 of 18 | Stomp | root rides root | 0.00 | -0.58 | -3.23 |
| Mireback | 11 of 13 | 7 of 18 | Tail sweep | tail3 has no counterpart; rides the body | 0.00 | -1.10 | -3.23 |
| Mireback | 11 of 13 | 7 of 18 | Rear and slam | root rides root | 0.00 | -2.09 | -3.23 |
| Galewing | 9 of 15 | 9 of 18 | Bite | head rides head | 0.00 | -0.04 | -0.73 |
| Galewing | 9 of 15 | 9 of 18 | Stomp | root rides root | 0.33 | -0.07 | -0.73 |
| Galewing | 9 of 15 | 9 of 18 | Tail sweep | tail3 rides tail | 0.00 | -0.14 | -0.73 |
| Galewing | 9 of 15 | 9 of 18 | Rear and slam | root rides root | 0.00 | -0.29 | -0.73 |
| Mantis | 13 of 19 | 5 of 18 | Bite | head rides head | 0.00 | -0.10 | -0.21 |
| Mantis | 13 of 19 | 5 of 18 | Stomp | root rides root | 0.00 | -0.12 | -0.21 |
| Mantis | 13 of 19 | 5 of 18 | Tail sweep | tail3 has no counterpart; rides the body | 0.00 | -0.15 | -0.21 |
| Mantis | 13 of 19 | 5 of 18 | Rear and slam | root rides root | 0.00 | -0.27 | -0.21 |
| Broodmother | 28 of 29 | 6 of 18 | Bite | head rides head | 0.00 | +0.12 | -0.26 |
| Broodmother | 28 of 29 | 6 of 18 | Stomp | root rides root | 0.00 | +0.25 | -0.26 |
| Broodmother | 28 of 29 | 6 of 18 | Tail sweep | tail3 has no counterpart; rides the body | 0.16 | -0.18 | -0.26 |
| Broodmother | 28 of 29 | 6 of 18 | Rear and slam | root rides root | 0.00 | -3.02 | -0.26 |
| Siegeshell | 28 of 30 | 6 of 18 | Bite | head rides head | 0.81 | +0.44 | +0.36 |
| Siegeshell | 28 of 30 | 6 of 18 | Stomp | root rides root | 3.99 | +0.38 | +0.36 |
| Siegeshell | 28 of 30 | 6 of 18 | Tail sweep | tail3 has no counterpart; rides the body | 2.38 | +0.36 | +0.36 |
| Siegeshell | 28 of 30 | 6 of 18 | Rear and slam | root rides root | 1.53 | +0.08 | +0.36 |
| Sandmaw | 12 of 14 | 10 of 18 | Bite | head rides head | 0.03 | -4.99 | +0.00 |
| Sandmaw | 12 of 14 | 10 of 18 | Stomp | root rides root | 0.00 | -3.33 | +0.00 |
| Sandmaw | 12 of 14 | 10 of 18 | Tail sweep | tail3 rides tail4 | 0.55 | -2.15 | +0.00 |
| Sandmaw | 12 of 14 | 10 of 18 | Rear and slam | root rides root | 0.00 | -4.82 | +0.00 |

The role map, bone by bone:

- **Mireback**: root <- root, head <- head, jaw <- (own), tongue <- (own), throat <- neck, shoulder.l <- shoulder.l, forearm.l <- forearm.l, shoulder.r <- shoulder.r, forearm.r <- forearm.r, thigh.l <- thigh.l, shin.l <- shin.l, thigh.r <- thigh.r, shin.r <- shin.r
- **Galewing**: root <- root, chest <- spine, neck <- neck, head <- head, tail <- tail1, shoulder.l <- (own), arm.l <- (own), hand.l <- (own), shoulder.r <- (own), arm.r <- (own), hand.r <- (own), thigh.l <- thigh.l, shin.l <- shin.l, thigh.r <- thigh.r, shin.r <- shin.r
- **Mantis**: root <- root, abdomen <- spine, thorax <- chest, neck <- neck, head <- head, arm.l <- (own), blade.l <- (own), arm.r <- (own), blade.r <- (own), wing.l <- (own), wing.r <- (own), femur.ml <- shoulder.l, tibia.ml <- forearm.l, femur.mr <- shoulder.r, tibia.mr <- forearm.r, femur.hl <- thigh.l, tibia.hl <- shin.l, femur.hr <- thigh.r, tibia.hr <- shin.r
- **Broodmother**: root <- root, head <- head, fangs <- (own), pedicel <- spine, abdomen <- chest, coxa.l1 <- shoulder.l, femur.l1 <- shoulder.l, tibia.l1 <- forearm.l, coxa.r1 <- shoulder.r, femur.r1 <- shoulder.r, tibia.r1 <- forearm.r, coxa.l2 <- shoulder.l, femur.l2 <- shoulder.l, tibia.l2 <- forearm.l, coxa.r2 <- shoulder.r, femur.r2 <- shoulder.r, tibia.r2 <- forearm.r, coxa.l3 <- thigh.l, femur.l3 <- thigh.l, tibia.l3 <- shin.l, coxa.r3 <- thigh.r, femur.r3 <- thigh.r, tibia.r3 <- shin.r, coxa.l4 <- thigh.l, femur.l4 <- thigh.l, tibia.l4 <- shin.l, coxa.r4 <- thigh.r, femur.r4 <- thigh.r, tibia.r4 <- shin.r

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| Mireback with the Bite (thrown 28, landed 0) | 11 / 18 | Bu3 Ch3 Sh2 El3 Bl0 Du0 | 0 | 2 | 210 | reactable mix, never idle, back reachable, not safe, neither free nor hopeless |
| Galewing with the Bite (thrown 585, landed 22) | 2 / 18 | Bu0 Ch0 Sh0 El2 Bl0 Du0 | 0 | 0 | 222 | reactable mix, whole move set, back reachable, not safe, ride long enough, neither free nor hopeless |
| Mantis with the Bite (thrown 102, landed 7) | 6 / 18 | Bu2 Ch1 Sh0 El3 Bl0 Du0 | 0 | 0 | 181 | back reachable, not safe, ride long enough, poise breaks |
| Broodmother with the Bite (thrown 138, landed 48) | 1 / 18 | Bu0 Ch0 Sh0 El0 Bl0 Du1 | 0 | 0 | 64 | whole move set, never idle, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |
| Siegeshell with the Bite (thrown 0, landed 0) | 1 / 18 | Bu0 Ch1 Sh0 El0 Bl0 Du0 | 0 | 1 | 325 | reactable mix, whole move set, no single move, never idle, threat bands, back reachable, not safe |
| Sandmaw with the Bite (thrown 0, landed 0) | 5 / 18 | Bu3 Ch2 Sh0 El0 Bl0 Du0 | 0 | 1 | 225 | reactable mix, whole move set, threat bands |

## E5 · Traits (the Ridgeback)

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| as built | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| hears (its own lore, no noise cells) | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| hears, with the Sandmaw's lore layout | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| sees nobody | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| sees nobody, hears, Sandmaw layout | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| sees nobody, hears, Sandmaw layout, **and the senses row** | 10 / 36 | Bu3 Ch3 Sh4 El0 Bl0 Du0 | 0 | 2 | 99 | whole move set |
| hears, Sandmaw layout, and the senses row | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| collides with solids | 5 / 36 | Bu0 Ch3 Sh2 El0 Bl0 Du0 | 0 | 0 | 59 | none |
| lands on bodies | 1 / 36 | Bu0 Ch1 Sh0 El0 Bl0 Du0 | 0 | 0 | 41 | none |
| rolls over | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 0 | 52 | none |
| the Mireback's hazards, declared | 6 / 36 | Bu0 Ch3 Sh3 El0 Bl0 Du0 | 0 | 1 | 67 | none |
| the Mireback's whole fight | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 0 | 33 | whole move set, back reachable, not safe, ride long enough, poise breaks, neither free nor hopeless |
| the Sandmaw's whole fight | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 36 | 600 | whole move set, openings punishable, never idle, threat bands, back reachable, not safe, ride long enough, poise breaks, hunt concludes, neither free nor hopeless |
| the Pair's whole fight | 0 / 36 | Bu0 Ch0 Sh0 El0 Bl0 Du0 | 0 | 36 | 600 | whole move set, openings punishable, never idle, threat bands, back reachable, not safe, ride long enough, poise breaks, hunt concludes, neither free nor hopeless |

## E6 · A hybrid

The Ridgeback's body with legs x1.15 (hips kept), the head as its weak point instead of the back, the Pair's Swat and the Veilstalker's Tail spear, and hearing with the Sandmaw's lore layout.

| Body | Idle sole, low / high (m) | Gait sole | Lowest sole in a move | Belly | Skate | Back (m) | Classes up from the floor | Body blows off the body (m) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hybrid | -0.08 / +0.03 | -0.26 | -0.45 (Rear and slam) | -0.08 | 0.38 | 6.0 | 1 | none |

| Body | Hunts won | By class | Unanswerable | Unresolved | Mean length (s) | Contract clauses broken (Champion) |
| --- | --- | --- | --- | --- | --- | --- |
| hybrid | 6 / 36 | Bu0 Ch2 Sh4 El0 Bl0 Du0 | 0 | 1 | 60 | whole move set, back reachable, not safe, poise breaks |

Landed / thrown, every move: Bite 56/94, Stomp 27/77, Tail sweep 19/142, Charge 11/60, Rear and slam 21/180, Shake 0/161, Back kick 1/8, Spike spray 10/51, Swat 41/68, Tail spear 27/51.

## E7 · The bake

Nine creature tables, baked in a scratch checkout at this branch's base.

| Check | Result |
| --- | --- |
| The same bake twice, release build | identical |
| A debug build's bake against a release build's | identical |
| The bake against the committed tables | six identical but for `rustfmt`'s spacing; the Galewing (1,052 values differ, worst 3,422 raw = 18.8 degrees), the Mantis (449, worst 22 raw = 0.12 degrees) and the Sandmaw (2,533, worst 6,078 raw = 33 degrees) differ |
| The Galewing and Mantis baked at the commit before the Siegeshell's | the same differences: not reproducible at their own commits |

**One bit.** Every `sin`, `cos`, `acos`, `atan2`, `powi` and `powf` in `crates/anim` wrapped so that with `NUDGE` set its result moves one unit in the last place -- what a different platform's maths library may do -- then the bake run with and without it:

| Species | Values that move | Worst |
| --- | --- | --- |
| Broodmother | 291 of 69,979 (0.42%) | 37 raw (0.20 degrees) |
| Mireback | 47 of 44,075 | 1 raw |
| Mantis | 36 of 87,997 | 1 raw |
| Galewing | 11 of 64,337 | 1 raw |
| Veilstalker | 9 of 72,538 | 1 raw |
| Pair | 4 of 89,242 | 1 raw |
| Ridgeback | 2 of 55,738 | 1 raw |
| Sandmaw, Siegeshell | 0 | 0 |

## E8 · Changing the topology in the source

Each mutation edited in a scratch checkout, then: `cargo check` of `sim`, `anim`, `hunt` (and their tests); the species guide's bootstrap (an empty baked table, species.md step 2) and the check again; the creature's tests; `bake_beast`; the tests again; six hunts. The log, as it ran:

```text
=== bm6
compile errors: 1
      1 crates/sim/src/species/broodmother/baked.rs | mismatched types: expected an array with a size of 72, found one with a size of 90
with the table stubbed, compile errors: 1 in 1 files
      1 crates/anim/src/beast/broodmother/mod.rs | mismatched types: expected an array with a size of 6, found one with a size of 8
=== bm4
compile errors: 1
      1 crates/sim/src/species/broodmother/baked.rs | mismatched types: expected an array with a size of 54, found one with a size of 90
with the table stubbed, compile errors: 1 in 1 files
      1 crates/anim/src/beast/broodmother/mod.rs | mismatched types: expected an array with a size of 4, found one with a size of 8
=== bm10
compile errors: 3
      1 crates/sim/src/species/broodmother/mod.rs | index out of bounds: the length is 8 but the index is 8: evaluation of `species::broodmother::PARTS` failed inside this call
      1 crates/sim/src/species/broodmother/mod.rs | index out of bounds: the length is 4 but the index is 4: evaluation of `species::broodmother::BONES` failed inside this call
      1 crates/sim/src/species/broodmother/baked.rs | mismatched types: expected an array with a size of 108, found one with a size of 90
with the table stubbed, compile errors: 2 in 1 files
      1 crates/sim/src/species/broodmother/mod.rs | index out of bounds: the length is 8 but the index is 8: evaluation of `species::broodmother::PARTS` failed inside this call
      1 crates/sim/src/species/broodmother/mod.rs | index out of bounds: the length is 4 but the index is 4: evaluation of `species::broodmother::BONES` failed inside this call
=== ss4
compile errors: 1
      1 crates/sim/src/species/siegeshell/baked.rs | mismatched types: expected an array with a size of 69, found one with a size of 93
with the table stubbed, compile errors: 0 in 0 files
tests before rebake: test result: FAILED. 23 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.98s 
thread 'the_shiver_throws_even_a_braced_rider_off_the_crown_and_not_off_the_plateau' (5845) panicked at crates/sim/tests/siegeshell.rs:555:5:
thread 'the_shrug_is_a_brace_on_its_side_and_the_shiver_more_than_one_on_the_crown' (5846) panicked at crates/sim/tests/siegeshell.rs:601:9:
thread 'the_shrug_throws_a_loose_rider_off_its_side_and_a_braced_one_holds' (5847) panicked at crates/sim/tests/siegeshell.rs:524:9:
test result: FAILED. 23 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.98s
bake exit 0: 
tests after rebake: test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.73s test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s 
thread 'every_registered_species_is_a_well_formed_table' (6477) panicked at crates/sim/tests/species.rs:99:13:
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
fight:   run 0: Died in 307s, 14 rides, 0 topples, 0 unanswerable, 0 health left   run 1: Died in 625s, 29 rides, 1 topples, 0 unanswerable, 0 health left   run 2: Died in 484s, 29 rides, 1 topples, 0 unanswerable, 0 health left   run 3: Died in 278s, 21 rides, 1 topples, 0 unanswerable, 0 health left   run 4: Died in 342s, 25 rides, 1 topples, 0 unanswerable, 0 health left   run 5: Died in 543s, 30 rides, 1 topples, 0 unanswerable, 0 health left   0/6 hunts won   health left: 0 a hunt, 0 a win;  unanswerable: 0 
=== ss8
compile errors: 3
      1 crates/sim/src/species/siegeshell/mod.rs | index out of bounds: the length is 6 but the index is 6: evaluation of `species::siegeshell::PARTS` failed inside this call
      1 crates/sim/src/species/siegeshell/mod.rs | index out of bounds: the length is 3 but the index is 3: evaluation of `species::siegeshell::BONES` failed inside this call
      1 crates/sim/src/species/siegeshell/baked.rs | mismatched types: expected an array with a size of 117, found one with a size of 93
with the table stubbed, compile errors: 2 in 1 files
      1 crates/sim/src/species/siegeshell/mod.rs | index out of bounds: the length is 6 but the index is 6: evaluation of `species::siegeshell::PARTS` failed inside this call
      1 crates/sim/src/species/siegeshell/mod.rs | index out of bounds: the length is 3 but the index is 3: evaluation of `species::siegeshell::BONES` failed inside this call
=== horn
compile errors: 1
      1 crates/sim/src/species/ridgeback/baked.rs | mismatched types: expected an array with a size of 60, found one with a size of 57
with the table stubbed, compile errors: 0 in 0 files
tests before rebake: test result: FAILED. 39 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s 
thread 'the_hips_and_the_tail_root_are_the_calm_places_to_stand' (7586) panicked at crates/sim/tests/monster.rs:1163:5:
thread 'the_shake_throws_a_loose_rider_off_the_back_and_a_braced_one_holds' (7592) panicked at crates/sim/tests/monster.rs:1135:9:
thread 'the_slam_beats_even_a_brace' (7593) panicked at crates/sim/tests/monster.rs:310:5:
thread 'the_sweep_goes_to_the_side_you_are_on' (7594) panicked at crates/sim/tests/monster.rs:786:5:
thread 'you_can_jump_the_shake_if_you_commit_before_the_whip' (7597) panicked at crates/sim/tests/monster.rs:1275:5:
test result: FAILED. 39 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
bake exit 0: 
tests after rebake: test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s 
fight:   run 0: Killed(3944) in 66s, 15 rides, 3 topples, 0 unanswerable, 400 health left   run 1: Died in 64s, 12 rides, 2 topples, 0 unanswerable, 0 health left   run 2: Killed(3946) in 66s, 13 rides, 3 topples, 0 unanswerable, 330 health left   run 3: Died in 39s, 0 rides, 0 topples, 0 unanswerable, 0 health left   run 4: Killed(3607) in 60s, 16 rides, 3 topples, 0 unanswerable, 560 health left   run 5: Killed(4172) in 70s, 16 rides, 3 topples, 0 unanswerable, 545 health left   4/6 hunts won, mean 65s   health left: 306 a hunt, 459 a win;  unanswerable: 0 
=== notail
compile errors: 5
      2 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL3` in this scope: not found in this scope
      1 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL2` in this scope: not found in this scope
      1 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL1` in this scope: not found in this scope
      1 crates/sim/src/species/ridgeback/baked.rs | mismatched types: expected an array with a size of 45, found one with a size of 57
with the table stubbed, compile errors: 4 in 1 files
      2 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL3` in this scope: not found in this scope
      1 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL2` in this scope: not found in this scope
      1 crates/sim/src/species/ridgeback/mod.rs | cannot find value `TAIL1` in this scope: not found in this scope
ALL-DONE
```

Removing the Ridgeback's tail *and* its three tail parts, renumbering the parts after them and pointing the move that rode `tail3` at the body (`notail2`), with the table stubbed and `--keep-going`: **fourteen errors in two files** -- seven in `crates/anim/src/beast/ridgeback/mod.rs` (the pose helpers name `TAIL1` to `TAIL4`) and seven in `crates/sim/tests/monster.rs` (`TAIL_BASE`, `TAIL_MID`, `TAIL_TIP`). `crates/game/src/species/ridgeback.rs` names two more and was not built here.
