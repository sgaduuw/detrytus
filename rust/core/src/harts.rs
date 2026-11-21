//! The hart controller (docs/platform.md, Harts): which harts exist and
//! run, their mailboxes, group sends and hot-plug events.
//!
//! The controller is a device on the bus. It only keeps state and
//! records requests; the machine starts and stops the harts themselves
//! after each step (`take_starts`, `take_stops`), so a start or a stop
//! takes effect once the instruction that asked for it has finished.

use std::collections::VecDeque;

use crate::bus::PAGE;
use crate::trits::{low, split_high};

pub const SLOTS_PAGE: i64 = -3;
pub const LOCAL_PAGE: i64 = -4;
/// The largest hart number; hart numbers run from −MAX_HART to MAX_HART.
pub const MAX_HART: i64 = 364;
pub const HARTS: i64 = 2 * MAX_HART + 1;
pub const MAILBOX_DEPTH: usize = 9;
/// SPAN 6 covers every slot: 3⁶ = 729.
pub const MAX_SPAN: i64 = 6;

const SLOT_TRITS: u32 = 3;
const SLOTS_CENTRE: i64 = SLOTS_PAGE * PAGE;
const LOCAL_CENTRE: i64 = LOCAL_PAGE * PAGE;

// Registers within a slot, as offsets from its centre.
const STATE: i64 = -12;
const START_PC: i64 = -9;
const START_ARG: i64 = -6;
const CONTROL: i64 = -3;
const SEND: i64 = 0;
const RECEIVE: i64 = 3;
const COUNT: i64 = 6;

// Registers of page −4, as offsets from its centre.
const TARGET: i64 = 27;
const SPAN: i64 = 30;
const GSEND: i64 = 33;
const LAST: i64 = 36;
const HARTS_REG: i64 = -27;
const EVENT_HART: i64 = -30;
const EVENT: i64 = -33;

/// A slot's STATE.
pub const ABSENT: i64 = -1;
pub const STOPPED: i64 = 0;
pub const RUNNING: i64 = 1;

/// What RECEIVE reads when there is no message: negative, since
/// messages never are.
pub const MAILBOX_EMPTY: i64 = -1;

/// Hot-plug event kinds, as EVENT reports them (`kind + 3·h`).
pub const ADDED: i64 = 1;
pub const REMOVAL_REQUESTED: i64 = -1;

/// The address of hart `h`'s slot register at `offset`: for tests.
pub fn slot_addr(h: i64, offset: i64) -> i64 {
    SLOTS_CENTRE + 27 * h + offset
}

#[derive(Clone, Default)]
struct Slot {
    state: i64,
    start_pc: i64,
    start_arg: i64,
    mailbox: VecDeque<i64>,
    target: i64,
    span: i64,
    last: i64,
    removal_requested: bool,
}

/// A request to start a hart, captured when CONTROL was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Start {
    pub hart: i64,
    pub pc: i64,
    pub arg: i64,
}

pub struct HartController {
    slots: Vec<Slot>,
    event_hart: i64,
    events: VecDeque<i64>,
    starts: Vec<Start>,
    stops: Vec<i64>,
    /// Running harts in number order, kept up to date so the machine
    /// need not scan all 729 slots every step.
    running: Vec<i64>,
}

fn index(h: i64) -> usize {
    (h + MAX_HART) as usize
}

pub fn valid(h: i64) -> bool {
    (-MAX_HART..=MAX_HART).contains(&h)
}

impl HartController {
    /// Harts in `present` exist; hart 0 runs and the rest are stopped.
    pub fn new(present: &[i64]) -> Self {
        let mut slots = vec![Slot { state: ABSENT, ..Slot::default() }; HARTS as usize];
        for &h in present {
            slots[index(h)].state = STOPPED;
        }
        slots[index(0)].state = RUNNING;
        HartController {
            slots,
            event_hart: 0,
            events: VecDeque::new(),
            starts: Vec::new(),
            stops: Vec::new(),
            running: vec![0],
        }
    }

    fn slot(&self, h: i64) -> &Slot {
        &self.slots[index(h)]
    }

    fn slot_mut(&mut self, h: i64) -> &mut Slot {
        &mut self.slots[index(h)]
    }

    pub fn state(&self, h: i64) -> i64 {
        if valid(h) { self.slot(h).state } else { ABSENT }
    }

    /// Hart numbers whose slot is running, in order.
    pub fn running(&self) -> &[i64] {
        &self.running
    }

    /// Whether hart `h`'s software interrupt is pending: its mailbox
    /// holds a message.
    pub fn doorbell(&self, h: i64) -> bool {
        !self.slot(h).mailbox.is_empty()
    }

    /// Whether hart `h`'s external interrupt is pending from hot-plug
    /// events.
    pub fn event_pending(&self, h: i64) -> bool {
        h == self.event_hart && !self.events.is_empty()
    }

