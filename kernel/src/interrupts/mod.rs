//! Interrupt handling subsystem
//!
//! This module provides interrupt and exception handling for the kernel.

pub mod idt;
pub mod handlers;
pub mod tss;
pub mod gdt;

use idt::InterruptDescriptorTable;
use spin::Once;

/// Static IDT instance
///
/// We use `spin::Once` to ensure the IDT is initialized exactly once.
static IDT: Once<InterruptDescriptorTable> = Once::new();

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

        idt
    });

    // Load the IDT into the CPU
    idt.load();

    // TODO: Log initialization when printk is available
    // printk!("IDT initialized");
}

/// Re-exports for convenience
pub use idt::{InterruptStackFrame, HandlerFunc};
