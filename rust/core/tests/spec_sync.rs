//! docs/isa.md and the Rust instruction table must agree, as
//! tests/test_spec_sync.py checks for the Python table: the spec is
//! prose a reader trusts, the table is what runs.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use detrytus_core::isa::OPS;

fn spec() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/isa.md");
    fs::read_to_string(path).unwrap()
}

/// Rows like "| 34 | `addi rd, rs1, imm` | ..." in section 7, before the
/// pseudo-instructions. The spec writes negatives with a true minus.
fn spec_opcodes(text: &str) -> BTreeMap<String, i64> {
    let start = text.find("## 7. Instructions").unwrap();
    let end = text.find("### 7.13").unwrap();
    let mut found = BTreeMap::new();
    for line in text[start..end].lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 3 || !cells[0].is_empty() {
            continue;
        }
        let Ok(opcode) = cells[1].replace('\u{2212}', "-").parse::<i64>() else {
            continue;
        };
        let Some(rest) =
            cells[2].strip_prefix('`').filter(|r| r.starts_with(|c: char| c.is_ascii_lowercase()))
        else {
            continue;
        };
        let name: String =
            rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '.').collect();
        // The name must end the code span or be followed by operands;
        // this skips b<cond>, whose name the table cannot carry.
        if !matches!(rest[name.len()..].chars().next(), Some(' ' | '`')) {
            continue;
        }
        found.insert(name, opcode);
    }
    found
}

#[test]
fn every_spec_instruction_matches_the_table() {
    let mut spec = spec_opcodes(&spec());
    // The branch-condition table has rows like "| 0 | `beq` |".
    spec.retain(|name, _| !["beq", "bne", "bgt", "blt", "bge", "ble"].contains(&name.as_str()));
    // Floating point is specified but not implemented yet.
    spec.retain(|name, v| !(name.starts_with('f') && (40..=54).contains(v)));
    assert!(spec.len() > 50, "the spec parser found too few rows: {}", spec.len());
    let table: BTreeMap<String, i64> =
        OPS.iter().filter(|op| op.name != "b").map(|op| (op.name.to_string(), op.opcode)).collect();
    assert_eq!(spec, table);
}

#[test]
fn the_branch_opcode_matches() {
    assert!(spec().contains("| \u{2212}12 | `b<cond> rs1, rs2, label` |"));
    assert_eq!(OPS.iter().find(|op| op.name == "b").unwrap().opcode, -12);
}
