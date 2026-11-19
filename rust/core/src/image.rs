//! `.dti` images, the format `detrytus asm -o` writes (layout in
//! `detrytus/image.py`). Reading skips the JSON metadata (labels and
//! symbols), which is for tools; writing produces exactly the bytes the
//! Python writer does, metadata included.

use std::collections::BTreeMap;

use crate::trits::{TRYTE_MAX, TRYTE_TRITS, from_trits, low};

const MAGIC: &[u8; 4] = b"DTI\x01";
const TRITS_PER_BYTE: usize = 5;

/// A run of trytes at consecutive addresses.
#[derive(Debug, PartialEq, Eq)]
pub struct Segment {
    pub start: i64,
    pub trytes: Vec<i64>,
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len());
        let end = end.ok_or("image is truncated")?;
        let chunk = &self.data[self.pos..end];
        self.pos = end;
        Ok(chunk)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64, String> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

fn unpack(data: &[u8], count: usize) -> Result<Vec<i64>, String> {
    let mut trits = Vec::with_capacity(data.len() * TRITS_PER_BYTE);
    for &byte in data {
        if byte > 242 {
            return Err(format!("byte {byte} is not five trits"));
        }
        let mut b = byte as i64;
        for _ in 0..TRITS_PER_BYTE {
            trits.push(b % 3 - 1);
            b /= 3;
        }
    }
    let width = TRYTE_TRITS as usize;
    Ok(trits.chunks(width).take(count).map(from_trits).collect())
}

/// A string as `json.dumps` writes it with its default `ensure_ascii`.
fn json_string(s: &str) -> String {
    let mut out = String::from('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            _ => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
        }
    }
    out.push('"');
    out
}

fn json_table<V: std::fmt::Display>(table: &BTreeMap<String, V>) -> String {
    let items: Vec<String> =
        table.iter().map(|(k, v)| format!("{}: {v}", json_string(k))).collect();
    format!("{{{}}}", items.join(", "))
}

fn pack(trytes: &[i64]) -> Vec<u8> {
    let mut trits: Vec<i64> = Vec::with_capacity(trytes.len() * TRYTE_TRITS as usize + 4);
    for &t in trytes {
        let mut x = t;
        for _ in 0..TRYTE_TRITS {
            let trit = low(x, 1);
            trits.push(trit);
            x = (x - trit) / 3;
        }
    }
    // The last byte is padded with zero trits, which still count (+1).
    trits.resize(trits.len().div_ceil(TRITS_PER_BYTE) * TRITS_PER_BYTE, 0);
    trits
        .chunks(TRITS_PER_BYTE)
        .map(|chunk| {
            let mut byte = 0;
            for (i, t) in chunk.iter().enumerate() {
                byte += (t + 1) * 3i64.pow(i as u32);
            }
            byte as u8
        })
        .collect()
}

/// An image file: trytes by address, labels and symbols, as
/// `detrytus/image.py` writes it (metadata keys sorted, segments in
/// address order).
pub fn dump<V: std::fmt::Display>(
    trytes: &BTreeMap<i64, i64>,
    labels: &BTreeMap<String, V>,
    symbols: &BTreeMap<String, V>,
) -> Vec<u8> {
    let meta =
        format!("{{\"labels\": {}, \"symbols\": {}}}", json_table(labels), json_table(symbols));
    let mut runs: Vec<(i64, Vec<i64>)> = Vec::new();
    for (&addr, &tryte) in trytes {
        match runs.last_mut() {
            Some((start, run)) if *start + run.len() as i64 == addr => run.push(tryte),
            _ => runs.push((addr, vec![tryte])),
        }
    }
    let mut out = MAGIC.to_vec();
    out.extend((meta.len() as u32).to_le_bytes());
    out.extend(meta.as_bytes());
    out.extend((runs.len() as u32).to_le_bytes());
    for (start, run) in &runs {
        out.extend(start.to_le_bytes());
        out.extend((run.len() as u32).to_le_bytes());
        out.extend(pack(run));
    }
    out
}

/// Parse an image's segments.
pub fn parse(data: &[u8]) -> Result<Vec<Segment>, String> {
    let mut r = Reader { data, pos: 0 };
    if r.take(4)? != MAGIC {
        return Err("not a detrytus image (or an unknown version)".into());
    }
    let meta = r.u32()? as usize;
    r.take(meta)?;
    let count = r.u32()?;
    let mut segments = Vec::new();
    for _ in 0..count {
        let start = r.i64()?;
        let n = r.u32()? as usize;
        let bytes = (n * TRYTE_TRITS as usize).div_ceil(TRITS_PER_BYTE);
        let trytes = unpack(r.take(bytes)?, n)?;
        if trytes.iter().any(|t| t.abs() > TRYTE_MAX) {
            return Err("a tryte is out of range".into());
        }
        segments.push(Segment { start, trytes });
    }
    if r.pos != data.len() {
        return Err("trailing bytes after the last segment".into());
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(segments: &[(i64, u32, &[u8])]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend(2u32.to_le_bytes());
        out.extend(b"{}");
        out.extend((segments.len() as u32).to_le_bytes());
        for (start, n, data) in segments {
            out.extend(start.to_le_bytes());
            out.extend(n.to_le_bytes());
            out.extend(*data);
        }
        out
    }

    #[test]
    fn unpacks_five_trits_to_a_byte() {
        // One tryte, value 1: trits +,0,0,0,0 | 0,0,0,(pad 0)
        // A byte is sum((trit + 1)·3^i).
        let first = 2 + 3 + 9 + 27 + 81; // 122
        let second = 1 + 3 + 9 + 27 + 81; // 121: four zero trits and a pad
        let data = image(&[(-1, 1, &[first, second])]);
        assert_eq!(parse(&data), Ok(vec![Segment { start: -1, trytes: vec![1] }]));
    }

    #[test]
    fn dump_and_parse_round_trip() {
        let trytes: BTreeMap<i64, i64> =
            [(-1, 5), (0, -9841), (1, 9841), (10, 7)].into_iter().collect();
        let labels: BTreeMap<String, i64> = [("start".to_string(), 0)].into_iter().collect();
        let data = dump(&trytes, &labels, &BTreeMap::<String, i64>::new());
        let segments = parse(&data).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], Segment { start: -1, trytes: vec![5, -9841, 9841] });
        assert!(data.windows(18).any(|w| w == br#"{"labels": {"start"#));
    }

    #[test]
    fn the_last_byte_is_padded_with_zero_trits() {
        // One tryte, value 1: nine trits +,0,...,0 padded to ten. Python
        // writes bytes 122 and 121 for it (see unpacks_five_trits_to_a_byte).
        assert_eq!(pack(&[1]), vec![122, 121]);
    }

    #[test]
    fn metadata_escapes_like_json_dumps() {
        assert_eq!(json_string("a\"b\\c"), r#""a\"b\\c""#);
        assert_eq!(json_string("é😀"), r#""\u00e9\ud83d\ude00""#);
    }

    #[test]
    fn refuses_damaged_images() {
        assert!(parse(b"XYZ\x01").is_err());
        let ok = image(&[(0, 1, &[121, 121])]);
        assert!(parse(&ok[..ok.len() - 1]).is_err());
        let mut long = ok.clone();
        long.push(0);
        assert!(parse(&long).is_err());
        assert!(parse(&image(&[(0, 1, &[243, 121])])).is_err());
    }
}
