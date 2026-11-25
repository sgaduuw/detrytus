//! `--net user`: the machine reaches the host's network through network
//! address translation done here, in user space, as QEMU's user
//! networking (slirp) does, so nothing needs root or a TAP device.
//!
//! The link plays a small network: the machine (10.0.0.2 by default,
//! whatever address it uses) and a gateway, 10.0.0.1, which this link
//! answers for. The gateway answers ARP and pings itself, and forwards
//! the rest to the host's own sockets:
//!
//! - **ICMP echo** goes out on a ping socket (an unprivileged ICMP
//!   datagram socket where the host allows one, else a raw socket, which
//!   needs root); with neither, pings past the gateway go unanswered.
//! - **UDP**: each port of the machine's gets a host socket; datagrams
//!   go out from it and what comes back to it comes back, from where it
//!   came.
//! - **TCP**: a connection from the machine is a host `TcpStream`. The
//!   link ends the machine's TCP itself, playing the far host: it
//!   answers the SYN once the host's connect has succeeded (a reset if
//!   it failed), acknowledges and passes on what the machine sends, and
//!   sends what the host reads, retransmitting until acknowledged.
//!
//! The gateway's own address stands for the host: what the machine sends
//! to 10.0.0.1 goes to 127.0.0.1, so `nc 10.0.0.1 8080` reaches a server
//! on the host. Nothing reaches the machine unasked: there is no port
//! forwarding into it.
//!
//! Real sockets keep real time, so the machine should run `--live`.

use std::collections::{HashMap, VecDeque};
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream, UdpSocket};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::{Duration, Instant};

use detrytus_core::net::Link;

/// The network the link plays, and the gateway's place on it.
pub const GATEWAY: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
const GATEWAY_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0xfe];

const ETH_ARP: u16 = 0x0806;
const ETH_IP: u16 = 0x0800;
const ICMP: u8 = 1;
const TCP: u8 = 6;
const UDP: u8 = 17;
const FIN: u8 = 1;
const SYN: u8 = 2;
const RST: u8 = 4;
const PSH: u8 = 8;
const ACK: u8 = 16;

/// The largest segment the link sends, and asks for.
const MSS: usize = 1460;
/// What a TCP connection holds each way before it stops taking more.
const BUFFER: usize = 65536;
/// How long the link waits for an acknowledgment before sending again,
/// how many times, and how long a host connect may take.
const RTO: Duration = Duration::from_millis(1000);
const TRIES: u32 = 8;
const CONNECT: Duration = Duration::from_secs(10);
/// A UDP port of the machine's keeps its host socket this long unused.
const UDP_IDLE: Duration = Duration::from_secs(120);

pub struct UserLink {
    /// Frames for the machine, oldest first.
    out: VecDeque<Vec<u8>>,
    /// The machine's addresses, as its last packet gave them.
    guest_mac: [u8; 6],
    guest_ip: Ipv4Addr,
    ping: Option<PingSocket>,
    /// Pings under way: (address, sequence number) to the machine's
    /// identifier, which a ping socket may have replaced.
    pings: HashMap<(Ipv4Addr, u16), (u16, Instant)>,
    udp: HashMap<u16, UdpFlow>,
    tcp: HashMap<FlowKey, TcpFlow>,
    ident: u16,
}

struct PingSocket {
    socket: UdpSocket,
    /// A raw socket gives each packet with its IP header.
    raw: bool,
}

struct UdpFlow {
    socket: UdpSocket,
    used: Instant,
}

/// A connection: the machine's port, and the address and port it went to.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct FlowKey {
    port: u16,
    to: Ipv4Addr,
    to_port: u16,
}

enum Host {
    Connecting(Receiver<std::io::Result<TcpStream>>),
    Open(TcpStream),
}

struct TcpFlow {
    host: Host,
    /// The next octet expected from the machine.
    rcv_nxt: u32,
    /// Our first sequence number, the oldest not acknowledged and the next.
    iss: u32,
    snd_una: u32,
    snd_nxt: u32,
    /// What the host sent, from snd_una on (after the SYN).
    to_guest: VecDeque<u8>,
    /// What the machine sent, not yet written to the host.
    to_host: Vec<u8>,
    window: usize,
    mss: usize,
    syn_acked: bool,
    guest_fin: bool,
    host_shut: bool,
    host_eof: bool,
    fin_sent: bool,
    fin_acked: bool,
    sent_at: Instant,
    tries: u32,
}

