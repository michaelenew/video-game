//! **The Dual mage's hall** (`docs/design/kits/dual-mage.md`, her hands):
//! dark, twilight and light, the keys; then the hands kept in the band,
//! venting, curse and purge, the air hop, what the lower bar unlocks, and
//! the Flurry dash's jump.
//!
//! A click opens a hold on the press and its tap comes out on the release,
//! so a move is caught on its windup ([`Seen::began`]), never the press.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::{Class, Force, Mechanic};
use crate::fixed::Fx;
use crate::moves::dual::*;
use crate::state::Action;

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::DualMage,
    arena: ArenaId::HALL_DUAL,
    what: "The Dual mage is a close-range caster with two forces, Dark and Light, one in each \
           hand, each with its own bar.",
    plays: "Left click fills Dark and right click fills Light, and after each quick hit she \
            hops a few steps the way you hold, so a run of hits is a run of hops. The fuller the \
            lower of her two bars, the more her body can do: blink, double jump, and at the top \
            grow wings. Let one bar run far ahead of the other and it burns her health, so keep \
            switching hands.",
    wins: "She wins by cursing the enemy with Dark, cashing the curse with Light, and spending a \
           full bar on a big finishing spell.",
    rooms: &ROOMS,
};

struct Meter {
    ascending: u16,
    jumped: bool,
    hopped: bool,
    dash_jump: bool,
}

fn meter(m: Mechanic) -> Option<Meter> {
    match m {
        Mechanic::Meter {
            ascending,
            jumped,
            hopped,
            dash_jump,
            ..
        } => Some(Meter {
            ascending,
            jumped,
            hopped,
            dash_jump,
        }),
        _ => None,
    }
}

fn both(s: &Seen) -> Option<(Meter, Meter)> {
    Some((meter(s.was().mechanic)?, meter(s.now().mechanic)?))
}

fn jumped(s: &Seen) -> bool {
    s.took_off()
}
fn dodged(s: &Seen) -> bool {
    s.dodged()
}
fn dark(s: &Seen) -> bool {
    s.began_standing(DARK_AUTO)
}
fn air_dark(s: &Seen) -> bool {
    s.began_aloft(DARK_AUTO)
}
/// A hop off an auto: carried the way she holds, as its swing ends.
fn hop(s: &Seen) -> bool {
    s.was().haul == 0
        && s.now().haul > 0
        && matches!(
            s.now().action,
            Action::Recovery {
                kind: DARK_AUTO | LIGHT_AUTO,
                ..
            }
        )
}
fn chain(s: &Seen) -> bool {
    s.began(CHAIN_LIGHTNING)
}
fn veil(s: &Seen) -> bool {
    s.began(VEIL)
}
fn binary(s: &Seen) -> bool {
    s.began(BINARY)
}
fn phase(s: &Seen) -> bool {
    s.began(PHASE)
}
fn flurry(s: &Seen) -> bool {
    s.began(FLURRY_DASH)
}
fn equinox(s: &Seen) -> bool {
    s.began(EQUINOX)
}
fn light(s: &Seen) -> bool {
    s.began(LIGHT_AUTO)
}
fn disk_ray(s: &Seen) -> bool {
    s.began(DISK_RAY)
}
fn sanctuary(s: &Seen) -> bool {
    s.began(SANCTUARY)
}
fn devour(s: &Seen) -> bool {
    s.began(DEVOUR)
}
fn smite(s: &Seen) -> bool {
    s.began(SMITE)
}
fn abyss(s: &Seen) -> bool {
    s.began(ABYSS)
}
fn judgement(s: &Seen) -> bool {
    s.began(JUDGEMENT)
}
/// Both bars a third full, side by side: the lower crossed it inside the
/// band.
fn in_the_band(s: &Seen) -> bool {
    let line = Fx::from_int(super::BARS_THIRD);
    crate::dual::lower(s.was()).raw() < line.raw()
        && crate::dual::lower(s.now()).raw() >= line.raw()
        && crate::dual::gap(s.now()).raw() <= Fx::from_int(crate::tuning::meter_band()).raw()
}
fn vented(s: &Seen) -> bool {
    let band = Fx::from_int(crate::tuning::meter_band()).raw();
    crate::dual::gap(s.was()).raw() > band
        && crate::dual::gap(s.now()).raw() <= band
        && matches!(
            super::kind_of(s.now().action),
            Some(DEVOUR | SMITE | ABYSS | JUDGEMENT | EQUINOX)
        )
}
fn cursed(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    crate::dual::stacks(was.curse) < 3 && crate::dual::stacks(now.curse) >= 3
}
fn purged(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    crate::dual::stacks(was.curse) >= 3
        && crate::dual::stacks(now.curse) == 0
        && super::struck(was, now)
        && super::kind_of(s.now().action).and_then(force) == Some(Force::Light)
}
fn air_hop(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| !was.hopped && now.hopped)
}
fn blink(s: &Seen) -> bool {
    s.dodged() && s.moved().raw() > Fx::ONE.raw()
}
fn second_jump(s: &Seen) -> bool {
    !s.was().grounded && both(s).is_some_and(|(was, now)| !was.jumped && now.jumped)
}
fn wings(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| was.ascending == 0 && now.ascending > 0)
}
fn flurry_jump(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| was.dash_jump && !now.dash_jump)
        && !s.now().grounded
        && s.now().vel.y.raw() > 0
}

