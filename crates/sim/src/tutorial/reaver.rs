//! **The Shadow Reaver's hall** (`docs/design/kits/shadow-reaver.md`, the
//! third pass): her finishers, her thrown blades, her shadow, then the dash
//! jump, the air pattern, the marks, the lotus drag and the cut recovery.

use super::{Bay, Dummy, Room, Seen, Task, Tutorial};
use crate::arena::ArenaId;
use crate::class::{Class, Ghost};
use crate::effects::EffectKind;
use crate::moves::reaver::*;
use crate::state::Action;

pub static TUTORIAL: Tutorial = Tutorial {
    class: Class::ShadowReaver,
    arena: ArenaId::HALL_REAVER,
    what: "The Shadow Reaver fights with two bodies: her own, and a shadow copy of herself that \
           follows at her shoulder and repeats every sword swing she makes.",
    plays: "Throw the shadow out to a spot on the floor and it cuts people from there; dash to it \
            and you cross the room in a blink. Every shadow blade that lands leaves a mark on the \
            target, up to five.",
    wins: "She is fragile, so she marks from a safe distance, then darts in with a left-click \
           finisher that spends every mark at once -- the more marks, the bigger the hit -- and \
           gets out again.",
    rooms: &ROOMS,
};

fn jumped(s: &Seen) -> bool {
    s.took_off()
}
fn dodged(s: &Seen) -> bool {
    s.dodged() && s.shadow().is_none_or(|(_, now)| now.dash == 0)
}
fn reave(s: &Seen) -> bool {
    s.began_standing(REAVE)
}
fn plunge(s: &Seen) -> bool {
    s.began_aloft(PLUNGE)
}
fn gallows(s: &Seen) -> bool {
    s.began(GALLOWS)
}
fn shuriken(s: &Seen) -> bool {
    s.began(SHURIKEN)
}
fn tumble(s: &Seen) -> bool {
    s.began_aloft(TUMBLE)
}
fn fan(s: &Seen) -> bool {
    s.began(FAN)
}
fn sent(s: &Seen) -> bool {
    s.shadow().is_some_and(|(was, now)| {
        matches!(was.doing, Ghost::Attending) && matches!(now.doing, Ghost::Casting { .. })
    })
}
fn sent_again(s: &Seen) -> bool {
    s.shadow().is_some_and(|(was, now)| {
        matches!(was.doing, Ghost::Waiting | Ghost::Casting { .. })
            && matches!(now.doing, Ghost::Casting { age: 0, .. })
            && !matches!(was.doing, Ghost::Casting { age: 0, .. })
    })
}
fn copied(s: &Seen) -> bool {
    s.began_any(&[REAVE, EXECUTIONER])
        && s.shadow()
            .is_some_and(|(was, _)| matches!(was.doing, Ghost::Waiting))
}
fn whirl(s: &Seen) -> bool {
    s.began(WHIRL)
}
fn rising_whirl(s: &Seen) -> bool {
    s.began(RISING_WHIRL)
}
fn collected(s: &Seen) -> bool {
    s.shadow().is_some_and(|(was, now)| {
        matches!(was.doing, Ghost::Waiting) && matches!(now.doing, Ghost::Attending)
    }) && s.appeared(EffectKind::ShadowBurst)
        && s.shadow().is_some_and(|(_, now)| now.rest == 0)
}
fn lotus(s: &Seen) -> bool {
    s.began(LOTUS)
}
fn executioner(s: &Seen) -> bool {
    s.began(EXECUTIONER)
}
fn dash(s: &Seen) -> bool {
    s.shadow()
        .is_some_and(|(was, now)| was.dash == 0 && now.dash > 0)
}
fn dash_aloft(s: &Seen) -> bool {
    dash(s) && !s.was().grounded
}
fn dash_jump(s: &Seen) -> bool {
    s.shadow()
        .is_some_and(|(was, now)| was.flip == 0 && now.flip > 0)
}
fn tumble_then_plunge(s: &Seen) -> bool {
    s.began_aloft(PLUNGE) && s.was().rise.used
}
fn three_marks(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    was.marks < 3 && now.marks >= 3
}
fn cashed(s: &Seen) -> bool {
    let (was, now) = s.dummy();
    was.marks >= 2
        && now.marks == 0
        && super::struck(was, now)
        && matches!(
            super::kind_of(s.now().action),
            Some(REAVE | PLUNGE | GALLOWS)
        )
}
fn cashed_five(s: &Seen) -> bool {
    cashed(s) && s.dummy().0.marks >= 5
}
fn lotus_dragged(s: &Seen) -> bool {
    s.before
        .effects
        .iter()
        .zip(s.after.effects.iter())
        .any(|(was, now)| match (was, now) {
            (Some(a), Some(b)) => {
                a.kind == EffectKind::GuillotineLotus
                    && b.kind == EffectKind::GuillotineLotus
                    && a.owner == 0
                    && b.owner == 0
                    && b.age > a.age.saturating_add(2)
            }
            _ => false,
        })
}
fn cut_short(s: &Seen) -> bool {
    matches!(s.was().action, Action::Recovery { kind, .. }
        if kind != SEND && kind != WHIRL && kind != RISING_WHIRL)
        && matches!(
            s.now().action,
            Action::Startup {
                kind: SEND | WHIRL,
                ..
            }
        )
}

