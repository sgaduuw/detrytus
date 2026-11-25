//! Physical memory and devices (docs/isa.md 5.3, docs/platform.md).
//!
//! The sign of a physical page number picks the region: page 0 is the
//! boot page (read-only to the CPU), positive pages up to `ram_pages`
//! are RAM, negative pages are devices. The platform's devices are built
//! in: the console at page −1, power control at page −2, the hart
//! controller at pages −3 and −4, the interrupt controller at page −13,
//! the block device at page −14, the binary floating point
//! coprocessor at page −15 and the network device at page −16.
//!
//! The bus also keeps each hart's `lr` reservation, so that a store by
//! one hart breaks another's.

use std::collections::{HashMap, VecDeque};

use crate::block::{BLOCK_PAGE, BLOCK_READ, BLOCK_TRYTES, BlockDevice, Request, Transfer};
use crate::block::{ROUNDS_PER_BLOCK, SETUP_ROUNDS};
use crate::fpu::{FPU_PAGE, Fpu};
use crate::harts::{HartController, LOCAL_PAGE, SLOTS_PAGE};
use crate::irq::{BLOCK_SOURCE, CONSOLE_SOURCE, IRQ_PAGE, InterruptController, Lines, NET_SOURCE};
use crate::net::{self, Buffer, FRAME_MAX, FRAME_MIN, NET_PAGE, NetDevice, POLL_ROUNDS};
use crate::net::{Sending, TX_SETUP_ROUNDS};
use crate::trits::{TRYTE_MOD, TRYTE_TRITS, low, split_high};

pub const PAGE_TRITS: u32 = 9;
pub const PAGE: i64 = 19683;
const HALF_PAGE: i64 = PAGE / 2;

pub const CONSOLE_PAGE: i64 = -1;
pub const POWER_PAGE: i64 = -2;
pub const CONSOLE_OUT: i64 = CONSOLE_PAGE * PAGE;
pub const CONSOLE_IN: i64 = CONSOLE_OUT + 3;
pub const POWER_OFF: i64 = POWER_PAGE * PAGE;

/// What console in reads when there is no character: nothing yet, or
/// nothing ever again. Any negative read is "no character".
pub const CONSOLE_EMPTY: i64 = -1;
pub const CONSOLE_CLOSED: i64 = -2;

/// What a physical access is for: devices hold neither code nor page
/// tables, and only stores are refused by the boot page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Fetch,
    Load,
    Store,
    Walk,
}

/// Why a physical access did not complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusError {
    /// Nothing behind the address, or an access it refuses.
    Fault(i64),
    /// The program wrote its exit code to the power-off device.
    Halt(i64),
}

/// Characters out, characters in: one code point per access. Once
/// closed, a read past the end of the input says no more will come.
#[derive(Default)]
pub struct Console {
    pub output: Vec<u32>,
    pub input: VecDeque<i64>,
    pub closed: bool,
}

impl Console {
    pub fn feed(&mut self, text: &str) {
        self.input.extend(text.chars().map(|c| c as i64));
    }

    /// Take the next code point, or say why there is none.
    pub fn read(&mut self) -> i64 {
        match self.input.pop_front() {
            Some(c) => c,
            None if self.closed => CONSOLE_CLOSED,
            None => CONSOLE_EMPTY,
        }
    }

    /// Interrupt source 1: a read would not return CONSOLE_EMPTY.
    pub fn wants_attention(&self) -> bool {
        self.closed || !self.input.is_empty()
    }

    /// The output as text; code points that are not characters (lone
    /// surrogates) become U+FFFD.
    pub fn text(&self) -> String {
        self.output.iter().map(|&c| char::from_u32(c).unwrap_or('\u{FFFD}')).collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Boot,
    Ram,
    Console,
    Power,
    Harts,
    Irq,
    Block,
    Fpu,
    Net,
}

pub fn page_of(addr: i64) -> i64 {
    split_high(addr, PAGE_TRITS)
}

/// Tryte-addressed physical memory. Pages are allocated on first write,
/// so a large RAM costs nothing until it is touched.
pub struct Bus {
    pub ram_pages: i64,
    pages: HashMap<i64, Box<[i16]>>,
    pub console: Console,
    pub harts: HartController,
    pub irq: InterruptController,
    pub block: BlockDevice,
    pub fpu: Fpu,
    pub net: NetDevice,
    /// The machine's clock, set each round, for devices that take time.
    pub now: i64,
    /// The hart whose access this is: the machine sets it before each
    /// hart's step.
    pub current: i64,
    /// Each hart's reserved word, by physical address.
    reservations: HashMap<i64, i64>,
}

impl Bus {
    /// A bus with one hart, hart 0.
    pub fn new(ram_pages: i64) -> Self {
        Self::with_harts(ram_pages, &[0])
    }

