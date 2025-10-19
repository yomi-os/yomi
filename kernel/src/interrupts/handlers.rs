//! Exception and interrupt handlers
//!
//! This module contains handler functions for CPU exceptions and hardware interrupts.

use super::idt::InterruptStackFrame;

/// Divide Error (#DE, 0) - Fault
///
/// Occurs when division by zero or division overflow happens.
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);

    // For now, just halt
    panic_with_stack_frame("DIVIDE ERROR", stack_frame);
}

/// Debug Exception (#DB, 1) - Fault/Trap
///
/// Occurs when a debug event happens (breakpoint, single-step, etc.).
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: DEBUG\n{:#?}", stack_frame);

    panic_with_stack_frame("DEBUG", stack_frame);
}

/// Non-Maskable Interrupt (#NMI, 2)
///
/// Hardware NMI interrupt.
pub extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);

    panic_with_stack_frame("NON-MASKABLE INTERRUPT", stack_frame);
}

/// Breakpoint Exception (#BP, 3) - Trap
///
/// Occurs when an INT3 instruction is executed.
/// This exception is commonly used by debuggers and should not panic.
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);

    // Breakpoint is expected in debugging, so we don't panic
    // Just prevent optimization from removing the stack_frame
    core::hint::black_box(&stack_frame);
}

/// Overflow Exception (#OF, 4) - Trap
///
/// Occurs when an INTO instruction is executed with OF flag set.
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);

    panic_with_stack_frame("OVERFLOW", stack_frame);
}

/// BOUND Range Exceeded (#BR, 5) - Fault
///
/// Occurs when a BOUND instruction detects out-of-bounds array access.
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);

    panic_with_stack_frame("BOUND RANGE EXCEEDED", stack_frame);
}

/// Invalid Opcode (#UD, 6) - Fault
///
/// Occurs when the processor tries to execute an invalid or undefined opcode.
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);

    panic_with_stack_frame("INVALID OPCODE", stack_frame);
}

/// Device Not Available (#NM, 7) - Fault
///
/// Occurs when an FPU instruction is executed but the FPU is not available.
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);

    panic_with_stack_frame("DEVICE NOT AVAILABLE", stack_frame);
}

/// Double Fault (#DF, 8) - Abort
///
/// Occurs when an exception occurs while trying to call the handler for a prior exception.
/// This is a critical error that requires special handling.
///
/// Note: This handler never returns.
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: DOUBLE FAULT (Error Code: 0x{:x})\n{:#?}", error_code, stack_frame);

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
    // TODO: Use printk! when available
    // printk!("EXCEPTION: INVALID TSS (Error Code: 0x{:x})\n{:#?}", error_code, stack_frame);

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
    // TODO: Use printk! when available
    // printk!("EXCEPTION: SEGMENT NOT PRESENT (Error Code: 0x{:x})\n{:#?}", error_code, stack_frame);

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
    // TODO: Use printk! when available
    // printk!("EXCEPTION: STACK-SEGMENT FAULT (Error Code: 0x{:x})\n{:#?}", error_code, stack_frame);

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
    // TODO: Use printk! when available
    // printk!("EXCEPTION: GENERAL PROTECTION FAULT");
    // printk!("Error Code: 0x{:x}", error_code);
    // printk!("{:#?}", stack_frame);

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

    // TODO: Use printk! when available
    // printk!("EXCEPTION: PAGE FAULT");
    // printk!("Accessed Address: 0x{:016x}", fault_addr);
    // printk!("Error Code: 0x{:x}", error_code);
    // Parse error code:
    // Bit 0: Present (0 = not present, 1 = protection violation)
    // Bit 1: Write (0 = read, 1 = write)
    // Bit 2: User (0 = kernel, 1 = user)
    // Bit 3: Reserved Write (1 = reserved bit set)
    // Bit 4: Instruction Fetch (1 = instruction fetch)
    // printk!("{:#?}", stack_frame);

    core::hint::black_box(fault_addr);
    core::hint::black_box(error_code);
    panic_with_stack_frame("PAGE FAULT", stack_frame);
}

/// x87 Floating-Point Exception (#MF, 16) - Fault
///
/// Occurs when the x87 FPU detects a floating-point error.
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: x87 FLOATING-POINT\n{:#?}", stack_frame);

    panic_with_stack_frame("x87 FLOATING-POINT", stack_frame);
}

/// Alignment Check (#AC, 17) - Fault
///
/// Occurs when unaligned memory access is detected with AC flag set.
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: ALIGNMENT CHECK (Error Code: 0x{:x})\n{:#?}", error_code, stack_frame);

    core::hint::black_box(error_code);
    panic_with_stack_frame("ALIGNMENT CHECK", stack_frame);
}

/// Machine Check (#MC, 18) - Abort
///
/// Occurs when the processor detects internal errors or bus errors.
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);

    core::hint::black_box(&stack_frame);
    panic!("EXCEPTION: MACHINE CHECK");
}

/// SIMD Floating-Point Exception (#XM, 19) - Fault
///
/// Occurs when an unmasked SSE floating-point exception is detected.
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    // TODO: Use printk! when available
    // printk!("EXCEPTION: SIMD FLOATING-POINT\n{:#?}", stack_frame);

    panic_with_stack_frame("SIMD FLOATING-POINT", stack_frame);
}

/// Helper function to panic with stack frame information
///
/// This is a temporary solution until we have proper logging infrastructure.
#[inline(never)]
fn panic_with_stack_frame(exception_name: &str, stack_frame: InterruptStackFrame) -> ! {
    // Prevent optimization from removing the stack_frame
    core::hint::black_box(&stack_frame);

    // TODO: When printk is available, print detailed information:
    // - Exception name
    // - Instruction pointer
    // - Code segment
    // - CPU flags
    // - Stack pointer
    // - Stack segment

    panic!("EXCEPTION: {}", exception_name);
}
