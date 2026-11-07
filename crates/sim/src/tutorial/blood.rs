//! **The Blood mage's hall** (`docs/design/kits/blood-mage.md`, the world
//! bleeds): the Red Hand, the spike, the scythe, the keys; then her ground,
//! the bleed into a spike, drinking back, reaping, the swim, and hearts.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::Class;
use crate::effects::EffectKind;
use crate::fixed::Fx;
use crate::moves::blood::*;

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::BloodMage,
    arena: ArenaId::HALL_BLOOD,
    what: "Whatever the Blood mage cuts bleeds -- a fighter, a creature, the floor -- and where \
           blood falls something grows. Her scythe and her red hand are free; what she grows \
           costs her own health, and the plant holds it until she reaps it back.",
    plays: "The scythe sows: its bite and its sweep leave pools. The spike grows: it raises a \
            pool into a thorned plant that launches and lashes -- a pool pays toward it, and on \
            bare ground she pays it all. The scythe reaps: through a plant it throws the thorns \
            and gives back what you paid.",
    wins: "She wins by fighting inside her patch: cast the head, zip in or reel them in, rend \
           them into her plants' thorns, and reap the plants, so every drop spilled feeds what \
           grows.",
    rooms: &ROOMS,
};

/// Her stains that have been set off: a lit fuse.
fn lit_stains(w: &crate::World, me: usize) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.owner as usize == me && e.kind == EffectKind::Pool && e.life != u16::MAX)
        .count()
}

fn jumped(s: &Seen) -> bool {
    s.took_off()
}
fn dodged(s: &Seen) -> bool {
    s.dodged() && s.now().swim == 0 && s.moved().raw() < Fx::from_int(super::SLID).raw()
}
fn rend(s: &Seen) -> bool {
    s.out(REND) && s.now().grounded
}
fn third_rend(s: &Seen) -> bool {
    s.out(REND) && s.now().string & 0x7F == 2
}
fn claw(s: &Seen) -> bool {
    s.began(CLAW)
}
fn air_rend(s: &Seen) -> bool {
    s.out(REND) && !s.now().grounded
}
fn rising_rend(s: &Seen) -> bool {
    s.began(RISING_REND)
}
fn spike(s: &Seen) -> bool {
    s.began(BLACK_SPIKE)
}
fn rising_spike(s: &Seen) -> bool {
    s.began(RISING_SPIKE)
}
fn grew(s: &Seen) -> bool {
    let plants = |w: &crate::World| match w.players[s.me].mechanic {
        crate::class::Mechanic::Blood(g) => g.plants.iter().flatten().count(),
        _ => 0,
    };
    plants(s.after) > plants(s.before)
}
fn sweep(s: &Seen) -> bool {
    s.out(SWEEP) && s.now().grounded
}
fn third_sweep(s: &Seen) -> bool {
    s.out(SWEEP) && s.now().string & 0x7F == 2
}
fn cast(s: &Seen) -> bool {
    s.appeared(EffectKind::ScytheHead)
}
fn zip(s: &Seen) -> bool {
    let zipping = |w: &crate::World| matches!(w.players[s.me].mechanic, crate::class::Mechanic::Blood(g) if g.zipping);
    zipping(s.after) && !zipping(s.before)
}
fn air_sweep(s: &Seen) -> bool {
    s.out(SWEEP) && !s.now().grounded
}
fn rising_reap(s: &Seen) -> bool {
    s.began(RISING_REAP)
}
fn coagulate(s: &Seen) -> bool {
    s.began(COAGULATE)
}
fn heart(s: &Seen) -> bool {
    s.began(HEART)
}
fn swim(s: &Seen) -> bool {
    s.was().swim == 0 && s.now().swim == 1
}
fn strong(s: &Seen) -> bool {
    let kind = super::kind_of(s.now().action);
    kind.is_some_and(|k| s.began(k)) && s.now().on_blood && s.now().thrown_at.raw() > Fx::ONE.raw()
}
fn bled(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    was.bleeding == 0 && now.bleeding > 0 && now.bled_by == 0
}
fn set_off(s: &Seen) -> bool {
    lit_stains(s.after, s.me) > lit_stains(s.before, s.me)
}
fn drank(s: &Seen) -> bool {
    s.now().health > s.was().health && s.now().grey < s.was().grey
}
fn reaped(s: &Seen) -> bool {
    s.after
        .effects
        .iter()
        .flatten()
        .any(|e| e.owner as usize == s.me && e.slot == THORNS && e.age <= 1)
}
fn surfaced_strong(s: &Seen) -> bool {
    s.was().swim == 1 && s.now().swim != 1 && strong(s)
}
fn thrall(s: &Seen) -> bool {
    s.appeared(EffectKind::Thrall)
}

