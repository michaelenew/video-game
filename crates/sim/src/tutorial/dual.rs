//! **The Dual mage's hall** (`docs/design/kits/dual-mage.md`, her hands, and
//! `docs/design/exploration/0037_dual_mage_the_razor_edge.md`): her magic is
//! aimed at her. The two balls and their holds, the chain, both clicks at
//! once; then the hands kept in the band, venting, the hop chain, the springs
//! and Ascension, and the chain ride's jump.
//!
//! A click opens a hold on the press and its tap comes out on the release,
//! so a move is caught on its windup ([`Seen::began`]), never the press.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::{Class, Mechanic};
use crate::fixed::Fx;
use crate::moves::dual::*;
use crate::state::Action;

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::DualMage,
    arena: ArenaId::HALL_DUAL,
    what: "The Dual mage holds two beings, Dark and Light, that are trying to kill her. \
           Everything they throw comes from behind her, at her; she gets out of the way, and \
           it lands on whatever is in front of her.",
    plays: "Left click is Dark and right click is Light: a quick strike she steps off, and a \
            hop the way you hold. Hold a click for the big one -- right after a strike lands, \
            it comes out fast. Both at once and the two collide where she was standing as she \
            springs clear. Every hit she lands fills a fifth of a bar; a big hold costs a third \
            of its bar, Q and E need a full one, and both full makes the next annihilation the \
            grand one. Let the two bars drift more than a third apart and she burns.",
    wins: "She wins by staying just out of reach -- in close enough that the strikes she dodges \
           land on them -- and building both bars to a grand annihilation, or spending one on a \
           major.",
    rooms: &ROOMS,
};

struct Meter {
    springs: u8,
    hopped: bool,
    dash_jump: bool,
}

