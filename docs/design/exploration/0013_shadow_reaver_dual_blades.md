---
status: decided 2026-10-10 by the owner's brief, designed and built by Claude; unplayed
started: 2026-10-10
supersedes: 0011_shadow_reaver_on_three_clicks.md where they differ; kits/shadow-reaver.md §"What is bound"
depends: 0011_shadow_reaver_on_three_clicks.md, ../kits/shadow-reaver.md, ../shadow-reaver-v2.md
---

# 0013 — The Shadow Reaver, second pass: dashing blades

## The brief

From the owner, after playing the first three-clicks build (0011):

> I like the new three clicks core layout, but the shadow reaver feels awkward right now. I want
> this class to capture a feeling more like MapleStory's Dual Blade where you're dashing around
> with spinning blades, sending the shadow out for high impact, and then closing gaps with strong
> finishers.

Then a cell-by-cell list, and two follow-ups mid-build:

> I also want the grounded rclick to be reworked so that it doesn't bring the shadow back to him
> on recast, but moves the shadow to the new location. It's limiting and annoying to have to keep
> bringing the shadow back. The grounded shadow cast also needs to be reworked so that it doesn't
> send the shadow to ground the reaver can't walk to, because right now it trivializes most
> movement sections.

Three choices were the owner's, asked and answered before the build (the brief named right click
twice): **finishers are on left click**, the shadow stays on right; the grounded "shadow blade
ring" is **a ring round her own body**, the lotus stays on `Q`; **Executioner goes to `E`**, and
Deadly mistake is retired.

## The grid

| | Left: finishers | Middle: neutral | Right: the shadow |
| --- | --- | --- | --- |
| **On foot** | **Reave** | **Blade ring** | **Send** (moves it) |
| **In the air** | **Plunge** | **Blade spin** | **Whirl** |
| **`space` + click** | **Gallows** | **Tumble** (and the double jump) | **Shadow arc** |

| Key | Move |
| --- | --- |
| `Q` | Guillotine lotus, as built |
| `E` | Executioner, as built |

Slot numbers kept the four she has always had (Reave in the poke's slot, Executioner,
the lotus, the send) and reworked the seven after them in place, so the Oven's storage did not
move. Retired: Slash, Kite cut, Guillotine drop, Swap, Moonsault, Hang the shadow, Deadly
mistake. Their knobs went with them.

## Left — finishers

Three moves that close the gap and hit hard.

- **Reave** (left, on foot). A short dash (`steps forward` 2.5 m) and a cut whose disc sits at
  the end of the dash. **The middle of that disc is a sweet spot**: a body within
  `Reave, sweet spot radius` of it takes `Reave, sweet spot damage` (×1.5) — 105 against 70.
  Spacing is the skill: thrown from about four metres, it lands sweet; from on top of somebody
  the dash runs into them and the cut is plain. The dash **stops where it strikes** — a step at
  a dash's speed otherwise carried on past the sweet spot, and off a ledge launched her at
  thirty metres a second (found in testing).
- **Plunge** (left, in the air). A short dive, a little forward (`Plunge, forward`), blade first.
  Driven rather than fallen, like Gallows and Landfall, and its hit comes on the frame her feet
  arrive: 130. **A ring of shadow runs outward from the landing** past the blade's own reach,
  `Plunge ring, reach` 3.5 m, for `Plunge ring, damage` 20 and a small shove — "a feeling of
  power", not a second hit. The ring reaches fighters, the creature and critters, each once.
- **Gallows** (space + left). As built — a blink up `Gallows, blinks up` 3 m, a beat, and down in
  an arc blade first — moved from middle to left and made heavier: 170 damage, **32 frames of
  recovery**, −19 on block. Its hitstun went up with it so a landed one is still +1.

## Middle — neutral

The tools for the spaces between commitments. All three are copied by the shadow, which is
where the "spinning blades everywhere" comes from: a Blade ring thrown with the shadow out is two
rings.

- **Blade ring** (middle, on foot). Blades spun round her waist: a **thin disc**,
  `radius` 2.2 m, `Blade ring and spin, half-thickness` 0.5 m above and below her middle, behind
  her as much as in front. 55 damage. The body turns once through it.
- **Blade spin** (middle, in the air). The same disc, very fast — 2 frames of startup, 4 active —
  and wide, 3 m, for 30. The body spins twice. **Thin is the point**: a ball three metres across
  reached three metres down to somebody standing on the floor under her.
- **Tumble** (space + middle, off the floor; **and space + middle in the air, once a trip off
  the ground**). A flipping jump (`self lift` 13 m/s) that throws three blades from her hand:
  the first along the crosshair's line, each after it turned down by `Tumble, fan spread` —
  forward and down. Each blade cuts a body once, 28 apiece. In the air it is her **double jump**:
  it spends the trip's takeoff (`Player::rise`), so a second one in the same jump is the Blade
  spin instead.

The new shape both spins use is `moves::Shape::Disc`, tested by `state::hitbox` with a height
band, and by the creature and critter tests on the same band.

## Right — the shadow

- **Send** (right, on foot). Two changes.
  - **Pressed again, it moves the shadow** from wherever it is to where she points now
    (`shadow::order`, `Order::Moved`). It no longer calls the shadow home: home is the dash,
    which collects it, or the leash. A lotus open on the shadow is dragged after it, wherever it
    goes — the drag is no longer only toward her.
  - **Only onto ground she could walk to** (`aim::walk_toward`). From her feet toward the
    crosshair's spot on the floor, half a body at a time: up nothing steeper than the land lets a
    body climb (`terrain_steepest`) and no box taller than a skin, over no pit, down any drop.
    Stopped short, it comes back a body's width from whatever stopped it. Pointed at the top of a
    dais it stops at the dais's foot; pointed over a gap it stops at the lip. Refused — nothing
    walkable even a step out — the crosshair flashes as before. It replaces the 2026-10-04
    forgiveness (`aim::footing_toward`, `Send shadow, forgiveness back`, both removed), which only
    asked whether the spot was over a pit, so a send pointed up a ledge put the shadow on top.
