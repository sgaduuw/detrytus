//! A JIT compiler for detrytus: runs of ternary instructions compiled
//! to host code with Cranelift, for whatever host this is built for
//! (x86-64, arm64, ...), using the CPU features found at run time.
//!
//! A block is the run of instructions from a pc up to and including
//! the first jump or branch, stopping short of anything compiled code
//! leaves to the interpreter (atomics, system instructions, a fetch that
//! would trap) and at the end of the page. Blocks are kept by virtual
//! pc and the physical address it fetches from, and all are dropped
//! when a store hits decoded code (`Bus::code_writes`).
//!
//! Compiled code does exactly what the interpreter would. Integer
//! arithmetic, compares, selects and control flow are compiled inline;
//! other register-only instructions call back into the core
//! (`Hart::jit_compute`).
//!
//! Loads and stores to RAM are compiled inline too, through a small
//! direct-mapped table from virtual page to the host memory holding the
//! RAM page (`Hart::jit_page`), kept while the hart's view of memory
//! (`Hart::jit_view`) is unchanged. An access the table misses calls
//! back into the core (`Hart::jit_load`, `Hart::jit_store`), which
//! refuses devices and anything that would trap, and fills the table
//! if the page qualifies. A refusal ends the block before that
//! instruction, and the interpreter runs it.

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{AbiParam, BlockArg, InstBuilder, MemFlagsData, Type, Value, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};

use detrytus_core::accel::{self, Accelerator, Class, Insn, MemoryView};
use detrytus_core::bus::{Bus, PAGE, page_of};
use detrytus_core::fastmap::FastMap;
use detrytus_core::hart::Hart;
use detrytus_core::isa::{self, Kind};
use detrytus_core::trits::{TRYTE_MOD, WORD_MAX, WORD_MOD, low, pow3, wrap};

/// The most instructions in one compiled function.
const REGION_MAX: usize = 256;

/// Visits before code at a pc is compiled.
const HOT: u32 = 1000;

/// Drop everything compiled past this many blocks, to bound memory.
const BLOCKS_MAX: usize = 1 << 16;

/// What a load helper returns when it refuses: no word is this small.
const REFUSED: i64 = i64::MIN;

/// Slots in each page table (a power of 3 would not help: slots are
/// picked by the page number's low bits).
const PAGE_SLOTS: usize = 256;

/// A page table slot: a virtual page number and where the RAM page it
/// maps to has its centre tryte, or `EMPTY`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Slot {
    vpage: i64,
    centre: *mut i16,
}

const SLOT_BYTES: i64 = 16;
const SLOT_CENTRE: i32 = 8;

/// No page number is this small.
const EMPTY: Slot = Slot { vpage: i64::MIN, centre: std::ptr::null_mut() };

/// Pages compiled code loads from and stores to directly, by virtual
/// page number, found while the hart's view of memory was `view`.
struct Pages {
    view: Option<MemoryView>,
    load: Box<[Slot]>,
    store: Box<[Slot]>,
}

impl Pages {
    fn new() -> Self {
        Pages {
            view: None,
            load: vec![EMPTY; PAGE_SLOTS].into(),
            store: vec![EMPTY; PAGE_SLOTS].into(),
        }
    }
}

/// Views kept at once: a kernel and the program it runs alternate, and
/// each keeps its pages across the switch.
const VIEWS: usize = 4;

/// The pages found in the last few views, most recent first.
struct Views(Vec<Pages>);

impl Views {
    fn new() -> Self {
        Views((0..VIEWS).map(|_| Pages::new()).collect())
    }

    /// The pages for a view: those found in it earlier, or none. A view
    /// that differs in anything (a fence, newly decoded code) shares
    /// nothing with an older one, which is never current again.
    fn get(&mut self, view: MemoryView) -> &mut Pages {
        let i = match self.0.iter().position(|p| p.view == Some(view)) {
            Some(i) => i,
            None => {
                let oldest = self.0.last_mut().expect("VIEWS > 0");
                oldest.load.fill(EMPTY);
                oldest.store.fill(EMPTY);
                oldest.view = Some(view);
                VIEWS - 1
            }
        };
        self.0[..=i].rotate_right(1);
        &mut self.0[0]
    }
}

fn slot_of(vpage: i64) -> usize {
    (vpage as usize) & (PAGE_SLOTS - 1)
}

/// Splitting a value into a multiple of PAGE (a tryte's range) and the
/// balanced remainder, without a division: v + SPLIT_BIAS is never
/// negative and stays below 2^46, and for such t,
/// floor(t / PAGE) = umulhi(t, SPLIT_MAGIC) >> SPLIT_SHIFT.
const SPLIT_STEPS: i64 = 1 << 30;
const SPLIT_BIAS: i64 = PAGE / 2 + PAGE * SPLIT_STEPS;
const SPLIT_SHIFT: i64 = 14;
const SPLIT_MAGIC: u64 = ((1u128 << (64 + SPLIT_SHIFT)) / PAGE as u128 + 1) as u64;

/// The multiplicative inverse of 3 modulo 2^64: x is a multiple of 3
/// exactly when x * INV3 <= u64::MAX / 3.
const INV3: u64 = 0xAAAA_AAAA_AAAA_AAAB;

