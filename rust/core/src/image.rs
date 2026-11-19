//! Reading `.dti` images, the format `detrytus asm -o` writes (layout in
//! `detrytus/image.py`). The JSON metadata (labels and symbols) is for
//! tools, so the core skips it.

use crate::trits::{TRYTE_MAX, TRYTE_TRITS, from_trits};

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
