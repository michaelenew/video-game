//! **The Elementalist's hall** (`docs/design/kits/elementalist.md`, on
//! three clicks): earth, fire and wind, the pushes; then the structure
//! jump, exploding a stone, wind through fire, the thermal and the cancel.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::{Class, Mechanic, Structure};
use crate::effects::EffectKind;
use crate::fixed::Fx;
use crate::gust::Gale;
use crate::moves::elementalist::*;
use crate::state::{Action, SLOT_COMMITTED, SLOT_HEAVY, SLOT_POKE, SLOT_SPECIAL};

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::Elementalist,
    arena: ArenaId::HALL_ELEMENTALIST,
    what: "The Elementalist builds the battlefield and then fights with what she built: left \
           click pulls stones out of the ground, middle click starts fires, right click throws \
           wind.",
    plays: "Q and E set off what she made. Q kicks a stone or flicks a fireball off a flame; E \
            smashes a stone into flying shards or turns a flame into a tornado. Her stones are \
            also her ladder.",
    wins: "Her moves are slow to start and easy to see coming, so she wins by setting traps at \
           range and then blowing them up -- not by trading blows up close.",
    rooms: &ROOMS,
};

fn stones(m: Mechanic) -> [Option<Structure>; crate::class::MAX_STRUCTURES] {
    match m {
        Mechanic::Structures(s) => s,
        _ => [None; crate::class::MAX_STRUCTURES],
    }
}

/// A stone of hers came up this frame: a slot holds one newer than it did.
fn new_stone(s: &Seen, aloft: bool) -> bool {
    let (was, now) = (stones(s.was().mechanic), stones(s.now().mechanic));
    now.iter().zip(was.iter()).any(|(n, w)| match (n, w) {
        (Some(n), Some(w)) => n.aloft == aloft && n.age < w.age,
        (Some(n), None) => n.aloft == aloft,
        _ => false,
    })
}

fn shards(w: &crate::World, me: usize) -> usize {
    w.debris
        .iter()
        .flatten()
        .filter(|d| d.owner as usize == me)
        .count()
}

fn burning_gales(w: &crate::World, me: usize) -> usize {
    w.gusts
        .iter()
        .flatten()
        .filter(|g| g.owner as usize == me && g.lit && g.gale == Gale::Disc)
        .count()
}

/// Her feet crossed this height this frame, not carried up by her own
/// wind: a stone threw her.
fn thrown_past(s: &Seen, metres: i32) -> bool {
    let line = Fx::from_int(metres);
    let wind = matches!(
        super::kind_of(s.now().action),
        Some(UPDRAFT | FIRE_FOUNTAIN | TREMOR)
    ) || s.effects(EffectKind::Updraft).1 > 0;
    s.was_high().raw() < line.raw() && s.height().raw() >= line.raw() && !wind
}

