//! Python bindings for the Rust core: the optional `detrytus-rs`
//! package, module `detrytus_rs`. Scaffolding while the Python tools
//! and tests still exist (docs/rust-plan.md, phase 4); removed at the
//! switchover. `detrytus/rust.py` wraps this in the Python core's
//! interface, so most code never imports it directly.

use std::collections::HashMap;

use detrytus_core::bus::Access;
use detrytus_core::{Machine as Core, Outcome};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// How a run ended: (kind, numbers, message). kind is "running",
/// "exit" (code, steps), "step_limit", "unhandled" (cause, tval, pc,
/// ring) or "unsupported" (message).
type PyOutcome = (&'static str, Vec<i64>, String);

fn outcome(o: Option<Outcome>) -> PyOutcome {
    match o {
        None => ("running", vec![], String::new()),
        Some(Outcome::Exit { code, steps }) => ("exit", vec![code, steps as i64], String::new()),
        Some(Outcome::StepLimit) => ("step_limit", vec![], String::new()),
        Some(Outcome::Unhandled { cause, tval, pc, ring }) => {
            ("unhandled", vec![cause, tval, pc, ring], String::new())
        }
        Some(Outcome::Unsupported(what)) => ("unsupported", vec![], what),
    }
}

/// A detrytus machine on the Rust core.
#[pyclass(unsendable)]
struct Machine {
    core: Core,
}

#[pymethods]
impl Machine {
    #[new]
    fn new(ram_pages: i64, seed: u32) -> Self {
        Machine { core: Core::new(ram_pages, seed) }
    }

    /// Write trytes without access checks, as a loader does.
    fn load(&mut self, trytes: HashMap<i64, i64>) {
        for (addr, tryte) in trytes {
            self.core.bus.poke(addr, tryte);
        }
    }

    fn run(&mut self, max_steps: u64) -> PyOutcome {
        outcome(Some(self.core.run(max_steps)))
    }

    fn step(&mut self) -> PyOutcome {
        outcome(self.core.step())
    }

    // --- the hart ----------------------------------------------------------

    fn x(&self, r: i64) -> PyResult<i64> {
        if !(-13..=13).contains(&r) {
            return Err(PyValueError::new_err(format!("no register r{r}")));
        }
        Ok(self.core.hart.x(r))
    }

    fn regs(&self) -> Vec<i64> {
        self.core.hart.regs().to_vec()
    }

    fn csr(&mut self, name: &str) -> Option<i64> {
        self.core.hart.csr(name)
    }

    fn bank(&self, ring: i64) -> PyResult<Vec<i64>> {
        if ring != 0 && ring != 1 {
            return Err(PyValueError::new_err("only rings 0 and +1 have banks"));
        }
        Ok(self.core.hart.bank(ring).to_vec())
    }

    fn stored_csrs(&self) -> Vec<(i64, i64)> {
        self.core.hart.stored_csrs()
    }

    #[getter]
    fn pc(&self) -> i64 {
        self.core.hart.pc
    }

    #[getter]
    fn ring(&self) -> i64 {
        self.core.hart.ring
    }

    #[getter]
    fn instret(&self) -> i64 {
        self.core.hart.instret
    }

    #[getter]
    fn time(&self) -> i64 {
        self.core.hart.time
    }

    #[getter]
    fn reservation(&self) -> Option<i64> {
        self.core.hart.reservation()
    }

    #[getter]
    fn step_ring(&self) -> Option<i64> {
        self.core.hart.step_ring()
    }

    // --- the bus -----------------------------------------------------------

    /// Read physical memory; kind is "fetch", "load", "store" or
    /// "walk". None where the access faults.
    fn read(&mut self, addr: i64, width: u32, kind: &str) -> PyResult<Option<i64>> {
        let access = match kind {
            "fetch" => Access::Fetch,
            "load" => Access::Load,
            "store" => Access::Store,
            "walk" => Access::Walk,
            _ => return Err(PyValueError::new_err(format!("unknown access kind {kind:?}"))),
        };
        if width != 1 && width != 3 {
            return Err(PyValueError::new_err("width is 1 (tryte) or 3 (word)"));
        }
        Ok(self.core.bus.read(addr, width, access).ok())
    }

    /// Every non-zero tryte of memory, in address order.
    fn trytes(&self) -> Vec<(i64, i64)> {
        self.core.bus.trytes()
    }

    fn console_text(&self) -> String {
        self.core.bus.console.text()
    }

    fn console_feed(&mut self, text: &str) {
        self.core.bus.console.feed(text);
    }
}

#[pymodule]
fn detrytus_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Machine>()
}