    pub fn take_starts(&mut self) -> Vec<Start> {
        std::mem::take(&mut self.starts)
    }

    pub fn take_stops(&mut self) -> Vec<i64> {
        std::mem::take(&mut self.stops)
    }

    // --- hot-plug, driven by the platform -----------------------------------

    /// Plug in hart `h`: it arrives stopped. False if it is already there.
    pub fn add(&mut self, h: i64) -> bool {
        if self.slot(h).state != ABSENT {
            return false;
        }
        *self.slot_mut(h) = Slot { state: STOPPED, ..Slot::default() };
        self.events.push_back(ADDED + 3 * h);
        true
    }

    /// Ask for hart `h` to be removed. A stopped hart goes at once; a
    /// running one when it stops itself. False if it is absent.
    pub fn request_removal(&mut self, h: i64) -> bool {
        let slot = self.slot_mut(h);
        match slot.state {
            ABSENT => return false,
            STOPPED => slot.state = ABSENT,
            _ => slot.removal_requested = true,
        }
        self.events.push_back(REMOVAL_REQUESTED + 3 * h);
        true
    }

    // --- register access by a hart --------------------------------------------

    /// The slot and offset an address names, if it is in a slot: page −3
    /// directly, or the middle of page −4 for the accessing hart's own.
    fn slot_of(addr: i64, current: i64) -> Option<(i64, i64)> {
        let local = addr - LOCAL_CENTRE;
        if split_high(local, SLOT_TRITS) == 0 {
            return Some((current, local));
        }
        let slots = addr - SLOTS_CENTRE;
        let h = split_high(slots, SLOT_TRITS);
        (crate::bus::page_of(addr) == SLOTS_PAGE).then_some((h, low(slots, SLOT_TRITS)))
    }

