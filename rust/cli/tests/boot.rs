//! Booting from a disk: `run --disk FILE` with no program loads the boot
//! ROM, which reads the image block 0's boot record names and starts it
//! (docs/platform.md, Booting from a disk).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use detrytus_asm::assemble;

const PAGE: i64 = 19683;
const HALF: i64 = PAGE / 2;
const MAGIC: i64 = 20261128;

/// Two pages from page 2: code in the first jumps to code in the second,
/// which prints and powers off with 7 + a0, so a status of 7 also says
/// the ROM started it with a0 = 0.
const PROGRAM: &str = "
        .equ  PAGE, 19683
        .section one, 2 * PAGE - 9840, 2 * PAGE - 9840
start:  mv    s0, a0
        li    t0, far
        jalr  zero, 0(t0)
        .section two, 3 * PAGE - 9840, 3 * PAGE - 9840
far:    la    t0, message
        li    t1, -19683
1:      lt    t2, 0(t0)
        beqz  t2, 2f
        sw    t2, 0(t1)
        addi  t0, t0, 1
        j     1b
2:      li    t1, -39366
        addi  a0, s0, 7
        sw    a0, 0(t1)
message: .string \"booted from the disk\\n\"
";

/// A disk of `blocks` blocks, as trytes, block by block.
struct Disk(Vec<Vec<i64>>);

impl Disk {
    fn new(blocks: usize) -> Disk {
        Disk(vec![vec![0; PAGE as usize]; blocks])
    }

    fn set_word(&mut self, block: usize, offset: i64, mut value: i64) {
        let i = (offset + HALF) as usize;
        for k in 0..3 {
            let low = (value + HALF).rem_euclid(PAGE) - HALF;
            self.0[block][i - 1 + k] = low;
            value = (value - low) / PAGE;
        }
    }

    /// The boot record: words 3 to 7 of block 0.
    fn record(&mut self, magic: i64, first: i64, count: i64, page: i64, entry: i64) {
        for (w, v) in [magic, first, count, page, entry].into_iter().enumerate() {
            self.set_word(0, -9840 + 3 * (3 + w as i64), v);
        }
    }

    /// PROGRAM's pages 2 and 3 in blocks 1 and 2, and a record for them.
    fn bootable() -> Disk {
        let image = assemble(PROGRAM, None).unwrap();
        let mut disk = Disk::new(3);
        for (addr, tryte) in image.trytes {
            let page = (addr + HALF).div_euclid(PAGE);
            disk.0[(page - 1) as usize][(addr - page * PAGE + HALF) as usize] = tryte;
        }
        disk.record(MAGIC, 1, 2, 2, 2 * PAGE - 9840);
        disk
    }

    fn write(&self, name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("detrytus-boot-{}-{name}", std::process::id()));
        let bytes: Vec<u8> =
            self.0.iter().flatten().flat_map(|&t| (t as i16).to_le_bytes()).collect();
        fs::write(&path, bytes).unwrap();
        path
    }
}

/// `detrytus run --disk` on this disk, with more arguments: the exit
/// status, stdout and stderr.
fn boot(disk: &Disk, name: &str, more: &[&str]) -> (i32, String, String) {
    let path = disk.write(name);
    let out = Command::new(env!("CARGO_BIN_EXE_detrytus"))
        .arg("run")
        .args(more)
        .arg("--disk")
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(&path).unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (out.status.code().unwrap(), text(out.stdout), text(out.stderr))
}

#[test]
fn boots_the_image_the_record_names() {
    let (status, out, err) = boot(&Disk::bootable(), "ok", &[]);
    assert_eq!((status, out.as_str(), err.as_str()), (7, "booted from the disk\n", ""));
}

#[test]
fn boots_in_the_debugger_too() {
    let path = Disk::bootable().write("debug");
    let commands = path.with_extension("commands");
    fs::write(&commands, "continue\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_detrytus"))
        .args(["debug", "--batch", "-x"])
        .arg(&commands)
        .arg("--disk")
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(&path).unwrap();
    fs::remove_file(&commands).unwrap();
    let out = String::from_utf8(out.stdout).unwrap();
    assert!(out.contains("booted from the disk\n"), "{out}");
    assert!(out.contains("exit code 7"), "{out}");
}

#[test]
fn a_disk_without_a_record_is_not_bootable() {
    let (status, out, _) = boot(&Disk::new(2), "blank", &[]);
    assert_eq!((status, out.as_str()), (1, "boot: no boot record in block 0\n"));
}

#[test]
fn a_record_of_no_blocks_is_refused() {
    let mut disk = Disk::bootable();
    disk.record(MAGIC, 1, 0, 2, 2 * PAGE - 9840);
    let (status, out, _) = boot(&disk, "empty", &[]);
    assert_eq!((status, out.as_str()), (1, "boot: the boot record is not valid\n"));
}

#[test]
fn a_record_aimed_at_the_boot_page_is_refused() {
    let mut disk = Disk::bootable();
    disk.record(MAGIC, 1, 2, 0, 2 * PAGE - 9840);
    let (status, out, _) = boot(&disk, "page0", &[]);
    assert_eq!((status, out.as_str()), (1, "boot: the boot record is not valid\n"));
}

#[test]
fn an_image_larger_than_ram_is_refused() {
    let (status, out, _) = boot(&Disk::bootable(), "small", &["--ram-pages", "2"]);
    assert_eq!((status, out.as_str()), (1, "boot: the image does not fit in RAM\n"));
}

#[test]
fn blocks_past_the_disk_fail_the_read() {
    let mut disk = Disk::bootable();
    disk.record(MAGIC, 2, 2, 2, 2 * PAGE - 9840);
    let (status, out, _) = boot(&disk, "short", &[]);
    assert_eq!((status, out.as_str()), (1, "boot: reading the disk failed\n"));
}

#[test]
fn no_program_and_no_disk_is_a_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_detrytus")).arg("run").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("a disk to boot from"));
}
