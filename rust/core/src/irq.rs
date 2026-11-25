//! The interrupt controller (docs/platform.md, Device interrupts): a
//! pared-down PLIC that gathers device interrupts and raises the
//! external interrupt, cause −2, on the hart each source is routed to.
//!
//! The controller keeps only what software sets (ENABLE, ROUTE) and
//! which sources are in service. Whether a source is pending is the
//! device's to say, so the bus passes the pending lines in on every
//! access that needs them: sources are level-triggered.

use crate::bus::PAGE;

pub const IRQ_PAGE: i64 = -13;
const CENTRE: i64 = IRQ_PAGE * PAGE;

/// The highest source number on this platform.
pub const SOURCES: i64 = 3;
pub const CONSOLE_SOURCE: i64 = 1;
pub const BLOCK_SOURCE: i64 = 2;
pub const NET_SOURCE: i64 = 3;

// Group 0's registers, as offsets from the page's centre.
const CLAIM: i64 = -3;
const COMPLETE: i64 = 0;
const SOURCES_REG: i64 = 3;

// A source's registers, as offsets from the centre of its group.
const PENDING: i64 = -3;
const ENABLE: i64 = 0;
const ROUTE: i64 = 3;

/// The address of source `s`'s register at `offset` (group 0 for the
/// per-hart registers): for tests.
pub fn irq_addr(s: i64, offset: i64) -> i64 {
    CENTRE + 9 * s + offset
}

/// Which sources want attention, indexed by source number (index 0 is
/// unused): the devices' lines at this moment.
pub type Lines = [bool; SOURCES as usize + 1];

#[derive(Default)]
pub struct InterruptController {
    enable: Lines,
    route: [i64; SOURCES as usize + 1],
    in_service: Lines,
}

/// The source and register offset an address names, if it is a source's.
fn source_of(addr: i64) -> Option<(i64, i64)> {
    let rel = addr - CENTRE;
    let s = (rel + 4).div_euclid(9);
    let offset = rel - 9 * s;
    (s >= 1 && matches!(offset, PENDING | ENABLE | ROUTE)).then_some((s, offset))
}

impl InterruptController {
    fn claimable(&self, s: usize, h: i64, lines: &Lines) -> bool {
        lines[s] && self.enable[s] && self.route[s] == h && !self.in_service[s]
    }

    /// Whether hart `h` has a source to claim: its external interrupt,
    /// from devices.
    pub fn pending_for(&self, h: i64, lines: &Lines) -> bool {
        (1..=SOURCES as usize).any(|s| self.claimable(s, h, lines))
    }

    pub fn read(&mut self, addr: i64, current: i64, lines: &Lines) -> i64 {
        if let Some((s, offset)) = source_of(addr) {
            if s > SOURCES {
                return 0;
            }
            let s = s as usize;
            return match offset {
                PENDING => lines[s] as i64,
                ENABLE => self.enable[s] as i64,
                _ => self.route[s],
            };
        }
        match addr - CENTRE {
            CLAIM => match (1..=SOURCES as usize).find(|&s| self.claimable(s, current, lines)) {
                Some(s) => {
                    self.in_service[s] = true;
                    s as i64
                }
                None => 0,
            },
            SOURCES_REG => SOURCES,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: i64, value: i64) {
        if let Some((s, offset)) = source_of(addr) {
            if s > SOURCES {
                return;
            }
            let s = s as usize;
            match offset {
                ENABLE => self.enable[s] = value == 1,
                ROUTE => self.route[s] = value,
                _ => {}
            }
            return;
        }
        if addr - CENTRE == COMPLETE && (1..=SOURCES).contains(&value) {
            self.in_service[value as usize] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Lines = [false; 4];
    const CONSOLE: Lines = [false, true, false, false];
    const BOTH: Lines = [false, true, true, false];

    #[test]
    fn groups_tile_the_page_from_its_centre() {
        assert_eq!(source_of(irq_addr(1, PENDING)), Some((1, PENDING)));
        assert_eq!(source_of(irq_addr(2, ROUTE)), Some((2, ROUTE)));
        assert_eq!(source_of(irq_addr(0, CLAIM)), None);
        assert_eq!(source_of(irq_addr(1, 1)), None);
        assert_eq!(irq_addr(0, 0), -255879);
    }

    #[test]
    fn a_source_interrupts_its_hart_once_enabled_and_until_claimed() {
        let mut c = InterruptController::default();
        assert!(!c.pending_for(0, &CONSOLE), "masked at reset");
        c.write(irq_addr(1, ENABLE), 1);
        assert!(c.pending_for(0, &CONSOLE));
        assert!(!c.pending_for(0, &NONE));
        c.write(irq_addr(1, ROUTE), 1);
        assert!(!c.pending_for(0, &CONSOLE) && c.pending_for(1, &CONSOLE));
        assert_eq!(c.read(irq_addr(0, CLAIM), 0, &CONSOLE), 0, "not routed to hart 0");
        assert_eq!(c.read(irq_addr(0, CLAIM), 1, &CONSOLE), 1);
        assert!(!c.pending_for(1, &CONSOLE), "in service");
        assert_eq!(c.read(irq_addr(0, CLAIM), 1, &CONSOLE), 0);
        c.write(irq_addr(0, COMPLETE), 1);
        assert!(c.pending_for(1, &CONSOLE), "still pending, so claimable again");
    }

    #[test]
    fn claim_takes_the_lowest_source_first() {
        let mut c = InterruptController::default();
        for s in 1..=2 {
            c.write(irq_addr(s, ENABLE), 1);
        }
        assert_eq!(c.read(irq_addr(0, CLAIM), 0, &BOTH), 1);
        assert_eq!(c.read(irq_addr(0, CLAIM), 0, &BOTH), 2);
        assert_eq!(c.read(irq_addr(0, CLAIM), 0, &BOTH), 0);
    }

    #[test]
    fn registers_read_back_and_missing_sources_read_zero() {
        let mut c = InterruptController::default();
        c.write(irq_addr(2, ENABLE), 5); // not +1: stored as 0
        c.write(irq_addr(2, ROUTE), -4);
        c.write(irq_addr(4, ENABLE), 1);
        assert_eq!(c.read(irq_addr(2, ENABLE), 0, &NONE), 0);
        assert_eq!(c.read(irq_addr(2, ROUTE), 0, &NONE), -4);
        assert_eq!(c.read(irq_addr(1, PENDING), 0, &CONSOLE), 1);
        assert_eq!(c.read(irq_addr(4, ENABLE), 0, &NONE), 0);
        assert_eq!(c.read(irq_addr(0, SOURCES_REG), 0, &NONE), 3);
    }
}
