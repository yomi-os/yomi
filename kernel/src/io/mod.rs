//! I/O and logging subsystem
//!
//! This module provides kernel logging functionality with log levels,
//! timestamps, and colored output.

pub mod logging;

pub use logging::{get_log_level, log, set_log_level, LogLevel};
