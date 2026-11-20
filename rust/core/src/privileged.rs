//! The hart's privileged side: traps, interrupts, CSRs, `tret` and `wfi`
//! (docs/isa.md 8), ported from the original Python core.

use crate::hart::{Exception, Exec, Hart, Stop, trap};
use crate::isa::{self, IE, IP, TCAUSE, TIMECMP, TPC, TSTATUS, TVAL, TVEC};
use crate::mmu::Ptbr;
use crate::trits::{field, low, set_trit, wrap};

/// Interrupt priority when several are pending: external, software, timer.
const IRQ_PRIORITY: [u32; 3] = [2, 3, 1];

fn bank_index(ring: i64) -> usize {
    debug_assert!(ring == 0 || ring == 1, "only rings 0 and +1 have banks");
    ring as usize
}

impl Hart {
    // --- traps and interrupts ----------------------------------------------

    /// Enter a trap. `target` forces the bank (interrupts, single step);
    /// otherwise delegation decides. A trap never lands in a ring below
    /// the one it interrupts, and a bank whose tvec is 0 has no handler.
    pub(crate) fn enter_trap(
        &mut self,
        cause: i64,
        tval: i64,
        tpc: i64,
        target: Option<i64>,
    ) -> Result<(), Stop> {
        self.record(crate::hart::Event::Trap { cause, tval, tpc });
        let target = target.unwrap_or_else(|| {
            let delegated = cause > 0 && field(self.csrs[&isa::EDELEG], cause as u32, 1) == 1;
            if delegated { 0 } else { 1 }
        });
        let target = target.max(self.ring);
        let bank = &mut self.banks[bank_index(target)];
        let tvec = bank[TVEC];
        if tvec == 0 {
            return Err(Stop::Unhandled { cause, tval, pc: tpc, ring: self.ring });
        }
        bank[TPC] = tpc;
        bank[TCAUSE] = cause;
        bank[TVAL] = tval;
        let status = set_trit(bank[TSTATUS], isa::ST_PR, self.ring);
        let status = set_trit(status, isa::ST_PIE, field(status, isa::ST_IE, 1));
        bank[TSTATUS] = set_trit(status, isa::ST_IE, -1);
        self.ring = target;
        self.reservation = None;
        let mode = low(tvec, 1);
        let base = tvec - mode;
        self.pc = if mode == 1 { wrap(base + 3 * cause) } else { base };
        Ok(())
    }

    /// ip for a ring's bank, with the timer trit computed from time.
    fn pending(&self, ring: i64) -> i64 {
        let bank = &self.banks[bank_index(ring)];
        let timer = if self.time >= bank[TIMECMP] { 1 } else { 0 };
        set_trit(bank[IP], 1, timer)
    }

    fn interruptible(&self, ring: i64) -> bool {
        if self.ring < ring {
            return true;
        }
        let status = self.banks[bank_index(ring)][TSTATUS];
        self.ring == ring && field(status, isa::ST_IE, 1) == 1
    }

    /// Take the highest-priority enabled, pending interrupt, if any.
    pub(crate) fn take_interrupt(&mut self) -> Result<bool, Stop> {
        for ring in [1, 0] {
            let enabled = self.banks[bank_index(ring)][IE];
            if enabled == 0 || !self.interruptible(ring) {
                continue;
            }
            let pending = self.pending(ring);
            for i in IRQ_PRIORITY {
                if field(pending, i, 1) == 1 && field(enabled, i, 1) == 1 {
                    self.enter_trap(-(i as i64), 0, self.pc, Some(ring))?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // --- CSRs --------------------------------------------------------------

    fn bank_slot(number: i64) -> Option<(i64, usize)> {
        [1, 0].into_iter().find_map(|ring| {
            let i = number - isa::csr_number(ring, 1, 0);
            (0..isa::BANK.len() as i64).contains(&i).then_some((ring, i as usize))
        })
    }

    pub(crate) fn read_csr(&mut self, number: i64) -> Result<i64, Exception> {
        if self.ring < isa::csr_ring(number) {
            return Err(trap(isa::ILLEGAL, 0));
        }
        if let Some((ring, i)) = Self::bank_slot(number) {
            return Ok(if i == IP { self.pending(ring) } else { self.banks[bank_index(ring)][i] });
        }
        let computed = match number {
            isa::CYCLE | isa::INSTRET => Some(self.instret),
            isa::TIME => Some(self.time),
            isa::HARTID => Some(self.hartid),
            isa::ENTROPY => Some(self.entropy.next48()),
            _ => None,
        };
        computed
            .map(wrap)
            .or_else(|| self.csrs.get(&number).copied())
            .ok_or_else(|| trap(isa::ILLEGAL, 0))
    }

    pub(crate) fn write_csr(&mut self, number: i64, value: i64) -> Exec {
        if !isa::csr_writable(number) {
            return Err(trap(isa::ILLEGAL, 0));
        }
        let Some((ring, i)) = Self::bank_slot(number) else {
            if number == isa::PTBR {
                if low(value, 1) == -1 {
                    return Ok(()); // a reserved mode is not stored (9.2)
                }
                self.ptbr = Ptbr::decode(value);
            }
            self.csrs.insert(number, value);
            return Ok(());
        };
        let bank = &mut self.banks[bank_index(ring)];
        bank[i] = if i == IP {
            // Only the software-interrupt trit is writable; the timer trit
            // follows timecmp and the external trit follows devices.
            set_trit(bank[IP], 3, field(value, 3, 1))
        } else {
            value
        };
        Ok(())
    }

    /// A CSR by name, read with ring +1's privilege: for tests and tools.
    /// Reading `entropy` advances its stream, as a program's read would.
    pub fn csr(&mut self, name: &str) -> Option<i64> {
        let number = isa::csr_by_name(name)?;
        let ring = std::mem::replace(&mut self.ring, 1);
        let value = self.read_csr(number).ok();
        self.ring = ring;
        value
    }

    /// A CSR by number, read with ring +1's privilege; None if there is
    /// no such CSR.
    pub fn csr_by_number(&mut self, number: i64) -> Option<i64> {
        let ring = std::mem::replace(&mut self.ring, 1);
        let value = self.read_csr(number).ok();
        self.ring = ring;
        value
    }

    // --- privileged instructions -------------------------------------------

    pub(crate) fn privileged(&self) -> Exec {
        if self.ring < 0 { Err(trap(isa::ILLEGAL, 0)) } else { Ok(()) }
    }

    pub(crate) fn tret(&mut self, d: &isa::Decoded) -> Exec {
        self.privileged()?;
        let ring = self.ring;
        let bank = &mut self.banks[bank_index(ring)];
        let status = bank[TSTATUS];
        let back = field(status, isa::ST_PR, 1);
        if back > ring {
            return Err(trap(isa::ILLEGAL, d.word));
        }
        bank[TSTATUS] = set_trit(status, isa::ST_IE, field(status, isa::ST_PIE, 1));
        self.next_pc = bank[TPC];
        if field(status, isa::ST_SS, 1) == 1 {
            self.step_ring = Some(ring);
        }
        self.ring = back;
        Ok(())
    }

    /// Nothing but the timer can wake a lone hart, so skip ahead to the
    /// earliest enabled timer instead of spinning.
    pub(crate) fn wfi(&mut self) {
        let deadline = self
            .banks
            .iter()
            .filter(|b| field(b[IE], 1, 1) == 1 && b[TIMECMP] > self.time)
            .map(|b| b[TIMECMP])
            .min();
        if let Some(t) = deadline {
            self.time = t;
        }
    }
}