/// What compiled code is called with. Its layout is fixed: compiled
/// code reads `regs` and `insns` and writes `next_pc` at these offsets.
#[repr(C)]
struct Ctx {
    regs: *mut i64,
    hart: *mut Hart,
    bus: *mut Bus,
    insns: *const Insn,
    next_pc: i64,
    /// The most instructions the call may run.
    budget: i64,
    /// `Pages::load` and `Pages::store`.
    load_pages: *mut Slot,
    store_pages: *mut Slot,
}

const CTX_REGS: i32 = 0;
const CTX_NEXT_PC: i32 = 32;
const CTX_BUDGET: i32 = 40;
const CTX_LOAD_PAGES: i32 = 48;
const CTX_STORE_PAGES: i32 = 56;

/// A compiled block: returns how many of its instructions ran, having
/// written the pc to go on from into `next_pc`.
type BlockFn = unsafe extern "C" fn(*mut Ctx) -> i64;

struct Block {
    f: BlockFn,
    /// The function's instructions, for helpers that run one.
    insns: Box<[Insn]>,
}

/// The JIT: compiled blocks and the Cranelift module holding their code.
pub struct Jit {
    module: JITModule,
    builder: FunctionBuilderContext,
    /// By (virtual pc, physical address); None where nothing compiles.
    blocks: FastMap<(i64, i64), Option<Block>>,
    /// How often each pc not yet compiled was asked for: code is
    /// compiled once it is hot, so code run a few times costs no
    /// compile.
    visits: FastMap<(i64, i64), u32>,
    /// `Bus::code_writes` when the blocks were made.
    writes: u64,
    helpers: Helpers,
    /// Visits before code at a pc is compiled.
    hot: u32,
    pages: Views,
}

#[derive(Clone, Copy)]
struct Helpers {
    load: FuncId,
    store: FuncId,
    compute: FuncId,
}

fn new_module() -> (JITModule, Helpers) {
    let mut flags = settings::builder();
    flags.set("opt_level", "none").unwrap();
    // The verifier checks Cranelift itself, at a cost of compile time.
    flags.set("enable_verifier", "false").unwrap();
    flags.set("regalloc_algorithm", "single_pass").unwrap();
    flags.set("use_colocated_libcalls", "false").unwrap();
    flags.set("is_pic", "false").unwrap();
    // The host's own instruction set, with the features this CPU has.
    let isa = cranelift_native::builder()
        .expect("Cranelift supports this host")
        .finish(settings::Flags::new(flags))
        .expect("the host's flags are valid");
    let mut jb = JITBuilder::with_isa(isa, default_libcall_names());
    jb.symbol("detrytus_load", helper_load as *const u8);
    jb.symbol("detrytus_store", helper_store as *const u8);
    jb.symbol("detrytus_compute", helper_compute as *const u8);
    let mut module = JITModule::new(jb);
    let ptr = module.target_config().pointer_type();
    let sig = |params: &[Type], ret: bool| {
        let mut s = module.make_signature();
        s.params.extend(params.iter().map(|&t| AbiParam::new(t)));
        if ret {
            s.returns.push(AbiParam::new(types::I64));
        }
        s
    };
    let load_sig = sig(&[ptr, types::I64, types::I64], true);
    let store_sig = sig(&[ptr, types::I64, types::I64, types::I64], true);
    let compute_sig = sig(&[ptr, types::I64, types::I64], true);
    let helpers = Helpers {
        load: module.declare_function("detrytus_load", Linkage::Import, &load_sig).unwrap(),
        store: module.declare_function("detrytus_store", Linkage::Import, &store_sig).unwrap(),
        compute: module
            .declare_function("detrytus_compute", Linkage::Import, &compute_sig)
            .unwrap(),
    };
    (module, helpers)
}

// --- helpers compiled code calls -------------------------------------------

/// Note the page an access at addr just reached, if compiled code may
/// reach it directly from now on.
fn remember(ctx: &mut Ctx, hart: &mut Hart, bus: &mut Bus, addr: i64, store: bool) {
    if let Some(centre) = hart.jit_page(bus, addr, store) {
        let vpage = page_of(addr);
        let table = if store { ctx.store_pages } else { ctx.load_pages };
        // SAFETY: the table has PAGE_SLOTS slots and outlives the call.
        unsafe { *table.add(slot_of(vpage)) = Slot { vpage, centre } };
    }
}

/// A load: the value, or REFUSED.
extern "C" fn helper_load(ctx: *mut Ctx, addr: i64, width: i64) -> i64 {
    // SAFETY: compiled code passes the Ctx `run` made, whose pointers
    // are live for the call; nothing else touches the hart or bus then.
    let ctx = unsafe { &mut *ctx };
    let (hart, bus) = unsafe { (&mut *ctx.hart, &mut *ctx.bus) };
    match hart.jit_load(bus, addr, width as u32) {
        Some(v) => {
            remember(ctx, hart, bus, addr, false);
            v
        }
        None => REFUSED,
    }
}

/// A store: 0 done, 1 refused, 2 done but it changed decoded code (so
/// the block, and every other, may be stale).
extern "C" fn helper_store(ctx: *mut Ctx, addr: i64, width: i64, value: i64) -> i64 {
    // SAFETY: as for helper_load.
    let ctx = unsafe { &mut *ctx };
    let (hart, bus) = unsafe { (&mut *ctx.hart, &mut *ctx.bus) };
    let before = bus.code_writes();
    match hart.jit_store(bus, addr, width as u32, value) {
        None => 1,
        Some(()) if bus.code_writes() != before => 2,
        Some(()) => {
            remember(ctx, hart, bus, addr, true);
            0
        }
    }
}

