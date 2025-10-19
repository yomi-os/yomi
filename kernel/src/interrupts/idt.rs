//! Interrupt Descriptor Table (IDT) implementation
//!
//! This module provides a type-safe interface for setting up and managing
//! the x86_64 Interrupt Descriptor Table.

use core::mem;

/// IDT (Interrupt Descriptor Table) with 256 entries
///
/// The IDT is used by the CPU to determine the correct handler function
/// for interrupts and exceptions.
#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: Entry,                    // 0
    pub debug: Entry,                           // 1
    pub non_maskable_interrupt: Entry,          // 2
    pub breakpoint: Entry,                      // 3
    pub overflow: Entry,                        // 4
    pub bound_range_exceeded: Entry,            // 5
    pub invalid_opcode: Entry,                  // 6
    pub device_not_available: Entry,            // 7
    pub double_fault: Entry,                    // 8
    reserved_1: Entry,                          // 9 (Coprocessor Segment Overrun)
    pub invalid_tss: Entry,                     // 10
    pub segment_not_present: Entry,             // 11
    pub stack_segment_fault: Entry,             // 12
    pub general_protection_fault: Entry,        // 13
    pub page_fault: Entry,                      // 14
    reserved_2: Entry,                          // 15
    pub x87_floating_point: Entry,              // 16
    pub alignment_check: Entry,                 // 17
    pub machine_check: Entry,                   // 18
    pub simd_floating_point: Entry,             // 19
    pub virtualization: Entry,                  // 20
    reserved_3: [Entry; 9],                     // 21-29
    pub security_exception: Entry,              // 30
    reserved_4: Entry,                          // 31
    interrupts: [Entry; 224],                   // 32-255 (IRQs and user-defined)
}

impl InterruptDescriptorTable {
    /// Creates a new IDT with all entries marked as missing
    pub const fn new() -> Self {
        Self {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            reserved_1: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment_fault: Entry::missing(),
            general_protection_fault: Entry::missing(),
            page_fault: Entry::missing(),
            reserved_2: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            reserved_3: [Entry::missing(); 9],
            security_exception: Entry::missing(),
            reserved_4: Entry::missing(),
            interrupts: [Entry::missing(); 224],
        }
    }

    /// Loads this IDT into the CPU using the `lidt` instruction
    ///
    /// # Safety
    ///
    /// This function is unsafe because loading an invalid IDT can cause
    /// undefined behavior.
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            limit: (mem::size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        };

        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

/// IDT entry (16 bytes)
///
/// Each entry describes the location and properties of an interrupt handler.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry {
    pointer_low: u16,       // Bits 0-15 of handler address
    gdt_selector: u16,      // Code segment selector
    options: EntryOptions,  // Type and attributes
    pointer_middle: u16,    // Bits 16-31 of handler address
    pointer_high: u32,      // Bits 32-63 of handler address
    reserved: u32,          // Reserved (must be 0)
}

impl Entry {
    /// Creates a missing (not present) entry
    pub const fn missing() -> Self {
        Self {
            pointer_low: 0,
            gdt_selector: 0,
            options: EntryOptions::minimal(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }

    /// Sets the handler function for this entry
    ///
    /// The handler will be called when this interrupt/exception occurs.
    pub fn set_handler_fn(&mut self, handler: HandlerFunc) -> &mut EntryOptions {
        let addr = handler as u64;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = 0x08; // Kernel code segment
        self.options.set_present(true);

        &mut self.options
    }

    /// Sets the handler function with error code for this entry
    ///
    /// Some exceptions (like Page Fault, General Protection Fault) push
    /// an error code onto the stack.
    pub fn set_handler_fn_with_error_code(
        &mut self,
        handler: HandlerFuncWithErrorCode,
    ) -> &mut EntryOptions {
        let addr = handler as u64;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = 0x08; // Kernel code segment
        self.options.set_present(true);

        &mut self.options
    }

    /// Sets the diverging handler function for this entry
    ///
    /// Used for handlers that never return (like Double Fault).
    pub fn set_handler_fn_diverging(
        &mut self,
        handler: DivergingHandlerFunc,
    ) -> &mut EntryOptions {
        let addr = handler as u64;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = 0x08; // Kernel code segment
        self.options.set_present(true);

        &mut self.options
    }

    /// Sets the diverging handler function with error code for this entry
    pub fn set_handler_fn_diverging_with_error_code(
        &mut self,
        handler: DivergingHandlerFuncWithErrorCode,
    ) -> &mut EntryOptions {
        let addr = handler as u64;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = 0x08; // Kernel code segment
        self.options.set_present(true);

        &mut self.options
    }
}

/// Handler function type without error code
pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);

/// Handler function type with error code
pub type HandlerFuncWithErrorCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);

/// Diverging handler function type (never returns)
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame) -> !;

/// Diverging handler function type with error code
pub type DivergingHandlerFuncWithErrorCode =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64) -> !;

/// Interrupt stack frame
///
/// This structure is automatically pushed onto the stack by the CPU
/// when an interrupt occurs.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

/// Entry options (Type and attributes field)
///
/// Bits 0-3: Gate Type (0b1110 = Interrupt Gate)
/// Bits 4-7: Reserved (0)
/// Bits 8-11: Reserved (0)
/// Bit 12: Reserved (0)
/// Bits 13-14: DPL (Descriptor Privilege Level)
/// Bit 15: Present
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EntryOptions(u16);

impl EntryOptions {
    /// Creates minimal options (Interrupt Gate, DPL=0, not present)
    const fn minimal() -> Self {
        // 0b1110_0000_0000 = Interrupt Gate (bits 8-11 = 0b1110)
        Self(0b1110_0000_0000)
    }

    /// Sets or clears the present bit
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.0 |= 1 << 15;
        } else {
            self.0 &= !(1 << 15);
        }
        self
    }

    /// Sets the privilege level (DPL)
    ///
    /// 0 = Kernel
    /// 3 = User
    #[allow(dead_code)]
    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0 &= !(0b11 << 13); // Clear DPL bits
        self.0 |= (dpl & 0b11) << 13; // Set new DPL
        self
    }
}

/// IDT pointer structure used by the `lidt` instruction
#[repr(C, packed)]
struct DescriptorTablePointer {
    /// Size of the IDT in bytes minus 1
    limit: u16,
    /// Virtual address of the IDT
    base: u64,
}
