//! Runs the conformance suite (conformance/, format in its README) on
//! the Rust assembler and core.
//!
//! Cases are TOML files under conformance/ (format in its README); this
//! runner checks the format strictly, so a misspelt key or a case
//! without an outcome fails instead of passing unchecked, then
//! assembles each case and runs it.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use detrytus_asm::{AsmError, assemble, assemble_object};
use detrytus_core::block::MemoryDisk;
use detrytus_core::bus::Access;
use detrytus_core::image;
use detrytus_core::{Hotplug, Machine, Outcome};
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

/// What a case runs: assembly text, files to assemble or load, or
/// texts to assemble as objects and link.
enum Program {
    Text(String),
    Sources(Vec<PathBuf>),
    Objects(Vec<String>),
}

struct Case {
    id: String,
    file: PathBuf,
    name: String,
    program: Program,
    ram_pages: i64,
    max_steps: i64,
    input: String,
    console_open: bool,
    disk_blocks: Option<i64>,
    harts: i64,
    hotplug: Vec<(i64, Hotplug)>,
    expect: Table,
}

/// A case's setting, or its file's default.
fn setting<'a>(raw: &'a Table, defaults: &'a Table, key: &str) -> Option<&'a Value> {
    raw.get(key).or_else(|| defaults.get(key))
}

fn text_of(raw: &Table, defaults: &Table, key: &str) -> String {
    setting(raw, defaults, key).and_then(Value::as_str).unwrap_or("").to_string()
}

const CASE_KEYS: [&str; 14] = [
    "name",
    "program",
    "objects",
    "prelude",
    "postlude",
    "source",
    "sources",
    "ram_pages",
    "max_steps",
    "input",
    "console_open",
    "disk_blocks",
    "harts",
    "hotplug",
];
const OUTCOMES: [&str; 5] =
    ["exit_code", "step_limit", "unhandled_cause", "asm_error", "link_error"];
const CHECKS: [&str; 7] =
    ["console", "registers", "csrs", "ring", "words", "trytes", "unhandled_hart"];

fn unknown_keys(where_: &str, table: &Table, allowed: &[&str]) -> Result<(), String> {
    let mut unknown: Vec<&String> =
        table.keys().filter(|k| !allowed.contains(&k.as_str())).collect();
    if unknown.is_empty() {
        return Ok(());
    }
    unknown.sort();
    Err(format!("{where_}: unknown keys {unknown:?}"))
}

/// The format's rules, which the Python runner used to enforce.
fn check_format(path: &str, data: &Table) -> Result<(), String> {
    unknown_keys(path, data, &["defaults", "case"])?;
    let empty = Table::new();
    let defaults = data.get("defaults").and_then(Value::as_table).unwrap_or(&empty);
    let default_keys: Vec<&str> = CASE_KEYS.iter().copied().filter(|k| *k != "name").collect();
    unknown_keys(&format!("{path} [defaults]"), defaults, &default_keys)?;
    let cases = data.get("case").and_then(Value::as_array).filter(|c| !c.is_empty());
    let cases = cases.ok_or(format!("{path}: no cases"))?;
    let mut names = std::collections::HashSet::new();
    for raw in cases {
        let raw = raw.as_table().ok_or(format!("{path}: a case is not a table"))?;
        let name = raw.get("name").and_then(Value::as_str).filter(|n| !n.contains('/'));
        let name = name.ok_or(format!("{path}: every case needs a name, without '/'"))?;
        let where_ = format!("{path}: case {name:?}");
        if !names.insert(name) {
            return Err(format!("{path}: case names repeat"));
        }
        let mut keys: Vec<&str> = CASE_KEYS.to_vec();
        keys.push("expect");
        unknown_keys(&where_, raw, &keys)?;
        let expect = raw.get("expect").and_then(Value::as_table).unwrap_or(&empty);
        if expect.contains_key("asm_error") {
            unknown_keys(&format!("{where_} [expect]"), expect, &["asm_error", "asm_error_line"])?;
        } else if expect.contains_key("link_error") {
            unknown_keys(&format!("{where_} [expect]"), expect, &["link_error"])?;
        } else {
            let allowed: Vec<&str> = OUTCOMES.iter().chain(&CHECKS).copied().collect();
            unknown_keys(&format!("{where_} [expect]"), expect, &allowed)?;
        }
        if OUTCOMES.iter().filter(|o| expect.contains_key(**o)).count() != 1 {
            return Err(format!("{where_}: give exactly one of {OUTCOMES:?}"));
        }
        let given = ["program", "source", "sources", "objects"]
            .iter()
            .filter(|k| setting(raw, defaults, k).is_some())
            .count();
        if given != 1 {
            return Err(format!("{where_}: give exactly one of program, source, sources, objects"));
        }
        if expect.contains_key("unhandled_hart") && !expect.contains_key("unhandled_cause") {
            return Err(format!("{where_}: unhandled_hart needs unhandled_cause"));
        }
        if let Some(plugs) = setting(raw, defaults, "hotplug") {
            hotplug(plugs).map_err(|e| format!("{where_}: {e}"))?;
        }
    }
    Ok(())
}

