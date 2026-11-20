//! One hart: registers, the fetch loop and what each instruction does.
//! Traps, CSRs and interrupts are in `privileged.rs`.
//!
//! Every integer, logic, memory, atomic, control and system instruction
//! (docs/isa.md 7), ported from the original Python core. Not yet:
//! floating point and compressed bundles, which stop the machine as
//! [`Stop::Unsupported`] rather than being approximated.
//!
//! The machine steps harts one instruction at a time with no store
//! buffering, so memory is sequentially consistent and fences are
//! no-ops. The machine also keeps the clock and decides when a hart
//! waiting in `wfi` runs again; an `lr` reservation is lost to a trap,
//! a later `sc`, or another hart's store (kept by the bus).

use std::collections::HashMap;

use crate::alu;
use crate::bus::{Access, Bus, BusError};
use crate::entropy::Entropy;
use crate::isa::{self, Decoded, Kind};
use crate::mmu::{self, Mmu, Ptbr};
use crate::trits::{TRYTE_TRITS, WORD_TRITS, field, low, pow3, wrap};

/// Why the hart stopped running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The program powered off with this exit code.
    Halt(i64),
    /// A trap reached a ring with no handler (`tvec` = 0).
    Unhandled { cause: i64, tval: i64, pc: i64, ring: i64 },
    /// Something this core does not implement yet.
    Unsupported(String),
}

/// What interrupts an instruction.
pub(crate) enum Exception {
    Trap { cause: i64, tval: i64 },
    Stop(Stop),
}

pub(crate) type Exec = Result<(), Exception>;

pub(crate) fn trap(cause: i64, tval: i64) -> Exception {
    Exception::Trap { cause, tval }
}

/// Something neither core implements yet; the message is the Python
/// core's, word for word.
fn unsupported(message: &str) -> Exception {
    Exception::Stop(Stop::Unsupported(message.to_string()))
}

/// What a data access did, for a debugger's watchpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessKind {
    Read,
    Write,
    /// An atomic read-modify-write.
    Update,
}

/// Something that happened during a step, recorded when
/// [`Hart::events`] is on: what a debugger needs to stop at
/// watchpoints and caught traps, and to trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// An instruction was fetched and decoded at pc, about to run.
    Execute { pc: i64, word: i64 },
    /// A completed data access, at a virtual address.
    Access { addr: i64, width: u32, kind: AccessKind },
    /// A trap was entered (or found no handler).
    Trap { cause: i64, tval: i64, tpc: i64 },
}

pub struct Hart {
    regs: [i64; 27],
    pub pc: i64,
    pub(crate) next_pc: i64,
    pub ring: i64,
    pub instret: i64,
    /// The shared clock, as the machine last set it.
    pub time: i64,
    pub hartid: i64,
    /// Set by `wfi`: the machine runs the hart again once an interrupt
    /// is pending.
    pub(crate) waiting: bool,
    /// Interrupt lines from the hart controller, set by the machine:
    /// software (the mailbox) and external (hot-plug events).
    pub(crate) doorbell: bool,
    pub(crate) external: bool,
    /// The trap banks of rings 0 and +1, indexed by ring.
    pub(crate) banks: [[i64; 9]; 2],
    /// CSRs that are plain storage.
    pub(crate) csrs: HashMap<i64, i64>,
    pub(crate) ptbr: Ptbr,
    pub(crate) mmu: Mmu,
    pub(crate) reservation: Option<i64>,
    /// The bank to trap into after one more instruction (single step).
    pub(crate) step_ring: Option<i64>,
    pub(crate) entropy: Entropy,
    cache: HashMap<i64, Result<Decoded, isa::Illegal>>,
    /// When Some, every [`Event`] is appended here; a debugger drains it
    /// after each step. None (the default) costs nothing.
    pub events: Option<Vec<Event>>,
}

impl Hart {
    /// Hart 0 at reset: ring +1, pc 0. `seed` seeds the entropy CSR.
    pub fn new(seed: u32) -> Self {
        Self::start(seed, 0, 0, 0)
    }

