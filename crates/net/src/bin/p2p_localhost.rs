//! Two real peer-to-peer sessions on localhost.
//!
//! Spawns two GGRS P2P sessions on separate UDP ports, runs a match between
//! them, and checks they agree. This is the closest thing to a real network
//! test that fits on one machine: real sockets, real packets, real input
//! prediction and rollback -- only the latency is missing.
//!
//!     cargo run -p net --bin p2p_localhost

use net::{NetInput, handle_requests, p2p};
use sim::World;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const FRAMES: u32 = 900; // fifteen seconds of match
const PORT_A: u16 = 47801;
const PORT_B: u16 = 47802;

fn main() {
    let a: SocketAddr = format!("127.0.0.1:{PORT_A}").parse().unwrap();
    let b: SocketAddr = format!("127.0.0.1:{PORT_B}").parse().unwrap();

    // Both peers derive the same answer from the same two addresses.
    let handle_a = p2p::local_handle_for(a, b);
    let handle_b = p2p::local_handle_for(b, a);
    assert_ne!(
        handle_a, handle_b,
        "both peers claimed the same player slot"
    );
    println!("peer A is player {handle_a}, peer B is player {handle_b}");

    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    let ta = std::thread::spawn(move || run_peer("A", PORT_A, b, handle_a, tx));
    let tb = std::thread::spawn(move || run_peer("B", PORT_B, a, handle_b, tx2));

    let ra = ta.join().expect("peer A panicked");
    let rb = tb.join().expect("peer B panicked");
    drop(rx);

    println!(
        "\npeer A: frame {} checksum {:#018x}",
        ra.frame, ra.checksum
    );
    println!("peer B: frame {} checksum {:#018x}", rb.frame, rb.checksum);

    assert!(
        ra.advanced > 0 && rb.advanced > 0,
        "no frames were simulated"
    );
    assert_eq!(ra.frame, rb.frame, "peers ended on different frames");
    assert_eq!(
        ra.checksum, rb.checksum,
        "DESYNC: peers disagree about the world"
    );
    println!(
        "\nok: two peers stayed in sync over real UDP for {} frames",
        ra.frame
    );
}

struct Outcome {
    frame: u32,
    checksum: u64,
    advanced: u32,
}

fn run_peer(
    name: &'static str,
    port: u16,
    remote: SocketAddr,
    local_handle: usize,
    _log: mpsc::Sender<String>,
) -> Outcome {
    let mut session = p2p::start(port, remote, local_handle).expect("session");
    let mut world = World::new();
    let mut advanced = 0u32;

    // Wait for the handshake before feeding it input.
    let deadline = Instant::now() + Duration::from_secs(10);
    while session.current_state() != ggrs::SessionState::Running {
        session.poll_remote_clients();
        assert!(
            Instant::now() < deadline,
            "{name}: peers never synchronised"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    println!("{name}: synchronised");

    let mut rng = 0x243f_6a88_85a3_08d3_u64 ^ local_handle as u64;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };

    // Run at real time. GGRS paces itself against the peer, so racing ahead
    // just fills the input queue.
    let mut tick = Instant::now();
    while advanced < FRAMES {
        session.poll_remote_clients();
        if session.current_state() != ggrs::SessionState::Running {
            std::thread::sleep(Duration::from_millis(2));
            continue;
        }

        let input = NetInput((next() & 0x1ff) as u16);
        match session.add_local_input(local_handle, input) {
            Ok(()) => {}
            Err(ggrs::GgrsError::PredictionThreshold) => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => panic!("{name}: {e}"),
        }

        match session.advance_frame() {
            Ok(requests) => {
                handle_requests(&mut world, requests);
                advanced += 1;
            }
            Err(ggrs::GgrsError::PredictionThreshold) => {}
            Err(e) => panic!("{name}: advance failed: {e}"),
        }

        for event in session.events() {
            if let ggrs::GgrsEvent::DesyncDetected { frame, .. } = event {
                panic!("{name}: GGRS reported a desync at frame {frame}");
            }
        }

        tick += Duration::from_micros(16_667);
        if let Some(rest) = tick.checked_duration_since(Instant::now()) {
            std::thread::sleep(rest);
        }
    }

    Outcome {
        frame: world.frame,
        checksum: world.checksum(),
        advanced,
    }
}
