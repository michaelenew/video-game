# 0033 — Bodies that can move

*2026-10-26.* The owner, on 0032's hand-edited crossbreeds: *"They don't look
like plausible creatures. The poses are insane, the body plans are jammed
together, and most of them would barely be able to move. Plausible movement
is key, then plausible attack patterns."* And: *"fix it in the sim breeder."*

## What was wrong

0032 judged each stage from one still, and reshaped the skin. Walked
(`BRED_GAIT=<line>:<stage>` or `cast:<Name>`, `art/blender.sh art/gait.py`:
the body's boxes through eight points of its slowest gait, side on over a
top view), the breeder's skeletons could not move -- not the crossbreeds
only, the cast's own genomes too:

- **Legs through each other.** The walk's stride was a leg's reach, up to
  nine tenths of a leg each cycle, on bodies whose fore and hind feet stand
  a leg apart or less: every step swept a fore foot back through the hind
  one.
- **Knees out sideways.** A two-segment leg bent to nine tenths puts its knee
  a fifth of the leg off the line; the fore elbow bent back and the hind
  knee forward, so they met under the belly, and a rule written to step
  colliding legs round each other (`out_of_the_way`) turned the hind knees
  two metres out from the body. Folded wings and held scythes counted as
  obstacles too, and turned every leg under them out.
- **Pairs on one spot.** The balance's last resort clamped each pair to the
  torso's end one at a time, so several pairs could land together.
- **Scythes and wings on the floor**: a folded arm resting its point on the
  ground, a wing held just short of planting with its wrist down there.
- The simulation's own account called all of this balanced: 0.7 to 0.9
  plausible, with a walk and a run.

![The cast's walks before: legs bunched, crossing, stepped out](../gallery/0033_cast_walks_before.jpg)

## The checks

`crates/sim/tests/strides.rs`, `every_body_can_move`: the cast's genomes and
every stage of 0031's four lines, walked through sixteen points of the
slowest gait, measured off the built bones:

- on each side, a pair's foot stays ahead of the next pair's, and no leg
  segment passes within its thickness of another pair's;
- no knee or foot more than a fifth of the leg out to the side of its hip
  (a sprawling build, and a wing walked on, are allowed theirs);
- only feet, shins and the tail on the floor -- and a belly where the body
  rests on it, and a rear that drags;
- the mass over the feet that are down;
- every walking pair travels at least a twelfth of its leg a cycle.

41 of 59 failed when it was written. All pass now.

## What changed in the breeder

- **A stride no longer than the room between feet** (`locomotion::at`, and
  the crawl's): each foot keeps within half the clear ground to the next
  pair's, the same cadence carrying the same speed.
- **Every joint bends back** -- an elbow, and the hock a dog's or bird's
  hind leg shows -- **unless** a pair stands closer than two fifths of its
  leg to the pair behind it, an insect's legs off one short thorax, when it
  leads with its knee (`build::knee_ahead`).
- **Feet apart** (`posture`, `Solver::spread`): neighbouring pairs whose
  feet stand closer than a foot and a step each fan out, the front forward
  and the hind back, as far as the legs reach. Hips spaced at least their
  legs' thickness apart from the start.
- **The pairs move as one** in the balance's last resort.
- **Pitched to its legs**: a level body whose front pair stands taller than
  its hind tips to suit them, within its pitch's limit, instead of folding
  the long pair deep.
- **Out of the way** only of other standing legs, at the stride the gait
  now takes; folded wings and held arms are above the legs, not among them.
- **Clear of the floor**: a held scythe swings forward until its point is a
  tenth of the body's height up; a folded wing's band lies above its bone; a
  leaning wing stays short of the floor until it is planted.

Two moves were found not to reach somebody standing close: the **charge**,
whose head came down only once a big body was past them (it winds up head
down now), and the **gore**, whose toss went over a tall crossbreed's
target (it is aimed at standing height now).

![The cast's walks after](../gallery/0033_cast_walks.jpg)

## Carried like an animal

`every_body_is_carried_like_an_animal` asks the standing body two more
things: a level body tilted no more than a hyena's back on four legs (20
degrees) or a standing bird's on two (45), and a neck no steeper than a
horse's (55 degrees; 65 where the genome says the head is raised; an
upright body's head is on top of it, as a mantis's is). Necks went up to
vertical because the head was raised until it cleared the top of the back,
and a body as deep as it is wide put the top of the back high: a head now
rises a little above the spine and no further than a horse's
(`Solver::head`). Bodies stood at sixty degrees because a pitch slider bred
in from an upright body was taken whole by a level one: a level body's pitch
is held to its limit (`level_pitch_most`). `tests/posture.rs`'s "head above
the back" measures the back from the body's real depth now, not from a
depth half its width.

![After: crossbreeds](../gallery/0033_lines_walks.jpg)

## Still not plausible

The walk strips show what the checks do not yet ask:

- folded wings are drawn as slabs as deep as the body, and a wing walked on
  (Ridgeback to Galewing, stages 2 and 3) shows its folded panels as a big N
  beside the legs;
- the Mantis-like bodies stand on legs short for them, with scythes longer
  than the legs;
- the Ridgeback's torso is short for legs as long as its.

The hand edits of 0032 are retired (`game::touch::TOUCHES` is empty): they
reshaped boxes on bodies that could not walk.