    /// A bus whose hart controller has the harts in `present`.
    pub fn with_harts(ram_pages: i64, present: &[i64]) -> Self {
        Bus {
            ram_pages,
            pages: HashMap::new(),
            console: Console::default(),
            harts: HartController::new(present),
            irq: InterruptController::default(),
            block: BlockDevice::default(),
            fpu: Fpu::default(),
            net: NetDevice::default(),
            now: 0,
            current: 0,
            reservations: HashMap::new(),
        }
    }

    /// The devices' interrupt lines, by source.
    fn lines(&self) -> Lines {
        let mut lines = Lines::default();
        lines[CONSOLE_SOURCE as usize] = self.console.wants_attention();
        lines[BLOCK_SOURCE as usize] = self.block.finished;
        lines[NET_SOURCE as usize] = self.net.wants_attention();
        lines
    }

    /// Whether hart `h`'s external interrupt is pending: a device source
    /// it can claim, or hot-plug events for it.
    pub fn external_pending(&self, h: i64) -> bool {
        self.harts.event_pending(h) || self.irq.pending_for(h, &self.lines())
    }

    /// Take a block command: read its list of pages, check it, and start
    /// it, or fail it at once.
    fn start_transfer(&mut self, command: i64, first: i64, count: i64, list: i64) {
        let pages =
            if self.block.in_range(first, count) { self.page_list(list, count) } else { None };
        match pages {
            Some(pages) => self.block.begin(Transfer {
                command,
                first,
                pages,
                done_at: self.now + SETUP_ROUNDS + ROUNDS_PER_BLOCK * count,
            }),
            None => self.block.fail(),
        }
    }

    /// The `count` words at physical address `list`, if it is aligned and
    /// each word names a RAM page. Memory with nothing behind it, devices
    /// included, reads as 0, which names no RAM page.
    fn page_list(&self, list: i64, count: i64) -> Option<Vec<i64>> {
        if list.rem_euclid(3) != 0 {
            return None;
        }
        (0..count)
            .map(|i| Some(self.word(list + 3 * i)).filter(|p| (1..=self.ram_pages).contains(p)))
            .collect()
    }

    fn word(&self, addr: i64) -> i64 {
        self.get(addr - 1) + self.get(addr) * TRYTE_MOD + self.get(addr + 1) * TRYTE_MOD * TRYTE_MOD
    }

    /// Whether `len` trytes from `addr` are all RAM. RAM is one run of
    /// pages, so the two ends tell.
    fn in_ram(&self, addr: i64, len: i64) -> bool {
        let ram = 1..=self.ram_pages;
        len >= 1 && ram.contains(&page_of(addr)) && ram.contains(&page_of(addr + len - 1))
    }

    /// Take a send: read the frame, one octet per tryte, and start it, or
    /// fail it at once.
    fn start_send(&mut self, addr: i64, len: i64) {
        let fits = (FRAME_MIN as i64..=FRAME_MAX as i64).contains(&len);
        let frame = if self.net.link.is_some() && fits && self.in_ram(addr, len) {
            (0..len).map(|i| u8::try_from(self.get(addr + i)).ok()).collect()
        } else {
            None
        };
        match frame {
            Some(frame) => {
                self.net.begin_send(Sending { frame, done_at: self.now + TX_SETUP_ROUNDS + len })
            }
            None => self.net.fail_send(),
        }
    }

    fn start_receive(&mut self, buffer: Buffer) {
        if self.in_ram(buffer.addr, buffer.size) {
            self.net.arm(buffer);
        } else {
            self.net.fail_receive();
        }
    }

    /// When the network device next needs a round, for a machine whose
    /// harts all wait: the round before its send finishes, the round
    /// before the link is next looked at if a frame could come, or now if
    /// a frame is ready for the armed buffer.
    pub fn net_wake(&self, now: i64) -> Option<i64> {
        if self.net.armed.is_some() && !self.net.arrived.is_empty() {
            return Some(now);
        }
        let poll = self.net.link.as_ref().filter(|l| l.may_receive()).map(|_| {
            let next = now + 1;
            next + (POLL_ROUNDS - next.rem_euclid(POLL_ROUNDS)) % POLL_ROUNDS - 1
        });
        let send = self.net.done_at().map(|t| (t - 1).max(now));
        poll.into_iter().chain(send).min()
    }

