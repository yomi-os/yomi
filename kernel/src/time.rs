// Copyright 2025 Yomi OS Development Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Time management subsystem
//!
//! This module provides time-related functionality including
//! system uptime, timestamps, and time utilities.

#![allow(dead_code)]

use crate::interrupts::timer;

/// Returns the current tick count
///
/// Each tick represents one timer interrupt. The frequency is determined
/// by the PIT configuration (typically 100 Hz).
///
/// # Returns
///
/// The number of timer ticks since the system started.
///
/// # Example
///
/// ```
/// let current_ticks = time::ticks();
/// println!("Ticks: {}", current_ticks);
/// ```
pub fn ticks() -> u64 {
    timer::ticks()
}

/// Returns the system uptime in milliseconds
///
/// # Returns
///
/// System uptime in milliseconds since boot.
///
/// # Example
///
/// ```
/// let uptime = time::uptime_ms();
/// println!("Uptime: {} ms", uptime);
/// ```
pub fn uptime_ms() -> u64 {
    timer::uptime_ms()
}

/// Returns the system uptime in seconds
///
/// # Returns
///
/// System uptime in seconds since boot.
///
/// # Example
///
/// ```
/// let uptime = time::uptime_seconds();
/// println!("Uptime: {} seconds", uptime);
/// ```
pub fn uptime_seconds() -> u64 {
    timer::uptime_seconds()
}

/// Time duration in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    millis: u64,
}

impl Duration {
    /// Creates a new Duration from milliseconds
    pub const fn from_millis(millis: u64) -> Self {
        Self { millis }
    }

    /// Creates a new Duration from seconds
    pub const fn from_secs(secs: u64) -> Self {
        Self {
            millis: secs * 1000,
        }
    }

    /// Returns the duration in milliseconds
    pub const fn as_millis(&self) -> u64 {
        self.millis
    }

    /// Returns the duration in seconds
    pub const fn as_secs(&self) -> u64 {
        self.millis / 1000
    }
}

/// System timestamp in milliseconds since boot
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    millis: u64,
}

impl Timestamp {
    /// Returns the current timestamp
    pub fn now() -> Self {
        Self {
            millis: uptime_ms(),
        }
    }

    /// Returns the elapsed time since this timestamp
    pub fn elapsed(&self) -> Duration {
        let current = uptime_ms();
        Duration::from_millis(current.saturating_sub(self.millis))
    }

    /// Returns the timestamp in milliseconds
    pub const fn as_millis(&self) -> u64 {
        self.millis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_from_millis() {
        let duration = Duration::from_millis(5000);
        assert_eq!(duration.as_millis(), 5000);
        assert_eq!(duration.as_secs(), 5);
    }

    #[test]
    fn test_duration_from_secs() {
        let duration = Duration::from_secs(10);
        assert_eq!(duration.as_millis(), 10000);
        assert_eq!(duration.as_secs(), 10);
    }

    #[test]
    fn test_timestamp_ordering() {
        let ts = Timestamp::now();
        // Simulate time passing (in real tests, TICKS would increment)
        let elapsed = ts.elapsed();
        // Elapsed time should be non-negative
        assert_eq!(elapsed.as_millis(), 0); // In unit tests without timer running
    }
}
