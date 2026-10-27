//! **The Blood mage's hall** (`docs/design/kits/blood-mage.md`, Territory
//! and the spike as a finisher): thorns, spikes, the scythe, the keys; then
//! her ground, the bleed into a spike, drinking back, spending grey, the
//! swim, and hearts.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::Class;
use crate::effects::EffectKind;
use crate::fixed::Fx;
use crate::moves::blood::*;

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::BloodMage,
    arena: ArenaId::HALL_BLOOD,
    what: "The Blood mage pays for every spell with her own health, and the blood she spills -- \
           hers and her enemy's -- stays on the floor as red stains and a dark ground that \
           belongs to her.",
    plays: "Standing on her ground she hits half again as hard and can sink into it and swim. \
            Cut the enemy so they bleed a trail of stains, drive a spike into the stains to blow \
            them up, and walk over stains to drink back the health you spent.",
    wins: "She wins by spreading her ground, fighting from inside it, and cashing in the health \
           she has on loan -- the grey part of her bar -- with a big held-to-the-end finisher.",
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
fn lash(s: &Seen) -> bool {
    s.out(THORN_LASH) && s.now().grounded
}
fn bramble(s: &Seen) -> bool {
    s.channelled(BRAMBLE)
}
fn vines(s: &Seen) -> bool {
    s.began(VINES)
}
fn air_lash(s: &Seen) -> bool {
    s.out(THORN_LASH) && !s.now().grounded
}
fn thornfall(s: &Seen) -> bool {
    s.began(THORNFALL)
}
fn spike(s: &Seen) -> bool {
    s.began(BLACK_SPIKE) && s.now().tier <= 1
}
fn big_spike(s: &Seen) -> bool {
    s.began(BLACK_SPIKE) && s.now().tier >= 3
}
fn cascade(s: &Seen) -> bool {
    s.began(CASCADE)
}
fn stake(s: &Seen) -> bool {
    s.began(STAKE)
}
fn upheaval(s: &Seen) -> bool {
    s.began(UPHEAVAL)
}
fn sweep(s: &Seen) -> bool {
    s.out(SWEEP) && s.now().grounded
}
fn third_sweep(s: &Seen) -> bool {
    s.out(SWEEP) && s.now().string & 0x7F == 2
}
fn reap(s: &Seen) -> bool {
    s.began(REAP)
}
fn harvest(s: &Seen) -> bool {
    s.began(HARVEST)
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
fn teleport(s: &Seen) -> bool {
    s.dodged() && s.moved().raw() > Fx::from_int(super::BLINKED).raw()
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
fn spent_grey(s: &Seen) -> bool {
    s.was().grey > 0 && s.began_any(&[VINES, CASCADE, HARVEST]) && s.now().fuel > 0
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
        title: "Left click: thorns",
        advanced: false,
        intro: "Tapped, left click lashes a thorn at the crosshair that makes them bleed: a \
                stain drops under them every half second. Held, it opens a field of thorns that \
                grows while you hold. Held to the very end, it is Black vines, which spends all \
                your grey.",
        tasks: &[
            Task {
                what: "Thorn lash",
                keys: "Tap left click at the dummy",
                check: lash,
                lands: true,
            },
            Task {
                what: "Hold: Bramble",
                keys: "Hold left click at the floor",
                check: bramble,
                lands: false,
            },
            Task {
                what: "Hold to the end: Black vines",
                keys: "Keep holding left click almost two seconds",
                check: vines,
                lands: true,
            },
            Task {
                what: "In the air",
                keys: "Space, then tap left click",
                check: air_lash,
                lands: false,
            },
            Task {
                what: "Takeoff: Thornfall",
                keys: "Space + left click together",
                check: thornfall,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: spikes",
        advanced: false,
        intro: "Middle click drives a black spike up at the crosshair. Hold it and more spikes \
                come up in a ring -- up to seven. Held to the end, it is a Crimson cascade that \
                sets off every stain near the crosshair. In the air it is a Stake that pins.",
        tasks: &[
            Task {
                what: "Black spike",
                keys: "Tap middle click at the dummy",
                check: spike,
                lands: true,
            },
            Task {
                what: "Hold for a ring of spikes",
                keys: "Hold middle click a second, let go",
                check: big_spike,
                lands: true,
            },
            Task {
                what: "Hold to the end: Crimson cascade",
                keys: "Keep holding middle click a second and a half",
                check: cascade,
                lands: true,
            },
            Task {
                what: "In the air: Stake",
                keys: "Space, then middle click",
                check: stake,
                lands: false,
            },
            Task {
                what: "Takeoff: Upheaval",
                keys: "Space + middle click together, crosshair on the dummy",
                check: upheaval,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: the scythe",
        advanced: false,
        intro: "Tapped, right click sweeps the scythe -- longer and harder the more grey you \
                carry -- and tapped again strings up to three sweeps. Held, it is a lunge, the \
                Reap. Held to the end, a Harvest: a full turn that drinks every stain round you.",
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
                what: "Hold: Reap",
                keys: "Hold right click, let go",
                check: reap,
                lands: true,
            },
            Task {
                what: "Hold to the end: Harvest",
                keys: "Keep holding right click almost a second",
                check: harvest,
                lands: true,
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
                lands: true,
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
                thralls. Shift is three moves in one: aimed at a stain it teleports you into it; \
                on your ground it sinks you in to swim; anywhere else it is a dodge.",
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
                what: "Teleport to a stain",
                keys: "Crosshair on a stain, Shift + a direction",
                check: teleport,
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
        intro: "Her core combo. A bleed drops stains under the target; a spike on a stain sets \
                it off, and the eruption jumps from stain to stain. Lash, wait for a stain or \
                two, then spike the trail.",
        tasks: &[
            Task {
                what: "Make the dummy bleed",
                keys: "Thorn lash it",
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
                you reach it: walk over one, teleport into one, or Harvest round you.",
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
        title: "Advanced: spend your grey",
        advanced: true,
        intro: "The three held-to-the-end moves burn all your grey for extra damage, and pay it \
                back as red if they land. So spend health first, then finish before the grey \
                fades.",
        tasks: &[Task {
            what: "A finisher on borrowed health",
            keys: "Two or three casts, then hold any click to the end",
            check: spent_grey,
            lands: true,
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