impl Default for UserLink {
    fn default() -> UserLink {
        UserLink::new()
    }
}

impl UserLink {
    pub fn new() -> UserLink {
        UserLink {
            out: VecDeque::new(),
            guest_mac: [0xff; 6],
            guest_ip: Ipv4Addr::new(10, 0, 0, 2),
            ping: ping_socket(),
            pings: HashMap::new(),
            udp: HashMap::new(),
            tcp: HashMap::new(),
            ident: 0,
        }
    }

    // --- from the machine ---------------------------------------------------

    fn arp(&mut self, f: &[u8]) {
        if f.len() < 42 || be16(&f[14..]) != 1 || be16(&f[16..]) != ETH_IP || be16(&f[20..]) != 1 {
            return;
        }
        let (sha, spa, tpa) = (&f[22..28], ip_at(&f[28..]), ip_at(&f[38..]));
        if tpa != GATEWAY || spa == GATEWAY {
            return;
        }
        let mut r = Vec::with_capacity(42);
        r.extend_from_slice(sha);
        r.extend_from_slice(&GATEWAY_MAC);
        r.extend_from_slice(&ETH_ARP.to_be_bytes());
        r.extend_from_slice(&[0, 1, 8, 0, 6, 4, 0, 2]);
        r.extend_from_slice(&GATEWAY_MAC);
        r.extend_from_slice(&GATEWAY.octets());
        r.extend_from_slice(sha);
        r.extend_from_slice(&spa.octets());
        self.out.push_back(r);
    }

    fn ipv4(&mut self, f: &[u8]) {
        let p = &f[14..];
        if p.len() < 20 || p[0] >> 4 != 4 {
            return;
        }
        let hl = (p[0] & 15) as usize * 4;
        let total = be16(&p[2..]) as usize;
        if hl < 20 || total < hl || total > p.len() || checksum(&p[..hl], 0) != 0 {
            return;
        }
        if be16(&p[6..]) & 0x3fff != 0 {
            return; // a fragment: the machine sends none
        }
        let (src, dst) = (ip_at(&p[12..]), ip_at(&p[16..]));
        self.guest_mac.copy_from_slice(&f[6..12]);
        self.guest_ip = src;
        let body = &p[hl..total];
        match p[9] {
            ICMP => self.icmp(dst, body),
            UDP => self.udp_out(src, dst, body),
            TCP => self.tcp_in(src, dst, body),
            _ => {}
        }
    }

    fn icmp(&mut self, dst: Ipv4Addr, m: &[u8]) {
        if m.len() < 8 || m[0] != 8 || checksum(m, 0) != 0 {
            return;
        }
        if dst == GATEWAY {
            let mut reply = m.to_vec();
            reply[0] = 0;
            set_checksum(&mut reply, 2, 0);
            self.send_ip(GATEWAY, ICMP, &reply);
            return;
        }
        let Some(ping) = &self.ping else { return };
        let (id, seq) = (be16(&m[4..]), be16(&m[6..]));
        let now = Instant::now();
        self.pings.retain(|_, (_, at)| now.duration_since(*at) < Duration::from_secs(30));
        self.pings.insert((dst, seq), (id, now));
        let _ = ping.socket.send_to(m, SocketAddrV4::new(host_side(dst), 0));
    }

