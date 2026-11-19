//! Paged virtual memory: the page walk, permission checks and the TLB
//! (docs/isa.md 9). Mirrors `detrytus/mmu.py`.
//!
//! The TLB keeps every successful translation until `fence.vm` removes
//! it and never evicts on its own: the strictest behaviour the spec
//! allows, so a kernel that forgets a fence sees stale translations at
//! once. A second map remembers each check already passed, keyed by
//! (ASID, page, access, ring, SUA); it is cleared with the TLB, so it
//! only saves repeating a check.

use std::collections::HashMap;

use crate::bus::{Access as BusAccess, Bus, PAGE_TRITS};
use crate::isa;
use crate::trits::{field, low, pow3, split_high};

const LEVEL_TRITS: u32 = 8;
const ROOT_LEVEL: u32 = 2;

// PTE trits (9.3)
const V: u32 = 0;
const R: usize = 1;
const W: usize = 2;
const X: usize = 3;
const U: usize = 4;
const G: usize = 5;
const A: usize = 6;
const D: usize = 7;
const S: usize = 8;

/// What a translation is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Access {
    Fetch,
    Load,
    Store,
    /// Read-modify-write: needs R and W, faults like a store.
    Amo,
}

impl Access {
    fn fault_cause(self) -> i64 {
        match self {
            Access::Fetch => isa::INSTR_PAGE_FAULT,
            Access::Load => isa::LOAD_PAGE_FAULT,
            Access::Store | Access::Amo => isa::STORE_PAGE_FAULT,
        }
    }

    fn needs(self) -> &'static [usize] {
        match self {
            Access::Fetch => &[X],
            Access::Load => &[R],
            Access::Store => &[W],
            Access::Amo => &[R, W],
        }
    }
}

/// A trap the translation raises: (cause, tval).
pub type Fault = (i64, i64);

/// The `ptbr` CSR, decoded once per write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ptbr {
    pub mode: i64,
    pub asid: i64,
    /// Physical page whose centre holds the root table.
    pub root: i64,
}

impl Ptbr {
    pub fn decode(value: i64) -> Self {
        Ptbr { mode: low(value, 1), asid: field(value, 1, 8), root: split_high(value, PAGE_TRITS) }
    }
}

/// A translation from the TLB: where the virtual page's base page
/// starts, and the leaf's flag trits (indexed V, R, W, ...).
#[derive(Clone, Copy)]
struct Leaf {
    page_centre: i64,
    flags: [i64; 9],
}

pub fn page_number(addr: i64) -> i64 {
    split_high(addr, PAGE_TRITS)
}

/// The table index for a level: root trits 26..25, mid 24..17, leaf 16..9.
fn index(va: i64, level: u32) -> i64 {
    let width = if level == ROOT_LEVEL { 2 } else { LEVEL_TRITS };
    field(va, PAGE_TRITS + LEVEL_TRITS * level, width)
}

type CheckKey = (i64, i64, Access, i64, bool);

#[derive(Default)]
pub struct Mmu {
    tlb: HashMap<(i64, i64), Leaf>,
    global: HashMap<i64, Leaf>,
    checked: HashMap<CheckKey, i64>,
}

impl Mmu {
    pub fn translate(
        &mut self,
        bus: &mut Bus,
        va: i64,
        access: Access,
        ring: i64,
        sua: bool,
        ptbr: Ptbr,
    ) -> Result<i64, Fault> {
        let vpn = page_number(va);
        let key = (ptbr.asid, vpn, access, ring, sua);
        if let Some(centre) = self.checked.get(&key) {
            return Ok(centre + low(va, PAGE_TRITS));
        }
        let cached = self.global.get(&vpn).or_else(|| self.tlb.get(&(ptbr.asid, vpn))).copied();
        let leaf = match cached {
            Some(leaf) => {
                check(&leaf.flags, va, access, ring, sua)?;
                leaf
            }
            None => {
                let leaf = walk(bus, va, access, ptbr)?;
                check(&leaf.flags, va, access, ring, sua)?;
                if leaf.flags[G] == 1 {
                    self.global.insert(vpn, leaf);
                } else {
                    self.tlb.insert((ptbr.asid, vpn), leaf);
                }
                leaf
            }
        };
        self.checked.insert(key, leaf.page_centre);
        Ok(leaf.page_centre + low(va, PAGE_TRITS))
    }

    /// `fence.vm`: `va` and `asid` are None when their register field is r0.
    pub fn fence(&mut self, va: Option<i64>, asid: Option<i64>) {
        self.checked.clear();
        let vpn = va.map(page_number);
        let page_matches = |page: i64| vpn.is_none_or(|v| v == page);
        match asid {
            None => {
                self.tlb.retain(|&(_, page), _| !page_matches(page));
                self.global.retain(|&page, _| !page_matches(page));
            }
            // Naming an address space leaves global entries alone.
            Some(asid) => self.tlb.retain(|&(a, page), _| !(a == asid && page_matches(page))),
        }
    }
}

fn walk(bus: &mut Bus, va: i64, access: Access, ptbr: Ptbr) -> Result<Leaf, Fault> {
    let fault = (access.fault_cause(), va);
    let mut table = ptbr.root * pow3(PAGE_TRITS);
    for level in (0..=ROOT_LEVEL).rev() {
        let pte = bus
            .read(table + 3 * index(va, level), 3, BusAccess::Walk)
            .map_err(|_| (isa::ACCESS_FAULT, va))?;
        match field(pte, V, 1) {
            0 => return Err(fault),
            // A pointer: carries no permissions, and the leaf level has
            // nothing below it to point to.
            -1 => {
                if level == 0 || field(pte, R as u32, 7) != 0 {
                    return Err(fault);
                }
                table = page_number(pte) * pow3(PAGE_TRITS);
            }
            _ => return leaf(pte, va, level).ok_or(fault),
        }
    }
    unreachable!("level 0 always returns or faults")
}

fn leaf(pte: i64, va: i64, level: u32) -> Option<Leaf> {
    let mut flags = [0; 9];
    for (i, f) in flags.iter_mut().enumerate() {
        *f = field(pte, i as u32, 1);
    }
    if flags[R..S].contains(&-1) {
        return None; // '-' in a flag trit is reserved
    }
    let span = LEVEL_TRITS * level; // trits of virtual page number the leaf covers
    let ppn = page_number(pte);
    if low(ppn, span) != 0 {
        return None; // a superpage must be aligned to its own size
    }
    // The base page this va falls in, within a possibly larger leaf.
    let within = split_high(low(va, PAGE_TRITS + span), PAGE_TRITS);
    Some(Leaf { page_centre: (ppn + within) * pow3(PAGE_TRITS), flags })
}

fn check(flags: &[i64; 9], va: i64, access: Access, ring: i64, sua: bool) -> Result<(), Fault> {
    let user = flags[U] == 1;
    let kernel = ring >= 0;
    let allowed = access.needs().iter().all(|&t| flags[t] == 1)
        && (kernel || user)
        // the kernel never runs user code, and reads it only with SUA
        && !(kernel && user && (access == Access::Fetch || !sua))
        && flags[A] == 1
        && !(matches!(access, Access::Store | Access::Amo) && flags[D] != 1);
    if allowed { Ok(()) } else { Err((access.fault_cause(), va)) }
}
