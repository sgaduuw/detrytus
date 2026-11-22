//! A live console for `run`: what is typed reaches the machine as it is
//! typed, not queued up front as `--input` is.
//!
//! A thread reads stdin and hands the bytes over. On a terminal, the
//! terminal is put in a raw-ish mode (no line editing, no echo, no
//! signals) so that every key, backspace and Ctrl-C included, goes to
//! the program, which echoes what it wants to; Ctrl-D then closes the
//! console and Ctrl-] stops the machine. The terminal is put back when
//! the run ends. From a pipe, the bytes are passed on as they come and
//! the end of the pipe closes the console.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// Ctrl-D: on a terminal, closes the console.
const END: u8 = 0x04;
/// Ctrl-]: on a terminal, stops the machine (as telnet's escape).
const STOP: u8 = 0x1d;

/// What has been typed since the last look.
#[derive(Default)]
pub struct Typed {
    pub text: String,
    /// No more will come: the console closes once `text` is read.
    pub closed: bool,
    /// Ctrl-] on a terminal: stop the machine.
    pub stop: bool,
}

pub struct Live {
    /// Bytes from stdin; None once it has ended.
    bytes: Receiver<Option<Vec<u8>>>,
    /// The start of a character whose other bytes are still to come.
    partial: Vec<u8>,
    /// Set on a terminal: the mode to put back, and Ctrl-D/Ctrl-] apply.
    raw: Option<RawMode>,
    ended: bool,
}

impl Live {
    pub fn start(terminal: bool) -> Live {
        let raw = if terminal { RawMode::enter() } else { None };
        let (send, bytes) = channel();
        std::thread::spawn(move || {
            let mut stdin = std::io::stdin();
            let mut buf = [0u8; 256];
            loop {
                match stdin.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = send.send(None);
                        break;
                    }
                    Ok(n) => {
                        if send.send(Some(buf[..n].to_vec())).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Live { bytes, partial: Vec::new(), raw, ended: false }
    }

    /// What has been typed, waiting up to `wait` for the first of it.
    pub fn take(&mut self, wait: Duration) -> Typed {
        let mut typed = Typed::default();
        if self.ended {
            return typed;
        }
        let mut bytes = Vec::new();
        let first = if wait.is_zero() {
            self.bytes.try_recv().map_err(|_| RecvTimeoutError::Timeout)
        } else {
            self.bytes.recv_timeout(wait)
        };
        let mut next = first.ok();
        while let Some(chunk) = next {
            match chunk {
                Some(b) => bytes.extend_from_slice(&b),
                None => {
                    typed.closed = true;
                    break;
                }
            }
            next = self.bytes.try_recv().ok();
        }
        if self.raw.is_some()
            && let Some(i) = bytes.iter().position(|&b| b == END || b == STOP)
        {
            typed.stop = bytes[i] == STOP;
            typed.closed = true;
            bytes.truncate(i);
        }
        self.partial.extend_from_slice(&bytes);
        typed.text = self.decode(typed.closed);
        self.ended = typed.closed;
        typed
    }

    /// The whole characters in `partial`, leaving an unfinished one there
    /// unless nothing more will come. Bytes that are not UTF-8 become
    /// U+FFFD.
    fn decode(&mut self, last: bool) -> String {
        let mut text = String::new();
        loop {
            match std::str::from_utf8(&self.partial) {
                Ok(s) => {
                    text.push_str(s);
                    self.partial.clear();
                    return text;
                }
                Err(e) => {
                    let good = e.valid_up_to();
                    text.push_str(std::str::from_utf8(&self.partial[..good]).unwrap());
                    match e.error_len() {
                        Some(bad) => {
                            text.push('\u{FFFD}');
                            self.partial.drain(..good + bad);
                        }
                        None => {
                            self.partial.drain(..good);
                            if last {
                                text.push('\u{FFFD}');
                                self.partial.clear();
                            }
                            return text;
                        }
                    }
                }
            }
        }
    }
}

/// The terminal's settings before the run, put back when dropped.
struct RawMode(String);

impl RawMode {
    /// None where stty is missing or refuses: the run goes on with the
    /// terminal as it is.
    fn enter() -> Option<RawMode> {
        let saved = Command::new("stty").arg("-g").stdin(Stdio::inherit()).output().ok()?;
        if !saved.status.success() {
            return None;
        }
        let saved = String::from_utf8(saved.stdout).ok()?.trim().to_string();
        let set = Command::new("stty")
            .args(["-icanon", "-echo", "-isig", "-iexten", "-ixon", "min", "1", "time", "0"])
            .stdin(Stdio::inherit())
            .status()
            .ok()?;
        set.success().then_some(RawMode(saved))
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = Command::new("stty").arg(&self.0).stdin(Stdio::inherit()).status();
    }
}