pub const ROOMS: [Room; 11] = [
    Room {
        title: "The Dual mage",
        advanced: false,
        intro: "Read the premise, then try your feet. Walk forward into the next room when \
                you are ready: every room opens as you walk into it. Her two bars are under \
                your health: Dark on the left, Light on the right.",
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
        title: "Left click: Dark",
        advanced: false,
        intro: "Tapped, left click throws a dark ball that curses whoever it hits, and fills \
                your Dark bar. Hold a direction as you tap and she hops that way after it. Held a \
                moment, it is Chain lightning, leaping from body to body. The same hand can't \
                tap twice in half a second -- that is what the other hand is for.",
        tasks: &[
            Task {
                what: "Dark ball, on the floor",
                keys: "Tap left click at the dummy",
                check: dark,
                lands: true,
            },
            Task {
                what: "Dark ball, in the air",
                keys: "Space, then tap left click",
                check: air_dark,
                lands: false,
            },
            Task {
                what: "Hop after it",
                keys: "Hold W, A, S or D and tap left click",
                check: hop,
                lands: false,
            },
            Task {
                what: "Hold: Chain lightning",
                keys: "Hold left click, crosshair near the dummy",
                check: chain,
                lands: true,
            },
            Task {
                what: "Takeoff: Veil",
                keys: "Space + left click together (untouchable as you rise)",
                check: veil,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: both at once",
        advanced: false,
        intro: "Middle click uses both forces. On the floor, Binary: two stars that circle you \
                and strike whoever comes close. In the air, Phase: a short blink along the \
                crosshair, once a jump. Held, the Flurry dash: blows out to the crosshair, then \
                she runs to them.",
        tasks: &[
            Task {
                what: "Binary",
                keys: "Tap middle click on the floor",
                check: binary,
                lands: false,
            },
            Task {
                what: "In the air: Phase",
                keys: "Space, then tap middle click",
                check: phase,
                lands: false,
            },
            Task {
                what: "Hold: the Flurry dash",
                keys: "Hold middle click, crosshair on the dummy",
                check: flurry,
                lands: true,
            },
            Task {
                what: "Takeoff: Equinox",
                keys: "Space + middle click together (it evens your bars)",
                check: equinox,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: Light",
        advanced: false,
        intro: "Tapped, right click throws a light ball that fills your Light bar and burns away \
                every curse on whoever it hits. It hops you too. Held, the Disk ray: a sun opens \
                over your head and fires a beam to the crosshair.",
        tasks: &[
            Task {
                what: "Light ball",
                keys: "Tap right click at the dummy",
                check: light,
                lands: true,
            },
            Task {
                what: "Hold: Disk ray",
                keys: "Hold right click, crosshair on the dummy",
                check: disk_ray,
                lands: true,
            },
            Task {
                what: "Takeoff: Sanctuary",
                keys: "Space + right click together",
                check: sanctuary,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "The keys",
        advanced: false,
        intro: "F and R are spells that let a bar down: F, Devour, a dark maw; R, Smite, a \
                column of light. Q and E are her majors, and each spends a whole bar: Q, the \
                Abyss, spends Dark; E, Judgement, spends Light. They grow with the bar they \
                spend. Shift dodges -- and once your lower bar is a third full, blinks.",
        tasks: &[
            Task {
                what: "Devour",
                keys: "F at the dummy",
                check: devour,
                lands: true,
            },
            Task {
                what: "Smite",
                keys: "R at the dummy",
                check: smite,
                lands: true,
            },
            Task {
                what: "Abyss",
                keys: "Q at the dummy",
                check: abyss,
                lands: false,
            },
            Task {
                what: "Judgement",
                keys: "E at the dummy",
                check: judgement,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: switch hands",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from her. She plays well \
                without any of it. Each hand fills its own bar. Let one get far ahead and it \
                runs away and burns your health; alternate and both climb together. Get both a \
                third full without letting them split.",
        tasks: &[Task {
            what: "Both bars a third full, together",
            keys: "Alternate left and right taps",
            check: in_the_band,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: vent a runaway bar",
        advanced: true,
        intro: "The key spells let their own bar down and the majors empty theirs: that is how a \
                bar that has run away is pulled back. Let one get ahead -- three taps of one \
                hand -- then bring it back.",
        tasks: &[Task {
            what: "Pull a runaway bar back",
            keys: "Dark ahead: F or Q. Light ahead: R or E. Or Space + middle click",
            check: vented,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: curse, then purge",
        advanced: true,
        intro: "Dark lays curses on the target, up to five, and they fade one at a time. Light \
                cashes every curse at once, and three or more stagger them. Curse, then purge, \
                before they fade.",
        tasks: &[
            Task {
                what: "Three curses on the dummy",
                keys: "Left click, F, or Chain lightning",
                check: cursed,
                lands: false,
            },
            Task {
                what: "Purge them",
                keys: "Right click, R, or the Disk ray",
                check: purged,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the hop chain",
        advanced: true,
        intro: "Her way of moving is fighting: every auto with a direction held is a hop, and \
                alternating hands gets round the lockout -- left, right, left, right across the \
                room. The first auto off the floor in a jump rises like a second jump.",
        tasks: &[
            Task {
                what: "A hop",
                keys: "Hold W and tap left click",
                check: hop,
                lands: false,
            },
            Task {
                what: "The rising hop",
                keys: "Jump, then hold W and tap an auto",
                check: air_hop,
                lands: false,
            },
        ],
        dummy: Dummy::Aside,
        bay: Bay::Shelf(250),
    },
    Room {
        title: "Advanced: what the lower bar unlocks",
        advanced: true,
        intro: "Her body follows the lower of her two bars. A third full, the dodge is a blink. \
                Two thirds full, she has a second jump. Both full together, she ascends: wings \
                for six seconds, every cast at full power. Build both bars with alternating taps \
                and try each.",
        tasks: &[
            Task {
                what: "Blink",
                keys: "Lower bar a third full: Shift + a direction",
                check: blink,
                lands: false,
            },
            Task {
                what: "Second jump",
                keys: "Lower bar two thirds full: Space, then Space again",
                check: second_jump,
                lands: false,
            },
            Task {
                what: "Ascend",
                keys: "Fill both bars to the top",
                check: wings,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the Flurry dash jump",
        advanced: true,
        intro: "The Flurry dash is her long-range way in, and a way across a gap: press Space \
                just before she arrives at the end of the run and the jump keeps most of its \
                speed.",
        tasks: &[Task {
            what: "Jump out of the run",
            keys: "Hold middle click at a far spot, then Space as you arrive",
            check: flurry_jump,
            lands: false,
        }],
        dummy: Dummy::Aside,
        bay: Bay::Gap(150, 800),
    },
];
