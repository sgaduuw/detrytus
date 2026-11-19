//! Runs the conformance suite (conformance/, format in its README) on
//! the Rust assembler and core.
//!
//! Cases are read from the same TOML files as the Python runner and
//! assembled here. The Python runner validates the files' format, so
//! this one trusts it.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use detrytus_asm::{AsmError, assemble};
use detrytus_core::bus::Access;
use detrytus_core::image;
use detrytus_core::{Machine, Outcome};
use toml::{Table, Value};

const DEFAULT_RAM_PAGES: i64 = 729;
const DEFAULT_MAX_STEPS: i64 = 100_000;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance").canonicalize().unwrap()
}

fn case_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            case_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "toml") {
            out.push(path);
        }
    }
}

/// What a case runs: assembly text, or files to assemble or load.
enum Program {
    Text(String),
    Sources(Vec<PathBuf>),
}

struct Case {
    id: String,
    file: PathBuf,
    name: String,
    program: Program,
    ram_pages: i64,
    max_steps: i64,
    input: String,
    expect: Table,
}

/// A case's setting, or its file's default.
fn setting<'a>(raw: &'a Table, defaults: &'a Table, key: &str) -> Option<&'a Value> {
    raw.get(key).or_else(|| defaults.get(key))
}

fn text_of(raw: &Table, defaults: &Table, key: &str) -> String {
    setting(raw, defaults, key).and_then(Value::as_str).unwrap_or("").to_string()
}

fn load(file: &Path) -> Vec<Case> {
    let data: Table = fs::read_to_string(file).unwrap().parse().unwrap();
    let empty = Table::new();
    let defaults = data.get("defaults").and_then(Value::as_table).unwrap_or(&empty);
    let relative = file.strip_prefix(root()).unwrap().with_extension("");
    let relative = relative.to_str().unwrap().replace('\\', "/");
    let dir = file.parent().unwrap();
    data["case"]
        .as_array()
        .unwrap()
        .iter()
        .map(|raw| {
            let raw = raw.as_table().unwrap();
            let setting = |key: &str| setting(raw, defaults, key);
            let name = raw["name"].as_str().unwrap().to_string();
            let program = if setting("program").is_some() {
                let parts = [
                    text_of(raw, defaults, "prelude"),
                    text_of(raw, defaults, "program"),
                    text_of(raw, defaults, "postlude"),
                ];
                let parts: Vec<String> = parts.into_iter().filter(|p| !p.is_empty()).collect();
                Program::Text(parts.join("\n"))
            } else {
                let names: Vec<&str> = match setting("source") {
                    Some(v) => vec![v.as_str().unwrap()],
                    None => setting("sources")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect(),
                };
                Program::Sources(names.iter().map(|n| dir.join(n)).collect())
            };
            Case {
                id: format!("{relative}::{name}"),
                file: file.to_path_buf(),
                name,
                program,
                ram_pages: setting("ram_pages")
                    .map_or(DEFAULT_RAM_PAGES, |v| v.as_integer().unwrap()),
                max_steps: setting("max_steps")
                    .map_or(DEFAULT_MAX_STEPS, |v| v.as_integer().unwrap()),
                input: text_of(raw, defaults, "input"),
                expect: raw.get("expect").and_then(Value::as_table).cloned().unwrap_or_default(),
            }
        })
        .collect()
}

/// The case's trytes, or why it did not build: an assembly error, or
/// another problem (a missing file, overlapping images).
fn build(case: &Case) -> Result<Vec<(i64, i64)>, Result<AsmError, String>> {
    match &case.program {
        // Named after the case, beside its file, as the Python runner does.
        Program::Text(text) => {
            let name = format!("{} [{}]", case.file.display(), case.name);
            let image = assemble(text, Some(&name)).map_err(Ok)?;
            Ok(image.trytes.into_iter().collect())
        }
        Program::Sources(paths) => {
            let mut trytes = Vec::new();
            let mut used = HashSet::new();
            for path in paths {
                let data = fs::read(path).map_err(|e| Err(format!("{}: {e}", path.display())))?;
                let these: Vec<(i64, i64)> = if data.starts_with(b"DTI\x01") {
                    let segments = image::parse(&data).map_err(Err)?;
                    segments
                        .iter()
                        .flat_map(|s| (s.start..).zip(s.trytes.iter().copied()))
                        .collect()
                } else {
                    let text = String::from_utf8(data).map_err(|e| Err(e.to_string()))?;
                    let shown = path.display().to_string();
                    assemble(&text, Some(&shown)).map_err(Ok)?.trytes.into_iter().collect()
                };
                if let Some((addr, _)) = these.iter().find(|(a, _)| !used.insert(*a)) {
                    return Err(Err(format!("images overlap at address {addr}")));
                }
                trytes.extend(these);
            }
            Ok(trytes)
        }
    }
}

