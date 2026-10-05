//! Sealing a room: what a public broker sees is noise under a meaningless name.
//!
//! A room on a public broker is a topic anyone can subscribe to, and the two
//! notes traded in it hold each player's network addresses. So every note is
//! encrypted, and the topic is not the room's name. Both come from the link:
//!
//! ```text
//! https://…/?room=k3x9qw#key=7ycq…   (26 letters, about 130 bits, chosen at random)
//! ```
//!
//! The key is in the part after `#`, which a browser never sends to the server
//! the page came from -- so not even the host of the page learns it. From the
//! room name and the key, two things are derived, each with SHA-256 under its
//! own label so that knowing one tells you nothing about the other:
//!
//! - the **topic**: what the brokers see, a hex string;
//! - the **seal**: a 256-bit key for ChaCha20-Poly1305, which encrypts each
//!   note and also proves it was written by someone holding the link. A note
//!   that does not open is dropped, so a stranger cannot even forge a hello.
//!
//! SHA-256 of a long random secret is a fine way to make a key; the usual
//! caution about hashing passwords is about secrets short enough to guess, and
//! 130 random bits are not. A link with no key still works, sealed under the
//! room name alone -- which keeps out somebody watching every topic, but not
//! somebody who can guess six letters.
//!
//! [`Sealed`] is a [`Board`] wrapped around any other board, which is why
//! encryption touched nothing else: the room never sees a sealed note, and the
//! brokers never see an open one.

use crate::meet::{Board, Reach};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use sha2::{Digest, Sha256};

/// What every sealed note starts with, and the version of this format.
const SEALED: &str = "arena-sealed1 ";

/// A room's two derived secrets.
#[derive(Clone)]
pub struct RoomKey {
    topic: String,
    seal: [u8; 32],
}

/// Letters, digits, `-` and `_`, lower case, at most 64: what a link can be
/// trusted to carry unchanged, and the same name however it was typed.
pub fn tidy(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn derive(label: &str, room: &str, secret: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(label.as_bytes());
    h.update([0]);
    h.update(tidy(room).as_bytes());
    h.update([0]);
    h.update(tidy(secret).as_bytes());
    h.finalize().into()
}

impl RoomKey {
    /// `secret` is the link's `key`, or empty for a link without one.
    pub fn new(room: &str, secret: &str) -> RoomKey {
        let topic = derive("arena-rollback topic v1", room, secret);
        let hex: String = topic[..12].iter().map(|b| format!("{b:02x}")).collect();
        RoomKey {
            topic: format!("arena-rollback/r/{hex}"),
            seal: derive("arena-rollback seal v1", room, secret),
        }
    }

    /// The name the brokers know the room by. Also the name of a
    /// `BroadcastChannel`, so two tabs in different rooms never overhear.
    pub fn topic(&self) -> &str {
        &self.topic
    }

    fn cipher(&self) -> ChaCha20Poly1305 {
        ChaCha20Poly1305::new(Key::from_slice(&self.seal))
    }

    /// A note, encrypted. The topic is bound in as associated data, so a note
    /// lifted from one room does not open in another.
    pub fn close(&self, note: &str) -> String {
        let mut nonce = [0u8; 12];
        // A nonce must never repeat under one key, and twelve random bytes
        // will not in any number of notes a room sees. Should the platform have
        // no randomness at all, fail loudly rather than reuse zeros.
        getrandom::getrandom(&mut nonce).expect("no source of randomness for a nonce");
        let sealed = self
            .cipher()
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: note.as_bytes(),
                    aad: self.topic.as_bytes(),
                },
            )
            .expect("encrypting a short note cannot fail");
        let mut bytes = nonce.to_vec();
        bytes.extend_from_slice(&sealed);
        format!("{SEALED}{}", base64url(&bytes))
    }

    /// The note inside, if it was sealed with this room's key and not altered.
    pub fn open(&self, sealed: &str) -> Option<String> {
        let bytes = unbase64url(sealed.strip_prefix(SEALED)?)?;
        if bytes.len() < 12 + 16 {
            return None;
        }
        let (nonce, body) = bytes.split_at(12);
        let note = self
            .cipher()
            .decrypt(
                Nonce::from_slice(nonce),
                Payload {
                    msg: body,
                    aad: self.topic.as_bytes(),
                },
            )
            .ok()?;
        String::from_utf8(note).ok()
    }
}

