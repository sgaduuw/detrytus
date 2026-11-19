//! Physical memory and devices (docs/isa.md 5.3, docs/platform.md).
//!
//! The sign of a physical page number picks the region: page 0 is the
//! boot page (read-only to the CPU), positive pages up to `ram_pages`
//! are RAM, negative pages are devices. The platform's two devices are
//! built in: the console at page −1 and power control at page −2.

use std::collections::{HashMap, VecDeque};

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
}

impl Bus {
    pub fn new(ram_pages: i64) -> Self {
        Bus { ram_pages, pages: HashMap::new(), console: Console::default() }
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
            _ => Err(BusError::Fault(addr)),
        }
    }

    fn get(&self, addr: i64) -> i64 {
        let page = page_of(addr);
        self.pages.get(&page).map_or(0, |p| p[(addr - page * PAGE + HALF_PAGE) as usize] as i64)
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
            Region::Power | Region::Boot => {}
            Region::Ram if width == 1 => self.poke(addr, value),
            Region::Ram => {
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
        assert_eq!(bus.read(-3 * PAGE, 3, Access::Load), Err(BusError::Fault(-3 * PAGE)));
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
}
