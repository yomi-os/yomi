//! I/O and logging subsystem
//!
//! This module provides kernel logging functionality with log levels,
//! timestamps, and colored output.

pub mod logging;

pub use logging::{LogLevel, log, set_log_level, get_log_level};
