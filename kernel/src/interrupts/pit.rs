//! PIT (Programmable Interval Timer) implementation
//!
//! The 8253/8254 PIT is a legacy timer chip used to generate periodic interrupts.
//! This module provides initialization and configuration for timer interrupts.

use super::port::Port;

/// PIT base frequency in Hz
///
/// The PIT crystal oscillator runs at 1.193182 MHz
const PIT_FREQUENCY: u32 = 1193182;

/// PIT I/O ports
const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

/// PIT (Programmable Interval Timer)
pub struct Pit {
    channel_0: Port<u8>,
    command: Port<u8>,
}

impl Pit {
    /// Creates a new PIT instance
    pub const fn new() -> Self {
        Self {
            channel_0: Port::new(PIT_CHANNEL_0),
            command: Port::new(PIT_COMMAND),
        }
    }

    /// Sets the timer frequency
    ///
    /// This configures Channel 0 to generate interrupts at the specified frequency.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Desired timer frequency in Hz (e.g., 100 Hz = 10ms interval)
    ///
    /// # Example
    ///
    /// ```
    /// let mut pit = Pit::new();
    /// pit.set_frequency(100); // 100 Hz = 10ms tick interval
    /// ```
    ///
    /// # Safety
    ///
    /// This function writes to PIT I/O ports and should be called during
    /// kernel initialization before enabling timer interrupts.
    pub fn set_frequency(&mut self, frequency: u32) {
        // Validate frequency range
        assert!(frequency > 0, "PIT frequency must be greater than 0");

        // Calculate divisor
        let divisor = PIT_FREQUENCY / frequency;

        // Ensure divisor fits in u16 and meets mode 3 requirements
        assert!(
            (2..=65535).contains(&divisor),
            "PIT divisor {} out of range (2-65535); frequency {} Hz is invalid",
            divisor,
            frequency
        );

        unsafe {
            // Channel 0, Mode 3 (square wave generator), 16-bit binary
            // Command byte: 00 (Channel 0) | 11 (access mode: lobyte/hibyte) | 011 (mode 3) | 0 (binary)
            // = 0x36 (00110110)
            self.command.write(0x36);

            // Send divisor (low byte, then high byte)
            let divisor = divisor as u16;
            self.channel_0.write((divisor & 0xFF) as u8);
            self.channel_0.write((divisor >> 8) as u8);
        }
    }
}

impl Default for Pit {
    fn default() -> Self {
        Self::new()
    }
}
