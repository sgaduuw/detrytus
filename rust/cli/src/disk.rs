//! `--disk FILE`: the block device's blocks in a host file, two bytes per
//! tryte, little-endian, so block n starts at byte n × 39,366
//! (docs/platform.md, The block device).

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use detrytus_core::block::{BLOCK_TRYTES, Disk};
use detrytus_core::trits::TRYTE_MOD;

use crate::program::strerror;
use crate::{BAD_INPUT, Fail, NO_INPUT};

const BLOCK_BYTES: u64 = 2 * BLOCK_TRYTES as u64;
const TRYTE_MAX: i16 = (TRYTE_MOD / 2) as i16;

/// Where the disk is and how big: `--disk FILE` and `--disk-blocks N`.
#[derive(Clone)]
pub struct DiskSpec {
    pub path: String,
    pub blocks: Option<i64>,
}

pub struct FileDisk {
    path: String,
    file: File,
    blocks: i64,
    /// Whether a write has failed and said so, so it says so once.
    failed: bool,
}

impl DiskSpec {
    /// Open the file, checking that it holds only trytes. With
    /// --disk-blocks a missing file is created.
    pub fn open(&self) -> Result<FileDisk, Fail> {
        let fail =
            |e: std::io::Error| Fail::new(format!("{}: {}", self.path, strerror(&e)), NO_INPUT);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(self.blocks.is_some())
            .truncate(false)
            .open(&self.path)
            .map_err(fail)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(fail)?;
        if data.len() % 2 != 0 {
            return Err(self.bad("an odd number of bytes, so not whole trytes"));
        }
        if let Some(at) = data
            .chunks(2)
            .position(|b| i16::from_le_bytes([b[0], b[1]]).unsigned_abs() > TRYTE_MAX as u16)
        {
            return Err(self.bad(&format!("byte {} holds a value that is not a tryte", 2 * at)));
        }
        let blocks =
            self.blocks.unwrap_or_else(|| (data.len() as u64).div_ceil(BLOCK_BYTES) as i64);
        Ok(FileDisk { path: self.path.clone(), file, blocks, failed: false })
    }

    fn bad(&self, why: &str) -> Fail {
        Fail::new(format!("{}: {why}", self.path), BAD_INPUT)
    }
}

impl FileDisk {
    fn complain(&mut self, e: std::io::Error) {
        if !self.failed {
            eprintln!("detrytus: {}: {}", self.path, strerror(&e));
            self.failed = true;
        }
    }
}

impl Disk for FileDisk {
    fn blocks(&self) -> i64 {
        self.blocks
    }

    /// Past the end of the file, blocks read as zeros.
    fn read(&mut self, first: i64, count: i64) -> Vec<i16> {
        let mut bytes = vec![0u8; count as usize * BLOCK_BYTES as usize];
        let result = self.file.seek(SeekFrom::Start(first as u64 * BLOCK_BYTES)).and_then(|_| {
            let mut filled = 0;
            loop {
                match self.file.read(&mut bytes[filled..])? {
                    0 => return Ok(()),
                    n => filled += n,
                }
                if filled == bytes.len() {
                    return Ok(());
                }
            }
        });
        if let Err(e) = result {
            self.complain(e);
        }
        bytes.chunks(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect()
    }

    /// One host write for the whole run, so a crash keeps whole commands.
    fn write(&mut self, first: i64, trytes: &[i16]) {
        let bytes: Vec<u8> = trytes.iter().flat_map(|t| t.to_le_bytes()).collect();
        let result = self
            .file
            .seek(SeekFrom::Start(first as u64 * BLOCK_BYTES))
            .and_then(|_| self.file.write_all(&bytes))
            .and_then(|()| self.file.flush());
        if let Err(e) = result {
            self.complain(e);
        }
    }
}
