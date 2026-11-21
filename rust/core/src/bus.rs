//! Physical memory and devices (docs/isa.md 5.3, docs/platform.md).
//!
//! The sign of a physical page number picks the region: page 0 is the
//! boot page (read-only to the CPU), positive pages up to `ram_pages`
//! are RAM, negative pages are devices. The platform's devices are built
//! in: the console at page −1, power control at page −2 and the hart
//! controller at pages −3 and −4.
//!
//! The bus also keeps each hart's `lr` reservation, so that a store by
//! one hart breaks another's.

use std::collections::{HashMap, VecDeque};

use crate::harts::{HartController, LOCAL_PAGE, SLOTS_PAGE};
use crate::trits::{TRYTE_MOD, TRYTE_TRITS, low, split_high};

pub const PAGE_TRITS: u32 = 9;
pub const PAGE: i64 = 19683;
const HALF_PAGE: i64 = PAGE / 2;

pub const CONSOLE_PAGE: i64 = -1;
pub const POWER_PAGE: i64 = -2;
pub const CONSOLE_OUT: i64 = CONSOLE_PAGE * PAGE;
pub const CONSOLE_IN: i64 = CONSOLE_OUT + 3;
pub const POWER_OFF: i64 = POWER_PAGE * PAGE;

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

/// Characters out, characters in: one code point per access.
#[derive(Default)]
pub struct Console {
    pub output: Vec<u32>,
    pub input: VecDeque<i64>,
}

impl Console {
    pub fn feed(&mut self, text: &str) {
        self.input.extend(text.chars().map(|c| c as i64));
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
            current: 0,
            reservations: HashMap::new(),
        }
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
            Region::Console if addr == CONSOLE_IN => {
                Ok(self.console.input.pop_front().unwrap_or(-1))
            }
            Region::Harts => Ok(self.harts.read(addr, self.current)),
            Region::Console | Region::Power => Ok(0),
            Region::Boot | Region::Ram if width == 1 => Ok(self.get(addr)),
            Region::Boot | Region::Ram => Ok(self.get(addr - 1)
                + self.get(addr) * TRYTE_MOD
                + self.get(addr + 1) * TRYTE_MOD * TRYTE_MOD),
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
        assert_eq!(bus.read(CONSOLE_IN, 3, Access::Load), Ok(-1));
        bus.write(CONSOLE_OUT, 3, 'é' as i64).unwrap();
        bus.write(CONSOLE_OUT, 3, -5).unwrap(); // not a code point: ignored
        assert_eq!(bus.console.text(), "é");
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