/// `hotplug = [{ time = 5, add = 1 }, { time = 9, remove = 1 }]`
fn hotplug(value: &Value) -> Result<Vec<(i64, Hotplug)>, String> {
    let bad =
        || "hotplug entries are { time = T, add = H } or { time = T, remove = H }".to_string();
    let entries = value.as_array().ok_or_else(bad)?;
    entries
        .iter()
        .map(|e| {
            let e = e.as_table().ok_or_else(bad)?;
            unknown_keys("hotplug", e, &["time", "add", "remove"])?;
            let time = e.get("time").and_then(Value::as_integer).ok_or_else(bad)?;
            match (e.get("add"), e.get("remove")) {
                (Some(h), None) => Ok((time, Hotplug::Add(h.as_integer().ok_or_else(bad)?))),
                (None, Some(h)) => Ok((time, Hotplug::Remove(h.as_integer().ok_or_else(bad)?))),
                _ => Err(bad()),
            }
        })
        .collect()
}

fn load(file: &Path) -> Vec<Case> {
    let data: Table = fs::read_to_string(file).unwrap().parse().unwrap();
    if let Err(problem) = check_format(&file.display().to_string(), &data) {
        panic!("{problem}");
    }
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
            } else if let Some(objects) = setting("objects") {
                let texts = objects.as_array().unwrap().iter();
                Program::Objects(texts.map(|v| v.as_str().unwrap().to_string()).collect())
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
                console_open: setting("console_open").is_some_and(|v| v.as_bool().unwrap()),
                disk_blocks: setting("disk_blocks").map(|v| v.as_integer().unwrap()),
                harts: setting("harts").map_or(1, |v| v.as_integer().unwrap()),
                hotplug: setting("hotplug").map_or_else(Vec::new, |v| hotplug(v).unwrap()),
                expect: raw.get("expect").and_then(Value::as_table).cloned().unwrap_or_default(),
            }
        })
        .collect()
}

/// Why a case did not build.
enum Failed {
    Asm(AsmError),
    Link(String),
    /// Another problem: a missing file, overlapping images.
    Other(String),
}

