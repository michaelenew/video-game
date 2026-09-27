//! The Champion's weapons are drawn where the hit test is.
//!
//! `view::arms` draws the sword, the hammer and the spear as three things
//! unlike each other, and puts the one in her hands on the hit volume while a
//! cut is out. These hold it to that: the tip on the capsule's far end on
//! every active frame of the chain, the three silhouettes apart, and nothing
//! drawn on anybody else. See `docs/design/kits/champion.md`, "The weapons you
//! can see".

use sim::moves::champion as c;
use view::arms::{self, Torso};
use view::math::{self, Quat, V3};
use view::skeleton;

fn v(p: sim::V3) -> V3 {
    [
        p.x.to_f32_for_render(),
        p.y.to_f32_for_render(),
        p.z.to_f32_for_render(),
    ]
}

/// A torso stood upright where the body is, facing where it faces.
fn torso(p: &sim::state::Player) -> Torso {
    let at = v(p.pos);
    let turn = view::body_turn([
        p.facing.x.to_f32_for_render(),
        p.facing.z.to_f32_for_render(),
    ]);
    Torso {
        chest: math::add(at, [0.0, 1.3, 0.0]),
        chest_turn: turn,
        hips: math::add(at, [0.0, 0.95, 0.0]),
        hips_turn: turn,
    }
}

/// Where the clip puts the two hands on a frame, in the arena, for this body.
fn hands(clip: view::Clip, frame: u16, p: &sim::state::Player) -> (V3, V3) {
    let s = skeleton::skeleton_for(sim::Class::Champion);
    let skin = skeleton::solve(&s, &clip.at(frame as u32));
    let facing = [
        p.facing.x.to_f32_for_render(),
        p.facing.z.to_f32_for_render(),
    ];
    let pos = v(p.pos);
    (
        view::into_world(skin.origin[view::hand_joint(true).index()], pos, facing),
        view::into_world(skin.origin[view::hand_joint(false).index()], pos, facing),
    )
}

/// Throw one move out of reach of anybody, calling `each` on every active
/// frame with the fighter and the clip frame it is on.
fn throw(kind: u8, mut each: impl FnMut(&sim::state::Player, u16)) {
    let mut w = sim::World::with_classes([sim::Class::Champion, sim::Class::Bulwark]);
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    if let sim::class::Mechanic::Forms {
        chain, chain_left, ..
    } = &mut w.players[0].mechanic
    {
        *chain = c::link_of(kind).expect("a chain link");
        *chain_left = 400;
    }
    let button = match c::weapon(kind) {
        c::SWORD => sim::Input::LEFT,
        c::HAMMER => sim::Input::MIDDLE,
        _ => sim::Input::RIGHT,
    };
    let (startup, active, _) = sim::moves::frames(sim::Class::Champion, kind);
    let mut seen = 0;
    for _ in 0..120u16 {
        w.advance([sim::Input::aimed(button, 0), sim::Input::aimed(0, 1 << 15)]);
        if let sim::state::Action::Active { kind: k, left } = w.players[0].action {
            if k == kind {
                each(&w.players[0], startup + active.saturating_sub(left));
                seen += 1;
            }
        }
    }
    assert!(seen > 0, "move {kind} never went active");
}

const CHAIN: [u8; 9] = [
    c::SWORD_GROUND,
    c::BACKCUT,
    c::UPCUT,
    c::HAMMER_GROUND,
    c::UPROOT,
    c::EARTHBREAKER,
    c::SPEAR_GROUND,
    c::SKEWER,
    c::WHIRL,
];

#[test]
fn the_weapon_in_her_hands_ends_where_the_cut_does() {
    for kind in CHAIN {
        let m = sim::moves::get(sim::Class::Champion, kind);
        let clip = view::play::move_clip(sim::Class::Champion, kind);
        // A hammer is only drawn so much longer than itself. A volume that
        // reaches further than that from the hand -- the top of an overhead,
        // for a frame -- is pointed at and not reached, and the trail draws
        // the rest; more than one such frame in a move is a weapon drawn
        // somewhere the cut is not.
        let mut short = 0;
        throw(kind, |p, frame| {
            let hb = sim::state::hitbox(p).expect("an active frame has a volume");
            let (left, right) = hands(clip, frame, p);
            let a = arms::champion_arms(p, left, right, &torso(p)).expect("she is armed");
            assert_eq!(
                a.held,
                c::weapon(kind),
                "{}: the wrong weapon is out",
                m.name
            );
            let w = a.weapons[a.held as usize];
            assert!(w.held);
            let off = math::length(math::sub(w.tip, v(hb.to)));
            if off > 0.01 {
                short += 1;
                assert!(
                    short <= 1,
                    "{} frame {frame}: the drawn tip is {off:.2} m from the volume's end",
                    m.name
                );
            }
            // And the weapon runs back through her leading hand.
            let axis = w.axis();
            let to_hand = math::sub(right, w.butt);
            let off_haft = math::length(math::cross(axis, to_hand));
            assert!(
                off_haft < 0.01,
                "{} frame {frame}: the leading hand is {off_haft:.2} m off the weapon",
                m.name
            );
            let along = math::dot(axis, to_hand);
            assert!(
                along > 0.0 && along < w.length(),
                "{} frame {frame}: the leading hand is past the end of the weapon",
                m.name
            );
        });
    }
}

