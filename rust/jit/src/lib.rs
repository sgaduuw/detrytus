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
//! (`Hart::jit_compute`), as do loads and stores (`Hart::jit_load`,
//! `Hart::jit_store`), which refuse devices and anything that would
//! trap. A refusal ends the block before that instruction, and the
//! interpreter runs it.

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{AbiParam, BlockArg, InstBuilder, MemFlagsData, Type, Value, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};

use detrytus_core::accel::{self, Accelerator, Class, Insn};
use detrytus_core::bus::{Bus, page_of};
use detrytus_core::fastmap::FastMap;
use detrytus_core::hart::Hart;
use detrytus_core::isa::{self, Kind};
use detrytus_core::trits::{WORD_MAX, WORD_MOD, low, pow3, wrap};

/// The most instructions in one compiled function.
const REGION_MAX: usize = 256;

/// Visits before code at a pc is compiled.
const HOT: u32 = 1000;

/// Drop everything compiled past this many blocks, to bound memory.
const BLOCKS_MAX: usize = 1 << 16;

/// What a load helper returns when it refuses: no word is this small.
const REFUSED: i64 = i64::MIN;

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
}

const CTX_REGS: i32 = 0;
const CTX_NEXT_PC: i32 = 32;
const CTX_BUDGET: i32 = 40;

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

/// A load: the value, or REFUSED.
extern "C" fn helper_load(ctx: *mut Ctx, addr: i64, width: i64) -> i64 {
    // SAFETY: compiled code passes the Ctx `run` made, whose pointers
    // are live for the call; nothing else touches the hart or bus then.
    let ctx = unsafe { &mut *ctx };
    let (hart, bus) = unsafe { (&mut *ctx.hart, &mut *ctx.bus) };
    hart.jit_load(bus, addr, width as u32).unwrap_or(REFUSED)
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
        Some(()) => 0,
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
            let mut g = Gen { b, trusted, n, regs: cx, budget: cx, exit };
            g.regs = g.b.ins().load(ptr, trusted, cx, CTX_REGS);
            g.budget = g.b.ins().load(types::I64, trusted, cx, CTX_BUDGET);
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
                        let w = g.b.ins().iconst(types::I64, width);
                        let call = g.b.ins().call(load, &[cx, addr, w]);
                        let v = g.b.inst_results(call)[0];
                        let refused = g.b.ins().icmp_imm_s(IntCC::Equal, v, REFUSED);
                        g.exit_if(refused, 0, pc);
                        g.set(regs, d.rd, v);
                    }
                    Kind::St | Kind::Sw => {
                        let width = if d.op.kind == Kind::St { 1 } else { 3 };
                        let (a, value) = (g.reg(regs, d.rs1), g.reg(regs, d.rs2));
                        let addr = g.b.ins().iadd_imm_s(a, d.imm);
                        let addr = g.reduce(addr);
                        let w = g.b.ins().iconst(types::I64, width);
                        let call = g.b.ins().call(store, &[cx, addr, w, value]);
                        let status = g.b.inst_results(call)[0];
                        let refused = g.b.ins().icmp_imm_s(IntCC::Equal, status, 1);
                        g.exit_if(refused, 0, pc);
                        let changed = g.b.ins().icmp_imm_s(IntCC::Equal, status, 2);
                        g.exit_if(changed, 1, next);
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
    regs: Value,
    budget: Value,
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
            let hart_ptr: *mut Hart = hart;
            let mut ctx = Ctx {
                // SAFETY: hart_ptr comes from a live &mut Hart.
                regs: unsafe { (*hart_ptr).regs_mut().as_mut_ptr() },
                hart: hart_ptr,
                bus,
                insns: block.insns.as_ptr(),
                next_pc: pc,
                budget: (budget - total) as i64,
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
    }
}
