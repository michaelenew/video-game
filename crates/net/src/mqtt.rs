//! Just enough MQTT 3.1.1 to use a public broker as a [`Board`](crate::meet::Board).
//!
//! MQTT is a publish/subscribe protocol: a client connects to a broker,
//! subscribes to a topic, and receives everything anyone publishes to it.
//! Several brokers run free public instances that a browser can reach over a
//! WebSocket, which makes a topic per room a meeting point nobody has to host.
//!
//! Five packets out (CONNECT, SUBSCRIBE, PUBLISH, PINGREQ, DISCONNECT) and the
//! three that come back that matter (CONNACK, SUBACK, PUBLISH). Everything is
//! at quality-of-service 0 -- at most once, no acknowledgements -- because the
//! meeting protocol repeats itself anyway. Written out here rather than taken
//! from a crate because it is this small, and because this half has no
//! platform in it: the browser supplies the WebSocket, and the bytes are the
//! same on every machine, so they are tested on the desktop.
//!
//! The specification is OASIS "MQTT Version 3.1.1"; section numbers below are
//! from it.

/// What a broker sent that the board cares about.
#[derive(Debug, PartialEq, Eq)]
pub enum Incoming {
    /// CONNACK (3.2): 0 is accepted; anything else is a refusal.
    ConnAck(u8),
    /// SUBACK (3.9): the topic is live.
    SubAck,
    /// PUBLISH (3.3): a note.
    Publish { topic: String, payload: Vec<u8> },
    /// Anything else, ignored: PINGRESP, mostly.
    Other,
}

/// How long the broker waits without hearing from us before it drops the
/// connection (3.1.2.10). [`PING_EVERY_MS`] stays well inside it.
pub const KEEP_ALIVE_S: u16 = 60;
pub const PING_EVERY_MS: u64 = 25_000;

/// The "remaining length" of a fixed header (2.2.3): seven bits a byte, low
/// first, the top bit meaning "more follows".
fn put_length(out: &mut Vec<u8>, mut n: usize) {
    loop {
        let mut byte = (n % 128) as u8;
        n /= 128;
        if n > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if n == 0 {
            break;
        }
    }
}

/// A length-prefixed UTF-8 string (1.5.3).
fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn packet(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![kind];
    put_length(&mut out, body.len());
    out.extend_from_slice(body);
    out
}

/// CONNECT (3.1) with a clean session and no credentials.
pub fn connect(client_id: &str) -> Vec<u8> {
    let mut body = Vec::new();
    put_str(&mut body, "MQTT");
    body.push(4); // protocol level: 3.1.1
    body.push(0x02); // clean session
    body.extend_from_slice(&KEEP_ALIVE_S.to_be_bytes());
    put_str(&mut body, client_id);
    packet(0x10, &body)
}

/// What a CONNACK's refusal code means (3.2.2.3), for a person reading why a
/// broker turned us away.
pub fn refusal(code: u8) -> String {
    let why = match code {
        1 => "it does not speak MQTT 3.1.1",
        2 => "it rejected our client id",
        3 => "the service is unavailable",
        4 => "it wants a user name and password",
        5 => "we are not authorised",
        _ => "an unknown reason",
    };
    format!("code {code}, {why}")
}

/// SUBSCRIBE (3.8) to one topic at QoS 0.
pub fn subscribe(packet_id: u16, topic: &str) -> Vec<u8> {
    let mut body = packet_id.to_be_bytes().to_vec();
    put_str(&mut body, topic);
    body.push(0);
    packet(0x82, &body)
}

/// PUBLISH (3.3) at QoS 0, not retained: a note is for whoever is in the room
/// now, and a broker that kept it would hand a stale hello to the next visitor.
pub fn publish(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(2 + topic.len() + payload.len());
    put_str(&mut body, topic);
    body.extend_from_slice(payload);
    packet(0x30, &body)
}

/// PINGREQ (3.12).
pub fn ping() -> Vec<u8> {
    vec![0xc0, 0x00]
}

/// DISCONNECT (3.14).
pub fn disconnect() -> Vec<u8> {
    vec![0xe0, 0x00]
}

