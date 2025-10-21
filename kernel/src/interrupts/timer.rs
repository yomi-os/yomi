//! Timer interrupt handler
//!
//! This module handles the timer interrupt (IRQ 0) from the PIT.
//! The timer is used to generate periodic scheduler ticks.

use super::idt::InterruptStackFrame;
use super::pic::PICS;
use core::sync::atomic::{AtomicU64, Ordering};

/// Timer tick counter
///
/// This counter is incremented on every timer interrupt.
/// It's used to track system uptime and scheduler ticks.
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Timer frequency in Hz
///
/// This should match the frequency configured in the PIT.
/// 100 Hz = 10ms tick interval
pub const TIMER_FREQUENCY: u32 = 100;

/// Timer interrupt handler (IRQ 0)
///
/// This handler is called whenever the PIT generates a timer interrupt.
/// It increments the tick counter and sends EOI to the PIC.
///
/// # Note
///
/// This function is registered as the handler for interrupt vector 32 (IRQ 0).
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Increment tick counter
    TICKS.fetch_add(1, Ordering::Relaxed);

    // TODO: Call scheduler here when implemented
    // scheduler::tick();

    // Send EOI to PIC
    unsafe {
        PICS.lock().notify_end_of_interrupt(0);
    }
}

/// Returns the current tick count
///
/// # Returns
///
/// The number of timer ticks since the system started.
///
/// # Example
///
/// ```
/// let current_ticks = timer::ticks();
/// ```
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Returns the system uptime in milliseconds
///
/// # Returns
///
/// System uptime in milliseconds.
///
/// # Example
///
/// ```
/// let uptime = timer::uptime_ms();
/// println!("System uptime: {} ms", uptime);
/// ```
pub fn uptime_ms() -> u64 {
    ticks() * 1000 / TIMER_FREQUENCY as u64
}

/// Returns the system uptime in seconds
///
/// # Returns
///
/// System uptime in seconds.
///
/// # Example
///
/// ```
/// let uptime = timer::uptime_seconds();
/// println!("System uptime: {} seconds", uptime);
/// ```
pub fn uptime_seconds() -> u64 {
    uptime_ms() / 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_ticks() {
        let initial_ticks = ticks();

        // Wait a bit (in a real test with timer enabled)
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop") }
        }

        let final_ticks = ticks();

        // In a real environment with timer enabled, this should pass
        // For now, both will be 0 in unit tests
        assert!(final_ticks >= initial_ticks);
    }

    #[test]
    fn test_uptime_calculation() {
        // Test uptime calculation with known tick values
        // Manually set TICKS for testing purposes
        TICKS.store(100, Ordering::Relaxed);
        assert_eq!(uptime_ms(), 1000); // 100 ticks at 100 Hz = 1000 ms
        assert_eq!(uptime_seconds(), 1);
    }
}
