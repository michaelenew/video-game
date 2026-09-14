//! Everything you can type, in one place.
//!
//! This is the **single source of truth** for controls, flags and environment
//! variables — not a document *about* them. The in-game legend is generated
//! from the same tables, and a test checks that every key the game actually
//! handles appears here. A help text maintained separately from the thing it
//! describes is wrong within a month, and wrong help is worse than none: it
//! sends you looking for a feature that moved.
//!
//! No dependencies, so `cargo run -p manual` answers in the time it takes to
//! compile a few hundred lines rather than a graphics engine.

/// One thing you can type, and what it does.
pub struct Entry {
    /// What you type.
    pub invocation: &'static str,
    /// What it does, in one line.
    pub what: &'static str,
    /// The compact form for the on-screen legend, where it earns a place.
    pub short: Option<&'static str>,
}

pub struct Section {
    pub title: &'static str,
    pub blurb: &'static str,
    pub entries: &'static [Entry],
}

const fn e(invocation: &'static str, what: &'static str) -> Entry {
    Entry {
        invocation,
        what,
        short: None,
    }
}

const fn s(invocation: &'static str, what: &'static str, short: &'static str) -> Entry {
    Entry {
        invocation,
        what,
        short: Some(short),
    }
}

pub const SECTIONS: &[Section] = &[
    Section {
        title: "Running it",
        blurb: "Rust 1.85+. Everything runs from the workspace root.",
        entries: &[
            e(
                "cargo run -p game",
                "The prototype: 3D arena, two fighters, live frame data.",
            ),
            e(
                "./scripts/dev.sh",
                "Full development mode: hitbox wireframes and the Oven, both open. Passes extra arguments through.",
            ),
            e(
                "cargo run -p game -- --dev",
                "The same, if you would rather not use the script.",
            ),
            e("cargo run -p game -- --help", "This text."),
            e(
                "cargo run -p game -- --p1 <class> --p2 <class>",
                "Pick classes. Matched loosely: bulwark, champion, reaver, elementalist, blood, dual.",
            ),
            e(
                "cargo run -p game -- --hunt",
                "Start against the Ridgeback instead of each other. H switches either way in game.",
            ),
            e(
                "cargo run -p game -- --port <n> --peer <ip:port>",
                "Peer-to-peer against someone else. Rollback netcode, no server.",
            ),
            e(
                "cargo run -p manual",
                "This text, without building the game.",
            ),
        ],
    },
    Section {
        title: "Fighting",
        blurb: "Camera-relative: W is away from the camera, not along a world axis.",
        entries: &[
            s(
                "Mouse",
                "Aim. Where you look is where you are pointed.",
                "Mouse aims",
            ),
            s(
                "Click / Esc",
                "Capture the mouse, and release it.",
                "click to capture, Esc to release",
            ),
            s(
                "W A S D",
                "Move, relative to the camera.",
                "WASD move (camera-relative)",
            ),
            s(
                "Space",
                "Jump. Hold it to go higher; releasing is final. On the Champion, pressing a weapon inside the first few frames of a jump throws that weapon's takeoff instead of an aerial.",
                "Space jump (hold = higher)",
            ),
            s(
                "Shift + direction",
                "Dodge. Airborne, an airdodge — once per jump.",
                "Shift+dir dodge",
            ),
            s(
                "Ctrl or C",
                "Crouch. Ducks overheads, costs you speed.",
                "Ctrl crouch",
            ),
            s(
                "J or left click",
                "Poke. The fast one. In the air it hangs you and shoves you the way you are holding. On the Champion it is the sword, and held down it walks that weapon's whole three-hit string.",
                "J poke",
            ),
            s(
                "Shift + J",
                "The committed attack. Slower, hurts, roots you.",
                "Shift+J committed",
            ),
            s(
                "K or right click",
                "Guard. The first few frames parry. Three classes spend it instead, none of them having a shield to raise: the Champion's spear, the Dual mage's light auto, and the Shadow Reaver's shadow -- hers is the one the crosshair aims, so it goes on the hand doing the aiming.",
                "K guard",
            ),
            s(
                "U or middle click",
                "The third attack button. Only the Champion has one: it is the hammer.",
                "U third attack",
            ),
            s(
                "Q",
                "The class special. The fire pillar, the grapple — the move only that class has. The Champion has none: its three weapons are its three clicks.",
                "Q special",
            ),
            s(
                "E",
                "The class mechanic. Different on every class: throw the shield, Rush, raise a structure. Three classes put a real ability here instead, with a wind-up you can be punished during — and on the Shadow Reaver it is not even the mechanic, because hers went to right click.",
                "E mechanic",
            ),
        ],
    },
    Section {
        title: "The Champion",
        blurb: "Three weapons on three clicks, a three-hit string where every hit is a free choice of weapon, and a dash that changes what all three of them do. The button is the weapon; the situation picks the move.",
        entries: &[
            s(
                "Left click",
                "Sword. Arc across the front. Fast, wide, least commitment — the move you string with.",
                "LMB sword",
            ),
            s(
                "Middle click (or U)",
                "Hammer. Arc down to the floor. Slow, short, and it staggers — the move you start with.",
                "MMB hammer",
            ),
            s(
                "Right click",
                "Spear. A line straight ahead. Longest reach, and it goes over anyone crouching.",
                "RMB spear",
            ),
            e(
                "Keep swinging",
                "Land a hit and the same three buttons throw the second of three, then the third. Every hit is a free choice of all three weapons, so sword into spear into hammer is an ordinary thing to do. The string flows only while you are connecting: blocked or thrown at nothing, you pay the whole recovery. It ends if you stop for half a second, are hit, block, dodge, or leave the ground.",
            ),
            e(
                "The third hit",
                "Crescent is a full turning cut and the widest thing in the game. Earthbreaker goes through a guard. Impale reaches four and a half metres. All three commit your feet.",
            ),
            e(
                "Space and a weapon",
                "That weapon's takeoff, thrown as your feet leave the floor. Sword rises into an angled slash and hits hardest; hammer is the uppercut, which launches and holds on, and space again takes you both higher; spear cracks the shaft into the ground for the most height in the class plus a shove the way you are holding. Press jump first and the weapon a few frames later — that order always works.",
            ),
            e(
                "In the air",
                "The same three buttons, different moves. Sword cuts downward; hammer winds up slowly and spikes an airborne target into the floor; spear fans around the aim and shoves you the way you are holding if it connects.",
            ),
            s(
                "E",
                "Rush. A dash on one charge, and it cancels any recovery. It does not end a string — one charge buys you a reposition in the middle of one.",
                "E rush",
            ),
            e(
                "While rushing",
                "Sword cuts as you run past without stopping the dash. Hammer drags along the floor and takes the legs of anyone you pass. Spear stabs for the biggest single hit in the class, or vaults if you are pointing at the floor.",
            ),
        ],
    },
    Section {
        title: "The Dual mage",
        blurb: "Two forces, one in each arm, and a bar between them. Which button you attack with is which way you drift, and depth is power -- but past a threshold it burns you.",
        entries: &[
            s(
                "Left click",
                "Dark auto. A punch with the left arm, and a wing that opens behind you on that side and comes round to the front. Moves you five darker, and makes you dark.",
                "LMB dark auto",
            ),
            s(
                "Right click",
                "Light auto. The same punch and wing mirrored onto the right arm. Moves you five lighter, and makes you light. There is no guard on this class.",
                "RMB light auto",
            ),
            s(
                "Shift + left click",
                "Lance. A line at whatever the crosshair is on. Committed, and it moves you darker whether or not it connects.",
                "Shift+LMB lance",
            ),
            s(
                "E",
                "Sweep. Both arms across the whole front. No side of its own, so it pushes you further along whichever way you were already going.",
                "E sweep",
            ),
            s(
                "Q",
                "Judgement. The finisher: a delayed strike where the crosshair is.",
                "Q judgement",
            ),
            e(
                "The tip",
                "The last frame of either auto is the wing's tip, and it hits far harder. It is the only part that reaches straight out in front of you, so landing it is a question of standing at the edge of your range rather than on top of them.",
            ),
            e(
                "Which force you are",
                "Whichever auto you threw last. Everything else you throw is made of that force and pushes the bar twelve the same way, so the two clicks are the steering and everything else is the accelerator.",
            ),
            e(
                "Getting back",
                "Throwing a far-side auto is the only way back toward centre: casts follow whichever force you are carrying, and only an auto changes that.",
            ),
            e(
                "The ends of the bar",
                "Driven all the way to either end and it takes you: three seconds of heavy drain you cannot steer or stop, and it puts you back at the centre staggered.",
            ),
        ],
    },
    Section {
        title: "The Shadow Reaver",
        blurb: "Two bodies. The shadow is never away — it is at your shoulder or out on the field — and everything the class does is a function of the line between the two.",
        entries: &[
            e(
                "The shadow copies you",
                "Whatever you swing, it swings a few frames later for a quarter of the damage. Held at your shoulder that is a quarter again on everything; sent out, it is a second threat somewhere you are not.",
            ),
            s(
                "Right click",
                "Send the shadow where you are pointing, fast, and it stops there. Press again and it dashes home through anything in the way, cutting and slowing it — and taking an open Guillotine lotus with it. It answers whatever else you are doing: pressed during the tail of another move it cuts that tail short, and pressed a few frames early it is remembered rather than dropped.",
                "RMB send shadow",
            ),
            s(
                "Q",
                "Guillotine lotus. Six blades erupt from the shadow, hang open, and chase it home — so recalling the shadow with right click drags them the length of the arena.",
                "Q lotus",
            ),
            s(
                "E",
                "Executioner, the committed melee. Shift + left click throws the same move. It is on the key rather than the mouse because it is a swing off the body -- the mouse is spent on the shadow, which is the thing you actually aim.",
                "E executioner",
            ),
            e(
                "Shift + forward",
                "With the crosshair on the shadow, the dodge becomes the dash to it: invulnerable across the gap, and you pick the shadow up when you arrive. Straight to wherever it is standing -- up onto a platform included -- and only something with no way through at all can refuse it. In the air it is the airdodge that does it, and it costs the airdodge. Pointed anywhere else it is the ordinary dodge.",
            ),
            e(
                "Space, the moment a dash lands",
                "The dash jump. You arrive still moving, and a jump pressed in that short window takes the speed up with you instead of leaving it on the floor -- the earlier you find it, the further you go. It is the one thing that can cut a dodge's tail short, and it only answers a dash.",
            ),
            e(
                "The leash",
                "Walk far enough from a shadow standing out on the field and it comes and finds you, cutting on the way. Straying is a decision, not a mistake.",
            ),
        ],
    },
    Section {
        title: "Hunting the Ridgeback",
        blurb: "H starts a hunt. Its back is the only part worth hitting, so the fight is about getting up there.",
        entries: &[
            s(
                "H",
                "Hunt the Ridgeback, or go back to fighting each other. Restarts the match either way.",
                "H hunt",
            ),
            e(
                "Land on it",
                "There is no mount button. Jump onto the tail, or drop onto its back from a platform, and you are on it.",
            ),
            e(
                "W A S D",
                "Aboard, these are relative to the surface under your feet. The creature turning turns you with it.",
            ),
            e(
                "Ctrl or C",
                "Aboard, crouch braces. It multiplies your grip, and it is the only defence up there -- there is no dodge on a back two metres wide.",
            ),
            e(
                "Space",
                "Leave, carrying whatever the creature was doing with you. The answer to the rear-and-slam, which nothing holds through.",
            ),
            e(
                "The red strip",
                "The ridge: unarmoured, and out of reach from the ground. Enough damage there puts the creature on its side.",
            ),
        ],
    },
    Section {
        title: "Player two, same keyboard",
        blurb: "For sitting next to someone. Set the training dummy to 4 first.",
        entries: &[
            e("Arrow keys", "Move."),
            e("Right Ctrl", "Jump."),
            e("; and '", "Turn left and right — player two has no mouse."),
            e(". , /", "Poke, guard, special."),
            e("L", "The class mechanic."),
            e("Right Shift", "Held with the above, the stronger version."),
        ],
    },
    Section {
        title: "Practice",
        blurb: "What the other fighter does while you work on something.",
        entries: &[
            s("1", "Dummy stands still.", "1-4 dummy"),
            e("2", "Dummy blocks."),
            e(
                "3",
                "Dummy attacks on a cadence, so the parry window is practisable.",
            ),
            e("4", "Dummy is a second player on the keys above."),
            s(
                "Tab",
                "Cycle player one's class. Restarts the match.",
                "Tab class",
            ),
            s(
                "F8",
                "Show or hide the class pickers beside each health bar. On under --dev. Clicking one cycles that player's class, and it needs a free cursor -- which is what the Oven gives you.",
                "F8 class pickers",
            ),
            s("R", "Reset the match.", "R reset"),
            s("P", "Pause.", "P pause"),
            s("]", "Step one frame. Pauses if it was running.", "] step"),
        ],
    },
    Section {
        title: "Looking at it",
        blurb: "The tools for working out why something happened.",
        entries: &[
            s(
                "F1",
                "Debug overlay: hitbox and hurtbox wireframes, guard arcs, facing.",
                "F1 debug",
            ),
            s(
                "F2",
                "Freeze the skeleton at rest. Tells a bad clip from a bad rig.",
                "F2 bind pose",
            ),
            s(
                "F7",
                "The Oven: every tuned number in the game, live.",
                "F7 oven",
            ),
            s(
                "F9",
                "The animation hub: every clip, editable while it runs.",
                "F9 animation",
            ),
        ],
    },
    Section {
        title: "Camera and feel",
        blurb: "Saved to ~/.config/arena/settings.conf as you change them.",
        entries: &[
            s(
                "- and =",
                "Mouse sensitivity, in multiplicative notches.",
                "- / = mouse",
            ),
            s("F3 and F4", "Field of view, 2 degrees a step.", "F3 F4 fov"),
            s(
                "F5 and F6",
                "Camera distance -- how big the fighter draws, 0.4 m a step.",
                "F5 F6 camera distance",
            ),
        ],
    },
    Section {
        title: "The Oven",
        blurb: "F7. Three hundred tuned numbers, grouped by family and searchable.",
        entries: &[
            e(
                "search",
                "Matches labels, families and identifiers. Opens what it finds.",
            ),
            e(
                "slider / ↺",
                "Change a value live. The arrow puts one back.",
            ),
            e("Revert", "Everything back to what is committed."),
            e(
                "Bake",
                "Write crates/sim/src/tuned.rs, commit it and push on the current branch.",
            ),
            e(
                "cargo run -p sim --bin bake_tuning",
                "The same write, without launching the game.",
            ),
        ],
    },
    Section {
        title: "The animation hub",
        blurb: "F9. Every clip in the game, editable while it runs. The kinematics are \
                handled; what you set is poses, when they happen, and the curve between them.",
        entries: &[
            e(
                "clip list",
                "Grouped by family. A clip nobody has authored says so, and opens anyway.",
            ),
            e(
                "timeline",
                "Drag a key to move it. Click anywhere else to scrub. The coloured lines on an attack are the last startup, the first active and the first recovery frame.",
            ),
            e(
                "add / delete / mirror key",
                "Adding takes the pose that was already on screen, so inserting a key never moves anything.",
            ),
            e(
                "copy / paste / paste mirrored",
                "A walk's second step is the first one mirrored, and so is half of everything else.",
            ),
            e(
                "hold key",
                "Freeze on the selected key, for posing. Onion draws the keys either side of it.",
            ),
            e(
                "timing out of this key",
                "The curve between this pose and the next: drag the two handles, or take a preset. Below the floor pulls back before it goes; above the ceiling carries past and returns.",
            ),
            e(
                "reach",
                "Drag the end of a limb and the joints follow. Level and toe are the two things a foot does on the floor.",
            ),
            e(
                "looseness",
                "Lag is how many frames behind the keys a part runs; ring is how far it carries past. Weight should read as follow-through, never as delay.",
            ),
            e(
                "save and bake",
                "Rewrites the recipe file in the shape a person would have written, then re-bakes in a fresh process -- which is also how you find out it compiles.",
            ),
        ],
    },
    Section {
        title: "Other binaries",
        blurb: "",
        entries: &[
            e(
                "cargo run -p sim --bin frametable",
                "Every move's frame data, on-block and on-hit advantage, air stats.",
            ),
            e(
                "cargo run -p anim --bin bake",
                "Re-bake the animation clips from their recipes.",
            ),
            e(
                "cargo run -p anim --bin bake_beast",
                "Re-bake the Ridgeback's pose table from its recipes in crates/anim/src/beast/clips.rs. Its parts are simulation geometry, so this one writes into crates/sim.",
            ),
            e(
                "cargo run -p anim --bin preview_beast -- <clip>",
                "Draw the Ridgeback as a contact sheet PNG, into target/beast-preview: the creature from the side, from above, and every frame overlaid. --all for every clip, --states for the poses the simulation produces rather than the baked ones. Green is a surface you can stand on, red is a weak point, and the dashed line is how high a full hop reaches.",
            ),
            e(
                "cargo run -p sim --bin beastcheck",
                "What the creature measures: how high every surface you can stand on is, standing and in each state that lowers one, against how high a fighter can actually jump. The climb is a geometry problem, and this is the geometry.",
            ),
            e(
                "cargo run -p anim --bin preview -- <clip>",
                "Draw a clip as a contact sheet PNG, into target/anim-preview. Add --feet for a per-frame table of what each foot is doing, or --all for everything.",
            ),
            e(
                "cargo run -p anim --bin export -- docs/preview/anim.json",
                "Write the skeletons and every baked frame out as JSON, for the browser bench in docs/preview.",
            ),
            e(
                "cargo run -p hunt --bin fight",
                "Play a scripted hunt and report on it: how much of what the creature throws can be answered on sight, how long the openings are, how varied its moves are, and how long anyone stays on its back.",
            ),
            e(
                "cargo run -p hunt --bin fight -- --class <name> --repeats <n> --trace --seed <n> --hunters <1|2> --frames <n>",
                "The same, with a different class, several seeds, or the play sequence printed move by move.",
            ),
            e(
                "cargo run -p net --bin soak",
                "Headless rollback soak: thousands of frames, checked for divergence.",
            ),
            e(
                "cargo run -p net --bin p2p_localhost",
                "Two real peers over UDP on localhost, checked for desync.",
            ),
        ],
    },
    Section {
        title: "Scripts",
        blurb: "",
        entries: &[
            e(
                "./scripts/screenshot.sh [out.png] [seconds]",
                "Render headlessly with Xvfb. No GPU needed.",
            ),
            e(
                "./scripts/p2p-localhost.sh",
                "Two game windows playing each other locally.",
            ),
            e(
                "./crates/web/build-sandbox.sh",
                "A self-contained browser frame-data tool.",
            ),
            e("./scripts/dev.sh", "The game in full development mode."),
            e("./scripts/help.sh", "This text."),
        ],
    },
    Section {
        title: "Environment variables",
        blurb: "Mostly for headless capture and for scripting comparisons.",
        entries: &[
            e(
                "DEMO=1",
                "Drive player one from a script instead of the keyboard.",
            ),
            e("DEBUG_OVERLAY=1", "Start with the F1 overlay on."),
            e(
                "SHOT_FRAME=<n>",
                "Run to exactly frame n and stop. Makes two captures comparable.",
            ),
            e("SHOT_PITCH=<radians>", "Start the camera at a known pitch."),
            e("SHOT_YAW=<radians>", "Start the camera at a known bearing."),
            e(
                "SHOT_DIST=<metres>",
                "Pull the camera in for a capture. The arena default of eleven metres makes a pose unreadable.",
            ),
            e(
                "BIND_POSE=1",
                "Start with the skeleton frozen at rest, for checking proportions.",
            ),
            e("OVEN=1", "Start with the Oven open."),
            e(
                "OVEN_SEARCH=<text>",
                "Start the Oven with a search already typed.",
            ),
            e(
                "ARENA_SETTINGS=<path>",
                "Use a different settings file. How two people share one machine.",
            ),
        ],
    },
    Section {
        title: "Working on it",
        blurb: "",
        entries: &[
            e(
                "cargo test --workspace",
                "Everything. Determinism, combat, feel, camera, the Oven.",
            ),
            e("cargo clippy --workspace --all-targets", "Lints."),
            e("cargo fmt --all", "Format."),
            e(
                "docs/design/README.md",
                "What is decided, open, and parked.",
            ),
            e(
                "docs/design/feel-log.md",
                "What was tried, and what it felt like.",
            ),
            e(
                "docs/design/monsters.md",
                "The Ridgeback: the fight, the ride, the control algorithm, and how the fight is measured.",
            ),
        ],
    },
];