    /// Move frames: finish the send in flight onto the link, take what
    /// has arrived from the link, and fill the armed buffer.
    fn advance_net(&mut self) {
        if let Some(frame) = self.net.end_send(self.now)
            && let Some(link) = &mut self.net.link
        {
            link.send(&frame);
        }
        if self.now.rem_euclid(POLL_ROUNDS) == 0
            && let Some(link) = &mut self.net.link
        {
            let mut frames = Vec::new();
            while let Some(frame) = link.receive() {
                frames.push(frame);
            }
            for frame in frames {
                self.net.arrive(frame);
            }
        }
        if let Some((buffer, frame)) = self.net.next_delivery() {
            for (i, &octet) in frame.iter().enumerate() {
                self.poke(buffer.addr + i as i64, octet as i64);
            }
            let (lo, hi) = (buffer.addr, buffer.addr + frame.len() as i64 - 1);
            self.reservations.retain(|_, r| *r + 1 < lo || *r - 1 > hi);
        }
    }

    /// Let the devices that take time move on to `now`.
    pub fn advance(&mut self) {
        self.advance_block();
        self.advance_net();
    }

    /// Finish the block command in flight if its time has come: copy its
    /// blocks into its pages, or its pages to its blocks.
    fn advance_block(&mut self) {
        if !self.block.done_at().is_some_and(|t| t <= self.now) {
            return;
        }
        let t = self.block.transfer.take().unwrap();
        let disk = self.block.disk.as_mut().expect("a transfer needs a disk");
        let count = t.pages.len() as i64;
        if t.command == BLOCK_READ {
            let data = disk.read(t.first, count);
            for (&page, block) in t.pages.iter().zip(data.chunks(BLOCK_TRYTES)) {
                let mem = self
                    .pages
                    .entry(page)
                    .or_insert_with(|| vec![0; PAGE as usize].into_boxed_slice());
                mem.copy_from_slice(block);
            }
            let lo = t.pages.iter().map(|p| p * PAGE - HALF_PAGE);
            for start in lo.collect::<Vec<_>>() {
                self.reservations.retain(|_, r| *r + 1 < start || *r - 1 > start + PAGE - 1);
            }
        } else {
            let mut data = Vec::with_capacity(t.pages.len() * BLOCK_TRYTES);
            for page in &t.pages {
                match self.pages.get(page) {
                    Some(mem) => data.extend_from_slice(mem),
                    None => data.extend(std::iter::repeat_n(0, BLOCK_TRYTES)),
                }
            }
            disk.write(t.first, &data);
        }
        self.block.end();
    }

    /// Reserve the word at a physical address for the current hart.
    pub fn reserve(&mut self, addr: i64) {
        self.reservations.insert(self.current, addr);
    }

    /// Whether hart h still holds a reservation, without using it up.
    pub fn holds_reservation(&self, h: i64) -> bool {
        self.reservations.contains_key(&h)
    }

    /// Whether the current hart's reservation survives; it is used up
    /// either way.
    pub fn take_reservation(&mut self) -> bool {
        self.reservations.remove(&self.current).is_some()
    }

    /// Break other harts' reservations on any word a store touches.
    fn break_reservations(&mut self, addr: i64, width: u32) {
        let (lo, hi) = if width == 3 { (addr - 1, addr + 1) } else { (addr, addr) };
        let current = self.current;
        self.reservations.retain(|&h, &mut r| h == current || r + 1 < lo || r - 1 > hi);
    }

    fn region(&self, addr: i64, access: Access) -> Result<Region, BusError> {
        let page = page_of(addr);
        let device = matches!(access, Access::Fetch | Access::Walk);
        match page {
            0 if access == Access::Store => Err(BusError::Fault(addr)),
            0 => Ok(Region::Boot),
            p if p > 0 && p <= self.ram_pages => Ok(Region::Ram),
            CONSOLE_PAGE if !device => Ok(Region::Console),
            POWER_PAGE if !device => Ok(Region::Power),
            SLOTS_PAGE | LOCAL_PAGE if !device => Ok(Region::Harts),
            IRQ_PAGE if !device => Ok(Region::Irq),
            BLOCK_PAGE if !device => Ok(Region::Block),
            FPU_PAGE if !device => Ok(Region::Fpu),
            NET_PAGE if !device => Ok(Region::Net),
            _ => Err(BusError::Fault(addr)),
        }
    }