#[test]
fn the_weapon_never_points_anywhere_but_the_cut() {
    // Including the frames whose volume reaches past what the weapon can be
    // drawn at: those may fall short, but never aside.
    for kind in CHAIN {
        let clip = view::play::move_clip(sim::Class::Champion, kind);
        throw(kind, |p, frame| {
            let Some(hb) = sim::state::hitbox(p) else {
                return;
            };
            let (left, right) = hands(clip, frame, p);
            let a = arms::champion_arms(p, left, right, &torso(p)).expect("she is armed");
            let w = a.weapons[a.held as usize];
            let toward = math::sub(v(hb.to), right);
            let aside = math::length(math::cross(
                math::normalize_or(toward, [1.0, 0.0, 0.0]),
                math::sub(w.tip, right),
            ));
            assert!(
                aside < 0.01,
                "move {kind} frame {frame}: the tip is {aside:.2} m aside"
            );
        });
    }
}

#[test]
fn the_three_weapons_are_three_silhouettes() {
    let p = sim::state::Player::new(sim::Class::Champion);
    let (left, right) = ([0.2, 1.1, -0.05], [0.35, 1.25, -0.05]);
    let a = arms::champion_arms(&p, left, right, &torso(&p)).expect("she is armed");
    let length = |w: u8| a.weapons[w as usize].length();
    // The heaviest single piece of each: where the mass is.
    let heaviest = |w: u8| {
        arms::pieces(&a, w)
            .iter()
            .filter(|piece| piece.shown)
            .map(|piece| piece.size[0] * piece.size[2] * piece.size[1].min(0.6))
            .fold(0.0f32, f32::max)
    };
    let (sword, hammer, spear) = (length(c::SWORD), length(c::HAMMER), length(c::SPEAR));
    assert!(
        spear > 1.6 * sword && spear > 1.6 * hammer,
        "the spear ({spear:.2} m) is not by far the longest (sword {sword:.2}, hammer {hammer:.2})"
    );
    assert!(
        (sword - hammer).abs() > 0.15,
        "the sword ({sword:.2} m) and the hammer ({hammer:.2} m) are the same length"
    );
    assert!(
        heaviest(c::HAMMER) > 4.0 * heaviest(c::SWORD).max(heaviest(c::SPEAR)),
        "the hammer's head does not outweigh everything on the other two"
    );
    // All three are on her at once, and only one is in her hands.
    assert_eq!(a.weapons.iter().filter(|w| w.held).count(), 1);
    for w in [c::SWORD, c::HAMMER, c::SPEAR] {
        assert!(arms::pieces(&a, w).iter().any(|piece| piece.shown));
    }
}

#[test]
fn the_weapon_out_is_the_one_last_thrown() {
    let mut p = sim::state::Player::new(sim::Class::Champion);
    for (form, want) in [
        (sim::class::Form::Sword, c::SWORD),
        (sim::class::Form::Hammer, c::HAMMER),
        (sim::class::Form::Spear, c::SPEAR),
    ] {
        if let sim::class::Mechanic::Forms { form: f, .. } = &mut p.mechanic {
            *f = form;
        }
        assert_eq!(arms::in_hand(&p), Some(want));
    }
    // A running move overrides it.
    p.action = sim::state::Action::Startup {
        kind: c::HAMMER_GROUND,
        left: 3,
    };
    assert_eq!(arms::in_hand(&p), Some(c::HAMMER));
}

#[test]
fn the_trail_is_the_volume_and_fades() {
    let mut last = None;
    throw(c::SWORD_GROUND, |p, _| {
        let t = arms::trail(p);
        if let Some(t) = t {
            let hb = sim::state::hitbox(p).expect("a volume");
            assert!(math::length(math::sub(t.outer[t.count - 1], v(hb.to))) < 0.001);
            assert_eq!(t.fade, 1.0);
        }
        last = Some(*p);
    });
    let mut p = last.expect("the cut went out");
    let m = sim::moves::get(sim::Class::Champion, c::SWORD_GROUND);
    p.action = sim::state::Action::Recovery {
        kind: c::SWORD_GROUND,
        left: m.recovery - 2,
    };
    let ghost = arms::trail(&p).expect("the trail lingers");
    assert!(ghost.fade > 0.0 && ghost.fade < 1.0);
    p.action = sim::state::Action::Recovery {
        kind: c::SWORD_GROUND,
        left: 1,
    };
    assert!(arms::trail(&p).is_none());
    p.action = sim::state::Action::Free;
    assert!(arms::trail(&p).is_none());
}

#[test]
fn nobody_else_carries_the_champions_weapons() {
    let identity = Quat::from_y(0.0);
    let t = Torso {
        chest: [0.0; 3],
        chest_turn: identity,
        hips: [0.0; 3],
        hips_turn: identity,
    };
    for class in sim::class::ALL_CLASSES {
        if class == sim::Class::Champion {
            continue;
        }
        let p = sim::state::Player::new(class);
        assert!(
            arms::champion_arms(&p, [0.0; 3], [0.0; 3], &t).is_none(),
            "{} is drawn carrying the Champion's weapons",
            class.name()
        );
        assert!(arms::trail(&p).is_none());
    }
}
