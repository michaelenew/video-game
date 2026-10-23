//! **The Champion's hall** (`docs/design/kits/champion.md`, the 2026-10-12
//! rework): one room per weapon -- the string, the hold, the air, the
//! takeoff -- and the Rush; then the beat, momentum, the pole vault, the
//! spear's second jump, the clash, and the Rush into and out of a string.
//!
//! **A move is caught on its first live frame** ([`Seen::out`]), not its
//! windup: a held link shows the tap's windup before it becomes the hold,
//! and a takeoff can take over a swing that had begun on the floor.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::{Class, Fury, Mechanic};
use crate::moves::champion::*;
use crate::state::{Action, PARRY_FLOURISH};

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::Champion,
    arena: ArenaId::HALL_CHAMPION,
    what: "The Champion carries one weapon that changes shape: left click is a sword, middle \
           click a hammer, right click a spear.",
    plays: "Every attack is one hit of a three-hit string, and each hit can be any weapon. Tap \
            to keep the string going; hold a button to end it with a big finisher. E is the \
            Rush, a fast charge that also cuts the end off a swing.",
    wins: "Mixing weapons and hitting on the beat fills his momentum, and a finisher spends it \
           all at once. He wins by keeping up the pressure: hit, follow up, launch them, and be \
           there before they land.",
    rooms: &ROOMS,
};

/// His mechanic's parts the rooms read: the Rush's run and recharge, the
/// string, frames off the floor, and the fury.
struct Forms {
    rush: u16,
    recharge: u16,
    chain: u8,
    chain_hit: bool,
    lifted: u8,
    fury: Fury,
}

fn forms(m: Mechanic) -> Option<Forms> {
    match m {
        Mechanic::Forms {
            rush,
            recharge,
            chain,
            chain_hit,
            lifted,
            fury,
            ..
        } => Some(Forms {
            rush,
            recharge,
            chain,
            chain_hit,
            lifted,
            fury,
        }),
        _ => None,
    }
}

fn both(s: &Seen) -> Option<(Forms, Forms)> {
    Some((forms(s.was().mechanic)?, forms(s.now().mechanic)?))
}

fn jumped(s: &Seen) -> bool {
    s.took_off()
}
fn dodged(s: &Seen) -> bool {
    s.dodged()
}

/// Out on the floor: with her feet down as it began.
fn out_standing(s: &Seen, kind: u8) -> bool {
    s.out(kind) && s.now().grounded
}

fn sword(s: &Seen) -> bool {
    out_standing(s, SWORD_GROUND)
}
fn backcut(s: &Seen) -> bool {
    s.out(BACKCUT)
}
fn upcut(s: &Seen) -> bool {
    s.out(UPCUT)
}
fn crescent(s: &Seen) -> bool {
    s.out(CRESCENT)
}
fn thousand_cuts(s: &Seen) -> bool {
    s.out(THOUSAND_CUTS)
}
fn air_sword(s: &Seen) -> bool {
    s.out(AIR_SWORD)
}
fn rising_cut(s: &Seen) -> bool {
    s.out(RISING_CUT)
}

fn hammer(s: &Seen) -> bool {
    out_standing(s, HAMMER_GROUND)
}
fn uproot(s: &Seen) -> bool {
    s.out(UPROOT)
}
fn earthbreaker(s: &Seen) -> bool {
    s.out(EARTHBREAKER)
}
fn shockwave(s: &Seen) -> bool {
    s.out(SHOCKWAVE)
}
fn meteor(s: &Seen) -> bool {
    s.out(METEOR)
}
fn air_hammer(s: &Seen) -> bool {
    s.out(AIR_HAMMER)
}
fn uppercut(s: &Seen) -> bool {
    s.out(UPPERCUT)
}

fn spear(s: &Seen) -> bool {
    out_standing(s, SPEAR_GROUND)
}
fn skewer(s: &Seen) -> bool {
    s.out(SKEWER)
}
fn whirl(s: &Seen) -> bool {
    s.out(WHIRL)
}
fn repel(s: &Seen) -> bool {
    s.out(REPEL)
}
fn heaven_piercer(s: &Seen) -> bool {
    s.out(HEAVEN_PIERCER)
}
fn air_spear(s: &Seen) -> bool {
    s.out(AIR_SPEAR)
}
fn pole_drive(s: &Seen) -> bool {
    s.out(POLE_DRIVE)
}

/// **A Rush, spent**: the recharge jumps. A finisher's drive sets the run
/// too, without spending anything, so the run alone is not a Rush.
fn rushed(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| now.rush > 0 && now.recharge > was.recharge.saturating_add(1))
}
fn rush_attack(s: &Seen) -> bool {
    s.out(RUSH_SLASH) || s.out(RUSH_SWEEP) || s.out(RUSH_STAB)
}
fn rush_cancel(s: &Seen) -> bool {
    matches!(s.was().action, Action::Recovery { .. })
        && matches!(s.now().action, Action::Free)
        && rushed(s)
}
fn shoulder(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    both(s).is_some_and(|(_, f)| f.chain == 1 && f.chain_hit && f.rush == 0)
        && matches!(s.now().action, Action::Free)
        && super::struck(was, now)
}
fn refund(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| was.recharge > 1 && now.recharge == 0)
}