pub const ROOMS: [Room; 10] = [
    Room {
        title: "The Shadow Reaver",
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
        title: "Left click: the finishers",
        advanced: false,
        intro: "Left click is her sword, and these three are the only moves that spend marks. \
                Each one is a different way in: a lunge along the floor, a dive from above, a \
                drive straight up.",
        tasks: &[
            Task {
                what: "Reave, on the floor",
                keys: "Left click, about four metres from the dummy",
                check: reave,
                lands: true,
            },
            Task {
                what: "Plunge, from the air",
                keys: "Space, then left click while looking down at the dummy",
                check: plunge,
                lands: true,
            },
            Task {
                what: "Gallows, a drive upward",
                keys: "Space + left click together (look up to drive at the crosshair)",
                check: gallows,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Middle click: thrown blades",
        advanced: false,
        intro: "Middle click throws. Every blade that lands marks the target, and the shadow \
                throws one too from wherever it is. In the air the same click is a spray of \
                blades, or her second jump.",
        tasks: &[
            Task {
                what: "Shuriken, on the floor",
                keys: "Middle click at the dummy",
                check: shuriken,
                lands: true,
            },
            Task {
                what: "Tumble, her double jump",
                keys: "Space, then in the air Space + middle click",
                check: tumble,
                lands: false,
            },
            Task {
                what: "Fan, in the air",
                keys: "Space, then middle click (it hangs you up for a moment)",
                check: fan,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Right click: the shadow",
        advanced: false,
        intro: "Right click sends the shadow to the spot under the crosshair -- only somewhere \
                you could walk to. Out there it copies your sword swings, and comes home on its \
                own after three seconds. Walk onto it to take it back with a burst.",
        tasks: &[
            Task {
                what: "Send the shadow",
                keys: "Right click with the crosshair on the floor",
                check: sent,
                lands: false,
            },
            Task {
                what: "Send it again to move it",
                keys: "Right click somewhere else while it is out",
                check: sent_again,
                lands: false,
            },
            Task {
                what: "Let it copy a swing",
                keys: "Send it beside the dummy, then left click",
                check: copied,
                lands: true,
            },
            Task {
                what: "Whirl, from the air",
                keys: "Space, then right click: the shadow flies out and spins",
                check: whirl,
                lands: true,
            },
            Task {
                what: "Rising whirl",
                keys: "Space + right click together",
                check: rising_whirl,
                lands: false,
            },
            Task {
                what: "Take it back by walking onto it",
                keys: "Send it, then walk into it",
                check: collected,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Q, E and the dash",
        advanced: false,
        intro: "Q opens twelve blades round wherever the shadow is. E is a huge slow overhead. \
                Shift toward the shadow -- crosshair on it -- dashes you to it across the room, \
                and takes it back with a burst.",
        tasks: &[
            Task {
                what: "Guillotine lotus",
                keys: "Send the shadow beside the dummy, then Q",
                check: lotus,
                lands: true,
            },
            Task {
                what: "Executioner",
                keys: "E, close to the dummy",
                check: executioner,
                lands: true,
            },
            Task {
                what: "Dash to the shadow",
                keys: "Send it, put the crosshair on it, then Shift + W",
                check: dash,
                lands: false,
            },
            Task {
                what: "Dash to it from the air",
                keys: "Send it, jump, crosshair on it, Shift + W",
                check: dash_aloft,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: the dash jump",
        advanced: true,
        intro: "Everything past here is advanced: ways to get the most from her. She plays well \
                without any of it. The dash jump is her longest move: jump in the last moment \
                of a dash to the shadow and she flips off it, keeping most of the dash's speed. \
                Send the shadow to the lip of the first shelf, dash, and press Space just as \
                you reach it -- a fresh press, not held from before.",
        tasks: &[Task {
            what: "Flip off the shadow",
            keys: "Right click, Shift + W at it, Space as you arrive",
            check: dash_jump,
            lands: false,
        }],
        dummy: Dummy::Aside,
        bay: Bay::Gap(150, 800),
    },
    Room {
        title: "Advanced: Tumble into Plunge",
        advanced: true,
        intro: "Her air pattern. Jump, Tumble for the second jump -- its blades mark whoever is \
                below -- then Plunge down onto the mark you just made, which cashes it.",
        tasks: &[Task {
            what: "Tumble, then Plunge, in one jump",
            keys: "Space; in the air Space + middle click; then left click looking down",
            check: tumble_then_plunge,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: marks, and cashing them",
        advanced: true,
        intro: "The pips over the dummy are your marks. Shadow blades put them on -- shuriken, \
                whirl, lotus, the burst -- and one fades every two seconds. A finisher spends \
                them all: two marks is half again the damage, five is four times it.",
        tasks: &[
            Task {
                what: "Put three marks on the dummy",
                keys: "Send the shadow beside it; shuriken, Q, whirl",
                check: three_marks,
                lands: false,
            },
            Task {
                what: "Cash two or more with a finisher",
                keys: "Left click (Reave, Plunge or Gallows) into it",
                check: cashed,
                lands: false,
            },
            Task {
                what: "Cash all five",
                keys: "Build to five marks, then finish",
                check: cashed_five,
                lands: false,
            },
        ],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: dragging the lotus",
        advanced: true,
        intro: "Open the lotus on a shadow that is already out, then send the shadow somewhere \
                else: all twelve blades are dragged after it through everything in between.",
        tasks: &[Task {
            what: "Drag the lotus",
            keys: "Right click out, Q, then right click past the dummy",
            check: lotus_dragged,
            lands: true,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
    Room {
        title: "Advanced: cutting a recovery short",
        advanced: true,
        intro: "Right click is never locked out by her own moves. Pressed while a swing is \
                still recovering, it ends the recovery and sends the shadow at once -- swing, \
                and reposition without waiting.",
        tasks: &[Task {
            what: "Send out of a recovery",
            keys: "E on the dummy, then right click before you can move again",
            check: cut_short,
            lands: false,
        }],
        dummy: Dummy::Stands,
        bay: Bay::Flat,
    },
];
