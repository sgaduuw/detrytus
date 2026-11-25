//! `--net` and `--mac`: what the network device's link is
//! (docs/platform.md, The network device). `loop` hands every frame back;
//! `user` reaches the host's network through address translation in user
//! space (`nat.rs`);
//! `udp:LOCAL,PEER` carries each frame as one UDP datagram between a
//! local port and a peer, which may be another detrytus, a QEMU
//! `-netdev dgram` or `socket,udp=` backend, or a bridge to the host's
//! network.

use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use detrytus_core::net::{FRAME_MAX, Link, Loopback};

use crate::program::strerror;
use crate::{Fail, NO_INPUT, usage};

/// The link `--net` names, opened when the run starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetSpec {
    Loopback,
    User,
    Udp { local: String, peer: String },
}

impl NetSpec {
    pub fn parse(value: Option<String>) -> Result<NetSpec, Fail> {
        let bad = || usage("--net takes loop, user or udp:LOCAL,PEER");
        let value = value.ok_or_else(bad)?;
        if value == "loop" {
            return Ok(NetSpec::Loopback);
        }
        if value == "user" {
            return Ok(NetSpec::User);
        }
        let (local, peer) =
            value.strip_prefix("udp:").and_then(|v| v.split_once(',')).ok_or_else(bad)?;
        if local.is_empty() || peer.is_empty() {
            return Err(bad());
        }
        Ok(NetSpec::Udp { local: endpoint(local), peer: endpoint(peer) })
    }

    pub fn open(&self) -> Result<Box<dyn Link>, Fail> {
        match self {
            NetSpec::Loopback => Ok(Box::new(Loopback::default())),
            NetSpec::User => Ok(Box::new(crate::nat::UserLink::new())),
            NetSpec::Udp { local, peer } => Ok(Box::new(UdpLink::open(local, peer)?)),
        }
    }
}

/// A bare port means one on this host.
fn endpoint(text: &str) -> String {
    if text.bytes().all(|b| b.is_ascii_digit()) { format!("127.0.0.1:{text}") } else { text.into() }
}

/// `--mac` as six octets in hexadecimal, separated by colons.
pub fn parse_mac(value: Option<String>) -> Result<u64, Fail> {
    let bad = || usage("--mac takes six hexadecimal octets, as 02:00:00:00:00:01");
    let value = value.ok_or_else(bad)?;
    let octets: Vec<&str> = value.split(':').collect();
    if octets.len() != 6 || octets.iter().any(|o| o.len() != 2) {
        return Err(bad());
    }
    octets.iter().try_fold(0u64, |mac, o| {
        u8::from_str_radix(o, 16).map(|b| mac << 8 | b as u64).map_err(|_| bad())
    })
}

/// Frames as UDP datagrams to and from one peer. The wire loses frames
/// rather than stopping the machine: a datagram that cannot be sent is
/// dropped, as on a cable with nothing at the other end.
pub struct UdpLink {
    socket: UdpSocket,
}

impl UdpLink {
    fn open(local: &str, peer: &str) -> Result<UdpLink, Fail> {
        let fail = |what: &str, e: std::io::Error| {
            Fail::new(format!("--net: {what}: {}", strerror(&e)), NO_INPUT)
        };
        let resolve = |what: &str| -> Result<SocketAddr, Fail> {
            what.to_socket_addrs()
                .map_err(|e| fail(what, e))?
                .next()
                .ok_or_else(|| Fail::new(format!("--net: {what}: no such address"), NO_INPUT))
        };
        let (local_addr, peer_addr) = (resolve(local)?, resolve(peer)?);
        let socket = UdpSocket::bind(local_addr).map_err(|e| fail(local, e))?;
        socket.connect(peer_addr).map_err(|e| fail(peer, e))?;
        socket.set_nonblocking(true).map_err(|e| fail(local, e))?;
        Ok(UdpLink { socket })
    }
}

impl Link for UdpLink {
    fn send(&mut self, frame: &[u8]) {
        let _ = self.socket.send(frame);
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        // One larger than a frame, so that a datagram too long to be one
        // arrives too long, and the device drops it, rather than cut down.
        let mut buf = [0u8; FRAME_MAX + 1];
        // A refused earlier send can surface here once; try past it.
        for _ in 0..3 {
            match self.socket.recv(&mut buf) {
                Ok(n) => return Some(buf[..n].to_vec()),
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(_) => continue,
            }
        }
        None
    }

    fn may_receive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_specs_parse() {
        assert_eq!(NetSpec::parse(Some("loop".into())).ok().unwrap(), NetSpec::Loopback);
        assert_eq!(NetSpec::parse(Some("user".into())).ok().unwrap(), NetSpec::User);
        assert_eq!(
            NetSpec::parse(Some("udp:5555,10.0.0.2:5556".into())).ok().unwrap(),
            NetSpec::Udp { local: "127.0.0.1:5555".into(), peer: "10.0.0.2:5556".into() }
        );
        for bad in ["", "tap0", "udp:5555", "udp:,5556", "udp:5555,"] {
            assert!(NetSpec::parse(Some(bad.into())).is_err(), "{bad}");
        }
    }

    #[test]
    fn macs_parse() {
        assert_eq!(parse_mac(Some("02:00:00:00:00:2a".into())).ok().unwrap(), 0x02_00_00_00_00_2a);
        for bad in ["02:00:00:00:00", "02:00:00:00:00:0g", "2:00:00:00:00:01:"] {
            assert!(parse_mac(Some(bad.into())).is_err(), "{bad}");
        }
    }

    #[test]
    fn two_udp_links_carry_frames_both_ways() {
        let a_port = UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let b_port = UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let (a, b) = (format!("127.0.0.1:{a_port}"), format!("127.0.0.1:{b_port}"));
        let mut left = UdpLink::open(&a, &b).ok().unwrap();
        let mut right = UdpLink::open(&b, &a).ok().unwrap();
        left.send(&[1, 2, 3]);
        right.send(&[4, 5]);
        let wait = |link: &mut UdpLink| {
            for _ in 0..200 {
                if let Some(f) = link.receive() {
                    return f;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            panic!("no frame");
        };
        assert_eq!(wait(&mut right), vec![1, 2, 3]);
        assert_eq!(wait(&mut left), vec![4, 5]);
        assert_eq!(left.receive(), None);
    }
}
