//! The Champion's weapon is drawn where the hit test is, in the shape the
//! move calls for.
//!
//! `view::arms` draws one golden liquid that coils round her forearm between
//! moves and runs into a sword, a hammer or a spear in the middle of them,
//! three shapes unlike each other, and puts the weapon on the hit volume while
//! a cut is out. These hold it to that: whole and on the capsule's far end on
//! every active frame of the chain, a few frames to change shape, a few
//! seconds kept and a slow melt after, the three silhouettes apart, and
//! nothing drawn on anybody else. See `docs/design/kits/champion.md`, "The
//! weapon you can see".

use sim::moves::champion as c;
use view::arms::{self, Torso};
use view::math::{self, Quat, V3};
use view::skeleton;
use view::trail;

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
        shoulder: math::add(at, [-0.2, 1.45, 0.0]),
        elbow: math::add(at, [-0.25, 1.2, 0.05]),
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
    // Pressed once: a weapon held down is a charge (exploration 0015).
    for f in 0..120u16 {
        let press = if f == 0 { button } else { 0 };
        w.advance([sim::Input::aimed(press, 0), sim::Input::aimed(0, 1 << 15)]);
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
            assert!(
                a.whole(),
                "{} frame {frame}: the weapon is not whole on an active frame",
                m.name
            );
            let w = a.weapons[a.held as usize];
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

/// A Champion whose weapon was last called as `form`, `rest` frames after
/// the move that called it ended.
fn formed(form: sim::class::Form, rest: u16) -> sim::state::Player {
    let mut p = sim::state::Player::new(sim::Class::Champion);
    if let sim::class::Mechanic::Forms {
        form: f, liquid, ..
    } = &mut p.mechanic
    {
        *f = form;
        *liquid =
            sim::class::Quicksilver::new(rest.saturating_add(30), rest, None, u16::MAX, u16::MAX);
    }
    p
}

#[test]
fn the_three_weapons_are_three_silhouettes() {
    let (left, right) = ([0.2, 1.1, -0.05], [0.35, 1.25, -0.05]);
    let shape = |form, w: u8| {
        let p = formed(form, 0);
        let a = arms::champion_arms(&p, left, right, &torso(&p)).expect("she is armed");
        assert!(a.whole());
        assert_eq!(a.held, w);
        // One shape at a time: the others are not there at all.
        for other in [c::SWORD, c::HAMMER, c::SPEAR] {
            let shown = arms::pieces(&a, other).iter().any(|piece| piece.shown);
            assert_eq!(shown, other == w, "weapon {other} drawn while {w} is out");
        }
        assert!(
            a.beads.iter().all(|b| !b.shown),
            "drops left over a whole weapon"
        );
        let length = a.weapons[w as usize].length();
        // The heaviest single piece: where the mass is.
        let heaviest = arms::pieces(&a, w)
            .iter()
            .filter(|piece| piece.shown)
            .map(|piece| piece.size[0] * piece.size[2] * piece.size[1].min(0.6))
            .fold(0.0f32, f32::max);
        (length, heaviest)
    };
    use sim::class::Form;
    let (sword, sword_mass) = shape(Form::Sword, c::SWORD);
    let (hammer, hammer_mass) = shape(Form::Hammer, c::HAMMER);
    let (spear, spear_mass) = shape(Form::Spear, c::SPEAR);
    assert!(
        spear > 1.6 * sword && spear > 1.6 * hammer,
        "the spear ({spear:.2} m) is not by far the longest (sword {sword:.2}, hammer {hammer:.2})"
    );
    assert!(
        (sword - hammer).abs() > 0.15,
        "the sword ({sword:.2} m) and the hammer ({hammer:.2} m) are the same length"
    );
    assert!(
        hammer_mass > 4.0 * sword_mass.max(spear_mass),
        "the hammer's head does not outweigh everything on the other two"
    );
}

#[test]
fn between_moves_it_is_a_coil_round_her_arm() {
    let p = sim::state::Player::new(sim::Class::Champion);
    let t = torso(&p);
    let right = math::add(t.elbow, [0.0, -0.05, 0.27]);
    let a = arms::champion_arms(&p, [0.2, 1.1, -0.05], right, &t).expect("she is armed");
    assert_eq!(a.weight, [0.0; 3], "a fresh Champion holds a weapon");
    for w in [c::SWORD, c::HAMMER, c::SPEAR] {
        assert!(arms::pieces(&a, w).iter().all(|piece| !piece.shown));
    }
    // Every drop is on the arm: within a hand of the upper arm or the forearm,
    // and between the shoulder and the wrist.
    let off_bone = |from: V3, to: V3, at: V3| {
        let axis = math::normalize_or(math::sub(to, from), [0.0, 0.0, 1.0]);
        let reach = math::length(math::sub(to, from));
        let d = math::sub(at, from);
        let along = math::dot(d, axis).clamp(0.0, reach);
        math::length(math::sub(d, math::scale(axis, along)))
    };
    for b in &a.beads {
        assert!(b.shown);
        let off = off_bone(t.shoulder, t.elbow, b.centre).min(off_bone(t.elbow, right, b.centre));
        assert!(off < 0.1, "a drop {off:.2} m off the arm");
    }
    // It winds the whole arm, not a bracelet: drops on both bones.
    let near = |from, to| {
        a.beads
            .iter()
            .filter(|b| off_bone(from, to, b.centre) < 0.1)
            .count()
    };
    assert!(near(t.shoulder, t.elbow) >= 4 && near(t.elbow, right) >= 4);
    // The glow has nowhere to be but the hand.
    assert_eq!(a.head(), right);
}

#[test]
fn it_keeps_its_shape_for_seconds_then_melts_slowly() {
    use sim::class::Form;
    let weight = |rest| {
        let p = formed(Form::Hammer, rest);
        arms::liquid(&p).expect("a Champion").weights()[c::HAMMER as usize]
    };
    assert_eq!(weight(0), 1.0);
    // A few seconds: still a hammer, whole.
    assert_eq!(weight(arms::HOLD), 1.0);
    const { assert!(arms::HOLD >= 120, "kept for under two seconds") };
    // Then slowly: most of a second at least, and never all at once.
    const {
        assert!(
            arms::MELT >= 45,
            "melts in under three quarters of a second"
        )
    };
    let mut last = 1.0;
    for f in 1..=arms::MELT {
        let now = weight(arms::HOLD + f);
        assert!(now < last, "the melt stalled at frame {f}");
        assert!(
            last - now < 0.05,
            "the melt jumped {:.2} at frame {f}",
            last - now
        );
        last = now;
    }
    assert_eq!(last, 0.0);
}

/// Advance a Champion and a distant dummy one frame, pressing `button`.
fn press(w: &mut sim::World, button: u16) {
    w.advance([sim::Input::aimed(button, 0), sim::Input::aimed(0, 1 << 15)]);
}

#[test]
fn a_new_shape_takes_a_few_frames_and_is_whole_before_the_cut() {
    let mut w = sim::World::with_classes([sim::Class::Champion, sim::Class::Bulwark]);
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    // From the coil: a sword.
    press(&mut w, sim::Input::LEFT);
    let mut from_coil = 0;
    let mut first = None;
    for f in 0..200u16 {
        let p = w.players[0];
        let l = arms::liquid(&p).expect("a Champion");
        if let Some(kind) = p.action.attack_kind() {
            first.get_or_insert(kind);
        }
        if l.solid < 1.0 && p.action.attack_kind().is_some() {
            from_coil += 1;
        }
        if matches!(p.action, sim::state::Action::Active { .. }) {
            assert_eq!(l.solid, 1.0, "frame {f}: not whole on an active frame");
        }
        if p.action.attack_kind().is_none() && first.is_some() {
            break;
        }
        w.advance([sim::Input::aimed(0, 0), sim::Input::aimed(0, 1 << 15)]);
    }
    assert!(first.is_some(), "the sword never came out");
    assert!(
        from_coil as u16 <= arms::FORM_IN,
        "{from_coil} frames to run into a sword"
    );
    const {
        assert!(
            arms::FORM_IN <= 6,
            "a change of shape is more than a few frames"
        )
    };
    // While the sword is still whole: the hammer, which runs out of a sword.
    let p = w.players[0];
    assert_eq!(arms::liquid(&p).expect("a Champion").solid, 1.0);
    press(&mut w, sim::Input::MIDDLE);
    let mut changing = 0;
    for _ in 0..20 {
        let p = w.players[0];
        let l = arms::liquid(&p).expect("a Champion");
        if l.weapon == c::HAMMER && l.solid < 1.0 {
            changing += 1;
            assert_eq!(
                l.was,
                Some(c::SWORD),
                "the hammer ran out of something else"
            );
            assert_eq!(l.was_solid, 1.0);
            let weights = l.weights();
            let total: f32 = weights.iter().sum();
            assert!(
                (total - 1.0).abs() < 1e-4,
                "liquid lost in the change: {total}"
            );
        }
        w.advance([sim::Input::aimed(0, 0), sim::Input::aimed(0, 1 << 15)]);
    }
    assert!(changing as u16 <= arms::FORM_IN);
}

#[test]
fn the_timings_fit_the_clocks() {
    // The snapshot's clocks stop counting where they stop meaning anything:
    // past the end of a melt, and past the end of a change of shape.
    use sim::class::Quicksilver as Q;
    const { assert!(arms::HOLD + arms::MELT < Q::REST_MAX) };
    const { assert!(arms::FORM_IN < Q::FORGED_MAX) };
    let q = Q::new(3, 700, Some(sim::class::Form::Spear), 9, 2000);
    assert_eq!(
        (q.forged(), q.rest(), q.was(), q.was_forged(), q.was_rest()),
        (3, 700, Some(sim::class::Form::Spear), 9, Q::REST_MAX)
    );
}

#[test]
fn the_shape_is_the_snapshots() {
    // Drawn from the clocks in the snapshot and nothing of the renderer's own:
    // the same player draws the same liquid every time.
    let mut w = sim::World::with_classes([sim::Class::Champion, sim::Class::Bulwark]);
    press(&mut w, sim::Input::RIGHT);
    let p = w.players[0];
    let t = torso(&p);
    let a = arms::champion_arms(&p, [0.2, 1.1, -0.05], [0.35, 1.25, -0.05], &t);
    let b = arms::champion_arms(&p, [0.2, 1.1, -0.05], [0.35, 1.25, -0.05], &t);
    assert_eq!(a, b);
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
fn the_cut_glows_while_the_volume_is_out_and_fades_through_the_follow_through() {
    let mut last = None;
    throw(c::SWORD_GROUND, |p, _| {
        let cut = arms::cut(p, 0.0).expect("a cut is drawn while the volume is out");
        assert_eq!(cut.which, c::SWORD);
        // The first hit of the string is never quite solid; the rest are.
        let most = if cut.place == 0 { 0.7 } else { 1.0 };
        assert_eq!(cut.glow, most);
        last = Some(*p);
    });
    let mut p = last.expect("the cut went out");
    let m = sim::moves::get(sim::Class::Champion, c::SWORD_GROUND);
    p.action = sim::state::Action::Startup {
        kind: c::SWORD_GROUND,
        left: 1,
    };
    assert_eq!(
        arms::cut(&p, 0.0).map(|c| c.glow),
        Some(0.0),
        "the lead-in is dark"
    );
    p.action = sim::state::Action::Recovery {
        kind: c::SWORD_GROUND,
        left: m.recovery - 2,
    };
    let early = arms::cut(&p, 0.0).expect("the follow-through glows").glow;
    let later = arms::cut(&p, 0.5).expect("and between frames").glow;
    assert!(0.0 < later && later < early && early < 1.0);
    p.action = sim::state::Action::Recovery {
        kind: c::SWORD_GROUND,
        left: 1,
    };
    assert!(arms::cut(&p, 0.0).is_none(), "gone before she is free");
    p.action = sim::state::Action::Free;
    assert!(arms::cut(&p, 0.0).is_none());
}

/// The rule the arc keeps: **its head is the blade**. Whatever the path
/// behind, the last column is where the weapon is drawn this picture.
#[test]
fn the_arc_ends_on_the_blade_and_wears_away_behind_it() {
    let mut path = trail::Path::default();
    let mut prev = None;
    let mut frame = 0.0;
    throw(c::SWORD_GROUND, |p, _| {
        let before = prev.unwrap_or(*p);
        let t = torso(p);
        let (left, right) = ([0.2, 1.1, -0.05], [0.35, 1.25, -0.05]);
        for alpha in [0.0, 0.25, 0.5, 0.75] {
            let a = arms::champion_arms_between(&before, p, alpha, left, right, &t)
                .expect("she is armed");
            let cut = arms::cut(p, alpha).expect("a cut");
            let now = frame + alpha;
            path.push(arms::sample(&a, &cut, now), arms::WINDOW);
            let r = trail::ribbon(&path, now, arms::WINDOW);
            if path.samples().len() < 2 {
                continue;
            }
            let head = r.count - 1;
            let blade = a.weapons[c::SWORD as usize].tip;
            assert!(
                math::length(math::sub(r.outer[head], blade)) < 1e-4,
                "the head is the tip"
            );
            assert_eq!(r.life[head], 1.0);
            assert!(r.life[..r.count].windows(2).all(|w| w[0] <= w[1] + 1e-6));
            assert!(r.count <= trail::RIBBON);
        }
        frame += 1.0;
        prev = Some(*p);
    });
    // Long after, only the window's worth is kept.
    let last = *path.samples().last().expect("a path");
    for k in 1..40 {
        path.push(
            trail::Sample {
                at: last.at + k as f32 * 0.5,
                ..last
            },
            arms::WINDOW,
        );
    }
    let s = path.samples();
    assert!(s.last().unwrap().at - s[1].at < arms::WINDOW);
    // And a picture back in time forgets what came after it.
    let back = s[2].at;
    path.push(trail::Sample { at: back, ..last }, arms::WINDOW);
    assert!(path.samples().iter().all(|x| x.at <= back));
}

/// Between two frames of a cut the sword turns from where it pointed to
/// where it points, about the hand, keeping its length: no tick round in
/// three jumps, and no shortening as it sweeps.
#[test]
fn the_sword_sweeps_between_frames() {
    let mut prev: Option<sim::state::Player> = None;
    let (left, right) = ([0.2, 1.1, -0.05], [0.35, 1.25, -0.05]);
    throw(c::SWORD_GROUND, |p, _| {
        let Some(before) = prev else {
            prev = Some(*p);
            return;
        };
        let t = torso(p);
        let tip = |alpha: f32| {
            arms::champion_arms_between(&before, p, alpha, left, right, &t)
                .expect("armed")
                .weapons[c::SWORD as usize]
                .tip
        };
        let now = arms::champion_arms(p, left, right, &t)
            .expect("armed")
            .weapons[c::SWORD as usize]
            .tip;
        assert!(math::length(math::sub(tip(1.0), now)) < 1e-5);
        let (a, b, mid) = (tip(0.0), tip(1.0), tip(0.5));
        let reach = |x: V3| math::length(math::sub(x, right));
        assert!((reach(mid) - (reach(a) + reach(b)) * 0.5).abs() < 1e-3);
        if math::length(math::sub(a, b)) > 0.05 {
            let off = |x: V3| math::length(math::sub(x, a)) / math::length(math::sub(b, a));
            assert!(off(mid) > 0.2 && off(mid) < 0.8, "half way is between");
        }
        prev = Some(*p);
    });
}

#[test]
fn nobody_else_carries_the_champions_weapons() {
    let identity = Quat::from_y(0.0);
    let t = Torso {
        chest: [0.0; 3],
        chest_turn: identity,
        hips: [0.0; 3],
        hips_turn: identity,
        shoulder: [0.0; 3],
        elbow: [0.0; 3],
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
        assert!(arms::cut(&p, 0.0).is_none());
    }
}