    pub fn read(&mut self, addr: i64, current: i64) -> i64 {
        if let Some((h, offset)) = Self::slot_of(addr, current) {
            let slot = self.slot_mut(h);
            return match offset {
                STATE => slot.state,
                START_PC => slot.start_pc,
                START_ARG => slot.start_arg,
                RECEIVE if h == current => slot.mailbox.pop_front().unwrap_or(MAILBOX_EMPTY),
                RECEIVE => MAILBOX_EMPTY,
                COUNT => slot.mailbox.len() as i64,
                _ => 0,
            };
        }
        let current = self.slot(current);
        match addr - LOCAL_CENTRE {
            TARGET => current.target,
            SPAN => current.span,
            LAST => current.last,
            HARTS_REG => HARTS,
            EVENT_HART => self.event_hart,
            EVENT => self.events.pop_front().unwrap_or(0),
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: i64, current: i64, value: i64) {
        if let Some((h, offset)) = Self::slot_of(addr, current) {
            match offset {
                START_PC => self.slot_mut(h).start_pc = value,
                START_ARG => self.slot_mut(h).start_arg = value,
                CONTROL => self.control(h, current, value),
                SEND => {
                    let ok = self.deliver(h, value);
                    self.slot_mut(current).last = if ok { 1 } else { -1 };
                }
                _ => {}
            }
            return;
        }
        match addr - LOCAL_CENTRE {
            TARGET => self.slot_mut(current).target = value,
            SPAN => self.slot_mut(current).span = value.clamp(0, MAX_SPAN),
            GSEND => self.group_send(current, value),
            EVENT_HART => self.event_hart = value,
            _ => {}
        }
    }

    fn control(&mut self, h: i64, current: i64, value: i64) {
        let slot = self.slot_mut(h);
        match value {
            1 if slot.state == STOPPED => {
                slot.state = RUNNING;
                let start = Start { hart: h, pc: slot.start_pc, arg: slot.start_arg };
                self.starts.push(start);
                let at = self.running.partition_point(|&r| r < h);
                self.running.insert(at, h);
            }
            -1 if h == current && slot.state == RUNNING => {
                slot.mailbox.clear();
                slot.state = if slot.removal_requested { ABSENT } else { STOPPED };
                slot.removal_requested = false;
                self.stops.push(h);
                self.running.retain(|&r| r != h);
            }
            _ => {}
        }
    }

    /// Queue a word for hart `h`: false if it is not running, its
    /// mailbox is full, or the word is negative.
    fn deliver(&mut self, h: i64, value: i64) -> bool {
        if !valid(h) || value < 0 {
            return false;
        }
        let slot = self.slot_mut(h);
        if slot.state != RUNNING || slot.mailbox.len() >= MAILBOX_DEPTH {
            return false;
        }
        slot.mailbox.push_back(value);
        true
    }

    /// Send to every running hart that agrees with TARGET above its
    /// lowest SPAN trits, except the sender.
    fn group_send(&mut self, current: i64, value: i64) {
        let Slot { target, span, .. } = *self.slot(current);
        let span = span as u32;
        let group = split_high(target, span);
        // A negative word reaches nobody, even in an empty group.
        let mut ok = value >= 0;
        for h in -MAX_HART..=MAX_HART {
            let member = h != current && split_high(h, span) == group;
            if ok && member && self.slot(h).state == RUNNING {
                ok &= self.deliver(h, value);
            }
        }
        self.slot_mut(current).last = if ok { 1 } else { -1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL: i64 = LOCAL_CENTRE;

    #[test]
    fn slots_tile_page_minus_3() {
        assert_eq!(slot_addr(-MAX_HART, STATE), SLOTS_CENTRE - 9841 + 1);
        assert_eq!(HartController::slot_of(slot_addr(5, COUNT), 0), Some((5, COUNT)));
        assert_eq!(HartController::slot_of(slot_addr(-364, -13), 0), Some((-364, -13)));
        assert_eq!(HartController::slot_of(LOCAL + SEND, 7), Some((7, SEND)));
        assert_eq!(HartController::slot_of(LOCAL + TARGET, 7), None);
    }

    #[test]
    fn mailboxes_queue_nine_words_and_report_refusals() {
        let mut c = HartController::new(&[0, 1]);
        c.write(slot_addr(1, CONTROL), 0, 1);
        assert_eq!(c.take_starts(), vec![Start { hart: 1, pc: 0, arg: 0 }]);
        for i in 0..9 {
            c.write(slot_addr(1, SEND), 0, 10 + i);
            assert_eq!(c.read(LOCAL + LAST, 0), 1);
        }
        c.write(slot_addr(1, SEND), 0, 99);
        assert_eq!(c.read(LOCAL + LAST, 0), -1);
        assert_eq!(c.read(slot_addr(1, COUNT), 0), 9);
        let others = c.read(slot_addr(1, RECEIVE), 0);
        assert_eq!(others, MAILBOX_EMPTY, "only the owner receives");
        assert_eq!(c.read(LOCAL + RECEIVE, 1), 10);
        assert!(c.doorbell(1));
        for _ in 0..8 {
            c.read(LOCAL + RECEIVE, 1);
        }
        assert_eq!(c.read(LOCAL + RECEIVE, 1), MAILBOX_EMPTY);
        c.write(slot_addr(1, SEND), 0, 0);
        assert_eq!(c.read(LOCAL + LAST, 0), 1, "0 is a message");
        c.write(slot_addr(1, SEND), 0, -1);
        assert_eq!(c.read(LOCAL + LAST, 0), -1, "a negative word is not");
        assert_eq!(c.read(LOCAL + RECEIVE, 1), 0);
        assert_eq!(c.read(LOCAL + RECEIVE, 1), MAILBOX_EMPTY);
        assert!(!c.doorbell(1));
        c.write(slot_addr(-1, SEND), 0, 5); // absent
        assert_eq!(c.read(LOCAL + LAST, 0), -1);
    }

    #[test]
    fn group_sends_reach_running_harts_in_the_subtree() {
        let present: Vec<i64> = (-13..=13).collect();
        let mut c = HartController::new(&present);
        for h in [-1, 1, 2, 4] {
            c.write(slot_addr(h, CONTROL), 0, 1);
        }
        c.write(LOCAL + TARGET, 0, 0);
        c.write(LOCAL + SPAN, 0, 1); // the middle triad: −1, 0, +1
        c.write(LOCAL + GSEND, 0, 7);
        assert!(c.doorbell(-1) && c.doorbell(1) && !c.doorbell(0) && !c.doorbell(2));
        c.write(LOCAL + SPAN, 0, 9); // clamped to every hart
        assert_eq!(c.read(LOCAL + SPAN, 0), MAX_SPAN);
        c.write(LOCAL + GSEND, 0, 8);
        assert!(c.doorbell(2) && c.doorbell(4) && !c.doorbell(3));
        assert_eq!(c.read(LOCAL + LAST, 0), 1);
        c.write(LOCAL + GSEND, 0, -8);
        assert_eq!(c.read(LOCAL + LAST, 0), -1, "a negative word is refused");
        assert_eq!(c.read(slot_addr(2, COUNT), 0), 1, "and reaches nobody");
    }

    #[test]
    fn stopping_is_for_the_hart_itself_and_removal_waits_for_it() {
        let mut c = HartController::new(&[0, 1]);
        c.write(slot_addr(1, CONTROL), 0, 1);
        c.write(slot_addr(1, CONTROL), 0, -1);
        assert_eq!(c.state(1), RUNNING, "another hart cannot stop it");
        assert!(c.request_removal(1));
        assert_eq!(c.read(LOCAL + EVENT, 0), REMOVAL_REQUESTED + 3);
        c.write(LOCAL + CONTROL, 1, -1);
        assert_eq!(c.state(1), ABSENT);
        assert_eq!(c.take_stops(), vec![1]);
        assert!(c.add(1));
        assert!(c.event_pending(0));
        assert_eq!(c.read(LOCAL + EVENT, 0), ADDED + 3);
        assert_eq!(c.state(1), STOPPED);
    }
}