/// Run instruction `index` of the block, at pc: 0 done, 1 it would trap.
extern "C" fn helper_compute(ctx: *mut Ctx, index: i64, pc: i64) -> i64 {
    // SAFETY: as for helper_load; insns has the block's instructions.
    let ctx = unsafe { &mut *ctx };
    let (hart, insn) = unsafe { (&mut *ctx.hart, &*ctx.insns.add(index as usize)) };
    if hart.jit_compute(&insn.d, pc) { 0 } else { 1 }
}

impl Default for Jit {
    fn default() -> Self {
        Self::new()
    }
}

impl Jit {
    pub fn new() -> Self {
        let (module, helpers) = new_module();
        Jit {
            module,
            builder: FunctionBuilderContext::new(),
            blocks: FastMap::default(),
            visits: FastMap::default(),
            writes: 0,
            helpers,
            hot: HOT,
            pages: Views::new(),
        }
    }

    /// A JIT that compiles code once it has been asked for it `hot`
    /// times (1: at once), rather than the default, for tests.
    pub fn with_threshold(hot: u32) -> Self {
        Jit { hot: hot.max(1), ..Self::new() }
    }

    fn flush(&mut self) {
        self.blocks.clear();
        self.visits.clear();
        let (module, helpers) = new_module();
        let old = std::mem::replace(&mut self.module, module);
        // SAFETY: every block made from the old module was just dropped.
        unsafe { old.free_memory() };
        self.helpers = helpers;
    }

    /// The code reachable from pc without leaving its page: through
    /// fall-throughs and branch and jump targets, stopping at what
    /// compiled code leaves to the interpreter. Instructions in address
    /// order, with their pcs.
    fn gather(hart: &mut Hart, bus: &mut Bus, pc: i64) -> Vec<(i64, Insn)> {
        let page = page_of(pc);
        let mut found: FastMap<i64, Insn> = FastMap::default();
        let mut work = vec![pc];
        let mut refused = FastMap::default();
        while let Some(at) = work.pop() {
            if found.len() >= REGION_MAX {
                break;
            }
            if page_of(at) != page || found.contains_key(&at) || refused.contains_key(&at) {
                continue;
            }
            let Some(insn) = hart.jit_fetch(bus, at) else {
                refused.insert(at, ());
                continue;
            };
            if accel::class(&insn.d) == Class::Other {
                refused.insert(at, ());
                continue;
            }
            found.insert(at, insn);
            let d = &insn.d;
            let next = wrap(at + insn.advance);
            let base = at - low(at, 1);
            match d.op.kind {
                Kind::Jal => work.push(wrap(base + 3 * d.imm)),
                Kind::Jalr => {}
                Kind::B => work.extend([next, wrap(base + 3 * d.off)]),
                Kind::Br3 => work.extend([next, wrap(base + 3 * d.offn), wrap(base + 3 * d.offp)]),
                _ => work.push(next),
            }
        }
        let mut code: Vec<(i64, Insn)> = found.into_iter().collect();
        code.sort_unstable_by_key(|(at, _)| *at);
        code
    }