    fn get(&self, addr: i64) -> i64 {
        let page = page_of(addr);
        self.pages.get(&page).map_or(0, |p| p[(addr - page * PAGE + HALF_PAGE) as usize] as i64)
    }

    /// Memory for a debugger: None for devices, whose reads can have side
    /// effects, and for addresses with nothing behind them.
    pub fn peek(&mut self, addr: i64, width: u32) -> Option<i64> {
        self.read(addr, width, Access::Walk).ok()
    }

    /// Every non-zero tryte of memory, in address order.
    pub fn trytes(&self) -> Vec<(i64, i64)> {
        let mut out: Vec<(i64, i64)> = self
            .pages
            .iter()
            .flat_map(|(&page, mem)| {
                mem.iter()
                    .enumerate()
                    .filter(|(_, t)| **t != 0)
                    .map(move |(i, &t)| (page * PAGE + i as i64 - HALF_PAGE, t as i64))
            })
            .collect();
        out.sort_unstable();
        out
    }

    /// Write a tryte without access checks: for loaders, not the CPU.
    pub fn poke(&mut self, addr: i64, tryte: i64) {
        let page = page_of(addr);
        let mem =
            self.pages.entry(page).or_insert_with(|| vec![0; PAGE as usize].into_boxed_slice());
        mem[(addr - page * PAGE + HALF_PAGE) as usize] = low(tryte, TRYTE_TRITS) as i16;
    }

    /// Read a tryte (width 1) or the word centred on addr (width 3). A
    /// word's alignment is the caller's to check.
    pub fn read(&mut self, addr: i64, width: u32, access: Access) -> Result<i64, BusError> {
        match self.region(addr, access)? {
            Region::Console if addr == CONSOLE_IN => Ok(self.console.read()),
            Region::Harts => Ok(self.harts.read(addr, self.current)),
            Region::Irq => {
                let lines = self.lines();
                Ok(self.irq.read(addr, self.current, &lines))
            }
            Region::Block => Ok(self.block.read(addr)),
            Region::Fpu => Ok(self.fpu.read(addr, self.current)),
            Region::Net => Ok(self.net.read(addr)),
            Region::Console | Region::Power => Ok(0),
            Region::Boot | Region::Ram if width == 1 => Ok(self.get(addr)),
            Region::Boot | Region::Ram => Ok(self.word(addr)),
        }
    }