    /// Hart `hartid` in its reset state, starting at `pc` with `arg` in
    /// a0 (docs/isa.md 5.4, 5.6).
    pub fn start(seed: u32, hartid: i64, pc: i64, arg: i64) -> Self {
        let mut regs = [0; 27];
        regs[(isa::reg_number("a0").unwrap() + 13) as usize] = arg;
        Hart {
            regs,
            pc,
            next_pc: pc,
            ring: 1,
            instret: 0,
            time: 0,
            hartid,
            waiting: false,
            doorbell: false,
            external: false,
            banks: [[0; 9]; 2],
            csrs: isa::stored_csrs().map(|n| (n, 0)).collect(),
            ptbr: Ptbr::decode(0),
            mmu: Mmu::default(),
            reservation: None,
            step_ring: None,
            entropy: Entropy::new(seed),
            cache: HashMap::new(),
            events: None,
        }
    }

    // --- registers ---------------------------------------------------------

    pub fn x(&self, r: i64) -> i64 {
        self.regs[(r + 13) as usize]
    }

    fn set_x(&mut self, r: i64, value: i64) {
        if r != 0 {
            self.regs[(r + 13) as usize] = value;
        }
    }

    /// A register by any of its names: `a0`, `r1`, `sp`, ...
    pub fn reg(&self, name: &str) -> Option<i64> {
        isa::reg_number(name).map(|r| self.x(r))
    }

    // --- state, for tools and for comparing cores ------------------------------

    /// Registers r-13 to r13, in order.
    pub fn regs(&self) -> [i64; 27] {
        self.regs
    }

    /// A trap bank (ring 0 or +1), in the order of [`isa::BANK`]. The
    /// stored ip, without the timer trit computed from time.
    pub fn bank(&self, ring: i64) -> [i64; 9] {
        self.banks[ring as usize]
    }

    /// The CSRs held as plain storage, by number, in number order.
    pub fn stored_csrs(&self) -> Vec<(i64, i64)> {
        let mut csrs: Vec<(i64, i64)> = self.csrs.iter().map(|(&n, &v)| (n, v)).collect();
        csrs.sort_unstable();
        csrs
    }

    pub fn reservation(&self) -> Option<i64> {
        self.reservation
    }

    pub fn step_ring(&self) -> Option<i64> {
        self.step_ring
    }

    // --- the fetch loop ----------------------------------------------------

    /// Run one instruction, or take an interrupt, at the time the
    /// machine has set.
    pub fn step(&mut self, bus: &mut Bus) -> Result<(), Stop> {
        let stepping = self.step_ring.take();
        if self.take_interrupt()? {
            return Ok(());
        }
        let pc = self.pc;
        match self.execute(bus, pc) {
            Ok(()) => {
                self.instret += 1;
                match stepping {
                    Some(ring) => self.enter_trap(isa::SINGLE_STEP, self.pc, self.pc, Some(ring)),
                    None => Ok(()),
                }
            }
            Err(Exception::Trap { cause, tval }) => self.enter_trap(cause, tval, pc, None),
            Err(Exception::Stop(stop)) => Err(stop),
        }
    }

    fn execute(&mut self, bus: &mut Bus, pc: i64) -> Exec {
        if low(pc, 1) != 0 {
            return Err(unsupported("compressed bundles (slot B) are not implemented"));
        }
        let physical = self.translate(bus, pc, mmu::Access::Fetch)?;
        let word = bus.read(physical, 3, Access::Fetch).map_err(|e| match e {
            BusError::Fault(addr) => trap(isa::ACCESS_FAULT, addr),
            BusError::Halt(code) => Exception::Stop(Stop::Halt(code)),
        })?;
        let d = self.decode(word)?;
        self.record(Event::Execute { pc, word });
        self.next_pc = wrap(pc + 3);
        self.run(bus, &d)?;
        self.pc = self.next_pc;
        Ok(())
    }

    fn decode(&mut self, word: i64) -> Result<Decoded, Exception> {
        if low(word, 5) == isa::BUNDLE_OPCODE {
            return Err(unsupported("compressed bundles are not implemented"));
        }
        match self.cache.entry(word).or_insert_with(|| isa::decode(word)) {
            Ok(d) => Ok(*d),
            Err(_) => Err(trap(isa::ILLEGAL, word)),
        }
    }