    fn udp_out(&mut self, src: Ipv4Addr, dst: Ipv4Addr, d: &[u8]) {
        if d.len() < 8 || (be16(&d[4..]) as usize) < 8 || be16(&d[4..]) as usize > d.len() {
            return;
        }
        let len = be16(&d[4..]) as usize;
        if be16(&d[6..]) != 0 && checksum(&d[..len], pseudo(src, dst, UDP, len)) != 0 {
            return;
        }
        let (sport, dport) = (be16(d), be16(&d[2..]));
        if dst.is_broadcast() || dst.octets()[3] == 255 {
            return;
        }
        let now = Instant::now();
        let flow = match self.udp.entry(sport) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else { return };
                if socket.set_nonblocking(true).is_err() {
                    return;
                }
                e.insert(UdpFlow { socket, used: now })
            }
        };
        flow.used = now;
        let _ = flow.socket.send_to(&d[8..len], SocketAddrV4::new(host_side(dst), dport));
    }

    fn tcp_in(&mut self, src: Ipv4Addr, dst: Ipv4Addr, s: &[u8]) {
        if s.len() < 20 || checksum(s, pseudo(src, dst, TCP, s.len())) != 0 {
            return;
        }
        let off = (s[12] >> 4) as usize * 4;
        if off < 20 || off > s.len() {
            return;
        }
        let key = FlowKey { port: be16(s), to: dst, to_port: be16(&s[2..]) };
        let (seq, ack, flags) = (be32(&s[4..]), be32(&s[8..]), s[13]);
        let window = be16(&s[14..]) as usize;
        let data = &s[off..];
        let Some(flow) = self.tcp.get_mut(&key) else {
            if flags & RST != 0 {
                return;
            }
            if flags & SYN != 0 && flags & ACK == 0 {
                self.tcp_open(key, seq, window, mss_option(&s[20..off]));
            } else {
                self.reset_reply(key, seq, ack, flags, data.len());
            }
            return;
        };
        if flags & RST != 0 {
            if seq == flow.rcv_nxt {
                self.tcp.remove(&key); // the host stream closes with it
            }
            return;
        }
        if flags & SYN != 0 {
            // The machine's SYN again: our SYN+ACK went missing, or it
            // has not come yet.
            if !flow.syn_acked && matches!(flow.host, Host::Open(_)) {
                flow.snd_nxt = flow.iss;
            }
            return;
        }
        if flags & ACK == 0 {
            return;
        }
        let acked = ack.wrapping_sub(flow.snd_una);
        if acked <= flow.snd_nxt.wrapping_sub(flow.snd_una) {
            flow.window = window;
            let mut n = acked as usize;
            if n > 0 && !flow.syn_acked {
                flow.syn_acked = true;
                n -= 1;
            }
            let data_out = flow.to_guest.len();
            if n > data_out {
                flow.fin_acked = true;
                n = data_out;
            }
            flow.to_guest.drain(..n);
            if acked > 0 {
                flow.snd_una = ack;
                flow.tries = 0;
                flow.sent_at = Instant::now();
            }
        }
        if data.is_empty() && flags & FIN == 0 {
            return;
        }
        let fin = flags & FIN != 0;
        if seq == flow.rcv_nxt && !flow.guest_fin && flow.to_host.len() + data.len() <= BUFFER {
            flow.to_host.extend_from_slice(data);
            flow.rcv_nxt = flow.rcv_nxt.wrapping_add(data.len() as u32);
            if fin {
                flow.guest_fin = true;
                flow.rcv_nxt = flow.rcv_nxt.wrapping_add(1);
            }
        }
        let (snd_nxt, rcv_nxt, room) = (flow.snd_nxt, flow.rcv_nxt, flow.room());
        self.send_tcp(key, ACK, snd_nxt, rcv_nxt, room, &[], false);
    }

    fn tcp_open(&mut self, key: FlowKey, seq: u32, window: usize, mss: usize) {
        if self.tcp.len() >= 64 {
            return;
        }
        let to = SocketAddr::V4(SocketAddrV4::new(host_side(key.to), key.to_port));
        let (tell, told) = channel();
        std::thread::spawn(move || {
            let _ = tell.send(TcpStream::connect_timeout(&to, CONNECT));
        });
        let clock = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let iss = clock.map_or(0, |d| d.as_nanos() as u32) ^ (key.port as u32) << 16;
        self.tcp.insert(
            key,
            TcpFlow {
                host: Host::Connecting(told),
                rcv_nxt: seq.wrapping_add(1),
                iss,
                snd_una: iss,
                snd_nxt: iss,
                to_guest: VecDeque::new(),
                to_host: Vec::new(),
                window,
                mss,
                syn_acked: false,
                guest_fin: false,
                host_shut: false,
                host_eof: false,
                fin_sent: false,
                fin_acked: false,
                sent_at: Instant::now(),
                tries: 0,
            },
        );
    }

    /// Answer a segment that has no connection with a reset.
    fn reset_reply(&mut self, key: FlowKey, seq: u32, ack: u32, flags: u8, len: usize) {
        if flags & ACK != 0 {
            self.send_tcp(key, RST, ack, 0, 0, &[], false);
        } else {
            let end = seq.wrapping_add(len as u32 + (flags & (SYN | FIN) != 0) as u32);
            self.send_tcp(key, RST | ACK, 0, end, 0, &[], false);
        }
    }

    // --- from the host ------------------------------------------------------

    fn poll(&mut self) {
        self.poll_ping();
        self.poll_udp();
        self.poll_tcp();
    }

    fn poll_ping(&mut self) {
        let mut buf = [0u8; 1600];
        loop {
            let Some(ping) = &self.ping else { return };
            let (n, from) = match ping.socket.recv_from(&mut buf) {
                Ok(got) => got,
                Err(_) => return,
            };
            let SocketAddr::V4(from) = from else { continue };
            let mut m = &buf[..n];
            if ping.raw || (m.len() >= 20 && m[0] >> 4 == 4) {
                let hl = (m[0] & 15) as usize * 4;
                if m.len() < hl {
                    continue;
                }
                m = &m[hl..];
            }
            if m.len() < 8 || m[0] != 0 {
                continue;
            }
            let addr = guest_side(*from.ip());
            let Some((id, _)) = self.pings.remove(&(addr, be16(&m[6..]))) else { continue };
            let mut reply = m.to_vec();
            reply[4..6].copy_from_slice(&id.to_be_bytes());
            set_checksum(&mut reply, 2, 0);
            self.send_ip(addr, ICMP, &reply);
        }
    }

    fn poll_udp(&mut self) {
        let mut buf = [0u8; 1472];
        let now = Instant::now();
        self.udp.retain(|_, f| now.duration_since(f.used) < UDP_IDLE);
        let mut got = Vec::new();
        for (&port, flow) in &self.udp {
            while let Ok((n, SocketAddr::V4(from))) = flow.socket.recv_from(&mut buf) {
                got.push((port, from, buf[..n].to_vec()));
            }
        }
        for (port, from, data) in got {
            let src = guest_side(*from.ip());
            let mut d = Vec::with_capacity(8 + data.len());
            d.extend_from_slice(&from.port().to_be_bytes());
            d.extend_from_slice(&port.to_be_bytes());
            d.extend_from_slice(&((8 + data.len()) as u16).to_be_bytes());
            d.extend_from_slice(&[0, 0]);
            d.extend_from_slice(&data);
            let sum = pseudo(src, self.guest_ip, UDP, d.len());
            set_checksum(&mut d, 6, sum);
            if be16(&d[6..]) == 0 {
                d[6..8].copy_from_slice(&[0xff, 0xff]);
            }
            self.send_ip(src, UDP, &d);
        }
    }

    fn poll_tcp(&mut self) {
        let keys: Vec<FlowKey> = self.tcp.keys().copied().collect();
        for key in keys {
            let mut flow = self.tcp.remove(&key).unwrap();
            if self.step(key, &mut flow) {
                self.tcp.insert(key, flow);
            }
        }
    }

    /// Move what connection `key` can move: false once it is over.
    fn step(&mut self, key: FlowKey, f: &mut TcpFlow) -> bool {
        if let Host::Connecting(told) = &f.host {
            match told.try_recv() {
                Err(TryRecvError::Empty) => return true,
                Ok(Ok(stream)) if stream.set_nonblocking(true).is_ok() => {
                    let _ = stream.set_nodelay(true);
                    f.host = Host::Open(stream);
                    f.sent_at = Instant::now() - RTO; // send the SYN+ACK now
                }
                _ => {
                    self.send_tcp(key, RST | ACK, 0, f.rcv_nxt, 0, &[], false);
                    return false;
                }
            }
        }
        let Host::Open(stream) = &mut f.host else { return false };
        // The machine's data to the host, and its end.
        while !f.to_host.is_empty() {
            match stream.write(&f.to_host) {
                Ok(0) => break,
                Ok(n) => {
                    f.to_host.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return self.abort(key, f),
            }
        }
        if f.guest_fin && f.to_host.is_empty() && !f.host_shut {
            let _ = stream.shutdown(Shutdown::Write);
            f.host_shut = true;
        }
        // The host's data for the machine.
        let mut buf = [0u8; 4096];
        while !f.host_eof && f.syn_acked && f.to_guest.len() < BUFFER {
            match stream.read(&mut buf) {
                Ok(0) => f.host_eof = true,
                Ok(n) => f.to_guest.extend(&buf[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return self.abort(key, f),
            }
        }
        if f.fin_acked && f.guest_fin {
            return false; // closed both ways
        }
        // Sending again, from the oldest not acknowledged.
        let now = Instant::now();
        if (f.snd_nxt != f.snd_una || !f.syn_acked) && now.duration_since(f.sent_at) >= RTO {
            f.tries += 1;
            if f.tries > TRIES {
                return self.abort(key, f);
            }
            f.snd_nxt = f.snd_una;
            if !f.fin_acked {
                f.fin_sent = false;
            }
        }
        if !f.syn_acked {
            if f.snd_nxt == f.iss {
                let room = f.room();
                self.send_tcp(key, SYN | ACK, f.iss, f.rcv_nxt, room, &[], true);
                f.snd_nxt = f.iss.wrapping_add(1);
                f.sent_at = now;
            }
            return true;
        }
        // New segments, as far as the machine's window goes (one octet
        // into a closed one, to learn when it opens).
        loop {
            let out = f.snd_nxt.wrapping_sub(f.snd_una) as usize - f.fin_sent as usize;
            let unsent = f.to_guest.len() - out.min(f.to_guest.len());
            let window = f.window.max(if out == 0 { 1 } else { 0 });
            let n = unsent.min(f.mss).min(window.saturating_sub(out));
            let fin = f.host_eof && !f.fin_sent && n == unsent;
            if n == 0 && !fin {
                break;
            }
            let data: Vec<u8> = f.to_guest.range(out..out + n).copied().collect();
            let flags = ACK | if n > 0 { PSH } else { 0 } | if fin { FIN } else { 0 };
            let room = f.room();
            self.send_tcp(key, flags, f.snd_nxt, f.rcv_nxt, room, &data, false);
            if f.snd_nxt == f.snd_una {
                f.sent_at = now;
            }
            f.snd_nxt = f.snd_nxt.wrapping_add(n as u32 + fin as u32);
            f.fin_sent |= fin;
            if fin || n == 0 {
                break;
            }
        }
        true
    }

    fn abort(&mut self, key: FlowKey, f: &TcpFlow) -> bool {
        self.send_tcp(key, RST | ACK, f.snd_nxt, f.rcv_nxt, 0, &[], false);
        false
    }

    // --- frames for the machine ---------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn send_tcp(
        &mut self,
        key: FlowKey,
        flags: u8,
        seq: u32,
        ack: u32,
        room: usize,
        data: &[u8],
        mss: bool,
    ) {
        let hl = if mss { 24 } else { 20 };
        let mut s = Vec::with_capacity(hl + data.len());
        s.extend_from_slice(&key.to_port.to_be_bytes());
        s.extend_from_slice(&key.port.to_be_bytes());
        s.extend_from_slice(&seq.to_be_bytes());
        s.extend_from_slice(&ack.to_be_bytes());
        s.push((hl as u8 / 4) << 4);
        s.push(flags);
        s.extend_from_slice(&(room.min(65535) as u16).to_be_bytes());
        s.extend_from_slice(&[0, 0, 0, 0]);
        if mss {
            s.extend_from_slice(&[2, 4]);
            s.extend_from_slice(&(MSS as u16).to_be_bytes());
        }
        s.extend_from_slice(data);
        let sum = pseudo(key.to, self.guest_ip, TCP, s.len());
        set_checksum(&mut s, 16, sum);
        self.send_ip(key.to, TCP, &s);
    }

    fn send_ip(&mut self, src: Ipv4Addr, proto: u8, body: &[u8]) {
        let mut f = Vec::with_capacity(34 + body.len());
        f.extend_from_slice(&self.guest_mac);
        f.extend_from_slice(&GATEWAY_MAC);
        f.extend_from_slice(&ETH_IP.to_be_bytes());
        f.extend_from_slice(&[0x45, 0]);
        f.extend_from_slice(&((20 + body.len()) as u16).to_be_bytes());
        f.extend_from_slice(&self.ident.to_be_bytes());
        self.ident = self.ident.wrapping_add(1);
        f.extend_from_slice(&[0x40, 0, 64, proto, 0, 0]);
        f.extend_from_slice(&src.octets());
        f.extend_from_slice(&self.guest_ip.octets());
        let sum = checksum(&f[14..34], 0);
        f[24..26].copy_from_slice(&sum.to_be_bytes());
        f.extend_from_slice(body);
        self.out.push_back(f);
    }
}

impl TcpFlow {
    /// The window to offer the machine.
    fn room(&self) -> usize {
        BUFFER - self.to_host.len()
    }
}

impl Link for UserLink {
    fn send(&mut self, frame: &[u8]) {
        if frame.len() < 14 {
            return;
        }
        let dst = &frame[..6];
        if dst != GATEWAY_MAC && dst != [0xff; 6] {
            return;
        }
        match be16(&frame[12..]) {
            ETH_ARP => self.arp(frame),
            ETH_IP if dst == GATEWAY_MAC => self.ipv4(frame),
            _ => {}
        }
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        if self.out.is_empty() {
            self.poll();
        }
        self.out.pop_front()
    }

    fn may_receive(&self) -> bool {
        true
    }
}

// --- helpers -----------------------------------------------------------------

/// The host's address for one the machine used: the gateway is the host.
fn host_side(a: Ipv4Addr) -> Ipv4Addr {
    if a == GATEWAY { Ipv4Addr::LOCALHOST } else { a }
}

/// The machine's address for one on the host's side.
fn guest_side(a: Ipv4Addr) -> Ipv4Addr {
    if a.is_loopback() { GATEWAY } else { a }
}

fn be16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn ip_at(b: &[u8]) -> Ipv4Addr {
    Ipv4Addr::new(b[0], b[1], b[2], b[3])
}

/// The Internet checksum of `b`, starting from the sum `start`.
fn checksum(b: &[u8], start: u32) -> u16 {
    let mut sum = start;
    for pair in b.chunks(2) {
        sum += (pair[0] as u32) << 8 | *pair.get(1).unwrap_or(&0) as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn set_checksum(b: &mut [u8], at: usize, start: u32) {
    b[at] = 0;
    b[at + 1] = 0;
    let sum = checksum(b, start);
    b[at..at + 2].copy_from_slice(&sum.to_be_bytes());
}

/// The sum of the pseudo-header for UDP and TCP.
fn pseudo(src: Ipv4Addr, dst: Ipv4Addr, proto: u8, len: usize) -> u32 {
    let (s, d) = (u32::from(src), u32::from(dst));
    (s >> 16) + (s & 0xffff) + (d >> 16) + (d & 0xffff) + proto as u32 + len as u32
}

/// The MSS a SYN's options give, or TCP's default.
fn mss_option(mut o: &[u8]) -> usize {
    while !o.is_empty() {
        match o[0] {
            0 => break,
            1 => o = &o[1..],
            kind => {
                let len = *o.get(1).unwrap_or(&0) as usize;
                if len < 2 || len > o.len() {
                    break;
                }
                if kind == 2 && len == 4 {
                    return (be16(&o[2..]) as usize).clamp(64, MSS);
                }
                o = &o[len..];
            }
        }
    }
    536
}

/// A socket for ICMP echo: an unprivileged ping socket if the host
/// allows one, else a raw socket, which needs root; None if neither.
#[cfg(unix)]
fn ping_socket() -> Option<PingSocket> {
    use std::os::fd::FromRawFd;
    unsafe extern "C" {
        fn socket(domain: i32, kind: i32, protocol: i32) -> i32;
    }
    const AF_INET: i32 = 2;
    const SOCK_DGRAM: i32 = 2;
    const SOCK_RAW: i32 = 3;
    const IPPROTO_ICMP: i32 = 1;
    for (kind, raw) in [(SOCK_DGRAM, false), (SOCK_RAW, true)] {
        // SAFETY: socket() takes plain integers and returns a new
        // descriptor or -1; a new descriptor is ours to own.
        let fd = unsafe { socket(AF_INET, kind, IPPROTO_ICMP) };
        if fd >= 0 {
            // SAFETY: fd is a fresh socket nothing else owns.
            let socket = unsafe { UdpSocket::from_raw_fd(fd) };
            if socket.set_nonblocking(true).is_ok() {
                return Some(PingSocket { socket, raw });
            }
        }
    }
    None
}

#[cfg(not(unix))]
fn ping_socket() -> Option<PingSocket> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    const GUEST: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
    const GUEST_MAC: [u8; 6] = [2, 0, 0, 0, 0, 1];

    fn ip_frame(dst: Ipv4Addr, proto: u8, body: &[u8]) -> Vec<u8> {
        let mut f = Vec::new();
        f.extend_from_slice(&GATEWAY_MAC);
        f.extend_from_slice(&GUEST_MAC);
        f.extend_from_slice(&ETH_IP.to_be_bytes());
        f.extend_from_slice(&[0x45, 0]);
        f.extend_from_slice(&((20 + body.len()) as u16).to_be_bytes());
        f.extend_from_slice(&[0, 0, 0, 0, 64, proto, 0, 0]);
        f.extend_from_slice(&GUEST.octets());
        f.extend_from_slice(&dst.octets());
        let sum = checksum(&f[14..34], 0);
        f[24..26].copy_from_slice(&sum.to_be_bytes());
        f.extend_from_slice(body);
        f
    }

    fn segment(
        dst: Ipv4Addr,
        sport: u16,
        dport: u16,
        seq: u32,
        ack: u32,
        flags: u8,
        data: &[u8],
    ) -> Vec<u8> {
        let mut s = Vec::new();
        s.extend_from_slice(&sport.to_be_bytes());
        s.extend_from_slice(&dport.to_be_bytes());
        s.extend_from_slice(&seq.to_be_bytes());
        s.extend_from_slice(&ack.to_be_bytes());
        s.extend_from_slice(&[0x50, flags, 0x05, 0xb4, 0, 0, 0, 0]);
        s.extend_from_slice(data);
        let sum = pseudo(GUEST, dst, TCP, s.len());
        set_checksum(&mut s, 16, sum);
        ip_frame(dst, TCP, &s)
    }

    /// The next frame, waiting for the host up to two seconds.
    fn next(link: &mut UserLink) -> Vec<u8> {
        for _ in 0..400 {
            if let Some(f) = link.receive() {
                return f;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("no frame");
    }

    /// A frame for the machine: its IP payload, checked.
    fn payload(f: &[u8]) -> (Ipv4Addr, u8, Vec<u8>) {
        assert_eq!(&f[..6], &GUEST_MAC);
        assert_eq!(&f[6..12], &GATEWAY_MAC);
        assert_eq!(checksum(&f[14..34], 0), 0);
        let total = be16(&f[16..]) as usize;
        let src = ip_at(&f[26..]);
        let body = f[34..14 + total].to_vec();
        if f[23] != ICMP {
            assert_eq!(checksum(&body, pseudo(src, GUEST, f[23], body.len())), 0);
        }
        (src, f[23], body)
    }

    #[test]
    fn the_gateway_answers_arp_and_pings() {
        let mut link = UserLink::new();
        let mut arp = vec![0xff; 6];
        arp.extend_from_slice(&GUEST_MAC);
        arp.extend_from_slice(&[8, 6, 0, 1, 8, 0, 6, 4, 0, 1]);
        arp.extend_from_slice(&GUEST_MAC);
        arp.extend_from_slice(&GUEST.octets());
        arp.extend_from_slice(&[0; 6]);
        arp.extend_from_slice(&GATEWAY.octets());
        link.send(&arp);
        let r = link.receive().unwrap();
        assert_eq!(&r[..6], &GUEST_MAC);
        assert_eq!(be16(&r[20..]), 2);
        assert_eq!(&r[22..28], &GATEWAY_MAC);
        assert_eq!(ip_at(&r[28..]), GATEWAY);

        let mut echo = vec![8, 0, 0, 0, 0x12, 0x34, 0, 7, 1, 2, 3];
        set_checksum(&mut echo, 2, 0);
        link.send(&ip_frame(GATEWAY, ICMP, &echo));
        let (src, proto, body) = payload(&link.receive().unwrap());
        assert_eq!((src, proto, body[0]), (GATEWAY, ICMP, 0));
        assert_eq!(&body[4..], &echo[4..]);
        assert_eq!(checksum(&body, 0), 0);
    }

    #[test]
    fn udp_goes_to_the_host_and_answers_come_back() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = server.local_addr().unwrap().port();
        let mut link = UserLink::new();
        let mut d = vec![0x13, 0x88, (port >> 8) as u8, port as u8, 0, 13, 0, 0];
        d.extend_from_slice(b"hello");
        let sum = pseudo(GUEST, GATEWAY, UDP, d.len());
        set_checksum(&mut d, 6, sum);
        link.send(&ip_frame(GATEWAY, UDP, &d));
        let mut buf = [0u8; 100];
        let (n, from) = server.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");
        server.send_to(b"back", from).unwrap();
        let (src, proto, body) = payload(&next(&mut link));
        assert_eq!((src, proto), (GATEWAY, UDP));
        assert_eq!((be16(&body), be16(&body[2..])), (port, 5000));
        assert_eq!(&body[8..], b"back");
    }

    #[test]
    fn a_tcp_connection_carries_data_both_ways_and_closes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut link = UserLink::new();
        let tcp =
            |seq, ack, flags, data: &[u8]| segment(GATEWAY, 40000, port, seq, ack, flags, data);
        link.send(&tcp(100, 0, SYN, &[]));
        let (mut host, _) = listener.accept().unwrap();
        let (_, _, synack) = payload(&next(&mut link));
        assert_eq!(synack[13], SYN | ACK);
        assert_eq!(be32(&synack[8..]), 101);
        let iss = be32(&synack[4..]);
        link.send(&tcp(101, iss + 1, ACK, &[]));
        link.send(&tcp(101, iss + 1, ACK | PSH, b"ping"));
        let (_, _, ack) = payload(&next(&mut link));
        assert_eq!((ack[13], be32(&ack[8..])), (ACK, 105));
        link.poll(); // writes it to the host
        let mut buf = [0u8; 4];
        host.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        host.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"ping");

        host.write_all(b"pong").unwrap();
        drop(host); // and the host closes
        let (_, _, data) = payload(&next(&mut link));
        assert_eq!(be32(&data[4..]), iss + 1);
        let mut got = data[20..].to_vec();
        let mut fin = data[13] & FIN != 0;
        while !fin {
            let (_, _, more) = payload(&next(&mut link));
            got.extend_from_slice(&more[20..]);
            fin = more[13] & FIN != 0;
        }
        assert_eq!(got, b"pong");
        link.send(&tcp(105, iss + 6, ACK | FIN, &[]));
        let (_, _, last) = payload(&next(&mut link));
        assert_eq!((last[13], be32(&last[8..])), (ACK, 106));
        link.receive();
        assert!(link.tcp.is_empty(), "closed both ways");
    }

    #[test]
    fn a_refused_connection_is_reset() {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let mut link = UserLink::new(); // the listener has gone
        link.send(&segment(GATEWAY, 40001, port, 7, 0, SYN, &[]));
        let (_, _, rst) = payload(&next(&mut link));
        assert_eq!((rst[13], be32(&rst[8..])), (RST | ACK, 8));
        assert!(link.tcp.is_empty());
    }
}
