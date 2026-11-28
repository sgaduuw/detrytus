//! The real-time clock: wall-clock time for software, in the power page
//! (docs/platform.md, The real-time clock).
//!
//! Two read-only registers: CLOCK, whole seconds since the Unix epoch
//! (1970-01-01 00:00:00 UTC), and CLOCK_MICROS, the microseconds past
//! that second. Reading CLOCK latches the microseconds for the hart that
//! read it, so the pair a hart reads is one moment, however a second
//! turns over between the two loads.
//!
//! The time comes from the host, or, for a run that must repeat exactly,
//! from the machine's own clock: a fixed start plus one microsecond per
//! tick of `time`, the rate a live console paces idle time at.
//!
//! Beside it, a monotonic clock: MONO, whole seconds since the machine
//! started, and MONO_NANOS, the nanoseconds past that second, latched
//! the same way. Nothing sets it and the host's clock being set does
//! not move it, so software measures intervals with it.

use crate::fastmap::FastMap as HashMap;

/// Microseconds in a second.
pub const MICROS: i64 = 1_000_000;
/// Nanoseconds in a second.
pub const NANOS: i64 = 1_000_000_000;

/// Where the time comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The host's clock, read at each access.
    Host,
    /// `start` seconds after the epoch at tick 0, then a microsecond a
    /// tick: the same every run. The monotonic clock is then the ticks
    /// alone, a microsecond each.
    Ticks { start: i64 },
}

pub struct Clock {
    pub source: Source,
    /// The microseconds latched by each hart's last read of CLOCK.
    latched: HashMap<i64, i64>,
    /// When the machine started, by the host's monotonic clock.
    started: std::time::Instant,
    /// The nanoseconds latched by each hart's last read of MONO.
    latched_nanos: HashMap<i64, i64>,
}

impl Default for Clock {
    fn default() -> Self {
        Clock {
            source: Source::Host,
            latched: HashMap::default(),
            started: std::time::Instant::now(),
            latched_nanos: HashMap::default(),
        }
    }
}

impl Clock {
    /// Microseconds since the epoch at tick `now`.
    fn micros(&self, now: i64) -> i64 {
        match self.source {
            Source::Host => {
                use std::time::{SystemTime, UNIX_EPOCH};
                match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_micros() as i64,
                    // A host clock set before 1970.
                    Err(e) => -(e.duration().as_micros() as i64),
                }
            }
            Source::Ticks { start } => start * MICROS + now,
        }
    }

    /// Read CLOCK for hart `hart` at tick `now`: whole seconds, rounded
    /// down, so the microseconds are never negative.
    pub fn seconds(&mut self, hart: i64, now: i64) -> i64 {
        let t = self.micros(now);
        self.latched.insert(hart, t.rem_euclid(MICROS));
        t.div_euclid(MICROS)
    }

    /// Read CLOCK_MICROS: what hart `hart`'s last read of CLOCK latched,
    /// 0 before its first.
    pub fn micros_latched(&self, hart: i64) -> i64 {
        self.latched.get(&hart).copied().unwrap_or(0)
    }

    /// Nanoseconds since the machine started, at tick `now`.
    fn mono_nanos(&self, now: i64) -> i64 {
        match self.source {
            Source::Host => self.started.elapsed().as_nanos() as i64,
            Source::Ticks { .. } => now * (NANOS / MICROS),
        }
    }

    /// Read MONO for hart `hart` at tick `now`: whole seconds since the
    /// machine started.
    pub fn mono_seconds(&mut self, hart: i64, now: i64) -> i64 {
        let t = self.mono_nanos(now);
        self.latched_nanos.insert(hart, t.rem_euclid(NANOS));
        t.div_euclid(NANOS)
    }

    /// Read MONO_NANOS: what hart `hart`'s last read of MONO latched, 0
    /// before its first.
    pub fn nanos_latched(&self, hart: i64) -> i64 {
        self.latched_nanos.get(&hart).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_clock_moves_a_microsecond_a_tick() {
        let mut c = Clock { source: Source::Ticks { start: 1_000 }, ..Clock::default() };
        assert_eq!(c.seconds(0, 0), 1_000);
        assert_eq!(c.micros_latched(0), 0);
        assert_eq!(c.seconds(0, 2_500_001), 1_002);
        assert_eq!(c.micros_latched(0), 500_001);
    }

    #[test]
    fn each_hart_keeps_its_own_latch() {
        let mut c = Clock { source: Source::Ticks { start: 0 }, ..Clock::default() };
        c.seconds(0, 7);
        c.seconds(1, 9);
        assert_eq!(c.micros_latched(0), 7);
        assert_eq!(c.micros_latched(1), 9);
        assert_eq!(c.micros_latched(-1), 0);
    }

    #[test]
    fn before_the_epoch_the_microseconds_stay_positive() {
        let mut c = Clock { source: Source::Ticks { start: -2 }, ..Clock::default() };
        assert_eq!(c.seconds(0, 250_000), -2);
        assert_eq!(c.micros_latched(0), 250_000);
        let mut c = Clock { source: Source::Ticks { start: 0 }, ..Clock::default() };
        assert_eq!(c.seconds(0, -1), -1);
        assert_eq!(c.micros_latched(0), 999_999);
    }

    #[test]
    fn a_fixed_monotonic_clock_counts_ticks() {
        let mut c = Clock { source: Source::Ticks { start: 1_000 }, ..Clock::default() };
        assert_eq!(c.mono_seconds(0, 2_500_001), 2);
        assert_eq!(c.nanos_latched(0), 500_001_000);
        assert_eq!(c.nanos_latched(1), 0);
    }

    #[test]
    fn the_host_monotonic_clock_never_goes_back() {
        let mut c = Clock::default();
        let first = (c.mono_seconds(0, 0), c.nanos_latched(0));
        let second = (c.mono_seconds(0, 0), c.nanos_latched(0));
        assert!(first.0 >= 0 && second >= first);
    }

    #[test]
    fn the_host_clock_is_after_this_was_written() {
        let mut c = Clock::default();
        assert!(c.seconds(0, 0) > 1_795_000_000); // late November 2026
    }
}