fn true_link(s: &Seen) -> bool {
    let kind = super::kind_of(s.now().action);
    kind.is_some_and(|k| s.out(k) && in_string(k)) && both(s).is_some_and(|(_, f)| f.fury.true_link)
}
fn true_third(s: &Seen) -> bool {
    true_link(s) && (s.out(UPCUT) || s.out(EARTHBREAKER) || s.out(WHIRL))
}
fn full_meter(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| was.fury.tier() < 3 && now.fury.tier() == 3)
}
fn spent_meter(s: &Seen) -> bool {
    both(s).is_some_and(|(was, now)| {
        was.fury.momentum > 0 && now.fury.momentum == 0 && now.fury.spent >= 90
    })
}
fn pole_vault(s: &Seen) -> bool {
    s.out(POLE_VAULT)
}
fn air_pole_drive(s: &Seen) -> bool {
    s.out(POLE_DRIVE) && both(s).is_some_and(|(was, _)| was.lifted > 4)
}
fn spear_after_drive(s: &Seen) -> bool {
    s.out(AIR_SPEAR) && pole_driven(s)
}
/// This trip off the floor has had its takeoff: the pole drive, spent.
fn pole_driven(s: &Seen) -> bool {
    matches!(
        s.now().mechanic,
        Mechanic::Forms {
            rise_used: true,
            ..
        }
    )
}
fn clash(s: &Seen) -> bool {
    let (_, dummy) = s.dummy();
    s.was().parried < PARRY_FLOURISH
        && s.now().parried >= PARRY_FLOURISH
        && s.now().health >= s.was().health
        && matches!(dummy.action, Action::Stagger { .. })
}