fn jumped(s: &Seen) -> bool {
    s.took_off()
}
fn dodged(s: &Seen) -> bool {
    s.dodged()
}
fn raise(s: &Seen) -> bool {
    new_stone(s, false) && !s.channelled(SLOT_COMMITTED)
}
fn fissure(s: &Seen) -> bool {
    s.began(SLOT_COMMITTED)
}
fn earth_jump(s: &Seen) -> bool {
    s.began(EARTH_JUMP)
}
fn landfall(s: &Seen) -> bool {
    s.began_aloft(LANDFALL)
}
fn pillar(s: &Seen) -> bool {
    s.out(SLOT_SPECIAL) && matches!(s.was().action, Action::Startup { .. })
}
fn strike(s: &Seen) -> bool {
    s.out(SLOT_SPECIAL) && !matches!(s.was().action, Action::Startup { .. })
}
fn carpet(s: &Seen) -> bool {
    s.began(FIRE_CARPET)
}
fn fountain(s: &Seen) -> bool {
    s.began(FIRE_FOUNTAIN)
}
/// The ball let go of after a hold of at most / at least this many frames.
fn ball(s: &Seen, grown: bool) -> bool {
    match s.was().action {
        Action::Channel {
            kind: AIR_BALL,
            held,
        } => {
            !matches!(s.now().action, Action::Channel { kind: AIR_BALL, .. })
                && (held >= 30) == grown
        }
        _ => false,
    }
}
fn small_ball(s: &Seen) -> bool {
    ball(s, false)
}
fn grown_ball(s: &Seen) -> bool {
    ball(s, true)
}
fn gale(s: &Seen) -> bool {
    s.began_aloft(GALE)
}
fn updraft(s: &Seen) -> bool {
    s.began_standing(UPDRAFT)
}
fn air_updraft(s: &Seen) -> bool {
    s.began_aloft(UPDRAFT)
}
fn bolt(s: &Seen) -> bool {
    s.began(SLOT_POKE)
}
fn cataclysm(s: &Seen) -> bool {
    s.began(SLOT_HEAVY)
}
/// **A dodge broke through a stone.** The mark the dodge leaves for the
/// break (`Player::breaking`) is set and spent inside one step, so no
/// snapshot shows it: what does is a dodge beginning on the frame a stone of
/// hers is gone and broken ground lies where it stood.
fn broke_through(s: &Seen) -> bool {
    let count = |m| stones(m).iter().flatten().count();
    s.dodged() && count(s.now().mechanic) < count(s.was().mechanic) && s.appeared(EffectKind::Rough)
}
fn structure_jump(s: &Seen) -> bool {
    thrown_past(s, 8)
}
fn double(s: &Seen) -> bool {
    thrown_past(s, 25)
}
fn exploded(s: &Seen) -> bool {
    shards(s.before, s.me) == 0 && shards(s.after, s.me) > 0
}
fn lit_burst(s: &Seen) -> bool {
    exploded(s) && s.appeared(EffectKind::Embers)
}
fn shatter_jump(s: &Seen) -> bool {
    exploded(s) && matches!(super::kind_of(s.now().action), Some(EARTH_JUMP))
}
fn burning_gale(s: &Seen) -> bool {
    burning_gales(s.after, s.me) > burning_gales(s.before, s.me)
}
fn thermal(s: &Seen) -> bool {
    let (was, now) = s.effects(EffectKind::FireCarpet);
    now < was && matches!(super::kind_of(s.now().action), Some(UPDRAFT))
}
fn pushed_out_of_recovery(s: &Seen) -> bool {
    matches!(s.was().action, Action::Recovery { kind, .. } if kind != SLOT_POKE && kind != SLOT_HEAVY)
        && matches!(
            s.now().action,
            Action::Startup {
                kind: SLOT_POKE | SLOT_HEAVY,
                ..
            }
        )
}