pub const ROOMS: [Room; 11] = [
    Room {
        title: "The Blood mage",
        advanced: false,
        intro: "Read the premise, then try your feet. Walk forward into the next room when \
                you are ready: every room opens as you walk into it. Watch your health bar: the \
                grey part is health you have spent and can still drink back.",
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
        title: "Left click: the Red Hand",
        advanced: false,
        intro: "Her other hand, gloved in her own blood. Tapped, left click rends -- a raking \
                claw at arm's length -- and tapped again strings up to three, the third the \
                heaviest, knocking them up. Held, it charges the Red claw: the longer you hold, \
                the further out along the floor it lands and the bigger it is -- hold too long \
                and it lands past them -- and it slows what it rakes. Raking one of your plants \
                feeds it.",
        tasks: &[
            Task {
                what: "Rend",
                keys: "Tap left click, close to the dummy",
                check: rend,
                lands: true,
            },
            Task {
                what: "Hold: the Red claw",
                keys: "Hold left click, close to the dummy",
                check: claw,
                lands: false,
            },
            Task {
                what: "A third rend",
                keys: "Tap left click three times in a row",
                check: third_rend,
                lands: true,
            },
            Task {
                what: "In the air",
                keys: "Space, then tap left click",
                check: air_rend,
                lands: false,
            },
            Task {
                what: "Takeoff: Rising rend",
                keys: "Space + left click together",
                check: rising_rend,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: the spike",
        advanced: false,
        intro: "Middle click raises the blood under the crosshair into a spike. A pool pays \
                toward it; on bare ground you pay it all in your own blood, and the plant holds \
                it as grey until you reap it back. A small pool rises fast, a big one slow. It \
                throws up whatever is over it, you too, and stands as a thorned plant that \
                lashes anybody beside it.",
        tasks: &[
            Task {
                what: "Black spike on a pool",
                keys: "Sweep the floor, then middle click on the stain",
                check: spike,
                lands: false,
            },
            Task {
                what: "A plant stands",
                keys: "Raise any pool",
                check: grew,
                lands: false,
            },
            Task {
                what: "Takeoff: Rising spike",
                keys: "Space + middle click together",
                check: rising_spike,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: the scythe",
        advanced: false,
        intro: "Tapped, right click sweeps the scythe -- longer and harder the more grey you \
                carry -- and tapped again strings up to three; on the floor it cuts the floor, \
                which bleeds. Held, it throws the head out on its chain: it arcs and falls, \
                and bites what it meets; a fighter it bites is leashed to you. Then left click \
                is the chain: tap it to be pulled to the head, hold it to reel them in -- into \
                a plant of yours and they are impaled -- or let go part way to rip it out of \
                them. Right click calls it home.",
        tasks: &[
            Task {
                what: "Sweep",
                keys: "Tap right click, close to the dummy",
                check: sweep,
                lands: true,
            },
            Task {
                what: "A third sweep",
                keys: "Tap right click three times in a row",
                check: third_sweep,
                lands: true,
            },
            Task {
                what: "Hold: Cast",
                keys: "Hold right click at the dummy or a wall",
                check: cast,
                lands: false,
            },
            Task {
                what: "Zip",
                keys: "While the head is out, tap left click",
                check: zip,
                lands: false,
            },
            Task {
                what: "In the air",
                keys: "Space, then tap right click",
                check: air_sweep,
                lands: false,
            },
            Task {
                what: "Takeoff: Rising reap",
                keys: "Space + right click together",
                check: rising_reap,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Q, E and Shift",
        advanced: false,
        intro: "Q pulls every stain near you to the crosshair and pools them. E plants a heart, \
                once you have earned one by dealing damage: it holds your ground and sends out \
                thralls. Shift on your ground sinks you in to swim; anywhere else it is a dodge.",
        tasks: &[
            Task {
                what: "Coagulate",
                keys: "Q, crosshair where the stains should go",
                check: coagulate,
                lands: false,
            },
            Task {
                what: "Plant a heart",
                keys: "E at the floor (once the heart is earned)",
                check: heart,
                lands: false,
            },
            Task {
                what: "Swim",
                keys: "Hold Shift standing on your ground",
                check: swim,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: fight on your ground",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from her. She plays well \
                without any of it. Everything she throws lays her dark ground where it touches. \
                Any move begun standing on it hits half again as hard -- she glows. Claim the \
                floor from outside, step onto it, then fight.",
        tasks: &[Task {
            what: "A strong hit from your ground",
            keys: "Lay ground near the dummy, stand on it, attack",
            check: strong,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: bleed, then spike",
        advanced: true,
        intro: "The cast's bite bleeds: a stain drops under the target every half second. A \
                spike on a stain sets it off, and the eruption jumps from stain to stain. Cast, \
                wait for a stain or two, then spike the trail.",
        tasks: &[
            Task {
                what: "Make the dummy bleed",
                keys: "Hold right click at it",
                check: bled,
                lands: false,
            },
            Task {
                what: "Set off its stains",
                keys: "Middle click on the stains under it",
                check: set_off,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: drink it back",
        advanced: true,
        intro: "Every spell turns red health grey, and grey fades. A stain gives it back only if \
                you reach it: walk over one, or drag the scythe's head home through it.",
        tasks: &[Task {
            what: "Drink a stain",
            keys: "Spend some health, then walk over a stain of yours",
            check: drank,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: reap",
        advanced: true,
        intro: "The scythe through one of your plants breaks it, and its thorns fly on the way \
                the blade was going, worth what the plant held -- and what you paid for it comes \
                back. Raise a plant between you and the dummy, then sweep through it.",
        tasks: &[Task {
            what: "Reap a plant",
            keys: "Raise a pool, then sweep through the plant",
            check: reaped,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: swim and surface",
        advanced: true,
        intro: "Sunk into her ground she moves fast and most blows pass over her. A click comes \
                up out of it as that move's strong version.",
        tasks: &[Task {
            what: "Surface into a strong hit",
            keys: "Swim up to the dummy, then click",
            check: surfaced_strong,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: hearts and thralls",
        advanced: true,
        intro: "A heart holds the ground joined to it and sends out thralls every few seconds; \
                on her ground they run faster and bite harder. Plant one near the dummy and keep \
                the dummy on your ground.",
        tasks: &[Task {
            what: "A heart sends a thrall",
            keys: "Earn a heart, E near the dummy, wait",
            check: thrall,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
];