/// Any board, with every note sealed on the way out and opened on the way in.
pub struct Sealed<B: Board> {
    board: B,
    key: RoomKey,
}

impl<B: Board> Sealed<B> {
    pub fn new(board: B, key: RoomKey) -> Self {
        Sealed { board, key }
    }
}

impl<B: Board> Board for Sealed<B> {
    fn post(&mut self, note: &str) {
        let sealed = self.key.close(note);
        self.board.post(&sealed);
    }

    /// Notes that do not open are somebody else's, or nobody's: skipped.
    fn take(&mut self) -> Option<String> {
        while let Some(sealed) = self.board.take() {
            if let Some(note) = self.key.open(&sealed) {
                return Some(note);
            }
        }
        None
    }

    fn reach(&mut self, now_ms: u64) -> Reach {
        self.board.reach(now_ms)
    }
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64 in the URL-safe alphabet, unpadded. Fifteen lines rather than a
/// dependency.
fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | (b as u32) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

fn unbase64url(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut n = 0u32;
    let mut bits = 0;
    for c in text.bytes() {
        let v = ALPHABET.iter().position(|&a| a == c)? as u32;
        n = n << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((n >> bits) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loopback::Hall;

    #[test]
    fn base64url_round_trips_every_length() {
        for len in 0..40 {
            let bytes: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
            assert_eq!(unbase64url(&base64url(&bytes)), Some(bytes), "{len}");
        }
        // The standard's own example, in the URL-safe alphabet.
        assert_eq!(base64url(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64url(b"fo"), "Zm8");
    }

    #[test]
    fn a_note_opens_with_its_room_key_and_no_other() {
        let key = RoomKey::new("k3x9qw", "7ycqaaaabbbbccccddddeeeeff");
        let sealed = key.close("arena1 hello 1 0 t");
        assert!(!sealed.contains("hello"), "{sealed}");
        assert_eq!(key.open(&sealed).as_deref(), Some("arena1 hello 1 0 t"));
        // Same room, another key; another room, same key.
        assert_eq!(RoomKey::new("k3x9qw", "x").open(&sealed), None);
        assert_eq!(
            RoomKey::new("other", "7ycqaaaabbbbccccddddeeeeff").open(&sealed),
            None
        );
        // One flipped character anywhere and it does not open.
        let mut bent = sealed.clone().into_bytes();
        let at = bent.len() - 3;
        bent[at] = if bent[at] == b'A' { b'B' } else { b'A' };
        assert_eq!(key.open(&String::from_utf8(bent).unwrap()), None);
    }

    #[test]
    fn the_same_note_seals_differently_every_time() {
        // A fresh nonce each time, so a repeated hello is not recognisable as
        // one by somebody watching the topic.
        let key = RoomKey::new("r", "s");
        assert_ne!(key.close("same"), key.close("same"));
    }

    #[test]
    fn the_topic_says_nothing_and_is_the_same_however_the_link_was_typed() {
        let a = RoomKey::new("K3X9QW", "Secret");
        let b = RoomKey::new("k3x9qw", "secret");
        assert_eq!(a.topic(), b.topic());
        assert!(!a.topic().contains("k3x9qw"));
        assert_ne!(a.topic(), RoomKey::new("k3x9qw", "other").topic());
    }

    #[test]
    fn a_sealed_board_hands_over_only_notes_sealed_with_its_key() {
        let hall = Hall::default();
        let key = RoomKey::new("r", "s");
        let mut ours = Sealed::new(hall.board(), key.clone());
        let mut theirs = Sealed::new(hall.board(), key);
        let mut stranger = Sealed::new(hall.board(), RoomKey::new("r", "guess"));
        let mut raw = hall.board();
        stranger.post("arena1 hello 9 0 t");
        raw.post("arena1 hello 8 0 t");
        ours.post("arena1 hello 1 0 t");
        assert_eq!(theirs.take().as_deref(), Some("arena1 hello 1 0 t"));
        assert_eq!(theirs.take(), None);
    }
}
