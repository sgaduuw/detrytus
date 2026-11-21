//! The block device (docs/platform.md, The block device): a disk that
//! moves runs of whole blocks between itself and RAM pages on its own,
//! then raises interrupt source 2.
//!
//! The device keeps its registers and the command in flight. The bus
//! carries a command out when it finishes, since that needs RAM; where
//! the blocks live is a [`Disk`]'s business, so the core touches no
//! files.

use std::collections::HashMap;

use crate::bus::PAGE;

pub const BLOCK_PAGE: i64 = -14;
const CENTRE: i64 = BLOCK_PAGE * PAGE;

/// A block is one page.
pub const BLOCK_TRYTES: usize = PAGE as usize;

// Registers, as offsets from the page's centre.
const BLOCKS: i64 = -9;
const BLOCK: i64 = -6;
const COUNT: i64 = -3;
const LIST: i64 = 0;
const COMMAND: i64 = 3;
const STATUS: i64 = 6;
const ACK: i64 = 9;

/// COMMAND values: the sign is the direction.
pub const BLOCK_READ: i64 = 1;
pub const BLOCK_WRITE: i64 = -1;

/// STATUS values.
pub const BLOCK_BUSY: i64 = 1;
pub const BLOCK_DONE: i64 = 0;
pub const BLOCK_FAILED: i64 = -1;

/// Rounds a command takes: a fixed setup, then a little per block, so
/// one command for many blocks is cheaper than many commands.
pub const SETUP_ROUNDS: i64 = 1000;
pub const ROUNDS_PER_BLOCK: i64 = 100;

/// The address of the register at `offset`: for tests.
pub fn block_addr(offset: i64) -> i64 {
    CENTRE + offset
}

/// Where the blocks live. Trytes come and go a run of whole blocks at a
/// time, in address order within each block.
pub trait Disk {
    fn blocks(&self) -> i64;
    /// `count` blocks from `first`: `count × BLOCK_TRYTES` trytes.
    fn read(&mut self, first: i64, count: i64) -> Vec<i16>;
    /// Whole blocks from `first`.
    fn write(&mut self, first: i64, trytes: &[i16]);
}

/// A disk in memory: for tests and the conformance suite.
pub struct MemoryDisk {
    blocks: i64,
    data: HashMap<i64, Vec<i16>>,
}

impl MemoryDisk {
    pub fn new(blocks: i64) -> Self {
        MemoryDisk { blocks, data: HashMap::new() }
    }
}

impl Disk for MemoryDisk {
    fn blocks(&self) -> i64 {
        self.blocks
    }

    fn read(&mut self, first: i64, count: i64) -> Vec<i16> {
        (first..first + count)
            .flat_map(|b| self.data.get(&b).cloned().unwrap_or_else(|| vec![0; BLOCK_TRYTES]))
            .collect()
    }

    fn write(&mut self, first: i64, trytes: &[i16]) {
        for (i, block) in trytes.chunks(BLOCK_TRYTES).enumerate() {
            self.data.insert(first + i as i64, block.to_vec());
        }
    }
}

/// A command the device has accepted and not finished.
pub struct Transfer {
    pub command: i64,
    pub first: i64,
    pub pages: Vec<i64>,
    pub done_at: i64,
}

pub struct BlockDevice {
    pub disk: Option<Box<dyn Disk>>,
    block: i64,
    count: i64,
    list: i64,
    status: i64,
    /// A command has finished and not been acknowledged: source 2.
    pub finished: bool,
    pub transfer: Option<Transfer>,
}

impl Default for BlockDevice {
    fn default() -> Self {
        BlockDevice {
            disk: None,
            block: 0,
            count: 1,
            list: 0,
            status: BLOCK_DONE,
            finished: false,
            transfer: None,
        }
    }
}