    // --- memory ------------------------------------------------------------

    /// Virtual to physical. Ring +1 is never translated (docs/isa.md 8.1).
    fn translate(
        &mut self,
        bus: &mut Bus,
        addr: i64,
        access: mmu::Access,
    ) -> Result<i64, Exception> {
        if self.ring > 0 || self.ptbr.mode != 1 {
            return Ok(addr);
        }
        let sua = field(self.banks[0][isa::TSTATUS], isa::ST_SUA, 1) == 1;
        let (ring, ptbr) = (self.ring, self.ptbr);
        self.mmu.translate(bus, addr, access, ring, sua, ptbr).map_err(|(c, v)| trap(c, v))
    }

    fn aligned(addr: i64, width: u32) -> Exec {
        if width == 3 && low(addr, 1) != 0 {
            return Err(trap(isa::MISALIGNED, addr));
        }
        Ok(())
    }

    fn bus_error(addr: i64) -> impl Fn(BusError) -> Exception {
        move |e| match e {
            BusError::Fault(_) => trap(isa::ACCESS_FAULT, addr),
            BusError::Halt(code) => Exception::Stop(Stop::Halt(code)),
        }
    }

    pub(crate) fn record(&mut self, event: Event) {
        if let Some(events) = &mut self.events {
            events.push(event);
        }
    }

    fn load(&mut self, bus: &mut Bus, addr: i64, width: u32) -> Result<i64, Exception> {
        Ok(self.load_at(bus, addr, width)?.0)
    }

    /// A load's value and the physical address it read.
    fn load_at(&mut self, bus: &mut Bus, addr: i64, width: u32) -> Result<(i64, i64), Exception> {
        Self::aligned(addr, width)?;
        let physical = self.translate(bus, addr, mmu::Access::Load)?;
        let value = bus.read(physical, width, Access::Load).map_err(Self::bus_error(addr))?;
        self.record(Event::Access { addr, width, kind: AccessKind::Read });
        Ok((value, physical))
    }

    fn store(&mut self, bus: &mut Bus, addr: i64, width: u32, value: i64) -> Exec {
        Self::aligned(addr, width)?;
        let physical = self.translate(bus, addr, mmu::Access::Store)?;
        bus.write(physical, width, value).map_err(Self::bus_error(addr))?;
        self.record(Event::Access { addr, width, kind: AccessKind::Write });
        Ok(())
    }

    /// Where addr points for the current ring, without side effects: no
    /// permission check, nothing cached, None where nothing is mapped.
    /// For debuggers.
    pub fn probe(&self, bus: &mut Bus, addr: i64) -> Option<i64> {
        if self.ring > 0 || self.ptbr.mode != 1 {
            return Some(addr);
        }
        mmu::probe(bus, addr, self.ptbr)
    }

    /// Atomic read-modify-write of a word; returns the old value. One
    /// translation covers both halves and needs read and write
    /// permission, so a read-only page faults as a store would.
    fn rmw(
        &mut self,
        bus: &mut Bus,
        addr: i64,
        update: impl Fn(i64) -> i64,
    ) -> Result<i64, Exception> {
        Self::aligned(addr, 3)?;
        let physical = self.translate(bus, addr, mmu::Access::Amo)?;
        let old = bus.read(physical, 3, Access::Load).map_err(Self::bus_error(addr))?;
        bus.write(physical, 3, update(old)).map_err(Self::bus_error(addr))?;
        self.record(Event::Access { addr, width: 3, kind: AccessKind::Update });
        Ok(old)
    }

    fn amo_addr(&self, d: &Decoded) -> Result<i64, Exception> {
        let addr = self.x(d.rs1);
        if low(addr, 1) != 0 {
            return Err(trap(isa::MISALIGNED, addr));
        }
        Ok(addr)
    }

    // --- instructions ------------------------------------------------------

