//! Kernel logging with log levels and timestamps
//!
//! This module provides structured logging functionality with:
//! - Log levels (DEBUG, INFO, WARN, ERROR, FATAL)
//! - Timestamps (seconds.milliseconds format)
//! - ANSI color coding for different levels
//! - Log level filtering
//!
//! # Examples
//!
//! ```
//! log_info!("Kernel initialized");
//! log_debug!("Page table at {:#x}", addr);
//! log_warn!("Timer drift detected: {} ticks", drift);
//! log_error!("Page fault at {:#x}", faulting_addr);
//! ```

use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

/// Log level enumeration
///
/// Log levels are ordered by severity:
/// DEBUG < INFO < WARN < ERROR < FATAL
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    /// Detailed debug information
    DEBUG = 0,
    /// Informational messages
    INFO = 1,
    /// Warning messages
    WARN = 2,
    /// Error messages
    ERROR = 3,
    /// Fatal errors (system will halt)
    FATAL = 4,
}

impl LogLevel {
    /// Returns the string representation of the log level
    pub const fn as_str(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => " INFO",
            LogLevel::WARN => " WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::FATAL => "FATAL",
        }
    }

    /// Returns the ANSI color code for the log level
    pub const fn color_code(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "\x1b[36m", // Cyan
            LogLevel::INFO => "\x1b[32m",  // Green
            LogLevel::WARN => "\x1b[33m",  // Yellow
            LogLevel::ERROR => "\x1b[31m", // Red
            LogLevel::FATAL => "\x1b[35m", // Magenta
        }
    }
}

/// Global log level filter
///
/// Messages with level lower than this value will be filtered out.
static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::DEBUG as u8);

/// Sets the minimum log level
///
/// Messages with level lower than the specified level will be filtered out.
///
/// # Examples
///
/// ```
/// use kernel::io::logging::{LogLevel, set_log_level};
///
/// // Only show INFO and above
/// set_log_level(LogLevel::INFO);
/// ```
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Gets the current log level
///
/// # Returns
///
/// The current minimum log level
pub fn get_log_level() -> LogLevel {
    let level = LOG_LEVEL.load(Ordering::Relaxed);
    match level {
        0 => LogLevel::DEBUG,
        1 => LogLevel::INFO,
        2 => LogLevel::WARN,
        3 => LogLevel::ERROR,
        4 => LogLevel::FATAL,
        _ => LogLevel::INFO, // Default fallback
    }
}

/// Logs a message with the specified log level
///
/// This function is the core logging implementation. It:
/// 1. Checks if the message should be logged based on the current log level
/// 2. Formats the timestamp
/// 3. Adds ANSI color codes
/// 4. Writes to the serial port
///
/// # Arguments
///
/// * `level` - The log level for this message
/// * `args` - The formatted message arguments
///
/// # Examples
///
/// ```
/// use kernel::io::logging::{LogLevel, log};
///
/// log(LogLevel::INFO, format_args!("System initialized"));
/// log(LogLevel::ERROR, format_args!("Error code: {}", error_code));
/// ```
pub fn log(level: LogLevel, args: fmt::Arguments) {
    // Filter out messages below the current log level
    if level < get_log_level() {
        return;
    }

    use core::fmt::Write;

    // Disable interrupts while logging to avoid race conditions
    crate::interrupts::without_interrupts(|| {
        // Get system uptime for timestamp
        let uptime_ms = crate::interrupts::timer::uptime_ms();
        let secs = uptime_ms / 1000;
        let ms = uptime_ms % 1000;

        // Acquire serial port lock
        let mut serial = crate::serial::SERIAL1.lock();

        // Write timestamp
        let _ = write!(serial, "[{}.{:03}] ", secs, ms);

        // Write log level with color
        let _ = write!(serial, "{}[{}]\x1b[0m ", level.color_code(), level.as_str());

        // Write message
        let _ = serial.write_fmt(args);

        // Write newline
        let _ = writeln!(serial);
    });
}

/// Macro for logging debug messages
///
/// Debug messages are only shown when the log level is set to DEBUG.
///
/// # Examples
///
/// ```
/// log_debug!("Memory allocator initialized");
/// log_debug!("Page table at {:#x}", page_table_addr);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::io::logging::log(
            $crate::io::logging::LogLevel::DEBUG,
            format_args!($($arg)*)
        )
    };
}

/// Macro for logging informational messages
///
/// # Examples
///
/// ```
/// log_info!("Kernel initialized");
/// log_info!("Available memory: {} KB", mem_kb);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::io::logging::log(
            $crate::io::logging::LogLevel::INFO,
            format_args!($($arg)*)
        )
    };
}

/// Macro for logging warning messages
///
/// # Examples
///
/// ```
/// log_warn!("Timer drift detected");
/// log_warn!("Low memory: {} bytes free", free_bytes);
/// ```
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::io::logging::log(
            $crate::io::logging::LogLevel::WARN,
            format_args!($($arg)*)
        )
    };
}

/// Macro for logging error messages
///
/// # Examples
///
/// ```
/// log_error!("Page fault at {:#x}", addr);
/// log_error!("Failed to allocate frame");
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::io::logging::log(
            $crate::io::logging::LogLevel::ERROR,
            format_args!($($arg)*)
        )
    };
}

/// Macro for logging fatal error messages
///
/// Fatal errors typically indicate unrecoverable conditions.
///
/// # Examples
///
/// ```
/// log_fatal!("Double fault occurred");
/// log_fatal!("Hardware initialization failed");
/// ```
#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        $crate::io::logging::log(
            $crate::io::logging::LogLevel::FATAL,
            format_args!($($arg)*)
        )
    };
}

/// Macro for backward compatibility with traditional printk
///
/// This macro is equivalent to log_info!
///
/// # Examples
///
/// ```
/// printk!("Kernel initialization complete");
/// printk!("Device detected: {}", device_name);
/// ```
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        $crate::log_info!($($arg)*)
    };
}

/// Basic print macro without log level or timestamp
///
/// This is useful for simple output that doesn't need formatting.
///
/// # Examples
///
/// ```
/// print!("Hello");
/// print!("Value: {}", value);
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Basic println macro without log level or timestamp
///
/// This is useful for simple output that doesn't need formatting.
///
/// # Examples
///
/// ```
/// println!("Hello, world!");
/// println!("Value: {}", value);
/// ```
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}
