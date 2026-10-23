//! **Aggregates on the wire**: how a cell leader's summary of its region
//! reaches the leaders next door, over the lines the match already has
//! (`docs/design/exploration/0018_crowds.md` §5).
//!
//! GGRS owns the socket once a session starts, and the socket is the
//! [`crate::Mesh`]: one line to each other player. A crowd packet rides the
//! same lines with a tag in front -- the bytes GGRS's own socket would read as
//! "not a GGRS message" and drop. The mesh puts them in a **post box**
//! instead, shared with the game by an `Arc`, and takes outgoing ones from
//! it whenever GGRS polls. So a leader posts its aggregate and a receiver
//! collects what arrived, with nothing of the session changed.
//!
//! A packet is 24 bytes: the tag, and the aggregate's fields in little
//! endian. Two leaders agreeing on an aggregate is checked where it is
//! collected ([`PostBox::collect`]), the NEO rule applied to a stream: one
//! leader's word stands unchallenged for an epoch, two agreeing stand at
//! once, two disagreeing are reported and neither is believed.

use crate::Peer;
use sim::crowd::Aggregate;
use sim::region::RegionId;
use std::sync::{Arc, Mutex};

/// The first bytes of a crowd packet: what tells it from a GGRS message.
pub const TAG: [u8; 2] = [0xC2, 0x0D];

/// A packet's size.
pub const SIZE: usize = 24;

/// An aggregate as it goes on a line.
pub fn encode(a: &Aggregate) -> [u8; SIZE] {
    let mut b = [0u8; SIZE];
    b[..2].copy_from_slice(&TAG);
    b[2..4].copy_from_slice(&a.region.q.to_le_bytes());
    b[4..6].copy_from_slice(&a.region.r.to_le_bytes());
    b[6..8].copy_from_slice(&a.epoch.to_le_bytes());
    b[8..10].copy_from_slice(&a.fighters.to_le_bytes());
    b[10..12].copy_from_slice(&a.middle[0].to_le_bytes());
    b[12..14].copy_from_slice(&a.middle[1].to_le_bytes());
    b[14] = a.drift[0] as u8;
    b[15] = a.drift[1] as u8;
    b[16..18].copy_from_slice(&a.hurt.to_le_bytes());
    b[18] = a.creatures;
    b[19..23].copy_from_slice(&a.doors.to_le_bytes()[..4]);
    b
}

/// A packet read back, if it is one.
pub fn decode(b: &[u8]) -> Option<Aggregate> {
    if b.len() < SIZE || b[..2] != TAG {
        return None;
    }
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let i16_at = |i: usize| i16::from_le_bytes([b[i], b[i + 1]]);
    Some(Aggregate {
        region: RegionId::new(i16_at(2), i16_at(4)),
        epoch: u16_at(6),
        fighters: u16_at(8),
        middle: [i16_at(10), i16_at(12)],
        drift: [b[14] as i8, b[15] as i8],
        hurt: u16_at(16),
        creatures: b[18],
        doors: u32::from_le_bytes([b[19], b[20], b[21], b[22]]),
    })
}

/// What has been heard of one aggregate: who said it, and what.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Word {
    from: Peer,
    said: Aggregate,
    /// The collector's clock when it was heard.
    at: u32,
}

/// **The post box**: outgoing packets to send, incoming words to collect.
/// Shared between the mesh (which carries) and the game (which writes and
/// reads), by `Arc`.
#[derive(Default)]
pub struct PostBox {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    out: Vec<(Peer, [u8; SIZE])>,
    heard: Vec<Word>,
    /// Lines whose far end has gone, not yet taken by the game.
    closed: Vec<Peer>,
    /// Aggregates that two leaders disagreed on, for the HUD and the tests.
    pub disputes: u32,
}

/// A shared post box.
pub type Post = Arc<PostBox>;

impl PostBox {
    pub fn new() -> Post {
        Arc::new(PostBox::default())
    }

    /// **The line to `peer` has closed**: whoever was at the far end has
    /// left -- closed the page, or quit. The mesh says so once; the game
    /// takes it ([`PostBox::take_closed`]) and gives the seat up at once,
    /// rather than waiting out the session's long silence for it.
    pub fn line_closed(&self, peer: Peer) {
        self.inner.lock().unwrap().closed.push(peer);
    }

    /// Every line closed since the last time this was asked, taken.
    pub fn take_closed(&self) -> Vec<Peer> {
        std::mem::take(&mut self.inner.lock().unwrap().closed)
    }

