//! The prototype.
//!
//! Bevy is the renderer and nothing else. It owns no gameplay state: every
//! frame this reads the simulation, interpolates, and writes transforms. That
//! separation is what keeps rollback possible, and it is why the engine choice
//! stays reversible.
//!
//! Controls, per `docs/design/controls.md`:
//!   WASD move · Space jump · Shift+direction dodge (airdodge once per jump)
//!   J bash · Shift+J slam · K guard · L shield throw/recall · Shift+L grapple
//!   1-4 dummy mode · 5-7 sparring bot · F1 debug overlay · P pause · ] step one frame · R reset
//!   H hunt the Ridgeback, or fight the other player
//!
//! Player two: arrows, RCtrl, Period, Comma, Slash, RShift.
//!
//! Pick classes with `--p1 <class> --p2 <class>` (and `--p3`, `--p4` with
//! `--players`), or cycle player one's class
//! in-game with Tab. Names are matched loosely: bulwark, champion, reaver,
//! elementalist, blood, dual.

mod arenas;
mod bag;
mod bake;
mod beast;
mod blades;
mod brushwork;
mod cairns;
mod claimed;
mod coat;
mod critters;
mod crosshair;
mod debug;
mod falls;
mod forms;
mod fx;
mod glint;
mod ground;
mod grove;
mod grown;
mod heel;
mod hub;
mod hud;
mod impact;
mod ink;
mod inspect;
mod keep;
mod land;
mod menu;
mod models;
mod online;
mod outline;
mod paint;
mod palette;
mod picker;
mod platform;
mod profile;
mod scrape;
mod settings;
mod shapes;
mod signs;
mod sky;
mod slash;
mod sound;
mod species;
mod splats;
mod squeeze;
mod still;
mod stream;
mod swing;
mod town;
mod trophies;
mod valley;
mod veil;
mod warm;
mod wash;
mod window;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use online::Driver;
use sim::state::MAX_PLAYERS;
use sim::{Input as SimInput, World};
use view::interp::TickClock;
use view::play::{Crossfade, PoseInput};
use view::skeleton::{JOINTS, Joint, Skeleton, skeleton_for};
use view::{CameraRig, aim_from_radians, camera::RigConfig, interpolate, pitch_from_radians};

/// Loose class-name matching, so `--p1 reaver` works without remembering the
/// full name. Only the classes on offer (`sim::class::ROTATION`): `--p1
/// bulwark` names nobody, and the seat is the Champion.
fn parse_class(name: &str) -> Option<sim::Class> {
    use sim::Class::*;
    let n = name.to_lowercase();
    ALL.iter()
        .copied()
        .find(|c| {
            c.name().to_lowercase().replace(' ', "").starts_with(&n) || matches(n.as_str(), *c)
        })
        .or(match n.as_str() {
            "reaver" | "shadow" => Some(ShadowReaver),
            "blood" => Some(BloodMage),
            "dual" => Some(DualMage),
            _ => None,
        })
}

const ALL: [sim::Class; 5] = sim::class::ROTATION;

fn matches(n: &str, c: sim::Class) -> bool {
    c.name().to_lowercase().contains(n) && !n.is_empty()
}

/// What this run starts in: `--hunt [creature]` and `--arena <name>`.
///
/// Flags as well as keys, because the headless screenshot script takes flags
/// and not keystrokes. See [`picker`].
fn chosen_start(opts: &platform::Options) -> picker::Start {
    let mut start = picker::start(
        opts.flag("--hunt"),
        opts.value("--hunt"),
        opts.value("--arena"),
        opts.value("--temper"),
    );
    // **The valley is where a run starts** (`docs/design/valley.md`): Hearth's
    // square, unless a fight was asked for -- `--hunt`, `--arena`, or
    // `--versus` for the proving ground as it always was.
    if start.hunt.is_none() && start.arena.is_none() && !opts.flag("--versus") {
        start.arena = Some(sim::valley::START);
    }
    start
}

/// `--open`: every waystone in the valley lit, as if every creature had been
/// beaten. For walking the whole valley without the fights.
fn open_valley() -> bool {
    platform::flag("--open")
}

/// `--temper` was given: every temper is on offer to `T`, earned or not.
fn any_temper() -> bool {
    platform::value("--temper").is_some() || platform::flag("--temper")
}

/// A hunt, with only the people who are actually hunting in it.
///
/// **Player two is out of it unless a person has their keys.** The training
/// dummy is a body standing in the arena, and the creature attends to whoever
/// is nearest -- so a player who backed off further than the dummy stood saw
/// the animal turn round and walk away from them to stand over a fighter doing
/// nothing. `cargo run -p hunt --bin fight` has always taken absent hunters
/// out for the same reason; the game did not, which is why the harness never
/// saw it. Pressing `4` and then `R` brings the second hunter in.
fn seated(w: World, dummy: Dummy) -> World {
    w.seated(seats_of(dummy))
}

/// How many seats a dummy mode plays: both when a person has the second
/// keys, one otherwise. The simulation keeps the count (`World::seats`) so a
/// fresh fight built inside a tick seats itself.
fn seats_of(dummy: Dummy) -> u8 {
    if dummy == Dummy::Human { 2 } else { 1 }
}

/// Every seat's class: `--p1` to `--p4`, the first of the rotation (the
/// Champion) where none is named.
fn chosen_classes(opts: &platform::Options) -> [sim::Class; MAX_PLAYERS] {
    std::array::from_fn(|i| {
        opts.value(&format!("--p{}", i + 1))
            .and_then(parse_class)
            .unwrap_or(sim::class::ROTATION[0])
    })
}

/// **How many play** (`--players`, `?players=` in a link): two to four. More
/// than two is a room of friends on other machines -- one keyboard drives two
/// seats -- and every one of them is seated from the first frame.
pub fn chosen_players(opts: &platform::Options) -> u8 {
    opts.value("--players")
        .and_then(|n| n.parse::<u8>().ok())
        .unwrap_or(2)
        .clamp(2, MAX_PLAYERS as u8)
}

/// The world a set of settings starts in: the run's own at startup, or a
/// room's link when one is made or joined from the Esc menu. One function for
/// both, which is what makes the world two clients build from one link the
/// same world.
fn start_world(opts: &platform::Options, dummy: Dummy) -> World {
    // `--range <name>`: walked into one of the valley's creature ranges, its
    // hunt begun and its egg lying behind it (`sim::keep`).
    if let Some(r) = opts.value("--range").and_then(|name| {
        let wanted = name.to_lowercase().replace(['-', ' ', '\''], "_");
        sim::valley::ranges::RANGES
            .iter()
            .find(|r| r.name.to_lowercase().replace(['-', ' ', '\''], "_") == wanted)
    }) {
        let w = World::arrive(
            chosen_classes(opts),
            sim::valley::Journey::default(),
            r.arena,
            Some(0),
            2,
        );
        return seated(w, dummy);
    }
    let w = picker::world(chosen_start(opts), chosen_classes(opts));
    match chosen_players(opts) {
        n if n > 2 => w.seated(n),
        _ => seated(w, dummy),
    }
}

fn main() {
    // `--help` before anything else, so asking what the flags are does not
    // require a window, a GPU, or the patience to wait for Bevy to start.
    if platform::flag("--help") || platform::flag("-h") {
        print!("{}", manual::render());
        return;
    }
    platform::report_panics();
    platform::one_move_an_event();
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(window()),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    synchronous_pipeline_compilation: platform::flag("--sync-pipelines"),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: platform::asset_root(),
                    // No settings file beside a model: asked for, it is a
                    // request a page's host answers with a 404.
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.08)))
        .init_resource::<Sim>()
        .init_resource::<arenas::Drawn>()
        .init_resource::<stream::Stream>()
        .init_resource::<window::Windows>()
        .init_resource::<Rig>()
        .init_resource::<debug::ShowDebug>()
        .init_resource::<Look>()
        .init_resource::<palette::Palette>()
        .init_resource::<hub::Hub>()
        .init_resource::<Fades>()
        .init_resource::<ShadowFades>()
        .init_resource::<CopyLooks>()
        .init_resource::<fx::Flowing>()
        .init_resource::<ShieldHands>()
        .init_resource::<Hands>()
        .init_resource::<Torsos>()
        .init_resource::<palette::UiFocus>()
        .init_resource::<hud::ShowClassButtons>()
        .init_resource::<menu::Menu>()
        .init_resource::<keep::Bag>()
        .init_resource::<keep::Melt>()
        .init_resource::<keep::Puffs>()
        .init_resource::<scrape::Dust>()
        .init_resource::<bag::Keeping>()
        .init_resource::<heel::Heels>()
        .init_resource::<sound::Soundscape>()
        .init_resource::<Joints>()
        .add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .insert_resource(settings::Settings::load())
        .insert_resource(trophies::Trophies::load())
        .insert_resource(cairns::Cairns::load())
        .init_resource::<InsideOwnHead>()
        .init_resource::<inspect::Inspect>()
        .init_resource::<beast::Boom>()
        .init_resource::<Scripted>()
        .init_resource::<Sparring>()
        .add_plugins(MaterialPlugin::<beast::MarkMaterial>::default())
        .add_plugins(paint::PaintPlugin)
        .add_plugins(still::StillPlugin)
        .add_plugins(splats::SplatsPlugin)
        .add_plugins(swing::SwingPlugin)
        .add_plugins(warm::WarmPlugin)
        .add_plugins(ink::InkPlugin)
        .add_plugins(brushwork::BrushworkPlugin)
        .add_plugins(models::ModelsPlugin)
        .add_plugins(impact::ImpactPlugin)
        // The line round every silhouette, as a pass over the finished picture.
        // Only where the platform can read a depth buffer: see
        // `platform::draws_outlines`. Registering it anyway would not fail
        // quietly -- the pipeline is built whether or not a camera asks for it,
        // and on WebGL2 building it is the panic.
        .add_plugins(outline::OutlinePlugin.only_if(platform::draws_outlines()))
        // The watercolour's paper half, round the line (`wash`, `look::wash`).
        .add_plugins(wash::WashPlugin {
            outlines: platform::draws_outlines(),
        })
        .add_systems(
            PostUpdate,
            wash::follow.after(TransformSystem::TransformPropagate),
        )
        .add_plugins(profile::ProfilePlugin)
        .add_systems(
            Startup,
            (
                setup,
                beast::setup,
                beast::setup_bodies,
                beast::setup_signs,
                critters::setup,
                signs::setup,
                ground::setup,
                claimed::setup,
                hud::setup,
                hud::setup_regions,
                hud::setup_picker,
                valley::setup_text,
                crosshair::setup,
                glint::setup,
                veil::setup,
                keep::setup,
                scrape::setup,
            ),
        )
        .add_systems(
            Update,
            (
                // Who owns the mouse and keyboard this frame, before anything
                // reads them.
                palette::sample_focus,
                hud::sample_button_focus,
                // Mouse look runs next: aim is an input to the tick, not a
                // decoration applied after it.
                mouse_look,
                // The arena is drawn after the tick, and the dome rides the
                // camera afterwards: it is 480 m across, and a jump course is
                // longer than that, so a dome left at the origin is a sphere
                // the player can walk out of.
                (
                    tick_sim,
                    same_frame,
                    arenas::dress,
                    sky::weather,
                    stream::stream,
                    valley::update,
                    falls::update,
                    sky::follow,
                    sound::play,
                    sound::setup,
                )
                    .chain(),
                // One entry, so the chain stays under twenty: the balls go
                // where the poses just put the joints.
                // And the dust off a wall goes where the poses put the hand.
                (
                    apply_poses,
                    pose_balls,
                    place_copies,
                    pose_models,
                    swing_coats,
                    scrape::place,
                )
                    .chain(),
                place_shields,
                // Grouped because Bevy's chained tuple holds twenty systems
                // and this is the twenty-first. They are independent of each
                // other anyway: each puts one pool of meshes where the
                // simulation says its things are.
                (
                    place_structures,
                    place_beams,
                    place_bolts,
                    place_debris,
                    place_gusts,
                    place_discs,
                    fx::flow,
                    place_wingspans,
                    place_marks,
                    place_scythes,
                    place_champion_arms,
                    // The trails' emitters (`swing.rs`); the trails are laid
                    // once the creatures' are in too, below.
                    (
                        swing::emit_champions,
                        swing::emit_scythes,
                        swing::emit_volume_swings,
                        place_effects,
                        place_spells,
                    )
                        .chain(),
                    place_champion_glow,
                    place_third_flare,
                    place_pips,
                    place_curses,
                    claimed::drape,
                    place_auras,
                ),
                (beast::place, beast::paint_parts, beast::pose_bodies).chain(),
                drive_camera,
                fade_own_body,
                hud::toggle_class_buttons,
                hud::class_buttons,
                (hud::update, hud::update_heart),
                hud::update_class_buttons,
                crosshair::update,
                debug::draw,
                beast::overlay,
                // The creatures' volleys are the last thing to say where a
                // trail is; then the trails.
                (beast::signs, swing::place).chain(),
                palette::toggle,
                palette::draw,
            )
                .chain(),
        )
        // The small bodies, after the creatures: placed from the snapshot,
        // and their overlay with the creature's. Their own tuple because the
        // one above is full.
        .add_systems(
            Update,
            (
                critters::place,
                critters::overlay,
                signs::place,
                ground::place,
                ground::overlay,
                hud::update_picker,
                hud::update_momentum,
                hud::update_tempo,
                valley::update_text,
                glint::update,
                veil::place,
                online::announce,
                hud::update_online,
                hud::update_seats,
                hud::update_rival,
                hud::update_regions,
                hide_empty_seats,
                menu::show_progress,
                (menu::notice_cairns, menu::draw).chain(),
                // What you carry and your morph (`sim::keep`).
                (
                    heel::follow,
                    keep::place_morphs,
                    keep::place_blobs,
                    keep::place_loose,
                    keep::update_text,
                    keep::draw_bag,
                ),
            )
                .chain()
                .after(beast::signs)
                .before(palette::toggle),
        )
        // The window through what the camera passes through, on to where
        // the camera has just been put.
        .add_systems(Update, window::follow.after(drive_camera))
        // The close look's orbit, read after the fight's look has been and
        // before the camera is placed from it.
        .add_systems(
            Update,
            inspect::steer.after(mouse_look).before(drive_camera),
        )
        // The bag's key, before anything asks who owns the mouse: opening
        // it is what frees the cursor.
        .add_systems(Update, keep::toggle_bag.before(palette::sample_focus))
        // A second tuple only because Bevy's is full: the hub draws last, over
        // everything, and after the poses it is previewing have been placed.
        .add_systems(
            Update,
            (
                hub::toggle,
                hub::advance,
                hub::collect_bake,
                hub::onion_skin,
                hub::draw,
            )
                .chain()
                .after(palette::draw),
        )
        .run();
}

/// The window the arena is drawn in.
///
/// In the browser there is no window to make: there is a canvas already on the
/// page, and the game has to be told which one and told to keep the browser's
/// own shortcuts off the keys it uses — Space scrolls a page, and a player who
/// jumps should not find the page has jumped instead.
fn window() -> Window {
    Window {
        title: "Arena — prototype".into(),
        resolution: (1280.0_f32, 760.0_f32).into(),
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#arena".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        #[cfg(target_arch = "wasm32")]
        prevent_default_event_handling: true,
        ..default()
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// The simulation, plus the previous snapshot so the renderer can interpolate.
#[derive(Resource)]
pub struct Sim {
    pub prev: World,
    pub cur: World,
    pub clock: TickClock,
    paused: bool,
    step_once: bool,
    /// `SHOT_FRAME=N` runs to frame N and stops. Makes a screenshot land on an
    /// exact simulation frame instead of wherever `sleep` happened to leave it,
    /// which is what turns "does this look right" into a repeatable comparison.
    stop_at: Option<u32>,
    dummy: Dummy,
    /// **The dummy brought into a fight that would leave it out** -- a hunt,
    /// the valley's peaceful places -- from the Esc menu (`menu::dummy`). A
    /// duel has it whatever this says ([`World::present`]).
    with_dummy: bool,
    driver: Driver,
    /// Show the skeleton at rest instead of animating it.
    ///
    /// Useful rather than decorative: it is how you tell "this clip is wrong"
    /// from "this rig is wrong" while looking at the thing, in one keypress.
    bind_pose: bool,
    /// The last few seconds of simulation, so `[` can step **backwards**.
    ///
    /// Stepping forward on its own is not enough to study anything that
    /// happens on one frame, and the game has at least one of those: the
    /// Elementalist's structure jump chains because a stone's top catches her
    /// feet by three and a half centimetres, on exactly one frame of the rise.
    /// Watching that means going past it and coming back, repeatedly.
    ///
    /// A ring rather than a growing log, written in place, so a frame of
    /// stepping never allocates -- a `World` is a small flat copy by
    /// construction (`sim/tests/budget.rs`) and the renderer already clones one
    /// per tick for the interpolator.
    history: Rewind,
    /// The scripted double structure jump, if one is playing. See
    /// [`rehearsal`].
    rehearsing: Option<u32>,
    /// A trip the picker has asked for and the next tick will carry:
    /// [`picker`]. On the wire rather than a fresh `World` built here, so a
    /// peer changes arena on the same frame.
    travel: sim::input::Travel,
    /// **The fight so far, as a replay** (`sim::replay`): where it started
    /// and every frame's inputs, kept from the first frame so that "that felt
    /// wrong" can be saved after the fact (`Y`, or the Esc menu) and put
    /// through the harness. Begun again whenever the world is replaced
    /// outside a tick -- a match starting, the rehearsal -- and cut back when
    /// `[` rewinds, so it always describes exactly the world on screen.
    tape: sim::replay::Tape,
    /// A replay being played back in the game (`--replay <file>`): the
    /// inputs come off it instead of the keyboard, for both seats, and the
    /// camera follows the look it recorded. `None` once it runs out.
    playing: Option<Playback>,
    /// Where the last save went, for the menu.
    pub saved: Option<String>,
    /// `--record`: save a replay on its own at the end of every round.
    record: bool,
    /// The round whose end was saved, so `--record` saves each once.
    round_saved: bool,
    /// **The region books** (`docs/design/regions.md`): the tape of recent
    /// frames, the live regions' checksums, the neighbours' heartbeats and
    /// the gate, kept on every confirmed frame by both drivers; the watchdog
    /// with them in dev mode.
    pub ledger: regions::Ledger,
    /// How many of the watchdog's findings have been said aloud.
    regions_said: u32,
}

/// A tape on its way through the game. See [`Sim::playing`].
struct Playback {
    tape: sim::replay::Tape,
    /// The next frame to play.
    at: usize,
}

/// A ring of past snapshots, and how far back through it we have stepped.
struct Rewind {
    /// Oldest to newest, written at `next`, wrapping.
    ring: Vec<World>,
    next: usize,
    /// How many of `ring` hold a real frame -- it is short at startup.
    filled: usize,
}

/// Rather over two seconds at 60 Hz. A `World` is about 2 KB, so the whole
/// thing is a third of a megabyte and is allocated once.
const REWIND_FRAMES: usize = 150;

impl Rewind {
    fn new(seed: &World) -> Rewind {
        Rewind {
            ring: vec![seed.clone(); REWIND_FRAMES],
            next: 0,
            filled: 0,
        }
    }

    /// Remember the state *before* a tick, so stepping back lands on it.
    fn push(&mut self, w: &World) {
        self.ring[self.next].clone_from(w);
        self.next = (self.next + 1) % REWIND_FRAMES;
        self.filled = (self.filled + 1).min(REWIND_FRAMES);
    }

    /// Take the newest frame back off, or `None` once the ring runs dry.
    fn pop(&mut self) -> Option<World> {
        if self.filled == 0 {
            return None;
        }
        self.next = (self.next + REWIND_FRAMES - 1) % REWIND_FRAMES;
        self.filled -= 1;
        Some(self.ring[self.next].clone())
    }

    fn clear(&mut self) {
        self.filled = 0;
        self.next = 0;
    }
}

/// Where each fighter's shield hand ended up this frame, in world space.
///
/// The shield is a separate object because its position is independent of the
/// character -- that is the whole mechanic -- but while it is *in hand* it
/// should be in a hand, and a hand is now a thing the skeleton has. Written by
/// the posing pass and read by the one that places shields, which runs after.
#[derive(Resource, Default)]
struct ShieldHands([(Vec3, Quat); MAX_PLAYERS]);

/// Where both of each fighter's hands are drawn, in the arena: left, right.
/// Written by the posing pass and read by the one that places the Blood
/// mage's scythe, which rides the hands the clips put on its haft.
#[derive(Resource, Default)]
struct Hands([[Vec3; 2]; MAX_PLAYERS]);

/// Where each fighter's chest and hips are drawn, in the arena: a point and a
/// rotation each; and the shoulder and elbow of the arm that holds a weapon.
/// Written by the posing pass and read by the one that winds the Champion's
/// liquid weapon up her arm, so it bends and turns with the arm rather than
/// floating.
#[derive(Resource, Default)]
struct Torsos([[(Vec3, Quat); 2]; MAX_PLAYERS], [[Vec3; 2]; MAX_PLAYERS]);

/// One cross-fade per fighter. Renderer-local: a rollback rewinds it to
/// whatever it was, which is wrong by a few frames of blend weight and
/// invisible. See `view::play::Crossfade`.
#[derive(Resource, Default)]
struct Fades([Crossfade; MAX_PLAYERS]);

/// The same, for each fighter's shadow. Its own, because the two bodies are
/// rarely doing the same thing: hers cuts and his copies it four frames later,
/// and one fade shared between them would blur whichever was second.
#[derive(Resource, Default)]
struct ShadowFades([Crossfade; MAX_PLAYERS]);

/// Seat player two for the next fresh fight from what this machine has
/// asked for: a person on the second keys, or the dummy brought in from the
/// Esc menu. Whether it is in is the simulation's to know, and the tape's:
/// see `World::seats`.
fn reseat(sim: &mut Sim) {
    let seats = if sim.with_dummy {
        2
    } else {
        seats_of(sim.dummy)
    };
    if seats != sim.cur.seats {
        let frame = sim.cur.frame;
        sim.tape.note(frame, sim::replay::Event::Seats(seats));
        sim.cur.seats = seats;
    }
}

/// **The Esc menu's dummy** (`menu::dummy`): bring it in or send it away,
/// change what it does, or its class. Training only, as the dummy keys are.
/// In or out restarts the fight, since a fighter is placed as a fight is
/// built; a class change restarts it anyway, as Tab does; what it does
/// changes on the spot.
fn change_dummy(sim: &mut Sim, mode: Dummy, with: bool, class: sim::Class) {
    if sim.driver.online() {
        return;
    }
    sim.dummy = mode;
    let moved = with != sim.with_dummy;
    sim.with_dummy = with;
    reseat(sim);
    if class != sim.cur.players[1].class {
        sim.travel = sim::input::Travel::class(1, class);
    } else if moved {
        sim.travel = sim::input::Travel::RESTART;
    }
}

/// Training-mode opponent. Player two is a scripted dummy until someone takes
/// the second set of keys -- or until a sparring bot does.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dummy {
    Idle,
    Block,
    Attack,
    Human,
    /// `hunt::Duelist`, at one of its five levels (`hunt::Level`). Keys 5, 6
    /// and 7 are easy, normal and hard; the Esc menu has all five, and
    /// `--bot <level>` starts with one.
    Bot(hunt::Level),
}

/// Which opponent a run starts with: `--bot beginner|easy|normal|hard|expert`
/// (`?bot=hard` in a browser), or the dummy standing still.
fn starting_dummy() -> Dummy {
    let Some(name) = platform::value("--bot") else {
        return Dummy::Idle;
    };
    let name = name.trim().to_lowercase();
    let level = hunt::Level::ALL
        .into_iter()
        .find(|l| l.name() == name)
        .unwrap_or(hunt::Level::Normal);
    Dummy::Bot(level)
}

/// The sparring bot, when player two is one.
///
/// Renderer-side for the same reason [`Scripted`] is: it produces inputs, and
/// inputs are what cross the wire. It is rebuilt whenever the level changes,
/// with a fresh seed, so the personality it plays with is a new one each time
/// -- the whole point of it is not to be the same opponent twice.
#[derive(Resource, Default)]
struct Sparring(Option<hunt::Duelist>);

impl Default for Sim {
    fn default() -> Self {
        // Against a person, player two is that person: alive in a hunt, and on
        // nobody's script.
        let opts = platform::options();
        let dummy = if online::wanted(opts) {
            Dummy::Human
        } else {
            starting_dummy()
        };
        let mut w = start_world(opts, dummy);
        shot_bars(&mut w);
        shot_weight(&mut w);
        shot_momentum(&mut w);
        shot_form(&mut w);
        shot_at(&mut w);
        shot_move(&mut w);
        shot_climb(&mut w);
        shot_blood(&mut w);
        shot_reap(&mut w);
        shot_chase(&mut w);
        // `--replay <file>`: the fight is the tape's, from its own start.
        let playing = platform::value("--replay").and_then(|path| {
            let text = platform::load_replay(path)?;
            match sim::replay::Tape::from_text(&text).and_then(|t| t.world().map(|w| (t, w))) {
                Ok((tape, start)) => {
                    platform::log(&format!(
                        "replaying {path}: {} frames of {}",
                        tape.len(),
                        tape.start.to_line()
                    ));
                    w = start;
                    Some(Playback { tape, at: 0 })
                }
                Err(e) => {
                    platform::log(&format!("not replaying {path}: {e}"));
                    None
                }
            }
        });
        let seed = w.clone();
        let driver = online::start(opts, &w);
        Sim {
            prev: w.clone(),
            cur: w,
            clock: TickClock::new(),
            paused: false,
            step_once: false,
            dummy,
            with_dummy: false,
            driver,
            // `BIND_POSE=1` starts frozen, so the proportions of a build can be
            // captured without a keypress.
            bind_pose: platform::env("BIND_POSE").as_deref() == Some("1"),
            stop_at: env_num("SHOT_FRAME"),
            history: Rewind::new(&seed),
            ledger: regions::Ledger::new(&seed, 0, dev_mode()),
            regions_said: 0,
            rehearsing: None,
            travel: sim::input::Travel::NONE,
            tape: sim::replay::Tape::begin(&seed, build()),
            playing,
            saved: None,
            record: platform::flag("--record"),
            round_saved: false,
        }
    }
}

/// The unit form a fighter's part is drawn as, scaled per frame to its box
/// (`skeleton::Build::box_of`). A rounded box rather than a cube: the box is
/// still the box, so nothing about the pose or the hit test moves, but a limb
/// has no edge to catch the light as a line and the head is nearly a ball.
/// See `docs/design/forms.md`.
fn limb_form(joint: view::skeleton::Joint) -> Mesh {
    use view::skeleton::Joint;
    match joint {
        // Round about the bone, blunt at the ends, meeting the next limb in a
        // ball at the joint (`BodyBall`).
        Joint::ArmL
        | Joint::ArmR
        | Joint::ForearmL
        | Joint::ForearmR
        | Joint::ThighL
        | Joint::ThighR
        | Joint::ShinL
        | Joint::ShinR
        | Joint::Neck => shapes::soft_cylinder(Vec3::ONE, 0.5, 10),
        Joint::Head => shapes::soft_box(Vec3::ONE, 0.9, 12, None),
        // Inside the chest and the foot: a mannequin looks as it did before
        // they had joints.
        Joint::ClavicleL | Joint::ClavicleR | Joint::ToeL | Joint::ToeR => {
            shapes::soft_box(Vec3::ONE, 0.5, 6, None)
        }
        Joint::HandL | Joint::HandR | Joint::FootL | Joint::FootR => {
            shapes::soft_box(Vec3::ONE, 0.5, 8, None)
        }
        Joint::Root | Joint::Spine | Joint::Chest => shapes::soft_box(Vec3::ONE, 0.35, 10, None),
    }
}

/// The joints a limb bends at, each drawn as a ball the limb's own width so
/// the bend shows a knee rather than a gap: shoulder, elbow, wrist, hip, knee,
/// ankle. The torso's joints are inside the torso and need none.
const BALLS: [view::skeleton::Joint; 12] = {
    use view::skeleton::Joint;
    [
        Joint::ArmL,
        Joint::ForearmL,
        Joint::HandL,
        Joint::ArmR,
        Joint::ForearmR,
        Joint::HandR,
        Joint::ThighL,
        Joint::ShinL,
        Joint::FootL,
        Joint::ThighR,
        Joint::ShinR,
        Joint::FootR,
    ]
};

/// How wide the ball at a joint is: the narrower cross-section of the limb
/// that hangs from it, a hair under so it never shows through a straight
/// limb and only fills the wedge a bend opens.
fn ball_width(skeleton: &Skeleton, joint: view::skeleton::Joint) -> f32 {
    let size = view::pose::part_size(skeleton, joint);
    size[0].min(size[2]) * 0.98
}

/// Where each fighter's joints are this frame, in the fighter's own frame:
/// what [`pose_balls`] puts the balls at. Written by the posing pass. The
/// Reaping's copies too, and how solid each is, for the Reaver's model
/// (`reaver`), which is posed from here rather than solved again.
#[derive(Resource, Default)]
struct Joints {
    body: [Option<view::skeleton::Skin>; MAX_PLAYERS],
    shadow: [Option<view::skeleton::Skin>; MAX_PLAYERS],
    copies: [[Option<view::skeleton::Skin>; REAP_COPIES]; MAX_PLAYERS],
    copy_solid: [[f32; REAP_COPIES]; MAX_PLAYERS],
}

/// A ball at one of a fighter's joints. See [`BALLS`].
#[derive(Component)]
struct BodyBall {
    owner: usize,
    joint: view::skeleton::Joint,
}

/// The same, on the Reaver's second body.
#[derive(Component)]
struct ShadowBall {
    owner: usize,
    joint: view::skeleton::Joint,
}

/// **A seat nobody is in draws nothing**: its fighter is parked out of the
/// world (`World::present`), and a body hanging under the floor is a thing the
/// camera can find. Only the root is switched; its parts inherit.
fn hide_empty_seats(sim: Res<Sim>, mut roots: Query<(&Fighter, &mut Visibility)>) {
    let present = sim.cur.present();
    for (fighter, mut seen) in roots.iter_mut() {
        let want = if fighter.0 < present {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
    }
}

/// Put the joint balls where the posing pass left the joints.
fn pose_balls(
    sim: Res<Sim>,
    joints: Res<Joints>,
    mut balls: Query<(&BodyBall, &mut Transform), Without<ShadowBall>>,
    mut shadow_balls: Query<(&ShadowBall, &mut Transform), Without<BodyBall>>,
) {
    let skeletons: [Skeleton; MAX_PLAYERS] =
        std::array::from_fn(|i| skeleton_for(sim.cur.players[i].class));
    for (ball, mut tf) in balls.iter_mut() {
        let Some(skin) = joints.body[ball.owner] else {
            continue;
        };
        let at = skin.origin[ball.joint.index()];
        tf.translation = Vec3::new(at[0], at[1], at[2]);
        tf.scale = Vec3::splat(ball_width(&skeletons[ball.owner], ball.joint));
    }
    for (ball, mut tf) in shadow_balls.iter_mut() {
        let Some(skin) = joints.shadow[ball.owner] else {
            continue;
        };
        let at = skin.origin[ball.joint.index()];
        tf.translation = Vec3::new(at[0], at[1], at[2]);
        tf.scale = Vec3::splat(ball_width(&skeletons[ball.owner], ball.joint));
    }
}

/// The build this is, as the tape names it: the commit, the way the hello
/// two players trade names it (`online::terms`).
pub fn build() -> &'static str {
    env!("ARENA_BUILD")
}

/// `SHOT_BLOOD=1` starts a Blood mage in front of her own work, so a capture
/// can look at what 0016 and 0021 draw without playing up to it: three
/// thralls and a heart out in front, the Black vines to the side, stains of a
/// few sizes (one lit), her ground under her and out to the heart, and the
/// heart meter half full. Pair it with `SHOT_FRAME=2` and `DEMO=0`. Ignored for any other
/// class, and read once at start.
fn shot_blood(w: &mut World) {
    use sim::effects::{Effect, EffectKind};
    use sim::moves::blood as b;
    if platform::env("SHOT_BLOOD").is_none() || w.players[0].class != sim::Class::BloodMage {
        return;
    }
    let p = w.players[0];
    // `SHOT_BLOOD=ground`: only her ground under her and her heart meter
    // full -- for a capture of a move from her ground, a heart planted, or
    // the swim -- and nothing else staged.
    if platform::env("SHOT_BLOOD").as_deref() == Some("ground") {
        w.players[0].heart_meter = sim::tuning::heart_meter_full().max(0) as u16;
        sim::ground::lay(&mut w.ground, 0, p.pos, sim::Fx::from_int(4), u16::MAX);
        return;
    }
    // `SHOT_BLOOD=trail`: the trail a bleeding body leaves -- five stains a
    // stride apart across her view -- and the other fighter at its end, for
    // a capture of a spike setting the chain off.
    if platform::env("SHOT_BLOOD").as_deref() == Some("trail") {
        // Across her view, six metres out, so the whole of it is on screen.
        let f = p.facing;
        let spot = |ahead: i32, side: i32| {
            let (a, s) = (sim::Fx::from_int(ahead), sim::Fx::from_int(side));
            sim::V3::new(
                p.pos.x.add(f.x.mul(a)).sub(f.z.mul(s)),
                p.pos.y,
                p.pos.z.add(f.z.mul(a)).add(f.x.mul(s)),
            )
        };
        for k in 0..5 {
            let stain = Effect::pool(0, p.class, b::THORN_LASH, spot(6, 3 * k - 6), 50);
            if let Some(slot) = w.effects.iter_mut().find(|s| s.is_none()) {
                *slot = Some(stain);
            }
        }
        w.players[1].pos = spot(6, 8);
        return;
    }
    let ahead = |forward: i32, side: i32| {
        let f = p.facing;
        sim::V3::new(
            p.pos
                .x
                .add(f.x.mul(sim::Fx::from_int(forward)))
                .sub(f.z.mul(sim::Fx::from_int(side))),
            p.pos.y,
            p.pos
                .z
                .add(f.z.mul(sim::Fx::from_int(forward)))
                .add(f.x.mul(sim::Fx::from_int(side))),
        )
    };
    let mut put = |e: Effect| {
        if let Some(slot) = w.effects.iter_mut().find(|s| s.is_none()) {
            *slot = Some(e);
        }
    };
    let at =
        |kind, slot, pos, reach| Effect::cast(kind, 0, p.class, slot, pos, sim::V3::ZERO, reach);
    for (k, side) in [-2, 0, 2].into_iter().enumerate() {
        let mut thrall = Effect::pool(0, p.class, b::HEART, ahead(9, side), 60 + 30 * k as i32);
        thrall.kind = EffectKind::Thrall;
        thrall.life = u16::MAX;
        // Turned to her, so the capture sees their faces.
        thrall.dir = sim::V3::ZERO.sub(p.facing);
        put(thrall);
    }
    let mut heart = at(
        EffectKind::Heart,
        b::HEART,
        ahead(13, 0),
        sim::tuning::heart_ground(),
    );
    heart.banked = 600;
    put(heart);
    put(at(
        EffectKind::Vines,
        b::VINES,
        ahead(7, -6),
        sim::Fx::from_int(3),
    ));
    // Stains of a few sizes, one of them lit and about to go up, so their
    // shapes show.
    for (k, (forward, side, volume)) in [(4, 3, 120), (6, 4, 60), (3, -3, 90)]
        .into_iter()
        .enumerate()
    {
        let mut stain = Effect::pool(0, p.class, b::THORN_LASH, ahead(forward, side), volume);
        if k == 2 {
            stain.life = 30;
        }
        put(stain);
    }
    // And her heart meter half full, so its bar shows.
    w.players[0].heart_meter = (sim::tuning::heart_meter_full() / 2).max(0) as u16;
    // `SHOT_BLOOD=swim`: and her sunk in it, for `SHOT_FRAME=0` -- the first
    // step brings her up, with no `shift` held.
    if platform::env("SHOT_BLOOD").as_deref() == Some("swim") {
        w.players[0].swim = sim::state::SWIM_UNDER;
    }
    // Her ground: under her, and a field out to the heart.
    for (forward, side, radius) in [(0, 0, 3), (4, 1, 2), (8, -1, 3)] {
        sim::ground::lay(
            &mut w.ground,
            0,
            ahead(forward, side),
            sim::Fx::from_int(radius),
            u16::MAX,
        );
    }
}

/// `SHOT_REAP=1` starts with player two six metres in front of player one
/// and **the Reaping** just begun on him -- the picture of a Reaver's cash-in
/// at a full tally -- so a capture can look at it without playing a whole
/// tally up. In a hunt, `SHOT_REAP=leg` and `SHOT_REAP=back` start it on the
/// creature instead: on a leg (the first part named one), or on top of its
/// back (the first named ridge, back, barrel or plateau). `SHOT_FRAME` picks the
/// moment (it lives `reaping_life`); pair it with `DEMO=0`. Read once at
/// start.
fn shot_reap(w: &mut World) {
    use sim::effects::{Effect, EffectKind};
    let Some(how) = platform::env("SHOT_REAP") else {
        return;
    };
    let p = w.players[0];
    if let ("leg" | "back", Some(beast)) = (how.as_str(), w.monster_mut()) {
        let names: &[&str] = if how == "leg" {
            &["leg", "shin", "thigh"]
        } else {
            &["ridge", "back", "barrel", "plateau"]
        };
        let sp = beast.sp();
        let Some(part) = names
            .iter()
            .find_map(|n| sp.parts.iter().position(|q| q.name.contains(n)))
        else {
            return;
        };
        let sh = sp.shape(part);
        let half = sim::Fx::from_int(2);
        let mid = |lo: sim::Fx, hi: sim::Fx| lo.add(hi).div(half);
        let up = if how == "back" {
            sh.max.y
        } else {
            mid(sh.min.y, sh.max.y)
        };
        let at = sim::V3::new(mid(sh.min.x, sh.max.x), up, mid(sh.min.z, sh.max.z));
        // The creature put so the place sits in front of the camera, which
        // starts looking along +x, holding still, as far off as it takes to
        // see the whole ring of them: further the bigger and higher it is.
        beast.yaw = sim::Fx::ZERO;
        beast.pos = p.pos;
        let near = beast.world_of(part, at);
        let high = near.y.sub(p.pos.y);
        let spread = beast.part_reach_from(part, near);
        let off = sim::Fx::from_int(6)
            .add(spread.mul(sim::Fx::from_int(2)))
            .add(high);
        beast.pos = sim::V3::new(
            p.pos.x.add(off).sub(near.x.sub(p.pos.x)),
            p.pos.y,
            p.pos.z.sub(near.z.sub(p.pos.z)),
        );
        beast.brain.think_left = u16::MAX;
        let met = beast.world_of(part, at);
        let thick = beast.part_reach_from(part, met);
        // Her blow along +x, from where she stands toward it.
        let dir = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
        let mut reaping = Effect::cast(
            EffectKind::Reaping,
            0,
            sim::Class::ShadowReaver,
            0,
            met,
            dir,
            thick,
        );
        reaping.home = sim::V3::new(met.x, p.pos.y, met.z);
        if let Some(slot) = w.effects.iter_mut().find(|s| s.is_none()) {
            *slot = Some(reaping);
        }
        return;
    }
    let six = sim::Fx::from_int(6);
    let at = sim::V3::new(
        p.pos.x.add(p.facing.x.mul(six)),
        p.pos.y,
        p.pos.z.add(p.facing.z.mul(six)),
    );
    w.players[1].pos = at;
    w.players[1].facing = sim::V3::ZERO.sub(p.facing);
    let dir = sim::V3::new(p.facing.x, sim::Fx::ZERO, p.facing.z).normalized();
    // As the sim puts one on a fighter: at his middle, a body thick.
    let mut reaping = Effect::cast(
        EffectKind::Reaping,
        0,
        sim::Class::ShadowReaver,
        0,
        sim::aim::origin(at),
        dir,
        sim::tuning::body_radius(),
    );
    reaping.home = at;
    if let Some(slot) = w.effects.iter_mut().find(|s| s.is_none()) {
        *slot = Some(reaping);
    }
}

/// `SHOT_BARS=dark,light` starts a Dual mage with her two bars there, so a
/// headless capture can look at the wings without playing up to them. A
/// sibling of `SHOT_FRAME`: a way of landing a screenshot on a state rather
/// than a moment. Ignored for any other class, and read once at start.
fn shot_bars(w: &mut World) {
    let Some(spec) = platform::env("SHOT_BARS") else {
        return;
    };
    let mut parts = spec.split(',').map(|s| s.trim().parse::<i32>().ok());
    let (Some(Some(dark)), Some(Some(light))) = (parts.next(), parts.next()) else {
        return;
    };
    for p in w.players.iter_mut() {
        if let sim::Mechanic::Meter {
            dark: d, light: l, ..
        } = &mut p.mechanic
        {
            *d = sim::Fx::from_int(dark);
            *l = sim::Fx::from_int(light);
        }
    }
}

/// `SHOT_MOVE=<move>` starts a hunt with the creature winding up that move at
/// player one, from the distance the move is thrown at, so a capture can look
/// at a telegraph without waiting for the animal to choose it. The same kind
/// of hook as `SHOT_BARS`; pair it with `SHOT_FRAME` to land partway through
/// the windup, and `DEMO=0` so nobody moves. The name is matched loosely
/// against the move list: `bite`, `slam`, `spray`.
fn shot_move(w: &mut World) {
    let Some(name) = platform::env("SHOT_MOVE") else {
        return;
    };
    let name = name.trim().to_lowercase();
    let me = w.players[0].pos;
    if w.monster().is_none() {
        shot_pack_move(w, &name);
        return;
    }
    let Some(beast) = w.monster_mut() else {
        return;
    };
    let Some(kind) = beast
        .sp()
        .moves
        .iter()
        .position(|m| m.name.to_lowercase().contains(&name))
    else {
        return;
    };
    let m = beast.sp().attack(kind as u8);
    // Put it where it would throw this at player one, at the move's own
    // distance and in front of the camera, which starts looking along +x:
    // facing them for a move aimed ahead, and turned away for one aimed
    // behind.
    beast.yaw = if m.aim_cos.raw() < 0 {
        sim::Fx::ZERO
    } else {
        sim::Fx::ratio(1, 2)
    };
    beast.pos = sim::V3::new(me.x.add(m.ideal_range), sim::Fx::ZERO, me.z);
    beast.brain.seen = me;
    beast.brain.grace = 0;
    beast.brain.think_left = u16::MAX;
    // A move thrown at a point on the floor -- a lob, a tongue that stops
    // where it was aimed -- is aimed at them.
    beast.aim_at(me);
    beast.doing = sim::monster::Doing::Startup {
        kind: kind as u8,
        left: m.startup,
    };
    // The worm throws three from under the sand and the rest standing.
    if beast.species == sim::species::SpeciesId::SANDMAW {
        sim::species::sandmaw::fight::ready_for(beast, kind as u8);
    }
    let species = beast.species;
    if species == sim::species::SpeciesId::PAIR {
        shot_pair_move(w, kind as u8);
    }
    if species == sim::species::SpeciesId::VEILSTALKER {
        shot_veil_move(w, kind as u8);
    } // The bird flies its own position from its lore: placed there, aloft for
    // an air move, and started a frame early.
    if species == sim::species::SpeciesId::GALEWING {
        sim::species::galewing::fight::ready_for(w, kind as u8);
    }
    if species == sim::species::SpeciesId::SIEGESHELL {
        shot_siege_move(w, kind as u8);
    }
}

/// `SHOT_CLIMB=up` starts a hunt with the creature at the proving ground's
/// west platform, side on to the camera, already getting up it
/// (`sim::climb`); `SHOT_CLIMB=down` on it, getting down. Pair it with
/// `SHOT_FRAME` to land partway through, and `DEMO=0` so nobody moves.
fn shot_climb(w: &mut World) {
    let Some(how) = platform::env("SHOT_CLIMB") else {
        return;
    };
    let down = how.trim() == "down";
    let ground = w.terrain();
    let Some(beast) = w.monster_mut() else {
        return;
    };
    let sp = beast.sp();
    // Facing -z, its lip at the platform's north face (z = 4 m); getting
    // down, standing on it with its lip at the south face (z = -4 m).
    let lip = sim::climb::probe(sp);
    let (z, y) = if down {
        (sim::Fx::from_int(-4).add(lip), sim::Fx::ratio(3, 2))
    } else {
        (sim::Fx::from_int(4).add(lip), sim::Fx::ZERO)
    };
    beast.pos = sim::V3::new(sim::Fx::from_int(-7), y, z);
    beast.yaw = sim::Fx::ratio(3, 4);
    beast.brain.grace = u16::MAX;
    let top = ground.ground_under(sim::V3::new(
        beast.pos.x,
        sim::Fx::from_int(10),
        sim::Fx::ZERO,
    ));
    let rise = if down { sim::Fx::ZERO.sub(top) } else { top };
    let total = sim::climb::frames(sp, rise);
    beast.doing = sim::monster::Doing::Climb {
        left: total,
        total,
        rise: sim::lore::to_cm(rise),
    };
    // Player one off to the side, looking along +x at its flank.
    let at = beast.pos;
    w.players[0].pos = sim::V3::new(
        sim::Fx::from_int(-13),
        sim::Fx::ZERO,
        at.z.sub(sim::Fx::from_int(2)),
    );
    w.players[0].facing = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
}

/// [`shot_move`] for the Siegeshell, which is forty metres long and throws
/// half its moves with a leg: a leg move is that leg's foot eight metres in
/// front of the camera, the body walking at it; the Shed has player one on
/// its flank; the Plough comes down the valley behind them; the beam is at
/// the siege line with player one watching the wall.
fn shot_siege_move(w: &mut World, kind: u8) {
    use sim::species::siegeshell::{self as ss, fight, gait};
    use sim::{Fx, V3};
    let me = w.players[0].pos;
    let mut stand = None;
    let Some(beast) = w.monster_mut() else {
        return;
    };
    beast.brain.mirror = false;
    match kind {
        ss::FOOTFALL | ss::STAMP | ss::DRAG => {
            // Coming at the camera, its fore left leg in front of it.
            beast.yaw = Fx::ratio(1, 2);
            beast.pos = me;
            let foot = gait::foot_in_gait(beast, 0);
            let want = me.add(V3::new(Fx::from_int(8), Fx::ZERO, Fx::ZERO));
            beast.pos = beast
                .pos
                .add(V3::new(want.x.sub(foot.x), Fx::ZERO, want.z.sub(foot.z)));
            beast.doing = sim::monster::Doing::Prowl;
            if kind != ss::FOOTFALL {
                let a = ss::SPECIES.attack(kind);
                fight::leg::set(
                    beast,
                    fight::leg::Channel {
                        kind: if kind == ss::STAMP {
                            fight::leg::STAMP
                        } else {
                            fight::leg::DRAG
                        },
                        leg: 0,
                        phase: fight::leg::STARTUP,
                        left: a.startup,
                    },
                );
                fight::leg::aim_at(beast, me);
            }
        }
        ss::SHED => {
            // Abreast of player one, them out on its left flank.
            beast.yaw = Fx::ratio(1, 4);
            beast.pos = me.add(V3::new(Fx::from_int(22), Fx::ZERO, Fx::ZERO));
            beast.aim_at(me);
        }
        ss::PLOUGH => {
            // Behind them, coming down the valley the way they look.
            beast.yaw = Fx::ZERO;
            beast.pos = me.sub(V3::new(Fx::from_int(30), Fx::ZERO, Fx::ZERO));
            beast.aim_at(me.add(V3::new(Fx::from_int(20), Fx::ZERO, Fx::ZERO)));
        }
        ss::BEAM => {
            // At the siege line, and player one off its flank, looking at
            // the wall.
            beast.yaw = Fx::ZERO;
            beast.pos = V3::new(Fx::from_int(105), Fx::ZERO, Fx::ZERO);
            fight::body::set(beast, fight::body::SIEGE, true);
            stand = Some(V3::new(Fx::from_int(100), Fx::ZERO, Fx::from_int(-18)));
        }
        _ => {
            // The shell's own moves: thirty metres out, side on.
            beast.yaw = Fx::ratio(1, 4);
            beast.pos = me.add(V3::new(Fx::from_int(40), Fx::ZERO, Fx::ZERO));
        }
    }
    if let Some(at) = stand {
        w.players[0].pos = at;
    }
}

/// [`shot_move`] for the Veilstalker: the move starts a frame early, so its
/// first frame -- the feet setting, the mimic's ghost placed -- happens in
/// the world; the pounce is thrown from the top of the trunk nearest the
/// camera's line, at player one.
fn shot_veil_move(w: &mut World, kind: u8) {
    use sim::species::veilstalker::{self as vs, fight};
    let me = w.players[0].pos;
    let ground = w.terrain();
    let Some(beast) = w.monster_mut() else {
        return;
    };
    let a = vs::SPECIES.attack(kind);
    beast.doing = sim::monster::Doing::Startup {
        kind,
        left: a.startup + 1,
    };
    if kind == vs::POUNCE {
        // On a trunk top in front of the camera, nine metres or so out.
        let (x, z) = sim::arena::veilstalker::TRUNKS
            .iter()
            .copied()
            .min_by_key(|(x, z)| {
                let d = sim::V3::new(
                    sim::Fx::ratio(*x, 100),
                    sim::Fx::ZERO,
                    sim::Fx::ratio(*z, 100),
                )
                .sub(me)
                .flat_len();
                (d.sub(sim::Fx::from_int(9))).abs().raw()
            })
            .unwrap_or((0, 0));
        beast.pos = sim::V3::new(
            sim::Fx::ratio(x, 100),
            sim::Fx::from_int(5),
            sim::Fx::ratio(z, 100),
        );
        beast.own[fight::body::FLAGS] |= fight::flag::PERCHED;
        let to = me.sub(beast.pos);
        beast.yaw = sim::math::atan2_turns(to.z, to.x);
        fight::mark_pounce(beast, me, &ground);
    }
}

/// [`shot_move`] for the cats: a leap is lobbed at player one; the twin
/// pounce is both of them, one in front of the camera and one behind; for
/// anything else the other cat is sent off to the far wall and kept there.
fn shot_pair_move(w: &mut World, kind: u8) {
    use sim::species::pair;
    let me = w.players[0].pos;
    let ground = w.terrain();
    let first = w.monsters[0].expect("the cat just placed");
    for (slot, m) in w.monsters.iter_mut().enumerate() {
        let Some(m) = m else { continue };
        if slot == 0 && kind != pair::TWIN {
            if pair::MOVES[kind as usize].lobbed {
                m.lob(kind);
            }
            continue;
        }
        m.brain.seen = me;
        m.brain.grace = 0;
        m.brain.think_left = u16::MAX;
        if kind == pair::TWIN {
            // Opposite sides: the second where the first is, mirrored
            // through player one.
            if slot == 1 {
                m.pos = me.add(me.sub(first.pos));
                m.yaw = sim::Fx::ZERO;
            }
            m.doing = sim::monster::Doing::Startup {
                kind,
                left: pair::SPECIES.attack(kind).startup,
            };
            m.hit_used = false;
            m.lob(kind);
            pair::fight::mark_at(m, me, &ground, &[None; sim::stones::MAX_STONES]);
        } else {
            m.pos = sim::V3::new(me.x, sim::Fx::ZERO, me.z.add(sim::Fx::from_int(10)));
            m.doing = sim::monster::Doing::Prowl;
        }
    }
}

/// [`shot_move`] for a fight that is only a pack: one body of a kind that
/// throws the move, put at the move's own distance from player one -- in
/// front of the camera, or behind the heels for a scuttle -- and winding it
/// up, with the rest of the pack held back in the den by a long grace. The
/// Gnawers' pile-on is called as the pack calls it, on a ring of them.
fn shot_pack_move(w: &mut World, name: &str) {
    use sim::critter::{flag, is, pose};
    let me = w.players[0].pos;
    let sp = w.critters.sp();
    let Some(kind) = sp
        .moves
        .iter()
        .position(|m| m.name.to_lowercase().contains(name))
    else {
        return;
    };
    let kind = kind as u8;
    let m = sp.attack(kind);
    let Some(pack) = w.pack.as_mut() else {
        return;
    };
    pack.grace = u16::MAX;
    let home = pack.home;
    let lane = |d: sim::Fx| sim::V3::new(me.x.add(d), me.y, me.z);
    if sp.id == sim::species::SpeciesId::GNAWERS && kind == sim::species::gnawers::PILE_ON {
        // A ring of them at four metres, and the pack calls it.
        for (k, c) in (0i32..).zip(
            w.critters
                .iter_mut()
                .filter(|c| c.alive() && !c.has(flag::LEADER)),
        ) {
            let turn = sim::Fx::ratio(k * 2 + 1, 12);
            c.pos = me.add(sim::V3::from_turns(turn).scale(sim::Fx::from_int(4)));
            c.vel = sim::V3::ZERO;
        }
        if let Some(pack) = w.pack.as_mut() {
            pack.seen[0].pos = me;
            pack.seen[0].vel = sim::V3::ZERO;
            pack.seen[0].alive = true;
            sim::species::gnawers::pile_on(pack, &mut w.critters, 0);
        }
        return;
    }
    if sp.id == sim::species::SpeciesId::HORNBACK {
        shot_herd_move(w, kind);
        return;
    }
    let mut chosen = None;
    for (i, c) in w.critters.iter_mut().enumerate() {
        if !c.alive() {
            continue;
        }
        let throws = sp.kind(c.kind).moves.iter().any(|mv| mv.kind == kind);
        if chosen.is_none() && throws {
            chosen = Some(i);
        } else {
            c.pos = home;
        }
    }
    let Some(i) = chosen else { return };
    let behind = sp.moves[kind as usize].clip == pose::SCUTTLE;
    let c = &mut w.critters[i];
    // Where it winds up from: the Gnawers close to their bites before the
    // windup begins (their `DartFrom` and `HamstringFrom`); anything else at
    // the near half of the move's range.
    let from = {
        use sim::species::gnawers as g;
        match (sp.id == sim::species::SpeciesId::GNAWERS, kind) {
            (true, g::DART) => g::knob_fx(g::Knob::DartFrom),
            (true, g::HAMSTRING) => g::knob_fx(g::Knob::HamstringFrom),
            _ => m.ideal_range.sub(m.range_span.mul(sim::Fx::ratio(1, 2))),
        }
    };
    c.pos = if behind { lane(from.neg()) } else { lane(from) };
    c.yaw = if behind { 0 } else { 1 << 15 };
    c.vel = sim::V3::ZERO;
    c.state = is::STARTUP;
    c.act = kind;
    c.timer = m.startup;
    c.set(flag::HIT_USED, false);
    if sp
        .kind(c.kind)
        .moves
        .iter()
        .any(|mv| mv.kind == kind && mv.token)
    {
        c.set(flag::TOKEN, true);
    }
}

/// [`shot_pack_move`] for the Hornback herd, whose moves are about where
/// things stand: the charge at player one with a boulder three metres behind
/// them, so the lane is drawn ending in the rock and the rock ringed; the
/// bellow with the herd home behind the bull and its lane drawn through
/// player one, lees and all; the close moves from their own range; a cow's
/// kick with player one behind it; a buck with player one on its back. The
/// herd is roused and its grace spent, so the rules draw what they would.
fn shot_herd_move(w: &mut World, kind: u8) {
    use sim::critter::{flag, is};
    use sim::species::hornback as h;
    let sp = w.critters.sp();
    let m = sp.attack(kind);
    // The boulders down, and the herd roused.
    for _ in 0..2 {
        w.advance([SimInput::default(); sim::state::MAX_PLAYERS]);
    }
    if let Some(pack) = w.pack.as_mut() {
        pack.mood = sim::pack::mood::HUNTING;
        pack.grace = 0;
    }
    w.advance([SimInput::default(); sim::state::MAX_PLAYERS]);
    if let Some(pack) = w.pack.as_mut() {
        pack.grace = 0;
    }
    let home = w.pack.map_or(sim::V3::ZERO, |p| p.home);
    let Some(b) = w.critters.iter().position(|c| c.kind == h::BULL) else {
        return;
    };
    let rock = sim::hazard::all(&w.lore)
        .find(|(_, hz)| hz.index() == h::BOULDER)
        .map(|(_, hz)| hz.centre());
    let flat = |v: sim::V3| sim::V3::new(v.x, sim::Fx::ZERO, v.z);
    let face = |from: sim::V3, to: sim::V3| sim::pack::yaw_of(to.sub(from));
    let mut me = w.players[0].pos;
    let mut body = b;
    match kind {
        h::CHARGE => {
            // In front of a rock, on the line from the bull through you.
            // Laid along +x, so a camera turned a quarter (`SHOT_YAW=1.57`)
            // sees the whole lane across the screen.
            if let Some(r) = rock {
                let out = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
                me = flat(r).add(out.scale(sim::Fx::from_int(4)));
                let bull_at = me.add(out.scale(sim::Fx::from_int(11)));
                w.critters[b].pos = bull_at;
                w.critters[b].yaw = face(bull_at, me);
            }
        }
        h::BELLOW => {
            // The herd home, the bull between it and player one.
            let out = flat(me.sub(home)).normalized();
            let bull_at = me.sub(out.scale(sim::Fx::from_int(10)));
            w.critters[b].pos = bull_at;
            w.critters[b].yaw = face(bull_at, me);
        }
        h::KICK | h::BUCK => {
            let Some(i) = w
                .critters
                .iter()
                .position(|c| c.kind == h::COW && c.alive())
            else {
                return;
            };
            body = i;
            let cow = w.critters[i];
            if kind == h::KICK {
                me = cow.pos.sub(cow.facing().scale(sim::Fx::from_int(3)));
            } else {
                w.players[0].mount = sim::critter::mount_of(i);
                w.players[0].local = sim::V3::ZERO;
                me = cow.back_point(sp, sim::V3::ZERO);
            }
        }
        _ => {
            // The close moves from where they are thrown: in front, or at the
            // flank for the shoulder.
            let bull_at = sim::V3::new(
                me.x.add(m.ideal_range).add(sim::Fx::from_int(2)),
                sim::Fx::ZERO,
                me.z,
            );
            w.critters[b].pos = bull_at;
            w.critters[b].yaw = face(bull_at, me);
            if kind == h::SHOULDER {
                w.critters[b].yaw = face(
                    bull_at,
                    bull_at.add(sim::V3::new(sim::Fx::ZERO, sim::Fx::ZERO, sim::Fx::ONE)),
                );
                w.critters[b].pos =
                    sim::V3::new(me.x.add(sim::Fx::from_int(2)), sim::Fx::ZERO, me.z);
            }
            if kind == h::TRAMPLE {
                w.players[0].action = sim::state::Action::Stagger { left: 40 };
            }
        }
    }
    w.players[0].pos = me;
    if let Some(pack) = w.pack.as_mut() {
        pack.seen[0].pos = me;
        pack.seen[0].vel = sim::V3::ZERO;
        pack.seen[0].alive = true;
    }
    let c = &mut w.critters[body];
    c.vel = sim::V3::ZERO;
    c.state = is::STARTUP;
    c.act = kind;
    c.timer = m.startup.max(1);
    c.set(flag::HIT_USED, false);
}

/// `SHOT_WEIGHT=n` starts every Bulwark's shield holding `n`, so a capture can
/// look at a loaded shield without blocking up to it. The same kind of hook as
/// `SHOT_BARS`, for the same reason. It drains like any weight, so pair it with
/// an early `SHOT_FRAME`.
fn shot_weight(w: &mut World) {
    let Some(weight) = platform::env("SHOT_WEIGHT").and_then(|s| s.trim().parse::<i32>().ok())
    else {
        return;
    };
    for p in w.players.iter_mut() {
        if let sim::Mechanic::Shield(s) = p.mechanic {
            p.mechanic = sim::Mechanic::Shield(s.with_weight(sim::Fx::from_int(weight)));
        }
    }
}

/// `SHOT_CHASE=1`, with `--range <name>`, starts a chase far from home
/// (exploration 0023): fighter one carries the range's egg, out on the road
/// nearest it, and its creatures are out of their range after them, a little
/// way back toward it. Pair it with `DEMO=0` and a `SHOT_FRAME` of a second or
/// two, to land on them closing.
fn shot_chase(w: &mut World) {
    if platform::env("SHOT_CHASE").is_none() || !w.valley.on || !w.hunting() {
        return;
    }
    let Some(s) = w.hunted().into_iter().flatten().next() else {
        return;
    };
    w.valley.keep.carry_egg(0, s);
    for l in w.loose.iter_mut() {
        if l.is_some_and(|l| l.item == sim::keep::Item::Egg(s)) {
            *l = None;
        }
    }
    let o = w.map_origin();
    let mid = w.den().middle().add(o);
    let road = &sim::valley::layout::road()[0];
    let Some(&(x, z)) = road.iter().min_by_key(|(x, z)| {
        sim::math::wide_flat_dist(mid, sim::V3::new(*x, sim::Fx::ZERO, *z)).raw()
    }) else {
        return;
    };
    let ground = w.terrain();
    let at = |x: sim::Fx, z: sim::Fx| {
        let up = sim::V3::new(x.sub(o.x), sim::Fx::from_int(1000), z.sub(o.z));
        sim::V3::new(up.x, ground.floor_below(up), up.z)
    };
    let back = sim::math::wide_normalized(sim::V3::new(mid.x.sub(x), sim::Fx::ZERO, mid.z.sub(z)));
    for (i, f) in w.players.iter_mut().enumerate().take(2) {
        f.pos = at(x.sub(sim::Fx::from_int(2 * i as i32)), z);
        f.facing = back;
        f.vel = sim::V3::ZERO;
        if let sim::Mechanic::Shadow(_) = f.mechanic {
            f.mechanic = sim::Mechanic::Shadow(sim::class::Shadow::attending(f.pos, f.facing));
        }
    }
    for (k, m) in w.monsters.iter_mut().flatten().enumerate() {
        let d = sim::Fx::from_int(22 + 4 * k as i32);
        let q = sim::V3::new(x, sim::Fx::ZERO, z).add(back.scale(d));
        m.pos = at(q.x, q.z.add(sim::Fx::from_int(3 * k as i32)));
        m.brain.seen = w.players[0].pos;
    }
    for (k, c) in w.critters.iter_mut().enumerate() {
        let q = sim::V3::new(x, sim::Fx::ZERO, z).add(back.scale(sim::Fx::from_int(20)));
        c.pos = at(
            q.x.add(sim::Fx::from_int(k as i32 % 4)),
            q.z.add(sim::Fx::from_int(k as i32 / 4)),
        );
    }
}

/// `SHOT_MOMENTUM=n` starts every Champion with `n` momentum (exploration
/// 0015), so a capture can look at the meter and the glow on his weapon
/// without landing his way up to them. The same kind of hook as
/// `SHOT_WEIGHT`; it holds for two seconds and then drains.
fn shot_momentum(w: &mut World) {
    let Some(n) = platform::env("SHOT_MOMENTUM").and_then(|s| s.trim().parse::<i32>().ok()) else {
        return;
    };
    for p in w.players.iter_mut() {
        if let sim::Mechanic::Forms { fury, .. } = &mut p.mechanic {
            fury.momentum = n.clamp(0, u16::MAX as i32) as u16;
            fury.momentum_left = sim::tuning::momentum_holds().min(u8::MAX as u16) as u8;
        }
    }
}

/// `SHOT_FORM=<weapon>,<n>` starts every Champion with her liquid in that
/// weapon's shape (`sword`, `hammer`, `spear`), as it is `n` frames after the
/// move that called it ended: so `SHOT_FORM=hammer,0` is a whole hammer and
/// one past `view::arms::HOLD` is a hammer starting to melt back into the
/// coil (`docs/design/kits/champion.md`, "The weapon you can see"). The clock
/// runs on from there, so pair it with `SHOT_FRAME` and count. The same kind
/// of hook as `SHOT_MOMENTUM`.
fn shot_form(w: &mut World) {
    let Some(spec) = platform::env("SHOT_FORM") else {
        return;
    };
    let mut it = spec.split(',');
    let name = it.next().unwrap_or("").trim().to_lowercase();
    let Some(form) = sim::class::FORMS.into_iter().find(|f| f.name() == name) else {
        return;
    };
    let rest = it
        .next()
        .and_then(|n| n.trim().parse::<u16>().ok())
        .unwrap_or(0);
    for p in w.players.iter_mut() {
        if let sim::Mechanic::Forms {
            form: f, liquid, ..
        } = &mut p.mechanic
        {
            *f = form;
            *liquid = sim::class::Quicksilver::new(
                rest.saturating_add(60),
                rest,
                None,
                u16::MAX,
                u16::MAX,
            );
        }
    }
}

/// `SHOT_AT=x,z` stands player one at that point of the place the world is
/// in (metres), on whatever is under it, facing `+x` -- `SHOT_AT=x,z,y` on
/// whatever is under that height: a capture of somewhere
/// that is not a start, such as the Waterfall at the head of the Pinewood
/// (`SHOT_AT=772,-4` with `--arena pinewood --open`). A round start puts
/// fighters back on their marks, so in a fight or a course it does nothing.
/// The same kind of hook as `SHOT_WEIGHT`. The numbers may be decimals.
///
/// `SHOT_FALL=1` with it leaves player one **in the air** at that height,
/// falling, rather than on what is under it: a capture of a ledge being
/// caught, `SHOT_AT=20,6.55,10.4` beside the range tower's north face with
/// `--arena range`, `DEMO=0` and `SHOT_FRAME=40` lands on the hang.
fn shot_at(w: &mut World) {
    let Some(spec) = platform::env("SHOT_AT") else {
        return;
    };
    let metres = |v: f32| sim::Fx::from_raw((v * 65536.0).round() as i32);
    let mut it = spec
        .split(',')
        .filter_map(|v| v.trim().parse::<f32>().ok())
        .map(metres);
    let (Some(x), Some(z)) = (it.next(), it.next()) else {
        return;
    };
    // A third number is the height to look down from: under a lid, in a
    // cave, the floor below that rather than the roof.
    let from = it.next().unwrap_or(sim::Fx::from_int(1000));
    let up = sim::V3::new(x, from, z);
    if platform::env("SHOT_FALL").as_deref() == Some("1") {
        let p = &mut w.players[0];
        p.pos = up;
        p.vel = sim::V3::ZERO;
        p.grounded = false;
        p.facing = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
        return;
    }
    // Or the top of anything hanging there -- a shelf up a tree, a roof
    // over a crown -- which a floor is not.
    let ground = w.terrain();
    let y = ground
        .around(up, sim::Fx::ZERO)
        .filter(|s| s.over(up.x, up.z, sim::Fx::ZERO) && s.max.y.raw() <= up.y.raw())
        .map(|s| s.max.y)
        .fold(ground.floor_below(up), |a, b| {
            if b.raw() > a.raw() { b } else { a }
        });
    let p = &mut w.players[0];
    p.pos = sim::V3::new(up.x, y, up.z);
    p.vel = sim::V3::ZERO;
    p.facing = sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO);
}

impl Sim {
    fn local_player(&self) -> usize {
        self.driver.local_player()
    }

    /// Is the simulation being stepped rather than run?
    ///
    /// The HUD reads it to put up the frame readout: stepping without one is
    /// guessing, and the things worth stepping through are exactly the ones too
    /// quick to read at speed.
    pub fn stepping(&self) -> bool {
        self.paused || self.cur.paused
    }

    /// How many frames the replay of this fight holds so far.
    pub fn tape_len(&self) -> usize {
        self.tape.len()
    }
}

/// The scripted hunter, when `DEMO=1` is driving a hunt.
///
/// It is the same bot `cargo run -p hunt --bin fight` measures, so what you
/// watch here and what the fight report scores are the same play sequence.
/// Renderer-side state: it produces *inputs*, and inputs are transmitted rather
/// than recomputed, so nothing about it can reach a peer's simulation.
#[derive(Resource)]
struct Scripted(hunt::Hunter);

impl Default for Scripted {
    fn default() -> Self {
        Scripted(hunt::Hunter::new(0))
    }
}

/// Start pitch override, for capturing the camera at a known angle.
/// `SHOT_SLOW=n` (or `n,from`): one simulated frame every `n` pictures, each
/// picture `1/n` further between them, whatever the machine's speed -- from
/// frame `from` on, if it is given, and at the usual pace before it. So a
/// capture on a renderer that draws three pictures a second still draws every
/// moment between frames, as a fast machine would, and the things the
/// renderer remembers from picture to picture (the arc of a cut) remember
/// what they would in play.
fn shot_slow(frame: u32) -> Option<u32> {
    static SLOW: std::sync::OnceLock<Option<(u32, u32)>> = std::sync::OnceLock::new();
    let (slow, from) = (*SLOW.get_or_init(|| {
        let spec = platform::env("SHOT_SLOW")?;
        let mut parts = spec.split(',').map(|x| x.trim().parse::<u32>().ok());
        let slow = parts.next().flatten()?.max(1);
        Some((slow, parts.next().flatten().unwrap_or(0)))
    }))?;
    (frame >= from).then_some(slow)
}

/// One picture of [`shot_slow`]: the ticks to run, and the clock held at
/// the share between frames this picture is drawn at.
fn slow_motion(clock: &mut TickClock, slow: u32) -> u32 {
    static PICTURE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let k = PICTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % slow;
    clock.hold(k as f32 / slow as f32);
    u32::from(k == 0)
}

fn env_f32(key: &str) -> Option<f32> {
    platform::env_parsed(key)
}

/// `--dev` turns everything on at once: hitbox and hurtbox wireframes, and the
/// Oven.
///
/// It exists because that combination *is* the working mode right now, and a
/// mode you reach for every session should not need two keypresses and a
/// reminder of which two.
///
/// **F10 switches it mid-session**, because the moment you want it is the
/// moment something has gone wrong -- a room that will not form, say -- and
/// relaunching with `--dev` throws that moment away. Switched on, it shows the
/// overlay and the Esc menu's connection details; the Oven stays one F7 away
/// rather than covering the menu you switched it on to read.
pub fn dev_mode() -> bool {
    dev_switch().load(std::sync::atomic::Ordering::Relaxed)
}

fn dev_switch() -> &'static std::sync::atomic::AtomicBool {
    static DEV: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
    DEV.get_or_init(|| std::sync::atomic::AtomicBool::new(platform::flag("--dev")))
}

/// F10: dev mode on or off, and the overlay with it.
fn toggle_dev_mode(show: &mut debug::ShowDebug) {
    let on = !dev_mode();
    dev_switch().store(on, std::sync::atomic::Ordering::Relaxed);
    show.0 = on;
    platform::log(if on {
        "dev mode on (F10): the overlay, and connection details in the Esc menu"
    } else {
        "dev mode off (F10)"
    });
}

fn env_num(key: &str) -> Option<u32> {
    platform::env_parsed(key)
}

/// Where each local player is looking. Renderer-side state: both angles are
/// quantised and handed to the simulation as input, but the floats themselves
/// never cross the wire and never enter a snapshot.
///
/// Pitch goes with the yaw now. It used to live only here, on the grounds that
/// it moved the camera and changed nothing about the fight -- which stopped
/// being true when abilities started landing where the crosshair is.
#[derive(Resource)]
struct Look {
    yaw: f32,
    pitch: f32,
    /// Player two's yaw, for two people on one keyboard. They have no mouse, so
    /// they turn with keys.
    yaw_two: f32,
    grabbed: bool,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            // Player one spawns at -X looking toward +X, where player two is.
            yaw: env_f32("SHOT_YAW").unwrap_or(0.0),
            // Below the horizon, not level, and far enough below it that the
            // eye has ridden out far enough to show your own fighter -- see
            // `Zones::start_pitch`.
            pitch: env_f32("SHOT_PITCH").unwrap_or(view::camera::Zones::tuned().start_pitch()),
            yaw_two: std::f32::consts::PI,
            grabbed: false,
        }
    }
}

impl Look {
    fn aim(&self) -> u16 {
        aim_from_radians(self.yaw)
    }

    /// Turn to the look a recorded input carries, so a replay's camera is
    /// the camera the person had. The inverse of [`Look::aim`] and
    /// [`Look::tilt`], to within the wire's quantum.
    fn follow(&mut self, input: SimInput) {
        self.yaw = input.aim as f32 / 65536.0 * std::f32::consts::TAU;
        self.pitch = input.pitch as f32 / 65536.0 * std::f32::consts::TAU;
    }

    fn tilt(&self) -> i16 {
        pitch_from_radians(self.pitch)
    }

    fn aim_two(&self) -> u16 {
        aim_from_radians(self.yaw_two)
    }
}

#[derive(Resource)]
struct Rig(CameraRig);

/// How far the camera has climbed into the local fighter's head, 0 to 1.
///
/// Lives outside the rig because the renderer needs it and the rig should not
/// know about meshes.
#[derive(Resource, Default)]
struct InsideOwnHead(f32);

impl Default for Rig {
    fn default() -> Self {
        // Settings override this on the first frame; the default keeps the rig
        // sane if the resource is somehow missing.
        Rig(CameraRig::new(RigConfig::default()))
    }
}

#[derive(Component)]
struct Fighter(usize);

#[derive(Component)]
struct BodyPart {
    owner: usize,
    joint: Joint,
}

/// One piece of the Reaver's shadow -- the second skeleton.
///
/// A whole body rather than a marker on the floor. The shadow copies her
/// swings, so it is a thing that can hit you, and a thing that can hit you has
/// to look like one.
#[derive(Component)]
struct ShadowPart {
    owner: usize,
    joint: Joint,
}

/// The root the shadow's parts hang off, so they can be posed in body space
/// exactly the way hers are.
#[derive(Component)]
struct ShadowRoot(usize);

/// **Five of her** (2026-10-19): one of the copies of her shadow body the
/// Reaping throws round the one a full tally was cashed on. A pool per
/// Reaver, hidden until a Reaping of hers is in the world, for the reason
/// the shadow is a pool. See [`place_copies`].
#[derive(Component)]
struct CopyRoot {
    owner: usize,
    copy: usize,
}

/// A piece of one of those copies: a limb, or (`ball`) the ball at a joint.
#[derive(Component)]
struct CopyPart {
    owner: usize,
    copy: usize,
    joint: Joint,
    ball: bool,
}

/// Each copy's own material, so each can fade on its own clock.
#[derive(Resource, Default)]
struct CopyLooks(Vec<[Handle<StandardMaterial>; REAP_COPIES]>);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ShieldMesh(usize);

/// One drawable piece of a persistent effect.
///
/// Four per effect, which is what the widest of them needs: a Grasp is four
/// arms and each one is its own skillshot you have to be able to see coming. A
/// fire pillar uses two (a base and a column, drawn apart because they are two
/// different threats), a black spike two (the field on the floor and the spike
/// standing in it), a thrown blade one.
#[derive(Component)]
struct EffectMesh {
    slot: usize,
    part: usize,
}

/// How many pieces one effect can be drawn as.
///
/// A bolt of Chain lightning: a black stroke and its white rim for each kink,
/// and the flash where it lands, black in white.
const BOLT_STROKES: usize = 8;
const BOLT_PARTS: usize = 2 * BOLT_STROKES + 2;

/// **Counted from the widest effect rather than written down.** It was a
/// literal six, correct on the day the Guillotine lotus had six blades -- and
/// when the lotus grew to twelve the pool did not, so the first six blades had
/// a model and the other six were nothing but the debug overlay's outline. A
/// renderer that quietly draws *part* of a thing is worse than one that fails
/// to draw it: the hit test was right the whole time, so half the blades were
/// cutting people with nothing on screen to say so.
const EFFECT_PARTS: usize = {
    let blades = sim::effects::LOTUS_BLADES;
    let arms = sim::effects::GRASP_ARMS;
    let beads = sim::effects::TETHER_BEADS;
    let rough = sim::effects::ROUGH_BEADS;
    let ring = sim::effects::RING_PIECES;
    let flames = sim::effects::CARPET_FLAMES;
    let ring = if BOLT_PARTS > ring { BOLT_PARTS } else { ring };
    let most = if blades > arms { blades } else { arms };
    let most = if flames > most { flames } else { most };
    let most = if beads > most { beads } else { most };
    let most = if rough > most { rough } else { most };
    let most = if GREMLIN_PARTS > most {
        GREMLIN_PARTS
    } else {
        most
    };
    let most = if VINE_PARTS > most { VINE_PARTS } else { most };
    let most = if HEART_PARTS > most {
        HEART_PARTS
    } else {
        most
    };
    let most = if STAIN_PARTS > most {
        STAIN_PARTS
    } else {
        most
    };
    if ring > most { ring } else { most }
};

/// **The glow off a Blood mage on her own ground** (2026-10-18), or the
/// ripple of her swimming in it: `part` 0 is the ring at her feet, 1 the
/// flame rising off her.
#[derive(Component)]
struct AuraMesh {
    owner: usize,
    part: usize,
}

/// How many pieces the aura is drawn in.
const AURA_PARTS: usize = 2;

/// One of the Elementalist's structures.
#[derive(Component)]
struct StructureMesh {
    owner: usize,
    index: usize,
}

/// The Elementalist's auto, drawn as the thing it is: a thin cylinder from her
/// chest along the line she is aiming, ending where the shot stopped.
///
/// One per fighter, because only one shot can be out at a time. It exists at
/// all because the move used to be invisible -- the pose put a hand out and
/// nothing left it, so what the shot did and which way it went could only be
/// read from the debug overlay.
#[derive(Component)]
struct BeamMesh(usize);

/// One fire bolt in flight.
#[derive(Component)]
struct BoltMesh(usize);

/// One piece of debris, thrown when Cataclysm destroys a structure.
#[derive(Component)]
struct DebrisMesh(usize);

/// One of the Elementalist's Air bolts in flight.
#[derive(Component)]
struct GustMesh(usize);

/// One of her Gales -- the frisbee.
///
/// Its own pool and its own mesh rather than a scaled copy of the bolt's,
/// because the *shape* is the move. A squashed sphere in a barely-opaque
/// material has no flat face and no rim, so however thin it is scaled it reads
/// as a glowing blob; a disc has an edge you can see it turn on. Two pools
/// rather than one with the mesh swapped per frame, which is the same reason
/// every other pool here is fixed: what a shot is drawn as must not be decided
/// on the rollback path.
#[derive(Component)]
struct DiscMesh(usize);

/// One of the six wings on the Dual mage's **back**: her two bars, drawn.
///
/// Not the swept blade above, which is an attack. These are the meter: three
/// a side, dark on her left and light on her right, one materialising at each
/// third of its bar, so the gap between the two is readable across the arena
/// by both players and six is ascension. `view::wings` decides the root, the
/// axes and the count, and `view/tests/wings.rs` holds the count to the bar.
#[derive(Component)]
struct WingSpanMesh {
    owner: usize,
    force: sim::class::Force,
    slot: view::wings::Slot,
    /// Seconds since it appeared, while it is unfolding -- a wing
    /// materialises over a few frames rather than popping. Presentation only.
    unfolding: f32,
}

/// How long a wing takes to unfold once its third of the bar is reached.
const WING_UNFOLD_SECONDS: f32 = 0.22;

/// The wing `view::wings` describes, as one two-sided mesh in the wing's own
/// plane at unit span: `x` out along the wing, `y` across it. The coverts and
/// every feather are convex, so each is fanned from its own middle and the
/// wing is their overlap -- translucent, so the overlaps read as depth.
/// Scaled to each wing's own length when it is placed.
fn wing_mesh() -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut fan = |outline: &[[f32; 2]]| {
        let n = outline.len();
        let centre = outline.iter().fold([0.0f32, 0.0f32], |c, p| {
            [c[0] + p[0] / n as f32, c[1] + p[1] / n as f32]
        });
        let first = positions.len() as u32;
        positions.push([centre[0], centre[1], 0.0]);
        for p in outline {
            positions.push([p[0], p[1], 0.0]);
        }
        for i in 0..n as u32 {
            let a = first + 1 + i;
            let b = first + 1 + (i + 1) % n as u32;
            // Both faces, so it reads from the front and from behind: the two
            // places the two players are.
            indices.extend_from_slice(&[first, a, b, first, b, a]);
        }
    };
    fan(&view::wings::COVERTS);
    for f in view::wings::FEATHERS.iter() {
        fan(&view::wings::feather_outline(f));
    }
    let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], 0.5 - p[1]]).collect();
    let normals: Vec<[f32; 3]> = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// The aim marker a channelled move is wound out along.
///
/// One per fighter: a channel is an action, and nobody is in two at once. It is
/// **not** a thing in the world -- nothing collides with it, nothing is hit by
/// it, and the simulation does not know it is drawn. All it does is answer the
/// only question a channel asks, which is *how far out is this going right
/// now*.
#[derive(Component)]
struct MarkMesh(usize);

/// The Blood mage's scythe.
///
/// One per fighter, because the class is picked at runtime and can change
/// mid-match with Tab. Drawn as the line `view::scythe` gives back, which is
/// the hit volume while she is swinging and the same reach hung off her grip
/// while she is not -- the one condition under which a reach is allowed to
/// grow with a bar is that the blade is drawn at the length it hits at.
#[derive(Component)]
struct ScytheMesh {
    owner: usize,
    /// Which piece: the haft, or one of the two lengths of the curved blade.
    piece: usize,
}

/// A weapon is a haft and a blade in two lengths, so its curve can be seen.
const SCYTHE_PIECES: usize = 3;

/// One piece of one of the shapes the Champion's weapon takes, or one drop
/// of the liquid it is between them.
///
/// A fixed pool -- every fighter gets all three shapes' worth and the drops,
/// hidden unless they are a Champion -- because the class is picked at runtime
/// and spawning meshes on Tab would put allocation on the frame. What shape
/// and stuff each piece is comes from `view::arms::MAKE`, and where it is from
/// `view::arms::pieces`; a drop is `view::arms::Arms::beads`.
#[derive(Component)]
struct ArmsMesh {
    owner: usize,
    /// A weapon, or [`BEAD`] for a drop of the liquid.
    weapon: u8,
    piece: usize,
}

/// The [`ArmsMesh::weapon`] of a drop of the Champion's liquid.
const BEAD: u8 = u8::MAX;

/// The Champion's materials beyond his weapons' own. Made once. The arc his
/// cuts leave is `swing.rs`.
#[derive(Resource)]
struct ArmsLook {
    /// The flare of the string's third hit. See `place_third_flare`.
    flare: Handle<StandardMaterial>,
    /// **The glow at the head of the weapon in his hands** (exploration
    /// 0015): one per momentum tier, ember to white, the HUD bar's colours.
    glow: [Handle<StandardMaterial>; 4],
}

/// The glow itself, one per fighter. See `place_champion_glow`.
#[derive(Component)]
struct GlowMesh {
    owner: usize,
}

/// The third hit's flare, one per fighter. See `place_third_flare`.
#[derive(Component)]
struct FlareMesh {
    owner: usize,
}

/// How many steps of solidity a pool is drawn in: one material per step,
/// because a material is shared by everything wearing it and the solidity is
/// per pool. Six is enough to watch one fade and few enough to make once.
const ESSENCE_STEPS: usize = 6;
/// How much bigger a thrall is drawn on her ground: presentation.
const THRALL_ON_GROUND: f32 = 1.2;

/// One of the Reaver's marks on a body, drawn as a pip over its head.
///
/// `body` is a fighter's index, or [`PIP_QUARRY`] for the creature. **Pips are
/// drawn on the victim** because the victim is who has to read them: a fighter
/// watching his own marks climb knows what is coming and where from. Read
/// straight off `marks`, which is what the cash-in spends -- the drawing and
/// the rule cannot disagree about how many there are.
#[derive(Component)]
struct PipMesh {
    body: usize,
    index: u8,
}

/// [`PipMesh::body`] for the creature.
const PIP_QUARRY: usize = MAX_PLAYERS;

/// One stack of the Dual mage's curse on a body (2026-10-12), drawn as a dark
/// mote circling it at the chest -- one per stack, so the person carrying it
/// and the mage about to purge it both count the same thing. Read straight off
/// `dual::stacks`, which is what the purge spends. `body` is a fighter's
/// index, or [`PIP_QUARRY`] for the creature.
#[derive(Component)]
struct CurseMesh {
    body: usize,
    index: u8,
}

/// Enough motes for the largest curse a byte can hold.
const MAX_MOTES: u8 = 7;

/// One piece of a Dual mage's spell **off her own body** (2026-10-12): the
/// Halo, the Smite's and Judgement's column, the
/// Grasp's hands, the Devour's maw, the Descent, the Veil -- everything she
/// throws whose volume is her own blow rather than a thing in the world, and
/// the wind-up that telegraphs it. See [`spell_piece`].
#[derive(Component)]
struct SpellMesh {
    owner: usize,
    part: usize,
}

/// How many pieces one of her spells is drawn as, at most.
const SPELL_PARTS: usize = 40;

/// Enough pips for the largest cap the Oven allows.
const MAX_PIPS: u8 = 12;

/// Materials for the persistent effects, made once. Which one an entity wears
/// changes as slots are reused, so they are kept rather than rebuilt.
#[derive(Resource)]
struct EffectLook {
    fire: Handle<StandardMaterial>,
    blood: Handle<StandardMaterial>,
    /// The Blood mage's thralls and golem: blood you can see through.
    thrall: Handle<StandardMaterial>,
    /// Her Black vines: barbed black you can see through.
    vines: Handle<StandardMaterial>,
    shade: Handle<StandardMaterial>,
    /// A pool of essence, from faint to nearly solid: the more essence is
    /// left in it, the more solid the figure.
    essence: [Handle<StandardMaterial>; ESSENCE_STEPS],
    /// A heart of hers.
    heart: Handle<StandardMaterial>,
    /// The Dual mage's two forces. Opposites on purpose -- one burns white and
    /// the other drinks the light -- because the whole class is which of the
    /// two you are holding.
    light: Handle<StandardMaterial>,
    dark: Handle<StandardMaterial>,
    /// The wings on the Dual mage's back, one per force. Translucent and
    /// two-sided where the tether and the burst are not: they hang off the
    /// body the player is looking at, and a solid vane the size of her would
    /// hide the fight behind it.
    wing_light: Handle<StandardMaterial>,
    wing_dark: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    /// A stone with fire on it. See `sim::class::Structure::lit`.
    stone_lit: Handle<StandardMaterial>,
    /// The beam and the bolt it lights. Brighter than the pillar and barely
    /// opaque: it is light rather than matter, and it is on screen for two
    /// frames, so it has to read instantly or not at all.
    beam: Handle<StandardMaterial>,
    /// A unit cylinder, cone and sphere, scaled per frame to whatever the
    /// simulation says the volume is. Three meshes rather than one because the
    /// *shape* is the tell: a spike you can see standing in a field is what
    /// makes the field something you decide to walk around, and a flat disc on
    /// the floor is something you notice once you are in it.
    column: Handle<Mesh>,
    spike: Handle<Mesh>,
    ball: Handle<Mesh>,
    hoop: Handle<Mesh>,
    /// The Dual mage's sun-disk: a flat plate of warm light.
    sun: Handle<StandardMaterial>,
    /// Chain lightning: black at the core, and a white rim drawn as the
    /// inside of a slightly fatter shell (only its far faces are drawn), so
    /// the black reads with a white outline against anything behind it.
    bolt: Handle<StandardMaterial>,
    bolt_rim: Handle<StandardMaterial>,
    /// The storm cloud: smoke rather than matter, dark and see-through, and
    /// fainter still at its wisps.
    storm: Handle<StandardMaterial>,
    wisp: Handle<StandardMaterial>,
    /// The fighters' own forms, for the thralls: a limb, a torso and a head
    /// (`limb_form`).
    limb: Handle<Mesh>,
    block: Handle<Mesh>,
    head: Handle<Mesh>,
    /// The Reaver's blades (`blades`): a throwing star, a shard out of the
    /// floor, a crescent cut, and the dark violet steel they are made of.
    shuriken: Handle<Mesh>,
    shard: Handle<Mesh>,
    crescent: Handle<Mesh>,
    steel: Handle<StandardMaterial>,
    forged: Handle<StandardMaterial>,
    /// The Blood mage's scythe, as a weapon (`blades::haft`, `blades::scythe_blade`).
    haft: Handle<Mesh>,
    scythe_blade: Handle<Mesh>,
    /// The elements as things (`fx`): what a column or a ball is drawn as
    /// when it is fire, wind or earth (`EffectLook::dressed`).
    flame: Handle<Mesh>,
    fireball: Handle<Mesh>,
    rubble: Handle<Mesh>,
    whirl: Handle<Mesh>,
    windball: Handle<Mesh>,
    winddisc: Handle<Mesh>,
    comet: Handle<Mesh>,
    rock: Handle<Mesh>,
    /// Moving air, streaked and see-through: the Elementalist's wind.
    air: Handle<StandardMaterial>,
    /// The Blood mage's things as things (`fx`): a ridged thorn, a braided
    /// vine, a drop, a spill on the floor, a tongue of essence.
    thorn: Handle<Mesh>,
    vine: Handle<Mesh>,
    droplet: Handle<Mesh>,
    puddle: Handle<Mesh>,
    tongue: Handle<Mesh>,
    /// The Champion's: a crescent of light, and the gold it is made of.
    crescent_wave: Handle<Mesh>,
    gilt: Handle<StandardMaterial>,
    /// The Dual mage's: a star of light, a cloud, a sunburst, a shard, and
    /// a disc with a sigil on it in each force.
    star: Handle<Mesh>,
    cloud: Handle<Mesh>,
    sunburst: Handle<Mesh>,
    point: Handle<Mesh>,
    disc: Handle<Mesh>,
    sigil_light: Handle<StandardMaterial>,
    sigil_dark: Handle<StandardMaterial>,
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    settings: Res<settings::Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skins: ResMut<Assets<still::SkinMaterial>>,
    mut copies: ResMut<CopyLooks>,
    mut images: ResMut<Assets<Image>>,
    mut flowing: ResMut<fx::Flowing>,
) {
    let eye = commands
        .spawn((
            Camera3d::default(),
            // Bevy's default is a 45-degree vertical field of view, which is a
            // portrait-lens view of an arena you are meant to be moving around
            // inside. The real value is a setting; this is just the starting point.
            Projection::Perspective(PerspectiveProjection {
                fov: settings.fov_radians(),
                ..default()
            }),
            Transform::from_xyz(0.0, 6.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            MainCamera,
            // **No multisampling.** The outline pass below wants a depth buffer it
            // can read, and a multisampled one cannot be read on every backend. The
            // edges are smoothed afterwards instead, which costs one pass and works
            // the same everywhere.
            Msaa::Off,
            bevy::core_pipeline::fxaa::Fxaa::default(),
        ))
        .id();
    if let Some(paper) = wash::Wash::of(wash::style()).filter(|_| platform::draws_outlines()) {
        commands.entity(eye).insert(paper);
    }
    if platform::draws_outlines() {
        // The line round everything: a pass over the finished picture rather
        // than a second copy of every object (`outline`), and the depth-only
        // pass it reads -- the cheapest thing a frame can be asked to draw, and
        // the only way to get a depth buffer that can be sampled rather than
        // only written to. WebGL2 has neither, which is why this is a question.
        commands.entity(eye).insert((
            outline::Outline::from(look::edge::LINE),
            bevy::core_pipeline::prepass::DepthPrepass,
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        arenas::Sun,
    ));
    // A second, dimmer light from behind and opposite, with no shadows. The
    // old camera looked down on the arena from outside it; this one looks along
    // the floor at the inside faces of the walls, which the key light never
    // reaches. Without a fill they read as flat black and the fight happens in
    // front of a void.
    //
    // It carries the sky's colour (`arenas::Skylight`), and it is **directional
    // on purpose**: that is what makes a shadow the complement of its light
    // rather than making the whole arena the complement of its light. Most of
    // the fill used to be the ambient term instead, which lands equally on
    // every face including the ones the sun is already on, and a blue wash over
    // a green field cancels most of the green -- the Gulf's islands came out as
    // sage. The sky is dimmer than the sun and reaches shadows more than it
    // reaches what is already lit, and both of those have to be true here too.
    commands.spawn((
        DirectionalLight {
            illuminance: 2_900.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-8.0, 6.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
        arenas::Skylight,
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.72, 0.85),
        brightness: 130.0,
        ..default()
    });

    // The floor and the arena's geometry are drawn by `arenas::dress`, from
    // whichever arena the simulation is in.

    // Fighters: a root per player, six primitive parts parented to it.
    for (owner, colour) in hud::SEAT_COLOURS.iter().enumerate() {
        // The still world's shade falls on them as it does on the ground
        // (`still::SkinMaterial`).
        let skin = skins.add(still::skin(StandardMaterial {
            base_color: *colour,
            perceptual_roughness: 0.65,
            ..default()
        }));
        commands
            .spawn((Fighter(owner), Transform::default(), Visibility::default()))
            .with_children(|root| {
                // Unit cubes, scaled every frame from whichever build the
                // fighter currently has. Baking the size into the mesh would
                // mean rebuilding sixteen meshes every time somebody presses
                // Tab to change class.
                for joint in JOINTS {
                    root.spawn((
                        Mesh3d(meshes.add(limb_form(joint))),
                        MeshMaterial3d(skin.clone()),
                        Transform::default(),
                        BodyPart { owner, joint },
                    ));
                }
                for joint in BALLS {
                    root.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap())),
                        MeshMaterial3d(skin.clone()),
                        Transform::default(),
                        BodyBall { owner, joint },
                    ));
                }
            });

        // The Reaver's second body. A full skeleton's worth of parts, hidden
        // for the five classes that have no shadow -- a pool rather than
        // something spawned when a shadow appears, for the same reason the
        // effects are a pool: allocating meshes on the rollback path is the
        // most expensive thing that could happen in a tick.
        let shade = materials.add(StandardMaterial {
            // Grey and see-through. It is her, with everything that identifies
            // her taken out: no team colour, because the thing a player must
            // read in a fight is which of the two bodies is the real one.
            base_color: Color::srgba(0.20, 0.21, 0.26, 0.45),
            perceptual_roughness: 0.9,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands
            .spawn((ShadowRoot(owner), Transform::default(), Visibility::Hidden))
            .with_children(|root| {
                for joint in JOINTS {
                    root.spawn((
                        Mesh3d(meshes.add(limb_form(joint))),
                        MeshMaterial3d(shade.clone()),
                        Transform::default(),
                        ShadowPart { owner, joint },
                    ));
                }
                for joint in BALLS {
                    root.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap())),
                        MeshMaterial3d(shade.clone()),
                        Transform::default(),
                        ShadowBall { owner, joint },
                    ));
                }
            });

        // Five of her, for the Reaping: the shadow's body again, each copy in
        // a material of its own so it can fade on its own clock.
        let copy_looks: [Handle<StandardMaterial>; REAP_COPIES] = std::array::from_fn(|_| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(COPY_GREY[0], COPY_GREY[1], COPY_GREY[2], 0.0),
                emissive: LinearRgba::rgb(0.05, 0.04, 0.12),
                perceptual_roughness: 0.9,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })
        });
        for (copy, look) in copy_looks.iter().enumerate() {
            commands
                .spawn((
                    CopyRoot { owner, copy },
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .with_children(|root| {
                    for joint in JOINTS {
                        root.spawn((
                            Mesh3d(meshes.add(limb_form(joint))),
                            MeshMaterial3d(look.clone()),
                            Transform::default(),
                            bevy::pbr::NotShadowCaster,
                            CopyPart {
                                owner,
                                copy,
                                joint,
                                ball: false,
                            },
                        ));
                    }
                    for joint in BALLS {
                        root.spawn((
                            Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap())),
                            MeshMaterial3d(look.clone()),
                            Transform::default(),
                            bevy::pbr::NotShadowCaster,
                            CopyPart {
                                owner,
                                copy,
                                joint,
                                ball: true,
                            },
                        ));
                    }
                });
        }
        copies.0.push(copy_looks);

        // The shield is a separate object because its position is independent
        // of the character -- that is the whole mechanic. See bulwark.md.
        const SHIELD: Vec3 = Vec3::new(0.75, 0.9, 0.14);
        let brass = [0.92, 0.76, 0.38];
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(SHIELD))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(brass[0], brass[1], brass[2]),
                perceptual_roughness: 0.5,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
            ShieldMesh(owner),
        ));
    }

    // A fixed pool, one pair of cylinders per effect slot, because the
    // simulation's effect array is itself fixed. Spawning and despawning meshes
    // as effects come and go would put allocation on the rollback path.
    let unit = meshes.add(Cylinder::new(0.5, 1.0));
    let spike = meshes.add(Cone {
        radius: 0.5,
        height: 1.0,
    });
    let ball = meshes.add(Sphere::new(0.5));
    let flame_tex = images.add(fx::flame_texture());
    let wind_tex = images.add(fx::wind_texture());
    let stone_tex = images.add(fx::stone_texture());
    let blood_tex = images.add(fx::blood_texture());
    let essence_tex = images.add(fx::essence_texture());
    let light_tex = images.add(fx::light_texture());
    let dark_tex = images.add(fx::dark_texture());
    let smoke_tex = images.add(fx::smoke_texture());
    let sigil_tex = images.add(fx::sigil_texture());
    let feather_tex = images.add(fx::feather_texture());
    let look = EffectLook {
        // Translucent, not solid -- it is flame, and a wall of solid orange
        // plastic reads as a structure rather than a hazard you could
        // arguably see an opponent through.
        //
        // Tongues of it (`fx::flame_texture`), flowing up through a heat the
        // mesh carries from a white root to red tips (`fx::heat`).
        fire: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.75, 0.45, 0.8),
            base_color_texture: Some(flame_tex.clone()),
            emissive: LinearRgba::rgb(2.0, 0.75, 0.15),
            emissive_texture: Some(flame_tex.clone()),
            perceptual_roughness: 0.9,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        // Wet: clotted dark in patches (`fx::blood_texture`), and glossy.
        blood: materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.05, 0.12),
            base_color_texture: Some(blood_tex.clone()),
            emissive: LinearRgba::rgb(0.5, 0.0, 0.12),
            perceptual_roughness: 0.35,
            reflectance: 0.6,
            ..default()
        }),
        thrall: materials.add(StandardMaterial {
            base_color: Color::srgba(0.62, 0.02, 0.05, 0.82),
            base_color_texture: Some(blood_tex.clone()),
            emissive: LinearRgba::rgb(0.22, 0.0, 0.01),
            perceptual_roughness: 0.4,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        vines: materials.add(StandardMaterial {
            base_color: Color::srgba(0.07, 0.05, 0.06, 0.8),
            base_color_texture: Some(images.add(fx::vine_texture())),
            emissive: LinearRgba::rgb(0.06, 0.0, 0.02),
            perceptual_roughness: 0.7,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        // A blade of shadow. Dark and translucent so six of them opening at
        // once do not black out whatever they are opening around -- the thing
        // the player has to read is where the victim is, not the flower.
        shade: materials.add(StandardMaterial {
            base_color: Color::srgba(0.10, 0.09, 0.16, 0.80),
            emissive: LinearRgba::rgb(0.12, 0.10, 0.30),
            perceptual_roughness: 0.85,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        essence: std::array::from_fn(|step| {
            // **Essence, not blood** (2026-10-14): a crimson gone toward
            // violet, lit from inside, so it reads as something she draws
            // out of a body rather than what a wound leaves on the floor.
            let solid = 0.18 + 0.72 * step as f32 / (ESSENCE_STEPS - 1) as f32;
            // Folded through itself (`fx::essence_texture`), and flowing.
            materials.add(StandardMaterial {
                base_color: Color::srgba(0.42, 0.05, 0.34, solid),
                base_color_texture: Some(essence_tex.clone()),
                emissive: LinearRgba::rgb(0.70 * solid, 0.04, 0.50 * solid),
                emissive_texture: Some(essence_tex.clone()),
                perceptual_roughness: 0.5,
                alpha_mode: AlphaMode::Blend,
                double_sided: true,
                cull_mode: None,
                ..default()
            })
        }),
        // **A heart** (2026-10-18): a crimson brighter and warmer than her
        // essence, nearly solid and burning from inside, so it is the one
        // thing of hers that reads as an object rather than a stain.
        heart: materials.add(StandardMaterial {
            // Lit only a little from inside: brighter, the bloom washed it
            // out to a pink that read as a sweet rather than a heart. Veined
            // (`fx::heart_texture`), so it is flesh.
            base_color: Color::srgba(0.75, 0.03, 0.09, 0.97),
            base_color_texture: Some(images.add(fx::heart_texture())),
            emissive: LinearRgba::rgb(0.6, 0.0, 0.05),
            perceptual_roughness: 0.75,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        // Hot white with a violet edge: a detonation and the ground it leaves.
        // Bright enough to read as *the* thing on screen, which is what a
        // finisher at full depth is supposed to be.
        light: materials.add(StandardMaterial {
            base_color: Color::srgba(0.98, 0.95, 1.0, 0.7),
            base_color_texture: Some(light_tex.clone()),
            double_sided: true,
            cull_mode: None,
            emissive: LinearRgba::rgb(5.0, 4.4, 6.4),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        // And its opposite. Deep violet, nearly opaque and barely lit: a tether
        // has to be visible along its whole length against anything it crosses,
        // and the one thing it must not do is wash out the body on the end of
        // it, which is what the player is actually looking at.
        wing_light: materials.add(StandardMaterial {
            base_color_texture: Some(feather_tex.clone()),
            base_color: Color::srgba(1.0, 0.95, 0.78, 0.34),
            emissive: LinearRgba::rgb(1.8, 1.5, 0.9),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        wing_dark: materials.add(StandardMaterial {
            base_color_texture: Some(feather_tex.clone()),
            base_color: Color::srgba(0.38, 0.14, 0.62, 0.34),
            emissive: LinearRgba::rgb(0.7, 0.2, 1.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        dark: materials.add(StandardMaterial {
            base_color: Color::srgba(0.16, 0.06, 0.26, 0.92),
            base_color_texture: Some(dark_tex.clone()),
            emissive: LinearRgba::rgb(0.55, 0.12, 0.95),
            emissive_texture: Some(dark_tex.clone()),
            double_sided: true,
            cull_mode: None,
            perceptual_roughness: 0.8,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        stone: materials.add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.60, 0.56),
            base_color_texture: Some(stone_tex.clone()),
            perceptual_roughness: 0.95,
            ..default()
        }),
        // A lit stone: the same rock, glowing at the seams. Bright enough to
        // read as a stove from across the arena, which is what it is.
        //
        // Burning in its cracks (`fx::ember_texture`), so it reads as rock
        // with a fire inside rather than as a rock painted orange.
        stone_lit: materials.add(StandardMaterial {
            base_color: Color::srgb(0.78, 0.66, 0.58),
            base_color_texture: Some(stone_tex.clone()),
            emissive: LinearRgba::rgb(4.0, 4.0, 4.0),
            emissive_texture: Some(images.add(fx::ember_texture())),
            perceptual_roughness: 0.9,
            ..default()
        }),
        beam: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.86, 0.45, 0.85),
            base_color_texture: Some(wind_tex.clone()),
            emissive: LinearRgba::rgb(6.0, 3.4, 0.9),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        air: materials.add(StandardMaterial {
            base_color: Color::srgba(0.88, 0.94, 1.0, 0.6),
            base_color_texture: Some(wind_tex.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        flame: meshes.add(fx::flame()),
        fireball: meshes.add(fx::fireball()),
        rubble: meshes.add(fx::rubble()),
        whirl: meshes.add(fx::whirl()),
        windball: meshes.add(fx::windball()),
        winddisc: meshes.add(fx::winddisc()),
        comet: meshes.add(fx::comet()),
        rock: meshes.add(fx::rock(7)),
        thorn: meshes.add(fx::thorn()),
        vine: meshes.add(fx::vine()),
        droplet: meshes.add(fx::droplet()),
        puddle: meshes.add(fx::puddle(3)),
        tongue: meshes.add(fx::tongue()),
        crescent_wave: meshes.add(fx::crescent()),
        star: meshes.add(fx::star()),
        cloud: meshes.add(fx::cloud()),
        sunburst: meshes.add(fx::sunburst()),
        point: meshes.add(fx::point()),
        disc: meshes.add(fx::disc()),
        sigil_light: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.95, 0.85, 0.8),
            base_color_texture: Some(sigil_tex.clone()),
            emissive: LinearRgba::rgb(4.0, 3.4, 2.4),
            emissive_texture: Some(sigil_tex.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        sigil_dark: materials.add(StandardMaterial {
            base_color: Color::srgba(0.45, 0.2, 0.75, 0.85),
            base_color_texture: Some(sigil_tex.clone()),
            emissive: LinearRgba::rgb(0.7, 0.2, 1.3),
            emissive_texture: Some(sigil_tex.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        gilt: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.82, 0.4, 0.85),
            base_color_texture: Some(wind_tex.clone()),
            emissive: LinearRgba::rgb(4.0, 2.6, 0.8),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        column: unit.clone(),
        spike,
        ball,
        hoop: meshes.add(Torus::new(0.94, 1.0)),
        shuriken: meshes.add(blades::shuriken()),
        shard: meshes.add(blades::shard()),
        crescent: meshes.add(blades::crescent()),
        steel: materials.add(blades::steel()),
        forged: materials.add(blades::forged()),
        haft: meshes.add(blades::haft()),
        scythe_blade: meshes.add(blades::scythe_blade()),
        sun: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.9, 0.62, 0.7),
            base_color_texture: Some(light_tex.clone()),
            emissive: LinearRgba::rgb(4.2, 3.3, 1.6),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        bolt: materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 0.0),
            unlit: true,
            ..default()
        }),
        bolt_rim: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            emissive: LinearRgba::rgb(3.0, 3.0, 3.4),
            unlit: true,
            cull_mode: Some(bevy::render::render_resource::Face::Front),
            ..default()
        }),
        storm: materials.add(StandardMaterial {
            base_color: Color::srgba(0.12, 0.11, 0.15, 0.6),
            base_color_texture: Some(smoke_tex.clone()),
            emissive: LinearRgba::rgb(0.02, 0.015, 0.04),
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        wisp: materials.add(StandardMaterial {
            base_color: Color::srgba(0.16, 0.15, 0.2, 0.3),
            base_color_texture: Some(smoke_tex.clone()),
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        limb: meshes.add(limb_form(view::skeleton::Joint::ArmL)),
        block: meshes.add(limb_form(view::skeleton::Joint::Chest)),
        head: meshes.add(limb_form(view::skeleton::Joint::Head)),
    };
    for slot in 0..sim::effects::MAX_EFFECTS {
        for part in 0..EFFECT_PARTS {
            commands.spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(look.fire.clone()),
                Transform::default(),
                Visibility::Hidden,
                EffectMesh { slot, part },
                // Light and smoke and magic: a bolt's or a cloud's shadow on
                // the floor reads as a solid thing standing in the air.
                bevy::pbr::NotShadowCaster,
            ));
        }
    }

    // The glow off her on her ground: a pool of its own, because it is a
    // reading of her body rather than an effect. Her ground itself is one
    // surface (`claimed`).
    for owner in 0..MAX_PLAYERS {
        for part in 0..AURA_PARTS {
            commands.spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(look.fire.clone()),
                Transform::default(),
                Visibility::Hidden,
                AuraMesh { owner, part },
                bevy::pbr::NotShadowCaster,
            ));
        }
    }

    // Structures get their own pool, because they are not effects: they have no
    // clock and they belong to the Elementalist's mechanic, which is the only
    // place that knows about them.
    // Each a column of earth with its own roughness, so three stones up at
    // once are three stones (`shapes::rock_column`; `docs/design/forms.md`).
    for owner in 0..MAX_PLAYERS {
        for index in 0..sim::class::MAX_STRUCTURES {
            commands.spawn((
                Mesh3d(meshes.add({
                    let mut stone = shapes::rock_column(
                        (owner * sim::class::MAX_STRUCTURES + index) as u32 + 1,
                    );
                    fx::wrap_uv(&mut stone);
                    stone
                })),
                MeshMaterial3d(look.stone.clone()),
                Transform::default(),
                Visibility::Hidden,
                StructureMesh { owner, index },
            ));
        }
    }

    // One beam per fighter and one mesh per fire bolt in flight. Both pools
    // are fixed for the same reason every other one is: spawning meshes as
    // shots come and go would put allocation on the rollback path.
    for owner in 0..MAX_PLAYERS {
        commands.spawn((
            Mesh3d(unit.clone()),
            MeshMaterial3d(look.beam.clone()),
            Transform::default(),
            Visibility::Hidden,
            BeamMesh(owner),
        ));
    }
    let pellet = meshes.add(Sphere::new(0.5));
    let wing = meshes.add(wing_mesh());
    for slot in 0..sim::bolt::MAX_BOLTS {
        commands.spawn((
            Mesh3d(look.comet.clone()),
            MeshMaterial3d(look.fire.clone()),
            Transform::default(),
            Visibility::Hidden,
            BoltMesh(slot),
        ));
    }
    // Debris thrown when Cataclysm destroys a structure. Stone-skinned rather
    // than fire: it is the structure itself going out in pieces, not the
    // blast that broke it.
    for slot in 0..sim::debris::MAX_DEBRIS {
        commands.spawn((
            Mesh3d(look.rock.clone()),
            MeshMaterial3d(look.stone.clone()),
            Transform::default(),
            Visibility::Hidden,
            DebrisMesh(slot),
        ));
    }
    // The Elementalist's air shots. Beam-skinned rather than stone: it is air,
    // and the same barely-opaque material the rest of what she throws with her
    // hands is drawn in. A pool each, because the two are different shapes: a
    // bolt is a stretched bead and a Gale is a disc with a rim.
    for slot in 0..sim::gust::MAX_GUSTS {
        commands.spawn((
            Mesh3d(look.comet.clone()),
            MeshMaterial3d(look.air.clone()),
            Transform::default(),
            Visibility::Hidden,
            GustMesh(slot),
        ));
        commands.spawn((
            Mesh3d(look.winddisc.clone()),
            MeshMaterial3d(look.air.clone()),
            Transform::default(),
            Visibility::Hidden,
            DiscMesh(slot),
        ));
    }
    // The six wings on the Dual mage's back. Spawned for every fighter rather
    // than for her alone, because the class is picked at runtime and can
    // change mid-match with Tab.
    for owner in 0..MAX_PLAYERS {
        // The six on her back, in the colour of the force each one is.
        for (force, material) in [
            (sim::class::Force::Dark, look.wing_dark.clone()),
            (sim::class::Force::Light, look.wing_light.clone()),
        ] {
            for slot in view::wings::ORDER {
                commands.spawn((
                    Mesh3d(wing.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                    WingSpanMesh {
                        owner,
                        force,
                        slot,
                        unfolding: 0.0,
                    },
                ));
            }
        }
    }
    // The Reaver's marks, over every body that can carry them.
    for body in 0..=PIP_QUARRY {
        for index in 0..MAX_PIPS {
            commands.spawn((
                Mesh3d(pellet.clone()),
                MeshMaterial3d(look.shade.clone()),
                Transform::default(),
                Visibility::Hidden,
                PipMesh { body, index },
            ));
        }
    }
    // The Dual mage's curse, a mote per stack round every body.
    for body in 0..=PIP_QUARRY {
        for index in 0..MAX_MOTES {
            commands.spawn((
                Mesh3d(pellet.clone()),
                MeshMaterial3d(look.dark.clone()),
                Transform::default(),
                Visibility::Hidden,
                CurseMesh { body, index },
            ));
        }
    }
    // And her spells off her own body, a pool per fighter.
    for owner in 0..MAX_PLAYERS {
        for part in 0..SPELL_PARTS {
            commands.spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(look.light.clone()),
                Transform::default(),
                Visibility::Hidden,
                SpellMesh { owner, part },
                bevy::pbr::NotShadowCaster,
            ));
        }
    }
    // The aim marker a channelled move is wound out along.
    for owner in 0..MAX_PLAYERS {
        commands.spawn((
            Mesh3d(pellet.clone()),
            MeshMaterial3d(look.blood.clone()),
            Transform::default(),
            Visibility::Hidden,
            MarkMesh(owner),
        ));
    }
    // The Blood mage's scythe: a turned haft and a curved blade
    // (`blades`), in forged iron: the blood is what it leaves on the floor.
    for owner in 0..MAX_PLAYERS {
        for piece in 0..SCYTHE_PIECES {
            commands.spawn((
                Mesh3d(if piece == 0 {
                    look.haft.clone()
                } else {
                    look.scythe_blade.clone()
                }),
                MeshMaterial3d(look.forged.clone()),
                Transform::default(),
                Visibility::Hidden,
                ScytheMesh { owner, piece },
            ));
        }
    }

    // The Champion's three weapons. Each piece is a unit mesh of its shape,
    // scaled per frame; see `view::arms`.
    let block = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    // All of it one gold: the weapon is a liquid, and these are only how it
    // catches the light where it has set into an edge, a body or a grip.
    let stuff = |stuff: view::arms::Stuff| -> StandardMaterial {
        use view::arms::Stuff;
        match stuff {
            // Set to an edge: pale and polished, the brightest thing on her.
            //
            // Half metallic, not all: a fully metallic surface takes its
            // colour from what it reflects, and with nothing round it to
            // reflect it draws brown. A little light of its own keeps it gold
            // on the shadow side too.
            Stuff::Edge => StandardMaterial {
                base_color: Color::srgb(1.0, 0.9, 0.6),
                emissive: LinearRgba::rgb(0.30, 0.22, 0.08),
                metallic: 0.55,
                perceptual_roughness: 0.2,
                ..default()
            },
            Stuff::Gold => StandardMaterial {
                base_color: Color::srgb(1.0, 0.74, 0.25),
                emissive: LinearRgba::rgb(0.22, 0.14, 0.03),
                metallic: 0.55,
                perceptual_roughness: 0.28,
                ..default()
            },
            // Where hands go: deeper and duller.
            Stuff::Deep => StandardMaterial {
                base_color: Color::srgb(0.78, 0.5, 0.14),
                emissive: LinearRgba::rgb(0.10, 0.06, 0.01),
                metallic: 0.5,
                perceptual_roughness: 0.4,
                ..default()
            },
            // Still running: glossy, and lit a little more from inside, so
            // the coil reads against her sleeve in any light.
            Stuff::Liquid => StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.32),
                emissive: LinearRgba::rgb(0.45, 0.30, 0.06),
                metallic: 0.55,
                perceptual_roughness: 0.12,
                ..default()
            },
        }
    };
    use view::arms::Stuff;
    let stuffs = [Stuff::Edge, Stuff::Gold, Stuff::Deep, Stuff::Liquid];
    // Chased, all of it (`fx::gilt_texture`): worked metal, not yellow.
    let chased = images.add(fx::gilt_texture());
    let stuff_handles: [Handle<StandardMaterial>; 4] = std::array::from_fn(|i| {
        materials.add(StandardMaterial {
            base_color_texture: Some(chased.clone()),
            ..stuff(stuffs[i])
        })
    });
    let forged_blade = meshes.add(fx::blade());
    let forged_point = meshes.add(fx::point());
    for owner in 0..MAX_PLAYERS {
        for weapon in 0..view::arms::WEAPONS {
            for piece in 0..view::arms::PIECES {
                let (shape, made_of) = view::arms::MAKE[weapon][piece];
                // The sword's blade forged (`fx::blade`), every point ground
                // to the blade's own section (`fx::point`).
                let mesh = match shape {
                    view::arms::Shape::Block
                        if weapon == sim::moves::champion::SWORD as usize
                            && made_of == view::arms::Stuff::Edge =>
                    {
                        forged_blade.clone()
                    }
                    view::arms::Shape::Block => block.clone(),
                    view::arms::Shape::Rod => unit.clone(),
                    view::arms::Shape::Point => forged_point.clone(),
                    view::arms::Shape::Knob => look.ball.clone(),
                };
                let at = stuffs.iter().position(|s| *s == made_of).unwrap_or(0);
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(stuff_handles[at].clone()),
                    Transform::default(),
                    Visibility::Hidden,
                    ArmsMesh {
                        owner,
                        weapon: weapon as u8,
                        piece,
                    },
                ));
            }
        }
        for bead in 0..view::arms::BEADS {
            commands.spawn((
                Mesh3d(look.droplet.clone()),
                MeshMaterial3d(stuff_handles[3].clone()),
                Transform::default(),
                Visibility::Hidden,
                ArmsMesh {
                    owner,
                    weapon: BEAD,
                    piece: bead,
                },
            ));
        }
        commands.spawn((
            Mesh3d(look.fireball.clone()),
            MeshMaterial3d(look.beam.clone()),
            Transform::default(),
            Visibility::Hidden,
            GlowMesh { owner },
        ));
        commands.spawn((
            Mesh3d(look.ball.clone()),
            MeshMaterial3d(look.beam.clone()),
            Transform::default(),
            Visibility::Hidden,
            FlareMesh { owner },
        ));
    }
    let glow = std::array::from_fn(|tier| {
        let c = hud::momentum_colour(tier as u8).to_linear();
        materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba::new(c.red, c.green, c.blue, 0.7)),
            emissive: LinearRgba::rgb(c.red * 3.0, c.green * 3.0, c.blue * 3.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    });
    let flare = materials.add(StandardMaterial {
        // Kept well under white: the tonemapper washes a bright gold out to
        // the white the momentum glow already owns.
        base_color: Color::LinearRgba(LinearRgba::new(1.0, 0.45, 0.0, 0.85)),
        emissive: LinearRgba::rgb(2.6, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(ArmsLook { glow, flare });
    // Fire rises, wind streams, the beam pours out of her hand.
    flowing.0.extend([
        (look.fire.clone(), Vec2::new(0.05, 0.9)),
        (look.air.clone(), Vec2::new(0.0, -1.4)),
        (look.beam.clone(), Vec2::new(0.0, -4.0)),
        (look.gilt.clone(), Vec2::new(0.0, -2.5)),
        (look.light.clone(), Vec2::new(0.0, -0.8)),
        (look.dark.clone(), Vec2::new(0.04, 0.12)),
        (look.sun.clone(), Vec2::new(0.0, -0.5)),
        (look.storm.clone(), Vec2::new(0.03, 0.02)),
        (look.wisp.clone(), Vec2::new(0.05, 0.03)),
        (look.thrall.clone(), Vec2::new(0.02, 0.15)),
    ]);
    // Essence turns slowly over on itself.
    for step in &look.essence {
        flowing.0.push((step.clone(), Vec2::new(0.03, 0.12)));
    }
    commands.insert_resource(look);
}

/// Stop drawing the local fighter once the camera is inside them.
///
/// Past the handover the eye is at the fighter's own eyes, so their head fills
/// the screen and there is nothing to see but the inside of a box.
///
/// **Faded, not hidden.** The rig comes in quickly once the aim crosses the
/// horizon, and a body that popped out at some threshold on the way would read
/// as a rendering fault. Fading it with the climb makes the handover one
/// continuous motion: the fighter rises toward the middle of the screen and
/// thins out as they get there.
///
/// Only ever the fighter this client is driving. The other one is what you are
/// trying to look at.
fn fade_own_body(
    sim: Res<Sim>,
    inside: Res<InsideOwnHead>,
    parts: Query<(&BodyPart, &MeshMaterial3d<still::SkinMaterial>)>,
    rigs: Res<models::Rigs>,
    mut materials: ResMut<Assets<still::SkinMaterial>>,
) {
    let me = sim.local_player();
    // The rig has already worked out how much of the body to take away and
    // why -- coming up on the crosshair, or the eye simply being close to it.
    let alpha = (1.0 - inside.0).clamp(0.0, 1.0);
    let mine = parts
        .iter()
        .filter(|(part, _)| part.owner == me)
        .map(|(_, material)| &material.0)
        .chain(rigs.looks[me].iter());
    for material in mine {
        // Looked at before it is taken mutably: `Assets::get_mut` marks the
        // asset changed whether or not anything is written to it, and a
        // material marked changed is rebuilt on the GPU that frame -- which
        // this did, for one's own skin, sixteen parts over, every frame. The
        // one `get_mut` left is reached only when the alpha has moved.
        let Some(skin) = materials.get(material) else {
            continue;
        };
        if (skin.base.base_color.alpha() - alpha).abs() < 0.001 {
            continue;
        }
        let Some(skin) = materials.get_mut(material) else {
            continue;
        };
        skin.base.base_color.set_alpha(alpha);
        // Blending only while it is actually translucent. An always-blended
        // fighter sorts against the other one and against the arena for no
        // reason the rest of the time.
        // A cut-out stays one (hair cards): turned opaque it is the cards'
        // rectangles, turned blended it shows its own layers through.
        if !matches!(skin.base.alpha_mode, AlphaMode::Mask(_)) {
            skin.base.alpha_mode = if alpha >= 1.0 {
                AlphaMode::Opaque
            } else {
                AlphaMode::Blend
            };
        }
    }
}

/// The classes drawn as characters rather than mannequins (`reaver`): each
/// model's joints put where the skeletons were just solved -- her own, her
/// shadow's, the Reaping's copies' -- and the mannequins' parts hidden while
/// the models stand in for them.
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // a Bevy system: its arguments are what it reads
fn pose_models(
    sim: Res<Sim>,
    joints: Res<Joints>,
    rigs: Res<models::Rigs>,
    mut inks: ResMut<Assets<ink::InkMaterial>>,
    mut models: Query<(&models::Model, &mut Visibility)>,
    mut bones: Query<&mut Transform, Without<Fighter>>,
    mut parts: Query<(&BodyPart, &mut Visibility), Without<models::Model>>,
    mut balls: Query<(&BodyBall, &mut Visibility), (Without<models::Model>, Without<BodyPart>)>,
    mut shadow_parts: Query<
        (&ShadowPart, &mut Visibility),
        (Without<models::Model>, Without<BodyPart>, Without<BodyBall>),
    >,
    mut shadow_balls: Query<
        (&ShadowBall, &mut Visibility),
        (
            Without<models::Model>,
            Without<BodyPart>,
            Without<BodyBall>,
            Without<ShadowPart>,
        ),
    >,
    mut copy_parts: Query<
        (&CopyPart, &mut Visibility),
        (
            Without<models::Model>,
            Without<BodyPart>,
            Without<BodyBall>,
            Without<ShadowPart>,
            Without<ShadowBall>,
        ),
    >,
) {
    let class = |owner: usize| sim.cur.players[owner].class;
    let modelled = |owner: usize| rigs.ready(owner, class(owner));
    for (model, mut seen) in models.iter_mut() {
        let want = if modelled(model.owner) && model.shown_as(class(model.owner)) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
    }
    for owner in (0..MAX_PLAYERS).filter(|&o| modelled(o)) {
        models::pose(
            rigs.her(owner, class(owner)),
            joints.body[owner].as_ref(),
            &mut bones,
        );
        if class(owner) != sim::Class::ShadowReaver {
            continue;
        }
        models::pose(
            rigs.shadow[owner],
            joints.shadow[owner].as_ref(),
            &mut bones,
        );
        for copy in 0..REAP_COPIES {
            models::pose(
                rigs.copies[owner][copy],
                joints.copies[owner][copy].as_ref(),
                &mut bones,
            );
            let solid = (joints.copy_solid[owner][copy] / COPY_SOLID).min(1.0);
            models::fade_copy(&rigs, owner, copy, solid, &mut inks);
        }
    }
    let mannequin = |owner: usize| {
        if modelled(owner) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        }
    };
    let put = |seen: &mut Visibility, owner: usize| {
        let want = mannequin(owner);
        if *seen != want {
            *seen = want;
        }
    };
    for (part, mut seen) in parts.iter_mut() {
        put(&mut seen, part.owner);
    }
    for (ball, mut seen) in balls.iter_mut() {
        put(&mut seen, ball.owner);
    }
    for (part, mut seen) in shadow_parts.iter_mut() {
        put(&mut seen, part.owner);
    }
    for (ball, mut seen) in shadow_balls.iter_mut() {
        put(&mut seen, ball.owner);
    }
    for (part, mut seen) in copy_parts.iter_mut() {
        put(&mut seen, part.owner);
    }
}

/// Swing the coat tails of every seat drawn by its model (`coat`), from the
/// skeleton just solved and where the fighter is.
fn swing_coats(
    sim: Res<Sim>,
    time: Res<Time>,
    joints: Res<Joints>,
    rigs: Res<models::Rigs>,
    mut coats: ResMut<coat::Coats>,
    fighters: Query<(&Fighter, &Transform)>,
    mut bones: Query<&mut Transform, Without<Fighter>>,
) {
    for (fighter, body) in fighters.iter() {
        let owner = fighter.0;
        let class = sim.cur.players[owner].class;
        if !models::drawn(class) {
            continue;
        }
        let (Some(tails), Some(skin)) = (rigs.tails(owner, class), joints.body[owner].as_ref())
        else {
            continue;
        };
        coat::swing(
            &mut coats,
            owner,
            tails,
            skin,
            body,
            time.delta_secs(),
            &mut bones,
        );
    }
}

/// Put the structure meshes where the stones' field says they are.
///
/// **The field, not the Elementalist's mechanic**, since 2026-09-23: the same
/// `stones::gather` every collision reads, so a Bulwark's planted shield --
/// which joins the field as a wall sized by its weight -- is drawn here at
/// exactly the size bodies and shots are stopped at. Each at its own size.
fn place_structures(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut meshes: Query<(
        &StructureMesh,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let field = sim::stones::gather(&sim.cur.players);
    for (tag, mut tf, mut vis, mut skin) in meshes.iter_mut() {
        let Some(raised) = field[tag.owner * sim::class::MAX_STRUCTURES + tag.index] else {
            *vis = Visibility::Hidden;
            continue;
        };
        // Lit, it glows: what a stone is worth to stand on or hide behind
        // is a thing the other player has to be able to read.
        let want = if raised.lit > 0 {
            &look.stone_lit
        } else {
            &look.stone
        };
        if skin.0 != *want {
            skin.0 = want.clone();
        }
        let radius = raised.radius().to_f32_for_render();
        // It is earth: it climbs out of the floor rather than appearing in the
        // air. The whole column slides up from fully buried, so the visible
        // part grows from the ground and the silhouette is always a slab
        // standing on the floor rather than a block hanging in it.
        //
        // Along a **curve**, not a ramp: it holds near the floor -- the beat
        // where the telegraph is readable and someone can still move -- then
        // erupts. Same duration either way; completely different to play
        // against, which is the whole argument for curves over single numbers.
        //
        // The rise comes from the simulation rather than being worked out again
        // here, because it is no longer decoration: it is where the top of the
        // stone is, and the top of the stone is what you can stand on. A
        // renderer that recomputed it could disagree with the surface the game
        // is holding you up with.
        let rise = raised.risen().to_f32_for_render();
        let height = raised.height().to_f32_for_render();
        *vis = Visibility::Inherited;
        tf.translation = Vec3::new(
            raised.at.x.to_f32_for_render(),
            raised.at.y.to_f32_for_render() + height * (rise - 0.5),
            raised.at.z.to_f32_for_render(),
        );
        tf.scale = Vec3::new(radius * 2.0, height, radius * 2.0);
    }
}

/// Draw whatever line-shaped volume a fighter has out this frame.
///
/// The Elementalist's beam is what it was written for, and the rule it follows
/// is why it now draws more than that: **straight from `state::hitbox`**, which
/// is also what the hit test and the debug overlay read, so the three cannot
/// disagree about where the shot went. Anything whose volume is a line rather
/// than a bubble gets drawn by it -- the Champion's swings, and the Dual mage's
/// wing, which comes out of one fist and opens outward over four frames.
///
/// For the beam the move is two frames long, which is the point: you see a
/// line, at the angle you aimed it, ending on whatever stopped it.
fn place_beams(sim: Res<Sim>, mut meshes: Query<(&BeamMesh, &mut Transform, &mut Visibility)>) {
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        // Not the Dual mage's autos either: the ball and the trail it leaves
        // are drawn as what they are (`spell_piece`), and the stretch it
        // covered this frame is not a rod.
        //
        // And not the Champion's: her weapons are drawn now, on the volume,
        // and the arc of the cut is `swing::place`. A glowing rod
        // the width of the capsule on top of a sword was two drawings of one
        // thing, and the rod is the one that said nothing about the weapon.
        // The debug overlay still draws the capsule itself.
        //
        // And not a swing or a cascade of blows, anybody's: a blade brought
        // round and a blow in flight leave trails (`swing.rs`), drawn from
        // the weapon or the ball itself, and a rod along the volume on top of
        // one was the box the trail replaced.
        let p = &sim.cur.players[tag.0];
        let swung = p.action.attack_kind().is_some_and(|k| {
            matches!(
                sim::moves::get(p.class, k).shape,
                sim::moves::Shape::Swing(_) | sim::moves::Shape::Cascade
            )
        });
        let shot = sim::state::hitbox(p).filter(|hb| {
            !matches!(p.class, sim::Class::Champion | sim::Class::ShadowReaver)
                && hb.is_a_beam()
                && !orb_out(p)
                && !swung
        });
        let Some(hb) = shot else {
            *vis = Visibility::Hidden;
            continue;
        };
        let (from, to) = (fx3(hb.from), fx3(hb.to));
        let along = to - from;
        let length = along.length();
        if length < 0.01 {
            *vis = Visibility::Hidden;
            continue;
        }
        // The unit cylinder stands along Y, so it is turned on to the shot's
        // own direction -- which is how the drawing gets its pitch for free.
        *vis = Visibility::Inherited;
        tf.translation = from + along * 0.5;
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, along / length);
        // Thinner than the volume it stands for. A beam drawn at its full hit
        // radius reads as a pillar of light and hides the fighter behind it;
        // the volume is the overlay's job to show, and this one's job is to
        // say *where the shot went*.
        let width = hb.radius.to_f32_for_render();
        tf.scale = Vec3::new(width, length, width);
    }
}

/// Put the fire bolts where they are, pointing the way they are going.
fn place_bolts(sim: Res<Sim>, mut meshes: Query<(&BoltMesh, &mut Transform, &mut Visibility)>) {
    let radius = sim::tuning::fire_bolt_radius().to_f32_for_render();
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Some(shot) = sim.cur.bolts[tag.0] else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = fx3(shot.pos);
        // Stretched along its flight, so a bolt reads as travelling rather
        // than as a bead hanging in the air.
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, fx3(shot.dir).normalize_or_zero());
        tf.scale = Vec3::new(radius * 2.0, radius * 5.0, radius * 2.0);
    }
}

/// Put the debris where it is, pointing the way it is going -- the same
/// treatment `place_bolts` gives a fire bolt, and for the same reason: a
/// shard is a thing in flight, not a bead hanging in the air.
fn place_debris(sim: Res<Sim>, mut meshes: Query<(&DebrisMesh, &mut Transform, &mut Visibility)>) {
    let radius = sim::tuning::debris_radius().to_f32_for_render();
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Some(shard) = sim.cur.debris[tag.0] else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = fx3(shard.pos);
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, fx3(shard.dir).normalize_or_zero());
        tf.scale = Vec3::splat(radius * 2.0);
    }
}

/// Put the Elementalist's Air bolts where they are.
///
/// Stretched along its flight, the same treatment `place_bolts` gives a fire
/// bolt and for the same reason: it reads as travelling rather than as a bead
/// hanging in the air.
///
/// The scale comes off `Gust::girth`, which is the number the hit test asks
/// for -- the overlay rule (`CLAUDE.md`) applied to something that is not
/// technically an overlay. A shot drawn one size and tested at another is a lie
/// you cannot see through, and the Gale is where that mattered: see
/// [`place_discs`].
fn place_gusts(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut meshes: Query<(
        &GustMesh,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, mut mesh, mut skin) in meshes.iter_mut() {
        let Some(shot) = sim.cur.gusts[tag.0] else {
            *vis = Visibility::Hidden;
            continue;
        };
        use sim::gust::Gale;
        if matches!(shot.gale, Gale::Disc) {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        // A shot carrying fire is drawn as fire, and the ember always is: what
        // a shot is worth on arrival is a thing the other player has to be
        // able to read in flight.
        let want = if shot.lit || matches!(shot.gale, Gale::Ember) {
            &look.fire
        } else {
            &look.air
        };
        if skin.0 != *want {
            skin.0 = want.clone();
        }
        // A bolt is a comet with its tail behind it; a coal is a ball of fire.
        let form = match shot.gale {
            Gale::Bolt => &look.comet,
            _ => &look.fireball,
        };
        if mesh.0 != *form {
            mesh.0 = form.clone();
        }
        let radius = shot.girth().to_f32_for_render();
        tf.translation = fx3(shot.pos);
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, fx3(shot.dir).normalize_or_zero());
        tf.scale = match shot.gale {
            // Stretched along its flight, so a bolt reads as travelling rather
            // than as a bead hanging in the air.
            Gale::Bolt => Vec3::new(radius * 2.0, radius * 5.0, radius * 2.0),
            // A thrown coal: round, and drawn at the size it bursts from.
            _ => Vec3::splat(radius * 2.0),
        };
    }
}

/// Put the Gales where they are, lying in the plane they were thrown in.
///
/// **A frisbee, not a shield and not a ball.** The disc's own axis -- the one a
/// thrown frisbee spins about -- is square to the line of effect that aimed the
/// shot, and lies in the *vertical plane containing that line*. So a Gale
/// thrown level is flat; one thrown down at the floor is tipped nose-down by
/// exactly the angle it was thrown at, and it slices along its own path rather
/// than being pushed through the air face-first.
///
/// ```text
///    n  the disc's axis: square to `dir`, in the vertical plane through it
///    ^
///    |    ,--.            thrown level, the disc lies flat
///    o---( == )------>  dir
///         `--'
///
///
///    o                    thrown downward, the axis tips *forward* with the
///     `.            n     throw -- square to `dir` still, and in that same
///       `.         /      vertical plane still -- so the disc tips with it
///         `.      /       and stays edge-on to its own path, slicing along
///           `.   /        it rather than being shoved through the air
///             `./         face-first
///              /`.
///             /   `v  dir
///
///    (seen from the side, the disc *is* the line through the throw: its
///    plane holds `dir`, so edge-on it and the path are drawn the same)
/// ```
///
/// `n = up - (up . dir) dir`, normalised: the vertical, with whatever part of
/// it runs along the throw taken out. That is the one direction that is both
/// square to the throw and in the plane the throw shares with the vertical,
/// which is the whole of the rule above.
///
/// **This does not put the drawing at odds with the hit test.** The sideways
/// half-width of the disc is `dir x n`, which is *horizontal* whichever way the
/// throw is pitched -- both `dir` and `n` lie in the same vertical plane, so
/// their cross product is square to it. The test reads the centre path and a
/// girth around it and asks that the path be inside the body's own height where
/// it arrives (`aim::first_along`), which is a thin disc of that girth riding
/// the path. Tipping that disc inside the plane its path already lies in leaves
/// the horizontal footprint exactly where it was; all it moves is where the
/// leading and trailing edges sit vertically, by less than the body height the
/// test already spans. Step aside and you are clear, stand above it and you are
/// clear -- before and after.
fn place_discs(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut meshes: Query<(
        &DiscMesh,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, mut skin) in meshes.iter_mut() {
        let Some(shot) = sim.cur.gusts[tag.0] else {
            *vis = Visibility::Hidden;
            continue;
        };
        if !matches!(shot.gale, sim::gust::Gale::Disc) {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        // A Gale flown through fire is drawn as fire, as the bolts are.
        let want = if shot.lit { &look.fire } else { &look.air };
        if skin.0 != *want {
            skin.0 = want.clone();
        }
        let radius = shot.girth().to_f32_for_render();
        let dir = fx3(shot.dir).normalize_or_zero();
        // Thrown straight up or straight down there is no vertical left to take
        // the throw out of, and no plane to be square to either. Any axis will
        // do and the upright one is the least surprising; the aim's own pitch
        // limit means it is never actually reached.
        let axis = (Vec3::Y - dir * Vec3::Y.dot(dir))
            .try_normalize()
            .unwrap_or(Vec3::Y);
        tf.translation = fx3(shot.pos);
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, axis);
        // Thin, but not a sheet: a disc you cannot see turn is a disc nobody
        // reads as spinning, and edge-on is exactly the view the player has of
        // one they threw.
        tf.scale = Vec3::new(radius * 2.0, radius * 0.18, radius * 2.0);
    }
}

/// Draw the Dual mage's wing as the band the hit test reads.
///
/// Is this fighter throwing one of the Dual mage's autos?
fn orb_out(p: &sim::state::Player) -> bool {
    p.class == sim::Class::DualMage
        && p.action
            .attack_kind()
            .is_some_and(sim::moves::dual::is_an_auto)
}

/// The six wings on the Dual mage's back: three a side, one per third of its
/// bar, each a wing-shaped vane in a fixed place that is there or is not.
///
/// Drawn from the *interpolated* frame like the body rather than from the
/// latest snapshot like the attack volumes, because they hang off her and a
/// wing that lagged the back it grows from by half a frame would visibly
/// detach every time she turned.
fn place_wingspans(
    sim: Res<Sim>,
    time: Res<Time>,
    mut meshes: Query<(&mut WingSpanMesh, &mut Transform, &mut Visibility)>,
) {
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    for (mut tag, mut tf, mut vis) in meshes.iter_mut() {
        // The bars are read off the current snapshot -- they are the meter,
        // and a meter is not a thing to blend -- and the body they hang off is
        // read off the blended frame, which is where the body is drawn.
        let mut p = sim.cur.players[tag.owner];
        let drawn = frame.players[tag.owner];
        p.pos = sim::V3::new(
            sim::Fx::from_raw((drawn.pos[0] * 65536.0) as i32),
            sim::Fx::from_raw((drawn.pos[1] * 65536.0) as i32),
            sim::Fx::from_raw((drawn.pos[2] * 65536.0) as i32),
        );
        p.facing = sim::V3::new(
            sim::Fx::from_raw((drawn.facing[0] * 65536.0) as i32),
            sim::Fx::ZERO,
            sim::Fx::from_raw((drawn.facing[2] * 65536.0) as i32),
        );
        let Some(wings) = view::wings::wings(&p) else {
            *vis = Visibility::Hidden;
            tag.unfolding = 0.0;
            continue;
        };
        let wing = wings
            .iter()
            .find(|w| w.force == tag.force && w.slot == tag.slot)
            .copied()
            .expect("all six wings are always answered for");
        if !wing.shown {
            *vis = Visibility::Hidden;
            tag.unfolding = 0.0;
            continue;
        }
        *vis = Visibility::Inherited;
        // Materialise: unfold from the root over a few frames, then hold. The
        // shape never changes with the bar; only whether it is there.
        tag.unfolding = (tag.unfolding + time.delta_secs()).min(WING_UNFOLD_SECONDS);
        let t = tag.unfolding / WING_UNFOLD_SECONDS;
        let unfold = t * t * (3.0 - 2.0 * t);
        let along = Vec3::from_array(wing.along);
        let across = Vec3::from_array(wing.across);
        let normal = along.cross(across);
        tf.translation = Vec3::from_array(wing.root);
        tf.rotation = Quat::from_mat3(&Mat3::from_cols(along, across, normal));
        // The mesh is unit length; the slot says how long this wing is.
        tf.scale = Vec3::new(unfold.max(0.01) * wing.length, wing.length, 1.0);
    }
}

/// Put each channelling fighter's aim marker where their aim currently lands.
///
/// **The renderer aims nothing.** `aim_path` is already solved, every frame of
/// the channel, by the same `sim::aim` call the finished move will use -- see
/// `state::step_channel`. Reading its far end is the whole of this function,
/// and it is why the marker cannot promise a depth the arms do not deliver.
fn place_marks(sim: Res<Sim>, mut meshes: Query<(&MarkMesh, &mut Transform, &mut Visibility)>) {
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Some(piece) = mark_piece(&sim.cur.players[tag.0]) else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = piece.at;
        tf.scale = piece.scale;
        tf.rotation = piece.turn;
    }
}

/// Put each Blood mage's scythe where `view::scythe` says the blade is.
///
/// **The renderer decides nothing about the reach.** The line comes out of the
/// same function the hit test lengthens the sweep with, and the only thing
/// added here is where the grip is drawn -- the body's left hand, which
/// `apply_poses` already works out in the arena for whatever a hand holds.
fn place_scythes(
    sim: Res<Sim>,
    hands: Res<Hands>,
    mut meshes: Query<(&ScytheMesh, &mut Transform, &mut Visibility)>,
) {
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Some(scythe) = drawn_scythe(&sim, &hands, tag.owner) else {
            *vis = Visibility::Hidden;
            continue;
        };
        // Two pieces of one weapon: the haft from the butt to the neck, and
        // the blade from the neck to the tip, bowed the way the hit test's
        // blade is. A third is not drawn.
        if tag.piece >= 2 {
            *vis = Visibility::Hidden;
            continue;
        }
        let (from, to) = match tag.piece {
            0 => (Vec3::from(scythe.butt), Vec3::from(scythe.neck)),
            _ => (Vec3::from(scythe.neck), Vec3::from(scythe.tip)),
        };
        let along = to - from;
        let length = along.length();
        if length < 0.01 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        // The haft is centred on its span; the blade starts at its socket.
        tf.translation = if tag.piece == 0 {
            from + along * 0.5
        } else {
            from
        };
        // A full basis rather than a single axis, so the blade's flat lies in
        // the plane of the haft and the bow -- the plane of the cut -- and
        // does not roll to a world direction of its own. Y runs along the
        // piece; Z is the blade's breadth, in that plane; X is its edge-on
        // thickness, square to it.
        let y = along / length;
        let bow = Vec3::from(scythe.bow);
        let x = y.cross(bow).try_normalize().unwrap_or(Vec3::X);
        let z = x.cross(y);
        tf.rotation = Quat::from_mat3(&Mat3::from_cols(x, y, z));
        // The haft is a pole; the blade is drawn at its own length, its
        // shape kept (`blades::scythe_blade` is a unit blade), so it bows
        // out the way the volume does as her reach grows.
        tf.scale = if tag.piece == 0 {
            Vec3::new(0.065, length, 0.065)
        } else {
            Vec3::splat(length)
        };
    }
}

/// Put the Champion's weapon where `view::arms` says it is, in whatever shape
/// the liquid is in: the coil round her forearm, a weapon in her hands on the
/// hit volume, or the drops running between the two.
///
/// Like the scythe, **the renderer decides nothing about the reach**: the tip
/// of the weapon out comes from the same `state::hitbox` the hit test reads,
/// and the only things added here are where the hands and torso are drawn.
fn place_champion_arms(
    sim: Res<Sim>,
    hands: Res<Hands>,
    torsos: Res<Torsos>,
    mut meshes: Query<(&ArmsMesh, &mut Transform, &mut Visibility)>,
) {
    let mut placed: [Option<view::arms::Arms>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    let mut pieces = [[None; view::arms::WEAPONS]; MAX_PLAYERS];
    for (owner, slot) in placed.iter_mut().enumerate() {
        *slot = drawn_arms(&sim, &hands, &torsos, owner);
        if let Some(arms) = slot.as_ref() {
            for (which, out) in pieces[owner].iter_mut().enumerate() {
                *out = Some(view::arms::pieces(arms, which as u8));
            }
        }
    }
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Some(arms) = placed[tag.owner].as_ref() else {
            *vis = Visibility::Hidden;
            continue;
        };
        let piece = if tag.weapon == BEAD {
            arms.beads[tag.piece]
        } else {
            match pieces[tag.owner][tag.weapon as usize] {
                Some(all) => all[tag.piece],
                None => {
                    *vis = Visibility::Hidden;
                    continue;
                }
            }
        };
        if !piece.shown {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let y = Vec3::from(piece.along);
        let z = Vec3::from(piece.across);
        let x = y.cross(z).try_normalize().unwrap_or(Vec3::X);
        let z = x.cross(y);
        tf.translation = Vec3::from(piece.centre);
        tf.rotation = Quat::from_mat3(&Mat3::from_cols(x, y, z));
        tf.scale = Vec3::from(piece.size);
    }
}

/// Where a fighter's chest, hips and weapon elbow are drawn, as `view::arms`
/// reads them.
/// **The Blood mage's scythe as it is drawn this picture**: on the drawn
/// hands, between the last two simulated frames by the share the body is
/// drawn at. The scythe itself and the trail behind it (`swing::emit_scythes`)
/// both ask this, so the trail is on the blade.
fn drawn_scythe(sim: &Sim, hands: &Hands, owner: usize) -> Option<view::scythe::Scythe> {
    let [left, right] = hands.0[owner];
    let cur = &sim.cur.players[owner];
    let prev = if sim.prev.map_origin() == sim.cur.map_origin() {
        &sim.prev.players[owner]
    } else {
        cur
    };
    view::scythe::scythe_between(prev, cur, sim.clock.alpha(), left.into(), right.into())
}

/// **The Champion's weapon as it is drawn this picture**: on the drawn hands
/// and torso, and between the last two simulated frames by the share the
/// body is drawn at, so the weapon sweeps with the body rather than ticking
/// round once a frame. Everything drawn on the weapon -- its glow, the arc of
/// its cut (`swing::place`) -- asks this, so they are all where it is.
fn drawn_arms(sim: &Sim, hands: &Hands, torsos: &Torsos, owner: usize) -> Option<view::arms::Arms> {
    let [left, right] = hands.0[owner];
    let torso = torso_of(torsos, owner);
    let cur = &sim.cur.players[owner];
    // Across a change of the map's frame the frame before is somewhere else.
    let prev = if sim.prev.map_origin() == sim.cur.map_origin() {
        &sim.prev.players[owner]
    } else {
        cur
    };
    view::arms::champion_arms_between(
        prev,
        cur,
        sim.clock.alpha(),
        left.into(),
        right.into(),
        &torso,
    )
}

fn torso_of(torsos: &Torsos, owner: usize) -> view::arms::Torso {
    let [(chest, chest_turn), (hips, hips_turn)] = torsos.0[owner];
    view::arms::Torso {
        chest: chest.into(),
        chest_turn: quat(chest_turn),
        hips: hips.into(),
        hips_turn: quat(hips_turn),
        shoulder: torsos.1[owner][0].into(),
        elbow: torsos.1[owner][1].into(),
    }
}

/// **The glow on the Champion's weapon** (exploration 0015): a ball of light
/// at the head of whichever weapon is in his hands. It is his momentum -- none
/// below a third, then ember, *Heated*, *Blazing* and white at full, the HUD
/// bar's own colours -- and it swells while he holds a charge, to its full
/// size at a full charge, so the hold is read off the weapon as it winds.
fn place_champion_glow(
    sim: Res<Sim>,
    look: Res<ArmsLook>,
    hands: Res<Hands>,
    torsos: Res<Torsos>,
    mut meshes: Query<(
        &GlowMesh,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, mut mat) in meshes.iter_mut() {
        let p = &sim.cur.players[tag.owner];
        let sim::class::Mechanic::Forms { fury, .. } = p.mechanic else {
            *vis = Visibility::Hidden;
            continue;
        };
        let Some(arms) = drawn_arms(&sim, &hands, &torsos, tag.owner) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let tier = fury.tier();
        // The hold, as a share of the press that makes it a finisher: it
        // swells as the press is held and is at its biggest when the link has
        // become its finisher.
        let held = match p.action {
            sim::state::Action::Channel { kind, held } => {
                let startup = sim::moves::get(p.class, kind).startup as f32;
                ((startup + held as f32) / sim::tuning::hold_from().max(1) as f32).min(1.0)
            }
            sim::state::Action::Startup { kind, .. } | sim::state::Action::Active { kind, .. }
                if sim::moves::champion::is_full(kind) =>
            {
                1.0
            }
            _ => 0.0,
        };
        // Sized to be read from behind him across a fight: the first
        // capture at a quarter of a metre was a bead on the tip. A finisher
        // has spent the meter, and burns by what it spent.
        let finishing = matches!(
            p.action.attack_kind(),
            Some(k) if sim::moves::champion::is_full(k)
        );
        let size = if finishing {
            0.25 + 0.35 * fury.spent as f32 / 100.0
        } else {
            let base = match tier {
                0 => 0.0,
                1 => 0.24,
                2 => 0.36,
                _ => 0.5,
            };
            base + 0.4 * held
        };
        if size <= 0.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        tf.translation = Vec3::from(arms.head());
        tf.scale = Vec3::splat(size);
        // At full charge it is white whatever the meter says: the charge is
        // the one big hit, and it looks like one.
        let shade = if held >= 1.0 { 3 } else { tier as usize };
        let want = &look.glow[shade.min(3)];
        if mat.0 != *want {
            mat.0 = want.clone();
        }
    }
}

fn quat(q: Quat) -> view::math::Quat {
    view::math::Quat([q.x, q.y, q.z, q.w])
}

/// **The third hit's flare** (2026-10-20, from play): a gold light at the head
/// of the weapon from the first frame of the string's third hit, swelling
/// through its wind-up, burning through the swing and gone through the first
/// frames of the recovery. So the hit the string is for is told from the two
/// before it while it is still coming -- and its absence, when the count went
/// wrong, is as plain as the tempo bar's empty third pip.
fn place_third_flare(
    sim: Res<Sim>,
    look: Res<ArmsLook>,
    hands: Res<Hands>,
    torsos: Res<Torsos>,
    mut meshes: Query<(
        &FlareMesh,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, mut mat) in meshes.iter_mut() {
        let p = &sim.cur.players[tag.owner];
        let size = third_flare_size(p);
        let arms = drawn_arms(&sim, &hands, &torsos, tag.owner);
        let (Some(arms), true) = (arms, size > 0.0) else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = Vec3::from(arms.head());
        tf.scale = Vec3::splat(size);
        if mat.0 != look.flare {
            mat.0 = look.flare.clone();
        }
    }
}

/// How big the third hit's flare is this frame: nothing unless the move
/// running is the string's third.
fn third_flare_size(p: &sim::state::Player) -> f32 {
    use sim::state::Action;
    if p.class != sim::Class::Champion {
        return 0.0;
    }
    let third =
        |kind| sim::moves::champion::in_string(kind) && view::arms::place_in_string(p, kind) == 2;
    match p.action {
        Action::Startup { kind, left } if third(kind) => {
            let startup = sim::moves::get(p.class, kind).startup.max(1) as f32;
            0.3 + 0.5 * (1.0 - left as f32 / startup)
        }
        Action::Channel { kind, .. } if third(kind) => 0.8,
        Action::Active { kind, .. } if third(kind) => 0.9,
        Action::Recovery { kind, left } if third(kind) => {
            let m = sim::moves::get(p.class, kind);
            let gone = m.recovery.saturating_sub(left) as f32;
            (0.9 * (1.0 - gone / view::arms::TRAIL_GHOST as f32)).max(0.0)
        }
        _ => 0.0,
    }
}

/// Put the Reaver's pips over whoever is carrying marks.
fn place_pips(sim: Res<Sim>, mut meshes: Query<(&PipMesh, &mut Transform, &mut Visibility)>) {
    for (pip, mut tf, mut vis) in meshes.iter_mut() {
        let Some(at) = pip_spot(&sim.cur, pip.body, pip.index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = at;
        tf.scale = Vec3::splat(0.16);
    }
}

/// Put the curse's motes round whoever carries one.
fn place_curses(sim: Res<Sim>, mut meshes: Query<(&CurseMesh, &mut Transform, &mut Visibility)>) {
    for (mote, mut tf, mut vis) in meshes.iter_mut() {
        let Some((at, size)) = curse_spot(&sim.cur, mote.body, mote.index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Inherited;
        tf.translation = at;
        tf.scale = Vec3::splat(size);
    }
}

/// Where mote `index` of the curse on `body` goes and how big, or `None` if
/// that body carries fewer stacks than that. A ring round the chest that
/// turns on the frame counter, so it is the same picture after a rollback;
/// the fuller the curse the faster it turns and the bigger every mote, so
/// four stacks against five reads across the arena.
fn curse_spot(w: &World, body: usize, index: u8) -> Option<(Vec3, f32)> {
    let (curse, middle, round) = if body == PIP_QUARRY {
        let beast = w.monster()?;
        let head = beast.world_of(species::look(beast.species).head, sim::V3::ZERO);
        (beast.curse, fx3(head), 1.3)
    } else {
        let p = &w.players[body];
        let height = sim::tuning::body_height().to_f32_for_render();
        (p.curse, fx3(p.pos) + Vec3::Y * height * 0.6, 0.65)
    };
    let n = sim::dual::stacks(curse);
    if index >= n {
        return None;
    }
    let spin = w.frame as f32 * (0.03 + 0.012 * n as f32);
    let turn = index as f32 / n as f32 * std::f32::consts::TAU + spin;
    let bob = (spin * 2.0 + index as f32).sin() * 0.12;
    Some((
        middle + Vec3::new(turn.cos() * round, bob, turn.sin() * round),
        0.14 + 0.025 * n as f32,
    ))
}

/// Put the pieces of every Dual mage's spell where [`spell_piece`] says.
fn place_spells(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut emits: ResMut<swing::Emits>,
    camera: Query<&GlobalTransform, With<MainCamera>>,
    mut meshes: Query<(
        &SpellMesh,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let alpha = sim.clock.alpha();
    let eye = camera.single().map(|c| c.translation()).unwrap_or(Vec3::Y);
    let same_frame = sim.prev.map_origin() == sim.cur.map_origin();
    for (tag, mut tf, mut vis, mut mesh, mut mat) in meshes.iter_mut() {
        let Some(piece) = spell_piece(&sim.cur, tag.owner, tag.part) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let then = same_frame
            .then(|| spell_piece(&sim.prev, tag.owner, tag.part))
            .flatten();
        let moved = then.map(|t| piece.at - t.at);
        let piece = between(then, piece, alpha);
        if let Some(leaves) = piece.trail {
            emits.add(leaves.emit(
                swing::key(swing::source::SPELL, tag.owner, leaves.part),
                &piece,
                moved,
                eye,
            ));
            if leaves.only {
                *vis = Visibility::Hidden;
                continue;
            }
        }
        let (want_mesh, want_skin) = (look.dressed(&piece), look.dressed_material(&piece));
        *vis = Visibility::Inherited;
        if mesh.0 != want_mesh {
            mesh.0 = want_mesh;
        }
        if mat.0 != want_skin {
            mat.0 = want_skin;
        }
        tf.translation = piece.at;
        tf.scale = piece.scale;
        tf.rotation = piece.turn;
    }
}

/// **A Dual mage's spell, off her own body**, piece by piece: what is drawn
/// for the move she is throwing, on the frame the world is in. A pure function
/// of the snapshot, like [`effect_piece`].
///
/// Two halves. **The wind-up is a telegraph** -- the Smite's and Judgement's
/// ring of light on the floor where the column will come down, the Grasp's
/// and the Abyss's dark one, the Devour's maw opening, the palm gathering --
/// growing over the startup, so the person opposite can read where it lands
/// and when. **The active frames are the spell** -- drawn at the volume the hit
/// test uses (`sim::state::hitbox`), so what is seen striking is what strikes.
fn spell_piece(w: &World, owner: usize, part: usize) -> Option<Piece> {
    use sim::moves::dual as d;
    use sim::state::Action;
    let p = &w.players[owner];
    if p.class != sim::Class::DualMage || p.health <= 0 {
        return None;
    }
    let height = sim::tuning::body_height().to_f32_for_render();
    let feet = fx3(p.pos);
    let waist = feet + Vec3::Y * height * 0.5;
    let pulse = (w.frame as f32 * 0.25).sin() * 0.5 + 0.5;
    // The Veil and her ward are states, not moves: drawn whatever she is
    // throwing, in the last pieces of the pool.
    if part >= SPELL_PARTS - 3 {
        let k = (part - (SPELL_PARTS - 3)) as f32;
        if sim::dual::veiled(p) {
            let turn = w.frame as f32 * 0.4 + k * std::f32::consts::TAU / 3.0;
            return Some(floating_in(
                Skin::Dark,
                waist + Vec3::new(turn.cos() * 0.35, (k - 1.0) * 0.5, turn.sin() * 0.35),
                0.55,
            ));
        }
        if sim::dual::warded(p) {
            let turn = w.frame as f32 * 0.05 + k * std::f32::consts::TAU / 3.0;
            return Some(floating_in(
                Skin::Light,
                feet + Vec3::new(turn.cos() * 0.7, 0.15 + 0.6 * k, turn.sin() * 0.7),
                0.1,
            ));
        }
        return None;
    }
    let (kind, startup, left) = match p.action {
        // **The auto's ball** (2026-10-13): gathering at her shoulder
        // through the wind-up, then flying its curve with a trail behind it.
        Action::Startup { kind, left } if d::is_an_auto(kind) => {
            let m = sim::moves::get(p.class, kind);
            let through = 1.0 - left as f32 / m.startup.max(1) as f32;
            return orb_piece(p, kind, part, None, through);
        }
        Action::Active { kind, left } if d::is_an_auto(kind) => {
            return orb_piece(p, kind, part, Some(left), 1.0);
        }
        Action::Startup { kind, left } => (kind, true, left),
        Action::Active { kind, left } => (kind, false, left),
        // **The run to the disk**: the trails scattering as she goes through.
        Action::Recovery {
            kind: d::FLURRY_DASH,
            ..
        } if p.haul > 0 => return flurry_piece(w, p, part, None),
        _ => return None,
    };
    let m = sim::moves::get(p.class, kind);
    // How far through the wind-up, or through the active frames.
    let through = if startup {
        1.0 - left as f32 / m.startup.max(1) as f32
    } else {
        1.0 - left as f32 / m.active.max(1) as f32
    };
    let aimed = fx3(p.aim_at());
    let volume = sim::state::hitbox(p);
    let ring = |centre: Vec3, radius: f32, count: usize, skin: Skin, size: f32| -> Option<Piece> {
        (part < count).then(|| {
            let turn = part as f32 / count as f32 * std::f32::consts::TAU + w.frame as f32 * 0.04;
            floating_in(
                skin,
                centre + Vec3::new(turn.cos() * radius, 0.0, turn.sin() * radius),
                size,
            )
        })
    };
    let disc = |centre: Vec3, radius: f32, skin: Skin| -> Option<Piece> {
        (part == 0).then(|| standing(Shape::Column, skin, centre, radius, 0.0, 0.05))
    };
    match kind {
        // **Columns out of the sky**: a ring on the floor through the
        // wind-up, the column itself on the active frames.
        d::SMITE | d::JUDGEMENT => {
            let radius = if startup {
                let mut probe = m;
                probe.radius = m.radius.mul(sim::state::depth_of_size(p, kind));
                probe.radius.to_f32_for_render()
            } else {
                volume?.radius.to_f32_for_render()
            };
            if startup {
                if part == 0 {
                    return Some(standing(
                        Shape::Column,
                        Skin::Light,
                        aimed,
                        radius * through,
                        0.0,
                        0.05,
                    ));
                }
                if part == 1 {
                    return Some(standing(
                        Shape::Column,
                        Skin::Light,
                        aimed,
                        0.08 + 0.12 * through,
                        0.0,
                        16.0,
                    ));
                }
                return ring(aimed + Vec3::Y * 0.1, radius, 10, Skin::Light, 0.12)
                    .filter(|_| part >= 2);
            }
            let at = fx3(volume?.from);
            match part {
                0 => Some(standing(
                    Shape::Column,
                    Skin::Light,
                    at,
                    radius * (1.0 - 0.3 * through),
                    0.0,
                    18.0,
                )),
                _ => ring(
                    at + Vec3::Y * 0.2,
                    radius * (1.0 + through),
                    12,
                    Skin::Light,
                    0.25,
                ),
            }
        }
        // **Hands out of the floor**: a dark ring gathering where she
        // pointed, then spikes of shadow closing on it.
        d::GRASPING_SHADE => {
            let radius = m.radius.to_f32_for_render();
            if startup {
                return disc(aimed, radius * through, Skin::Dark);
            }
            let at = fx3(volume?.from);
            (part < 6).then(|| {
                let turn = part as f32 / 6.0 * std::f32::consts::TAU;
                let lean = radius * (1.0 - through) * 0.8;
                standing(
                    Shape::Spike,
                    Skin::Dark,
                    at + Vec3::new(turn.cos() * lean, 0.0, turn.sin() * lean),
                    0.18,
                    0.0,
                    1.6 * (0.4 + 0.6 * through),
                )
            })
        }
        // **The maw**: two halves of dark drawing apart through the wind-up
        // and snapping shut on the active frames.
        d::DEVOUR => {
            let radius = m.radius.to_f32_for_render();
            let at = if startup { aimed } else { fx3(volume?.from) } + Vec3::Y * height * 0.5;
            let open = if startup { through } else { 1.0 - through };
            match part {
                0 => Some(floating_in(
                    Skin::Dark,
                    at + Vec3::Y * open * 1.1,
                    radius * 0.8,
                )),
                1 => Some(floating_in(
                    Skin::Dark,
                    at - Vec3::Y * open * 1.1,
                    radius * 0.8,
                )),
                _ => None,
            }
        }
        // **The black sun**, telegraphed: its ground darkening where it will
        // stand. The sun itself is the effect.
        d::ABYSS if startup => disc(
            aimed,
            m.radius
                .mul(sim::state::depth_of_size(p, kind))
                .to_f32_for_render()
                * through,
            Skin::Dark,
        ),
        // **Rings round her**: the Halo, the Sanctuary's pulse and the
        // Descent's landing burst out to the radius they strike at.
        d::HALO | d::SANCTUARY | d::DESCENT if !startup => {
            let v = volume?;
            let radius = v.radius.to_f32_for_render() * (0.35 + 0.65 * through);
            let lift = if kind == d::HALO { height * 0.5 } else { 0.2 };
            ring(feet + Vec3::Y * lift, radius, 12, Skin::Light, 0.22)
        }
        // **The Descent's dive**: a pillar of light round her on the way down.
        d::DESCENT => (part == 0).then(|| {
            standing(
                Shape::Column,
                Skin::Light,
                feet,
                0.6 + 0.2 * pulse,
                0.0,
                height + 2.5,
            )
        }),
        // **The Veil's puff**: shadow bursting round her as she goes.
        d::VEIL if !startup => {
            let v = volume?;
            ring(
                waist,
                v.radius.to_f32_for_render() * through,
                8,
                Skin::Dark,
                0.3,
            )
        }
        // **Binary and Equinox**: one of each force gathering in her hands --
        // the stars about to be raised, the bars about to be levelled.
        d::BINARY | d::EQUINOX if startup => {
            let side = Vec3::new(
                -p.facing.z.to_f32_for_render(),
                0.0,
                p.facing.x.to_f32_for_render(),
            );
            match part {
                0 => Some(floating_in(
                    Skin::Dark,
                    waist + side * 0.5,
                    0.12 + 0.18 * through,
                )),
                1 => Some(floating_in(
                    Skin::Light,
                    waist - side * 0.5,
                    0.12 + 0.18 * through,
                )),
                _ => None,
            }
        }
        // **The Shadowstep's going**: she thins into shadow on the spot.
        d::SHADOWSTEP if startup => {
            (part == 0).then(|| floating_in(Skin::Dark, waist, 0.7 * (1.0 - 0.6 * through)))
        }
        // **The storm cloud**, full over her head through the wind-up and
        // as the bolts leave it. The bolts are the effects'.
        d::CHAIN_LIGHTNING => hold_gathering(w, p, kind, if startup { through } else { 1.0 }, part),
        // **The disk**, full over her head through the wind-up; on the active
        // frames the blast runs from it to the point, drawn down the line the
        // hit test uses.
        d::DISK_RAY if startup => hold_gathering(w, p, kind, through, part),
        d::DISK_RAY => {
            let v = volume?;
            let (from, to) = (fx3(v.from), fx3(v.to));
            let swell = 1.0 + 0.25 * (w.frame as f32 * 0.9).sin();
            let r = v.radius.to_f32_for_render();
            match part {
                k if k < 15 => hold_gathering(w, p, kind, 1.0, k),
                15 => Some(rod(Skin::Light, from, to, r * 0.6)),
                16 => Some(rod(Skin::Sun, from, to, r * swell)),
                17 => Some(floating_in(Skin::Light, to, r * 1.8 * swell)),
                _ => None,
            }
        }
        // **The cascade** (2026-10-13): her autos thrown one into the next
        // down the line the hit test strikes along, to the disk at its end.
        d::FLURRY_DASH if !startup => flurry_piece(w, p, part, Some(left)),
        d::FLURRY_DASH => hold_gathering(w, p, kind, through, part),
        _ => None,
    }
}

/// **The Dual mage's auto, drawn**: the ball, and the trail it leaves along
/// the curve it flew -- both read off `sim::moves::orb_at`, the path the hit
/// test uses, so the trail on the screen is where it could hit. `left` is the
/// active frames remaining (`None` in the wind-up, where the ball gathers at
/// the start of its flight, `grown` of the way); on the last the ball has
/// arrived and is drawn at the sweet spot's size.
fn orb_piece(
    p: &sim::state::Player,
    kind: u8,
    part: usize,
    left: Option<u16>,
    grown: f32,
) -> Option<Piece> {
    let m = sim::moves::get(p.class, kind);
    let skin = if kind == sim::moves::dual::DARK_AUTO {
        Skin::Dark
    } else {
        Skin::Light
    };
    let dir = sim::moves::orb_line(p.facing, p.aim_dir(), p.grounded, p.stoop);
    let at = |t: f32| {
        fx3(sim::moves::orb_at(
            p.pos,
            p.facing,
            dir,
            &m,
            sim::Fx::from_raw((t.clamp(0.0, 1.0) * 65536.0) as i32),
        ))
    };
    let ball = m.radius.to_f32_for_render();
    let Some(left) = left else {
        return (part == 0).then(|| floating_in(skin, at(0.0), ball * (0.3 + 0.7 * grown)));
    };
    let (now, size) = if left == 0 {
        (1.0, sim::tuning::orb_sweet_radius().to_f32_for_render())
    } else {
        let (_, now) = sim::moves::orb_progress(&m, left);
        (now.to_f32_for_render(), ball)
    };
    // The ball, and the streak it leaves along its curve from her shoulder.
    (part == 0).then(|| {
        floating_in(skin, at(now), size).leaving(Leaves::streak(
            0,
            ball * 0.8,
            orb_tint(kind),
            ORB_TRAIL,
        ))
    })
}

/// How many frames back a Flurry dash blow's streak reaches: the whole of
/// the dash and the run after it, so every blow's path stands until she has
/// gone through them, the oldest faintest.
const FLURRY_TRAIL: f32 = 30.0;

/// How many frames back an auto's streak reaches: most of its flight.
const ORB_TRAIL: f32 = 5.0;

/// The colour of the streak behind a Dual mage's auto: violet behind the
/// dark, pale gold behind the light, the two halves of her.
fn orb_tint(kind: u8) -> Color {
    if kind == sim::moves::dual::DARK_AUTO {
        Color::srgb(0.55, 0.25, 0.95)
    } else {
        Color::srgb(1.0, 0.92, 0.70)
    }
}

/// **The Flurry dash, drawn** (2026-10-13): her autos cascading down the line
/// `sim::moves::flurry_line` draws -- each blow a ball flying its stretch with
/// a trail, bowing to alternate sides, leaving its trail standing -- and the
/// disk they strike at the end, as tall across as she is. `left` is the active
/// frames remaining; `None` is the run, when the streaks behind her fade as
/// she goes through them (they are trails, `swing.rs`, and linger on their own). The blow in flight is the one
/// `sim::moves::flurry_blow` says is out, so what flies is what hits.
fn flurry_piece(
    w: &World,
    p: &sim::state::Player,
    part: usize,
    left: Option<u16>,
) -> Option<Piece> {
    const PER: usize = 6;
    const SHOWN: usize = 5;
    let m = sim::moves::get(p.class, sim::moves::dual::FLURRY_DASH);
    let (a, d) = sim::moves::flurry_line(p.aim_path.from, p.aim_path.to);
    let (a, d) = (fx3(a), fx3(d));
    let (blows, out) = sim::moves::flurry_blow(&m, left.unwrap_or(0));
    let n = (blows as usize).clamp(1, SHOWN);
    let every = m.rehit.max(1) as f32;
    // Which blow is in flight, and how far along its stretch; past the last,
    // they have all landed.
    let (current, frac) = match (left, out) {
        (Some(l), Some(b)) => {
            let elapsed = m.active.saturating_sub(l) as f32;
            (
                b as usize,
                ((elapsed - b as f32 * every + 1.0) / every).clamp(0.0, 1.0),
            )
        }
        _ => (n, 1.0),
    };
    let way = d - a;
    let flat = Vec3::new(way.x, 0.0, way.z).normalize_or(Vec3::Z);
    let side = Vec3::new(-flat.z, 0.0, flat.x);
    let path = |b: usize, u: f32| {
        let bow = (std::f32::consts::PI * u).sin();
        let sign = if b % 2 == 0 { 1.0 } else { -1.0 };
        a.lerp(d, (b as f32 + u) / n as f32) + side * (sign * bow * 0.9) + Vec3::Y * (bow * 0.35)
    };
    let skin_of = |b: usize| if b % 2 == 0 { Skin::Dark } else { Skin::Light };
    let ball = 0.6;
    if part < SHOWN * PER {
        let (b, k) = (part / PER, part % PER);
        if b >= n || b > current {
            return None;
        }
        if b == current {
            // The blow in flight, and the streak it leaves along its bow.
            let kind = if b % 2 == 0 {
                sim::moves::dual::DARK_AUTO
            } else {
                sim::moves::dual::LIGHT_AUTO
            };
            return (k == 0).then(|| {
                floating_in(skin_of(b), path(b, frac), ball).leaving(Leaves::streak(
                    b as u8,
                    ball * 0.7,
                    orb_tint(kind),
                    FLURRY_TRAIL,
                ))
            });
        }
        // A landed blow's trail stays where it flew: the streak it left,
        // lingering for the whole of the dash (`FLURRY_TRAIL`).
        return None;
    }
    // The disk, swelling into being as the last blow flies at it.
    let grown = if current + 1 >= n {
        if current >= n { 1.0 } else { frac }
    } else {
        0.0
    };
    if grown <= 0.0 {
        return None;
    }
    let r = sim::tuning::body_height().to_f32_for_render() * 0.5 * grown;
    let shiver = 1.0 + 0.04 * (w.frame as f32 * 1.3).sin();
    match part - SHOWN * PER {
        0 => Some(plate(Skin::Dark, d, flat, r * shiver, 0.08)),
        1 => Some(hoop(Skin::Light, d, flat, r * 1.02)),
        2 => Some(hoop(Skin::Light, d, flat, r * 0.6 * shiver)),
        _ => None,
    }
}

/// **A long hold, `full` of the way to coming out** (2026-10-12): Chain
/// lightning's storm cloud churning over her head, the Disk ray's disk opening
/// there, or the Flurry dash's two forces wound round her hands.
fn hold_gathering(
    w: &World,
    p: &sim::state::Player,
    kind: u8,
    full: f32,
    part: usize,
) -> Option<Piece> {
    use sim::moves::dual as d;
    let over = fx3(sim::aim::overhead(p.pos));
    let height = sim::tuning::body_height().to_f32_for_render();
    let waist = fx3(p.pos) + Vec3::Y * height * 0.55;
    let spin = w.frame as f32;
    match kind {
        // **A storm cloud** (2026-10-13): a mass of dark smoke the width of a
        // room, flat underneath and heaped on top, wisps trailing off its
        // rim -- hung high enough (`overhead, hangs above her feet`) that the
        // bolts under it are what the camera sees, not the cloud.
        d::CHAIN_LIGHTNING => {
            const PUFFS: usize = 18;
            const WISPS: usize = 10;
            let reach = 1.0 + 2.0 * full;
            let drift = spin * 0.004;
            if part < PUFFS {
                let k = part as f32;
                let a = k * 2.399_963 + drift;
                let out = ((k + 0.5) / PUFFS as f32).sqrt();
                let breathe = 1.0 + 0.08 * (spin * 0.05 + k * 1.7).sin();
                let size = reach * (0.55 - 0.25 * out) * breathe;
                let heap = (1.0 - out) * reach * 0.25 + (k * 3.1).sin() * 0.15;
                return Some(Piece {
                    shape: Shape::Ball,
                    skin: Skin::Storm,
                    at: over + Vec3::new(a.cos() * out * reach, heap, a.sin() * out * reach * 0.85),
                    scale: Vec3::new(size * 2.0, size * 1.1, size * 1.7),
                    turn: Quat::IDENTITY,
                    trail: None,
                });
            }
            if part < PUFFS + WISPS {
                let k = (part - PUFFS) as f32;
                let a = k / WISPS as f32 * std::f32::consts::TAU + spin * 0.01 + (k * 1.3).sin();
                let sway = (spin * 0.03 + k * 2.1).sin();
                let r = reach * (1.05 + 0.2 * sway);
                return Some(Piece {
                    shape: Shape::Ball,
                    skin: Skin::Wisp,
                    at: over + Vec3::new(a.cos() * r, 0.1 * sway, a.sin() * r),
                    scale: Vec3::new(reach * 0.5, reach * 0.06, reach * 0.16),
                    turn: Quat::from_rotation_y(-a + std::f32::consts::FRAC_PI_2),
                    trail: None,
                });
            }
            None
        }
        // **The sun-disk** (2026-10-13): a flat plate of light upright across
        // her front, facing the way she faces, two rings round it and the
        // beads turning round the rim -- opening to full over the wind-up.
        d::DISK_RAY => {
            let face = Vec3::new(
                p.facing.x.to_f32_for_render(),
                0.0,
                p.facing.z.to_f32_for_render(),
            )
            .normalize_or(Vec3::Z);
            let side = Vec3::new(-face.z, 0.0, face.x);
            let open = 0.25 + 0.75 * full;
            match part {
                0 => Some(plate(Skin::Sun, over, face, 1.35 * open, 0.06)),
                1 => Some(hoop(Skin::Light, over, face, 1.7 * open)),
                2 => Some(hoop(Skin::Light, over, face, 1.95 * open)),
                k if k < 15 => {
                    let turn = (k - 3) as f32 / 12.0 * std::f32::consts::TAU - spin * 0.05;
                    let r = 2.25 * open;
                    Some(floating_in(
                        Skin::Light,
                        over + side * (turn.cos() * r) + Vec3::Y * (turn.sin() * r),
                        0.14 + 0.12 * full,
                    ))
                }
                _ => None,
            }
        }
        d::FLURRY_DASH => (part < 6).then(|| {
            let k = part as f32;
            let turn = k / 6.0 * std::f32::consts::TAU + spin * 0.35;
            let skin = if part % 2 == 0 {
                Skin::Dark
            } else {
                Skin::Light
            };
            floating_in(
                skin,
                waist
                    + Vec3::new(
                        turn.cos() * 0.55,
                        (spin * 0.3 + k).sin() * 0.2,
                        turn.sin() * 0.55,
                    ),
                0.08 + 0.12 * full,
            )
        }),
        _ => None,
    }
}

/// Where pip `index` over `body` goes, or `None` if that body carries fewer
/// marks than that.
///
/// A ring over the head rather than a row, so it reads the same from every
/// side -- the one watching his own marks climb is usually looking at the back
/// of his own head.
fn pip_spot(w: &World, body: usize, index: u8) -> Option<Vec3> {
    let (marks, top) = if body == PIP_QUARRY {
        let beast = w.monster()?;
        let head = beast.world_of(species::look(beast.species).head, sim::V3::ZERO);
        (beast.marks, fx3(head) + Vec3::Y * 1.5)
    } else {
        let p = &w.players[body];
        let height = sim::tuning::body_height().to_f32_for_render();
        (p.marks, fx3(p.pos) + Vec3::Y * (height + 0.35))
    };
    if index >= marks {
        return None;
    }
    let turn = index as f32 / marks.max(1) as f32 * std::f32::consts::TAU;
    Some(top + Vec3::new(turn.cos(), 0.0, turn.sin()) * 0.3)
}

/// Where a fighter's aim marker is, if they are channelling at all.
///
/// Split out so the geometry can be asserted without a renderer, the same way
/// `effect_piece` is: what wants checking is that the marker sits on the far
/// end of the solved path, because a marker that sits anywhere else is worse
/// than no marker.
fn mark_piece(p: &sim::state::Player) -> Option<Piece> {
    p.action.channelling()?;
    Some(floating(
        fx3(p.aim_path.to),
        sim::tuning::grasp_mark().to_f32_for_render(),
    ))
}

/// Put the effect meshes where the simulation says its effects are.
///
/// Shape comes from the same volumes the hit test uses, so what you see standing
/// in the arena is what will actually catch you. A renderer that reconstructed
/// the shape itself would drift from the rule, and a fire pillar that looks
/// bigger than it hits is worse than no fire pillar.
fn place_effects(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut emits: ResMut<swing::Emits>,
    camera: Query<&GlobalTransform, With<MainCamera>>,
    mut meshes: Query<(
        &EffectMesh,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let alpha = sim.clock.alpha();
    let eye = camera.single().map(|c| c.translation()).unwrap_or(Vec3::Y);
    let same_frame = sim.prev.map_origin() == sim.cur.map_origin();
    for (tag, mut tf, mut vis, mut mesh, mut mat) in meshes.iter_mut() {
        let effect = sim.cur.effects[tag.slot];
        let piece = effect.and_then(|e| effect_piece(&e, tag.part));
        let Some(mut piece) = piece else {
            *vis = Visibility::Hidden;
            continue;
        };
        // The same effect on the frame before, if it was there: one a frame
        // younger, of the same kind.
        let then = sim.prev.effects[tag.slot]
            .filter(|t| {
                same_frame && effect.is_some_and(|e| t.kind == e.kind && t.age + 1 == e.age)
            })
            .and_then(|t| effect_piece(&t, tag.part));
        let moved = then.map(|t| piece.at - t.at);
        piece = between(then, piece, alpha);
        if let Some(leaves) = piece.trail {
            emits.add(leaves.emit(
                swing::key(swing::source::EFFECT, tag.slot, leaves.part),
                &piece,
                moved,
                eye,
            ));
            if leaves.only {
                *vis = Visibility::Hidden;
                continue;
            }
        }
        let (want_mesh, want_skin) = (look.dressed(&piece), look.dressed_material(&piece));
        *vis = Visibility::Inherited;
        if mesh.0 != want_mesh {
            mesh.0 = want_mesh;
        }
        if mat.0 != want_skin {
            mat.0 = want_skin;
        }
        // A heart faces the camera, turned about its own centre.
        if let (Some(e), Ok(cam)) = (effect, camera.single()) {
            if e.kind == sim::effects::EffectKind::Heart && tag.part < HEART_FACES {
                let centre = heart_centre(fx3(e.pos));
                let to = cam.translation() - centre;
                // Tilted up at the camera as well: seen from over the
                // shoulder, a heart standing upright had its point squashed.
                let flat = (to.x * to.x + to.z * to.z).sqrt();
                let face = Quat::from_rotation_y(to.x.atan2(to.z))
                    * Quat::from_rotation_x(-to.y.atan2(flat));
                piece.at = centre + face * (piece.at - centre);
                piece.turn = face * piece.turn;
            }
        }
        tf.translation = piece.at;
        tf.scale = piece.scale;
        tf.rotation = piece.turn;
    }
}

type AuraPieces<'w, 's> = Query<
    'w,
    's,
    (
        &'static AuraMesh,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut Mesh3d,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
>;

/// Put a piece's mesh, material and place on an entity.
fn show_piece(
    look: &EffectLook,
    piece: Piece,
    tf: &mut Transform,
    vis: &mut Visibility,
    mesh: &mut Mesh3d,
    mat: &mut MeshMaterial3d<StandardMaterial>,
) {
    let (want_mesh, want_skin) = (look.dressed(&piece), look.dressed_material(&piece));
    *vis = Visibility::Inherited;
    if mesh.0 != want_mesh {
        mesh.0 = want_mesh;
    }
    if mat.0 != want_skin {
        mat.0 = want_skin;
    }
    tf.translation = piece.at;
    tf.scale = piece.scale;
    tf.rotation = piece.turn;
}

/// **Draw the glow off a Blood mage on her ground** (2026-10-18): the strong
/// version, seen. A ring of essence at her feet and a faint flame rising off
/// her body while she stands on it; swimming, a wider ripple on the surface
/// over her, and no flame.
fn place_auras(sim: Res<Sim>, look: Res<EffectLook>, mut meshes: AuraPieces) {
    for (tag, mut tf, mut vis, mut mesh, mut mat) in meshes.iter_mut() {
        let p = &sim.cur.players[tag.owner];
        let Some(piece) = aura_piece(p, sim.cur.frame, tag.part) else {
            *vis = Visibility::Hidden;
            continue;
        };
        show_piece(&look, piece, &mut tf, &mut vis, &mut mesh, &mut mat);
    }
}

fn aura_piece(p: &sim::state::Player, frame: u32, part: usize) -> Option<Piece> {
    if !p.on_blood || p.health <= 0 {
        return None;
    }
    let feet = fx3(p.pos);
    let body = sim::tuning::body_radius().to_f32_for_render();
    let t = frame as f32 / 60.0;
    if p.swimming() {
        let swell = 1.0 + 0.12 * (t * 5.0).sin();
        return (part == 0).then(|| {
            standing(
                Shape::Column,
                Skin::Essence(4),
                feet,
                body * 2.2 * swell,
                0.09,
                0.13,
            )
        });
    }
    let tall = sim::tuning::body_height().to_f32_for_render();
    match part {
        0 => Some(standing(
            Shape::Column,
            Skin::Essence(3),
            feet,
            body * 1.6,
            0.09,
            0.11,
        )),
        _ => {
            let sway = Vec3::new((t * 1.3).sin(), 0.0, (t * 1.7).cos()) * 0.08;
            let mut flame = limb(
                Shape::Spike,
                feet + Vec3::Y * 0.05,
                feet + sway + Vec3::Y * tall * 1.15,
                body * 1.5,
                Vec3::X,
            );
            flame.skin = Skin::Essence(2);
            Some(flame)
        }
    }
}

/// One drawable piece of one effect: which shape, which material, where, and
/// how big.
///
/// Shapes and skins by name rather than by asset handle, so the geometry is a
/// pure function of simulation state that a test can check without a renderer.
/// What it is checking is that the picture agrees with the hit test, and that
/// is worth being able to assert: a spike drawn wider than it drains is a
/// promise the game does not keep.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Piece {
    shape: Shape,
    skin: Skin,
    at: Vec3,
    scale: Vec3,
    /// How the unit shape is turned before it is scaled into place: the
    /// identity for everything that stands upright, a limb's direction for a
    /// thrall's arms and legs, and the way a disk faces or a bolt runs.
    turn: Quat,
    /// The trail it leaves behind it as it moves, if it leaves one
    /// (`swing.rs`): drawn from where the piece is drawn, so it is on it.
    trail: Option<Leaves>,
}

/// **What a moving piece leaves behind it**: which trail of its owner's it
/// is, how wide, how bright and what colour, how far back it reaches, and in
/// which style. Where it is comes from the piece itself.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Leaves {
    part: u8,
    half: f32,
    /// The band's half-width as a direction in the arena, where it is fixed
    /// to the thing (a flat blade of light flying flat); `None` for a band
    /// worked out from the piece (`emit`).
    band: Option<Vec3>,
    glow: f32,
    tint: Color,
    window: f32,
    style: swing::Style,
    /// The trail is the whole of it: the piece says where, and is not drawn
    /// itself -- a blade spun so fast it is only its arc, a wave of light
    /// that is only its sweep.
    only: bool,
}

impl Leaves {
    /// A streak `half` wide either side of the line of flight, in `tint`,
    /// reaching `window` frames back: the trail behind something thrown.
    /// The same, with its band fixed to `band` (half of it, either side).
    fn flat(self, band: Vec3) -> Leaves {
        Leaves {
            band: Some(band),
            ..self
        }
    }

    fn streak(part: u8, half: f32, tint: Color, window: f32) -> Leaves {
        Leaves {
            part,
            half,
            band: None,
            glow: 1.0,
            tint,
            window,
            style: swing::Style::Streak,
            only: false,
        }
    }

    /// Where `piece` is drawn, as the newest place on its trail. A blade's
    /// band runs along its own long axis (the piece's `turn` of `Y`), half
    /// out from the middle each way; a streak's lies across the way it moved,
    /// square to the eye.
    fn emit(&self, key: u32, piece: &Piece, moved: Option<Vec3>, eye: Vec3) -> swing::Emit {
        let (inner, outer) = match (self.band, self.style) {
            (Some(band), _) => (piece.at - band, piece.at + band),
            (None, swing::Style::Blade) => {
                let along = piece.turn * Vec3::Y * self.half;
                (piece.at - along, piece.at + along)
            }
            (None, swing::Style::Streak) => {
                swing::across(piece.at, moved.unwrap_or(Vec3::ZERO), eye, self.half)
            }
        };
        swing::Emit {
            key,
            inner,
            outer,
            glow: self.glow,
            tint: swing::linear(self.tint),
            window: self.window,
            style: self.style,
        }
    }
}

impl Piece {
    /// The same piece, leaving `trail` behind it.
    fn leaving(self, trail: Leaves) -> Piece {
        Piece {
            trail: Some(trail),
            ..self
        }
    }
}

/// **A piece between two frames**: `then` (the frame before) blended `alpha`
/// of the way into `now`, so a thing in flight moves every picture rather than
/// once a frame -- and the trail it leaves, drawn from it, is smooth. Only
/// for the same shape near enough to be the same thing; anything else is
/// drawn where it is now.
fn between(then: Option<Piece>, now: Piece, alpha: f32) -> Piece {
    match then {
        Some(t) if t.shape == now.shape && t.at.distance(now.at) < 6.0 => Piece {
            at: t.at.lerp(now.at, alpha),
            scale: t.scale.lerp(now.scale, alpha),
            turn: t.turn.slerp(now.turn, alpha),
            ..now
        },
        _ => now,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// An upright cylinder: a pillar, a stone, the floor of a drain field.
    Column,
    /// A cone standing on its base. The black spike, and nothing else yet.
    Spike,
    /// A sphere, which is exactly what the hit test for a travelling effect is.
    Ball,
    /// A thin hoop, lying flat until it is turned: the rings round the Dual
    /// mage's sun-disk.
    Hoop,
    /// A fighter's limb, a torso and a head: what a thrall is built of, so it
    /// is made the way the fighters are.
    Limb,
    Block,
    Head,
    /// The Reaver's blades (`blades`), unit-sized, scaled uniformly.
    Shuriken,
    Shard,
    Crescent,
}

/// The Elementalist's wind -- the Air ball and the two drafts -- in air, or
/// in fire once it has touched some (`sim::effects::Effect::lit`): what a
/// burning ball is worth has to be readable before it reaches you.
fn wind_skin(effect: &sim::effects::Effect) -> Skin {
    if effect.lit() { Skin::Fire } else { Skin::Air }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Skin {
    Fire,
    Blood,
    /// A thrall: translucent blood.
    Thrall,
    /// The Black vines: translucent black.
    Vines,
    /// A heart of hers: crimson burning from inside.
    Heart,
    /// The Reaver's shadow-work: near black, and lit from inside just enough to
    /// be visible against the floor it is usually crossing.
    Shade,
    /// A pool of essence, at a step of solidity from faint to nearly solid.
    Essence(u8),
    /// The Dual mage's two forces, and the only pair of skins in the game that
    /// exist to be told apart from each other rather than from the floor.
    ///
    /// Everything she throws is made of one of them, and which one is a thing
    /// the player sets deliberately and has to be able to read back. The bar on
    /// the HUD says it in a place nobody is looking during a fight; the thing
    /// standing in the arena says it where they are.
    Light,
    Dark,
    /// The sun-disk's plate, and the bolt's black core and white rim, and the
    /// storm cloud's smoke and wisps: see the fields of `EffectLook`.
    Sun,
    Bolt,
    BoltRim,
    Storm,
    Wisp,
    /// Broken ground: the stones' own material, so rough terrain reads as
    /// earth rather than as an effect.
    Stone,
    /// The Reaver's blades: dark violet steel, her violet at the edges.
    Steel,
    /// The same steel, solid: what stands -- a shard, a crescent.
    Forged,
    /// Moving air: the beam's pale, barely-there material, so a draft or a
    /// ring of air is seen without hiding what is in it.
    Air,
    /// The Champion's light: his gold, streaming -- the crescent he throws,
    /// the javelin, the cuts. His own, not the Dual mage's white: two
    /// classes' throws in one colour were one class's.
    Gilt,
}

impl EffectLook {
    fn mesh(&self, shape: Shape) -> Handle<Mesh> {
        match shape {
            Shape::Column => self.column.clone(),
            Shape::Spike => self.spike.clone(),
            Shape::Ball => self.ball.clone(),
            Shape::Hoop => self.hoop.clone(),
            Shape::Limb => self.limb.clone(),
            Shape::Block => self.block.clone(),
            Shape::Head => self.head.clone(),
            Shape::Shuriken => self.shuriken.clone(),
            Shape::Shard => self.shard.clone(),
            Shape::Crescent => self.crescent.clone(),
        }
    }

    /// The mesh a piece is drawn with: its shape, read with what it is made
    /// of -- a column of fire is a flame, a flat column of air a disc of wind
    /// (`fx`). Every one fills the room its shape does, so the hit test's
    /// size is still the drawing's.
    fn dressed(&self, piece: &Piece) -> Handle<Mesh> {
        let flat = piece.scale.y < 0.3 * piece.scale.x;
        match (piece.shape, piece.skin) {
            (Shape::Column, Skin::Fire) => self.flame.clone(),
            (Shape::Ball, Skin::Fire) => self.fireball.clone(),
            (Shape::Column, Skin::Stone) => self.rubble.clone(),
            (Shape::Column, Skin::Air) if flat => self.winddisc.clone(),
            (Shape::Column, Skin::Air) => self.whirl.clone(),
            (Shape::Ball, Skin::Air) => self.windball.clone(),
            (Shape::Spike, Skin::Blood | Skin::Vines) => self.thorn.clone(),
            (Shape::Spike, Skin::Essence(_)) => self.tongue.clone(),
            (Shape::Limb, Skin::Vines | Skin::Essence(_)) => self.vine.clone(),
            (Shape::Ball | Shape::Column, Skin::Essence(_) | Skin::Blood | Skin::Vines) if flat => {
                self.puddle.clone()
            }
            (Shape::Ball, Skin::Essence(_) | Skin::Blood | Skin::Heart) => self.droplet.clone(),
            (Shape::Column, Skin::Gilt) if flat => self.crescent_wave.clone(),
            (Shape::Column, Skin::Gilt) => self.comet.clone(),
            (Shape::Ball, Skin::Light) => self.star.clone(),
            (Shape::Ball, Skin::Dark) => self.fireball.clone(),
            (Shape::Ball, Skin::Storm | Skin::Wisp) => self.cloud.clone(),
            (Shape::Column, Skin::Light | Skin::Dark) if flat => self.disc.clone(),
            (Shape::Column, Skin::Sun) if flat => self.sunburst.clone(),
            (Shape::Spike, Skin::Light | Skin::Dark) => self.point.clone(),
            (shape, _) => self.mesh(shape),
        }
    }

    /// The material a piece is drawn in: its skin's, except a mark on the
    /// floor in either of the Dual mage's forces, which is a sigil.
    fn dressed_material(&self, piece: &Piece) -> Handle<StandardMaterial> {
        let flat = piece.scale.y < 0.3 * piece.scale.x;
        match (piece.shape, piece.skin) {
            (Shape::Column, Skin::Light) if flat => self.sigil_light.clone(),
            (Shape::Column, Skin::Dark) if flat => self.sigil_dark.clone(),
            _ => self.material(piece.skin),
        }
    }

    fn material(&self, skin: Skin) -> Handle<StandardMaterial> {
        match skin {
            Skin::Fire => self.fire.clone(),
            Skin::Blood => self.blood.clone(),
            Skin::Thrall => self.thrall.clone(),
            Skin::Vines => self.vines.clone(),
            Skin::Heart => self.heart.clone(),
            Skin::Shade => self.shade.clone(),
            Skin::Essence(step) => self.essence[(step as usize).min(ESSENCE_STEPS - 1)].clone(),
            Skin::Light => self.light.clone(),
            Skin::Dark => self.dark.clone(),
            Skin::Sun => self.sun.clone(),
            Skin::Bolt => self.bolt.clone(),
            Skin::BoltRim => self.bolt_rim.clone(),
            Skin::Storm => self.storm.clone(),
            Skin::Wisp => self.wisp.clone(),
            Skin::Stone => self.stone.clone(),
            Skin::Air => self.air.clone(),
            Skin::Gilt => self.gilt.clone(),
            Skin::Steel => self.steel.clone(),
            Skin::Forged => self.forged.clone(),
        }
    }
}

/// An upright shape standing on the ground at `at`, between two heights.
fn standing(shape: Shape, skin: Skin, at: Vec3, radius: f32, bottom: f32, top: f32) -> Piece {
    let height = (top - bottom).max(0.01);
    Piece {
        turn: Quat::IDENTITY,
        shape,
        skin,
        at: at + Vec3::Y * (bottom + height * 0.5),
        scale: Vec3::new(radius * 2.0, height, radius * 2.0),
        trail: None,
    }
}

/// A flat disk of `radius` and `thick`ness whose face looks along `facing`.
fn plate(skin: Skin, at: Vec3, facing: Vec3, radius: f32, thick: f32) -> Piece {
    Piece {
        shape: Shape::Column,
        skin,
        at,
        scale: Vec3::new(radius * 2.0, thick, radius * 2.0),
        turn: Quat::from_rotation_arc(Vec3::Y, facing.normalize_or(Vec3::Y)),
        trail: None,
    }
}

/// A hoop of `radius` round `at`, its face looking along `facing`.
fn hoop(skin: Skin, at: Vec3, facing: Vec3, radius: f32) -> Piece {
    Piece {
        shape: Shape::Hoop,
        skin,
        at,
        scale: Vec3::splat(radius),
        turn: Quat::from_rotation_arc(Vec3::Y, facing.normalize_or(Vec3::Y)),
        trail: None,
    }
}

/// A rod of `radius` from `a` to `b`.
fn rod(skin: Skin, a: Vec3, b: Vec3, radius: f32) -> Piece {
    let along = b - a;
    let length = along.length().max(0.01);
    Piece {
        shape: Shape::Column,
        skin,
        at: (a + b) * 0.5,
        scale: Vec3::new(radius * 2.0, length, radius * 2.0),
        turn: Quat::from_rotation_arc(Vec3::Y, along / length),
        trail: None,
    }
}

/// A ball in the air, which is exactly what the hit test for a travelling
/// effect is -- see `World::inside`.
fn floating(at: Vec3, radius: f32) -> Piece {
    floating_in(Skin::Blood, at, radius)
}

/// The same, in a skin of your choosing.
fn floating_in(skin: Skin, at: Vec3, radius: f32) -> Piece {
    Piece {
        turn: Quat::IDENTITY,
        shape: Shape::Ball,
        skin,
        at,
        scale: Vec3::splat(radius * 2.0),
        trail: None,
    }
}

fn effect_piece(effect: &sim::effects::Effect, part: usize) -> Option<Piece> {
    use sim::effects::{EffectKind, GRASP_ARMS, LOTUS_BLADES};

    let at = fx3(effect.pos);
    match effect.kind {
        EffectKind::FirePillar if part < 2 => {
            let (base, column) = effect.pillar_volumes();
            let it = if part == 0 { base } else { column };
            Some(standing(
                Shape::Column,
                Skin::Fire,
                at,
                it.radius.to_f32_for_render(),
                it.bottom.to_f32_for_render(),
                it.top.to_f32_for_render(),
            ))
        }
        // The exact same two volumes a fire pillar draws, at its own live,
        // moving centre instead of `effect.pos` -- see
        // `sim::effects::Effect::tornado_pos`. Not a shape of its own: this is
        // the point of folding the tornado into the same `Effect` a pillar is.
        EffectKind::FireTornado if part < 2 => {
            let at = fx3(effect.tornado_pos());
            let (base, column) = effect.pillar_volumes();
            let it = if part == 0 { base } else { column };
            Some(standing(
                Shape::Column,
                Skin::Fire,
                at,
                it.radius.to_f32_for_render(),
                it.bottom.to_f32_for_render(),
                it.top.to_f32_for_render(),
            ))
        }
        // The field it drains in, and the spike standing in the middle of it.
        // Two pieces because they say two different things: the disc is where
        // the drain reaches, and the spike is the thing you can see from across
        // the arena and decide to walk around. It was drawn as the disc alone
        // for a while, which is a hazard you find out about by standing in it.
        // A bare spike is just a spike. Only an eruption -- one that came up
        // out of a pool -- has the skirt: the disc of floor it came up
        // through, which is the disc it hit across.
        EffectKind::BlackSpike if part == 0 && !effect.erupted() => None,
        // The third tier's field (2026-10-12): the disc it hits across, and a
        // ring of spikes inside it that come up with each wave and sink back
        // between them -- a field of them, not one fat spike.
        EffectKind::BlackSpike
            if effect.banked == sim::state::SPIKE_FIELD && part <= FIELD_SPIKES =>
        {
            let volume = effect.spike_volume();
            let radius = volume.radius.to_f32_for_render();
            if part == 0 {
                return Some(standing(Shape::Column, Skin::Blood, at, radius, 0.0, 0.12));
            }
            let every = sim::tuning::spike_wave_every().max(1) as f32;
            let into = ((effect.age.max(1) - 1) as f32 % every) / every;
            let up = (1.0 - into).powi(2);
            let turn = part as f32 / FIELD_SPIKES as f32 * std::f32::consts::TAU;
            let spot = at + Vec3::new(turn.cos(), 0.0, turn.sin()) * radius * 0.6;
            Some(standing(
                Shape::Spike,
                Skin::Blood,
                spot,
                radius * 0.12,
                0.0,
                volume.top.to_f32_for_render() * up.max(0.05),
            ))
        }
        EffectKind::BlackSpike if part < 2 => {
            let volume = effect.spike_volume();
            let radius = volume.radius.to_f32_for_render();
            let height = volume.top.to_f32_for_render();
            // An eruption's skirt spreads out from it (2026-10-19), as wide as
            // what it reaches this frame; the spike in its middle is its full
            // size from the first, so a chain reads as one blast setting off
            // the next rather than a carpet appearing.
            let full = effect.reach.to_f32_for_render();
            Some(if part == 0 {
                standing(Shape::Column, Skin::Blood, at, radius, 0.0, 0.12)
            } else {
                standing(
                    Shape::Spike,
                    Skin::Blood,
                    at,
                    full * SPIKE_WAIST,
                    0.0,
                    height,
                )
            })
        }
        EffectKind::Bloodletter if part == 0 => Some(floating(
            fx3(effect.blade_at()),
            effect.field_radius().to_f32_for_render(),
        )),
        // The bolt: a ball, which is exactly what its hit test is.
        EffectKind::Haemorrhage if part == 0 => Some(floating(
            fx3(effect.bolt_at()),
            effect.field_radius().to_f32_for_render(),
        )),
        EffectKind::Grasp if part < GRASP_ARMS => Some(floating(
            fx3(effect.arm_at(part)),
            effect.field_radius().to_f32_for_render(),
        )),
        // One **disc** per blade, drawn around the shadow's live position
        // rather than the spot the move was thrown at -- which is what makes
        // them visibly chase it home. Same shape as the hit test, as
        // everywhere: a short wide cylinder lying in the flower's plane, which
        // is a shuriken thrown flat. It was a ball, and a ball of that radius
        // read as a beach ball rather than a blade.
        // The light Lance's burst: a ball where the line ran out, and how big
        // it is is the whole reading of how deep she was when she threw it.
        // `field_radius` already carries that, which is the point of it being
        // on the effect -- the thing drawn is the thing the hit test uses.
        EffectKind::LanceBurst if part == 0 => Some(floating_in(
            Skin::Light,
            at,
            effect.field_radius().to_f32_for_render(),
        )),
        // The dark Lance, as a **chain of beads** from her hand to the far end
        // of the throw -- or, once it has hold of somebody, from her hand to
        // them. A `Piece` has a place and a size and no direction, so a line has
        // to be drawn as a row of things; it turned out to be the better read
        // anyway, because a chain visibly stretches as the two of them part and
        // visibly is not there the frame the leash breaks. The bead size is
        // presentation, unlike the burst above: nothing is hit by the line
        // after its one pass, so there is no volume here to be honest about.
        // The Quake's patch: a low disc of broken ground at the radius the
        // shake reads, shivering while it shakes.
        EffectKind::Quake if part == 0 => Some(standing(
            Shape::Column,
            Skin::Stone,
            at + Vec3::X * ((effect.age % 3) as f32 - 1.0) * QUAKE_SHIVER,
            effect.field_radius().to_f32_for_render(),
            0.0,
            ROUGH_HEIGHT,
        )),
        // A draft: the column the bodies are tested in, drawn as air.
        EffectKind::Updraft | EffectKind::Downdraft if part == 0 => {
            let slab = effect.draft_volume();
            Some(standing(
                Shape::Column,
                wind_skin(effect),
                at,
                slab.radius.to_f32_for_render(),
                slab.bottom.to_f32_for_render(),
                slab.top.to_f32_for_render(),
            ))
        }
        // The air ring: a flat disc at the radius the shove reached.
        EffectKind::AirRing if part == 0 => Some(standing(
            Shape::Column,
            Skin::Air,
            at,
            effect.field_radius().to_f32_for_render(),
            0.0,
            ROUGH_HEIGHT,
        )),
        // The fire ring: pieces of fire around its live radius, each the
        // ring's own width.
        EffectKind::FireRing if part < sim::effects::RING_PIECES => Some(floating_in(
            Skin::Fire,
            fx3(effect.ring_piece(part))
                + Vec3::Y * sim::tuning::fire_ring_width().to_f32_for_render(),
            sim::tuning::fire_ring_width().to_f32_for_render(),
        )),
        // Fissure's surge: broken ground heaving at its front, at the width it
        // reaches -- the stone it carries is drawn as a stone.
        EffectKind::Surge if part == 0 => Some(standing(
            Shape::Column,
            Skin::Stone,
            fx3(effect.surge_front()) + Vec3::X * ((effect.age % 3) as f32 - 1.0) * QUAKE_SHIVER,
            effect.field_radius().to_f32_for_render(),
            0.0,
            ROUGH_HEIGHT * 2.0,
        )),
        // Rough terrain: a row of low slabs along the line the crack ran, at
        // the half-width the slow reads. See `Effect::rough_bead`.
        EffectKind::Rough if part < sim::effects::ROUGH_BEADS => Some(standing(
            Shape::Column,
            Skin::Stone,
            fx3(effect.rough_bead(part)),
            sim::tuning::rough_width().to_f32_for_render(),
            0.0,
            ROUGH_HEIGHT,
        )),
        // A cloud of embers: the ball the hit test reads, in the fire skin,
        // hanging where the shot burst -- or half-sunk into the floor where it
        // burst on the ground, which is what a burning patch looks like.
        EffectKind::Embers if part == 0 => Some(floating_in(
            Skin::Fire,
            at,
            effect.ember_volume().radius.to_f32_for_render(),
        )),
        // The Air ball: the ball the carry reads, at the size it is now --
        // growing while she holds it, shrinking as it rolls.
        EffectKind::AirBall if part == 0 => Some(floating_in(
            wind_skin(effect),
            fx3(effect.ball_middle()),
            effect.field_radius().to_f32_for_render(),
        )),
        // The Fire carpet: a stream of flames carried along its line from the
        // near end to the far one, so the fire is seen being pushed outward.
        // Each flame is no wider than the strip that burns. See
        // `Effect::carpet_flame`.
        EffectKind::FireCarpet if part < sim::effects::CARPET_FLAMES => {
            let (flame, radius) = effect.carpet_flame(part);
            Some(floating_in(
                Skin::Fire,
                fx3(flame),
                radius.to_f32_for_render(),
            ))
        }
        // The Fire fountain: the wash standing where she took off, and over
        // its first frames the burst -- a ball of fire at the move's radius,
        // the size it hits at.
        EffectKind::Fountain if part == 0 => {
            let slab = effect.fountain_volume();
            Some(standing(
                Shape::Column,
                Skin::Fire,
                at,
                slab.radius.to_f32_for_render(),
                slab.bottom.to_f32_for_render(),
                slab.top.to_f32_for_render(),
            ))
        }
        EffectKind::Fountain if part == 1 && effect.age <= sim::tuning::fountain_burst() => Some(
            floating_in(Skin::Fire, at, effect.source().radius.to_f32_for_render()),
        ),
        // **A ring of thorns** (2026-10-18): Thornfall's landing, and the
        // burst she arrives in from a teleport. Thorns burst out of the
        // floor round the edge of what it hits, leaning outward, over a
        // faint disc as wide as the hit -- a ball the size of the hit swallowed
        // the camera that had just landed in the middle of it. They are up by
        // the frame it hits and sink back over the rest of its life.
        EffectKind::Nova if part < THORN_RING => {
            let r = effect.field_radius().to_f32_for_render();
            let life = effect.life.max(1) as f32;
            let t = effect.age as f32 / life;
            let up = (effect.age as f32 / 2.0).min(1.0) * (1.0 - t * t);
            if part == 0 {
                return Some(standing(Shape::Column, Skin::Essence(2), at, r, 0.09, 0.11));
            }
            let k = part - 1;
            let a = k as f32 / (THORN_RING - 1) as f32 * std::f32::consts::TAU
                + wobble(seed_of(effect.pos)) * 6.0;
            let out = Vec3::new(a.cos(), 0.0, a.sin());
            let long = (0.7 + 0.5 * wobble(seed_of(effect.pos).wrapping_add(k as u32 * 13))) * up;
            let foot = at + out * (r * 0.8) + Vec3::Y * 0.05;
            let tip = foot + (out * 0.45 + Vec3::Y).normalize() * (1.5 * long).max(0.01);
            let mut thorn = limb(Shape::Spike, foot, tip, 0.3 * up.max(0.1), out);
            thorn.skin = Skin::Blood;
            Some(thorn)
        }
        // The Nail: a ball where its hit test is, in the Reaver's near-black,
        // because a nail is black.
        EffectKind::Nail if part == 0 => Some(floating_in(
            Skin::Shade,
            fx3(effect.bolt_at()),
            effect.field_radius().to_f32_for_render(),
        )),
        // The Dual mage's spellbook (2026-10-12). Every piece is drawn at the
        // volume it tests, and the rest is spectacle round it.
        //
        // **The twin stars**: one dark and one light at the size they strike,
        // each with a tail of smaller motes where it has just been.
        // Two stars on their orbits, each with the streak it leaves.
        EffectKind::Stars if part < 2 => {
            let r = effect.field_radius().to_f32_for_render();
            let (skin, kind) = if part == 0 {
                (Skin::Dark, sim::moves::dual::DARK_AUTO)
            } else {
                (Skin::Light, sim::moves::dual::LIGHT_AUTO)
            };
            Some(
                floating_in(skin, fx3(effect.star_at(part)), r).leaving(Leaves::streak(
                    part as u8,
                    r * 0.8,
                    orb_tint(kind),
                    8.0,
                )),
            )
        }
        // The Hemoplague (2026-10-12): a faint disc of blood on the floor at
        // exactly the radius it hurts in, and wisps of it drifting round
        // inside the column, rising and falling -- a cloud, and the edge of
        // it is the decision it offers.
        EffectKind::Plague if part == 0 => Some(standing(
            Shape::Column,
            Skin::Essence(1),
            at,
            effect.field_radius().to_f32_for_render(),
            0.0,
            0.1,
        )),
        EffectKind::Plague if part <= PLAGUE_WISPS => {
            let radius = effect.field_radius().to_f32_for_render();
            let height = sim::tuning::plague_height().to_f32_for_render();
            let t = effect.age as f32 / 60.0;
            let k = part as f32;
            let turn = k / PLAGUE_WISPS as f32 * std::f32::consts::TAU + t * 0.4;
            let out = radius * (0.35 + 0.45 * ((k * 1.7 + t * 0.6).sin() * 0.5 + 0.5));
            let rise = height * (0.25 + 0.6 * ((k * 2.3 + t * 0.9).sin() * 0.5 + 0.5));
            Some(floating_in(
                Skin::Essence(2),
                at + Vec3::new(turn.cos() * out, rise, turn.sin() * out),
                radius * 0.18,
            ))
        }
        // **The black sun**: its ground, the sun itself hanging over the
        // middle, and motes spiralling in from the rim to it. On its last
        // frames it collapses: the sun swells and the ground goes.
        EffectKind::Abyss if part < 10 => {
            let r = effect.field_radius().to_f32_for_render();
            let life = effect.life.max(1) as f32;
            let through = (effect.age as f32 / life).clamp(0.0, 1.0);
            let collapse = ((through - 0.8) / 0.2).clamp(0.0, 1.0);
            match part {
                0 => Some(standing(
                    Shape::Column,
                    Skin::Dark,
                    at,
                    r * (1.0 - collapse),
                    0.0,
                    0.12,
                )),
                1 => Some(floating_in(
                    Skin::Dark,
                    at + Vec3::Y * (1.2 + r * 0.3),
                    r * (0.22 + 0.25 * collapse),
                )),
                _ => {
                    let k = (part - 2) as f32;
                    let spiral = (effect.age as f32 * 0.02 + k / 8.0).fract();
                    let turn = k / 8.0 * std::f32::consts::TAU + effect.age as f32 * 0.09;
                    let reach = r * (1.0 - spiral);
                    Some(floating_in(
                        Skin::Dark,
                        at + Vec3::new(
                            turn.cos() * reach,
                            0.2 + spiral * (1.0 + r * 0.3),
                            turn.sin() * reach,
                        ),
                        0.16,
                    ))
                }
            }
        }
        // **A sigil**: a thin disc of light at the radius that wards and
        // burns, and a ring of light turning slowly round its rim. It thins
        // over its last second so leaving it is not a surprise.
        EffectKind::Sigil if part < 9 => {
            let r = effect.field_radius().to_f32_for_render();
            let left = effect.life.saturating_sub(effect.age) as f32;
            let fade = (left / 60.0).clamp(0.25, 1.0);
            if part == 0 {
                return Some(standing(Shape::Column, Skin::Light, at, r, 0.0, 0.04));
            }
            let k = (part - 1) as f32;
            let turn = k / 8.0 * std::f32::consts::TAU + effect.age as f32 * 0.02;
            Some(floating_in(
                Skin::Light,
                at + Vec3::new(turn.cos() * r, 0.25, turn.sin() * r),
                0.16 * fade,
            ))
        }
        // **A burst**. Judgement's outer columns stand where they strike,
        // thinning as they fade; Apotheosis is both forces bursting outward
        // from her; a Shadowstep's arrival and a Phase's flash are a ball of
        // their force, big on the frame it hits and shrinking after.
        EffectKind::Burst => {
            let life = effect.life.max(1) as f32;
            let left = 1.0 - (effect.age as f32 / life).clamp(0.0, 1.0);
            let r = effect.field_radius().to_f32_for_render();
            if effect.slot == sim::moves::dual::JUDGEMENT {
                let centre = effect.burst_at(part)?;
                return Some(standing(
                    Shape::Column,
                    Skin::Light,
                    fx3(centre),
                    r * (0.4 + 0.6 * left),
                    0.0,
                    18.0,
                ));
            }
            if effect.slot == sim::effects::APOTHEOSIS {
                let out = 1.0 - left;
                return match part {
                    0 => Some(floating_in(Skin::Light, at, r * (0.3 + 0.7 * out))),
                    1 => Some(floating_in(Skin::Dark, at, r * (0.2 + 0.5 * out))),
                    k if k < 12 => {
                        let turn = (k - 2) as f32 / 10.0 * std::f32::consts::TAU;
                        let skin = if k % 2 == 0 { Skin::Dark } else { Skin::Light };
                        Some(floating_in(
                            skin,
                            at + Vec3::new(turn.cos(), 0.0, turn.sin()) * r * out,
                            0.35,
                        ))
                    }
                    _ => None,
                };
            }
            // **A bolt of Chain lightning** (2026-10-12): black lightning
            // from where it leapt -- the cloud, or the last body -- to where
            // it struck, jagged and re-jagged every frame, and a flash where
            // it landed.
            // **A bolt of Chain lightning** (2026-10-13): black lightning with
            // a white edge, from where it leapt -- the cloud, or the last body
            // -- to where it struck, in jagged strokes re-struck every other
            // frame, and a black flash ringed in white where it landed. Each
            // stroke is a black rod inside a fatter white one of which only
            // the far side is drawn, so the white shows round the black.
            if effect.slot == sim::moves::dual::CHAIN_LIGHTNING {
                const STROKES: usize = BOLT_STROKES;
                let from = fx3(effect.home);
                let hash = |n: f32| ((n * 12.9898).sin() * 43758.547).fract() - 0.5;
                let strike = (effect.age / 2) as f32 * 7.0;
                let span = (at - from).length();
                let kink = |i: usize| {
                    if i == 0 || i == STROKES {
                        return Vec3::ZERO;
                    }
                    let seed = i as f32 + strike;
                    Vec3::new(hash(seed), hash(seed + 3.1), hash(seed + 5.7))
                        * (span * 0.08).min(0.9)
                };
                let point = |i: usize| from.lerp(at, i as f32 / STROKES as f32) + kink(i);
                let core = 0.03 + 0.025 * left;
                return match part {
                    k if k < STROKES => Some(rod(Skin::Bolt, point(k), point(k + 1), core)),
                    k if k < 2 * STROKES => {
                        let k = k - STROKES;
                        Some(rod(Skin::BoltRim, point(k), point(k + 1), core + 0.025))
                    }
                    k if k == 2 * STROKES => {
                        Some(floating_in(Skin::Bolt, at, r * 0.45 * (0.3 + 0.7 * left)))
                    }
                    k if k == 2 * STROKES + 1 => Some(Piece {
                        shape: Shape::Ball,
                        skin: Skin::BoltRim,
                        at,
                        scale: Vec3::splat(r * 1.1 * (0.3 + 0.7 * left)),
                        turn: Quat::IDENTITY,
                        trail: None,
                    }),
                    _ => None,
                };
            }
            // **The Flurry dash's disk breaking** (2026-10-13): shards of
            // both forces flung out across the disk's face as she runs
            // through it, carried on the way she was going.
            if effect.slot == sim::moves::dual::FLURRY_DASH {
                let out = 1.0 - left;
                let flat = Vec3::new(
                    at.x - effect.home.x.to_f32_for_render(),
                    0.0,
                    at.z - effect.home.z.to_f32_for_render(),
                )
                .normalize_or(Vec3::X);
                let side = Vec3::new(-flat.z, 0.0, flat.x);
                return (part < 12).then(|| {
                    let th = part as f32 / 12.0 * std::f32::consts::TAU + 0.3;
                    let outward = (side * th.cos() + Vec3::Y * th.sin()).normalize_or(Vec3::Y);
                    let skin = if part % 2 == 0 {
                        Skin::Dark
                    } else {
                        Skin::Light
                    };
                    Piece {
                        shape: Shape::Spike,
                        skin,
                        at: at + outward * r * (0.6 + 1.8 * out) + flat * out * 1.6
                            - Vec3::Y * out * out,
                        scale: Vec3::new(0.3, 0.7, 0.3) * (0.25 + 0.75 * left),
                        turn: Quat::from_rotation_arc(Vec3::Y, outward),
                        trail: None,
                    }
                });
            }
            let skin = if effect.slot == sim::moves::dual::SHADOWSTEP {
                Skin::Dark
            } else {
                Skin::Light
            };
            (part == 0).then(|| floating_in(skin, at, r * (0.4 + 0.6 * left)))
        }
        EffectKind::Tether if part < sim::effects::TETHER_BEADS => Some(floating_in(
            Skin::Dark,
            fx3(effect.tether_bead(part)),
            sim::tuning::body_radius().to_f32_for_render() * TETHER_BEAD,
        )),
        // Judgement's field: a wide, shallow disc of light on the floor. Drawn
        // at exactly the radius that burns, because walking to the edge of it
        // is the decision it offers.
        // A pool: a shadowy figure the size the hit test reads, standing where
        // the blood was spilled, shrinking as it drains and drawn more solid
        // the more essence is left in it -- so the figure on the floor is
        // the heal it is worth.
        EffectKind::Pool if part < STAIN_PARTS => Some(stain_piece(effect, at, part)),
        // **A heart** (2026-10-18): a knot of essence beating at waist
        // height over the ground it holds, a flame of it standing up through
        // the knot, and roots of it running down and out into the floor --
        // all of it swelling with each beat and smaller the less blood is
        // left in it.
        EffectKind::Heart if part < HEART_PARTS => Some(heart_piece(effect, at, part)),
        // A thrall: a little hulking gremlin of translucent blood -- a
        // hunched barrel of a body, a low head pushed forward with two horns
        // swept back, heavy shoulders, long bent arms ending in claws near
        // the floor, short bent legs -- as tall as its stain is worth,
        // shrinking as it bites and drains; the golem the same, bigger. It
        // faces the way it last went (`Effect::dir`), and its arms and legs
        // swing as it goes.
        // **On her ground it is bigger and burns brighter** (2026-10-18):
        // the thrall that hits harder there is seen to.
        EffectKind::Thrall if part < GREMLIN_PARTS => {
            let piece = gremlin_piece(effect, at, part);
            Some(if effect.struck.has(sim::state::ON_GROUND) {
                Piece {
                    at: at + (piece.at - at) * THRALL_ON_GROUND,
                    scale: piece.scale * THRALL_ON_GROUND,
                    skin: Skin::Essence((ESSENCE_STEPS - 1) as u8),
                    ..piece
                }
            } else {
                piece
            })
        }
        // The Black vines: a tangle of twisting vines arching out of the
        // floor from the middle to the front, barbs standing off them, over
        // a faint dark wash -- see `vine_piece`.
        EffectKind::Vines if part < VINE_PARTS => Some(vine_piece(effect, at, part)),
        // The Rising reap's sweep: balls of blood flung out along the cone,
        // the far ones the widest.
        EffectKind::Cascade
            if effect.slot != sim::moves::blood::CASCADE && part < sim::effects::TETHER_BEADS =>
        {
            let t = (part as f32 + 0.5) / sim::effects::TETHER_BEADS as f32;
            let life = effect.life.max(1) as f32;
            let out = (effect.age as f32 / life * 2.0).min(1.0);
            let reach = effect.reach.to_f32_for_render();
            let dir = fx3(effect.dir).normalize_or_zero();
            Some(floating_in(
                Skin::Blood,
                at + dir * reach * t * out + Vec3::Y * 1.0,
                0.2 + reach * 0.12 * t,
            ))
        }
        EffectKind::JudgementField if part == 0 => Some(standing(
            Shape::Column,
            Skin::Light,
            at,
            effect.field_radius().to_f32_for_render(),
            0.0,
            0.14,
        )),
        // The Reaver's (2026-10-10, 2026-10-12). The Whirl is blades spun round
        // the shadow's middle, live -- flat discs, the lotus's shuriken -- on a
        // ring inside the ball it cuts with, and the burst the same round her
        // own waist, flying out to its edge; the thrown blades are where they
        // are; the Plunge's ring is pieces of shadow on its live radius, the
        // fire ring's shape.
        EffectKind::ShadowWhirl | EffectKind::ShadowBurst if part < SPUN_BLADES => {
            let fast = if effect.kind == EffectKind::ShadowWhirl {
                0.55
            } else {
                0.9
            };
            // The burst flies out to its disc's edge over its short life; the
            // whirl spins inside its ball.
            let r = if effect.kind == EffectKind::ShadowBurst {
                let out = (effect.age as f32 / effect.life.max(1) as f32 * 3.0).min(1.0);
                effect.field_radius().to_f32_for_render() * (0.35 + 0.55 * out)
            } else {
                effect.field_radius().to_f32_for_render() * 0.75
            };
            let turn =
                effect.age as f32 * fast + part as f32 * std::f32::consts::TAU / SPUN_BLADES as f32;
            let mid = at + Vec3::Y * sim::tuning::body_height().to_f32_for_render() * 0.5;
            // Her blade, spinning, and the arc it cuts behind it.
            Some(
                shuriken(
                    mid + Vec3::new(turn.cos(), 0.0, turn.sin()) * r,
                    SPUN_BLADE,
                    effect.age,
                    part,
                )
                .leaving(Leaves::streak(
                    part as u8,
                    SPUN_BLADE * 0.5,
                    SHADOW_TINT,
                    SPUN_TRAIL,
                )),
            )
        }
        EffectKind::ShadowFan if part < sim::moves::reaver::blades(effect.slot) => {
            let r = effect.field_radius().to_f32_for_render();
            Some(
                shuriken(fx3(effect.fan_blade_at(part)), r * 2.0, effect.age, part)
                    .leaving(Leaves::streak(part as u8, r * 0.8, SHADOW_TINT, 4.0)),
            )
        }
        // Shards of shadow driven up out of the floor where the ring runs,
        // each its own height and lean, as wide as the ring cuts.
        EffectKind::ShadowRing if part < sim::effects::RING_PIECES => {
            let width = sim::tuning::shadow_ring_width().to_f32_for_render();
            let at = fx3(effect.ring_piece(part));
            let seed = (part as f32 * 2.399).sin() * 0.5 + 0.5;
            let rise = (effect.age as f32 / 4.0).min(1.0);
            let tall = width * (1.5 + 1.1 * seed) * rise;
            let out = (at - fx3(effect.pos)).with_y(0.0).normalize_or(Vec3::X);
            Some(Piece {
                shape: Shape::Shard,
                skin: Skin::Forged,
                at,
                scale: Vec3::new(width * 1.6, tall, width * 1.6),
                turn: Quat::from_rotation_arc(
                    Vec3::Y,
                    (Vec3::Y + out * (0.25 + 0.3 * seed)).normalize(),
                ) * Quat::from_rotation_y(out.x.atan2(out.z) + std::f32::consts::FRAC_PI_2),
                trail: None,
            })
        }
        // **The Champion's full charges** (exploration 0015). The crescent:
        // a wide flat sweep of light, as wide as the hit test where its head
        // is, trailing back along its line.
        EffectKind::Crescent if part == 0 => {
            let r = effect.field_radius().to_f32_for_render();
            let back = fx3(effect.dir).normalize_or_zero();
            // A moon thrown back-first along its flight (`fx::crescent`),
            // sweeping light behind it: the band across its width at the
            // head, the hit test's, swept back along its flight.
            let flat = Vec3::new(back.x, 0.0, back.z).normalize_or(Vec3::X);
            let side = fx3(effect.dir).cross(Vec3::Y).normalize_or(Vec3::X);
            Some(Piece {
                shape: Shape::Column,
                skin: Skin::Gilt,
                at: fx3(effect.head()),
                scale: Vec3::new(r * 2.0, 0.12, r * 2.0),
                turn: Quat::from_rotation_arc(Vec3::X, flat),
                trail: Some(Leaves::streak(0, r, LIGHT_TINT, 5.0).flat(side * r)),
            })
        }
        // The shockwave: broken ground on its live radius, standing where
        // it stopped for as long as it lies there.
        EffectKind::Shockwave if part < sim::effects::RING_PIECES => {
            let height = sim::tuning::body_height().to_f32_for_render() * 0.35;
            Some(standing(
                Shape::Column,
                Skin::Stone,
                fx3(effect.ring_piece(part)),
                0.45,
                0.0,
                height,
            ))
        }
        // The cuts: small and faint along the line he ran while they hang,
        // large on the frames they land.
        EffectKind::Cuts if part < sim::effects::TETHER_BEADS => {
            let t = (part as f32 + 0.5) / sim::effects::TETHER_BEADS as f32;
            let end = fx3(effect.cuts_end());
            let r = effect.field_radius().to_f32_for_render();
            let landing = effect.age >= sim::tuning::cuts_delay();
            let size = if landing { r * 0.8 } else { r * 0.2 };
            // Each a slash across the line he ran.
            let run = Vec3::new(end.x - at.x, 0.0, end.z - at.z).normalize_or(Vec3::X);
            Some(Piece {
                shape: Shape::Column,
                skin: Skin::Gilt,
                at: at.lerp(end, t),
                scale: Vec3::new(size * 2.0, 0.08, size * 2.0),
                turn: Quat::from_rotation_arc(Vec3::X, run),
                trail: None,
            })
        }
        // The javelin: one spear of light, its head at the javelin's and its
        // tail streaming back a spear's length, and the streak it leaves.
        EffectKind::Javelin if part == 0 => {
            let head = fx3(effect.head());
            let back = fx3(effect.dir).normalize_or_zero() * (3.0 * 0.7);
            Some(
                rod(Skin::Gilt, head - back, head, 0.12)
                    .leaving(Leaves::streak(0, 0.1, LIGHT_TINT, 4.0)),
            )
        }
        EffectKind::Reaping => reaping_cut(effect, part),
        EffectKind::GuillotineLotus if part < LOTUS_BLADES => Some(
            shuriken(
                fx3(effect.lotus_at(part, effect.pos)),
                effect.field_radius().to_f32_for_render() * 2.0,
                effect.age,
                part,
            )
            .leaving(Leaves::streak(
                part as u8,
                effect.field_radius().to_f32_for_render() * 0.8,
                SHADOW_TINT,
                4.0,
            )),
        ),
        _ => None,
    }
}

/// One of the Reaver's throwing stars (`blades::shuriken`), `size` across
/// its tips, lying flat and spinning on the simulation's clock: a turn every
/// third of a second, each blade of a set started at its own angle.
fn shuriken(at: Vec3, size: f32, age: u16, part: usize) -> Piece {
    let spin = age as f32 * 0.33 + part as f32 * 1.3;
    Piece {
        shape: Shape::Shuriken,
        skin: Skin::Steel,
        at,
        scale: Vec3::splat(size),
        turn: Quat::from_rotation_y(-spin),
        trail: None,
    }
}

/// The colour of the trail a shadow blade leaves: the Reaver's dusk violet.
const SHADOW_TINT: Color = Color::srgb(0.48, 0.40, 0.90);

/// The colour of the trail the Champion's light leaves: his finishers'.
const LIGHT_TINT: Color = Color::srgb(1.0, 0.95, 0.80);

/// How many frames back a spun blade's streak reaches: most of a turn's
/// worth at the burst's speed, a third of one at the whirl's.
const SPUN_TRAIL: f32 = 4.0;

/// How many blades the Whirl and the arc's cut are drawn as, spinning round
/// the shadow. Presentation, not a rule: the hit test is the ball they spin
/// inside.
const SPUN_BLADES: usize = 6;

/// How wide one of those blades is drawn. Presentation, like the count.
const SPUN_BLADE: f32 = 0.45;

/// How far the Quake's patch shivers from side to side while it shakes.
/// Presentation, not a rule: the hit test does not move.
const QUAKE_SHIVER: f32 = 0.04;

/// How tall broken ground is drawn. Presentation, not a rule: the slow reads
/// a body on the floor, and a slab this low is a texture rather than a wall.
const ROUGH_HEIGHT: f32 = 0.12;

/// How big one bead of a tether is against a body.
///
/// Presentation, not a rule -- see the tether's arm of [`effect_piece`]. Small:
/// what has to be read is the line and the thing on the end of it, and a chain
/// of beads the size of fists would hide the second.
const TETHER_BEAD: f32 = 0.3;

/// How fat the spike is against the field it stands in.
///
/// Presentation, not a rule: the field's radius is where the drain reaches, and
/// a cone that wide would be a tent rather than a spike. The disc underneath is
/// what tells you where the edge is.
const SPIKE_WAIST: f32 = 0.35;

/// How many spikes a third-tier Black spike's field is drawn as, round its
/// disc. Presentation: the hit test is the disc.
const FIELD_SPIKES: usize = 7;

/// How many wisps a Hemoplague is drawn as, drifting inside its column.
/// Presentation: the hit test is the column.
const PLAGUE_WISPS: usize = 8;

/// A stain's pieces: a few flat blobs for its body, three roots of three
/// segments each, and the standing essence -- a flame, its core and two
/// motes. See `stain_piece`.
const STAIN_PARTS: usize = 16;

/// The Black vines' pieces: a dark wash, six vines of three segments and a
/// dozen barbs. See `vine_piece`.
const VINE_PARTS: usize = 31;

/// A number in `0..1` from a seed, the same every frame: what makes each
/// stain and each tangle of vines its own shape without anything random.
fn wobble(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65536.0
}

/// A seed from where a thing was put down, so its shape stays put.
fn seed_of(at: sim::V3) -> u32 {
    (at.x.raw() as u32).wrapping_mul(73_856_093) ^ (at.z.raw() as u32).wrapping_mul(19_349_663)
}

/// Where a root or a vine has got to: from `from`, `segment` steps of
/// `length` along a heading that started at `turn` (radians) and bent a
/// little each step by its own wobble.
fn wander(from: Vec3, turn: f32, segment: usize, length: f32, seed: u32) -> (Vec3, f32) {
    let mut at = from;
    let mut heading = turn;
    for k in 0..segment {
        at += Vec3::new(heading.cos(), 0.0, heading.sin()) * length;
        heading += (wobble(seed.wrapping_add(k as u32 * 31)) - 0.5) * 1.6;
    }
    (at, heading)
}

/// One piece of a stain: **a pool of essence, not of blood** (2026-10-14)
/// -- two or three flat lobes off-centre from each other, three roots
/// creeping out of it in crooked steps, and **standing out of it, the
/// essence itself**: a slow, swaying flame as tall as the column the
/// crosshair and the spikes find it by (`Effect::pool_height`), a brighter
/// core in it and two motes rising off it. Its size is the hit test's
/// (`Effect::pool_radius`); drawn more solid the more essence is left. A
/// lit stain -- going up in a moment -- swells and throbs.
fn stain_piece(effect: &sim::effects::Effect, at: Vec3, part: usize) -> Piece {
    let r = effect.pool_radius().to_f32_for_render().max(0.12) * 1.3;
    let step = (effect.pool_share().to_f32_for_render() * (ESSENCE_STEPS - 1) as f32).round() as u8;
    let skin = Skin::Essence(step);
    let seed = seed_of(effect.home);
    let lit = effect.life != sim::state::UNLIT;
    let throb = if lit {
        1.15 + 0.15 * (effect.age as f32 * 0.9).sin()
    } else {
        1.0
    };
    let r = r * throb;
    let flat = |centre: Vec3, w: f32, d: f32, yaw: f32| Piece {
        shape: Shape::Ball,
        skin,
        // Over her ground's surface (`claimed::LIFT`), or it hides them.
        at: centre + Vec3::Y * 0.09,
        scale: Vec3::new(w, 0.04, d),
        turn: Quat::from_rotation_y(yaw),
        trail: None,
    };
    if part >= 12 {
        // The standing essence: a column of it, swaying, as tall as the
        // pool stands in the hit test.
        let tall = effect.pool_height().to_f32_for_render().max(0.3) * throb;
        let t = effect.age as f32 / 60.0 + wobble(seed) * 10.0;
        let sway = Vec3::new((t * 1.3).sin(), 0.0, (t * 1.7).cos()) * r * 0.12;
        let faint = Skin::Essence(step.min(2));
        let limb = |shape, from, to, thick, ahead| {
            let mut piece = limb(shape, from, to, thick, ahead);
            piece.skin = faint;
            piece
        };
        return match part {
            // A tapering flame of it, its tip swaying, and a brighter
            // tongue inside it leaning the other way.
            12 => limb(
                Shape::Spike,
                at + Vec3::Y * 0.02,
                at + sway * 2.0 + Vec3::Y * tall,
                r * 0.8,
                Vec3::X,
            ),
            13 => {
                let lean = Vec3::new(-sway.z, 0.0, sway.x) * 2.5;
                let foot = at + Vec3::new(sway.x, 0.0, -sway.z) * 1.5;
                let mut tongue = limb(
                    Shape::Spike,
                    foot + Vec3::Y * 0.02,
                    foot + lean + Vec3::Y * tall * 0.7,
                    r * 0.35,
                    Vec3::X,
                );
                tongue.skin = skin;
                tongue
            }
            _ => {
                let k = (part - 14) as f32;
                let up = (t * 0.45 + k * 0.5).fract();
                let turn = wobble(seed.wrapping_add(part as u32 * 3)) * std::f32::consts::TAU + t;
                let out = Vec3::new(turn.cos(), 0.0, turn.sin()) * r * 0.35;
                floating_in(
                    faint,
                    at + out + Vec3::Y * tall * (0.3 + 0.9 * up),
                    r * 0.12 * (1.0 - up * 0.6),
                )
            }
        };
    }
    if part < 3 {
        let k = part as u32;
        let a = wobble(seed.wrapping_add(k * 7)) * std::f32::consts::TAU;
        let off = if part == 0 {
            0.0
        } else {
            r * 0.45 * wobble(seed.wrapping_add(k * 13))
        };
        let size = if part == 0 {
            1.5
        } else {
            0.9 + 0.5 * wobble(seed.wrapping_add(k * 17))
        };
        return flat(
            at + Vec3::new(a.cos(), 0.0, a.sin()) * off,
            r * size,
            r * size * (0.55 + 0.35 * wobble(seed.wrapping_add(k * 19))),
            a,
        );
    }
    let root = (part - 3) / 3;
    let segment = (part - 3) % 3;
    let rseed = seed.wrapping_add(root as u32 * 101);
    let turn = (root as f32 / 3.0 + wobble(rseed) * 0.3) * std::f32::consts::TAU;
    let length = r * (0.55 - 0.1 * segment as f32);
    let start = at + Vec3::new(turn.cos(), 0.0, turn.sin()) * r * 0.5;
    let (from, heading) = wander(start, turn, segment, r * 0.55, rseed);
    let to = from + Vec3::new(heading.cos(), 0.0, heading.sin()) * length;
    let thick = r * (0.22 - 0.06 * segment as f32);
    let mut piece = limb(Shape::Limb, from, to, thick, Vec3::Y);
    piece.skin = skin;
    piece.at += Vec3::Y * 0.02;
    piece.scale.x = thick;
    piece.scale.z = thick * 0.35;
    piece
}

/// One piece of the Black vines: **a mass of vines with barbs off them** --
/// a faint dark wash out to where the vines have reached, six vines snaking
/// out of the middle in crooked arching segments, and barbs standing off
/// the vines. Each tangle is its own (`seed_of`), and it grows out to the
/// front over `tuning::vines_grow`.
fn vine_piece(effect: &sim::effects::Effect, at: Vec3, part: usize) -> Piece {
    const VINES: usize = 6;
    const SEGMENTS: usize = 3;
    let grow = (effect.age as f32 / sim::tuning::vines_grow().max(1) as f32).min(1.0);
    let reach = effect.reach.to_f32_for_render().max(0.5) * grow.max(0.05);
    let seed = seed_of(effect.pos);
    if part == 0 {
        return standing(Shape::Column, Skin::Vines, at, reach, 0.07, 0.1);
    }
    let length = reach / SEGMENTS as f32;
    let vine_at = |vine: usize, segment: usize| {
        let vseed = seed.wrapping_add(vine as u32 * 977);
        let turn = (vine as f32 / VINES as f32 + wobble(vseed) * 0.12) * std::f32::consts::TAU;
        let (from, heading) = wander(at, turn, segment, length, vseed);
        let to = from + Vec3::new(heading.cos(), 0.0, heading.sin()) * length;
        // Arching out of the floor and back into it.
        let rise = |k: usize| match k {
            0 => 0.05,
            k if k == SEGMENTS => 0.08,
            _ => 0.25 + 0.25 * wobble(vseed.wrapping_add(k as u32 * 5)),
        };
        (
            from + Vec3::Y * rise(segment) * grow,
            to + Vec3::Y * rise(segment + 1) * grow,
        )
    };
    let index = part - 1;
    if index < VINES * SEGMENTS {
        let (vine, segment) = (index / SEGMENTS, index % SEGMENTS);
        let (from, to) = vine_at(vine, segment);
        let thick = (0.24 - 0.05 * segment as f32) * grow.max(0.3);
        let mut piece = limb(Shape::Limb, from, to, thick, Vec3::Y);
        piece.skin = Skin::Vines;
        return piece;
    }
    // A barb: off a point along a segment, out to the side and up.
    let barb = index - VINES * SEGMENTS;
    let vine = barb % VINES;
    let segment = barb / VINES % SEGMENTS;
    let (from, to) = vine_at(vine, segment);
    let bseed = seed.wrapping_add(barb as u32 * 211);
    let along = from.lerp(to, 0.35 + 0.4 * wobble(bseed));
    let dir = (to - from).normalize_or(Vec3::X);
    let side = Vec3::Y.cross(dir).normalize_or(Vec3::Z) * if barb % 2 == 0 { 1.0 } else { -1.0 };
    let tip = along + (side * 0.7 + Vec3::Y * 0.6 + dir * 0.2).normalize() * 0.45 * grow;
    let mut piece = limb(Shape::Spike, along, tip, 0.13, dir);
    piece.skin = Skin::Vines;
    piece
}

/// How many pieces a ring of thorns is drawn in: the disc and its thorns.
const THORN_RING: usize = 13;

/// How many pieces a heart is drawn in: two lobes and a point, two vessels,
/// the ring round it, the pillar of light and three roots.
const HEART_PARTS: usize = 10;

/// **A piece of a heart** (2026-10-18): a heart's shape, not a heart's
/// anatomy -- two lobes over a point, and two short vessels rising out of
/// its top -- in a crimson that burns from inside, hung over the ground it
/// holds. It beats twice a second (a heavy beat and a light one), turns
/// slowly so its shape reads from every side, and has a ring of essence
/// turning round it and a faint pillar of light through it, so it is seen
/// from across a fight and never mistaken for a stain. Roots run down from
/// it into the floor. It is smaller the less blood is left in it: how long it
/// will stand, and how many thralls it has left in it, read off its size.
/// How much bigger than its first draft a heart is drawn: at the other end
/// of an arena it has to read as a heart, not a speck.
const HEART_SIZE: f32 = 1.35;

/// Where a heart standing at `at` hangs.
fn heart_centre(at: Vec3) -> Vec3 {
    at + Vec3::Y * 1.75
}

/// Of a heart's parts, the ones that are the heart itself (the lobes, the
/// point and the vessels) and are turned to face the camera.
const HEART_FACES: usize = 5;

fn heart_piece(effect: &sim::effects::Effect, at: Vec3, part: usize) -> Piece {
    let full = sim::tuning::heart_least().max(1) as f32 * 3.0;
    let worth = 0.6 + 0.4 * (effect.banked.max(0) as f32 / full).clamp(0.0, 1.0);
    let t = effect.age as f32 / 60.0;
    // Lub-dub: a heavy beat, a light one, a rest.
    let phase = (t * 1.1).fract();
    let lub = (-((phase - 0.05) * 22.0).powi(2)).exp();
    let dub = (-((phase - 0.3) * 22.0).powi(2)).exp() * 0.5;
    let beat = 1.0 + 0.16 * (lub + dub);
    let size = HEART_SIZE * worth * beat;
    let centre = heart_centre(at);
    // Its lobes lie along x and its face looks along z; `place_effects`
    // turns it to face the camera, so the outline is a heart from wherever
    // it is seen (turning freely, it was two balls one behind the other as
    // often as not). A slow sway keeps it alive.
    let turn = Quat::from_rotation_y((t * 0.7 + wobble(seed_of(effect.pos)) * 6.0).sin() * 0.25);
    let side = turn * Vec3::X;
    let lobe_r = 0.24 * size;
    match part {
        // The lobes, side by side and a little up.
        0 | 1 => {
            let s = if part == 0 { -1.0 } else { 1.0 };
            floating_in(
                Skin::Heart,
                centre + side * (0.19 * size * s) + Vec3::Y * (0.08 * size),
                lobe_r,
            )
        }
        // The point, a cone hung tip down under the lobes.
        2 => Piece {
            shape: Shape::Spike,
            skin: Skin::Heart,
            at: centre - Vec3::Y * (0.16 * size),
            scale: Vec3::new(0.78 * size, 0.62 * size, 0.42 * size),
            turn: turn * Quat::from_rotation_x(std::f32::consts::PI),
            trail: None,
        },
        // Two vessels out of its top, leaning apart.
        3 | 4 => {
            let s = if part == 3 { -1.0 } else { 1.0 };
            let foot = centre + side * (0.07 * size * s) + Vec3::Y * (0.22 * size);
            let tip = foot + side * (0.09 * size * s) + Vec3::Y * (0.24 * size);
            let mut vessel = rod(Skin::Heart, foot, tip, 0.05 * size);
            vessel.skin = Skin::Heart;
            vessel
        }
        // A ring of essence turning round it, tilted, pulsing with the beat.
        5 => {
            let tilt = Quat::from_rotation_y(t * 1.4) * Quat::from_rotation_x(0.35);
            hoop(
                Skin::Essence((ESSENCE_STEPS - 1) as u8),
                centre,
                tilt * Vec3::Y,
                0.62 * size,
            )
        }
        // A faint pillar of light through it, floor to well over it.
        6 => standing(Shape::Column, Skin::Essence(0), at, 0.18 * worth, 0.09, 3.4),
        // Roots down into the floor it holds.
        k => {
            let a = (k - 7) as f32 / 3.0 * std::f32::consts::TAU + t * 0.1;
            let out = Vec3::new(a.cos(), 0.0, a.sin());
            let reach = sim::tuning::heart_ground().to_f32_for_render() * 0.45;
            let mut root = limb(
                Shape::Spike,
                centre - Vec3::Y * (0.5 * size),
                at + out * reach + Vec3::Y * 0.09,
                0.06 * size,
                out,
            );
            root.skin = Skin::Essence(3);
            root
        }
    }
}

/// How many pieces a thrall's gremlin is drawn in.
const GREMLIN_PARTS: usize = 24;

/// A piece of a thrall, `size` across (width, along `up`, depth), its own up
/// along `up` and its front toward `ahead`.
fn framed(shape: Shape, centre: Vec3, up: Vec3, ahead: Vec3, size: Vec3) -> Piece {
    let y = up.normalize_or(Vec3::Y);
    let z = (ahead - y * ahead.dot(y)).normalize_or(Vec3::Z);
    let x = y.cross(z);
    Piece {
        shape,
        skin: Skin::Thrall,
        at: centre,
        scale: size,
        turn: Quat::from_mat3(&Mat3::from_cols(x, y, z)),
        trail: None,
    }
}

/// **Five of her** (2026-10-19): how many copies of her the Reaping throws
/// round him -- one a mark, at most.
const REAP_COPIES: usize = 5;

/// How far from his middle each copy stands, in metres, round a fighter: a
/// step inside the Executioner's reach, so the overhead comes down on him.
/// Round anything bigger, the same step out from all of it.
const COPY_RING: f32 = 1.4;

/// The copies' clock, in frames since the cash-in. They step in out of the
/// air over `COPY_IN`, play her Executioner squeezed so it strikes at
/// `COPY_STRIKE` and is active for `COPY_ACTIVE`, recover until
/// `COPY_SETTLED`, and then fold back into her one at a time, the first at
/// `COPY_FOLD` and each `COPY_FOLD_EVERY` after, each fold `COPY_FOLDING`
/// long.
const COPY_IN: f32 = 6.0;
const COPY_STRIKE: f32 = 14.0;
const COPY_ACTIVE: f32 = 4.0;
const COPY_SETTLED: f32 = 32.0;
const COPY_FOLD: f32 = 30.0;
const COPY_FOLD_EVERY: f32 = 4.0;
const COPY_FOLDING: f32 = 12.0;

/// The shadow body's own grey and see-through (`ShadowRoot`'s material):
/// the copies are the shadow, five times.
const COPY_GREY: [f32; 3] = [0.20, 0.21, 0.26];
const COPY_SOLID: f32 = 0.45;

/// How many copies a Reaping throws.
fn copies() -> usize {
    (sim::tuning::mark_cap() as usize).min(REAP_COPIES)
}

/// Where copy `copy` of a Reaping stands -- its feet, on the floor under
/// where the blow met -- and the way it faces: on a ring round that place,
/// spaced so that with her they are six round it and none of them stands
/// where she came from, every one looking in.
///
/// **They are always her size, and spread out round what they cut**
/// (2026-10-20, from play: copies grown to a Ridgeback's back "look weird").
/// The sim keeps how far the struck thing reaches out from the place across
/// the floor (`reach`: a body's radius, a leg's, half a back), and the ring
/// is that plus the step a fighter is cut from -- so round a fighter they
/// stand close, round a leg a little wider, and round a whole animal they
/// stand off on the floor all the way round it.
fn copy_spot(effect: &sim::effects::Effect, copy: usize) -> (Vec3, Vec3) {
    let met = fx3(effect.pos);
    let floor = fx3(effect.home).y;
    let to_her = {
        let d = fx3(effect.dir);
        -Vec3::new(d.x, 0.0, d.z).try_normalize().unwrap_or(Vec3::Z)
    };
    let body = sim::tuning::body_radius().to_f32_for_render();
    let ring = effect.reach.to_f32_for_render() + (COPY_RING - body);
    let turn = (copy + 1) as f32 / (copies() + 1) as f32 * std::f32::consts::TAU;
    let out = Quat::from_rotation_y(turn) * to_her;
    (Vec3::new(met.x, floor, met.z) + out * ring, -out)
}

/// What copy `copy` is doing `t` frames into a Reaping: her Executioner,
/// squeezed onto the copies' clock, held at its last frame of recovery
/// until it folds back into her.
fn copy_action(t: f32) -> sim::state::Action {
    use sim::state::Action;
    let kind = sim::moves::reaver::EXECUTIONER;
    let m = sim::moves::get(sim::Class::ShadowReaver, kind);
    // Frames left of a phase `frames` long, `t` of the way from `from` to
    // `to` on the copies' clock.
    let left = |from: f32, to: f32, frames: u16| {
        let gone = ((t - from) / (to - from)).clamp(0.0, 1.0);
        ((frames as f32 * (1.0 - gone)).ceil() as u16).clamp(1, frames.max(1))
    };
    if t < COPY_STRIKE {
        Action::Startup {
            kind,
            left: left(0.0, COPY_STRIKE, m.startup),
        }
    } else if t < COPY_STRIKE + COPY_ACTIVE {
        Action::Active {
            kind,
            left: left(COPY_STRIKE, COPY_STRIKE + COPY_ACTIVE, m.active),
        }
    } else {
        Action::Recovery {
            kind,
            left: left(COPY_STRIKE + COPY_ACTIVE, COPY_SETTLED, m.recovery),
        }
    }
}

/// How far copy `copy` has folded back into her `t` frames in: 0 standing
/// on its spot, 1 gone into her.
fn copy_folded(t: f32, copy: usize) -> f32 {
    let from = COPY_FOLD + copy as f32 * COPY_FOLD_EVERY;
    ((t - from) / COPY_FOLDING).clamp(0.0, 1.0)
}

/// How solid copy `copy` is `t` frames in: stepping in out of the air, the
/// shadow's own see-through while it strikes, and thinning as it reaches her.
fn copy_solid(t: f32, copy: usize) -> f32 {
    let arrived = (t / COPY_IN).clamp(0.0, 1.0);
    let folded = copy_folded(t, copy);
    COPY_SOLID * arrived * (1.0 - folded * folded)
}

/// The cuts the copies leave in him: one dark blade of shadow a copy,
/// across his middle on the line it struck from, flashing open on the frames
/// the overheads land and closing again. Presentation: the Reaping hits
/// nothing.
fn reaping_cut(effect: &sim::effects::Effect, part: usize) -> Option<Piece> {
    if part >= copies() {
        return None;
    }
    let t = effect.age as f32;
    let open = ((t - COPY_STRIKE + 1.0) / 2.0).clamp(0.0, 1.0)
        * (1.0 - ((t - COPY_STRIKE - COPY_ACTIVE) / 8.0).clamp(0.0, 1.0));
    if open <= 0.0 {
        return None;
    }
    let (_, facing) = copy_spot(effect, part);
    let tall = sim::tuning::body_height().to_f32_for_render();
    // At the place the blow met, a little toward the copy's side, down the
    // line its overhead fell along.
    let at = fx3(effect.pos) - facing * 0.15;
    let down = (Vec3::Y * 1.0 - facing * 0.35).normalize();
    // A crescent of shadow, the length of the overhead that left it.
    let mut cut = framed(
        Shape::Crescent,
        at,
        down,
        facing.cross(Vec3::Y).normalize_or(Vec3::X),
        Vec3::new((0.6 + 0.6 * open) * tall / 1.8, tall * 0.55 * open, 1.0),
    );
    cut.skin = Skin::Forged;
    Some(cut)
}

/// **Put the five copies of her where the Reaping says they are** (2026-10-19).
///
/// Posed the way her shadow is -- the same skeleton, the same solver, the
/// same pure pose function -- holding her Executioner on the copies' own
/// clock, which is the Reaping's age: in the snapshot, so a rollback puts
/// them back exactly where they were. Folding home, each slides into her
/// live position and thins away.
#[allow(clippy::type_complexity)]
fn place_copies(
    sim: Res<Sim>,
    looks: Res<CopyLooks>,
    mut joints: ResMut<Joints>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut roots: Query<(&CopyRoot, &mut Transform, &mut Visibility), Without<CopyPart>>,
    mut parts: Query<(&CopyPart, &mut Transform), Without<CopyRoot>>,
) {
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let alpha = sim.clock.alpha();
    let skeleton = skeleton_for(sim::Class::ShadowReaver);
    // Each Reaver's youngest Reaping, if she has one going.
    let mut reaping: [Option<sim::effects::Effect>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    for e in sim.cur.effects.iter().flatten() {
        if e.kind != sim::effects::EffectKind::Reaping {
            continue;
        }
        let slot = &mut reaping[(e.owner as usize).min(MAX_PLAYERS - 1)];
        if slot.is_none_or(|old| e.age < old.age) {
            *slot = Some(*e);
        }
    }
    let mut skins: [[Option<view::skeleton::Skin>; REAP_COPIES]; MAX_PLAYERS] =
        [[None; REAP_COPIES]; MAX_PLAYERS];
    for (root, mut tf, mut seen) in roots.iter_mut() {
        let (owner, copy) = (root.owner, root.copy);
        let shown = reaping[owner].filter(|_| copy < copies()).and_then(|e| {
            let t = (e.age as f32 + alpha).min(e.life as f32);
            let solid = copy_solid(t, copy);
            (solid > 0.01).then_some((e, t, solid))
        });
        joints.copy_solid[owner][copy] = shown.map_or(0.0, |(_, _, solid)| solid);
        let Some((effect, t, solid)) = shown else {
            if *seen != Visibility::Hidden {
                *seen = Visibility::Hidden;
            }
            continue;
        };
        *seen = Visibility::Inherited;
        let (spot, facing) = copy_spot(&effect, copy);
        // Stepping in from a little further out, and folding home to her --
        // wherever she is, a creature's back included.
        let arrived = (t / COPY_IN).clamp(0.0, 1.0);
        let spot = spot - facing * 0.6 * (1.0 - arrived) * (1.0 - arrived);
        let her = Vec3::from(frame.players[owner].pos);
        let fold = copy_folded(t, copy);
        let fold = fold * fold;
        tf.translation = spot.lerp(her, fold);
        tf.rotation = Quat::from_rotation_y(facing.x.atan2(facing.z));
        tf.scale = Vec3::splat(1.0 - 0.3 * fold);
        if let Some(look) = looks.0.get(owner).map(|l| &l[copy]) {
            let want = solid;
            if materials
                .get(look)
                .is_some_and(|m| (m.base_color.alpha() - want).abs() > 0.01)
            {
                if let Some(m) = materials.get_mut(look) {
                    m.base_color = Color::srgba(COPY_GREY[0], COPY_GREY[1], COPY_GREY[2], want);
                }
            }
        }
        let mut input = PoseInput::of(&frame.players[owner], sim::Class::ShadowReaver, &frame);
        input.action = copy_action(t);
        input.grounded = true;
        input.crouching = false;
        input.floating = false;
        input.speed = 0.0;
        input.travel = [0.0, 0.0];
        input.eased_speed = 0.0;
        input.eased_travel = [0.0, 0.0];
        input.rise = 0.0;
        input.turn_rate = 0.0;
        input.aim_pitch = 0.0;
        input.shudder = 0.0;
        input.ledge = None;
        input.bind_pose = sim.bind_pose;
        skins[owner][copy] = Some(view::skeleton::solve(
            &skeleton,
            &view::play::pose_for(input),
        ));
    }
    joints.copies = skins;
    for (part, mut tf) in parts.iter_mut() {
        let Some(skin) = skins[part.owner][part.copy].as_ref() else {
            continue;
        };
        if part.ball {
            let at = skin.origin[part.joint.index()];
            tf.translation = Vec3::new(at[0], at[1], at[2]);
            tf.scale = Vec3::splat(ball_width(&skeleton, part.joint));
        } else {
            let (centre, rot) = skin.box_of(&skeleton, part.joint);
            let size = view::pose::part_size(&skeleton, part.joint);
            tf.translation = Vec3::new(centre[0], centre[1], centre[2]);
            tf.rotation = Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]);
            tf.scale = Vec3::new(size[0], size[1], size[2]);
        }
    }
}

/// A limb from `from` to `to`, `thick` across, facing `ahead`.
fn limb(shape: Shape, from: Vec3, to: Vec3, thick: f32, ahead: Vec3) -> Piece {
    let along = to - from;
    framed(
        shape,
        from + along * 0.5,
        along,
        ahead,
        Vec3::new(thick, along.length().max(0.01), thick),
    )
}

/// One piece of a thrall: **a fighter's body, crouched and gone brutish** --
/// the same rounded torso, head, limbs, hands and feet the fighters are drawn
/// with (`limb_form`), with a chest too big for it, a low head pushed forward
/// under two swept horns, long arms that end in claws near the floor, and
/// short bent legs. Every point is in the thrall's own frame -- ahead, out to
/// its side, up -- as a share of its height, so the golem is the same body,
/// bigger. It faces the way it last went (`Effect::dir`), and its arms and
/// legs swing as it goes.
fn gremlin_piece(effect: &sim::effects::Effect, at: Vec3, part: usize) -> Piece {
    let h = effect.thrall_height().to_f32_for_render().max(0.25);
    let ahead = {
        let d = fx3(effect.dir);
        let flat = Vec3::new(d.x, 0.0, d.z);
        if flat.length_squared() > 1e-6 {
            flat.normalize()
        } else {
            Vec3::X
        }
    };
    let side = Vec3::Y.cross(ahead).normalize_or(Vec3::Z);
    let stride = (effect.age as f32 * 0.45).sin();
    let bob = stride.abs() * 0.03;
    let p = |f: f32, s: f32, u: f32| at + (ahead * f + side * s + Vec3::Y * u) * h;
    // Left for the even part of a pair, right for the odd; each side swings
    // opposite its arm.
    let k = if part % 2 == 0 { 1.0 } else { -1.0 };
    let arm = stride * 0.12 * k;
    let leg = -stride * 0.10 * k;
    let lean = (ahead * 0.6 + Vec3::Y).normalize();
    match part {
        // The pelvis, low.
        0 => framed(
            Shape::Block,
            p(-0.04, 0.0, 0.30 + bob),
            Vec3::Y,
            ahead,
            Vec3::new(0.34, 0.14, 0.22) * h,
        ),
        // The chest: wide, deep, and leaning hard forward.
        1 => framed(
            Shape::Block,
            p(0.10, 0.0, 0.52 + bob),
            lean,
            ahead,
            Vec3::new(0.54, 0.34, 0.32) * h,
        ),
        // The head, low and pushed out in front of the shoulders.
        2 => framed(
            Shape::Head,
            p(0.34, 0.0, 0.66 + bob),
            Vec3::Y,
            ahead,
            Vec3::new(0.20, 0.18, 0.22) * h,
        ),
        // Two horns, swept back and up off the brow.
        3 | 4 => limb(
            Shape::Spike,
            p(0.36, 0.07 * k, 0.74 + bob),
            p(0.22, 0.15 * k, 0.94 + bob),
            0.07 * h,
            ahead,
        ),
        // Upper arms, from the shoulder down and out to the elbow.
        5 | 6 => limb(
            Shape::Limb,
            p(0.14, 0.30 * k, 0.64 + bob),
            p(0.22 + arm, 0.38 * k, 0.40),
            0.13 * h,
            ahead,
        ),
        // The elbows, a ball the arm's width so the bend shows.
        7 | 8 => {
            let mut b = floating_in(Skin::Thrall, p(0.22 + arm, 0.38 * k, 0.40), 0.07 * h);
            b.scale = Vec3::splat(0.13 * h);
            b
        }
        // Forearms, thicker than the arm above them, to the wrists.
        9 | 10 => limb(
            Shape::Limb,
            p(0.22 + arm, 0.38 * k, 0.40),
            p(0.34 + arm * 1.6, 0.36 * k, 0.15),
            0.15 * h,
            ahead,
        ),
        // Hands, big, knuckles nearly on the floor.
        11 | 12 => framed(
            Shape::Block,
            p(0.38 + arm * 1.6, 0.36 * k, 0.08),
            Vec3::Y,
            ahead,
            Vec3::new(0.15, 0.12, 0.18) * h,
        ),
        // Claws, out of the hands along the floor.
        13 | 14 => limb(
            Shape::Spike,
            p(0.44 + arm * 1.6, 0.36 * k, 0.06),
            p(0.62 + arm * 1.6, 0.37 * k, 0.01),
            0.08 * h,
            ahead,
        ),
        // Thighs, from the hip forward to bent knees.
        15 | 16 => limb(
            Shape::Limb,
            p(-0.04, 0.12 * k, 0.28 + bob),
            p(0.08 + leg, 0.15 * k, 0.15),
            0.12 * h,
            ahead,
        ),
        // Shins, back down to the ankles.
        17 | 18 => limb(
            Shape::Limb,
            p(0.08 + leg, 0.15 * k, 0.15),
            p(-0.02 + leg * 1.4, 0.15 * k, 0.04),
            0.10 * h,
            ahead,
        ),
        // Feet.
        19 | 20 => framed(
            Shape::Block,
            p(0.04 + leg * 1.4, 0.15 * k, 0.025),
            Vec3::Y,
            ahead,
            Vec3::new(0.11, 0.05, 0.20) * h,
        ),
        // Shoulders: a ball where each arm leaves the chest.
        21 | 22 => {
            let mut b = floating_in(Skin::Thrall, p(0.14, 0.29 * k, 0.64 + bob), 0.08 * h);
            b.scale = Vec3::splat(0.17 * h);
            b
        }
        // A ridge of spines down its back.
        _ => limb(
            Shape::Spike,
            p(0.0, 0.0, 0.66 + bob),
            p(-0.14, 0.0, 0.84 + bob),
            0.10 * h,
            ahead,
        ),
    }
}

/// Where the class mechanic sits in the world, if anywhere. A shield in hand
/// rides on the character and draws nothing; a thrown one, a placed shadow or a
/// raised structure all get a marker.
fn mechanic_world_pos(m: &sim::class::Mechanic) -> Option<sim::V3> {
    use sim::class::Mechanic;
    match m {
        Mechanic::Shield(s) => s.world_pos(),
        // The shadow is drawn as a body of its own rather than as a marker --
        // see `place_shadows` -- so it is not one of these.
        Mechanic::Shadow(_) => None,
        Mechanic::Structures(slots) => slots.iter().flatten().next().map(|s| s.at),
        _ => None,
    }
}

/// A shield in hand rides on the character; a thrown one sits in the world.
///
/// **Its weight is drawn on it**: it thickens and darkens as it fills, so the
/// person on the other side can read how loaded the wall is from across the
/// arena -- see `bulwark-v2.md`. Read straight off the snapshot's number, per
/// the overlay rule, so what you see is what a slam would spend.
fn place_shields(
    sim: Res<Sim>,
    hands: Res<ShieldHands>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shields: Query<(
        &ShieldMesh,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, look) in shields.iter_mut() {
        let player = &sim.cur.players[tag.0];
        let held = matches!(
            player.mechanic,
            sim::Mechanic::Shield(sim::state::Shield::Held { .. })
        );
        let full = sim::bulwark::fullness(player).to_f32_for_render();
        tf.scale = Vec3::new(1.0 + 0.2 * full, 1.0 + 0.2 * full, 1.0 + 1.5 * full);
        // Written only when it has moved: a material touched every frame is
        // a material rebuilt on the GPU every frame, for both shields, held
        // or not -- two thirds of a millisecond a frame, measured 2026-10-06,
        // for a colour that is the same one almost always.
        let empty = Vec3::new(0.92, 0.76, 0.38);
        let heavy = Vec3::new(0.30, 0.20, 0.10);
        let c = empty.lerp(heavy, full);
        let want = Color::srgb(c.x, c.y, c.z);
        if materials.get(&look.0).is_some_and(|m| m.base_color != want)
            && let Some(m) = materials.get_mut(&look.0)
        {
            m.base_color = want;
        }
        match mechanic_world_pos(&sim.cur.players[tag.0].mechanic) {
            // Thrown or planted: it is somewhere in the arena on its own.
            Some(pos) => {
                *vis = Visibility::Inherited;
                tf.rotation = Quat::IDENTITY;
                // Planted, it crowns the wall it has become -- the wall itself
                // is drawn by `place_structures`, from the same field that
                // stops people -- so it still reads as *his* shield.
                let lift = match player.mechanic {
                    sim::Mechanic::Shield(sim::state::Shield::Planted { pos, weight }) => {
                        sim::bulwark::wall(pos, weight).height().to_f32_for_render()
                    }
                    _ => 0.0,
                };
                tf.translation = Vec3::new(
                    pos.x.to_f32_for_render(),
                    pos.y.to_f32_for_render() + lift,
                    pos.z.to_f32_for_render(),
                );
            }
            // In hand, and now that the skeleton has hands it can be in one.
            // It follows the forearm, the way a strapped shield does, so
            // raising the guard raises the shield without anybody animating it.
            None if held => {
                let (at, rot) = hands.0[tag.0];
                *vis = Visibility::Inherited;
                tf.rotation = rot;
                tf.translation = at + rot * Vec3::new(0.0, -0.12, 0.06);
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Read real input, run whole fixed ticks, keep the previous snapshot.
// A Bevy system's parameter list *is* its dependency declaration: every entry
// is something the scheduler has to know this system touches. Splitting one to
// get under a count would split the system, which is the opposite of the point.
#[allow(clippy::too_many_arguments)]
fn tick_sim(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    // The meeting's clock: the wall's, not the game's, which is clamped to a
    // quarter-second a frame and so runs slow exactly when a page is
    // struggling -- and the meeting's timeouts are about the network.
    real: Res<Time<Real>>,
    mut look: ResMut<Look>,
    focus: Res<palette::UiFocus>,
    mut sim: ResMut<Sim>,
    mut show: ResMut<debug::ShowDebug>,
    mut scripted: ResMut<Scripted>,
    mut sparring: ResMut<Sparring>,
    mut trophies: ResMut<trophies::Trophies>,
    mut scape: ResMut<sound::Soundscape>,
    mut keeping: ResMut<bag::Keeping>,
) {
    // A won hunt is the player's trophy (world W1), whoever's machine it was
    // won on: each peer writes its own. Read from the world as it stands at
    // the top of the frame, which is the last one drawn.
    if trophies.notice(&sim.cur) {
        if let Some((_, temper)) = sim.cur.hunt_won() {
            eprintln!(
                "trophy: {} at temper {temper}",
                sim.cur
                    .hunted()
                    .iter()
                    .flatten()
                    .map(|s| s.get().name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    // **The valley's waystones read the journey**, and the journey learns
    // what this player has beaten from their trophies: one creature on the
    // wire a frame, the first the journey has not got, until it has them all
    // (`sim::input::Destination::Credit`). Both peers send theirs, so the
    // valley is open as far as the more travelled of you has been.
    if sim.travel == sim::input::Travel::NONE && sim.cur.valley.on {
        if let Some(s) = valley::credit_due(&sim.cur, &trophies, open_valley()) {
            sim.travel = sim::input::Travel::credit(s);
        }
    }
    // F10 too: a function key types nothing, and the moment you want dev mode
    // may well be with a pasted link still in the Esc menu's box.
    if keys.just_pressed(KeyCode::F10) {
        toggle_dev_mode(&mut show);
    }
    // Typing in a text field must not also pause the match or cycle the class.
    // F7 stays live regardless, since it is the way back out.
    if focus.keyboard {
        return;
    }
    // **Online is two people at one keyboard, with one of them somewhere
    // else.** Against a person the world changes only through ticks both
    // machines play, so anything that changes the fight -- pause, step,
    // restart, class, arena, creature -- is asked for on the wire
    // (`sim::input::Travel`) and both machines do it on the same frame. In
    // training the same keys act on the spot. Only stepping *back* stays
    // training's: it rewinds a history only this machine has.
    let training = !sim.driver.online();
    if keys.just_pressed(KeyCode::KeyP) {
        if training {
            sim.paused = !sim.paused;
        } else {
            sim.travel = sim::input::Travel::PAUSE;
        }
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        if training {
            sim.step_once = true;
            sim.paused = true;
        } else {
            sim.travel = sim::input::Travel::STEP;
        }
    }
    // **Back a frame.** The other half of stepping, and the half that makes it
    // worth having: anything that happens on a single frame has to be gone past
    // and returned to before you can see what it did. Local play only -- a peer
    // is not rewinding with you.
    if training && keys.just_pressed(KeyCode::BracketLeft) {
        sim.paused = true;
        if let Some(back) = sim.history.pop() {
            // `prev` as well, or the interpolator spends a frame drawing the
            // step we just undid.
            sim.prev = back.clone();
            sim.tape.rewind_to(back.frame);
            scape.rewound_to(back.frame);
            keeping.rewound_to(back.frame);
            sim.cur = back;
            // A rewind past the start of a rehearsal is a rewind out of it.
            if sim.rehearsing.is_some_and(|from| sim.cur.frame < from) {
                sim.rehearsing = None;
            }
        }
    }
    // **Play the double structure jump.** Not a hint at the timing -- the
    // timing, performed: two structures three frames apart and a jump nine
    // frames after the first, which is the only shape that gets a third takeoff
    // (see `docs/design/kits/elementalist.md`). It is a thirty-three
    // millisecond double tap and nobody hits it by hand, which is the whole
    // reason to be able to watch one.
    //
    // Dev only, because it takes the controls away from you.
    if training && dev_mode() && keys.just_pressed(KeyCode::KeyG) {
        let classes = [sim::Class::Elementalist, sim.cur.players[1].class];
        let w = World::with_classes(classes);
        sim.prev = w.clone();
        sim.tape = sim::replay::Tape::begin(&w, build());
        scape.restart(w.frame);
        keeping.restart();
        sim.cur = w;
        sim.history.clear();
        sim.rehearsing = Some(0);
        sim.paused = false;
    }
    if keys.just_pressed(KeyCode::F1) {
        // Toggled here rather than in the debug module so all input reading
        // stays in one place.
        show.0 = !show.0;
    }
    if keys.just_pressed(KeyCode::F2) {
        sim.bind_pose = !sim.bind_pose;
    }
    if keys.just_pressed(KeyCode::Tab) {
        // Cycle your own class. Restarts the fight, since a class change
        // mid-round would leave the mechanic in someone else's state -- and
        // asked for on the wire rather than done here, like the picker below,
        // so against a person both machines restart on the same frame.
        let me = sim.local_player();
        let next = sim.cur.players[me].class.next_in_rotation();
        sim.travel = sim::input::Travel::class(me, next);
    }
    // A restart goes on the wire in training too, now that the simulation
    // seats the fresh fight itself (`World::seats`): one path, and the one
    // the replay records.
    if keys.just_pressed(KeyCode::Backspace) {
        sim.travel = sim::input::Travel::RESTART;
    }
    // The picker. `H` swaps between hunting the Ridgeback and fighting each
    // other; `Shift+H` steps to the next creature there is, in its own arena.
    // A restart either way, because a creature appearing in the middle of a
    // round would land on somebody -- and **asked for, not done**: the next
    // tick carries it on the wire and the simulation builds the new fight, so
    // a peer builds it on the same frame. See `picker`.
    if keys.just_pressed(KeyCode::KeyH) {
        let shifted = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        sim.travel = if shifted {
            picker::next(&sim.cur)
        } else {
            picker::toggle(&sim.cur)
        };
    }
    // `V`: back to the valley -- Hearth's square, with the journey kept.
    if keys.just_pressed(KeyCode::KeyV) {
        sim.travel = sim::input::Travel::VALLEY;
    }
    // `N`: the next jump course, in order of difficulty. The same trip on the
    // wire as `H`, with the arena in the byte. See `picker::next_course`.
    if keys.just_pressed(KeyCode::KeyN) {
        sim.travel = picker::next_course(&sim.cur);
    }
    // `T`: the same creature at the next temper on offer -- earned by beating
    // the one below it, or any of them with `--temper`. The same trip on the
    // wire as `H`, with the temper in the byte. See `picker::temper`.
    if keys.just_pressed(KeyCode::KeyT) {
        let trip = picker::temper(&sim.cur, &trophies, any_temper());
        if trip != sim::input::Travel::NONE {
            sim.travel = trip;
        }
    }
    for (key, mode) in [
        (KeyCode::Digit1, Dummy::Idle),
        (KeyCode::Digit2, Dummy::Block),
        (KeyCode::Digit3, Dummy::Attack),
        (KeyCode::Digit4, Dummy::Human),
        (KeyCode::Digit5, Dummy::Bot(hunt::Level::Easy)),
        (KeyCode::Digit6, Dummy::Bot(hunt::Level::Normal)),
        (KeyCode::Digit7, Dummy::Bot(hunt::Level::Hard)),
    ] {
        // Training's alone: the dummy is this machine's, and the seats are
        // the world's -- changed here, on one machine only, they would split
        // the two worlds apart. Online, every seat is somebody.
        if training && keys.just_pressed(key) {
            sim.dummy = mode;
            reseat(&mut sim);
        }
    }
    // `Y`: save the fight so far as a replay. Any time, in any mode -- the
    // tape is always running -- so a moment that felt wrong can be kept
    // after it happened and judged by `cargo run -p hunt --bin replay`.
    if keys.just_pressed(KeyCode::KeyY) {
        save_replay(&mut sim);
    }

    // The meeting first: the tick it opens the line on is the tick the match
    // starts.
    let was_online = sim.driver.online();
    online::meet(&mut sim, real.elapsed().as_millis() as u64);
    if !was_online && sim.driver.online() {
        // The mouse turns whichever fighter is yours, and the match starts
        // with it facing where yours stands facing. Player two spawns facing
        // player one, the other way from the yaw training left the mouse at,
        // and would otherwise turn round on the first frame.
        let facing = fx3(sim.cur.players[sim.local_player()].facing);
        look.yaw = facing.z.atan2(facing.x);
    }

    // Input the palette is claiming belongs to the palette. Clicking a slider
    // used to throw a poke as well, and typing in the search box drove the
    // fighter around -- J, K and L are attack keys.
    let held = if focus.pointer || focus.keyboard {
        SimInput::default()
    } else {
        read_input(&keys, &mouse)
    }
    .looking(look.aim(), look.tilt());
    let held_two = if focus.keyboard {
        SimInput::default()
    } else {
        read_player_two(&keys)
    }
    // Player two has no mouse, so they aim level.
    .looking(look.aim_two(), 0);

    if !sim.driver.online() {
        let ticks = if sim.paused {
            u32::from(std::mem::take(&mut sim.step_once))
        } else if let Some(slow) = shot_slow(sim.cur.frame) {
            slow_motion(&mut sim.clock, slow)
        } else {
            sim.clock.advance(time.delta_secs())
        };
        for _ in 0..ticks {
            if sim.stop_at.is_some_and(|n| sim.cur.frame >= n) {
                sim.paused = true;
                // `SHOT_ALPHA=a` draws the stopped frame `a` of the way on
                // from the one before it, rather than wherever the clock
                // happened to be: what is between two frames is on the
                // screen too.
                if let Some(alpha) = env_f32("SHOT_ALPHA") {
                    sim.clock.hold(alpha);
                }
                break;
            }
            // **The region gate**: the next frame goes ahead only once every
            // region next to a live one has been heard from recently enough
            // (`docs/design/regions.md`). With nobody to hear from, the
            // ledger plays the neighbours itself and the gate is always
            // open; it is here so that the day there is somebody, the local
            // driver already waits the way the online one will.
            let next = sim.cur.frame + 1;
            sim.ledger.tick();
            if !sim.ledger.may_advance(next) {
                continue;
            }
            // Scripted inputs are resampled per simulation tick, not per
            // rendered frame. One render frame can cover several ticks, and
            // reusing a sample across them smears a four-frame press into
            // whatever the frame rate happened to be -- which makes two
            // runs of the same script diverge.
            let scripted = demo_mode().then(|| script(&mut scripted.0, &sim.cur));
            // The rehearsal outranks the demo script and the keyboard, and
            // ends by handing the controls back rather than looping.
            let rehearsed = sim.rehearsing.and_then(|from| {
                let since = sim.cur.frame.saturating_sub(from);
                if since > REHEARSAL_FRAMES {
                    sim.rehearsing = None;
                    None
                } else {
                    Some(rehearsal(since, &sim.cur))
                }
            });
            let two = match sim.dummy {
                // Not in a hunt: the bot fights a fighter, and in a hunt
                // player two is out of it (see `seated`).
                Dummy::Bot(level) if !sim.cur.hunting() => spar(
                    &mut sparring,
                    level,
                    &sim.cur,
                    time.elapsed().as_nanos() as u32,
                ),
                mode => dummy_input(mode, sim.cur.frame, held_two),
            };
            // A trip rides on the first tick after it was asked for, and
            // only that one.
            let travel = std::mem::take(&mut sim.travel);
            // Two seats at one keyboard: you, and player two or the dummy.
            // The third and fourth are people on other machines (`online`),
            // and send nothing here.
            let mut pair = [SimInput::default(); MAX_PLAYERS];
            pair[0] = rehearsed
                .unwrap_or_else(|| scripted_or(scripted, held))
                .travelling(travel);
            pair[1] = two;
            // A replay playing back outranks everything: both seats are
            // the tape's, and the camera turns with the look it recorded.
            // It runs out by handing the controls back.
            if let Some(playback) = sim.playing.as_mut() {
                // What changed outside the inputs before this frame -- the
                // dummy keys' seating -- is applied as the tape says.
                let at = playback.at as u32;
                let mut seats = None;
                for (frame, event) in &playback.tape.events {
                    if *frame == at {
                        let sim::replay::Event::Seats(n) = event;
                        seats = Some(*n);
                    }
                }
                match playback.tape.frames.get(playback.at) {
                    Some(inputs) => {
                        pair = *inputs;
                        playback.at += 1;
                        look.follow(inputs[0]);
                    }
                    None => {
                        platform::log("the replay has run out; the controls are yours");
                        sim.playing = None;
                    }
                }
                if let Some(n) = seats {
                    sim.cur.seats = n;
                }
            }
            // Remembered before the tick, so one press of `[` lands on the
            // frame you were just looking at. Split out of the field access
            // because the ring and the world live on the same struct.
            let Sim {
                history,
                cur,
                tape,
                ledger,
                ..
            } = &mut *sim;
            history.push(cur);
            tape.record(cur.frame, pair);
            ledger.record(cur, pair);
            // The crowd field's aggregates due this frame, off the ledger.
            let (fed, n) = ledger.due(cur.frame.wrapping_add(1));
            sim.prev = sim.cur.clone();
            sim.cur.advance_with(pair, &fed[..n]);
            {
                let Sim { cur, ledger, .. } = &mut *sim;
                ledger.confirm_through(cur.frame, cur);
            }
            scape.observe(&sim.prev, &sim.cur);
            // What was picked up and fed, for this game's bag and morphs.
            let me = sim.local_player();
            keeping.record(&sim.prev, &pair, &sim.cur, me);
            if travel != sim::input::Travel::NONE {
                // A new fight: nothing to interpolate from. (Player two sits
                // a hunt out unless a person has their keys -- the
                // simulation's doing now, from `World::seats`.)
                sim.prev = sim.cur.clone();
            }
        }
        record_round_end(&mut sim);
        say_region_findings(&mut sim);
    } else {
        online::drawn(&mut sim, real.delta());
        let ticks = sim.clock.advance(time.delta_secs());
        let ticks = online::pace(&mut sim, ticks);
        for _ in 0..ticks {
            let scripted = demo_mode().then(|| script(&mut scripted.0, &sim.cur));
            let travel = std::mem::take(&mut sim.travel);
            let local = scripted_or(scripted, held).travelling(travel);
            online::step(&mut sim, local, &mut scape, &mut keeping);
        }
        record_round_end(&mut sim);
        say_region_findings(&mut sim);
    }
}

/// Every new finding of the region watchdog, said once, where a developer
/// will read it: the terminal, or the browser console.
fn say_region_findings(sim: &mut Sim) {
    let Some(watch) = sim.ledger.watch() else {
        return;
    };
    let said = sim.regions_said;
    if watch.seq() == said {
        return;
    }
    let mut fresh: Vec<_> = watch.issues().filter(|i| i.seq > said).collect();
    fresh.reverse();
    for issue in fresh {
        platform::say(&format!("regions: {issue}"));
    }
    sim.regions_said = watch.seq();
}

/// `--record`: the moment a round ends, save it -- once per round, and
/// sixteen frames in, past any rollback, the way a trophy is written.
fn record_round_end(sim: &mut Sim) {
    if !sim.record {
        return;
    }
    match sim.cur.phase {
        sim::state::Phase::RoundOver { left, .. } => {
            // The same wait a trophy makes (`trophies::notice`): twice the
            // rollback window, so a round ending that is taken back is not
            // saved.
            if !sim.round_saved && left.saturating_add(16) < sim::tuning::round_over_frames() {
                sim.round_saved = true;
                save_replay(sim);
            }
        }
        _ => sim.round_saved = false,
    }
}

/// Save the fight so far: the tape, finished where the world stands, to a
/// file beside the settings on a desktop or a download in a browser
/// (`platform::save_replay`). Named for what it was, so a folder of them
/// reads: `ridgeback-champion-3412f-9c1a.replay`.
pub fn save_replay(sim: &mut Sim) {
    sim.tape.finish(&sim.cur);
    let start = &sim.tape.start;
    let name = format!(
        "{}-{}-{}f-{:04x}.replay",
        start.hunt.map_or("versus".to_string(), |s| s.get().slug()),
        sim::replay::class_slug(start.classes[0]),
        sim.cur.frame,
        sim.cur.state_checksum() & 0xffff
    );
    let text = sim.tape.to_text();
    match platform::save_replay(&name, &text) {
        Some(at) => {
            platform::log(&format!(
                "replay saved: {at} ({} frames). Judge it: cargo run -p hunt --bin replay -- <file>",
                sim.tape.len()
            ));
            sim.saved = Some(at);
        }
        None => platform::log("the replay could not be saved"),
    }
}

fn scripted_or(scripted: Option<SimInput>, live: SimInput) -> SimInput {
    scripted.unwrap_or(live)
}

/// The training partner. `Attack` runs on a cadence so the parry window is
/// something you can actually practise against.
fn dummy_input(mode: Dummy, frame: u32, live: SimInput) -> SimInput {
    match mode {
        Dummy::Idle => SimInput::default(),
        Dummy::Block => SimInput::new(SimInput::RIGHT),
        Dummy::Attack if frame % 70 < 2 => SimInput::new(SimInput::LEFT),
        Dummy::Attack => SimInput::default(),
        Dummy::Human => live,
        // Driven by `spar`; reaching here means a hunt, where it sits out.
        Dummy::Bot(_) => SimInput::default(),
    }
}

/// One frame of the sparring bot, building it first if the level changed.
fn spar(sparring: &mut Sparring, level: hunt::Level, w: &World, seed: u32) -> SimInput {
    let bot = match &mut sparring.0 {
        Some(bot) if bot.level() == level => bot,
        slot => slot.insert(hunt::Duelist::new(1, level, seed)),
    };
    bot.watch(w);
    bot.act(w)
}

/// How long the rehearsal drives for, after which the controls come back.
const REHEARSAL_FRAMES: u32 = 40;

/// One frame of the double structure jump, `since` frames into it.
///
/// **The input, not a description of it.** Two structures raised three frames
/// apart -- the second press needs the button up in between, because Raise
/// fires on the earth click's press edge, which is also why the gap cannot be
/// shorter than two frames -- and then the jump nine frames after the first,
/// held so the rise sustains. The earth click is her left click since
/// 2026-10-09 (`sim::moves::elementalist::keys`); space is pressed only after
/// both raises, since space with the earth click is the earth jump.
///
/// Three frames rather than two is deliberate: two is a metre higher at its
/// best, and three gives five different jump frames that reach a third takeoff
/// instead of three, so it is the one to *watch* if you are trying to see the
/// shape of it.
///
/// Aimed at her own feet, because Raise puts the stone where the crosshair is
/// and the technique needs it underfoot.
fn rehearsal(since: u32, w: &World) -> SimInput {
    let mut v = 0u16;
    // Frame 0 and frame 3, with frames 1 and 2 releasing the button.
    if since == 0 || since == 3 {
        v |= sim::moves::elementalist::keys::EARTH;
    }
    if since >= 9 {
        v |= SimInput::SPACE;
    }
    let (yaw, tilt) = looking_at_her_feet(w);
    SimInput::new(v).looking(yaw, tilt)
}

/// The yaw and pitch that put the **crosshair** on the patch of floor the
/// Elementalist is standing on.
///
/// **Not straight down**, which is the first thing anybody writes here and is
/// wrong for the reason `docs/design/aiming.md` exists: the ray starts at the
/// camera's eye, which sits behind and above the shoulder, so a straight-down
/// look lands on the floor somewhere behind her. The first version of the
/// rehearsal did exactly that, raised both stones behind her heels, and reached
/// eighteen metres on two takeoffs -- a single, which is the very thing the
/// rehearsal exists to tell apart from a double.
///
/// Solved by iteration because the eye's own position depends on the pitch: a
/// few rounds settle it, the rig being smooth -- which is `sim::aim::look_onto`,
/// the raycast run backwards, and the same function the sparring bot aims with.
fn looking_at_her_feet(w: &World) -> (u16, i16) {
    let stood = w.players[0].pos;
    (
        0,
        sim::aim::look_onto(stood, 0, sim::Fx::ZERO, sim::Fx::ZERO, stood),
    )
}

/// `DEMO=1` drives player one from a script instead of the keyboard. Used to
/// verify posing and framing without a human at the controls.
fn demo_mode() -> bool {
    platform::env("DEMO").as_deref() == Some("1")
}

/// Quantised angle of a flat direction.
///
/// Floating point here is safe despite the determinism rules: this produces an
/// *input*, and inputs are transmitted rather than recomputed. A peer replays
/// the integer that arrived, never this function.
fn aim_toward(v: sim::V3) -> u16 {
    aim_from_radians(v.z.to_f32_for_render().atan2(v.x.to_f32_for_render()))
}

/// One frame of scripted play.
///
/// Against another fighter it is the fixed beat below, which exists to exercise
/// posing and framing. Against a creature it is the real hunter, because a
/// fixed beat played at a monster would be a demonstration of nothing.
fn script(hunter: &mut hunt::Hunter, w: &sim::World) -> SimInput {
    if let Some(input) = shot_press(w) {
        return input;
    }
    if let Some(beast) = w.monster() {
        // A plan is for one creature. Against another, the plan for that one,
        // if it has one; otherwise the fixed beat below.
        if hunter.species != beast.species {
            match hunt::Hunter::for_species(beast.species, hunter.who) {
                Some(fresh) => *hunter = fresh,
                None => return demo_input(w, w.frame),
            }
        }
        hunter.watch(w);
        return hunter.act(w);
    }
    demo_input(w, w.frame)
}

/// `SHOT_PRESS=keys:frames,keys:frames,...` (with `DEMO=1`) presses those
/// keys for player one for that many frames, one after another from the
/// first frame, and then nothing -- so a capture can land on a move of any
/// class at the frame it wants, held or tapped, without a person at the
/// controls. Keys are `L`, `M`, `R`, `E`, `Q`, `SPACE`, `SHIFT`, `W`, `A`,
/// `S` and `D`, joined with `+`; `_` is nothing. `SHOT_LOOK=<pitch>` is the
/// pitch to look at (the wire's own units; negative looks down), facing the
/// way she started. `SHOT_PRESS=R:20,_:2,R:3` holds right click twenty frames,
/// lets go for two, and taps it.
fn shot_press(w: &sim::World) -> Option<SimInput> {
    /// The steps (keys held, for how many frames) and the pitch.
    type Plan = Option<(Vec<(u16, u32)>, i16)>;
    static PLAN: std::sync::OnceLock<Plan> = std::sync::OnceLock::new();
    let (steps, pitch) = PLAN
        .get_or_init(|| {
            let spec = platform::env("SHOT_PRESS")?;
            let steps = spec
                .split(',')
                .filter_map(|step| {
                    let (keys, frames) = step.trim().split_once(':')?;
                    let bits = keys.split('+').fold(0u16, |bits, key| {
                        bits | match key.trim().to_uppercase().as_str() {
                            "L" => SimInput::LEFT,
                            "M" => SimInput::MIDDLE,
                            "R" => SimInput::RIGHT,
                            "E" => SimInput::MECHANIC,
                            "Q" => SimInput::SPECIAL,
                            "SPACE" => SimInput::SPACE,
                            "SHIFT" => SimInput::SHIFT,
                            "W" => SimInput::W,
                            "A" => SimInput::A,
                            "S" => SimInput::S,
                            "D" => SimInput::D,
                            _ => 0,
                        }
                    });
                    Some((bits, frames.trim().parse().ok()?))
                })
                .collect();
            let pitch = platform::env("SHOT_LOOK")
                .and_then(|p| p.trim().parse().ok())
                .unwrap_or(0);
            Some((steps, pitch))
        })
        .as_ref()?;
    let mut left = w.frame;
    let mut bits = 0;
    for &(keys, frames) in steps {
        if left < frames {
            bits = keys;
            break;
        }
        left -= frames;
    }
    Some(SimInput::new(bits).looking(aim_toward(w.players[0].facing), *pitch))
}

fn demo_input(w: &sim::World, frame: u32) -> SimInput {
    let beat = frame % 480;
    let mut v = 0u16;
    match beat {
        0..=70 => v |= SimInput::W,                      // close the gap
        75..=78 => v |= SimInput::LEFT,                  // bash
        110..=126 => v |= SimInput::RIGHT,               // guard
        150..=153 => v |= SimInput::MIDDLE,              // the third click
        190..=215 => v |= SimInput::CROUCH,              // duck
        240..=243 => v |= SimInput::SHIFT | SimInput::S, // dodge back
        250..=252 => v |= SimInput::SPACE,               // jump
        280..=283 => v |= SimInput::MECHANIC,            // throw the shield
        300..=303 => v |= SimInput::SPECIAL,             // the class special
        340..=343 => v |= SimInput::MECHANIC,            // recall it
        410..=440 => v |= SimInput::S,                   // reset spacing
        _ => {}
    }
    // The script looks at its opponent, which is what a player would do and
    // what makes "forward" mean "toward the fight".
    SimInput::aimed(v, aim_toward(w.players[1].pos.sub(w.players[0].pos)))
}

/// Second set of keys, so two people can play on one keyboard.
fn read_player_two(keys: &ButtonInput<KeyCode>) -> SimInput {
    let mut v = 0u16;
    if keys.pressed(KeyCode::Period) {
        v |= SimInput::LEFT;
    }
    if keys.pressed(KeyCode::Comma) {
        v |= SimInput::RIGHT;
    }
    if keys.pressed(KeyCode::Slash) {
        v |= SimInput::SPECIAL;
    }
    if keys.pressed(KeyCode::KeyL) {
        v |= SimInput::MECHANIC;
    }
    // The third attack button, which player two has no scroll wheel for.
    // `M` sits under the same finger `,` and `.` do.
    if keys.pressed(KeyCode::KeyM) {
        v |= SimInput::MIDDLE;
    }
    // Shift is a dodge and nothing else now -- see `docs/design/controls.md` --
    // so this is the modifier for the arrow keys rather than for the clicks.
    if keys.pressed(KeyCode::ShiftRight) {
        v |= SimInput::SHIFT;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        v |= SimInput::W;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        v |= SimInput::A;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        v |= SimInput::S;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        v |= SimInput::D;
    }
    if keys.pressed(KeyCode::ControlRight) {
        v |= SimInput::SPACE;
    }
    SimInput::new(v)
}

fn read_input(keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> SimInput {
    let mut v = 0u16;
    // Keyboard stand-ins for the clicks, so the prototype is playable without
    // fighting the browser-style context menu on right click.
    if keys.pressed(KeyCode::KeyJ) || mouse.pressed(MouseButton::Left) {
        v |= SimInput::LEFT;
    }
    if keys.pressed(KeyCode::KeyK) || mouse.pressed(MouseButton::Right) {
        v |= SimInput::RIGHT;
    }
    // The third attack button. It exists for the Champion, whose three mouse
    // buttons are three weapons -- see `sim::moves::champion` -- and `U` stands
    // in for it on a hand or a trackpad that cannot find a scroll click, which
    // is most of them.
    if keys.pressed(KeyCode::KeyU) || mouse.pressed(MouseButton::Middle) {
        v |= SimInput::MIDDLE;
    }
    // The two mouse side buttons -- the right thumb's, and the only buttons in
    // the scheme that cost no finger anything. `I` and `O` stand in for them
    // beside `U`, for a trackpad or a two-button mouse. See
    // `docs/design/exploration/0001_control_budget.md`.
    if keys.pressed(KeyCode::KeyI) || mouse.pressed(MouseButton::Back) {
        v |= SimInput::SIDE_A;
    }
    if keys.pressed(KeyCode::KeyO) || mouse.pressed(MouseButton::Forward) {
        v |= SimInput::SIDE_B;
    }
    // `F` and `R`: the index finger's two keys, one row up from `D` -- the
    // same price `Q` and `E` pay. The match reset that used to be on `R` is
    // on Backspace now.
    if keys.pressed(KeyCode::KeyF) {
        v |= SimInput::KEY_F;
    }
    if keys.pressed(KeyCode::KeyR) {
        v |= SimInput::KEY_R;
    }
    // Q and E, not a chord on a click. The special and the mechanic are the
    // two things a class does that nothing else does; burying them under a
    // modifier made them feel optional. E used to double as middle click, and
    // cannot any more now that middle click means something of its own.
    if keys.pressed(KeyCode::KeyQ) {
        v |= SimInput::SPECIAL;
    }
    if keys.pressed(KeyCode::KeyE) {
        v |= SimInput::MECHANIC;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        v |= SimInput::SHIFT;
    }
    if keys.pressed(KeyCode::KeyW) {
        v |= SimInput::W;
    }
    if keys.pressed(KeyCode::KeyA) {
        v |= SimInput::A;
    }
    if keys.pressed(KeyCode::KeyS) {
        v |= SimInput::S;
    }
    if keys.pressed(KeyCode::KeyD) {
        v |= SimInput::D;
    }
    if keys.pressed(KeyCode::Space) {
        v |= SimInput::SPACE;
    }
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::KeyC) {
        v |= SimInput::CROUCH;
    }
    SimInput::new(v)
}

/// Place every bone from the interpolated snapshot.
///
/// Pose is a pure function of simulation state -- see `view::play`. Nothing
/// here accumulates animation time, which is what lets a rollback rewind the
/// characters without them sliding.
/// One of the four pools of transforms the posing pass writes, tagged by which
/// pool it is.
///
/// Four queries in one system have to be provably disjoint or Bevy refuses to
/// run it, and the disjointness is spelled out as "everything I am not": a
/// fighter's root, a fighter's part, a shadow's root and a shadow's part are
/// four different entities and never the same one. Written as an alias because
/// saying it four times in a signature is the same sentence four times.
type Posed<'w, 's, Tag, A, B, C> = Query<
    'w,
    's,
    (&'static Tag, &'static mut Transform),
    (
        Without<A>,
        Without<B>,
        Without<C>,
        Without<BodyBall>,
        Without<ShadowBall>,
    ),
>;

// A Bevy system's parameter list *is* its dependency declaration, and this one
// now poses two bodies per fighter. Splitting it to get under a count would
// mean solving the same skeletons twice.
#[allow(clippy::too_many_arguments)]
fn apply_poses(
    sim: Res<Sim>,
    time: Res<Time>,
    mut fades: ResMut<Fades>,
    mut shadow_fades: ResMut<ShadowFades>,
    mut hands: ResMut<ShieldHands>,
    mut both: ResMut<Hands>,
    mut torsos: ResMut<Torsos>,
    hub: Option<Res<crate::hub::Hub>>,
    mut roots: Posed<Fighter, BodyPart, ShadowRoot, ShadowPart>,
    mut parts: Posed<BodyPart, Fighter, ShadowRoot, ShadowPart>,
    mut shadow_seen: Query<(&mut Visibility, &ShadowRoot)>,
    mut shadow_roots: Posed<ShadowRoot, Fighter, BodyPart, ShadowPart>,
    mut shadow_parts: Posed<ShadowPart, Fighter, BodyPart, ShadowRoot>,
    mut joints: ResMut<Joints>,
) {
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    // The time a picture stands for: real, except under `SHOT_SLOW`, where a
    // picture is a fraction of a frame however long it took to draw, and a
    // fade or a lean eased on the real clock would be over in a picture.
    let dt = match shot_slow(sim.cur.frame) {
        Some(slow) if !sim.paused => 1.0 / (view::TICK_HZ * slow as f32),
        _ => time.delta_secs(),
    };

    // One solve per fighter rather than one per bone: forward kinematics is a
    // single pass down the skeleton and there are sixteen bones hanging off it.
    let mut skeletons: [Skeleton; MAX_PLAYERS] =
        std::array::from_fn(|i| skeleton_for(sim.cur.players[i].class));
    let mut skins = Vec::with_capacity(MAX_PLAYERS);
    for (owner, skeleton) in skeletons.iter_mut().enumerate() {
        let p = frame.players[owner];
        let class = sim.cur.players[owner].class;
        let mut input = PoseInput::of(&p, class, &frame);
        input.bind_pose = sim.bind_pose;
        let mut pose = fades.0[owner].pose(input, dt);
        // The hub takes over whichever fighter it is previewing, so an edit is
        // visible on a real character in the real arena rather than in a
        // separate viewer that flatters it.
        if let Some(hub) = hub.as_ref() {
            if let Some(preview) = hub.preview_for(owner) {
                pose = preview;
            }
        }
        skins.push(view::skeleton::solve(skeleton, &pose));
    }
    for (owner, skin) in skins.iter().enumerate() {
        joints.body[owner] = Some(*skin);
    }
    // Placed after the poses, because posing is what eases the drawn body
    // round side-on to a wall (`Crossfade::turn`).
    for (fighter, mut tf) in roots.iter_mut() {
        let p = frame.players[fighter.0];
        let (origin, turn) = body_frame(&p, fades.0[fighter.0].turn());
        tf.translation = origin;
        tf.rotation = turn;
    }

    // The shield hand, in the arena rather than in the character's own space,
    // so whatever is holding something can be placed against it.
    for owner in 0..MAX_PLAYERS {
        let p = frame.players[owner];
        let (origin, turn) = body_frame(&p, fades.0[owner].turn());
        // The body's **left** hand, which in a drawn pose is not the slot of
        // that name -- see `view::hand_joint`.
        let (at, rot) = skins[owner].box_of(&skeletons[owner], view::hand_joint(true));
        hands.0[owner] = (
            origin + turn * Vec3::new(at[0], at[1], at[2]),
            turn * Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]),
        );
        // And both hands, for whatever is held in two.
        for (side, left) in [(0, true), (1, false)] {
            let (at, _) = skins[owner].box_of(&skeletons[owner], view::hand_joint(left));
            both.0[owner][side] = origin + turn * Vec3::new(at[0], at[1], at[2]);
        }
        // And the chest and hips, for whatever is carried on the body.
        for (slot, joint) in [(0, view::Joint::Chest), (1, view::Joint::Root)] {
            let (at, rot) = skins[owner].box_of(&skeletons[owner], joint);
            torsos.0[owner][slot] = (
                origin + turn * Vec3::new(at[0], at[1], at[2]),
                turn * Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]),
            );
        }
        // And the weapon arm's shoulder and elbow -- the right, which like the
        // hand is not the slot of that name in a drawn pose.
        for (slot, joint) in [(0, view::Joint::ArmR), (1, view::Joint::ForearmR)] {
            let at = skins[owner].origin[view::drawn_joint(joint).index()];
            torsos.1[owner][slot] = origin + turn * Vec3::new(at[0], at[1], at[2]);
        }
    }

    for (bp, mut tf) in parts.iter_mut() {
        let skeleton = &skeletons[bp.owner];
        let (centre, rot) = skins[bp.owner].box_of(skeleton, bp.joint);
        let size = view::pose::part_size(skeleton, bp.joint);
        tf.translation = Vec3::new(centre[0], centre[1], centre[2]);
        tf.rotation = Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]);
        tf.scale = Vec3::new(size[0], size[1], size[2]);
    }

    // The second body, on the same skeleton and through the same solver. It is
    // her, drawn somewhere else: the only thing that differs is which pose it
    // is holding and how much of it you can see through.
    let mut shadow_skins: [Option<view::skeleton::Skin>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    for (mut seen, root) in shadow_seen.iter_mut() {
        // An empty seat's shadow goes with its body (`hide_empty_seats`):
        // a fighter parked out of the world keeps the shadow it stood with.
        let ghost = frame.shadows[root.0].filter(|_| sim.cur.is_present(root.0));
        let Some(ghost) = ghost else {
            *seen = Visibility::Hidden;
            continue;
        };
        *seen = Visibility::Inherited;
        shadow_skins[root.0] = Some(view::skeleton::solve(
            &skeletons[root.0],
            &shadow_fades.0[root.0].shadow(
                shadow_input(&frame, root.0, ghost, sim.bind_pose),
                ghost.doing,
                dt,
            ),
        ));
    }
    for (root, mut tf) in shadow_roots.iter_mut() {
        let Some(ghost) = frame.shadows[root.0] else {
            continue;
        };
        tf.translation = Vec3::new(ghost.pos[0], ghost.pos[1], ghost.pos[2]);
        tf.rotation = Quat::from_rotation_y(ghost.facing[0].atan2(ghost.facing[2]));
    }
    joints.shadow = shadow_skins;
    for (part, mut tf) in shadow_parts.iter_mut() {
        let Some(skin) = shadow_skins[part.owner].as_ref() else {
            continue;
        };
        let skeleton = &skeletons[part.owner];
        let (centre, rot) = skin.box_of(skeleton, part.joint);
        let size = view::pose::part_size(skeleton, part.joint);
        tf.translation = Vec3::new(centre[0], centre[1], centre[2]);
        tf.rotation = Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]);
        tf.scale = Vec3::new(size[0], size[1], size[2]);
    }
}

/// Everything the shadow's pose depends on.
///
/// Hers, with the four things that are the shadow's own swapped in: what it is
/// doing, how fast it is going and which way, and the fact that it is always on
/// the floor. Built from her input rather than from scratch so that anything
/// posing learns to read -- the frame counter an idle loops on, the round's own
/// clock -- reaches both bodies without being wired up twice.
fn shadow_input(
    frame: &view::Frame,
    owner: usize,
    ghost: view::interp::ShadowView,
    bind_pose: bool,
) -> PoseInput {
    let mut input = PoseInput::of(&frame.players[owner], sim::Class::ShadowReaver, frame);
    input.action = ghost.action;
    input.speed = ghost.speed;
    input.travel = ghost.travel;
    input.grounded = true;
    input.crouching = false;
    input.rise = 0.0;
    input.turn_rate = 0.0;
    input.ledge = None;
    input.wall = None;
    input.bind_pose = bind_pose;
    input
}

/// Mouse look, and the cursor grab that makes it usable.
///
/// Deliberately not rate-limited or smoothed. The mouse is the aim, and any
/// filtering between the hand and the crosshair is felt immediately even when
/// it cannot be named.
/// Whether the cursor should be locked to the window this frame.
///
/// Extracted so the rule can be stated once and tested, rather than living
/// inside a system where the only way to check it is to play the game.
///
/// The Oven and the arena share a window, so the cursor has to move between
/// them without ceremony:
///
/// - **Opening the Oven hands the cursor back**, so the first click after F7
///   lands on a widget instead of being spent getting the pointer released.
/// - **Clicking the arena takes it again**, even with the Oven open — that is
///   how you go and feel the change you just made.
/// - **Escape returns it to the Oven**, and is the way out of a captured
///   window whether the Oven is open or not.
///
/// A click *on the panel* is not a click at the arena, which is the whole
/// original bug: every drag of a slider was re-grabbing the mouse.
fn cursor_should_be_captured(
    escape: bool,
    oven_just_opened: bool,
    clicked_in_arena: bool,
    captured: bool,
) -> bool {
    if escape || oven_just_opened {
        false
    } else if clicked_in_arena {
        true
    } else {
        captured
    }
}

#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are its resources
fn mouse_look(
    mut look: ResMut<Look>,
    mut settings: ResMut<settings::Settings>,
    mut motion: EventReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut focus: ResMut<palette::UiFocus>,
    mut menu: ResMut<menu::Menu>,
    mut bag: ResMut<keep::Bag>,
    mut lock_seen: Local<bool>,
    mut windows: Query<&mut Window>,
    inspect: Res<inspect::Inspect>,
) {
    // Sensitivity, adjustable mid-match and written straight to disk. Two people
    // sharing a machine should not have to agree on one number, and a setting
    // you have to quit and edit a file to change is a setting nobody changes.
    let mut changed = false;
    for (key, knob, up) in [
        (KeyCode::Minus, settings::Knob::Sensitivity, false),
        (KeyCode::NumpadSubtract, settings::Knob::Sensitivity, false),
        (KeyCode::Equal, settings::Knob::Sensitivity, true),
        (KeyCode::NumpadAdd, settings::Knob::Sensitivity, true),
        (KeyCode::F3, settings::Knob::Fov, false),
        (KeyCode::F4, settings::Knob::Fov, true),
        (KeyCode::PageDown, settings::Knob::Volume, false),
        (KeyCode::PageUp, settings::Knob::Volume, true),
    ] {
        if keys.just_pressed(key) {
            settings.nudge(knob, up);
            changed = true;
        }
    }
    if changed {
        settings.save();
    }

    // While the camera is on a close look (F6) the mouse goes round the
    // fighter instead, and their aim stays where it was.
    if look.grabbed && inspect.seat.is_none() {
        let sensitivity = settings.radians_per_pixel();
        let (mut dx, mut dy) = (0.0, 0.0);
        for ev in motion.read() {
            dx += ev.delta.x;
            dy += ev.delta.y;
        }
        look.yaw += dx * sensitivity;
        let zones = view::camera::Zones::tuned();
        look.pitch = (look.pitch - dy * sensitivity).clamp(-zones.down_limit, zones.up_limit);
    } else {
        motion.clear();
    }

    // Player two turns with keys, since there is only one mouse.
    let turn = 0.045;
    if keys.pressed(KeyCode::Semicolon) {
        look.yaw_two -= turn;
    }
    if keys.pressed(KeyCode::Quote) {
        look.yaw_two += turn;
    }

    // Click to capture, Escape to release. Without a release the window is a
    // trap, and this runs windowed on a desktop.
    //
    // A click the palette is claiming is not a click at the arena. Without this
    // every drag of a slider re-grabbed the mouse, so tuning a number meant
    // pressing Escape between each one -- which is the opposite of the rapid
    // loop the Oven exists to provide.
    // One rule, so there is nothing to learn: **while the Oven is open the
    // cursor is yours.** Capturing on any click meant every drag of a slider
    // re-grabbed the mouse and tuning a number cost an Escape each time, which
    // is the opposite of the rapid loop the Oven exists to provide. Releasing
    // only while the pointer hovers the panel would still have left the first
    // click after F7 spent on getting the cursor back.
    //
    // The keyboard keeps playing, so you can drag a value and immediately feel
    // it with W and J without closing anything.
    let clicked = mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right);
    // A page lets go of the mouse itself on Escape, and the key never reaches
    // the game: so a lock the page has dropped is an Escape. Only once the
    // lock has been seen, because a page grants it a frame or two after the
    // click that asked for it.
    let lost = look.grabbed && platform::pointer_lock_lost();
    if look.grabbed && !lost {
        *lock_seen = true;
    }
    let lost = lost && std::mem::take(&mut *lock_seen);
    let escape = keys.just_pressed(KeyCode::Escape) || lost;
    // **The Esc menu** (`menu.rs`): Escape out of the fight brings it up, and
    // Escape again puts it away; taking the mouse back puts it away too.
    // Escape with the bag open shuts the bag, and only the bag.
    if escape && bag.open && !look.grabbed {
        bag.open = false;
    } else if escape {
        menu.open = look.grabbed || !menu.open;
    }
    let want = cursor_should_be_captured(
        escape,
        std::mem::take(&mut focus.just_opened),
        clicked && !focus.pointer,
        look.grabbed,
    );
    if want {
        menu.open = false;
        bag.open = false;
    }
    if want != look.grabbed {
        look.grabbed = want;
        if !want {
            *lock_seen = false;
        }
        if let Ok(mut window) = windows.single_mut() {
            window.cursor_options.grab_mode = if want {
                bevy::window::CursorGrabMode::Locked
            } else {
                bevy::window::CursorGrabMode::None
            };
            window.cursor_options.visible = !want;
        }
    }
}

/// **The world moved into another place's coordinates this tick**
/// (`sim::valley::open`): the frame it moved from is moved with it, so the
/// interpolator blends two pictures of the same place rather than drawing
/// everybody sliding a reach across the screen for one frame.
fn same_frame(mut sim: ResMut<Sim>) {
    if sim.prev.arena == sim.cur.arena || !sim.cur.valley.on || !sim.prev.valley.on {
        return;
    }
    let by = sim.prev.map_origin().sub(sim.cur.map_origin());
    sim.prev.shift(by);
    sim.prev.arena = sim.cur.arena;
}

#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are its resources
fn drive_camera(
    sim: Res<Sim>,
    time: Res<Time>,
    look: Res<Look>,
    settings: Res<settings::Settings>,
    mut rig: ResMut<Rig>,
    mut inside: ResMut<InsideOwnHead>,
    mut boom: ResMut<beast::Boom>,
    inspect: Res<inspect::Inspect>,
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    mut origin: Local<Option<Vec3>>,
    mut portrait: Local<Option<Option<[f32; 4]>>>,
) {
    if settings.is_changed() {
        rig.0.set_fov(settings.fov_radians());
    }
    // The world moved into another place's coordinates: the camera's own
    // memory of where it was moves with it, or it swoops across the map.
    let o = sim.cur.map_origin();
    let o = Vec3::new(
        o.x.to_f32_for_render(),
        o.y.to_f32_for_render(),
        o.z.to_f32_for_render(),
    );
    if let Some(last) = origin.replace(o) {
        if last != o {
            rig.0.shift((last - o).to_array());
        }
    }
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    // The camera follows whichever fighter this client is driving, turned by
    // the mouse: `look.yaw` is what drives your fighter in either seat online.
    // (`yaw_two` is the keyboard turn of a second person on this keyboard,
    // who never has the camera.) Reading `yaw_two` for seat two is what left
    // the camera of whoever joined a room pointing where it started.
    let me = sim.local_player();
    let yaw = look.yaw;
    let framing = rig.0.update_around(
        time.delta_secs(),
        frame.players[me].pos,
        yaw,
        look.pitch,
        view::Surroundings {
            beasts: &sim.cur.monsters,
            aboard: sim.cur.players[me].aboard(),
            // Interpolated with everything else the fighter is drawn from, so
            // the framing does not step at the simulation's cadence.
            aloft: frame.players[me].aloft,
            clear: frame.players[me].clear,
            // Where the ground has turned them. `look.yaw` is the mouse and
            // only the mouse; the simulation has already added this to their
            // facing and to the ray the crosshair draws, so the camera has to
            // have it too or it points somewhere the fighter is not.
            carried: frame.players[me].carried,
            // Where the fight is, for the walls, towers and vaults the arm is
            // pulled in from.
            arena: sim.cur.terrain(),
        },
    );
    inside.0 = framing.hidden;
    let feet = frame.players[me].pos;
    *boom = beast::Boom {
        eye: Vec3::from_array(framing.eye),
        to: Vec3::new(
            feet[0],
            feet[1] + sim::tuning::body_height().to_f32_for_render() * 0.5,
            feet[2],
        ),
    };
    if let Ok((mut tf, mut projection)) = cam.single_mut() {
        tf.translation = Vec3::from_array(framing.eye);
        tf.look_at(Vec3::from_array(framing.look_at), Vec3::Y);
        // A close look at somebody (F6): round them, and nobody faded.
        let seat = inspect.seat.unwrap_or(me).min(MAX_PLAYERS - 1);
        if let Some((eye, at)) = inspect.frame(frame.players[seat].pos) {
            tf.translation = eye;
            tf.look_at(at, Vec3::Y);
            inside.0 = 0.0;
        }
        if let Some([seat, angle, distance, height]) = *portrait.get_or_insert_with(shot_portrait) {
            // A portrait instead of the fight's camera: round one fighter,
            // `angle` degrees from straight in front of them.
            let p = frame.players[(seat as usize).min(MAX_PLAYERS - 1)];
            let at = Vec3::new(p.pos[0], p.pos[1] + height, p.pos[2]);
            // `SHOT_SPIN` degrees a second round them: a turntable, for
            // judging a painted body in motion (exploration 0025).
            let spin =
                platform::env_parsed::<f32>("SHOT_SPIN").unwrap_or(0.0) * time.elapsed_secs();
            let turn = body_turn(p.facing) * Quat::from_rotation_y((angle + spin).to_radians());
            tf.translation = at + turn * Vec3::new(0.0, 0.15, distance);
            tf.look_at(at, Vec3::Y);
            // Nobody's own body is faded for being near the fight's eye.
            inside.0 = 0.0;
        }
        // `SHOT_BEAST`: the same, round the first creature -- how a creature
        // is looked at as a piece of art.
        if let Some([angle, distance, height]) = shot_beast() {
            if let Some(beast) = sim.cur.monsters[0] {
                let feet = fx3(beast.pos);
                let at = feet + Vec3::Y * height;
                let yaw = beast.yaw.to_f32_for_render();
                let spin =
                    platform::env_parsed::<f32>("SHOT_SPIN").unwrap_or(0.0) * time.elapsed_secs();
                let turn = Quat::from_rotation_y(yaw + (angle + spin).to_radians());
                tf.translation = at + turn * Vec3::new(0.0, distance * 0.25, distance);
                tf.look_at(at, Vec3::Y);
                inside.0 = 0.0;
            }
        }
        if settings.is_changed() {
            if let Projection::Perspective(p) = projection.as_mut() {
                p.fov = settings.fov_radians();
            }
        }
    }
}

/// `SHOT_BEAST=angle,distance,height`: the portrait camera round the first
/// creature, `angle` degrees round from its facing.
fn shot_beast() -> Option<[f32; 3]> {
    let spec = platform::env("SHOT_BEAST")?;
    let v: Vec<f32> = spec
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect();
    (v.len() == 3).then(|| [v[0], v[1], v[2]])
}

/// `SHOT_SPIN=<degrees a second>` turns that camera round them as the run goes.
/// `SHOT_PORTRAIT=seat,angle,distance,height` takes the camera off the fight
/// and puts it `distance` metres from a fighter, `angle` degrees round from
/// in front of them, looking at `height` above their feet: how a character is
/// looked at as a piece of art (`docs/design/exploration/0024_a_character_done_properly.md`).
/// Pair it with `DEMO=0` and a `SHOT_FRAME`.
fn shot_portrait() -> Option<[f32; 4]> {
    let spec = platform::env("SHOT_PORTRAIT")?;
    let v: Vec<f32> = spec
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect();
    (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
}

/// A simulation position, in the renderer's units.
/// How a fighter's body sits in the arena, as the engine's own rotation.
///
/// `view::body_turn` is the definition -- character space is `+Z` along the
/// facing with the left arm at `-X`, and that convention is shared with the
/// simulation, which swings one-armed moves from the same side the renderer
/// draws the arm on. This is the two-line conversion into Bevy's quaternion,
/// and it is the only place the renderer is allowed to build that rotation.
/// Where a fighter's body is drawn from, and how it is turned: the facing,
/// and over it the whole-body roll of a flip or a spin (`view::PlayerView::
/// roll`), turned about the hips rather than the feet so a flip goes over
/// rather than cartwheeling off its own toes. Everything carried on the body
/// is placed through this, so a blade in hand flips with the hand.
///
/// `turned` is how far the drawn body is turned off the facing about the
/// vertical, the posing's own eased answer (`view::play::Crossfade::turn`):
/// side-on to a wall it is sliding down (2026-10-20), nought otherwise.
fn body_frame(p: &view::PlayerView, turned: f32) -> (Vec3, Quat) {
    // Swimming, she is drawn sunk into her ground, a hump of her showing.
    let feet = Vec3::new(p.pos[0], p.pos[1] - p.sunk * SUNK_BY, p.pos[2]);
    let turn = body_turn(p.facing) * Quat::from_rotation_y(turned);
    if p.roll == [0.0, 0.0] {
        return (feet, turn);
    }
    let rot = turn * Quat::from_rotation_y(p.roll[1]) * Quat::from_rotation_x(p.roll[0]);
    let hips = Vec3::Y * ROLL_PIVOT;
    (feet + turn * hips - rot * hips, rot)
}

/// How high above the feet a flip turns the body over: about the hips.
/// Presentation, not a rule.
const ROLL_PIVOT: f32 = 0.95;

/// How far under her ground a swimming Blood mage is drawn: her shoulders
/// and head still above it. Presentation, not a rule -- her hurt volume is
/// `sim::tuning::swim_height`.
const SUNK_BY: f32 = 1.2;

fn body_turn(facing: [f32; 3]) -> Quat {
    let q = view::body_turn([facing[0], facing[2]]).0;
    Quat::from_xyzw(q[0], q[1], q[2], q[3])
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::UiFocus;
    use sim::effects::{Effect, EffectKind, GRASP_ARMS, LOTUS_BLADES};

    fn cast(kind: EffectKind, slot: u8) -> Effect {
        Effect::cast(
            kind,
            0,
            sim::Class::BloodMage,
            slot,
            sim::V3::ZERO,
            sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO),
            sim::moves::get(sim::Class::BloodMage, slot).reach,
        )
    }

    #[test]
    fn the_aim_marker_sits_on_the_far_end_of_the_solved_path() {
        // The marker's only job is to answer *how far out is this going*, and
        // the only way it can answer wrongly is by being somewhere other than
        // the end of the path the simulation has already solved. So that is the
        // whole assertion: the same point, to the millimetre, with no
        // arithmetic of its own on this side.
        let mut p = sim::state::Player::new(sim::Class::BloodMage);
        p.action = sim::state::Action::Channel {
            kind: sim::state::SLOT_SPECIAL,
            held: 7,
        };
        p.aim_path = sim::aim::Path {
            from: sim::V3::new(sim::Fx::ZERO, sim::Fx::from_int(1), sim::Fx::ZERO),
            to: sim::V3::new(
                sim::Fx::from_int(6),
                sim::Fx::from_int(1),
                sim::Fx::from_int(2),
            ),
        };
        let mark = mark_piece(&p).expect("a channelling fighter has a marker");
        assert_eq!(mark.at, fx3(p.aim_path.to));
        assert_eq!(mark.shape, Shape::Ball);
        let want = sim::tuning::grasp_mark().to_f32_for_render() * 2.0;
        assert_eq!(mark.scale, Vec3::splat(want));
    }

    #[test]
    fn nothing_but_a_channel_draws_a_marker() {
        // It is not a thing in the world -- no hitbox, no clock, nobody can
        // walk into it -- so it has to be gone the frame the wind-up is. A
        // marker left standing after the release would read as an ability that
        // is still out.
        let mut p = sim::state::Player::new(sim::Class::BloodMage);
        assert!(mark_piece(&p).is_none(), "a fighter doing nothing has one");
        p.action = sim::state::Action::Startup {
            kind: sim::state::SLOT_SPECIAL,
            left: 4,
        };
        assert!(mark_piece(&p).is_none(), "it outlived the channel");
    }

    #[test]
    fn a_black_spike_is_drawn_with_a_spike_in_it() {
        // It was a twelve-centimetre stain on the floor for a while, which is a
        // hazard you find out about by standing in it. The field is the disc;
        // the spike is what you can see from across the arena.
        let mut effect = cast(EffectKind::BlackSpike, sim::state::SLOT_MECHANIC);
        // The disc it came out of, as `World::advance` sets it on the spawn.
        effect.reach = sim::moves::get(sim::Class::BloodMage, sim::state::SLOT_MECHANIC).radius;
        assert!(
            effect_piece(&effect, 0).is_none(),
            "a bare spike is drawn with a skirt"
        );
        // Erupted from a pool, it has the skirt: the disc it hit across,
        // once it has spread out to it (2026-10-19).
        effect.banked = 1;
        effect.age = sim::tuning::erupt_spread();
        let field = effect_piece(&effect, 0).expect("the field is drawn");
        let spike = effect_piece(&effect, 1).expect("the spike is drawn");
        assert_eq!(field.shape, Shape::Column);
        assert_eq!(spike.shape, Shape::Spike);
        assert!(
            spike.scale.y > field.scale.y * 4.0,
            "the spike is no taller than the stain it stands in"
        );
        assert!(
            spike.scale.x < field.scale.x,
            "the spike is as wide as the whole field, which is a tent"
        );
    }

    #[test]
    fn the_drawn_spike_is_exactly_as_tall_as_the_volume_that_hits() {
        // The rule for every effect in the game: what you see is what catches
        // you. A field drawn shorter than it tests would be a hazard you think
        // you jumped over.
        let mut effect = cast(EffectKind::BlackSpike, sim::state::SLOT_MECHANIC);
        effect.reach = sim::moves::get(sim::Class::BloodMage, sim::state::SLOT_MECHANIC).radius;
        effect.banked = 1;
        let volume = effect.spike_volume();
        let spike = effect_piece(&effect, 1).expect("the spike is drawn");
        assert!(
            (spike.scale.y - volume.top.to_f32_for_render()).abs() < 0.001,
            "drawn {} tall, drains up to {}",
            spike.scale.y,
            volume.top.to_f32_for_render()
        );
        let field = effect_piece(&effect, 0).expect("the field is drawn");
        assert!(
            (field.scale.x * 0.5 - volume.radius.to_f32_for_render()).abs() < 0.001,
            "the drawn field is not the width of the drained one"
        );
    }

    #[test]
    fn a_thrown_blade_is_drawn_where_it_actually_is() {
        // The blade is the whole threat -- the move leaves no hitbox on the
        // caster -- so a picture of it anywhere but its own position would be
        // the only thing telling the other player where the danger is, lying.
        let mut effect = cast(EffectKind::Bloodletter, sim::state::SLOT_POKE);
        let mut furthest = 0.0f32;
        for _ in 0..effect.life {
            effect.age += 1;
            let drawn = effect_piece(&effect, 0).expect("the blade is drawn");
            assert_eq!(drawn.at, fx3(effect.blade_at()));
            furthest = furthest.max(drawn.at.x);
        }
        assert!(furthest > 1.0, "the blade never went anywhere");
        assert!(
            effect_piece(&effect, 1).is_none(),
            "a blade is one object, not two"
        );
    }

    #[test]
    fn all_four_arms_of_a_grasp_are_drawn() {
        // Each one is its own skillshot and each one has to be seen coming --
        // it is being caught by *all* of them that roots you, and a player who
        // can only see two cannot tell where the fourth is going to be.
        let mut effect = cast(EffectKind::Grasp, sim::state::SLOT_SPECIAL);
        effect.age = effect.life / 2;
        let drawn: Vec<Piece> = (0..GRASP_ARMS)
            .map(|arm| effect_piece(&effect, arm).expect("every arm is drawn"))
            .collect();
        for (arm, piece) in drawn.iter().enumerate() {
            assert_eq!(piece.at, fx3(effect.arm_at(arm)));
        }
        // Halfway through, the cone is open: no two arms are in the same place.
        for a in 0..GRASP_ARMS {
            for b in a + 1..GRASP_ARMS {
                assert!(
                    drawn[a].at.distance(drawn[b].at) > 0.1,
                    "arms {a} and {b} are drawn on top of each other"
                );
            }
        }
    }

    #[test]
    fn a_pool_is_drawn_as_a_spill_round_the_disc_it_is_tested_at() {
        // A stain is a place she wants the fight to be, and both players have
        // to be able to read it at a glance -- as blood spilled on the floor,
        // not a disc (2026-10-14). Its body sits on the disc the hit test and
        // the blink read, its roots creep out past it, every piece lies on the
        // floor, and a fuller one is bigger and more solid.
        let effect = Effect::pool(
            0,
            sim::Class::BloodMage,
            sim::state::SLOT_POKE,
            sim::V3::new(sim::Fx::from_int(2), sim::Fx::ZERO, sim::Fx::from_int(-1)),
            100,
        );
        let centre = fx3(effect.pos);
        let radius = effect.pool_radius().to_f32_for_render();
        let pieces: Vec<Piece> = (0..STAIN_PARTS)
            .map(|k| effect_piece(&effect, k).expect("every piece of a stain is drawn"))
            .collect();
        assert!(effect_piece(&effect, STAIN_PARTS).is_none());
        let Skin::Essence(step) = pieces[0].skin else {
            panic!("a pool is not drawn as essence");
        };
        assert!(
            pieces[0].at.xz().distance(centre.xz()) < 0.001,
            "its body is on its disc"
        );
        for (k, piece) in pieces.iter().enumerate() {
            // The spill is on the floor; the essence stands out of it, no
            // taller than the column the crosshair finds it by.
            let top = if k < 12 {
                0.1
            } else {
                effect.pool_height().to_f32_for_render().max(0.3) * 1.4
            };
            assert!(piece.at.y < top, "piece {k} is {} m up", piece.at.y);
            assert!(
                piece.at.xz().distance(centre.xz()) < radius * 4.0,
                "piece {k} is {} m from a stain {radius} m across",
                piece.at.xz().distance(centre.xz())
            );
        }
        // Not a ring: the roots are not the same piece turned round.
        let reach = |k: usize| pieces[k].at.xz().distance(centre.xz());
        assert!(
            (reach(3) - reach(6)).abs() > 0.001 || (reach(4) - reach(7)).abs() > 0.001,
            "two roots are drawn the same"
        );
        let mut less = effect;
        less.banked = 20;
        let smaller = effect_piece(&less, 0).expect("drawn");
        assert!(smaller.scale.x < pieces[0].scale.x);
        let Skin::Essence(faint) = smaller.skin else {
            panic!("a pool is not drawn as essence");
        };
        assert!(
            faint < step,
            "an emptier pool is drawn as solid as a fuller one"
        );
    }
    /// The same, for a class other than the Blood mage.
    fn cast_as(kind: EffectKind, class: sim::Class, slot: u8) -> Effect {
        Effect::cast(
            kind,
            0,
            class,
            slot,
            sim::V3::ZERO,
            sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO),
            sim::moves::get(class, slot).reach,
        )
    }

    #[test]
    fn every_blade_of_a_lotus_is_drawn() {
        // All twelve, not the first six. The pool the renderer spawns from used
        // to be a hand-written six, which was right when the flower had six
        // blades and quietly wrong the day it had twelve: the back half cut
        // people with nothing on screen to say they were there.
        let mut effect = cast_as(
            EffectKind::GuillotineLotus,
            sim::Class::ShadowReaver,
            sim::state::SLOT_SPECIAL,
        );
        effect.age = sim::tuning::lotus_erupt();
        let drawn: Vec<Piece> = (0..LOTUS_BLADES)
            .map(|blade| effect_piece(&effect, blade).expect("every blade is drawn"))
            .collect();
        for (blade, piece) in drawn.iter().enumerate() {
            assert_eq!(piece.at, fx3(effect.lotus_at(blade, effect.pos)));
        }
        // Open, so no two of them are in the same place -- which is also what
        // says the count the renderer walks is the count the flower has.
        for a in 0..LOTUS_BLADES {
            for b in a + 1..LOTUS_BLADES {
                assert!(
                    drawn[a].at.distance(drawn[b].at) > 0.1,
                    "blades {a} and {b} are drawn on top of each other"
                );
            }
        }
    }

    #[test]
    fn five_of_her_stand_round_him_clear_of_her_and_strike_together() {
        // The Reaping's copies: on a ring round him, none where she stands
        // (she came from -x, the way the blow went being +x), no two on top
        // of each other, every one looking at him, and all of them in the
        // overhead's active frames on the same frame.
        let effect = reaping_at(sim::tuning::cast_height(), sim::tuning::body_radius());
        let her = Vec3::NEG_X * COPY_RING;
        let spots: Vec<(Vec3, Vec3)> = (0..copies()).map(|c| copy_spot(&effect, c)).collect();
        assert_eq!(
            spots.len(),
            REAP_COPIES.min(sim::tuning::mark_cap() as usize)
        );
        for (k, (spot, facing)) in spots.iter().enumerate() {
            assert!(
                (spot.length() - COPY_RING).abs() < 1e-3,
                "copy {k} is off the ring"
            );
            assert_eq!(spot.y, 0.0, "copy {k} is not on the floor");
            assert!(spot.distance(her) > 1.0, "copy {k} stands where she does");
            assert!(
                facing.dot(-spot.normalize()) > 0.999,
                "copy {k} is not looking at him"
            );
            for (j, (other, _)) in spots.iter().enumerate().skip(k + 1) {
                assert!(spot.distance(*other) > 1.0, "copies {k} and {j} overlap");
            }
        }
        let strike = COPY_STRIKE + 1.0;
        assert!(matches!(
            copy_action(strike),
            sim::state::Action::Active { .. }
        ));
        assert!(
            (0..copies()).all(|c| copy_solid(strike, c) > 0.3),
            "a copy is not there when the overheads land"
        );
        assert!(
            (0..copies()).all(|c| copy_solid(effect.life as f32, c) < 0.01),
            "a copy is still standing when the Reaping is over"
        );
    }

    /// A Reaping met `high` off the floor at the origin, on something
    /// `thick`, her blow having gone along +x.
    fn reaping_at(high: sim::Fx, thick: sim::Fx) -> Effect {
        let mut e = Effect::cast(
            EffectKind::Reaping,
            0,
            sim::Class::ShadowReaver,
            0,
            sim::V3::new(sim::Fx::ZERO, high, sim::Fx::ZERO),
            sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO),
            thick,
        );
        e.home = sim::V3::ZERO;
        e
    }

    #[test]
    fn five_of_her_spread_out_round_what_they_cut_and_stay_her_size() {
        // A big creature's leg cut at the knee, and its back four metres up:
        // the copies stand on the floor outside the thing, never in it, the
        // ring wider the bigger the thing. (Every copy is her size: nothing
        // scales them but the fold back into her.)
        let ring = |e: &Effect| {
            let (spot, _) = copy_spot(e, 0);
            Vec2::new(spot.x, spot.z).length()
        };
        let fighter = reaping_at(sim::tuning::cast_height(), sim::tuning::body_radius());
        let leg = reaping_at(sim::Fx::from_int(1), sim::Fx::ONE);
        let back = reaping_at(sim::Fx::from_int(4), sim::Fx::from_int(4));
        assert!(
            ring(&leg) > ring(&fighter),
            "a leg is ringed as tight as a fighter"
        );
        assert!(
            ring(&back) > ring(&leg),
            "a whole back is ringed as tight as a leg"
        );
        for (name, e) in [("leg", leg), ("back", back)] {
            let thick = e.reach.to_f32_for_render();
            for c in 0..copies() {
                let (spot, _) = copy_spot(&e, c);
                assert_eq!(spot.y, 0.0, "a copy round the {name} is not on the floor");
                assert!(
                    Vec2::new(spot.x, spot.z).length() > thick + 0.5,
                    "copy {c} stands inside the {name}"
                );
            }
        }
    }

    #[test]
    fn the_pool_has_a_slot_for_every_piece_an_effect_draws() {
        // The general form of the bug above, and the reason `EFFECT_PARTS` is
        // counted rather than typed. The renderer spawns `EFFECT_PARTS` meshes
        // per effect and asks `effect_piece` for each; anything the piece
        // function will answer for beyond that has no mesh to be drawn with and
        // is invisible. So: nothing may be drawn at the first index past the
        // end of the pool.
        for code in 0..32u8 {
            let Some(kind) = EffectKind::from_code(code) else {
                continue;
            };
            let effect = cast_as(kind, sim::Class::ShadowReaver, sim::state::SLOT_SPECIAL);
            assert!(
                effect_piece(&effect, EFFECT_PARTS).is_none(),
                "{} draws a piece at part {EFFECT_PARTS}, which is past the end \
                 of the pool the renderer spawns -- it would never be seen",
                kind.name()
            );
        }
    }

    #[test]
    fn opening_the_oven_hands_the_cursor_back() {
        // Otherwise the first click after F7 is spent releasing the pointer
        // rather than landing on the slider you were reaching for.
        assert!(!cursor_should_be_captured(false, true, true, true));
    }

    #[test]
    fn clicking_the_panel_does_not_take_the_cursor() {
        // The original bug: every drag of a slider re-grabbed the mouse, so
        // changing a number cost an Escape each time. A click the palette is
        // claiming arrives here as `clicked_in_arena == false`.
        assert!(!cursor_should_be_captured(false, false, false, false));
    }

    #[test]
    fn clicking_the_arena_takes_it_again_even_with_the_oven_open() {
        // How you go and feel the change you just made, without closing
        // anything. `oven_just_opened` is only true on the frame F7 is pressed.
        assert!(cursor_should_be_captured(false, false, true, false));
        // And a capture persists without needing the button held.
        assert!(cursor_should_be_captured(false, false, false, true));
    }

    #[test]
    fn escape_always_returns_it() {
        // Back to the Oven when it is open, and out of a captured window when
        // it is not. Without a way out the window is a trap, and this runs
        // windowed on a desktop.
        assert!(!cursor_should_be_captured(true, false, true, true));
        assert!(!cursor_should_be_captured(true, true, true, true));
    }

    #[test]
    fn the_palette_takes_input_only_when_it_is_using_it() {
        // Pointer and keyboard are claimed separately on purpose: hovering a
        // slider must not stop W and J from working, or you could not feel the
        // change you just made without closing the panel.
        let hovering = UiFocus {
            pointer: true,
            ..UiFocus::default()
        };
        assert!(hovering.pointer && !hovering.keyboard);

        let typing = UiFocus {
            keyboard: true,
            ..UiFocus::default()
        };
        assert!(typing.keyboard, "a focused text field claims the keyboard");

        let idle = UiFocus::default();
        assert!(
            !idle.pointer && !idle.keyboard,
            "a shut palette claims nothing"
        );
    }
}

#[cfg(test)]
mod rehearsing {
    use super::*;

    /// Run the rehearsal and report the highest she gets, and how many takeoffs
    /// fired on the way up.
    fn play() -> (f32, u32) {
        let mut w = World::with_classes([sim::Class::Elementalist, sim::Class::Bulwark]);
        let (mut apex, mut takeoffs, mut climbing) = (0.0f32, 0, true);
        for i in 0..200u32 {
            let me = if i <= REHEARSAL_FRAMES {
                rehearsal(i, &w)
            } else {
                SimInput::default()
            };
            let before = w.players[0].vel.y;
            w.advance([me, SimInput::default()]);
            let p = &w.players[0];
            if climbing && p.vel.y.sub(before).to_f32_for_render() > 8.0 {
                takeoffs += 1;
            }
            if climbing && i > 9 && p.vel.y.raw() < 0 {
                climbing = false;
            }
            apex = apex.max(p.pos.y.to_f32_for_render());
        }
        (apex, takeoffs)
    }

    #[test]
    fn the_rehearsal_actually_performs_a_double() {
        // **The point of it existing.** It is a fixed input sequence standing in
        // for a thirty-three millisecond double tap nobody can do by hand, so if
        // it quietly stops landing the technique it is worse than nothing --
        // somebody would watch it and conclude the double looks like that.
        //
        // Three takeoffs is what makes a double a double rather than a single
        // off whichever stone happened to be bursting: the first eruption
        // catches her feet after she jumps, and the second catches her again.
        let (apex, takeoffs) = play();
        assert_eq!(
            takeoffs, 3,
            "the rehearsal reached {apex:.1} m on {takeoffs} takeoffs -- a double is three"
        );
        assert!(
            apex > 35.0,
            "the rehearsal only reached {apex:.1} m, which is single-structure height"
        );
    }

    #[test]
    fn it_hands_the_controls_back() {
        // A rehearsal that never ended would be a mode rather than a
        // demonstration. It drives for `REHEARSAL_FRAMES` and stops, which the
        // tick loop reads to clear `Sim::rehearsing`.
        const {
            assert!(
                REHEARSAL_FRAMES > 9,
                "the rehearsal stops before it presses jump"
            )
        };
        // And the shape of the input is the thing the kit document describes:
        // two presses three frames apart with the button up between them.
        let w = World::with_classes([sim::Class::Elementalist, sim::Class::Bulwark]);
        assert!(rehearsal(0, &w).has(sim::moves::elementalist::keys::EARTH));
        assert!(!rehearsal(1, &w).has(sim::moves::elementalist::keys::EARTH));
        assert!(!rehearsal(2, &w).has(sim::moves::elementalist::keys::EARTH));
        assert!(rehearsal(3, &w).has(sim::moves::elementalist::keys::EARTH));
        assert!(!rehearsal(8, &w).has(SimInput::SPACE));
        assert!(rehearsal(9, &w).has(SimInput::SPACE));
    }
}
