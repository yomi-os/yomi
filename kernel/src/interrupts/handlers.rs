//! Exception and interrupt handlers
//!
//! This module contains handler functions for CPU exceptions and hardware
//! interrupts.

use super::idt::InterruptStackFrame;

/// Divide Error (#DE, 0) - Fault
///
/// Occurs when division by zero or division overflow happens.
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: DIVIDE ERROR");
    panic_with_stack_frame("DIVIDE ERROR", stack_frame);
}

/// Debug Exception (#DB, 1) - Fault/Trap
///
/// Occurs when a debug event happens (breakpoint, single-step, etc.).
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    crate::log_debug!("EXCEPTION: DEBUG");
    panic_with_stack_frame("DEBUG", stack_frame);
}

/// Non-Maskable Interrupt (#NMI, 2)
///
/// Hardware NMI interrupt.
pub extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: NON-MASKABLE INTERRUPT");
    panic_with_stack_frame("NON-MASKABLE INTERRUPT", stack_frame);
}

/// Breakpoint Exception (#BP, 3) - Trap
///
/// Occurs when an INT3 instruction is executed.
/// This exception is commonly used by debuggers and should not panic.
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::log_debug!(
        "EXCEPTION: BREAKPOINT at RIP={:#x}",
        stack_frame.instruction_pointer
    );

    // Breakpoint is expected in debugging, so we don't panic
    // Just prevent optimization from removing the stack_frame
    core::hint::black_box(&stack_frame);
}

/// Overflow Exception (#OF, 4) - Trap
///
/// Occurs when an INTO instruction is executed with OF flag set.
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: OVERFLOW");
    panic_with_stack_frame("OVERFLOW", stack_frame);
}

/// BOUND Range Exceeded (#BR, 5) - Fault
///
/// Occurs when a BOUND instruction detects out-of-bounds array access.
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: BOUND RANGE EXCEEDED");
    panic_with_stack_frame("BOUND RANGE EXCEEDED", stack_frame);
}

/// Invalid Opcode (#UD, 6) - Fault
///
/// Occurs when the processor tries to execute an invalid or undefined opcode.
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: INVALID OPCODE");
    panic_with_stack_frame("INVALID OPCODE", stack_frame);
}

/// Device Not Available (#NM, 7) - Fault
///
/// Occurs when an FPU instruction is executed but the FPU is not available.
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: DEVICE NOT AVAILABLE");
    panic_with_stack_frame("DEVICE NOT AVAILABLE", stack_frame);
}

/// Double Fault (#DF, 8) - Abort
///
/// Occurs when an exception occurs while trying to call the handler for a prior
/// exception. This is a critical error that requires special handling.
///
/// Note: This handler never returns.
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    crate::log_fatal!("EXCEPTION: DOUBLE FAULT (Error Code: {:#x})", error_code);
    crate::log_fatal!("  RIP: {:#x}", stack_frame.instruction_pointer);
    crate::log_fatal!("  RSP: {:#x}", stack_frame.stack_pointer);

    // Prevent optimization from removing these
    core::hint::black_box(&stack_frame);
    core::hint::black_box(error_code);

    panic!("EXCEPTION: DOUBLE FAULT");
}

/// Invalid TSS (#TS, 10) - Fault
///
/// Occurs when the processor detects an invalid Task State Segment.
pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::log_error!("EXCEPTION: INVALID TSS (Error Code: {:#x})", error_code);
    core::hint::black_box(error_code);
    panic_with_stack_frame("INVALID TSS", stack_frame);
}

/// Segment Not Present (#NP, 11) - Fault
///
/// Occurs when trying to load a segment with a not-present descriptor.
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::log_error!(
        "EXCEPTION: SEGMENT NOT PRESENT (Error Code: {:#x})",
        error_code
    );
    core::hint::black_box(error_code);
    panic_with_stack_frame("SEGMENT NOT PRESENT", stack_frame);
}

/// Stack-Segment Fault (#SS, 12) - Fault
///
/// Occurs when stack operations reference memory outside the stack segment.
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::log_error!(
        "EXCEPTION: STACK-SEGMENT FAULT (Error Code: {:#x})",
        error_code
    );
    core::hint::black_box(error_code);
    panic_with_stack_frame("STACK-SEGMENT FAULT", stack_frame);
}