    /// Compile the code found from pc into one function entered at pc.
    /// Jumps within it are jumps in host code; leaving it returns.
    fn compile(&mut self, code: Vec<(i64, Insn)>, entry: i64) -> Option<Block> {
        if code.is_empty() {
            return None;
        }
        let index: FastMap<i64, usize> =
            code.iter().enumerate().map(|(i, (at, _))| (*at, i)).collect();
        // Leaders: where a run of instructions can be entered other than
        // from the one before it. Each checks the budget for its run.
        let mut leader = vec![false; code.len()];
        leader[index[&entry]] = true;
        for (i, (at, insn)) in code.iter().enumerate() {
            let d = &insn.d;
            let base = at - low(*at, 1);
            let targets: &[i64] = match d.op.kind {
                Kind::Jal => &[wrap(base + 3 * d.imm)],
                Kind::B => &[wrap(base + 3 * d.off)],
                Kind::Br3 => &[wrap(base + 3 * d.offn), wrap(base + 3 * d.offp)],
                _ => &[],
            };
            for t in targets {
                if let Some(&j) = index.get(t) {
                    leader[j] = true;
                }
            }
            if accel::class(d) == Class::Control && i + 1 < code.len() {
                leader[i + 1] = true;
            }
            // A run also starts after a gap in the code.
            let next = wrap(at + insn.advance);
            if i + 1 < code.len() && code[i + 1].0 != next {
                leader[i + 1] = true;
            }
        }
        // The length of the run each leader starts.
        let mut run_len = vec![0i64; code.len()];
        for i in (0..code.len()).rev() {
            let (at, insn) = &code[i];
            let falls = accel::class(&insn.d) != Class::Control
                && i + 1 < code.len()
                && code[i + 1].0 == wrap(at + insn.advance)
                && !leader[i + 1];
            run_len[i] = 1 + if falls { run_len[i + 1] } else { 0 };
        }

        let target = self.module.target_config();
        let ptr = target.pointer_type();
        let mut ctx = self.module.make_context();
        ctx.func.signature.params.push(AbiParam::new(ptr));
        ctx.func.signature.returns.push(AbiParam::new(types::I64));
        let insns: Vec<Insn> = code.iter().map(|(_, insn)| *insn).collect();
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut self.builder);
            let start = b.create_block();
            b.append_block_params_for_function_params(start);
            let blocks: Vec<_> = code.iter().map(|_| b.create_block()).collect();
            b.switch_to_block(start);
            let cx = b.block_params(start)[0];
            let load = self.module.declare_func_in_func(self.helpers.load, b.func);
            let store = self.module.declare_func_in_func(self.helpers.store, b.func);
            let compute = self.module.declare_func_in_func(self.helpers.compute, b.func);
            let trusted = MemFlagsData::trusted();
            let n = b.declare_var(types::I64);
            // Every way out: (count, pc).
            let exit = b.create_block();
            b.append_block_param(exit, types::I64);
            b.append_block_param(exit, types::I64);
            let mut g = Gen {
                b,
                trusted,
                n,
                ptr,
                regs: cx,
                budget: cx,
                load_pages: cx,
                store_pages: cx,
                exit,
            };
            g.regs = g.b.ins().load(ptr, trusted, cx, CTX_REGS);
            g.budget = g.b.ins().load(types::I64, trusted, cx, CTX_BUDGET);
            g.load_pages = g.b.ins().load(ptr, trusted, cx, CTX_LOAD_PAGES);
            g.store_pages = g.b.ins().load(ptr, trusted, cx, CTX_STORE_PAGES);
            let zero = g.b.ins().iconst(types::I64, 0);
            g.b.def_var(n, zero);
            g.b.ins().jump(blocks[index[&entry]], &[]);
            // Where control goes to pc: a block of this function, or out.
            let go = |g: &mut Gen, pc: i64| match index.get(&pc) {
                Some(&j) => {
                    g.b.ins().jump(blocks[j], &[]);
                }
                None => {
                    let to = g.b.ins().iconst(types::I64, pc);
                    g.finish(to);
                }
            };
            for (i, (pc, insn)) in code.iter().enumerate() {
                let pc = *pc;
                g.b.switch_to_block(blocks[i]);
                if leader[i] {
                    // Run on only if the whole run fits the budget.
                    let count = g.b.use_var(n);
                    let after = g.b.ins().iadd_imm_s(count, run_len[i]);
                    let over = g.b.ins().icmp(IntCC::SignedGreaterThan, after, g.budget);
                    g.exit_if(over, 0, pc);
                }
                let d = &insn.d;
                let next = wrap(pc + insn.advance);
                let base = pc - low(pc, 1);
                let regs = g.regs;
                match d.op.kind {
                    Kind::Add | Kind::Sub => {
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let v = if d.op.kind == Kind::Add {
                            g.b.ins().iadd(a, c)
                        } else {
                            g.b.ins().isub(a, c)
                        };
                        let v = g.reduce(v);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Addi => {
                        let a = g.reg(regs, d.rs1);
                        let v = g.b.ins().iadd_imm_s(a, d.imm);
                        let v = g.reduce(v);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Lui => {
                        let v = g.b.ins().iconst(types::I64, d.imm * pow3(8));
                        g.set(regs, d.rd, v);
                    }
                    Kind::Auipc => {
                        let v = g.b.ins().iconst(types::I64, wrap(pc + d.imm * pow3(8)));
                        g.set(regs, d.rd, v);
                    }
                    Kind::Cmp | Kind::Cmpi => {
                        let a = g.reg(regs, d.rs1);
                        let c = if d.op.kind == Kind::Cmp {
                            g.reg(regs, d.rs2)
                        } else {
                            g.b.ins().iconst(types::I64, d.imm)
                        };
                        let v = g.sign_of(a, c);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Min | Kind::Max => {
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let v = if d.op.kind == Kind::Min {
                            g.b.ins().smin(a, c)
                        } else {
                            g.b.ins().smax(a, c)
                        };
                        g.set(regs, d.rd, v);
                    }
                    Kind::Mul => {
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let v = g.mul_low(a, c);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Div | Kind::Rem => {
                        // C-style, and by zero (0, a): alu::div_trunc.
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let one = g.b.ins().iconst(types::I64, 1);
                        let zero = g.b.ins().icmp_imm_s(IntCC::Equal, c, 0);
                        let safe = g.b.ins().select(zero, one, c);
                        let v = if d.op.kind == Kind::Div {
                            let q = g.b.ins().sdiv(a, safe);
                            let none = g.b.ins().iconst(types::I64, 0);
                            g.b.ins().select(zero, none, q)
                        } else {
                            let r = g.b.ins().srem(a, safe);
                            g.b.ins().select(zero, a, r)
                        };
                        g.set(regs, d.rd, v);
                    }
                    Kind::Band | Kind::Bor | Kind::Bxor => {
                        // On the two's complement forms, wrapped back
                        // into the word range: never more than one
                        // modulus out, as |a op c| < 2^42.
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let v = match d.op.kind {
                            Kind::Band => g.b.ins().band(a, c),
                            Kind::Bor => g.b.ins().bor(a, c),
                            _ => g.b.ins().bxor(a, c),
                        };
                        let v = g.reduce(v);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Sel => {
                        let (a, neg, pos) =
                            (g.reg(regs, d.rs1), g.reg(regs, d.rs2), g.reg(regs, d.rs3));
                        let zero = g.b.ins().iconst(types::I64, 0);
                        let is_neg = g.b.ins().icmp_imm_s(IntCC::SignedLessThan, a, 0);
                        let is_pos = g.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, a, 0);
                        let v = g.b.ins().select(is_pos, pos, zero);
                        let v = g.b.ins().select(is_neg, neg, v);
                        g.set(regs, d.rd, v);
                    }
                    Kind::Lt | Kind::Lw => {
                        let width = if d.op.kind == Kind::Lt { 1 } else { 3 };
                        let a = g.reg(regs, d.rs1);
                        let addr = g.b.ins().iadd_imm_s(a, d.imm);
                        let addr = g.reduce(addr);
                        let (host, fast, slow) = g.find_page(addr, width, false);
                        let join = g.b.create_block();
                        g.b.append_block_param(join, types::I64);
                        g.b.switch_to_block(fast);
                        let v = if width == 1 {
                            g.b.ins().sload16(types::I64, trusted, host, 0)
                        } else {
                            let lo = g.b.ins().sload16(types::I64, trusted, host, -2);
                            let mid = g.b.ins().sload16(types::I64, trusted, host, 0);
                            let hi = g.b.ins().sload16(types::I64, trusted, host, 2);
                            let v = g.b.ins().imul_imm_s(hi, TRYTE_MOD);
                            let v = g.b.ins().iadd(v, mid);
                            let v = g.b.ins().imul_imm_s(v, TRYTE_MOD);
                            g.b.ins().iadd(v, lo)
                        };
                        g.b.ins().jump(join, &[BlockArg::Value(v)]);
                        g.b.switch_to_block(slow);
                        let w = g.b.ins().iconst(types::I64, width);
                        let call = g.b.ins().call(load, &[cx, addr, w]);
                        let v = g.b.inst_results(call)[0];
                        let refused = g.b.ins().icmp_imm_s(IntCC::Equal, v, REFUSED);
                        g.exit_if(refused, 0, pc);
                        g.b.ins().jump(join, &[BlockArg::Value(v)]);
                        g.b.switch_to_block(join);
                        let v = g.b.block_params(join)[0];
                        g.set(regs, d.rd, v);
                    }
                    Kind::St | Kind::Sw => {
                        let width = if d.op.kind == Kind::St { 1 } else { 3 };
                        let (a, value) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let addr = g.b.ins().iadd_imm_s(a, d.imm);
                        let addr = g.reduce(addr);
                        let (host, fast, slow) = g.find_page(addr, width, true);
                        let join = g.b.create_block();
                        g.b.switch_to_block(fast);
                        // The value's trytes, low first, as Bus::write
                        // splits it.
                        let (rest, lo) = g.split(value);
                        if width == 1 {
                            g.b.ins().istore16(trusted, lo, host, 0);
                        } else {
                            let (rest, mid) = g.split(rest);
                            let (_, hi) = g.split(rest);
                            g.b.ins().istore16(trusted, lo, host, -2);
                            g.b.ins().istore16(trusted, mid, host, 0);
                            g.b.ins().istore16(trusted, hi, host, 2);
                        }
                        g.b.ins().jump(join, &[]);
                        g.b.switch_to_block(slow);
                        let w = g.b.ins().iconst(types::I64, width);
                        let call = g.b.ins().call(store, &[cx, addr, w, value]);
                        let status = g.b.inst_results(call)[0];
                        let refused = g.b.ins().icmp_imm_s(IntCC::Equal, status, 1);
                        g.exit_if(refused, 0, pc);
                        let changed = g.b.ins().icmp_imm_s(IntCC::Equal, status, 2);
                        g.exit_if(changed, 1, next);
                        g.b.ins().jump(join, &[]);
                        g.b.switch_to_block(join);
                    }
                    Kind::Jal => {
                        let v = g.b.ins().iconst(types::I64, next);
                        g.set(regs, d.rd, v);
                        g.count();
                        go(&mut g, wrap(base + 3 * d.imm));
                        continue;
                    }
                    Kind::Jalr => {
                        let a = g.reg(regs, d.rs1);
                        let addr = g.b.ins().iadd_imm_s(a, d.imm);
                        let addr = g.reduce(addr);
                        // A target whose low trit is − traps: leave it to
                        // the interpreter. low(x, 1) = −1 when x mod 3 = 2.
                        let r = g.b.ins().srem_imm_s(addr, 3);
                        let r3 = g.b.ins().iadd_imm_s(r, 3);
                        let m = g.b.ins().srem_imm_s(r3, 3);
                        let bad = g.b.ins().icmp_imm_s(IntCC::Equal, m, 2);
                        g.exit_if(bad, 0, pc);
                        let v = g.b.ins().iconst(types::I64, next);
                        g.set(regs, d.rd, v);
                        g.count();
                        g.finish(addr);
                        continue;
                    }
                    Kind::B => {
                        let (a, c) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let taken = g.condition(d.cond, a, c);
                        g.count();
                        let (yes, no) = (g.b.create_block(), g.b.create_block());
                        g.b.ins().brif(taken, yes, &[], no, &[]);
                        g.b.switch_to_block(yes);
                        go(&mut g, wrap(base + 3 * d.off));
                        g.b.switch_to_block(no);
                        go(&mut g, next);
                        continue;
                    }
                    Kind::Br3 => {
                        let a = g.reg(regs, d.rs1);
                        g.count();
                        let (neg, rest, pos, zero) = (
                            g.b.create_block(),
                            g.b.create_block(),
                            g.b.create_block(),
                            g.b.create_block(),
                        );
                        let is_neg = g.b.ins().icmp_imm_s(IntCC::SignedLessThan, a, 0);
                        g.b.ins().brif(is_neg, neg, &[], rest, &[]);
                        g.b.switch_to_block(rest);
                        let is_pos = g.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, a, 0);
                        g.b.ins().brif(is_pos, pos, &[], zero, &[]);
                        g.b.switch_to_block(neg);
                        go(&mut g, wrap(base + 3 * d.offn));
                        g.b.switch_to_block(pos);
                        go(&mut g, wrap(base + 3 * d.offp));
                        g.b.switch_to_block(zero);
                        go(&mut g, next);
                        continue;
                    }
                    _ => {
                        // Register-only, run by the core.
                        let index = g.b.ins().iconst(types::I64, i as i64);
                        let at = g.b.ins().iconst(types::I64, pc);
                        let call = g.b.ins().call(compute, &[cx, index, at]);
                        let status = g.b.inst_results(call)[0];
                        let trapped = g.b.ins().icmp_imm_s(IntCC::Equal, status, 1);
                        g.exit_if(trapped, 0, pc);
                    }
                }
                g.count();
                go(&mut g, next);
            }
            g.b.switch_to_block(exit);
            let (count, pc) = (g.b.block_params(exit)[0], g.b.block_params(exit)[1]);
            g.b.ins().store(trusted, pc, cx, CTX_NEXT_PC);
            g.b.ins().return_(&[count]);
            g.b.seal_all_blocks();
            g.b.finalize(target);
        }
        let id = self.module.declare_anonymous_function(&ctx.func.signature).ok()?;
        self.module.define_function(id, &mut ctx).ok()?;
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions().ok()?;
        let code = self.module.get_finalized_function(id);
        // SAFETY: the function was built with exactly BlockFn's signature.
        let f = unsafe { std::mem::transmute::<*const u8, BlockFn>(code) };
        Some(Block { f, insns: insns.into_boxed_slice() })
    }
}

/// Building one function's code.
struct Gen<'a> {
    b: FunctionBuilder<'a>,
    /// Flags for the context and register accesses: aligned, never trap.
    trusted: MemFlagsData,
    /// Instructions run so far.
    n: Variable,
    ptr: Type,
    regs: Value,
    budget: Value,
    load_pages: Value,
    store_pages: Value,
    /// The shared way out, taking (count, pc).
    exit: cranelift_codegen::ir::Block,
}