    fn run(&mut self, bus: &mut Bus, d: &Decoded) -> Exec {
        let (a, b) = (self.x(d.rs1), self.x(d.rs2));
        if let Some(value) = Self::arithmetic(d, a, b) {
            self.set_x(d.rd, value);
            return Ok(());
        }
        match d.op.kind {
            Kind::Sel => {
                let value = match a.signum() {
                    -1 => b,
                    1 => self.x(d.rs3),
                    _ => 0,
                };
                self.set_x(d.rd, value);
            }
            Kind::Tex => {
                let (pos, len) = Self::field_args(d)?;
                self.set_x(d.rd, field(a, pos, len));
            }
            Kind::Tins => {
                let (pos, len) = Self::field_args(d)?;
                self.set_x(d.rd, alu::trit_insert(self.x(d.rd), a, pos, len));
            }
            Kind::Auipc => self.set_x(d.rd, wrap(self.pc + d.imm * pow3(8))),
            _ => return self.memory_or_control(bus, d, a, b),
        }
        Ok(())
    }

    /// rd from rs1, rs2 and the immediate alone, for the instructions
    /// that are nothing more.
    fn arithmetic(d: &Decoded, a: i64, b: i64) -> Option<i64> {
        Some(match d.op.kind {
            Kind::Add => wrap(a + b),
            Kind::Sub => wrap(a - b),
            Kind::Mul => alu::mul_parts(a, b).1,
            Kind::Mulh => alu::mul_parts(a, b).0,
            Kind::Div => alu::div_trunc(a, b).0,
            Kind::Rem => alu::div_trunc(a, b).1,
            Kind::Divr => alu::div_round(a, b).0,
            Kind::Remr => alu::div_round(a, b).1,
            Kind::Cmp => (a - b).signum(),
            Kind::Min => a.min(b),
            Kind::Max => a.max(b),
            Kind::Abs => a.abs(),
            Kind::Sh => alu::shift(a, b),
            Kind::Rot => alu::rotate(a, b),
            Kind::Shi => alu::shift(a, d.imm),
            Kind::Roti => alu::rotate(a, d.imm),
            Kind::Lzt => alu::leading_zero_trits(a),
            Kind::Tzt => alu::trailing_zero_trits(a),
            Kind::Nzt => alu::nonzero_trits(a),
            Kind::Tlut => alu::tlut(a, b, d.fn_),
            Kind::Band => wrap(a & b),
            Kind::Bor => wrap(a | b),
            Kind::Bxor => wrap(a ^ b),
            Kind::Bsh => alu::binary_shift(a, b),
            Kind::Addi => wrap(a + d.imm),
            Kind::Cmpi => (a - d.imm).signum(),
            Kind::Lui => d.imm * pow3(8),
            _ => return None,
        })
    }

    fn field_args(d: &Decoded) -> Result<(u32, u32), Exception> {
        let (pos, len) = (low(d.imm, 4), field(d.imm, 4, 4));
        if pos < 0 || len < 1 || pos + len > WORD_TRITS as i64 {
            return Err(trap(isa::ILLEGAL, d.word));
        }
        Ok((pos as u32, len as u32))
    }

    fn memory_or_control(&mut self, bus: &mut Bus, d: &Decoded, a: i64, b: i64) -> Exec {
        let addr = wrap(a + d.imm);
        match d.op.kind {
            Kind::Lt => {
                let v = self.load(bus, addr, 1)?;
                self.set_x(d.rd, v);
            }
            Kind::Lw => {
                let v = self.load(bus, addr, 3)?;
                self.set_x(d.rd, v);
            }
            Kind::St => self.store(bus, addr, 1, low(b, TRYTE_TRITS))?,
            Kind::Sw => self.store(bus, addr, 3, b)?,
            Kind::Jal => {
                self.set_x(d.rd, self.next_pc);
                self.next_pc = wrap(self.pc + 3 * d.imm);
            }
            Kind::Jalr => {
                if low(addr, 1) == -1 {
                    return Err(trap(isa::MISALIGNED, addr));
                }
                self.set_x(d.rd, self.next_pc);
                self.next_pc = addr;
            }
            Kind::B => {
                if isa::cond_taken(d.cond, (a - b).signum()) == Some(true) {
                    self.next_pc = wrap(self.pc + 3 * d.off);
                }
            }
            Kind::Br3 => match a.signum() {
                -1 => self.next_pc = wrap(self.pc + 3 * d.offn),
                1 => self.next_pc = wrap(self.pc + 3 * d.offp),
                _ => {}
            },
            _ => return self.atomic(bus, d, b),
        }
        Ok(())
    }