/// The case's trytes, or why it did not build.
fn build(case: &Case) -> Result<Vec<(i64, i64)>, Failed> {
    match &case.program {
        // Named after the case and their place in its list.
        Program::Objects(texts) => {
            let mut objects = Vec::new();
            for (k, text) in texts.iter().enumerate() {
                let file = case.id.split("::").next().unwrap_or_default();
                let name = format!("{file}.toml [{}] #{}", case.name, k + 1);
                let (object, _) = assemble_object(text, Some(&name)).map_err(Failed::Asm)?;
                objects.push((format!("#{}", k + 1), object));
            }
            let linked = detrytus_asm::link::link(&objects).map_err(Failed::Link)?;
            Ok(linked.image.trytes.into_iter().collect())
        }
        // Named after the case, beside its file, as the Python runner does.
        Program::Text(text) => {
            let name = format!("{} [{}]", case.file.display(), case.name);
            let image = assemble(text, Some(&name)).map_err(Failed::Asm)?;
            Ok(image.trytes.into_iter().collect())
        }
        Program::Sources(paths) => {
            let mut trytes = Vec::new();
            let mut used = HashSet::new();
            for path in paths {
                let data = fs::read(path)
                    .map_err(|e| Failed::Other(format!("{}: {e}", path.display())))?;
                let these: Vec<(i64, i64)> = if data.starts_with(b"DTI\x01") {
                    let segments = image::parse(&data).map_err(Failed::Other)?;
                    segments
                        .iter()
                        .flat_map(|s| (s.start..).zip(s.trytes.iter().copied()))
                        .collect()
                } else {
                    let text = String::from_utf8(data).map_err(|e| Failed::Other(e.to_string()))?;
                    let shown = path.display().to_string();
                    assemble(&text, Some(&shown)).map_err(Failed::Asm)?.trytes.into_iter().collect()
                };
                if let Some((addr, _)) = these.iter().find(|(a, _)| !used.insert(*a)) {
                    return Err(Failed::Other(format!("images overlap at address {addr}")));
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
        let want_hart = expect.get("unhandled_hart").and_then(Value::as_integer).unwrap_or(0);
        matches!(outcome, Outcome::Unhandled { cause, hart, .. }
            if *cause == int("unhandled_cause") && *hart == want_hart)
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
        let got = machine.boot_hart_mut().csr(&name).unwrap_or_else(|| panic!("no CSR {name}"));
        if got != value {
            failures.push(format!("csr {name}: wanted {value}, got {got}"));
        }
    }
    for (name, value) in entries("registers") {
        let got = machine.boot_hart().reg(&name).unwrap_or_else(|| panic!("no register {name}"));
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
        Ok(_) if e.contains_key("link_error") => {
            return vec![format!("linked, but expected link_error {:?}", e["link_error"])];
        }
        Ok(trytes) => trytes,
        Err(Failed::Asm(error)) => return assembly_failures(e, &error),
        Err(Failed::Link(problem)) => {
            return match e.get("link_error").and_then(Value::as_str) {
                Some(want) if want == problem => vec![],
                Some(want) => vec![format!("link_error: wanted {want:?}, got {problem:?}")],
                None => vec![format!("linking failed: {problem}")],
            };
        }
        Err(Failed::Other(problem)) => return vec![problem],
    };
    let mut machine = Machine::with_harts(case.ram_pages, 1, case.harts);
    for &(time, change) in &case.hotplug {
        if let Err(problem) = machine.schedule(time, change) {
            return vec![problem];
        }
    }
    for (addr, tryte) in trytes {
        machine.bus.poke(addr, tryte);
    }
    machine.bus.console.feed(&case.input);
    machine.bus.console.closed = !case.console_open;
    if let Some(blocks) = case.disk_blocks.filter(|&b| b > 0) {
        machine.bus.block.disk = Some(Box::new(MemoryDisk::new(blocks)));
    }
    let outcome = machine.run(case.max_steps as u64);
    let mut failures: Vec<String> = outcome_failure(e, &outcome).into_iter().collect();
    if let Some(want) = e.get("console").and_then(Value::as_str) {
        let got = machine.bus.console.text();
        if got != want {
            failures.push(format!("console: wanted {want:?}, got {got:?}"));
        }
    }
    if let Some(want) = e.get("ring").and_then(Value::as_integer)
        && machine.boot_hart().ring != want
    {
        failures.push(format!("ring: wanted {want}, got {}", machine.boot_hart().ring));
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

// --- the format checks themselves ------------------------------------------

fn format_problem(text: &str) -> Option<String> {
    check_format("cases.toml", &text.parse::<Table>().unwrap()).err()
}

const GOOD: &str = "[[case]]\nname = \"x\"\nprogram = \"nop\"\n";

#[test]
fn a_well_formed_case_passes_the_format_check() {
    assert_eq!(format_problem(&format!("{GOOD}expect.exit_code = 0\n")), None);
    let asm = "[[case]]\nname = \"x\"\nprogram = \"frob\"\nexpect.asm_error = \"e\"\nexpect.asm_error_line = 1\n";
    assert_eq!(format_problem(asm), None);
}

#[test]
fn malformed_case_files_are_refused() {
    let cases = [
        (format!("{GOOD}expect.exit_code = 0\nspeed = 1\n"), "unknown keys [\"speed\"]"),
        (format!("{GOOD}expect.exit_cod = 0\n"), "unknown keys [\"exit_cod\"]"),
        (GOOD.to_string(), "exactly one of [\"exit_code\""),
        (
            format!("{GOOD}expect.exit_code = 0\nexpect.step_limit = true\n"),
            "exactly one of [\"exit_code\"",
        ),
        (
            format!("{GOOD}source = \"a.s\"\nexpect.exit_code = 0\n"),
            "exactly one of program, source",
        ),
        ("[[case]]\nprogram = \"nop\"\nexpect.exit_code = 0\n".to_string(), "needs a name"),
        ("[defaults]\nram_pages = 3\n".to_string(), "no cases"),
        (
            format!("[defaults]\nname = \"x\"\n{GOOD}expect.exit_code = 0\n"),
            "[defaults]: unknown keys",
        ),
        (format!("{GOOD}expect.exit_code = 0\n{GOOD}expect.exit_code = 0\n"), "case names repeat"),
        (format!("{GOOD}expect.asm_error = \"e\"\nexpect.ring = 1\n"), "unknown keys [\"ring\"]"),
        (
            format!("{GOOD}expect.exit_code = 0\nexpect.unhandled_hart = 1\n"),
            "unhandled_hart needs unhandled_cause",
        ),
        (
            format!(
                "{GOOD}expect.exit_code = 0\nhotplug = [{{ time = 1, add = 1, remove = 2 }}]\n"
            ),
            "hotplug entries are",
        ),
        (
            format!("{GOOD}expect.exit_code = 0\nhotplug = [{{ time = 1, plug = 1 }}]\n"),
            "unknown keys [\"plug\"]",
        ),
    ];
    for (text, message) in cases {
        let problem = format_problem(&text).unwrap_or_else(|| panic!("accepted:\n{text}"));
        assert!(problem.contains(message), "{problem:?} lacks {message:?}");
    }
}