fn outcome_failure(expect: &Table, outcome: &Outcome) -> Option<String> {
    let int = |key: &str| expect[key].as_integer().unwrap();
    let ok = if expect.contains_key("exit_code") {
        matches!(outcome, Outcome::Exit { code, .. } if *code == int("exit_code"))
    } else if expect.contains_key("step_limit") {
        matches!(outcome, Outcome::StepLimit)
    } else if expect.contains_key("unhandled_cause") {
        matches!(outcome, Outcome::Unhandled { cause, .. } if *cause == int("unhandled_cause"))
    } else {
        false // expected an assembly error
    };
    (!ok).then(|| format!("outcome: wanted {expect:?}, got {outcome:?}"))
}

fn assembly_failures(expect: &Table, error: &AsmError) -> Vec<String> {
    let mut failures = Vec::new();
    match expect.get("asm_error").and_then(Value::as_str) {
        Some(want) if want == error.message => {}
        Some(want) => failures.push(format!("asm_error: wanted {want:?}, got {:?}", error.message)),
        None => failures.push(format!("assembly failed: {error}")),
    }
    if let Some(line) = expect.get("asm_error_line").and_then(Value::as_integer)
        && error.line != Some(line as usize)
    {
        failures.push(format!("asm_error_line: wanted {line}, got {:?}", error.line));
    }
    failures
}

fn named_checks(machine: &mut Machine, expect: &Table, failures: &mut Vec<String>) {
    let entries = |key: &str| -> Vec<(String, i64)> {
        expect.get(key).and_then(Value::as_table).map_or_else(Vec::new, |t| {
            t.iter().map(|(k, v)| (k.clone(), v.as_integer().unwrap())).collect()
        })
    };
    for (name, value) in entries("csrs") {
        let got = machine.hart.csr(&name).unwrap_or_else(|| panic!("no CSR {name}"));
        if got != value {
            failures.push(format!("csr {name}: wanted {value}, got {got}"));
        }
    }
    for (name, value) in entries("registers") {
        let got = machine.hart.reg(&name).unwrap_or_else(|| panic!("no register {name}"));
        if got != value {
            failures.push(format!("register {name}: wanted {value}, got {got}"));
        }
    }
    for (key, width) in [("words", 3), ("trytes", 1)] {
        for (addr, value) in entries(key) {
            let got = machine.bus.read(addr.parse().unwrap(), width, Access::Walk);
            if got != Ok(value) {
                failures.push(format!("{key} at {addr}: wanted {value}, got {got:?}"));
            }
        }
    }
}

/// Run a case; the failures, empty if it passed.
fn run(case: &Case) -> Vec<String> {
    let e = &case.expect;
    let trytes = match build(case) {
        Ok(_) if e.contains_key("asm_error") => {
            return vec![format!("assembled, but expected asm_error {:?}", e["asm_error"])];
        }
        Ok(trytes) => trytes,
        Err(Ok(error)) => return assembly_failures(e, &error),
        Err(Err(problem)) => return vec![problem],
    };
    let mut machine = Machine::new(case.ram_pages, 1);
    for (addr, tryte) in trytes {
        machine.bus.poke(addr, tryte);
    }
    machine.bus.console.feed(&case.input);
    let outcome = machine.run(case.max_steps as u64);
    let mut failures: Vec<String> = outcome_failure(e, &outcome).into_iter().collect();
    if let Some(want) = e.get("console").and_then(Value::as_str) {
        let got = machine.bus.console.text();
        if got != want {
            failures.push(format!("console: wanted {want:?}, got {got:?}"));
        }
    }
    if let Some(want) = e.get("ring").and_then(Value::as_integer)
        && machine.hart.ring != want
    {
        failures.push(format!("ring: wanted {want}, got {}", machine.hart.ring));
    }
    named_checks(&mut machine, e, &mut failures);
    failures
}

#[test]
fn conformance_suite() {
    let mut files = Vec::new();
    case_files(&root(), &mut files);
    let cases: Vec<Case> = files.iter().flat_map(|f| load(f)).collect();
    assert!(cases.len() > 100, "found only {} cases", cases.len());
    let mut problems = Vec::new();
    for case in &cases {
        let failures = run(case);
        if !failures.is_empty() {
            problems.push(format!("{}:\n    {}", case.id, failures.join("\n    ")));
        }
    }
    println!("{} passed, {} failed", cases.len() - problems.len(), problems.len());
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