    pub fn write(&mut self, addr: i64, width: u32, value: i64) -> Result<(), BusError> {
        match self.region(addr, Access::Store)? {
            Region::Console => {
                if addr == CONSOLE_OUT && (0..=0x10FFFF).contains(&value) {
                    self.console.output.push(value as u32);
                }
            }
            Region::Power if addr == POWER_OFF => return Err(BusError::Halt(value)),
            Region::Harts => self.harts.write(addr, self.current, value),
            Region::Irq => self.irq.write(addr, value),
            Region::Block => {
                if let Request::Start { command, first, count, list } =
                    self.block.write(addr, value)
                {
                    self.start_transfer(command, first, count, list);
                }
            }
            Region::Fpu => self.fpu.write(addr, self.current, value),
            Region::Net => match self.net.write(addr, value) {
                net::Request::Send { addr, len } => self.start_send(addr, len),
                net::Request::Receive(buffer) => self.start_receive(buffer),
                net::Request::Nothing => {}
            },
            Region::Power | Region::Boot => {}
            Region::Ram if width == 1 => {
                self.break_reservations(addr, 1);
                self.poke(addr, value);
            }
            Region::Ram => {
                self.break_reservations(addr, 3);
                let lo = low(value, TRYTE_TRITS);
                let rest = (value - lo) / TRYTE_MOD;
                let mid = low(rest, TRYTE_TRITS);
                self.poke(addr - 1, lo);
                self.poke(addr, mid);
                self.poke(addr + 1, (rest - mid) / TRYTE_MOD);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_centred_low_tryte_first() {
        let mut bus = Bus::new(10);
        let value = 3 + (-7) * PAGE + 5 * PAGE * PAGE;
        bus.write(PAGE, 3, value).unwrap();
        let trytes: Vec<i64> =
            (PAGE - 1..=PAGE + 1).map(|a| bus.read(a, 1, Access::Load).unwrap()).collect();
        assert_eq!(trytes, vec![3, -7, 5]);
        assert_eq!(bus.read(PAGE, 3, Access::Load), Ok(value));
    }

    #[test]
    fn regions_follow_the_page_sign() {
        let mut bus = Bus::new(10);
        assert_eq!(bus.write(0, 3, 1), Err(BusError::Fault(0)));
        assert_eq!(bus.read(0, 3, Access::Load), Ok(0));
        assert_eq!(bus.read(11 * PAGE, 3, Access::Load), Err(BusError::Fault(11 * PAGE)));
        assert_eq!(bus.read(CONSOLE_OUT, 3, Access::Fetch), Err(BusError::Fault(CONSOLE_OUT)));
        assert_eq!(bus.read(-5 * PAGE, 3, Access::Load), Err(BusError::Fault(-5 * PAGE)));
        assert_eq!(bus.read(-3 * PAGE, 3, Access::Fetch), Err(BusError::Fault(-3 * PAGE)));
        assert_eq!(bus.write(POWER_OFF, 3, 7), Err(BusError::Halt(7)));
    }

    #[test]
    fn the_console_reads_and_writes_code_points() {
        let mut bus = Bus::new(1);
        bus.console.feed("Z");
        assert_eq!(bus.read(CONSOLE_IN, 3, Access::Load), Ok(90));
        assert_eq!(bus.read(CONSOLE_IN, 3, Access::Load), Ok(CONSOLE_EMPTY));
        bus.console.feed("a");
        bus.console.closed = true;
        assert_eq!(bus.read(CONSOLE_IN, 3, Access::Load), Ok(97), "input before the end");
        assert_eq!(bus.read(CONSOLE_IN, 3, Access::Load), Ok(CONSOLE_CLOSED));
        bus.write(CONSOLE_OUT, 3, 'é' as i64).unwrap();
        bus.write(CONSOLE_OUT, 3, -5).unwrap(); // not a code point: ignored
        assert_eq!(bus.console.text(), "é");
    }

    #[test]
    fn a_block_list_must_be_aligned_words_naming_ram_pages() {
        let mut bus = Bus::new(5);
        bus.write(PAGE, 3, 2).unwrap();
        bus.write(PAGE + 3, 3, 5).unwrap();
        assert_eq!(bus.page_list(PAGE, 2), Some(vec![2, 5]));
        assert_eq!(bus.page_list(PAGE + 3, 2), None, "the word after it is 0");
        // A misaligned list fails even where its trytes spell a RAM page.
        bus.poke(PAGE + 5, 3);
        bus.poke(PAGE + 6, 0);
        bus.poke(PAGE + 7, 0);
        assert_eq!(bus.word(PAGE + 6), 3);
        assert_eq!(bus.page_list(PAGE + 6, 1), Some(vec![3]));
        bus.poke(PAGE + 1, 4);
        bus.poke(PAGE + 2, 0);
        bus.poke(PAGE + 3, 0);
        assert_eq!(bus.word(PAGE + 2), 4);
        assert_eq!(bus.page_list(PAGE + 2, 1), None);
    }

    #[test]
    fn a_block_read_breaks_reservations_on_its_pages() {
        use crate::block::{MemoryDisk, block_addr};
        let mut bus = Bus::new(5);
        bus.block.disk = Some(Box::new(MemoryDisk::new(2)));
        bus.write(PAGE, 3, 3).unwrap(); // the list: page 3
        bus.current = 1;
        bus.reserve(3 * PAGE + 9000);
        bus.current = 2;
        bus.reserve(4 * PAGE);
        bus.current = 0;
        bus.write(block_addr(0), 3, PAGE).unwrap(); // LIST
        bus.write(block_addr(3), 3, BLOCK_READ).unwrap(); // COMMAND
        bus.now = 1100;
        bus.advance();
        assert!(bus.block.finished);
        assert!(!bus.holds_reservation(1), "its page was overwritten");
        assert!(bus.holds_reservation(2), "another page");
    }

    #[test]
    fn a_store_breaks_other_harts_reservations_only() {
        let mut bus = Bus::new(10);
        bus.current = 1;
        bus.reserve(PAGE);
        bus.current = 2;
        bus.reserve(PAGE + 3);
        bus.write(PAGE + 2, 1, 5).unwrap(); // hart 2's own word: kept
        bus.current = 1;
        assert!(bus.take_reservation(), "hart 2's store missed hart 1's word");
        bus.reserve(PAGE);
        bus.current = 2;
        bus.write(PAGE + 1, 1, 5).unwrap();
        assert!(bus.take_reservation());
        bus.current = 1;
        assert!(!bus.take_reservation());
    }
}
