//! One hart: registers, the fetch loop and the unprivileged instructions.
//!
//! Mirrors `detrytus/cpu.py` and `detrytus/execute.py`. This is phase 2
//! of docs/rust-plan.md: the integer, tritwise, memory, atomic and
//! control instructions. Privilege arrives in phase 3; until then the
//! hart stays in ring +1, has no CSRs, and so no trap handler can be
//! installed, which makes every trap unhandled (as it is for any program
//! that leaves `tvec` at 0). Instructions that need CSRs or other rings
//! stop the machine as [`Stop::Unsupported`] rather than approximating.

use std::collections::HashMap;

use crate::alu;
use crate::bus::{Access, Bus, BusError};
use crate::isa::{self, Decoded, Kind};
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
enum Exception {
    Trap { cause: i64, tval: i64 },
    Stop(Stop),
}

type Exec = Result<(), Exception>;

fn trap(cause: i64, tval: i64) -> Exception {
    Exception::Trap { cause, tval }
}

fn unsupported(what: &str) -> Exception {
    Exception::Stop(Stop::Unsupported(format!("{what} is not implemented in the Rust core yet")))
}

pub struct Hart {
    regs: [i64; 27],
    pub pc: i64,
    next_pc: i64,
    pub ring: i64,
    pub instret: i64,
    pub time: i64,
    reservation: Option<i64>,
    cache: HashMap<i64, Result<Decoded, isa::Illegal>>,
}

impl Default for Hart {
    fn default() -> Self {
        Self::new()
    }
}

impl Hart {
    pub fn new() -> Self {
        Hart {
            regs: [0; 27],
            pc: 0,
            next_pc: 0,
            ring: 1,
            instret: 0,
            time: 0,
            reservation: None,
            cache: HashMap::new(),
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

    // --- the fetch loop ----------------------------------------------------

    pub fn step(&mut self, bus: &mut Bus) -> Result<(), Stop> {
        self.time += 1;
        // No interrupt can be pending: they need ie, a CSR (phase 3).
        let pc = self.pc;
        match self.execute(bus, pc) {
            Ok(()) => {
                self.instret += 1;
                Ok(())
            }
            Err(Exception::Trap { cause, tval }) => self.enter_trap(cause, tval, pc),
            Err(Exception::Stop(stop)) => Err(stop),
        }
    }

    fn enter_trap(&mut self, cause: i64, tval: i64, pc: i64) -> Result<(), Stop> {
        // Without CSRs no bank has a tvec, so no handler is installed.
        self.reservation = None;
        Err(Stop::Unhandled { cause, tval, pc, ring: self.ring })
    }

    fn execute(&mut self, bus: &mut Bus, pc: i64) -> Exec {
        if low(pc, 1) != 0 {
            return Err(unsupported("compressed bundles (slot B)"));
        }
        let word = bus.read(pc, 3, Access::Fetch).map_err(|e| match e {
            BusError::Fault(addr) => trap(isa::ACCESS_FAULT, addr),
            BusError::Halt(code) => Exception::Stop(Stop::Halt(code)),
        })?;
        let d = self.decode(word)?;
        self.next_pc = wrap(pc + 3);
        self.run(bus, &d)?;
        self.pc = self.next_pc;
        Ok(())
    }

    fn decode(&mut self, word: i64) -> Result<Decoded, Exception> {
        if low(word, 5) == isa::BUNDLE_OPCODE {
            return Err(unsupported("compressed bundles"));
        }
        match self.cache.entry(word).or_insert_with(|| isa::decode(word)) {
            Ok(d) => Ok(*d),
            Err(_) => Err(trap(isa::ILLEGAL, word)),
        }
    }

    // --- memory ------------------------------------------------------------

    /// Virtual to physical. Ring +1 is never translated (docs/isa.md
    /// 8.1), and this core has no other ring yet.
    fn translate(&self, addr: i64) -> i64 {
        addr
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

    fn load(&mut self, bus: &mut Bus, addr: i64, width: u32) -> Result<i64, Exception> {
        Self::aligned(addr, width)?;
        bus.read(self.translate(addr), width, Access::Load).map_err(Self::bus_error(addr))
    }

    fn store(&mut self, bus: &mut Bus, addr: i64, width: u32, value: i64) -> Exec {
        Self::aligned(addr, width)?;
        bus.write(self.translate(addr), width, value).map_err(Self::bus_error(addr))
    }

    /// Atomic read-modify-write of a word; returns the old value.
    fn rmw(
        &mut self,
        bus: &mut Bus,
        addr: i64,
        update: impl Fn(i64) -> i64,
    ) -> Result<i64, Exception> {
        Self::aligned(addr, 3)?;
        let physical = self.translate(addr);
        let old = bus.read(physical, 3, Access::Load).map_err(Self::bus_error(addr))?;
        bus.write(physical, 3, update(old)).map_err(Self::bus_error(addr))?;
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
                let v = self.load(bus, addr, 3)?;
                self.set_x(d.rd, v);
                self.reservation = Some(addr);
            }
            Kind::Sc => {
                let addr = self.amo_addr(d)?;
                let ok = self.reservation == Some(addr);
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
            // A lone hart with no caches already satisfies these: nothing
            // is reordered and the decode cache is keyed by word value.
            Kind::Fence | Kind::FenceI | Kind::Cclean | Kind::Cflush => Ok(()),
            Kind::Cinval if self.ring < 0 => Err(trap(isa::ILLEGAL, 0)),
            Kind::Cinval => Ok(()),
            _ => Err(unsupported(d.op.name)),
        }
    }
}