- **Whirl** (right, in the air). The shadow goes a short way (`reach` 4 m, a skillshot) — **the
  crosshair's point is the whirl's middle**, the shadow's feet half a body under it and never
  under the floor — and spins: a ball of blades round its middle that cuts every body in it every
  `Whirl, cuts every` frames (18 a cut) for `Whirl, lasts` — and then the shadow **comes home on
  its own**, cutting on the way as any return does. The first cut on a body marks it. While it
  is out, the dash to it works as ever, and the dash jump out of it.
- **Shadow arc** (space + right). Thrown **forward and up** — `Shadow arc, forward` 5 m ahead along
  her facing, `Shadow arc, up` 3.5 m above her feet (`aim::hung_ahead`) — on a curve that goes out
  first and up after, so its slash meets a body standing in front of her: 55, once a body,
  marking it. It hangs there for `Shadow arc, hangs for` (a second) and then comes home.

**Following the shadow out.** A forward dodge while a throw's shadow is still flying cancels what
is left of the throw (`state::follow_the_shadow`) and is the dash, chasing it — the crosshair
does not have to be on it, because it is going where she was looking a moment ago.

**The dash jump flips.** Pressed in the dash's last frames as before, it keeps the whole of the
dash's speed and now adds `Dash jump, extra hop` (4 m/s) on top of a jump, in a forward flip
(`Dash jump, flip lasts`, drawn off `Shadow::flip`). This is the "jump again to preserve momentum"
the brief asks to keep.

**Nothing on a clock is left on a ledge.** The Whirl and the arc put the shadow somewhere the
send could not — in the air, over a gap — and both bring it home when their time is up
(`Shadow::hang` is now "frames until it comes home", where it used to be "frames until it sinks
to the floor"). Only the send leaves the shadow standing, and the send only goes where she could
walk.

## What it does to movement — the honest part

The owner's second follow-up asked that the shadow stop trivialising movement sections. The
grounded send cannot reach anywhere she could not walk now. **But the lab search still finds the
Reaver crossing 50–75 m gaps**, through the moves the brief asked to keep: jump, Whirl in the air
(the shadow four metres out, over the gap), the forward dodge that follows it, and the dash jump
that keeps the dash's whole speed — about 50 m/s — plus the new hop.

`cargo run --release -p sim --bin envelope -- search --class "Shadow Reaver"`, shared / whole
kit, metres, against the 2026-10-09 fixtures:

| Rise | −8 | −4 | −2 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 8 | 10 | 12 | 16 | 20 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Now | 29.5/62.7 | 45.5/56.7 | 58.5/60.3 | 55.5/70.3 | 48.3/61.2 | 53.2/63.4 | 66.6/67.4 | 15.9/16.1 | 49.1/62.8 | 16.6/51.1 | 53.8/76.2 | –/– | 57.9/62.6 | –/58.0 | –/– |

Before it, lane −8 was 62.7/86.2. A search is a lower bound, so a lane that fell may be a route this run did not find again. So the send is no longer how she does it; the Whirl is. If the
movement sections should be harder for her, the levers are in REAVER-5 below, and it is the
owner's call which — each one changes something the brief asked for.

## Questions for play

| ID | Question | Built as |
| --- | --- | --- |
| **REAVER-5** | The Whirl → follow-dash → dash jump chain still crosses 50–75 m. Which lever, if any? (a) the dash jump keeps less speed off a shadow in the air; (b) the follow-dash needs the crosshair after all; (c) the Whirl's shadow is not dashable while it spins; (d) leave it — it is execution | Left as the brief describes it; numbers above |
| **REAVER-6** | Is the Reave's sweet spot readable — does a player learn the four-metre spacing? | The middle of the cut's disc, 0.6 m, ×1.5 |
| **REAVER-7** | Is the Plunge's ring worth its frames, or only noise? | 20 damage, 3.5 m, a small shove |
| **REAVER-8** | Should the Tumble's double jump refresh with the dash's arrival, like a landing? | Once a trip off the ground, spent by any takeoff |
| **REAVER-9** | Should the arc and the Whirl drag an open lotus? | Yes, like the send's move |
| **REAVER-2** | (0011) closed: the Kite cut is retired | — |
| **REAVER-3** | (0011) closed: Deadly mistake is retired | — |
| **REAVER-4** | (0011) closed: the Swap is retired | — |

## Presentation

Every new move borrows one of her four authored clips (`view::play`): the two drops the
Executioner's overhead, the two spins the Slash's cut, the Tumble the lotus's arm driven out and
down, the shadow's two the send. **The flips and the spins are a whole-body roll** on top of the
clip (`view::interp::roll_of`, applied about the hips in `game::body_frame`): one forward flip
for the dash jump and the Tumble, one turn for the ring, two for the spin — off the simulation's
clocks, so a rollback redraws them. Authored clips for each are the right next step. The two
spins' blades are drawn — six on the edge of the disc, hers and the shadow's copy — off
`state::hitbox`, the disc they cut with (`game::spin_spot`). The four new
effects (`ShadowWhirl`, `ShadowSlash`, `ShadowFan`, `ShadowRing`) are drawn in the shadow's
near-black, the debug overlay draws the volumes they test, and each has a sound
(`sound::cue`).