impl Gen<'_> {
    fn reg(&mut self, regs: Value, r: i64) -> Value {
        if r == 0 {
            return self.b.ins().iconst(types::I64, 0);
        }
        self.b.ins().load(types::I64, self.trusted, regs, (r + 13) as i32 * 8)
    }

    fn set(&mut self, regs: Value, r: i64, v: Value) {
        if r != 0 {
            self.b.ins().store(self.trusted, v, regs, (r + 13) as i32 * 8);
        }
    }

    /// A sum or difference of two words, or a word and a small
    /// immediate, wrapped back into the word range.
    fn reduce(&mut self, v: Value) -> Value {
        let high = self.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, v, WORD_MAX);
        let down = self.b.ins().iadd_imm_s(v, -WORD_MOD);
        let v = self.b.ins().select(high, down, v);
        let lowv = self.b.ins().icmp_imm_s(IntCC::SignedLessThan, v, -WORD_MAX);
        let up = self.b.ins().iadd_imm_s(v, WORD_MOD);
        self.b.ins().select(lowv, up, v)
    }

    /// −1, 0 or +1 as a < c, a = c, a > c.
    fn sign_of(&mut self, a: Value, c: Value) -> Value {
        let gt = self.b.ins().icmp(IntCC::SignedGreaterThan, a, c);
        let lt = self.b.ins().icmp(IntCC::SignedLessThan, a, c);
        let gt = self.b.ins().uextend(types::I64, gt);
        let lt = self.b.ins().uextend(types::I64, lt);
        self.b.ins().isub(gt, lt)
    }

    /// Whether a branch on cond is taken for operands a and c.
    fn condition(&mut self, cond: i64, a: Value, c: Value) -> Value {
        let mut taken = self.b.ins().iconst(types::I8, 0);
        for (sign, cc) in
            [(-1, IntCC::SignedLessThan), (0, IntCC::Equal), (1, IntCC::SignedGreaterThan)]
        {
            if isa::cond_taken(cond, sign) == Some(true) {
                let t = self.b.ins().icmp(cc, a, c);
                taken = self.b.ins().bor(taken, t);
            }
        }
        taken
    }

    /// The low word of the product a * c (`alu::mul_parts`): the
    /// product modulo 3^27, without 128-bit arithmetic. The quotient
    /// estimated in floating point is within 1 of the right one, and
    /// the remainder taken with it, computed modulo 2^64, is exact as
    /// it is far smaller than 2^63; `reduce` corrects the last step.
    fn mul_low(&mut self, a: Value, c: Value) -> Value {
        let af = self.b.ins().fcvt_from_sint(types::F64, a);
        let cf = self.b.ins().fcvt_from_sint(types::F64, c);
        let p = self.b.ins().fmul(af, cf);
        let inv = self.b.ins().f64const(1.0 / WORD_MOD as f64);
        let q = self.b.ins().fmul(p, inv);
        let q = self.b.ins().nearest(q);
        let q = self.b.ins().fcvt_to_sint_sat(types::I64, q);
        let p = self.b.ins().imul(a, c);
        let whole = self.b.ins().imul_imm_s(q, WORD_MOD);
        let r = self.b.ins().isub(p, whole);
        self.reduce(r)
    }

    /// v split as q * PAGE + r with r in a tryte's balanced range:
    /// (q, r), as `split_high` and `low` would give.
    fn split(&mut self, v: Value) -> (Value, Value) {
        let t = self.b.ins().iadd_imm_s(v, SPLIT_BIAS);
        let magic = self.b.ins().iconst(types::I64, SPLIT_MAGIC as i64);
        let q = self.b.ins().umulhi(t, magic);
        let q = self.b.ins().ushr_imm_s(q, SPLIT_SHIFT);
        let whole = self.b.ins().imul_imm_s(q, PAGE);
        let r = self.b.ins().isub(t, whole);
        let r = self.b.ins().iadd_imm_s(r, -(PAGE / 2));
        let q = self.b.ins().iadd_imm_s(q, -SPLIT_STEPS);
        (q, r)
    }

    /// Look up the page an access of `width` trytes at addr reaches.
    /// Ends in a branch to `fast`, where the host address of addr's
    /// tryte is the value returned, when the page is known and a word
    /// is aligned; otherwise to `slow`, which must make the access
    /// through the core. Leaves neither block current.
    fn find_page(
        &mut self,
        addr: Value,
        width: i64,
        store: bool,
    ) -> (Value, cranelift_codegen::ir::Block, cranelift_codegen::ir::Block) {
        let (vpage, offset) = self.split(addr);
        let table = if store { self.store_pages } else { self.load_pages };
        let index = self.b.ins().band_imm_s(vpage, PAGE_SLOTS as i64 - 1);
        let index = self.b.ins().imul_imm_s(index, SLOT_BYTES);
        let index =
            if self.ptr == types::I64 { index } else { self.b.ins().ireduce(self.ptr, index) };
        let slot = self.b.ins().iadd(table, index);
        let known = self.b.ins().load(types::I64, self.trusted, slot, 0);
        let hit = self.b.ins().icmp(IntCC::Equal, known, vpage);
        let hit = if width == 3 {
            // Aligned: addr, and so its offset, a multiple of 3; offset
            // + PAGE/2 + 2 is one too and never negative.
            let x = self.b.ins().iadd_imm_s(offset, PAGE / 2 + 2);
            let x = self.b.ins().imul_imm_s(x, INV3 as i64);
            let aligned =
                self.b.ins().icmp_imm_s(IntCC::UnsignedLessThanOrEqual, x, (u64::MAX / 3) as i64);
            self.b.ins().band(hit, aligned)
        } else {
            hit
        };
        let (fast, slow, found) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.b.ins().brif(hit, found, &[], slow, &[]);
        self.b.switch_to_block(found);
        let centre = self.b.ins().load(self.ptr, self.trusted, slot, SLOT_CENTRE);
        let bytes = self.b.ins().imul_imm_s(offset, 2);
        let bytes =
            if self.ptr == types::I64 { bytes } else { self.b.ins().ireduce(self.ptr, bytes) };
        let host = self.b.ins().iadd(centre, bytes);
        self.b.ins().jump(fast, &[]);
        (host, fast, slow)
    }

    /// One more instruction has run.
    fn count(&mut self) {
        let n = self.b.use_var(self.n);
        let n = self.b.ins().iadd_imm_s(n, 1);
        self.b.def_var(self.n, n);
    }

    /// Return the count, going on at pc.
    fn finish(&mut self, pc: Value) {
        let n = self.b.use_var(self.n);
        self.b.ins().jump(self.exit, &[BlockArg::Value(n), BlockArg::Value(pc)]);
    }

    /// If cond holds, return the count plus `extra`, going on at pc.
    fn exit_if(&mut self, cond: Value, extra: i64, pc: i64) {
        let on = self.b.create_block();
        let n = self.b.use_var(self.n);
        let n = if extra == 0 { n } else { self.b.ins().iadd_imm_s(n, extra) };
        let to = self.b.ins().iconst(types::I64, pc);
        let args = [BlockArg::Value(n), BlockArg::Value(to)];
        self.b.ins().brif(cond, self.exit, &args, on, &[]);
        self.b.switch_to_block(on);
    }
}

