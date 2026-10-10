//! Everything you can type, in one place.
//!
//! This is the **single source of truth** for controls, flags and environment
//! variables — not a document *about* them. A test checks that every key the
//! game actually handles appears here, and the browser build's controls panel
//! is generated from the same tables. A help text maintained separately from the thing it
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
}

pub struct Section {
    pub title: &'static str,
    pub blurb: &'static str,
    pub entries: &'static [Entry],
    /// Does this section describe something you can do in the browser build?
    ///
    /// **Declared, not inferred.** Every section has to answer, the way every
    /// move has to declare how it is aimed, because the alternative is a rule
    /// like "sections whose entries start with `cargo`" that is true until
    /// somebody writes a section it is not true of. What the browser cannot do
    /// is a short list -- a command line, a file, a peer -- and it is a list
    /// that changes, so it is written down rather than guessed at.
    pub in_browser: bool,
}

const fn e(invocation: &'static str, what: &'static str) -> Entry {
    Entry { invocation, what }
}

pub const SECTIONS: &[Section] = &[
    Section {
        title: "Running it",
        in_browser: false,
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
            e("cargo run -p game -- --help", "This text. `-h` also works."),
            e(
                "cargo run -p game -- --p1 <class> --p2 <class>",
                "Pick classes. Matched loosely: bulwark, champion, reaver, elementalist, blood, dual.",
            ),
            e(
                "cargo run -p game -- --bot <level>",
                "Start against the sparring bot: easy, normal or hard. 5, 6 and 7 switch to it in game.",
            ),
            e(
                "cargo run -p game -- --record",
                "Save a replay on its own at the end of every round, to the replays folder, named for the fight (ridgeback-champion-3412f-9c1a.replay). Y and the Esc menu's Save replay do the same for any moment. A replay is the fight's start and every frame's inputs, a few kilobytes a minute, exact: cargo run -p hunt --bin replay -- <file> judges it the way fight judges the scripted hunter, and says whether this build reproduced the fight bit for bit.",
            ),
            e(
                "cargo run -p game -- --replay <file>",
                "Play a replay back in the game: both seats' inputs come off the file and the camera follows the look it recorded, until it runs out and the controls are yours. A desktop only -- a page has no file to open. It has to be the build and the tuning the replay was made on, or the fight goes somewhere else from the same inputs; the file names both.",
            ),
            e(
                "cargo run -p hunt --bin duel",
                "Play the sparring bot against itself, every pairing, and count throws, hits and rounds. --level and --against set the two sides.",
            ),
            e(
                "cargo run -p game -- --hunt [creature]",
                "Start a hunt instead of fighting each other: the Ridgeback, or any creature that is built, by name, in its own arena. H switches either way in game, and Shift+H steps to the next creature. --hunt gnawers is the first pack creature: six knee-high biters and the Big One, in the Commons. --hunt hornback is a herd of eight cows and a bull in a meadow of boulders: stand a rock behind you and the charge stuns itself on it. --hunt hornback-escort is its crossing: walk a cart along a road the herd migrates over. --hunt mireback is the toad that takes the floor away: tar, fire, slag to climb its back by, and braziers to kick over, in the Mire. --hunt sandmaw is the worm that hunts by ear, in the Pan: it hears your footfalls and landings and feels you close, comes up where it heard you, and is only there when it stands -- make a noise where you want it, stand still to vanish, and beach it to climb its back. --hunt pair is two big cats that hunt as one, in the Den: one holds your front while the other goes where you are not looking -- keep both on screen, read the tail before you dodge a coil, dodge toward a pounce and across an ambush lane, and wait out the twin pounce until they leave the ground so they land on each other. --hunt broodmother is the spider and her clock, in the Hollows: six sacs on her back swell pale to red and burst into brood, a burst one grows back and a popped one never does, and the screech always leads to the slam that lays her sacs on the floor -- the window to pop them. --hunt veilstalker is the animal you never see, in the Ashwood: read what it touches -- three-toed prints in the snow and glowing in the ash, its breath, ripples in the stream -- and the paint your hits leave on its hide for six seconds; a decloak with no fresh prints under it is a mimic, so do nothing; walk out of the spear's lane and the pounce's circle, jump the rake, get a trunk between you and the quills, leave its smoke, and tip a brazier across its trail: fire makes it panic, and a hide hit enough stays mottled for good. --hunt mantis is the duellist, in the Shrine: it guards its front, parries what it sees coming -- late, through eyes fifteen to twenty frames behind -- coils and lunges when you commit, and takes a Ready stance against a move you keep throwing into its guard, marked as notches on its blades; break its guard inside its hold with an unblockable, hit its recoveries, and go round the side whose blade you broke. --hunt galewing is the raptor that lives where you cannot reach, on the Cliffs: dodge the Stoop at the hit and hit its wings while it is on the floor, crouch under the talon pass or swing at the wing going over, get into a stone's lee from the Downwash, leave the feather lane sideways, climb the tower while it rests on the perch -- and when it gathers itself to lift, choose: jump off, or ride it into the sky, brace on the spine through the wingbeats and the roll, and step off at the low swoop. Break a wing and it cannot climb; break both and it never flies again. --hunt siegeshell is the colossus built for two, in the Last Valley: a forty-metre shell on six tower legs walking to a town wall, and the walk is the clock -- jump the footfall ring on every beat, break two ankles on one side so it stumbles and its broken legs lie as a stair, climb to the crown while it is down, brace through the shrug and jump the shiver, and break its three anchors before it reaches the siege line and its beam breaches the wall twice; one of you on the legs and one at an anchor opens it. --hunt gnats is the dev pack: six small bodies and a queen, the critter machinery with nothing of its own. --hunt sentinel is the dev creature for the floor, the senses and the defended things: one of each hazard on the floor, a cone it sees in, a ring of noises it hears (F1 draws them), and in --arena range a gate and a cart. Neither dev species is in the Shift+H cycle or the trophy list; --hunt is the way to them. docs/design/review.md has every fight, how to start it and what to try first.",
            ),
            e(
                "cargo run -p game -- --arena <name>",
                "Fight in another arena: proving_ground (the default), a jump course (stair, causeway, spiral, falls, slalom, fork, spire, gulf, reach), lab (a dev arena for measuring movement: cargo run -p sim --bin envelope plays every class in it), or range -- a 240 m dev arena with one of everything an arena can have: sand, snow, a 12 m tower, a cave under a vault. With --hunt, the creature comes too.",
            ),
            e(
                "cargo run -p game -- --hunt <creature> --temper <n>",
                "Start the hunt at a temper, 0 to 3, whatever you have earned, and let T offer every temper. A temper is the same creature cleverer: it glances more often, leads you further, picks its best move more surely, and stays methodical as it tires. Its health and its hide do not change.",
            ),
            e(
                "cargo run -p game -- --port <n> --peer <ip:port>",
                "Peer-to-peer against someone else, on a LAN or wherever their address reaches. Rollback netcode, no server. Both of you start it with the other's address; --port defaults to the peer's port, and the same port on both machines is fine. The two say hello before the match: whoever started first is player one, and two different builds or two different --p1/--p2/--arena are refused rather than desynced. You practise while you wait; restarts, class changes, pausing and stepping are off for the match.",
            ),
            e(
                "cargo run -p game -- --join '<link>'",
                "Join a friend's room from the desktop: paste the link their page made, quoted, and it plays exactly as opening the link would -- the same room, classes and arena, through the same public brokers, over the same WebRTC connection. A desktop and a page meet as two pages do, so either can have made the room. It must be built from the commit the page was deployed from (the page's footer names it), or the room says the builds differ. Flags typed beside it win over the link's.",
            ),
            e(
                "cargo run -p game -- --room <name> --key <secret> [--broker <url>]",
                "The same, spelled out: --join is these, unpacked from the link. --key is the part after #key= in it, which seals the room: the brokers carry notes they cannot read, under a name that is not the room's. --broker <url> meets through one MQTT broker instead of the public three (ws:// for one on this machine). --board tabs is the browser's alone. --page <url> is the page a room made on this desktop links to (the published one by default), for a room on another deploy. --delay <frames> (?delay= in a page) holds your own inputs back that many frames, trading rollback for delay on your side only; the default is none, so online feels like local play.",
            ),
            e(
                "cargo run -p game --release -- --profile",
                "A frame-cost readout every two seconds, to the terminal: frame time, how many entities there are and how many meshes are on screen, and -- where the driver can time its own work -- what each render pass cost the GPU and how many triangles and pixels it drew. Beside it, the knobs that let one build be measured at several render settings: --cascades <n> and --shadow-reach <m> set the sun's shadow cascades (Bevy's defaults are 4 to 150 m), --shadow-map <px> the shadow map's size (2048), --shadow-filter hard|gaussian how its edge is softened (gaussian, thirteen taps a pixel; hard is one), and --no-outline, --no-fxaa and --no-prepass take a pass over the finished picture away, so its cost is what the frame time drops by. scripts/profile.sh runs it headlessly under lavapipe. What was found with it is in docs/design/architecture.md under The frame budget.",
            ),
            e(
                "cargo run -p manual",
                "This text, without building the game.",
            ),
            e(
                "cargo run -p manual -- --html",
                "The browser build's controls panel, from the same tables. Only the web build script calls it.",
            ),
        ],
    },
    Section {
        title: "In a browser",
        in_browser: true,
        blurb: "The same build. Play a friend with a link; the query string does what the flags do.",
        entries: &[
            e(
                "./crates/web/build-game.sh",
                "Build the page. Writes target/web, which is what GitHub Pages serves.",
            ),
            e(
                "?room=<name>",
                "Play a friend: the page's Play with a friend button makes one, and you send them the link. Both of you open it, and the two pages find each other through free public message brokers and then connect directly (WebRTC) -- no server of ours, and nothing of the match goes through anyone else. The link ends in #key=, a random secret that seals the room: the brokers see only notes they cannot read, under a name that is not the room's. A friend on a desktop joins the same link with --join. Whoever opened it first is player one; the classes and arena are the link's, so put ?p1=, ?p2= and ?arena= in it before you send it -- or change class once you are in, with Tab. You practise while you wait. Some networks, often mobile data, will not take a direct connection; the page says so.",
            ),
            e(
                "?room=<name>&board=tabs",
                "The same, between two tabs of this browser and nothing else: the room is a BroadcastChannel instead of a broker. For trying it alone, and for the smoke test.",
            ),
            e(
                "?room=<name>&broker=<url>",
                "The same, through one MQTT broker of your choosing instead of the public three: a wss:// URL that takes WebSocket connections (ws:// for one on this machine). Both of you need the same one.",
            ),
            e(
                "?p1=<class>&p2=<class>",
                "Pick classes, the same names the flags take. Tab still cycles player one in game, except in a match against a friend.",
            ),
            e(
                "?hunt or ?hunt=<creature>",
                "Start a hunt: the Ridgeback, or any creature that is built, in its own arena -- gnawers, hornback, hornback-escort, mireback, sandmaw, pair, broodmother, veilstalker, mantis, galewing, siegeshell. H switches either way in game, and Shift+H steps to the next creature.",
            ),
            e(
                "?arena=<name>",
                "Fight in another arena, exactly as --arena does: proving_ground, range, lab, or a jump course -- stair, causeway, spiral, falls, slalom, fork, spire, gulf, reach.",
            ),
            e(
                "?temper=<n>",
                "Start the hunt at a temper, 0 to 3, exactly as --temper does, earned or not.",
            ),
            e(
                "?bot=<level>",
                "Start against the sparring bot: easy, normal or hard.",
            ),
            e(
                "?dev",
                "Hitbox wireframes and the Oven, both open, exactly as --dev does.",
            ),
            e(
                "?shot_frame=<n>",
                "Any environment variable, spelled in the URL. Upper or lower case, dashes or underscores.",
            ),
            e(
                "Bake, and the hub's Save",
                "Both say so and stop: there is no checkout behind a web page. Tune live, then bake from a clone.",
            ),
        ],
    },
    Section {
        title: "Fighting",
        in_browser: true,
        blurb: "Camera-relative: W is away from the camera, not along a world axis.",
        entries: &[
            e("Mouse", "Aim. Where you look is where you are pointed."),
            e("Click / Esc", "Capture the mouse, and release it."),
            e(
                "Esc menu",
                "Esc steps out of the fight and brings up the menu; a click in the arena goes back. In it: Create a room, which makes a link for the fight you are practising, and a box to paste a friend's link and Join; while a room is active, its name, how meeting is going, the link with a Copy link button, and Leave. Save replay writes the fight so far -- every frame of it, from its start -- to a file (a download, in a browser) that cargo run -p hunt --bin replay judges. The creature list with your trophies and tempers is on the right, and only there while the menu is up.",
            ),
            e(
                "Y",
                "Save a replay of the fight so far, without opening the menu: the moment that felt wrong, kept after it happened. On a desktop it goes to ~/.config/arena/replays/ (or the folder ARENA_REPLAYS names), in a browser it downloads; the terminal or the console says where. Online, both players' inputs are on it as they were confirmed. cargo run -p hunt --bin replay -- <file> puts it through the harness, and cargo run -p game -- --replay <file> plays it back in the game.",
            ),
            e("W A S D", "Move, relative to the camera."),
            e(
                "Space",
                "Jump. Hold it to go higher; releasing is final. On the Champion, pressing a weapon inside the first few frames of a jump throws that weapon's takeoff instead of an aerial. On the Dual mage, a second press in the air jumps again while the lower of her two bars is at three quarters, and every press is a wing beat while she is ascended.",
            ),
            e(
                "Shift + direction",
                "Dodge, and shift's only job. Airborne, an airdodge — once per jump. It used to also be the attack modifier, and a modifier whose meaning depends on what else your hand is doing is one you cannot trust. On the Dual mage with the lower bar at half it is a blink: you are where the dodge would have ended, at once, and you stand through its tail.",
            ),
            e("Ctrl or C", "Crouch. Ducks overheads, costs you speed."),
            e(
                "J or left click",
                "Poke. The fast one. In the air it hangs you and shoves you the way you are holding. On the Champion it is the sword, and held down it walks that weapon's whole three-hit string.",
            ),
            e(
                "Q",
                "The committed attack, on most classes: the class special, slower and heavier, and it slows you to a crawl -- you keep the stick, you lose the jump and the dodge until it is over.",
            ),
            e(
                "K or right click",
                "Guard. The first few frames parry. Three classes spend it instead, none of them having a shield to raise: the Champion's spear, the Dual mage's light, and the Shadow Reaver's shadow -- hers is the one the crosshair aims, so it goes on the hand doing the aiming.",
            ),
            e(
                "U or middle click",
                "The third attack button. Three classes use it: the Champion's is the hammer, the Dual mage's is twilight -- both her forces at once, Binary on the floor and Phase in the air -- and the Elementalist's is fire: the pillar on the floor, the carpet in the air.",
            ),
            e(
                "Mouse side buttons (or I and O)",
                "Two more ability buttons, under the thumb of the hand that aims -- so they carry aimed things. Only the Elementalist reads them so far.",
            ),
            e(
                "F and R",
                "Two more ability buttons, under the index finger of the hand that moves -- one row up from D, the way Q and E sit one row up from A and W. They carry things about your own body rather than a place. Only the Elementalist reads them so far.",
            ),
            e(
                "Q",
                "The class special. The fire pillar, the grapple — the move only that class has. The Champion has none: its three weapons are its three clicks.",
            ),
            e(
                "E",
                "The class mechanic. Different on every class: throw the shield, Rush, raise a structure. Three classes put a real ability here instead, with a wind-up you can be punished during — and on the Shadow Reaver it is not even the mechanic, because hers went to right click.",
            ),
        ],
    },
    Section {
        title: "The Champion",
        in_browser: true,
        blurb: "Three weapons on three clicks, a three-hit string where every hit is a free choice of weapon, and a dash that changes what all three of them do. The button is the weapon; the situation picks the move.",
        entries: &[
            e(
                "Left click",
                "Sword. A diagonal cut, corner to corner, and it steps in as it swings. Fastest thing in the class at both ends, and the one you can keep walking during — the move you string with.",
            ),
            e(
                "Middle click (or U)",
                "Hammer. Arc down to the floor. Slow, short, and it staggers longer than anything else — the move you start with.",
            ),
            e(
                "Right click",
                "Spear. A one-armed jab straight ahead. Longest reach in the game, it goes over anyone crouching, and it leaves you standing exactly where you were.",
            ),
            e(
                "Keep swinging",
                "Land a hit and the same three buttons throw the second of three, then the third. Every hit is a free choice of all three weapons, so sword into spear into hammer is an ordinary thing to do. The string flows only while you are connecting: blocked or thrown at nothing, you pay the whole recovery. It ends if you stop for half a second, are hit, block, dodge, or leave the ground.",
            ),
            e(
                "The third hit",
                "Upcut rises and pops them off the floor. Earthbreaker goes through a guard and throws them up. Whirl sweeps the shaft round your whole body, low, and is the only attack in the game that threatens behind you. All three commit your feet.",
            ),
            e(
                "Jump during Earthbreaker",
                "Press space while the hammer's third hit is winding up and you go up with them: the knock-up is bigger and you leave the floor on the frame it lands, which puts the air hammer in reach off the end of an ordinary string. It is paid on contact, so a whiff costs you the recovery and gives you nothing.",
            ),
            e(
                "Space and a weapon",
                "That weapon's takeoff, thrown as your feet leave the floor. Sword rises into an angled slash and hits hardest; hammer is the uppercut, which launches and holds on, and space again takes you both higher; spear cracks the shaft into the ground for the most height in the class plus a shove the way you are holding. Press jump first and the weapon a few frames later — that order always works.",
            ),
            e(
                "In the air",
                "The same three buttons, different moves. Sword cuts downward; hammer winds up slowly and spikes an airborne target into the floor; spear fans around the aim and shoves you the way you are holding if it connects.",
            ),
            e(
                "E",
                "Rush. A dash on one charge, and it cancels any recovery. It does not end a string — one charge buys you a reposition in the middle of one.",
            ),
            e(
                "While rushing",
                "Sword cuts as you run past without stopping the dash. Hammer drags along the floor and takes the legs of anyone you pass. Spear stabs for the biggest single hit in the class, or vaults if you are pointing at the floor.",
            ),
        ],
    },
    Section {
        title: "The Dual mage",
        in_browser: true,
        blurb: "Two forces, one in each hand, and a bar for each -- and every move a spell. Left is dark, the hex: things that travel and linger. Right is light, the strike: things that happen now. Middle is twilight, both at once. A dark spell hexes what it hits, a light one hexes it the other colour, and the other force on a hexed body sets it off -- so alternating your hands is both the combo and the climb. Rebuilt 2026-10-09 and unplayed; docs/design/exploration/0010_dual_mage_spells.md is the design.",
        entries: &[
            e(
                "Left click",
                "Dark. On the floor, the Shade bolt: a small dark dart from your left hand along the crosshair -- quick, but in flight, so it can be stepped out of. In the air, Reel: the same dart, and if it hits somebody it pulls you to them. Space and left click is Nightfall: you jump, and a small dark well blooms where you left the floor, dragging whoever is near it in. Every dark spell feeds the dark bar and hexes what it hits with Umbra (a dark orb over their head).",
            ),
            e(
                "Right click",
                "Light. On the floor, the Sunray: an instant short beam from your right hand to the first thing on the crosshair's line -- already there, so it is blocked rather than dodged. In the air, Flare: a burst of light a few metres along your aim, and if it meets the floor, a wall or a body it kicks you back the other way -- aimed down, a lift. Space and right click is Dawn: a column of light that throws whoever is beside you up with you. Every light spell feeds the light bar and hexes Radiance (a light orb).",
            ),
            e(
                "Middle click",
                "Twilight, both at once: it feeds both bars and is worth your lower one. On the floor, Binary: two orbs, one of each force, wound round each other along the crosshair. In the air, Phase: you blink a few metres along the crosshair, stopped by walls, and a flash hurts whoever was beside you where you were -- it spends your airdodge. Space and middle click is Equinox: a jump as high as your lower bar is full.",
            ),
            e(
                "Q",
                "The Abyss, the dark major: a well where the crosshair meets the floor. For a second it drags everything in it toward its middle, hurts, gives you some of it back and hexes Umbra. Feeds the dark bar twenty.",
            ),
            e(
                "E",
                "Judgement, the light major: a delayed strike where the crosshair is, and a burning field that makes you fast while you stand in it. It hexes Radiance -- and sets off every Umbra hex it hits, which is what makes it the end of an Abyss. Feeds the light bar twenty.",
            ),
            e(
                "Hexes: Shatter and Wither",
                "One hex on a body at a time, and it fades on its own. The same force again only refreshes it. The other force sets it off: light on an Umbra hex is a **Shatter** -- a burst of damage and a stagger; dark on a Radiance hex is a **Wither** -- damage drained back to you and a slow. Binary on a clean body sets off both at half. So the bread and butter is dark, light, dark, light on one body.",
            ),
            e(
                "Each bar is its spells' power",
                "Everything you throw is worth what the bar of its force says, all the way along it: empty is thin, full is the most you can hold -- damage, the reactions, how big the well is. A middle-click spell reads your lower bar. What never changes is the frame data.",
            ),
            e(
                "The hill",
                "The two bars are compared every frame. Level -- within sixteen of each other -- nothing moves. Further apart than that, the higher one rises and the lower one falls, faster the wider the gap, and you burn. Both bars fall slowly on their own, always, so a height you stop feeding is a height you lose. Middle click feeds both and never widens the gap.",
            ),
            e(
                "Blink, second jump, wings",
                "The **lower** bar is what your body can do. At half the dodge is a blink. At three quarters you have a second jump and you fall slower -- which is also why your takeoffs are from the floor only: in the air, space is that jump. Both full and you ascend: wings, a wing beat on every press of space, no dodge, six seconds of heavy drain that landing hits pays back, and then both bars empty and a stagger.",
            ),
            e(
                "On your back",
                "The two bars are six wings, three a side, dark on the left and light on the right. A wing appears whole at each tick on that side of the bar. Lopsided wings are a mage about to burn; three and three is a mage about to fly.",
            ),
        ],
    },
    Section {
        title: "The Shadow Reaver",
        in_browser: true,
        blurb: "Two bodies. The shadow is never away — it is at your shoulder or out on the field — and everything the class does is a function of the line between the two. Her three clicks are the blade, the execution and the shadow: each has a move on the floor, one in the air and a jump attack with space. Rebuilt on three clicks 2026-10-09 and unplayed; docs/design/exploration/0011_shadow_reaver_on_three_clicks.md is the design.",
        entries: &[
            e(
                "The shadow copies you",
                "Whatever you swing, it swings a few frames later for a quarter of the damage. Held at your shoulder that is a quarter again on everything; sent out, it is a second threat somewhere you are not.",
            ),
            e(
                "Left click",
                "The blade. Slash on the floor, the cut the tally is built on. In the air, the Kite cut: a hit on a body carrying marks gives your airdodge back -- and the airdodge pointed at the shadow is the dash. Space and left click is the Moonsault: a back flip up and away whose blade launches whoever was pressing you.",
            ),
            e(
                "Middle click",
                "The execution. Executioner on the floor, the committed overhead that cashes a tally for the most. In the air, the Guillotine drop: straight down blade first, spiking anybody under you into the floor. Space and middle click is the Gallows: you are gone upward, hang a beat, and come down blade first.",
            ),
            e(
                "Right click",
                "The shadow. Send it where you are pointing, fast, and it stops there. Press again and it dashes home through anything in the way, cutting and slowing it — and taking an open Guillotine lotus with it. It answers whatever else you are doing: pressed during the tail of another move it cuts that tail short, and pressed a few frames early it is remembered rather than dropped. In the air with the shadow out, right click is the Swap: you and it trade places. Space and right click hangs the shadow in the air where you point; it waits there a moment, then sinks to the floor.",
            ),
            e(
                "Q",
                "Guillotine lotus. Six blades erupt from the shadow, hang open, and chase it home — so recalling the shadow with right click drags them the length of the arena.",
            ),
            e(
                "E",
                "Deadly mistake, a short counter stance. Struck in it by a fighter, you take nothing: you are behind them, and your shadow is left standing where you were. Whiffed, it is a long recovery.",
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
        title: "The Elementalist",
        in_browser: true,
        blurb: "Terrain author. Her three clicks are the class: left is earth, middle is fire, right is wind -- each a tap and a hold on the floor, something else in the air, and a jump attack with space. Q and E are the two pushes she uses on what she built. Built 2026-10-09 and unplayed; docs/design/exploration/0008_elementalist_on_three_clicks.md is the direction.",
        entries: &[
            e(
                "Left click",
                "Earth. Raise a stone where the crosshair is -- under your own feet included, and jump as it erupts to ride it up. Clicked out past an edge it stays up at your level; pointed at the floor below, it goes down there. Three on the field; a fourth collapses the oldest. **Hold it past the rise** and the stone stays churning while you crawl; let go and Fissure races from it toward the crosshair, as far as you held for, and the stone erupts at the crack's end. In the air it is Landfall: a plunge that levers a leaning slab out of the floor in front of you.",
            ),
            e(
                "Middle click (or U)",
                "Fire. The fire pillar, planted where the crosshair is; one at a time. **Hold it past the wind-up** and the fire gathers in your hands: you crawl, and letting go lands the pillar's whole burn as one Strike. In the air it is the Fire carpet: a strip of fire laid out in front of you along your look. Fly a Gale down it and the Gale is lit; Updraft into it and it is a Thermal.",
            ),
            e(
                "Right click",
                "Wind. The Air ball: raised on the floor where the crosshair is, growing for as long as you hold, and sent toward the crosshair when you let go. It shrinks as it rolls and everything standing in it goes with it -- you included; jump out and you keep its speed, walk sideways in it (A and D) and you steer it. Off an edge it sinks slowly and keeps carrying you; a wall or a stone knocks it off at an angle and costs it some size. The bigger you let it grow, the faster and further it goes. In the air it is the Gale, a disc that opens as it flies and hits hardest wide; it shoves stones, and through fire it comes out burning.",
            ),
            e(
                "Space + a click",
                "The jump attacks. Space and left click is the earth jump: a jump that brings a stone up with you -- keep going straight and you land on it in the air; off a stone on the floor the stone shatters and you go higher (running, look straight down and left-click to put a stone under your feet, then space and left click at once: the big jump, keeping all your run); off a stone in the air it is driven down to shatter. Space and middle click is the Fire fountain: a burst at your feet and a wash of fire left where you took off. Space and right click is the Updraft: a column of air that lifts you and anyone beside you and keeps your run -- and in the air, holding space, once a jump, it stretches the jump. An Updraft that meets your own fire is a Thermal: much higher, and along the carpet if it was the carpet.",
            ),
            e(
                "Q",
                "The weak push. Bolt: an instant line to whatever the crosshair is on -- it kicks a stone along the line and lights a fire bolt out of a pillar, and passes through people. The same in the air: Q is always the weak push.",
            ),
            e(
                "E",
                "The strong push. Cataclysm: a slow heavy along the same line that breaks a stone into debris or tears a pillar loose as a tornado -- on the floor and in the air alike.",
            ),
            e(
                "F",
                "Cinder spray, standing: a thrown coal that bursts into a cloud of embers -- fire for your shots to fly through. In the air it is Downdraft: a column under you driving you and everything in it down. Land while it blows and the air breaks outward from your feet; land it into fire and a ring of fire races outward instead.",
            ),
            e(
                "Mouse side button B (or O)",
                "Quake, where the crosshair is on the floor. The patch shakes through a slow wind-up: anyone moving through it staggers, anyone standing still is fine. Then it erupts -- damage to everyone in it -- and leaves a stone at its centre.",
            ),
            e(
                "R",
                "Tremor: Quake on your own feet. The same shake and the same eruption, and the stone comes up under you and takes you with it -- the structure jump with a telegraph attached, for when somebody has closed on you.",
            ),
            e(
                "Shift + direction, crosshair on a stone",
                "The dodge breaks through it: the stone is gone as you pass, its slot is free again, and where it stood is rough ground -- burning ground and a cloud of embers, if it was lit. In the air it is the airdodge, and costs the airdodge.",
            ),
            e(
                "Fire on a stone",
                "A pillar cast on a stone, a Cinder spray bursting beside one, or a burning Gale passing one sets it alight for a while. A lit stone burns whoever stands on it, and bursts into burning debris and a cloud of embers when anything shoves or breaks it. Earth builds the field; fire decides who may use it.",
            ),
        ],
    },
    Section {
        title: "The Blood mage",
        in_browser: true,
        blurb: "Her blood goes out and theirs comes back. Her three clicks are my blood, your blood and the scythe -- each a move on the floor, something else in the air, and a jump attack with space. Her own pools are doors, not heals: only somebody else's blood closes her wounds. Built 2026-10-09 and unplayed; docs/design/exploration/0009_blood_mage_on_three_clicks.md is the direction.",
        entries: &[
            e(
                "Left click",
                "My blood. Hold to charge the Blood nova, paid in your own health as you hold; let go and a sphere of blood bursts round you, hurts and throws back whoever is close, and leaves a pool of your blood where you stood. In the air it is the Haemorrhage: a bolt that opens a bleed, and a bleeding body -- fighter or creature -- drips pools as it goes.",
            ),
            e(
                "Middle click",
                "Your blood. Hold for reach and four arms close on the crosshair: all four catch, and the victim is hauled to your feet -- or, if they close on a wall, a stone or the creature, you are hauled to it. In the air it is the Nail: a long black spike thrown down the crosshair. Nail somebody who is already in the air and they are pinned there for a moment, bleeding on to the floor below.",
            ),
            e(
                "Right click",
                "The scythe. The Reaping sweep, whose reach grows with your grey health. In the air it is the Hook: thrown along the crosshair, it catches the first fighter, creature, stone or wall and pulls you to it.",
            ),
            e(
                "Space + a click",
                "The jump attacks. Space and left (held) is the Blood jet: you leave the floor on a jet of your own blood, further and harder the longer you hold, paid as you hold; whoever is in its wake is hurt, and a pool of your blood is left where you took off -- blink back to it. Space and middle is Marionette: their blood lifts them into the air (and you after them) -- Nail them up there. Space and right is Harvest: a big jump inside a scythe spin that drinks the pools it passes over.",
            ),
            e(
                "Q",
                "The Bloodletter: a blade thrown out a fixed distance and back, cutting on both passes. In no combo yet, and the candidate to cut once she has been played.",
            ),
            e(
                "E",
                "The Black spike: a spike out of the floor after a delay. On a pool, the pool erupts, and on a bleed's trail it chains along the pools to the bleeding body.",
            ),
            e(
                "Shift + direction, crosshair on a pool",
                "The blink: you are at the pool, and it is spent. Your own pools are for this -- they never heal you.",
            ),
        ],
    },
    Section {
        title: "The Bulwark",
        in_browser: true,
        blurb: "The wall. Every blow taken on the shield is stored in it as weight, and his moves spend it. His three clicks are the strike, the weight and the guard: each has a move on the floor, one in the air and a jump attack with space. His air game is about being thrown -- the shield is a springboard, a sail and a battery. Rebuilt on three clicks 2026-10-10 and unplayed; docs/design/exploration/0012_bulwark_on_three_clicks.md is the design.",
        entries: &[
            e(
                "Left click",
                "The strike. Bash on the floor, the safe poke. In the air, Rebound: a Bash that throws you back off anything it meets -- a body, a wall, a stone, your own planted shield. Space and left click is the Battering ram: a low leap forward, shield first, carrying whoever it meets.",
            ),
            e(
                "Middle click",
                "The weight. Slam on the floor and in the air, where it waits for your feet and counts the fall. Space and middle click is Unload: the shield driven into the floor, and the stored weight throws you upward -- higher the more it held. It spends everything.",
            ),
            e(
                "Right click",
                "The guard, held; its first frames parry, and every blow it takes is weight. Held in the air it is a sail: you fall slowly behind it. Space and right click is the Shield step: the shield is planted where you stand, and you spring off its top -- higher the heavier it is. You have no shield until you call it home.",
            ),
            e(
                "Q",
                "Grapple, the command grab. It beats a guard and loses to a dodge.",
            ),
            e(
                "E",
                "The shield: thrown when it is in your hand, recalled when it is planted -- hitting whoever is in the way home -- and, while it flies, a leap to it. No frames: it happens on the press.",
            ),
        ],
    },
    Section {
        title: "The valley",
        in_browser: true,
        blurb: "A run starts in Hearth, the walled town at the valley's mouth: walk out of its gates into a world of places joined together, a climb from the river to the Saddle with every creature in a room off the way, and the Ring in town for fighting each other. docs/design/valley.md has the whole map.",
        entries: &[
            e(
                "Walk into a way out",
                "A notch in a wall, a gate, a passage at the end of a reach, glowing: the way on. Both of you walk in and you go on together; one of you alone holds it for two seconds and takes the other along. The line at the top says where it leads.",
            ),
            e(
                "Waystones",
                "The ways up the valley are under waystones, lit by beating the creatures below them: one of the herd and the den opens the Shelves, one of the Pan and the Mire the Pinewood, two of the Pair, the Broodmother and the Ridgeback the Saddle, one of the Galewing and the Veilstalker the Shrine, and the Mantis the Long Valley. Your trophies count, and so do your friend's: the valley is open as far as the more travelled of you has been. Going back down is never shut.",
            ),
            e(
                "Rooms",
                "Walking into a creature's room starts its hunt. Win, and the place goes quiet: walk out the way you came. Lose, and you wake outside at the way you went in.",
            ),
            e(
                "Climbing",
                "Shelves, gaps, traverses along a face, chimneys between two walls -- the jump is all there is to it, and a fall costs what the fall rule says: free to nine metres, twenty-five a metre past that. A long way round always exists. Snow on a top is a cairn: touch it to rest, and if you die you are stood on the last one you touched.",
            ),
            e(
                "Vines and updrafts",
                "Green strands down a face are a vine: hold Space to climb it, crouch to slide down, let go of both to cling. A pale column of air on the Saddle is an updraft: jump into it and it carries you up.",
            ),
            e(
                "The Ring",
                "Through the north door of the town: the one place in the valley the two of you can fight each other, round by round, on the proving ground's floor plan. The training dummy and the sparring bot are player two here when nobody has the second keys. Hop the south wall and walk into the door to leave.",
            ),
            e(
                "V",
                "Back to the valley: Hearth's square, with what you have beaten kept. From a hunt, a course or a versus match started any other way, this is how you get to the valley. Online, both players go together.",
            ),
            e(
                "--versus, ?versus",
                "Start in the proving ground fighting each other, the way a run started before there was a valley. --arena and --hunt start where they say, as they always have.",
            ),
            e(
                "--open, ?open",
                "Every waystone lit, as if everything had been beaten: for walking the whole valley without the fights.",
            ),
            e(
                "--arena mouth, ?arena=mouth",
                "Start somewhere else in the valley by name: hearth, ring, mouth, bank, shelves, pinewood or saddle.",
            ),
        ],
    },
    Section {
        title: "Hunting the Ridgeback",
        in_browser: true,
        blurb: "H starts a hunt. Its back is the only part worth hitting, so the fight is about getting up there.",
        entries: &[
            e(
                "H",
                "Hunt the Ridgeback, or go back to fighting each other. Restarts the match either way.",
            ),
            e(
                "Shift+H",
                "Hunt the next creature there is, in its own arena: the Ridgeback and the ten after it, round and back. The dev species (gnats, sentinel) are not in the cycle; --hunt reaches them by name. Online, both players go together.",
            ),
            e(
                "T",
                "Hunt the same creature again at the next temper you have earned, and round to as tuned. Beating a creature earns its trophy at that temper and offers the next one up; the list on the right shows both. Your trophies are kept on this machine (a file beside your settings, or this browser's storage), and online each player keeps their own.",
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
        title: "Jump courses",
        in_browser: true,
        blurb: "Islands of rock hanging over a long drop, nothing to fight: the movement system is the challenge. Two easy, four hard, two barely possible -- every tier a guess until somebody plays them; docs/design/courses.md has the list.",
        entries: &[
            e(
                "N",
                "The next jump course, in order of difficulty: the Stair and the Causeway (easy); the Spiral round a great pillar, the Falls over an arch and down a waterfall of stones, the Slalom between pillars and through a cave mouth, and the Fork with its high and low roads (hard); the Spire and the Gulf (barely possible); then the Reach, a proving ground of marked distances for trying each class's mechanics. The hard courses and the Gulf have big jumps for the mechanics too; the course panel says where. From anywhere else, the first. Online, both players go together.",
            ),
            e(
                "--arena stair, ?arena=stair",
                "Start in a course by name: stair, causeway, spiral, falls, slalom, fork, spire, gulf or reach. With --p1 (or ?p1=) for the class.",
            ),
            e(
                "Falling",
                "Below the islands is a long way down. Fall and you are stood back on the last checkpoint you reached, fresh, with whatever you had out taken back; the clock keeps running. Backspace starts the course again.",
            ),
            e(
                "The panel on the right",
                "Every course, the one you are in, the last checkpoint you reached, the clock from leaving the start, and how often you fell. The clock stops at the finish: the nest on the last island.",
            ),
        ],
    },
    Section {
        title: "Player two, same keyboard",
        in_browser: true,
        blurb: "For sitting next to someone. Set the training dummy to 4 first.",
        entries: &[
            e("Arrow keys", "Move."),
            e("Right Ctrl", "Jump."),
            e("; and '", "Turn left and right — player two has no mouse."),
            e(". , /", "Poke, guard, special."),
            e("L", "The class mechanic."),
            e(
                "M",
                "The third attack button — there is no scroll wheel on this half of the keyboard.",
            ),
            e("Right Shift", "Held with a direction, dodge."),
        ],
    },
    Section {
        title: "Practice",
        in_browser: true,
        blurb: "What the other fighter does while you work on something.",
        entries: &[
            e("1", "Dummy stands still."),
            e("2", "Dummy blocks."),
            e(
                "3",
                "Dummy attacks on a cadence, so the parry window is practisable.",
            ),
            e("4", "Dummy is a second player on the keys above."),
            e(
                "5 6 7",
                "Dummy is a sparring bot -- easy, normal, hard. It sees late, aims imperfectly, and picks a new personality every time you press one.",
            ),
            e(
                "Tab",
                "Cycle your class. Restarts the fight. Works against a person too: both games restart on the same frame, with your new class and whatever they have.",
            ),
            e(
                "F8",
                "Hide or show the class pickers beside each health bar. On by default. Clicking one cycles that player's class -- against a person, only your own -- and it needs a free cursor: press Esc, or open the Oven.",
            ),
            e(
                "Backspace",
                "Reset the match. It was R, until R became an ability key. Against a person it resets both games, on the same frame.",
            ),
            e(
                "P",
                "Pause. Against a person it pauses both games, and either of you carries on.",
            ),
            e(
                "]",
                "Step one frame. Pauses if it was running. Against a person it steps both games.",
            ),
            e(
                "[",
                "Step one frame BACK, about two and a half seconds' worth. Pauses if it was running. Local play only -- a peer is not rewinding with you. While paused, the readout under the crosshair says what the frame is doing: where her feet are, how fast she is rising, and every structure's age and climb rate.",
            ),
            e(
                "G",
                "--dev only. Play the double structure jump: two structures three frames apart and a jump nine frames after the first, which is the only shape that gets a third takeoff. It is a 50 ms double tap that nobody lands by hand, so this is how you watch one -- at speed, or paused and stepped through with [ and ].",
            ),
        ],
    },
    Section {
        title: "Looking at it",
        in_browser: true,
        blurb: "The tools for working out why something happened.",
        entries: &[
            e(
                "F1",
                "Debug overlay: hitbox and hurtbox wireframes, guard arcs, facing.",
            ),
            e(
                "F2",
                "Freeze the skeleton at rest. Tells a bad clip from a bad rig.",
            ),
            e("F7", "The Oven: every tuned number in the game, live."),
            e(
                "F9",
                "The animation hub: every clip, editable while it runs.",
            ),
            e(
                "F10",
                "Dev mode on or off, mid-session: what --dev turns on, without relaunching. The overlay, and in the Esc menu the connection details for a room -- each meeting point's connection step by step, who has been heard from, the direct line -- with a Copy details button. A room that fails prints the same details to the console (the browser's, or the terminal) whether or not it is on.",
            ),
        ],
    },
    Section {
        title: "Camera and feel",
        in_browser: true,
        blurb: "Saved to ~/.config/arena/settings.conf as you change them.",
        entries: &[
            e("- and =", "Mouse sensitivity, in multiplicative notches."),
            e("F3 and F4", "Field of view, 2 degrees a step."),
            e(
                "Page Up and Page Down",
                "Volume, a tenth a step. Every sound is made from the fight's own numbers -- a blow's weight is its impact freeze, a telegraph is as long as its startup, a footfall is the floor it lands on -- and nothing is recorded (docs/design/sound.md). cargo run -p sound --example sheet renders the whole voice to WAVs and a sheet of spectrograms without the game.",
            ),
            // Camera distance was on F5 and F6 and is not a setting any more:
            // it is the framing sphere's radius, the radius decides where the
            // eye is, and the eye is where the aiming ray starts -- so two
            // players with different distances would place the same fire pillar
            // in different spots. It is tuned in the Oven under Camera. The
            // entry outlived the binding by long enough that the help was
            // offering a key nothing answered; `a_documented_function_key_does
            // _something` is what fails next time.
        ],
    },
    Section {
        title: "The Oven",
        in_browser: true,
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
                "Write crates/sim/src/tuned.rs and every creature's own tuned.rs, commit them and push on the current branch.",
            ),
            e(
                "cargo run -p sim --bin bake_tuning",
                "The same write, without launching the game.",
            ),
            e(
                "cargo run -p sim --bin bake_tuning -- --set <id>=<value>",
                "Set knobs by the identifier their line in a baked file carries, in the unit it shows, then bake: the slider from a terminal. How a new creature gets its first numbers without a window. Repeat --set for as many as you like.",
            ),
        ],
    },
    Section {
        title: "The animation hub",
        in_browser: true,
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
        in_browser: false,
        blurb: "",
        entries: &[
            e(
                "cargo run -p sim --bin frametable",
                "Every move's frame data, on-block and on-hit advantage, air stats. For the Dual mage, the tiers and the numbers of the hill between her bars too.",
            ),
            e(
                "cargo run -p sim --bin goad",
                "The Dual mage's two bars, driven by named input scripts -- alternate, one-sided, finisher, idle, ascend -- in an empty arena and against a dummy. Prints when each tier is reached, when the band is left, how long the lower bar lasts outside it, what the burn cost, and whether she ascended. Name one script for it frame by frame.",
            ),
            e(
                "cargo run -p sim --bin weight",
                "The Bulwark's shield weight, driven by named scripts -- openers, load, decay, pushback, stomp, slam, wall. What each class's opener deposits blocked and parried, a sword string loading it, a full shield draining on its clock, the shove at five weights, and each creature move into a raised guard. Name scripts to run only those.",
            ),
            e(
                "cargo run -p sim --bin tally",
                "The Shadow Reaver's v2, driven by named scripts -- range, stall, pattern, greedy, stick. Prints how many of the shadow's copies land at range with and without its self-aim, how a full tally fades, what the whole send-mark-cross-cash pattern deals and how long it takes, and what melee with the shadow at her heel is worth. Name one script for it frame by frame.",
            ),
            e(
                "cargo run -p anim --bin bake",
                "Re-bake the animation clips from their recipes.",
            ),
            e(
                "cargo run -p anim --bin bake_beast",
                "Re-bake every creature's pose table from its recipes in crates/anim/src/beast/<species>/, each into its own crates/sim/src/species/<species>/baked.rs. Its parts are simulation geometry, so this one writes into crates/sim. --species <name> for just one.",
            ),
            e(
                "cargo run -p anim --bin preview_beast -- <clip>",
                "Draw a creature as a contact sheet PNG, into target/beast-preview: the creature from the side, from above, and every frame overlaid. --all for every clip, --states for the poses the simulation produces rather than the baked ones, --species <name> for a creature other than the Ridgeback. Green is a surface you can stand on, red is a weak point, and the dashed line is how high a full hop reaches.",
            ),
            e(
                "cargo run -p sim --bin essence",
                "What the Blood mage measures: the pool each move leaves and how long it lives, what each move drinks when it lands over a pool, the grey bar over a scripted exchange, and the scythe's reach at each level of grey. The first thing to run after touching the class.",
            ),
            e(
                "cargo run -p sim --bin elemental",
                "What the Elementalist's v2 measures: what a hold of Q or E is worth and costs in ground, how high the Updraft lifts each class, what the two rings do at each distance, what fire does to an air shot, where Quake and Tremor put their stone, and what a dodge into a stone leaves. Six scripts, each also runnable alone: charge, lift, ring, spray, quake, break.",
            ),
            e(
                "cargo run -p sim --bin beastcheck",
                "What a creature measures: how high every surface you can stand on is, standing and in each state that lowers one, against how high a fighter can actually jump. The climb is a geometry problem, and this is the geometry. --species <name> for a creature other than the Ridgeback; everything it prints is read off the species' table.",
            ),
            e(
                "cargo run -p sim --bin critcheck",
                "Can every move touch something short? Stands one critter of a pack at 1, 2, 3 and 5 m in front of each class, crosshair on its middle, presses every move, and prints where it touched -- beside where the same move touches a fighter standing there, which is what it is held to. The dev pack's 0.6 m gnat by default; --species <pack> --kind <name> for another.",
            ),
            e(
                "cargo run -p anim --bin preview -- <clip>",
                "Draw a clip as a contact sheet PNG, into target/anim-preview. Add --feet for a per-frame table of what each foot is doing, or --all for everything.",
            ),
            e(
                "cargo run -p anim --bin audit",
                "Does the animation match the hit volume? Throws every attack of a class (--class <name>; the Champion by default, or name a clip), draws the volume the hit test has out on each frame over the clip's contact sheet in target/anim-audit, and prints how far each hand is off the volume's axis, how far the line through the hands is turned from it, and the worst per move.",
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
                "cargo run -p hunt --bin fight -- --class <name> --repeats <n> --trace --seed <n> --hunters <1|2> --frames <n> --species <name> --temper <n> --gamble",
                "The same, with a different class, several seeds, the play sequence printed move by move, or another creature (any with a hunter plan in crates/hunt/src/plans/), at a temper (0 to 3). Every class is played as itself (crates/hunt/src/class.rs), and THE CLASS -- or, with --repeats, the last line -- counts what it did: shadows sent and lotuses, pillars and shots, spikes and blinks, the Dual mage's hands and goads, the Bulwark's guards. --gamble plays the creature's second plan, the one that takes a risk the first will not (the Mireback's: let the tongue land once the toad is low, to be swallowed and hit the stomach), where it has one. Against the Mantis, MANTIS_PLAN=repeater, jumper or dodger plays one of the duellist's three ablations, MANTIS_HABIT=off turns its habit memory off for the run, and MANTIS_DEBUG=1 prints the hunt frame by frame. Against the Siegeshell, --hunters 2 is the pair it is tuned for: one on the legs and one climbing for the crown, the call between them read off where the climber stands; SIEGE_DEBUG=1 prints the hunt frame by frame.",
            ),
            e(
                "cargo run -p sound --example sheet",
                "Every sound the game makes, on one sheet, without starting it: a WAV per cell in target/sound-sheet/ and one picture of spectrograms -- blows by weight, by edge, by what was struck and how big it was; footfalls by floor; telegraphs by startup; growls by the animal; the parry, stone, fire, blood and wind. Listen, look, change a number in crates/sound/src/patch.rs, run it again: the same loop the skies have.",
            ),
            e(
                "cargo run -p hunt --bin replay -- <file> --trace",
                "Judge a fight somebody played. The file is a replay the game saved (Y, the Esc menu, or --record). It is put back through the simulation, checked against the frame and checksum the game ended on, and judged by the creature's report -- the one fight prints for the scripted hunter -- with a table of what each fighter threw, landed, dodged and took beside it; a versus replay gets the rounds and the two tables. --trace adds the play sequence. Exits non-zero if this build does not reproduce the fight (a different build or tuning: the file names both).",
            ),
            e(
                "cargo run -p hunt --bin brood -- --class <name> --all --repeats <n> --seed <n> --hunters <1|2> --balanced --quiet",
                "The Broodmother's three plans side by side -- balanced, brood only, mother only -- with her own lines: time and damage on the brood, the sacs and her; sacs popped, burst and held; pops per slam window; guarded bites; rooted time; strand trips; clutches. The balanced plan has to beat both.",
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
        in_browser: false,
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
            e(
                "./crates/web/build-game.sh",
                "The whole game as a web page, ready to publish.",
            ),
            e(
                "./scripts/web-smoke.sh [out.png]",
                "Load the built web page in headless Chromium: fails on any console error, failed request or missing controls entry, and leaves a screenshot. Then two tabs open one sealed room and must meet as players one and two.",
            ),
            e(
                "./scripts/room-desktop.sh [prefix]",
                "A page opens a sealed room and the desktop build joins it with --join, through mosquitto on this machine, under Xvfb: they must meet as players one and two and play without a desync. Leaves a screenshot of each and the desktop's log.",
            ),
            e(
                "./scripts/setup-tools.sh web|shot|browser|all",
                "Install, once, the tools a fresh machine lacks: wasm-bindgen for the web build, Xvfb and a software GPU for screenshots, Playwright for the smoke test. The one place those steps are written; start it in the background.",
            ),
            e("./scripts/dev.sh", "The game in full development mode."),
            e("./scripts/help.sh", "This text."),
        ],
    },
    Section {
        title: "Environment variables",
        in_browser: false,
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
            e(
                "SHOT_BARS=<dark>,<light>",
                "Start a Dual mage with her two bars there, so a capture can look at the wings without playing up to them. Ignored for any other class.",
            ),
            e(
                "SHOT_WEIGHT=<n>",
                "Start every Bulwark's shield holding n weight, so a capture can look at a loaded shield without blocking up to it. It drains, so pair it with an early SHOT_FRAME.",
            ),
            e(
                "SHOT_MOVE=<move>",
                "Start a hunt with the creature winding up that move at player one (the Ridgeback's bite, stomp, sweep, charge, slam, kick, spray; the Mireback's spew, flop, tongue, belch, backwash, inflate, wallow, a lob aimed at player one; the Sandmaw's rise, breach, undertow, spit, lash, swallow, sound, from under the sand or standing as the move is thrown; the Pair's pounce, rake, swat, feint, ambush, trip, twin -- both cats, either side of you; the Broodmother's stab, lunge, slam, screech, web shot, web line; the Mantis's scythe, second slash, lunge, guard, leap, dive, flare, pivot, prayer, ready; the Veilstalker's lunge, spear, rake, pounce -- off a trunk top -- quill, smoke, mimic; the Galewing's stoop, talon, downwash, volley -- from its cruising height -- screech, buffet, hop -- on the plateau; the Siegeshell's footfall, stamp and drag on a foot in front of you, shed off its flank, plough down the valley behind you, shrug, shiver, and the beam at the siege line), so a capture can look at its floor marker. A pack puts one body of a kind that throws it there instead, and --hunt gnawers with SHOT_MOVE=pile calls a pile-on on a ring of them; --hunt hornback lays each of the herd's moves out (charge at a boulder, best with SHOT_YAW=1.57; bellow, hook, guard, trample, shoulder, kick, buck). Pair it with SHOT_FRAME and DEMO=0.",
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
            e(
                "ARENA_REPLAYS=<folder>",
                "Where Y, Save replay and --record put replays, instead of ~/.config/arena/replays/.",
            ),
        ],
    },
    Section {
        title: "Working on it",
        in_browser: false,
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

/// The controls panel for the browser build, as HTML.
///
/// The second rendering of the same tables, after the manual text, and it
/// exists for the same reason that one does: the page a link
/// leads to is the first thing a person who has never seen this game reads, and
/// a hand-written copy of the controls on that page would be wrong by the next
/// time a key moves. There is nowhere for it to disagree.
///
/// Only the sections marked `in_browser`. A page that offered somebody
/// `cargo run -p game` would be telling them to do the thing they followed a
/// link to avoid.
pub fn browser_help() -> String {
    let mut out = String::new();
    for section in SECTIONS.iter().filter(|s| s.in_browser) {
        out.push_str(&format!("<section>\n<h3>{}</h3>\n", escape(section.title)));
        if !section.blurb.is_empty() {
            out.push_str(&format!(
                "<p class=\"blurb\">{}</p>\n",
                escape(section.blurb)
            ));
        }
        out.push_str("<dl>\n");
        for entry in section.entries {
            out.push_str(&format!(
                "<dt>{}</dt><dd>{}</dd>\n",
                escape(entry.invocation),
                escape(entry.what)
            ));
        }
        out.push_str("</dl>\n</section>\n");
    }
    out
}

/// The three characters that would otherwise close a tag we did not open.
///
/// Several entries are written `--p1 <class>` and `SHOT_FRAME=<n>`, so this is
/// load bearing rather than defensive.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_with_a_placeholder_in_it_stays_text() {
        // `--p1 <class>` and `SHOT_FRAME=<n>` are how this manual writes a
        // placeholder. Dropped into a page unescaped, the browser reads
        // `<class>` as a tag it does not know and swallows the rest of the
        // line -- so the entry that says how to pick a class is the entry that
        // disappears.
        assert_eq!(escape("--p1 <class>"), "--p1 &lt;class&gt;");
        assert_eq!(escape("a & b"), "a &amp; b");
        assert_eq!(escape("plain text"), "plain text");
    }
}