/// General Protection Fault (#GP, 13) - Fault
///
/// Occurs when the processor detects a protection violation.
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::log_error!("EXCEPTION: GENERAL PROTECTION FAULT");
    crate::log_error!("  Error Code: {:#x}", error_code);
    crate::log_error!("  RIP: {:#x}", stack_frame.instruction_pointer);

    core::hint::black_box(error_code);
    panic_with_stack_frame("GENERAL PROTECTION FAULT", stack_frame);
}

/// Page Fault (#PF, 14) - Fault
///
/// Occurs when:
/// - A page directory or page table entry is not present
/// - A protection check fails
/// - A reserved bit is set in the page directory or page table
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Read CR2 register to get the faulting address
    let fault_addr: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) fault_addr, options(nomem, nostack, preserves_flags));
    }

    crate::log_error!("EXCEPTION: PAGE FAULT");
    crate::log_error!("  Accessed Address: {:#x}", fault_addr);
    crate::log_error!("  Error Code: {:#x}", error_code);

    // Parse error code:
    let present = (error_code & 0x1) != 0;
    let write = (error_code & 0x2) != 0;
    let user = (error_code & 0x4) != 0;
    let reserved = (error_code & 0x8) != 0;
    let instruction = (error_code & 0x10) != 0;

    crate::log_error!(
        "    Present: {}, Write: {}, User: {}, Reserved: {}, Instruction: {}",
        present,
        write,
        user,
        reserved,
        instruction
    );
    crate::log_error!("  RIP: {:#x}", stack_frame.instruction_pointer);

    core::hint::black_box(fault_addr);
    core::hint::black_box(error_code);
    panic_with_stack_frame("PAGE FAULT", stack_frame);
}

/// x87 Floating-Point Exception (#MF, 16) - Fault
///
/// Occurs when the x87 FPU detects a floating-point error.
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: x87 FLOATING-POINT");
    panic_with_stack_frame("x87 FLOATING-POINT", stack_frame);
}

/// Alignment Check (#AC, 17) - Fault
///
/// Occurs when unaligned memory access is detected with AC flag set.
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::log_error!("EXCEPTION: ALIGNMENT CHECK (Error Code: {:#x})", error_code);
    core::hint::black_box(error_code);
    panic_with_stack_frame("ALIGNMENT CHECK", stack_frame);
}

/// Machine Check (#MC, 18) - Abort
///
/// Occurs when the processor detects internal errors or bus errors.
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    crate::log_fatal!("EXCEPTION: MACHINE CHECK");
    crate::log_fatal!("  RIP: {:#x}", stack_frame.instruction_pointer);

    core::hint::black_box(&stack_frame);
    panic!("EXCEPTION: MACHINE CHECK");
}

/// SIMD Floating-Point Exception (#XM, 19) - Fault
///
/// Occurs when an unmasked SSE floating-point exception is detected.
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    crate::log_error!("EXCEPTION: SIMD FLOATING-POINT");
    panic_with_stack_frame("SIMD FLOATING-POINT", stack_frame);
}

/// Helper function to panic with stack frame information
///
/// This function logs detailed stack frame information before panicking.
#[inline(never)]
fn panic_with_stack_frame(exception_name: &str, stack_frame: InterruptStackFrame) -> ! {
    crate::log_error!("Stack Frame:");
    crate::log_error!(
        "  Instruction Pointer: {:#x}",
        stack_frame.instruction_pointer
    );
    crate::log_error!("  Code Segment:        {:#x}", stack_frame.code_segment);
    crate::log_error!("  CPU Flags:           {:#x}", stack_frame.cpu_flags);
    crate::log_error!("  Stack Pointer:       {:#x}", stack_frame.stack_pointer);
    crate::log_error!("  Stack Segment:       {:#x}", stack_frame.stack_segment);

    // Prevent optimization from removing the stack_frame
    core::hint::black_box(&stack_frame);

    panic!("EXCEPTION: {}", exception_name);
}