impl Accelerator for Jit {
    fn run(&mut self, hart: &mut Hart, bus: &mut Bus, budget: u64) -> u64 {
        let mut total = 0;
        loop {
            if bus.code_writes() != self.writes || self.blocks.len() >= BLOCKS_MAX {
                self.flush();
                self.writes = bus.code_writes();
            }
            let pc = hart.pc;
            let Some(physical) = hart.code_address(bus, pc) else { break };
            // Looking the code up may decode it, which marks it, but
            // never counts a code write.
            let key = (pc, physical);
            if !self.blocks.contains_key(&key) {
                let seen = self.visits.entry(key).or_insert(0);
                *seen += 1;
                if *seen < self.hot {
                    break;
                }
                self.visits.remove(&key);
                let code = Self::gather(hart, bus, pc);
                let block = self.compile(code, pc);
                self.blocks.insert(key, block);
            }
            let Some(Some(block)) = self.blocks.get(&key) else { break };
            let pages = self.pages.get(hart.jit_view(bus));
            let hart_ptr: *mut Hart = hart;
            let mut ctx = Ctx {
                // SAFETY: hart_ptr comes from a live &mut Hart.
                regs: unsafe { (*hart_ptr).regs_mut().as_mut_ptr() },
                hart: hart_ptr,
                bus,
                insns: block.insns.as_ptr(),
                next_pc: pc,
                budget: (budget - total) as i64,
                load_pages: pages.load.as_mut_ptr(),
                store_pages: pages.store.as_mut_ptr(),
            };
            // SAFETY: the function was compiled for a Ctx like this one,
            // and hart and bus are not otherwise used during the call.
            let ran = unsafe { (block.f)(&mut ctx) } as u64;
            hart.pc = ctx.next_pc;
            total += ran;
            // Stopped at an instruction it leaves to the interpreter, or
            // at the budget.
            if ran == 0 || total >= budget {
                break;
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ctx_offsets_match_the_layout() {
        assert_eq!(std::mem::offset_of!(super::Ctx, regs), super::CTX_REGS as usize);
        assert_eq!(std::mem::offset_of!(super::Ctx, next_pc), super::CTX_NEXT_PC as usize);
        assert_eq!(std::mem::offset_of!(super::Ctx, budget), super::CTX_BUDGET as usize);
        assert_eq!(std::mem::offset_of!(super::Ctx, load_pages), super::CTX_LOAD_PAGES as usize);
        assert_eq!(std::mem::offset_of!(super::Ctx, store_pages), super::CTX_STORE_PAGES as usize);
        assert_eq!(std::mem::size_of::<super::Slot>(), super::SLOT_BYTES as usize);
        assert_eq!(std::mem::offset_of!(super::Slot, centre), super::SLOT_CENTRE as usize);
    }

    /// The split compiled code makes, done the same way on the host.
    fn split(v: i64) -> (i64, i64) {
        use super::*;
        let t = (v + SPLIT_BIAS) as u64;
        let q = (((t as u128 * SPLIT_MAGIC as u128) >> 64) as u64 >> SPLIT_SHIFT) as i64;
        (q - SPLIT_STEPS, t as i64 - q * PAGE - PAGE / 2)
    }

    #[test]
    fn splitting_matches_low_and_split_high() {
        use detrytus_core::trits::{WORD_MAX, low, split_high};
        let page = super::PAGE;
        let mut values = vec![0, 1, -1, WORD_MAX, -WORD_MAX, WORD_MAX - 1, -WORD_MAX + 1];
        for k in [-3i64, -2, -1, 0, 1, 2, 3, 12345, -98765] {
            for d in -2..=2 {
                values.push(k * page + page / 2 + d);
                values.push(k * page - page / 2 + d);
            }
        }
        let mut x: i64 = 1958;
        for _ in 0..100_000 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            values.push((x >> 20) % WORD_MAX);
        }
        for v in values {
            assert_eq!(split(v), (split_high(v, 9), low(v, 9)), "{v}");
        }
    }

    /// The product compiled code makes, done the same way on the host.
    fn mul_low(a: i64, c: i64) -> i64 {
        use detrytus_core::trits::{WORD_MAX, WORD_MOD};
        let q = (a as f64 * c as f64 * (1.0 / WORD_MOD as f64)).round_ties_even() as i64;
        let mut r = a.wrapping_mul(c).wrapping_sub(q.wrapping_mul(WORD_MOD));
        if r > WORD_MAX {
            r -= WORD_MOD;
        }
        if r < -WORD_MAX {
            r += WORD_MOD;
        }
        r
    }

    #[test]
    fn multiplying_matches_mul_parts() {
        use detrytus_core::alu::mul_parts;
        use detrytus_core::trits::WORD_MAX;
        let edges = [0, 1, -1, 2, WORD_MAX, -WORD_MAX, WORD_MAX - 1, WORD_MAX / 2, 3i64.pow(14)];
        let mut pairs: Vec<(i64, i64)> = Vec::new();
        for &a in &edges {
            for &c in &edges {
                pairs.push((a, c));
                pairs.push((-a, c));
            }
        }
        let mut x: i64 = 1958;
        let mut next = || {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 20) % WORD_MAX
        };
        for _ in 0..200_000 {
            pairs.push((next(), next()));
        }
        for (a, c) in pairs {
            assert_eq!(mul_low(a, c), mul_parts(a, c).1, "{a} * {c}");
        }
    }

    #[test]
    fn the_alignment_test_finds_multiples_of_three() {
        use super::*;
        for offset in -PAGE / 2..=PAGE / 2 {
            let x = ((offset + PAGE / 2 + 2) as u64).wrapping_mul(INV3);
            assert_eq!(x <= u64::MAX / 3, offset % 3 == 0, "{offset}");
        }
    }
}