pub const ROOMS: [Room; 11] = [
    Room {
        title: "The Champion",
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
        title: "Left click: the sword",
        advanced: false,
        intro: "The sword is quick and close. Tap three times for a three-hit string: tap \
                again just after each swing ends. Holding instead of tapping turns that hit \
                into a finisher. In the air, let go of Space before you click.",
        tasks: &[
            Task {
                what: "First hit",
                keys: "Tap left click",
                check: sword,
                lands: true,
            },
            Task {
                what: "Second hit",
                keys: "Tap left click again as the first ends",
                check: backcut,
                lands: true,
            },
            Task {
                what: "Third hit, a launcher",
                keys: "And a third tap",
                check: upcut,
                lands: true,
            },
            Task {
                what: "Hold: Crescent",
                keys: "Hold left click (a blade of light flies out)",
                check: crescent,
                lands: true,
            },
            Task {
                what: "Hold on the third: Thousand cuts",
                keys: "Tap, tap, then hold",
                check: thousand_cuts,
                lands: true,
            },
            Task {
                what: "In the air",
                keys: "Space, let go, then left click",
                check: air_sword,
                lands: true,
            },
            Task {
                what: "Takeoff: Rising cut",
                keys: "Space + left click together",
                check: rising_cut,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: the hammer",
        advanced: false,
        intro: "The hammer is slow and heavy. The same three-hit string, the same hold for a \
                finisher -- hold about half a second, the hammer winds up longer. Its third hit \
                cannot be blocked.",
        tasks: &[
            Task {
                what: "First hit",
                keys: "Tap middle click",
                check: hammer,
                lands: true,
            },
            Task {
                what: "Second hit",
                keys: "Tap middle click again",
                check: uproot,
                lands: true,
            },
            Task {
                what: "Third hit: Earthbreaker",
                keys: "And a third tap",
                check: earthbreaker,
                lands: true,
            },
            Task {
                what: "Hold: Shockwave",
                keys: "Hold middle click",
                check: shockwave,
                lands: true,
            },
            Task {
                what: "Hold on the third: Meteor",
                keys: "Tap, tap, then hold (he leaps to the crosshair)",
                check: meteor,
                lands: true,
            },
            Task {
                what: "In the air",
                keys: "Space, let go, then middle click",
                check: air_hammer,
                lands: false,
            },
            Task {
                what: "Takeoff: Uppercut",
                keys: "Space + middle click together, close to the dummy",
                check: uppercut,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: the spear",
        advanced: false,
        intro: "The spear reaches furthest. String, hold, air and takeoff again -- and its \
                takeoff, the Pole drive, throws you higher than any other.",
        tasks: &[
            Task {
                what: "First hit",
                keys: "Tap right click",
                check: spear,
                lands: true,
            },
            Task {
                what: "Second hit",
                keys: "Tap right click again",
                check: skewer,
                lands: true,
            },
            Task {
                what: "Third hit: Whirl",
                keys: "And a third tap",
                check: whirl,
                lands: true,
            },
            Task {
                what: "Hold: Repel",
                keys: "Hold right click",
                check: repel,
                lands: true,
            },
            Task {
                what: "Hold on the third: Heaven-piercer",
                keys: "Tap, tap, then hold",
                check: heaven_piercer,
                lands: true,
            },
            Task {
                what: "In the air",
                keys: "Space, let go, then right click",
                check: air_spear,
                lands: false,
            },
            Task {
                what: "Takeoff: Pole drive",
                keys: "Space + right click together",
                check: pole_drive,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "E: the Rush",
        advanced: false,
        intro: "E charges toward the crosshair. A click during the charge is a charging attack, \
                and E pressed while a swing is still finishing cuts the end off it. The charge \
                comes back a second after it ends.",
        tasks: &[
            Task {
                what: "Rush",
                keys: "Point the crosshair, then E",
                check: rushed,
                lands: false,
            },
            Task {
                what: "Attack out of a Rush",
                keys: "E, then any click straight away",
                check: rush_attack,
                lands: true,
            },
            Task {
                what: "Rush out of a swing",
                keys: "Any click, then E before it finishes",
                check: rush_cancel,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: on the beat",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from him. He plays well \
                without any of it. The string has a beat. Press the next hit a moment after the \
                last swing ends -- not as early as you can -- and it is a true link: more damage, \
                a heavier hit. The tempo bar under momentum marks the window. Mashing never \
                finds it.",
        tasks: &[
            Task {
                what: "A true link",
                keys: "Tap, wait a beat after the swing ends, tap",
                check: true_link,
                lands: true,
            },
            Task {
                what: "A true link on a third hit",
                keys: "Keep the beat through all three",
                check: true_third,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: momentum",
        advanced: true,
        intro: "Every hit that lands fills momentum; a different weapon from the last hit adds \
                more, and so does the beat. Full, every hit is worth more. A finisher spends \
                the whole bar for a much bigger hit -- so fill it, then hold.",
        tasks: &[
            Task {
                what: "Fill the bar",
                keys: "Mix weapons on the dummy: left, right, middle, on the beat",
                check: full_meter,
                lands: false,
            },
            Task {
                what: "Spend it on a finisher",
                keys: "With the bar full, hold any click",
                check: spent_meter,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the pole vault",
        advanced: true,
        intro: "During a Rush, right click with the crosshair on the floor: he plants the spear \
                and turns the run into height, higher than a jump and still moving. Aim at empty \
                floor -- running into a body ends the Rush. Try for the shelf.",
        tasks: &[Task {
            what: "Vault",
            keys: "E, then look down and right click straight away",
            check: pole_vault,
            lands: false,
        }],
        dummy: Dummy::Aside,
        bay: Bay::Shelf(450),
    },
    Room {
        title: "Advanced: jump, then right click",
        advanced: true,
        intro: "His air pattern. Keep Space held after a jump and right click at the top: the \
                Pole drive comes out in the air as a second jump, pushed the way you hold. Then \
                let go of Space and right click again for the air spear, which pushes you on \
                again if it hits.",
        tasks: &[
            Task {
                what: "Pole drive in the air",
                keys: "Space and keep holding; right click at the top, holding W",
                check: air_pole_drive,
                lands: false,
            },
            Task {
                what: "Then the air spear",
                keys: "Let go of Space, right click at the dummy",
                check: spear_after_drive,
                lands: true,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the clash",
        advanced: true,
        intro: "While a finisher (or Earthbreaker) is live, any blow that meets him is \
                cancelled: he takes nothing and the attacker is staggered. It is a timing, \
                not a trade: if your finisher reaches them first they are simply hit. This \
                dummy thrusts its spear every few seconds -- stand three or four metres off, \
                outside your hammer's reach, and have the Shockwave out as the spear arrives.",
        tasks: &[Task {
            what: "Clash its thrust",
            keys: "Three or four metres off, start holding middle click as the count reads 0.2",
            check: clash,
            lands: false,
        }],
        dummy: Dummy::Swings,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: Rush into the string",
        advanced: true,
        intro: "Running into a body with the Rush is a shoulder: it counts as the first hit of \
                a string. A third hit that lands gives the Rush straight back, so you can Rush \
                out of its recovery -- in, three hits, out.",
        tasks: &[
            Task {
                what: "Shoulder in",
                keys: "E straight at the dummy",
                check: shoulder,
                lands: false,
            },
            Task {
                what: "Earn the Rush back",
                keys: "Land a third hit while the Rush is recharging",
                check: refund,
                lands: false,
            },
            Task {
                what: "Rush out",
                keys: "E as the third hit finishes",
                check: rush_cancel,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
];