pub const ROOMS: [Room; 10] = [
    Room {
        title: "The Elementalist",
        advanced: false,
        intro: "Read the premise, then try your feet. Walk forward into the next room when \
                you are ready: every room opens as you walk into it.",
        tasks: &[
            Task {
                what: "Jump",
                keys: "Space (hold it to go higher)",
                check: jumped,
                lands: false,
            },
            Task {
                what: "Dodge",
                keys: "Shift + a direction",
                check: dodged,
                lands: false,
            },
        ],
        dummy: Dummy::Aside,
        bay: Bay::Flat,
    },
    Room {
        title: "Left click: earth",
        advanced: false,
        intro: "Left click raises a stone where the crosshair meets the floor -- up to three at \
                once, the oldest giving way. Held, the stone churns underground and races \
                toward the crosshair when you let go. In the air it is a dive that drives a slab \
                up where you land.",
        tasks: &[
            Task {
                what: "Raise a stone",
                keys: "Tap left click at the floor",
                check: raise,
                lands: false,
            },
            Task {
                what: "Hold: Fissure",
                keys: "Hold left click, aim, let go",
                check: fissure,
                lands: true,
            },
            Task {
                what: "Takeoff: Earth jump",
                keys: "Space + left click together (hold W to land on the stone)",
                check: earth_jump,
                lands: false,
            },
            Task {
                what: "In the air: Landfall",
                keys: "Space, then left click",
                check: landfall,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: fire",
        advanced: false,
        intro: "Middle click plants a pillar of fire at the crosshair; only one burns at a time. \
                Held, its burning lands all at once as a Strike. In the air it lays a carpet of \
                fire that hangs you up for a moment.",
        tasks: &[
            Task {
                what: "Fire pillar",
                keys: "Tap middle click at the dummy's feet",
                check: pillar,
                lands: true,
            },
            Task {
                what: "Hold: Strike",
                keys: "Hold middle click, aim, let go",
                check: strike,
                lands: true,
            },
            Task {
                what: "In the air: Fire carpet",
                keys: "Space, then middle click",
                check: carpet,
                lands: false,
            },
            Task {
                what: "Takeoff: Fire fountain",
                keys: "Space + middle click together",
                check: fountain,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: wind",
        advanced: false,
        intro: "Right click makes a ball of air at the crosshair that rolls away when you let \
                go; hold it to grow it, then walk in and ride it. In the air it is a Gale, a \
                disc that grows as it flies. With Space it is an Updraft that lifts you.",
        tasks: &[
            Task {
                what: "A small air ball",
                keys: "Tap right click at the floor",
                check: small_ball,
                lands: false,
            },
            Task {
                what: "A grown air ball",
                keys: "Hold right click half a second or more, let go",
                check: grown_ball,
                lands: false,
            },
            Task {
                what: "In the air: Gale",
                keys: "Space, then right click at the dummy",
                check: gale,
                lands: true,
            },
            Task {
                what: "Takeoff: Updraft",
                keys: "Space + right click together",
                check: updraft,
                lands: false,
            },
            Task {
                what: "Updraft in the air",
                keys: "Jump, then hold Space and right click (once a jump)",
                check: air_updraft,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Q, E and the dodge",
        advanced: false,
        intro: "Q and E pass through bodies and act only on what you made. Q kicks a stone \
                along the crosshair, or flicks a fireball off a pillar. E smashes a stone into \
                shards, or turns a pillar into a tornado. Dodge at your own stone, close, and \
                you break straight through it.",
        tasks: &[
            Task {
                what: "Kick a stone into the dummy",
                keys: "Raise a stone between you, crosshair on it, Q",
                check: bolt,
                lands: true,
            },
            Task {
                what: "Cataclysm",
                keys: "Raise a stone between you, crosshair on it, E",
                check: cataclysm,
                lands: true,
            },
            Task {
                what: "Break through a stone",
                keys: "Look down and raise one just in front of you (within 3 m), crosshair on it, Shift + W",
                check: broke_through,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the structure jump",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from her. She plays well \
                without any of it. Raise a stone under your own feet and jump just as it bursts \
                out: its climb is added to your jump. Look straight down, tap left click, and \
                hold Space a fifth of a second later. Too early and it is an ordinary hop off \
                the stone's top. Two stones raised a few frames apart can throw you twice.",
        tasks: &[
            Task {
                what: "Structure jump (8 m and up)",
                keys: "Look down, tap left click, then hold Space as it bursts",
                check: structure_jump,
                lands: false,
            },
            Task {
                what: "The double (25 m and up)",
                keys: "Two taps of left click very fast, then Space",
                check: double,
                lands: false,
            },
        ],
        dummy: Dummy::Aside,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: exploding a stone",
        advanced: true,
        intro: "Smash your own stone with E and it bursts into seven shards in a cone away from \
                you: her biggest damage at range, so put the stone between you and them. Light \
                the stone first with a pillar beside it and the shards burn.",
        tasks: &[
            Task {
                what: "Shards into the dummy",
                keys: "Raise a stone on the line to it, wait a moment, E",
                check: exploded,
                lands: true,
            },
            Task {
                what: "A burning stone",
                keys: "Pillar beside a stone, then Q or E on the stone",
                check: lit_burst,
                lands: false,
            },
            Task {
                what: "Shatter jump",
                keys: "Raise a stone under you, then Space + left click on it",
                check: shatter_jump,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: wind through fire",
        advanced: true,
        intro: "Wind through fire is fire. A Gale that flies through her pillar or her carpet \
                comes out burning and does twice the damage; an air ball or an updraft that \
                touches her fire burns whoever it holds.",
        tasks: &[Task {
            what: "A burning Gale",
            keys: "Pillar halfway to the dummy; jump; right click through it",
            check: burning_gale,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the thermal",
        advanced: true,
        intro: "An Updraft that meets her own fire throws her higher, and off a carpet it throws \
                her along the carpet too: her best way across a gap. Jump, lay a carpet, and \
                Updraft out of it while it holds you.",
        tasks: &[Task {
            what: "Ride a thermal",
            keys: "Space (held); middle click; then Space + right click",
            check: thermal,
            lands: false,
        }],
        dummy: Dummy::Aside,
        bay: Bay::Gap(150, 800),
    },
    Room {
        title: "Advanced: pushing out of a recovery",
        advanced: true,
        intro: "Q or E pressed while anything she made is still finishing fires the push at \
                once. Plant a pillar and flick a fireball off it before the pillar's swing has \
                ended.",
        tasks: &[Task {
            what: "Pillar, then Q straight away",
            keys: "Tap middle click, then Q as soon as the pillar is up",
            check: pushed_out_of_recovery,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
];
