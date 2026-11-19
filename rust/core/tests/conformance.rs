//! Runs the conformance suite (conformance/, format in its README) on
//! the Rust core.
//!
//! Cases are read from the same TOML files as the Python runner; their
//! programs come from the images Python assembles into
//! conformance/build/, since there is no Rust assembler yet. The Python
//! runner validates the files' format, so this one trusts it.

use std::fs;
use std::path::{Path, PathBuf};

use detrytus_core::bus::Access;
use detrytus_core::image;
use detrytus_core::{Machine, Outcome};
use toml::{Table, Value};

const DEFAULT_RAM_PAGES: i64 = 729;
const DEFAULT_MAX_STEPS: i64 = 100_000;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance")
}

fn case_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() && path.file_name().unwrap() != "build" {
            case_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "toml") {
            out.push(path);
        }
    }
}

/// A case name as a file name, as `detrytus.conformance.slug` makes it.
fn slug(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut out = String::new();
    for word in lower.split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit()) {
        if !word.is_empty() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push_str(word);
        }
    }
    out
}

struct Case {
    id: String,
    image: PathBuf,
    ram_pages: i64,
    max_steps: i64,
    input: String,
    expect: Table,
}

fn load(file: &Path) -> Vec<Case> {
    let data: Table = fs::read_to_string(file).unwrap().parse().unwrap();
    let empty = Table::new();
    let defaults = data.get("defaults").and_then(Value::as_table).unwrap_or(&empty);
    let relative = file.strip_prefix(root()).unwrap().with_extension("");
    let relative = relative.to_str().unwrap().replace('\\', "/");
    let cases = data["case"].as_array().unwrap();
    cases
        .iter()
        .map(|raw| {
            let raw = raw.as_table().unwrap();
            let setting = |key: &str| raw.get(key).or_else(|| defaults.get(key));
            let name = raw["name"].as_str().unwrap();
            Case {
                id: format!("{relative}::{name}"),
                image: root().join("build").join(&relative).join(format!("{}.dti", slug(name))),
                ram_pages: setting("ram_pages")
                    .map_or(DEFAULT_RAM_PAGES, |v| v.as_integer().unwrap()),
                max_steps: setting("max_steps")
                    .map_or(DEFAULT_MAX_STEPS, |v| v.as_integer().unwrap()),
                input: setting("input").map_or("", |v| v.as_str().unwrap()).to_string(),
                expect: raw.get("expect").and_then(Value::as_table).cloned().unwrap_or_default(),
            }
        })
        .collect()
}

fn outcome_failure(expect: &Table, outcome: &Outcome) -> Option<String> {
    let int = |key: &str| expect[key].as_integer().unwrap();
    let ok = if expect.contains_key("exit_code") {
        matches!(outcome, Outcome::Exit { code, .. } if *code == int("exit_code"))
    } else if expect.contains_key("step_limit") {
        matches!(outcome, Outcome::StepLimit)
    } else {
        matches!(outcome, Outcome::Unhandled { cause, .. } if *cause == int("unhandled_cause"))
    };
    (!ok).then(|| format!("outcome: wanted {expect:?}, got {outcome:?}"))
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
    let data = fs::read(&case.image).unwrap_or_else(|e| {
        panic!("{}: {e}; run `uv run python -m detrytus.conformance`", case.image.display())
    });
    let mut machine = Machine::new(case.ram_pages, 1);
    machine.load(&image::parse(&data).unwrap());
    machine.bus.console.feed(&case.input);
    let outcome = machine.run(case.max_steps as u64);
    let e = &case.expect;
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
    assert!(cases.len() > 80, "found only {} cases", cases.len());
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

#[test]
fn slugs_match_the_python_runner() {
    assert_eq!(slug("a load needs no dirty trit"), "a-load-needs-no-dirty-trit");
    assert_eq!(slug("ring +1 is never translated"), "ring-1-is-never-translated");
    assert_eq!(slug("  Odd -- spacing!  "), "odd-spacing");
}
