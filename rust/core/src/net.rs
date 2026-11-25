//! The network device (docs/platform.md, The network device): an
//! Ethernet interface that sends and receives whole frames by DMA, one
//! octet per tryte, then raises interrupt source 3.
//!
//! The device keeps its registers, the frame being sent and the frames
//! that have arrived and wait for a buffer. The bus moves frames between
//! the device and RAM, since that needs RAM; where frames go and come
//! from is a [`Link`]'s business, so the core touches no sockets.

use std::collections::VecDeque;

use crate::bus::PAGE;

pub const NET_PAGE: i64 = -16;
const CENTRE: i64 = NET_PAGE * PAGE;

// Registers, as offsets from the page's centre.
const PRESENT: i64 = -27;
const LINK: i64 = -24;
const MAC_HI: i64 = -21;
const MAC_LO: i64 = -18;
const DROPPED: i64 = -15;
const TX_ADDR: i64 = -12;
const TX_LEN: i64 = -9;
const TX_COMMAND: i64 = -6;
const TX_STATUS: i64 = -3;
const TX_ACK: i64 = 0;
const RX_ADDR: i64 = 3;
const RX_SIZE: i64 = 6;
const RX_COMMAND: i64 = 9;
const RX_STATUS: i64 = 12;
const RX_LEN: i64 = 15;
const RX_ACK: i64 = 18;

/// What PRESENT reads: the device's version.
pub const NET_VERSION: i64 = 1;

/// The command each channel takes.
pub const NET_SEND: i64 = 1;
pub const NET_RECEIVE: i64 = 1;

/// STATUS values, for both channels.
pub const NET_BUSY: i64 = 1;
pub const NET_DONE: i64 = 0;
pub const NET_FAILED: i64 = -1;

/// The shortest and longest frame: an Ethernet header (destination,
/// source, type) and at most 1,500 octets of payload. No preamble and no
/// frame check sequence.
pub const FRAME_MIN: usize = 14;
pub const FRAME_MAX: usize = 1514;

/// Frames that may wait in the device for a receive buffer; more are
/// dropped and counted in DROPPED.
pub const RX_QUEUE: usize = 9;

/// Rounds a send takes: a fixed setup, then one round per octet.
pub const TX_SETUP_ROUNDS: i64 = 100;

/// The link is looked at for arriving frames every this many rounds, so
/// a socket is not asked on every step.
pub const POLL_ROUNDS: i64 = 1000;

/// The address the interface has unless told otherwise: locally
/// administered, unicast.
pub const DEFAULT_MAC: u64 = 0x02_00_00_00_00_01;

/// The address of the register at `offset`: for tests.
pub fn net_addr(offset: i64) -> i64 {
    CENTRE + offset
}

/// Where frames go and come from: the wire.
pub trait Link {
    /// A frame leaves the interface.
    fn send(&mut self, frame: &[u8]);
    /// The next frame that has arrived, if any; never waits.
    fn receive(&mut self) -> Option<Vec<u8>>;
    /// Whether a frame could arrive without the machine doing anything:
    /// true for the outside world, which keeps time moving while every
    /// hart waits.
    fn may_receive(&self) -> bool;
}

/// A link that hands every frame sent straight back, as if a cable ran
/// from the interface to itself. Runs stay deterministic.
#[derive(Default)]
pub struct Loopback {
    frames: VecDeque<Vec<u8>>,
}

impl Link for Loopback {
    fn send(&mut self, frame: &[u8]) {
        self.frames.push_back(frame.to_vec());
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        self.frames.pop_front()
    }

    fn may_receive(&self) -> bool {
        !self.frames.is_empty()
    }
}

/// A frame on its way out.
pub struct Sending {
    pub frame: Vec<u8>,
    pub done_at: i64,
}

/// A receive buffer the program has given.
#[derive(Clone, Copy)]
pub struct Buffer {
    pub addr: i64,
    pub size: i64,
}

