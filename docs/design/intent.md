---
status: intent — the owner's, in the owner's words where possible
recorded: 2026-10-17
---

# Design intent — what the game is for

This is the layer above the rest of the design. The other documents say how a
thing works; this one says what it is *for*, so a decision with no rule to
settle it has something to be checked against. It records what the owner said
on 2026-10-17 and does not decide anything they left open (§4, §5).

## 1 · The game in one sentence

**Taking powerful tools to overcome challenges greater than those tools.**

Three appeals, each borrowed from a different kind of game:

| | Like | What it means here |
| --- | --- | --- |
| **The draw** | A soulslike (Dark Souls, Elden Ring) | Real challenges, overcome, are the reward in themselves. Nothing has to be handed out for winning to feel like winning. |
| **The mastery arc** | A fighting game | Getting better is learning your character: its frames, its options, its matchups. The ceiling is the player, not the character sheet. |
| **The look and feel** | A high-level MMO fight | Spectacle. The character feels *powerful* — big effects, big movement, a kit that reads as strong. |

The tension between the second and third rows is the point: the tools are
strong, and the challenges are stronger still.

## 2 · Monster fights: soulslike

**Fair, but tough. Winning is a legitimate accomplishment**, reached with
dedication, skill, and sometimes cleverness.

What that already means in the design, and should keep meaning:

- *Fair* is the [bestiary](bestiary.md)'s contract — every threat telegraphed,
  every opening real, the harness measuring the reactable share. A loss should
  read as the player's, never the creature's cheat.
- *Tough* is a fight you do not win the first time and can see yourself winning.
- *Cleverness* is a second road to the same win: a use of the arena, a class
  interaction, an approach the creature was not built to expect. A clever win
  is a legitimate one, not an exploit to patch — unless it removes the fight.

## 3 · Mobility: a fighting game's fairness, and danger that feels larger than it is

Mobility challenges — the [jump courses](courses.md), the climbs, getting from
one place to another — are like fighting games: **fair, but always feeling more
dangerous than they actually are.**

**The failure this exists to prevent is a rage game.** A rage game (*Getting
Over It*, *Jump King*) makes a single mistake cost a long stretch of progress.
That is not the intent. Two rules keep it from happening:

1. **Mobility as a true challenge has measured loss.** A miss costs a little,
   and you can see how little before you try. The courses' checkpoints already
   work this way: a fall stands you on the last one, a short way back.
2. **Mobility with a major loss has to feel tougher than it really is.** Where
   a fall really does cost a lot, the margins are wider than they look — the
   gap reads long and is generously short, the ledge looks thin and is not.
   The fear is the experience; the punishment is rare.

**When mobility is a gate** — something that stands between the player and an
accomplishment or the next part of the world — the difficulty should mostly
come from *outside* the movement itself:

- a fight during or after the section;
- a time trial;
- or **short** sections of true, unforgiving peril. Short is the operative word.

## 4 · Progression — open

The progression model is **not decided.** What is decided is what it must
protect: high-level play is **gated by skill, not by grind**. Three shapes are
in play, and the owner's leaning is the third:

| Shape | Why it fits | What limits it |
| --- | --- | --- |
| **Sidegrade unlocks** | Fighting-game variety: more avenues for mastery without changing who wins a matchup (much). | — |
| **Increasingly powerful gear** | A reward for exploring, and help into the next section. | It **must have a ceiling**, and the ceiling must come **early** in the time a player might put in. The game cannot be *about* the limit. |
| **Unlocking new parts of the character** (moves and the like) — **the owner's favourite** | The most thematic. | The input budget is very tight, and the classes mix and match very differently: the Elementalist is so built on interplay that it is unclear how part of her kit could be swapped out. |

**A direction the owner likes for the third:** weapons carry different
abilities *on the clicks* — the same theme, with some variety in kind and
strength. A weapon changes what your attack buttons do, rather than replacing
any of the rest of the kit.

How this meets what is already written in [parked.md](parked.md):

- The gear row is the bounded ramp parked.md credits to Monster Hunter ("a
  bounded ramp, then no roof"). It keeps parked.md's first conclusion only if
  the ceiling is low enough that gear cannot get a player past a wall the
  design means to be a skill check — that is the number the ramp has to be
  designed against.
- The weapon direction is close to parked.md's leading proposal, **offensive
  items modify the auto attack**, widened from the auto to every click. Parked
  said per-weapon ability variants were untenable *across a whole kit*; on the
  clicks alone the count is small, which is what makes it workable.
- Versus is untouched either way (parked.md's second conclusion): whatever is
  earned here is a coop and world thing, and a versus match uses the base kits
  unless a separate ruleset says otherwise.

## 5 · Story and "forward" — open, and wants a storyteller

There is **no sense of "forward" through the game yet.** The draw of the next
challenge is part of an answer, but not all of one. It could be a sandbox — but
a sandbox still needs an *end* the player cares about.

What exists is a premise ([world.md](world.md) §7: a town at the mouth of a
valley full of things, and something walking toward it) and a map
([valley.md](valley.md)). What the story is, and what role it plays, is
undecided.

The owner, in their own words: *"I think I am less of a storyteller than a
builder, so this probably needs engagement by someone who is a storyteller."*
This is an open invitation, not a gap to be filled by default: a story written
to plug the hole would be worse than none.