/// The whole manual, as text.
pub fn render() -> String {
    let width = SECTIONS
        .iter()
        .flat_map(|s| s.entries.iter())
        .map(|e| e.invocation.len())
        .max()
        .unwrap_or(0)
        .min(32);

    let mut out = String::from("\nArena prototype — everything you can type\n");
    for section in SECTIONS {
        out.push_str(&format!("\n{}\n", section.title.to_uppercase()));
        if !section.blurb.is_empty() {
            out.push_str(&format!("  {}\n", section.blurb));
        }
        for entry in section.entries {
            if entry.invocation.len() > width {
                out.push_str(&format!(
                    "  {}\n  {:width$}  {}\n",
                    entry.invocation, "", entry.what
                ));
            } else {
                out.push_str(&format!("  {:width$}  {}\n", entry.invocation, entry.what));
            }
        }
    }
    out.push('\n');
    out
}

/// The compact on-screen legend, built from the same tables.
///
/// One group per section that has short forms, which is what stops the legend
/// and the manual from drifting apart: there is nowhere for them to disagree.
/// Wrapped at a readable width rather than one line per section — the fighting
/// controls alone are eleven entries and ran off the screen.
pub fn legend() -> String {
    const WIDTH: usize = 62;
    let mut lines: Vec<String> = Vec::new();
    for section in SECTIONS {
        let mut line = String::new();
        for short in section.entries.iter().filter_map(|e| e.short) {
            if line.is_empty() {
                line = short.to_string();
            } else if line.len() + 3 + short.len() <= WIDTH {
                line.push_str(" / ");
                line.push_str(short);
            } else {
                lines.push(std::mem::take(&mut line));
                line = short.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines.join("\n")
}