/// Bytes from the broker, in whatever pieces the WebSocket delivered them,
/// back into packets. MQTT over WebSockets allows a packet to span frames and
/// a frame to hold several packets (6.0), so this buffers.
#[derive(Default)]
pub struct Reader {
    buf: Vec<u8>,
}

impl Reader {
    pub fn feed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// The next whole packet, if one has arrived.
    pub fn packet(&mut self) -> Option<Incoming> {
        let kind = *self.buf.first()?;
        let mut len = 0usize;
        let mut at = 1;
        let mut shift = 0;
        loop {
            let byte = *self.buf.get(at)?;
            len |= ((byte & 0x7f) as usize) << shift;
            at += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 21 {
                // Four length bytes is the protocol's maximum. Anything longer
                // is not MQTT; drop what we have rather than wait forever.
                self.buf.clear();
                return None;
            }
        }
        if self.buf.len() < at + len {
            return None;
        }
        let body: Vec<u8> = self.buf[at..at + len].to_vec();
        self.buf.drain(..at + len);
        Some(match kind >> 4 {
            2 => Incoming::ConnAck(body.get(1).copied().unwrap_or(0xff)),
            9 => Incoming::SubAck,
            3 => {
                let qos = (kind >> 1) & 3;
                let Some(n) = body
                    .get(..2)
                    .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
                else {
                    return Some(Incoming::Other);
                };
                let Some(topic) = body.get(2..2 + n) else {
                    return Some(Incoming::Other);
                };
                // QoS 1 and 2 carry a packet id after the topic. We subscribe
                // at 0, so a broker should never send them, but skip it if one
                // does rather than read it as the start of the note.
                let from = 2 + n + if qos > 0 { 2 } else { 0 };
                Incoming::Publish {
                    topic: String::from_utf8_lossy(topic).into_owned(),
                    payload: body.get(from..).unwrap_or_default().to_vec(),
                }
            }
            _ => Incoming::Other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_connect_is_byte_for_byte_the_specification_example() {
        // 3.1.2.11's example variable header, with our flags and id.
        let bytes = connect("ab");
        assert_eq!(
            bytes,
            [
                0x10, 14, 0, 4, b'M', b'Q', b'T', b'T', 4, 0x02, 0, 60, 0, 2, b'a', b'b'
            ]
        );
    }

    #[test]
    fn remaining_lengths_cross_the_byte_boundaries() {
        // 2.2.3's table: one byte to 127, two to 16 383, three beyond.
        for (n, want) in [
            (0usize, vec![0x00]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (16_383, vec![0xff, 0x7f]),
            (16_384, vec![0x80, 0x80, 0x01]),
        ] {
            let mut out = Vec::new();
            put_length(&mut out, n);
            assert_eq!(out, want, "{n}");
        }
    }

    #[test]
    fn a_publish_reads_back_even_split_across_frames() {
        // An SDP is a few hundred bytes to a couple of kilobytes, which takes
        // two length bytes, and a WebSocket may hand it over in pieces.
        let note = "x".repeat(1_500);
        let mut stream = connack_ok();
        stream.extend(publish("arena/room", note.as_bytes()));
        stream.extend(ping_response());
        let mut reader = Reader::default();
        let mut got = Vec::new();
        for chunk in stream.chunks(7) {
            reader.feed(chunk);
            while let Some(p) = reader.packet() {
                got.push(p);
            }
        }
        assert_eq!(
            got,
            [
                Incoming::ConnAck(0),
                Incoming::Publish {
                    topic: "arena/room".into(),
                    payload: note.into_bytes()
                },
                Incoming::Other,
            ]
        );
    }

    #[test]
    fn a_subscribe_asks_for_one_topic_at_qos_zero() {
        assert_eq!(subscribe(1, "t"), [0x82, 6, 0, 1, 0, 1, b't', 0]);
    }

    fn connack_ok() -> Vec<u8> {
        vec![0x20, 0x02, 0x00, 0x00]
    }

    fn ping_response() -> Vec<u8> {
        vec![0xd0, 0x00]
    }
}
