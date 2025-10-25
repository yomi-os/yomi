//! Interrupt handling subsystem
//!
//! This module provides interrupt and exception handling for the kernel.

pub mod idt;
pub mod handlers;
pub mod tss;
pub mod gdt;
pub mod port;
pub mod pic;
pub mod pit;
pub mod timer;

use idt::InterruptDescriptorTable;
use spin::Once;

/// Static IDT instance
///
/// We use `spin::Once` to ensure the IDT is initialized exactly once.
static IDT: Once<InterruptDescriptorTable> = Once::new();

/// IRQ (Interrupt Request) offsets
///
/// Hardware interrupts (IRQs) are mapped to interrupt vectors 32-47.
/// - IRQ 0-7: Master PIC (vectors 32-39)
/// - IRQ 8-15: Slave PIC (vectors 40-47)
const IRQ_OFFSET: usize = 32;

/// Initializes the Interrupt Descriptor Table
///
/// This function sets up the GDT, TSS with IST stacks, and all exception handlers.
/// It should be called early in the kernel initialization process.
///
/// # Panics
///
/// Panics if called more than once.
pub fn init() {
    // Step 1: Initialize TSS with IST stacks
    tss::init();

    // Step 2: Load GDT with TSS descriptor
    unsafe {
        gdt::init(tss::get_tss());
    }

    // Step 3: Initialize and load IDT
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        // CPU Exception handlers
        idt.divide_error.set_handler_fn(handlers::divide_error_handler);
        idt.debug.set_handler_fn(handlers::debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(handlers::non_maskable_interrupt_handler);
        idt.breakpoint.set_handler_fn(handlers::breakpoint_handler);
        idt.overflow.set_handler_fn(handlers::overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(handlers::bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(handlers::invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(handlers::device_not_available_handler);

        // Double Fault handler (diverging)
        // Use IST index 1 for dedicated stack to prevent triple-fault on stack corruption
        idt.double_fault
            .set_handler_fn_diverging_with_error_code(handlers::double_fault_handler)
            .set_ist(1);

        // Handlers with error codes
        idt.invalid_tss
            .set_handler_fn_with_error_code(handlers::invalid_tss_handler);
        idt.segment_not_present
            .set_handler_fn_with_error_code(handlers::segment_not_present_handler);
        idt.stack_segment_fault
            .set_handler_fn_with_error_code(handlers::stack_segment_fault_handler);
        idt.general_protection_fault
            .set_handler_fn_with_error_code(handlers::general_protection_fault_handler);
        idt.page_fault
            .set_handler_fn_with_error_code(handlers::page_fault_handler);

        // More exception handlers
        idt.x87_floating_point.set_handler_fn(handlers::x87_floating_point_handler);
        idt.alignment_check
            .set_handler_fn_with_error_code(handlers::alignment_check_handler);
        idt.machine_check
            .set_handler_fn_diverging(handlers::machine_check_handler);
        idt.simd_floating_point.set_handler_fn(handlers::simd_floating_point_handler);

        // Hardware interrupt handlers (IRQs)
        // Timer (IRQ 0 → vector 32)
        idt.get_interrupt_entry_mut(0).set_handler_fn(timer::timer_interrupt_handler);

        idt
    });

    // Load the IDT into the CPU
    idt.load();

    crate::log_debug!("IDT loaded with {} exception handlers", 20);
}

/// Initializes and enables timer interrupts
///
/// This function:
/// 1. Initializes the PIC (Programmable Interrupt Controller)
/// 2. Configures the PIT (Programmable Interval Timer) to the desired frequency
/// 3. Unmasks the timer interrupt (IRQ 0)
/// 4. Enables interrupts globally
///
/// # Safety
///
/// This function should only be called once during kernel initialization,
/// after the IDT has been set up.
///
/// # Panics
///
/// Panics if called before `init()`.
pub fn enable_timer_interrupts() {
    // Step 1: Initialize PIC
    unsafe {
        pic::PICS.lock().initialize();
    }

    // Step 2: Configure PIT to desired frequency
    let mut pit = pit::Pit::new();
    pit.set_frequency(timer::TIMER_FREQUENCY);

    // Step 3: Enable timer interrupt (IRQ 0)
    unsafe {
        pic::PICS.lock().unmask(0);
    }

    // Step 4: Enable interrupts globally
    unsafe {
        core::arch::asm!("sti");
    }

    crate::log_debug!("PIC and PIT initialized, interrupts enabled");
}

/// Disables interrupts
///
/// # Safety
///
/// This function modifies the CPU's interrupt flag.
/// It should only be used when necessary to prevent race conditions.
#[inline]
pub unsafe fn disable() {
    core::arch::asm!("cli", options(nomem, nostack));
}

/// Enables interrupts
///
/// # Safety
///
/// This function modifies the CPU's interrupt flag.
/// It should only be called when it's safe to handle interrupts.
#[inline]
pub unsafe fn enable() {
    core::arch::asm!("sti", options(nomem, nostack));
}

/// Checks if interrupts are enabled
///
/// # Returns
///
/// `true` if interrupts are enabled, `false` otherwise
#[inline]
pub fn are_enabled() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));
    }
    (flags & (1 << 9)) != 0 // Check IF (Interrupt Flag) bit
}

/// Executes a closure with interrupts disabled
///
/// This function disables interrupts before executing the closure,
/// and restores the previous interrupt state afterward.
///
/// # Safety
///
/// This function is safe because it properly restores the interrupt state.
///
/// # Examples
///
/// ```
/// interrupts::without_interrupts(|| {
///     // Critical section - interrupts are disabled here
/// });
/// ```
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let enabled = are_enabled();

    if enabled {
        unsafe { disable(); }
    }

    let result = f();

    if enabled {
        unsafe { enable(); }
    }

    result
}

/// Re-exports for convenience
pub use idt::{InterruptStackFrame, HandlerFunc};