fn meter(m: Mechanic) -> Option<Meter> {
    match m {
        Mechanic::Meter {
            springs,
            hopped,
            dash_jump,
            ..
        } => Some(Meter {
            springs,
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
/// A hop off an auto: carried the way she holds, as its strike ends.
fn hop(s: &Seen) -> bool {
    // As the strike passes her (2026-11-06): inside it, not after.
    s.was().haul == 0
        && s.now().haul > 0
        && matches!(
            s.now().action,
            Action::Active {
                kind: DARK_AUTO | LIGHT_AUTO,
                ..
            } | Action::Recovery {
                kind: DARK_AUTO | LIGHT_AUTO,
                ..
            }
        )
}
fn storm(s: &Seen) -> bool {
    s.began(STORM)
}
fn light(s: &Seen) -> bool {
    s.began(LIGHT_AUTO)
}
fn lance(s: &Seen) -> bool {
    s.began(LANCE)
}
fn chain(s: &Seen) -> bool {
    s.began(CHAIN)
}
fn yank(s: &Seen) -> bool {
    s.began(YANK)
}
fn ride(s: &Seen) -> bool {
    s.began(FLURRY_DASH)
}
fn annihilation(s: &Seen) -> bool {
    s.began_standing(ANNIHILATION)
}
fn air_annihilation(s: &Seen) -> bool {
    s.began_aloft(ANNIHILATION)
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
        && matches!(super::kind_of(s.now().action), Some(ABYSS | JUDGEMENT))
}
fn air_hop(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| !was.hopped && now.hopped)
}
/// A second spring in one airtime: less than the first, and still up.
fn second_spring(s: &Seen) -> bool {
    !s.now().grounded && both(s).is_some_and(|(was, now)| was.springs == 1 && now.springs == 2)
}
/// The grand annihilation: thrown with both bars full.
fn grand(s: &Seen) -> bool {
    s.began(ANNIHILATION) && crate::dual::brimming(s.was())
}
/// A hold thrown primed: a strike landed a moment ago.
fn primed(s: &Seen) -> bool {
    matches!(s.was().mechanic, Mechanic::Meter { primed: p, .. } if p > 0)
        && (s.began(STORM) || s.began(LANCE))
}
fn ride_jump(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| was.dash_jump && !now.dash_jump)
        && !s.now().grounded
        && s.now().vel.y.raw() > 0
}

pub const ROOMS: [Room; 10] = [
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
        intro: "Tapped, left click is a dark strike from behind her, at her: she steps off it \
                and it flies on into whatever she faces. Hold a direction as you tap and she hops \
                that way, as it passes her. Held a moment, the storm: she leaps to the crosshair \
                (in the air, straight toward it, up as well), the lightning aimed at her comes \
                down there and leaps to whoever is near, and she carries on through -- or the \
                way you hold. It costs a third of your Dark bar.",
        tasks: &[
            Task {
                what: "Dark strike, on the floor",
                keys: "Tap left click at the dummy",
                check: dark,
                lands: true,
            },
            Task {
                what: "Dark strike, in the air",
                keys: "Space, then tap left click",
                check: air_dark,
                lands: false,
            },
            Task {
                what: "Hop off it",
                keys: "Hold W, A, S or D and tap left click",
                check: hop,
                lands: false,
            },
            Task {
                what: "Hold: the storm",
                keys: "Land two dark strikes, then hold left click at the dummy",
                check: storm,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: Light",
        advanced: false,
        intro: "Tapped, right click is the same strike in light; one that lands fills your Light \
                bar. \
                Held, the lance: a great beam from behind her, through where she stood, to the \
                crosshair -- and she lunges clear of it, back, or the way you hold. It costs a \
                third of your Light bar.",
        tasks: &[
            Task {
                what: "Light strike",
                keys: "Tap right click at the dummy",
                check: light,
                lands: true,
            },
            Task {
                what: "Hold: the lance",
                keys: "Land two light strikes, then hold right click at the dummy",
                check: lance,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: the chain",
        advanced: false,
        intro: "Middle click flings both forces, linked, at her. Tapped, she lets the chain \
                fly by and it binds the first body it meets -- tap again while it is still on \
                them and it yanks them to her. Held, she catches its tail and is dragged down the \
                line toward the crosshair, through whoever is on it.",
        tasks: &[
            Task {
                what: "The chain",
                keys: "Tap middle click, crosshair on the dummy",
                check: chain,
                lands: true,
            },
            Task {
                what: "Yank",
                keys: "Tap middle click at the dummy, then tap it again",
                check: yank,
                lands: false,
            },
            Task {
                what: "Hold: ride the chain",
                keys: "Hold middle click, crosshair on the dummy",
                check: ride,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Both clicks, and the keys",
        advanced: false,
        intro: "Press left and right together and both beings lunge at her at once: she \
                springs clear -- up, or the way you hold -- and they annihilate where she was \
                standing. Close to the dummy, the blast is on them; in the air, it is her way \
                up. Q and E are her majors: each needs a full bar and spends it -- Q, the Abyss, \
                Dark; E, Judgement, Light. They come down where she stands and she dashes out, \
                so get in close first. The Abyss and then Judgement on the same spot is the big \
                one.",
        tasks: &[
            Task {
                what: "Annihilation beside the dummy",
                keys: "Left and right click together",
                check: annihilation,
                lands: true,
            },
            Task {
                what: "In the air: spring higher",
                keys: "Space, then left and right click together",
                check: air_annihilation,
                lands: false,
            },
            Task {
                what: "Abyss",
                keys: "Five dark strikes on the dummy, then Q beside it",
                check: abyss,
                lands: false,
            },
            Task {
                what: "Judgement",
                keys: "Five light strikes on the dummy, then E beside it",
                check: judgement,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: strike, then the big one",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from her. She plays well \
                without any of it. Every hit that lands fills the bar of its hand, so alternate \
                and both climb together. And a hold thrown right after a strike lands skips most \
                of its wind-up.",
        tasks: &[
            Task {
                what: "Both bars a third full, together",
                keys: "Alternate left and right taps on the dummy",
                check: in_the_band,
                lands: false,
            },
            Task {
                what: "A primed hold",
                keys: "Tap a click on the dummy, then hold one at once",
                check: primed,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: spend a bar",
        advanced: true,
        intro: "A major needs its bar full and spends all of it. Fill one -- five hits of one \
                hand -- and spend it before the two drift a third apart and start to burn you.",
        tasks: &[Task {
            what: "Spend the bar that is ahead",
            keys: "Dark ahead: Q. Light ahead: E",
            check: vented,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the hop chain",
        advanced: true,
        intro: "Every strike with a direction held is a hop, and alternating hands gets round \
                the lockout -- left, right, left, right across the room. The first strike off \
                the floor in a jump rises like a second jump.",
        tasks: &[
            Task {
                what: "A hop",
                keys: "Hold W and tap left click",
                check: hop,
                lands: false,
            },
            Task {
                what: "The rising hop",
                keys: "Jump, then hold W and tap a click",
                check: air_hop,
                lands: false,
            },
        ],
        dummy: Dummy::Aside,
        bay: Bay::Shelf(250),
    },
    Room {
        title: "Advanced: springs, and the grand annihilation",
        advanced: true,
        intro: "Each spring in one jump goes less high than the last, and every hit you land \
                gives one back -- so she stays up for as long as she keeps hitting. Fill both \
                bars to the top and the next annihilation is the grand one: far bigger, and it \
                spends them both.",
        tasks: &[
            Task {
                what: "Two springs in one jump",
                keys: "Space, then left and right together, twice",
                check: second_spring,
                lands: false,
            },
            Task {
                what: "The grand annihilation",
                keys: "Both bars full, then left and right together",
                check: grand,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: jump off the chain",
        advanced: true,
        intro: "Riding the chain is her long way in, and a way across a gap: press Space just \
                before she arrives at the end of the run and the jump keeps most of its speed.",
        tasks: &[Task {
            what: "Jump off the ride",
            keys: "Hold middle click at a far spot, then Space as you arrive",
            check: ride_jump,
            lands: false,
        }],
        dummy: Dummy::Aside,
        bay: Bay::Gap(150, 800),
    },
];