/// What a COMMAND write asks of the bus.
pub enum Request {
    Nothing,
    /// Start a transfer: read the list of `count` page numbers at `list`.
    Start {
        command: i64,
        first: i64,
        count: i64,
        list: i64,
    },
}

impl BlockDevice {
    pub fn blocks(&self) -> i64 {
        self.disk.as_ref().map_or(0, |d| d.blocks())
    }

    /// When the command in flight finishes, if there is one.
    pub fn done_at(&self) -> Option<i64> {
        self.transfer.as_ref().map(|t| t.done_at)
    }

    pub fn read(&self, addr: i64) -> i64 {
        match addr - CENTRE {
            BLOCKS => self.blocks(),
            BLOCK => self.block,
            COUNT => self.count,
            LIST => self.list,
            STATUS => self.status,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: i64, value: i64) -> Request {
        match addr - CENTRE {
            BLOCK => self.block = value,
            COUNT => self.count = value,
            LIST => self.list = value,
            ACK => self.finished = false,
            COMMAND if self.transfer.is_none() && (value == BLOCK_READ || value == BLOCK_WRITE) => {
                return Request::Start {
                    command: value,
                    first: self.block,
                    count: self.count,
                    list: self.list,
                };
            }
            _ => {}
        }
        Request::Nothing
    }

    /// Whether a run of `count` blocks from `first` is on the disk.
    pub fn in_range(&self, first: i64, count: i64) -> bool {
        count >= 1 && first >= 0 && first + count <= self.blocks()
    }

    pub fn fail(&mut self) {
        self.status = BLOCK_FAILED;
        self.finished = true;
    }

    pub fn begin(&mut self, transfer: Transfer) {
        self.status = BLOCK_BUSY;
        self.transfer = Some(transfer);
    }

    pub fn end(&mut self) {
        self.status = BLOCK_DONE;
        self.finished = true;
        self.transfer = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_read_back_and_no_disk_has_no_blocks() {
        let mut d = BlockDevice::default();
        assert_eq!(d.read(block_addr(BLOCKS)), 0);
        assert_eq!(d.read(block_addr(COUNT)), 1, "COUNT is 1 at reset");
        d.write(block_addr(BLOCK), 5);
        d.write(block_addr(LIST), 19683);
        assert_eq!(d.read(block_addr(BLOCK)), 5);
        assert_eq!(d.read(block_addr(LIST)), 19683);
        d.disk = Some(Box::new(MemoryDisk::new(4)));
        assert_eq!(d.read(block_addr(BLOCKS)), 4);
        assert!(d.in_range(3, 1) && !d.in_range(3, 2) && !d.in_range(0, 0) && !d.in_range(-1, 1));
    }

    #[test]
    fn commands_are_ignored_while_busy_and_other_values_always() {
        let mut d = BlockDevice::default();
        assert!(matches!(d.write(block_addr(COMMAND), 0), Request::Nothing));
        assert!(matches!(d.write(block_addr(COMMAND), BLOCK_WRITE), Request::Start { .. }));
        d.begin(Transfer { command: BLOCK_WRITE, first: 0, pages: vec![1], done_at: 9 });
        assert_eq!(d.read(block_addr(STATUS)), BLOCK_BUSY);
        assert!(matches!(d.write(block_addr(COMMAND), BLOCK_READ), Request::Nothing));
        d.end();
        assert!(d.finished);
        d.write(block_addr(ACK), 0);
        assert!(!d.finished);
        assert_eq!(d.read(block_addr(STATUS)), BLOCK_DONE);
    }

    #[test]
    fn a_memory_disk_reads_zeros_until_written() {
        let mut disk = MemoryDisk::new(3);
        assert!(disk.read(1, 2).iter().all(|&t| t == 0));
        let mut data = vec![0; 2 * BLOCK_TRYTES];
        data[BLOCK_TRYTES] = 7;
        disk.write(1, &data);
        assert_eq!(disk.read(2, 1)[0], 7);
    }
}