    fn atomic(&mut self, bus: &mut Bus, d: &Decoded, src: i64) -> Exec {
        let update: fn(i64, i64, i64) -> i64 = match d.op.kind {
            Kind::Amoswap => |_, src, _| src,
            Kind::Amoadd => |old, src, _| wrap(old + src),
            Kind::Amomin => |old, src, _| old.min(src),
            Kind::Amomax => |old, src, _| old.max(src),
            Kind::Amotlut => |old, src, table| alu::tlut(old, src, table),
            _ => return self.reserved_or_system(bus, d, src),
        };
        let addr = self.amo_addr(d)?;
        let table = d.fn_;
        let old = self.rmw(bus, addr, |old| update(old, src, table))?;
        self.set_x(d.rd, old);
        Ok(())
    }

    fn reserved_or_system(&mut self, bus: &mut Bus, d: &Decoded, src: i64) -> Exec {
        match d.op.kind {
            Kind::Lr => {
                let addr = self.amo_addr(d)?;
                let (v, physical) = self.load_at(bus, addr, 3)?;
                self.set_x(d.rd, v);
                self.reservation = Some(addr);
                bus.reserve(physical);
            }
            Kind::Sc => {
                let addr = self.amo_addr(d)?;
                let held = bus.take_reservation();
                let ok = self.reservation == Some(addr) && held;
                self.reservation = None;
                if ok {
                    self.store(bus, addr, 3, src)?;
                }
                self.set_x(d.rd, if ok { 0 } else { 1 });
            }
            Kind::Cas => {
                let expected = self.x(d.rd);
                let addr = self.amo_addr(d)?;
                // A failed compare writes the old value back: it still
                // needs write permission, and leaves memory unchanged.
                let old = self.rmw(bus, addr, |old| if old == expected { src } else { old })?;
                self.set_x(d.rd, old);
            }
            _ => return self.system(d),
        }
        Ok(())
    }

    fn system(&mut self, d: &Decoded) -> Exec {
        match d.op.kind {
            Kind::Sys => Err(trap(isa::SYSCALL, 0)),
            Kind::Brk => Err(trap(isa::BREAKPOINT, self.pc)),
            // Harts run one instruction at a time with no caches, so these
            // hold already: nothing is reordered, and the decode cache is
            // keyed by word value.
            Kind::Fence | Kind::FenceI | Kind::Cclean | Kind::Cflush => Ok(()),
            Kind::Cinval => self.privileged(),
            Kind::Tret => self.tret(d),
            Kind::FenceVm => {
                self.privileged()?;
                let va = (d.rs1 != 0).then(|| self.x(d.rs1));
                let asid = (d.rs2 != 0).then(|| self.x(d.rs2));
                self.mmu.fence(va, asid);
                Ok(())
            }
            Kind::Csrrw => self.csr_op(d, |_, src| src),
            Kind::Csrrtmax => self.csr_op(d, |old, src| alu::tlut(old, src, isa::TMAX)),
            Kind::Csrrtmin => self.csr_op(d, |old, src| alu::tlut(old, src, isa::TMIN)),
            Kind::Wfi => {
                self.wfi();
                Ok(())
            }
            _ => Err(unsupported(d.op.name)),
        }
    }

    fn csr_op(&mut self, d: &Decoded, combine: fn(i64, i64) -> i64) -> Exec {
        let number = low(d.imm, 9);
        let old = self.read_csr(number)?;
        if d.rs1 != 0 {
            let src = self.x(d.rs1);
            self.write_csr(number, combine(old, src))?;
        }
        self.set_x(d.rd, old);
        Ok(())
    }
}
