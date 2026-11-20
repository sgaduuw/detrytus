//! The `detrytus` command's output, held to recorded transcripts.
//!
//! Each file in tests/golden/ lists cases: arguments, optional stdin and
//! debugger script, and the exact stdout, stderr and exit status. They
//! were recorded when the Python tools were retired (docs/rust-plan.md,
//! phase 6), from inputs on which the Python and Rust commands agreed
//! character for character, so they carry that agreement forward.
//!
//! A case may also name a `file` the command writes, whose bytes must be
//! `file_hex`. In arguments and outputs, {root} is the repository,
//! {here} this directory and {tmp} a scratch directory for the case. To accept new
//! output after a deliberate change, run with DETRYTUS_UPDATE_GOLDEN=1
//! and review the diff.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use toml::{Table, Value};

fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").canonicalize().unwrap()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

struct Places {
    root: String,
    here: String,
    tmp: String,
}

impl Places {
    fn fill(&self, text: &str) -> String {
        text.replace("{here}", &self.here).replace("{root}", &self.root).replace("{tmp}", &self.tmp)
    }

    /// Paths back to placeholders, the most specific first.
    fn generalise(&self, text: &str) -> String {
        text.replace(&self.tmp, "{tmp}").replace(&self.here, "{here}").replace(&self.root, "{root}")
    }
}

fn strings(case: &Table, key: &str) -> Vec<String> {
    case.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Run one case; its actual (status, stdout, stderr), placeholders back
/// in, and the hex of the file it names, if it names one.
fn run(case: &Table, places: &Places) -> (i64, String, String, Option<String>) {
    let binary = env!("CARGO_BIN_EXE_detrytus");
    // Steps that build what the case needs, such as an image.
    for setup in case.get("setup").and_then(Value::as_array).into_iter().flatten() {
        let args: Vec<String> =
            setup.as_array().unwrap().iter().map(|v| places.fill(v.as_str().unwrap())).collect();
        let done = Command::new(binary).args(&args).current_dir(&places.root).output().unwrap();
        assert!(done.status.success(), "setup {args:?}: {}", String::from_utf8_lossy(&done.stderr));
    }
    if let Some(script) = case.get("script").and_then(Value::as_str) {
        fs::write(Path::new(&places.tmp).join("script.txt"), script).unwrap();
    }
    let args: Vec<String> = strings(case, "args").iter().map(|a| places.fill(a)).collect();
    let cwd =
        case.get("cwd").and_then(Value::as_str).map_or(places.root.clone(), |c| places.fill(c));
    let mut child = Command::new(binary)
        .args(&args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = case.get("stdin").and_then(Value::as_str).unwrap_or("").to_string();
    let mut pipe = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        use std::io::Write;
        let _ = pipe.write_all(stdin.as_bytes());
    });
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    let status = i64::from(out.status.code().unwrap_or(-1));
    let text = |b: &[u8]| places.generalise(&String::from_utf8_lossy(b));
    let file = case.get("file").and_then(Value::as_str).map(|f| {
        fs::read(places.fill(f)).map(|b| hex(&b)).unwrap_or_else(|e| format!("unreadable: {e}"))
    });
    (status, text(&out.stdout), text(&out.stderr), file)
}

#[test]
fn transcripts_match() {
    let update = std::env::var_os("DETRYTUS_UPDATE_GOLDEN").is_some();
    let mut files: Vec<PathBuf> = fs::read_dir(here())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    let (mut count, mut failures) = (0, Vec::new());
    for path in files {
        let mut data: Table = fs::read_to_string(&path).unwrap().parse().unwrap();
        let mut changed = false;
        for case in data.get_mut("case").and_then(Value::as_array_mut).unwrap() {
            let case = case.as_table_mut().unwrap();
            let tmp = tempdir(&format!("{}-{count}", path.file_stem().unwrap().to_string_lossy()));
            let places = Places {
                root: root().display().to_string(),
                here: here().display().to_string(),
                tmp: tmp.display().to_string(),
            };
            let (status, stdout, stderr, file) = run(case, &places);
            let _ = fs::remove_dir_all(&tmp);
            count += 1;
            let want = (
                case["status"].as_integer().unwrap(),
                case["stdout"].as_str().unwrap().to_string(),
                case["stderr"].as_str().unwrap().to_string(),
            );
            let want_file = case.get("file_hex").and_then(Value::as_str).map(str::to_string);
            if want == (status, stdout.clone(), stderr.clone()) && want_file == file {
                continue;
            }
            if update {
                case.insert("status".into(), Value::Integer(status));
                case.insert("stdout".into(), Value::String(stdout));
                case.insert("stderr".into(), Value::String(stderr));
                if let Some(file) = file {
                    case.insert("file_hex".into(), Value::String(file));
                }
                changed = true;
                continue;
            }
            let name = case["name"].as_str().unwrap();
            failures.push(format!(
                "{} :: {name}\n  status {} -> {status}\n  stdout {:?}\n      -> {stdout:?}\n  stderr {:?}\n      -> {stderr:?}\n  file {want_file:?}\n      -> {file:?}",
                path.display(),
                want.0,
                want.1,
                want.2
            ));
        }
        if changed {
            fs::write(&path, toml::to_string(&data).unwrap()).unwrap();
        }
    }
    assert!(count > 100, "found only {count} cases");
    assert!(
        failures.is_empty(),
        "{} of {count} transcripts differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("detrytus-golden-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}
