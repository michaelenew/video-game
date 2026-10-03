//! UDP to an address you were given: the desktop's `--port` and `--peer`.
//!
//! A [`Room`] like any other, over a board and a line that are the same UDP
//! port. Notes are datagrams that start with [`NOTE`]; everything else is a
//! GGRS packet. So two desktops trade the same hello as two browsers do --
//! which is what settles who is player one, and what refuses a match between
//! two different builds before it desyncs.
//!
//! The port stays shared after the room is done: the line GGRS runs over is
//! the same socket, still reading, and any hello the other side sends late is
//! dropped on the floor rather than handed to GGRS.

use crate::meet::{Board, Line, LineState, Reach, Room};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};

/// What every note datagram starts with. A GGRS packet never does: it starts
/// with a sixteen-bit magic number and a message tag, and this is eleven bytes
/// of ASCII.
const NOTE: &[u8] = b"arena-note\n";

/// Queued but untaken. Notes keep arriving for a moment after the room is
/// dropped, and nothing reads them then.
const KEEP: usize = 256;

struct Port {
    socket: UdpSocket,
    peer: Option<SocketAddr>,
    notes: VecDeque<String>,
    packets: VecDeque<Vec<u8>>,
    /// A GGRS packet has arrived: the far side has started its session.
    heard: bool,
    buf: Box<[u8; 4096]>,
}

impl Port {
    fn pump(&mut self) {
        loop {
            match self.socket.recv_from(&mut self.buf[..]) {
                Ok((n, _from)) => {
                    let bytes = &self.buf[..n];
                    if let Some(note) = bytes.strip_prefix(NOTE) {
                        if self.notes.len() >= KEEP {
                            self.notes.pop_front();
                        }
                        self.notes
                            .push_back(String::from_utf8_lossy(note).into_owned());
                    } else {
                        self.heard = true;
                        if self.packets.len() >= KEEP {
                            self.packets.pop_front();
                        }
                        self.packets.push_back(bytes.to_vec());
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                // Windows reports an earlier send's unreachable port here.
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => break,
            }
        }
    }

    fn send(&self, bytes: &[u8]) {
        if let Some(peer) = self.peer {
            // A lost datagram is a lost datagram; GGRS and the room both
            // repeat themselves.
            let _ = self.socket.send_to(bytes, peer);
        }
    }
}

#[derive(Clone)]
struct Shared(Arc<Mutex<Port>>);

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Port> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A bound UDP port, not yet pointed anywhere.
pub struct Direct(Shared);

impl Direct {
    /// Listen on `0.0.0.0:port`. Port 0 picks a free one.
    pub fn bind(port: u16) -> std::io::Result<Direct> {
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))?;
        socket.set_nonblocking(true)?;
        Ok(Direct(Shared(Arc::new(Mutex::new(Port {
            socket,
            peer: None,
            notes: VecDeque::new(),
            packets: VecDeque::new(),
            heard: false,
            buf: Box::new([0; 4096]),
        })))))
    }

    pub fn local_port(&self) -> u16 {
        self.0.lock().socket.local_addr().map_or(0, |a| a.port())
    }

    /// Meet whoever is at `peer`.
    pub fn toward(self, peer: SocketAddr, me: u64, terms: &str) -> Room<UdpBoard, UdpLine> {
        self.0.lock().peer = Some(peer);
        let port = self.0.clone();
        Room::new(
            UdpBoard(self.0),
            move || UdpLine {
                port: port.clone(),
                role: None,
                accepted: false,
            },
            me,
            terms,
        )
    }
}

/// Notes as datagrams to the one peer.
pub struct UdpBoard(Shared);

impl Board for UdpBoard {
    fn post(&mut self, note: &str) {
        let mut bytes = NOTE.to_vec();
        bytes.extend_from_slice(note.as_bytes());
        self.0.lock().send(&bytes);
    }

    fn take(&mut self) -> Option<String> {
        let mut port = self.0.lock();
        port.pump();
        port.notes.pop_front()
    }

    /// A socket that bound is as up as UDP gets. Whether anyone is listening
    /// is what the hello finds out.
    fn reach(&mut self, _now_ms: u64) -> Reach {
        Reach::Up
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Offering,
    Answering,
}

/// GGRS packets to the one peer. Nothing to negotiate: the address was on the
/// command line, so the offer and the answer are placeholders.
pub struct UdpLine {
    port: Shared,
    role: Option<Role>,
    accepted: bool,
}

impl Line for UdpLine {
    fn offer(&mut self) {
        self.role = Some(Role::Offering);
    }

    fn answer(&mut self, _offer: &str) {
        self.role = Some(Role::Answering);
    }

    fn accept(&mut self, _answer: &str) {
        self.accepted = true;
    }

    fn description(&mut self, _now_ms: u64) -> Option<String> {
        self.role.map(|_| "udp".to_string())
    }

    /// The offering side is open once it has the answer. The answering side
    /// waits for the first GGRS packet, so that it keeps answering repeated
    /// offers -- its answer can be lost -- until the other side has started.
    fn state(&mut self) -> LineState {
        match self.role {
            Some(Role::Offering) if self.accepted => LineState::Open,
            Some(Role::Answering) => {
                let mut port = self.port.lock();
                port.pump();
                if port.heard {
                    LineState::Open
                } else {
                    LineState::Negotiating
                }
            }
            _ => LineState::Negotiating,
        }
    }

    fn send(&mut self, packet: &[u8]) {
        self.port.lock().send(packet);
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        let mut port = self.port.lock();
        if port.packets.is_empty() {
            port.pump();
        }
        port.packets.pop_front()
    }
}