pub struct NetDevice {
    pub link: Option<Box<dyn Link>>,
    pub mac: u64,
    pub dropped: i64,
    tx_addr: i64,
    tx_len: i64,
    tx_status: i64,
    rx_addr: i64,
    rx_size: i64,
    rx_status: i64,
    rx_len: i64,
    /// A send has finished and not been acknowledged.
    pub tx_finished: bool,
    /// A receive has finished and not been acknowledged.
    pub rx_finished: bool,
    pub sending: Option<Sending>,
    /// The receive buffer waiting for a frame.
    pub armed: Option<Buffer>,
    /// Frames that have arrived and wait for a buffer.
    pub arrived: VecDeque<Vec<u8>>,
}

impl Default for NetDevice {
    fn default() -> Self {
        NetDevice {
            link: None,
            mac: DEFAULT_MAC,
            dropped: 0,
            tx_addr: 0,
            tx_len: 0,
            tx_status: NET_DONE,
            rx_addr: 0,
            rx_size: 0,
            rx_status: NET_DONE,
            rx_len: 0,
            tx_finished: false,
            rx_finished: false,
            sending: None,
            armed: None,
            arrived: VecDeque::new(),
        }
    }
}

/// What a COMMAND write asks of the bus.
pub enum Request {
    Nothing,
    /// Read the frame of `len` octets at `addr` and send it.
    Send {
        addr: i64,
        len: i64,
    },
    /// Check the buffer and arm it.
    Receive(Buffer),
}

impl NetDevice {
    /// Interrupt source 3: either channel has finished and not been
    /// acknowledged.
    pub fn wants_attention(&self) -> bool {
        self.tx_finished || self.rx_finished
    }

    /// When the frame in flight finishes, if there is one.
    pub fn done_at(&self) -> Option<i64> {
        self.sending.as_ref().map(|s| s.done_at)
    }

