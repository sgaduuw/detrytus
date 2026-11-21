//! `--mix`: how often each instruction occurs, and how many of them a
//! compressed form (docs/isa.md 6.1) could stand for. `run --mix` counts
//! what executes; `asm --mix` counts what is written, and how many words
//! pairing compressible neighbours into bundles would save.

use std::collections::{BTreeMap, BTreeSet};

use detrytus_asm::assembler::Listing;
use detrytus_core::compressed;
use detrytus_core::isa::{self, Format};

#[derive(Default)]
pub struct Mix {
    /// By mnemonic: how many, and how many fit a compressed form.
    rows: BTreeMap<String, (u64, u64)>,
    forms: BTreeMap<&'static str, u64>,
    total: u64,
}

/// A row's name: the instruction's mnemonic, with a branch's condition
/// (beq, not b) and without hints or orderings, which no form keeps
/// apart.
fn mnemonic(d: &isa::Decoded) -> String {
    match d.op.format {
        Format::B => format!("b{}", isa::cond_name(d.cond).unwrap_or("?")),
        _ => d.op.name.to_string(),
    }
}

fn percent(part: u64, whole: u64) -> String {
    format!("{:.1}%", 100.0 * part as f64 / whole.max(1) as f64)
}

impl Mix {
    /// Count one instruction word; whether it fits a compressed form.
    pub fn add(&mut self, word: i64) -> bool {
        let Ok(d) = isa::decode(word) else { return false };
        let form = compressed::form(&d);
        let row = self.rows.entry(mnemonic(&d)).or_default();
        row.0 += 1;
        self.total += 1;
        if let Some(form) = form {
            row.1 += 1;
            *self.forms.entry(form).or_default() += 1;
        }
        form.is_some()
    }

    /// The report: what was counted (`executed` or `written`), then a
    /// row per mnemonic and per form, most frequent first.
    pub fn report(&self, what: &str) -> String {
        let fits: u64 = self.forms.values().sum();
        let (total, share) = (self.total, percent(fits, self.total));
        let mut out = format!("{total} instructions {what}, {fits} compressible ({share})\n");
        out += &format!(
            "{:<12} {:>12} {:>7} {:>12} {:>7}\n",
            "mnemonic", "count", "share", "fit", "of it"
        );
        let mut rows: Vec<_> = self.rows.iter().collect();
        rows.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(a.0.cmp(b.0)));
        for (name, &(count, fit)) in rows {
            let (share, of_it) = (percent(count, self.total), percent(fit, count));
            out += &format!("{name:<12} {count:>12} {share:>7} {fit:>12} {of_it:>7}\n");
        }
        let mut forms: Vec<_> = self.forms.iter().collect();
        forms.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        out += &format!("\n{:<12} {:>12} {:>7}\n", "form", "count", "share");
        for (form, &count) in forms {
            out += &format!("{form:<12} {count:>12} {:>7}\n", percent(count, self.total));
        }
        out
    }
}

/// Where a direct branch or jump at `addr` can go.
fn targets(addr: i64, word: i64) -> Vec<i64> {
    let Ok(d) = isa::decode(word) else { return vec![] };
    match d.op.format {
        Format::B => vec![addr + 3 * d.off],
        Format::J => vec![addr + 3 * d.imm],
        Format::T => vec![addr + 3 * d.offn, addr + 3 * d.offp],
        _ => vec![],
    }
}

/// The static mix of an assembled program, and the bundle estimate: a
/// compressible instruction pairs with the next one when that is
/// compressible too and nothing can jump to it, since a jump enters
/// only slot A. Any label counts as a way in, as a jalr may use it.
pub fn of_listing(listing: &[Listing], labels: impl Iterator<Item = i128>) -> String {
    let mut code: Vec<(i64, i64)> =
        listing.iter().filter(|e| e.code).filter_map(|e| Some((e.addr as i64, e.word?))).collect();
    code.sort();
    let mut entries: BTreeSet<i64> = labels.map(|l| l as i64).collect();
    entries.extend(code.iter().flat_map(|&(addr, word)| targets(addr, word)));
    let mut mix = Mix::default();
    let fits: Vec<bool> = code.iter().map(|&(_, word)| mix.add(word)).collect();
    let mut pairs = 0;
    let mut i = 0;
    while i + 1 < code.len() {
        let (a, b) = (code[i].0, code[i + 1].0);
        if fits[i] && fits[i + 1] && b == a + 3 && !entries.contains(&b) {
            pairs += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    let words = code.len() as u64;
    format!(
        "{}\nbundles: {pairs} pairs would save {pairs} of {words} words ({})\n",
        mix.report("written"),
        percent(pairs, words)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use detrytus_asm::assembler::assemble;

    fn mix_of(source: &str) -> String {
        let image = assemble(source, None).unwrap();
        of_listing(&image.listing, image.labels.values().copied())
    }

    #[test]
    fn neighbours_pair_unless_the_second_is_a_way_in() {
        // Four compressible in a row make two bundles.
        let out = mix_of("addi a0, a0, 1\naddi a0, a0, 1\nmv a1, a0\nret\n");
        assert!(out.contains("bundles: 2 pairs would save 2 of 4 words (50.0%)"), "{out}");
        // A label or a branch on the second keeps it in slot A, so it
        // pairs with the third instead and the fourth is left alone.
        let out = mix_of("addi a0, a0, 1\nx: addi a0, a0, 1\nret\nret\n");
        assert!(out.contains("bundles: 1 pairs"), "{out}");
        let out = mix_of("addi a0, a0, 1\naddi a0, a0, 1\nret\nbnez a0, 3\n");
        assert!(out.contains("bundles: 1 pairs"), "{out}");
        // A word that does not fit splits a run, and so does data.
        let out = mix_of("addi a0, a0, 1\naddi a0, a1, 1\naddi a0, a0, 1\n");
        assert!(out.contains("bundles: 0 pairs would save 0 of 3 words"), "{out}");
        let out = mix_of("addi a0, a0, 1\n.word 0\naddi a0, a0, 1\n");
        assert!(out.contains("bundles: 0 pairs would save 0 of 2 words"), "{out}");
    }

    #[test]
    fn rows_count_by_mnemonic_most_frequent_first() {
        let mut mix = Mix::default();
        let image = assemble("beq a0, zero, 0\nbne a0, a1, 0\nbne a0, a1, 0\n", None).unwrap();
        for e in &image.listing {
            mix.add(e.word.unwrap());
        }
        let out = mix.report("executed");
        assert!(out.starts_with("3 instructions executed, 1 compressible (33.3%)\n"), "{out}");
        let bne = out.find("\nbne ").unwrap();
        assert!(bne < out.find("\nbeq ").unwrap(), "{out}");
        assert!(out.contains("c.beqz"), "{out}");
    }
}
