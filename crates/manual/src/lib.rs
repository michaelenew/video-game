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
                "Jump. Hold it to go higher; releasing is final.",
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
                "Poke. The fast one. In the air it hangs you and shoves you the way you are holding.",
                "J poke",
            ),
            s(
                "Shift + J",
                "The committed attack. Slower, hurts, roots you.",
                "Shift+J committed",
            ),
            s(
                "K or right click",
                "Guard. The first few frames parry.",
                "K guard",
            ),
            s(
                "Q",
                "The class special. The fire pillar, the uppercut, the grapple — the move only that class has.",
                "Q special",
            ),
            s(
                "E or middle click",
                "The class mechanic. Different on every class: throw the shield, change form, place the shadow, raise a structure.",
                "E mechanic",
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