    pub fn read(&self, addr: i64) -> i64 {
        match addr - CENTRE {
            PRESENT => NET_VERSION,
            LINK => self.link.is_some() as i64,
            MAC_HI => (self.mac >> 24) as i64,
            MAC_LO => (self.mac & 0xFF_FFFF) as i64,
            DROPPED => self.dropped,
            TX_ADDR => self.tx_addr,
            TX_LEN => self.tx_len,
            TX_STATUS => self.tx_status,
            RX_ADDR => self.rx_addr,
            RX_SIZE => self.rx_size,
            RX_STATUS => self.rx_status,
            RX_LEN => self.rx_len,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: i64, value: i64) -> Request {
        match addr - CENTRE {
            TX_ADDR => self.tx_addr = value,
            TX_LEN => self.tx_len = value,
            TX_ACK => self.tx_finished = false,
            RX_ADDR => self.rx_addr = value,
            RX_SIZE => self.rx_size = value,
            RX_ACK => self.rx_finished = false,
            TX_COMMAND if value == NET_SEND && self.sending.is_none() => {
                return Request::Send { addr: self.tx_addr, len: self.tx_len };
            }
            RX_COMMAND if value == NET_RECEIVE && self.armed.is_none() => {
                return Request::Receive(Buffer { addr: self.rx_addr, size: self.rx_size });
            }
            _ => {}
        }
        Request::Nothing
    }

    pub fn fail_send(&mut self) {
        self.tx_status = NET_FAILED;
        self.tx_finished = true;
    }

    pub fn begin_send(&mut self, sending: Sending) {
        self.tx_status = NET_BUSY;
        self.sending = Some(sending);
    }

    /// The frame in flight, if its time has come; the send is finished.
    pub fn end_send(&mut self, now: i64) -> Option<Vec<u8>> {
        if !self.done_at().is_some_and(|t| t <= now) {
            return None;
        }
        self.tx_status = NET_DONE;
        self.tx_finished = true;
        self.sending.take().map(|s| s.frame)
    }

    pub fn fail_receive(&mut self) {
        self.rx_status = NET_FAILED;
        self.rx_finished = true;
    }

    pub fn arm(&mut self, buffer: Buffer) {
        self.rx_status = NET_BUSY;
        self.armed = Some(buffer);
    }

    /// A frame has come off the link: keep it for a buffer, or drop it if
    /// it is no frame (too short or too long) or too many wait already.
    pub fn arrive(&mut self, frame: Vec<u8>) {
        let fits = (FRAME_MIN..=FRAME_MAX).contains(&frame.len());
        if fits && self.arrived.len() < RX_QUEUE {
            self.arrived.push_back(frame);
        } else {
            self.dropped += 1;
        }
    }

    /// The next frame for the armed buffer, and where it goes, if both
    /// are there. A frame too long for the buffer is dropped and counted,
    /// and the buffer stays armed for the next.
    pub fn next_delivery(&mut self) -> Option<(Buffer, Vec<u8>)> {
        let buffer = self.armed?;
        while let Some(frame) = self.arrived.pop_front() {
            if frame.len() as i64 <= buffer.size {
                self.armed = None;
                self.rx_status = NET_DONE;
                self.rx_len = frame.len() as i64;
                self.rx_finished = true;
                return Some((buffer, frame));
            }
            self.dropped += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_read_back_and_no_link_reads_0() {
        let mut d = NetDevice::default();
        assert_eq!(d.read(net_addr(PRESENT)), NET_VERSION);
        assert_eq!(d.read(net_addr(LINK)), 0);
        assert_eq!(d.read(net_addr(MAC_HI)), 0x02_00_00);
        assert_eq!(d.read(net_addr(MAC_LO)), 0x00_00_01);
        d.write(net_addr(TX_ADDR), 19683);
        d.write(net_addr(RX_SIZE), 1514);
        assert_eq!(d.read(net_addr(TX_ADDR)), 19683);
        assert_eq!(d.read(net_addr(RX_SIZE)), 1514);
        d.link = Some(Box::new(Loopback::default()));
        assert_eq!(d.read(net_addr(LINK)), 1);
    }

    #[test]
    fn a_send_is_refused_while_one_is_in_flight() {
        let mut d = NetDevice::default();
        assert!(matches!(d.write(net_addr(TX_COMMAND), 0), Request::Nothing));
        assert!(matches!(d.write(net_addr(TX_COMMAND), NET_SEND), Request::Send { .. }));
        d.begin_send(Sending { frame: vec![0; 14], done_at: 9 });
        assert_eq!(d.read(net_addr(TX_STATUS)), NET_BUSY);
        assert!(matches!(d.write(net_addr(TX_COMMAND), NET_SEND), Request::Nothing));
        assert_eq!(d.end_send(8), None, "not yet");
        assert_eq!(d.end_send(9), Some(vec![0; 14]));
        assert!(d.wants_attention());
        d.write(net_addr(TX_ACK), 0);
        assert!(!d.wants_attention());
        assert_eq!(d.read(net_addr(TX_STATUS)), NET_DONE);
    }

    #[test]
    fn frames_wait_for_a_buffer_and_overflow_is_counted() {
        let mut d = NetDevice::default();
        for i in 0..RX_QUEUE + 2 {
            d.arrive(vec![i as u8; 20]);
        }
        assert_eq!(d.dropped, 2);
        assert!(d.next_delivery().is_none(), "no buffer");
        d.arrived.clear();
        d.arrive(vec![0; FRAME_MIN - 1]);
        d.arrive(vec![0; FRAME_MAX + 1]);
        assert_eq!(d.dropped, 4, "no frames");
        for i in 0..RX_QUEUE {
            d.arrive(vec![i as u8; 20]);
        }
        d.arm(Buffer { addr: 19683, size: 20 });
        let (buffer, frame) = d.next_delivery().unwrap();
        assert_eq!((buffer.addr, frame[0]), (19683, 0));
        assert_eq!(d.read(net_addr(RX_LEN)), 20);
        assert!(d.rx_finished && d.armed.is_none());
        assert!(d.next_delivery().is_none(), "one frame per buffer");
    }

    #[test]
    fn a_frame_too_long_for_the_buffer_is_dropped() {
        let mut d = NetDevice::default();
        d.arrive(vec![1; 30]);
        d.arrive(vec![2; 16]);
        d.arm(Buffer { addr: 19683, size: 20 });
        let (_, frame) = d.next_delivery().unwrap();
        assert_eq!(frame, vec![2; 16]);
        assert_eq!(d.dropped, 1);
    }

    #[test]
    fn loopback_hands_frames_back_in_order() {
        let mut l = Loopback::default();
        assert!(!l.may_receive());
        l.send(&[1, 2]);
        l.send(&[3]);
        assert!(l.may_receive());
        assert_eq!(l.receive(), Some(vec![1, 2]));
        assert_eq!(l.receive(), Some(vec![3]));
        assert_eq!(l.receive(), None);
    }
}