    /// A leader posts its aggregate to a peer.
    pub fn post(&self, to: Peer, a: &Aggregate) {
        self.inner.lock().unwrap().out.push((to, encode(a)));
    }

    /// What the mesh should send: every posted packet, taken.
    pub fn take_outgoing(&self) -> Vec<(Peer, [u8; SIZE])> {
        std::mem::take(&mut self.inner.lock().unwrap().out)
    }

    /// The mesh heard a packet from a peer, at the collector's clock.
    pub fn heard(&self, from: Peer, bytes: &[u8], now: u32) {
        if let Some(said) = decode(bytes) {
            let mut inner = self.inner.lock().unwrap();
            // One word per (leader, region, epoch): a repeat is nothing.
            if !inner.heard.iter().any(|w| {
                w.from == from && w.said.region == said.region && w.said.epoch == said.epoch
            }) {
                inner.heard.push(Word {
                    from,
                    said,
                    at: now,
                });
            }
        }
    }

    /// **Collect what stands**: every aggregate two leaders agree on, or one
    /// leader has said and nobody has gainsaid for `unchallenged` of the
    /// clock. Disagreements are counted and neither word is believed. What
    /// is collected is forgotten here; what is not is kept for next time.
    pub fn collect(&self, now: u32, unchallenged: u32) -> Vec<Aggregate> {
        let mut inner = self.inner.lock().unwrap();
        let mut out = Vec::new();
        let mut keep = Vec::new();
        let heard = std::mem::take(&mut inner.heard);
        let mut done: Vec<(RegionId, u16)> = Vec::new();
        for (k, w) in heard.iter().enumerate() {
            let key = (w.said.region, w.said.epoch);
            if done.contains(&key) {
                continue;
            }
            let same: Vec<&Word> = heard
                .iter()
                .filter(|o| (o.said.region, o.said.epoch) == key)
                .collect();
            let agree = same.iter().filter(|o| o.said == w.said).count();
            let disagree = same.len() - agree;
            if disagree > 0 && agree < 2 {
                // Two leaders at odds, and no majority: nobody is believed,
                // and the stream's gate will hold until one is.
                inner.disputes += 1;
                done.push(key);
                continue;
            }
            if agree >= 2 || now.wrapping_sub(w.at) >= unchallenged {
                out.push(w.said);
                done.push(key);
            } else {
                keep.push(heard[k]);
            }
        }
        inner.heard = keep;
        out.sort_by_key(|a| (a.epoch, a.region.q, a.region.r));
        out
    }

    pub fn disputes(&self) -> u32 {
        self.inner.lock().unwrap().disputes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_packet_reads_back_as_it_was_written() {
        let a = Aggregate {
            region: RegionId::new(-3, 7),
            epoch: 41,
            fighters: 12,
            middle: [-250, 310],
            drift: [-7, 3],
            hurt: 540,
            creatures: 2,
            doors: 0b1011,
        };
        assert_eq!(decode(&encode(&a)), Some(a));
        assert_eq!(decode(&[1, 2, 3]), None);
    }

    #[test]
    fn two_leaders_agreeing_stand_at_once_one_alone_stands_after_a_while_and_a_dispute_stands_never()
     {
        let post = PostBox::new();
        let a = Aggregate {
            region: RegionId::new(1, 1),
            epoch: 3,
            fighters: 4,
            ..Aggregate::NONE
        };
        post.heard(Peer::Line(0), &encode(&a), 100);
        assert!(post.collect(100, 60).is_empty(), "one word, not yet");
        post.heard(Peer::Line(1), &encode(&a), 101);
        assert_eq!(post.collect(101, 60), vec![a], "two agreeing");
        assert!(post.collect(200, 60).is_empty(), "and forgotten");

        let b = Aggregate { epoch: 4, ..a };
        post.heard(Peer::Line(0), &encode(&b), 160);
        assert!(post.collect(170, 60).is_empty());
        assert_eq!(post.collect(220, 60), vec![b], "unchallenged for an epoch");

        let c = Aggregate { epoch: 5, ..a };
        let lie = Aggregate { fighters: 40, ..c };
        post.heard(Peer::Line(0), &encode(&c), 300);
        post.heard(Peer::Line(1), &encode(&lie), 300);
        assert!(post.collect(400, 60).is_empty(), "at odds, nobody believed");
        assert_eq!(post.disputes(), 1);
    }
}
