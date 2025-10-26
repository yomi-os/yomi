//! Task State Segment (TSS) implementation
//!
//! The TSS is used to store stack pointers for privilege level changes
//! and Interrupt Stack Table (IST) entries for critical interrupts.

use core::mem;

/// Size of the double fault stack (16 KiB)
const DOUBLE_FAULT_STACK_SIZE: usize = 16 * 1024;

/// Task State Segment structure for x86_64
///
/// The TSS holds stack pointers that the CPU uses when privilege level changes
/// occur or when IST is used for critical interrupts.
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    /// Privilege Stack Table - Stack pointers for privilege levels 0-2
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    /// Interrupt Stack Table - Stack pointers for critical interrupts (1-7)
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// I/O Map Base Address
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Creates a new TSS with all fields zeroed
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            reserved_1: 0,
            privilege_stack_table: [0; 3],
            reserved_2: 0,
            interrupt_stack_table: [0; 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: 0,
        }
    }
}

/// Static TSS instance
static mut TSS: TaskStateSegment = TaskStateSegment::new();

/// Static stack for double fault handler
///
/// This stack is dedicated to the double fault handler to prevent triple-faults
/// caused by stack corruption.
#[repr(align(16))]
struct DoubleStackStorage {
    storage: [u8; DOUBLE_FAULT_STACK_SIZE],
}

static mut DOUBLE_FAULT_STACK: DoubleStackStorage = DoubleStackStorage {
    storage: [0; DOUBLE_FAULT_STACK_SIZE],
};

/// Initializes the TSS with IST entries
///
/// This function sets up the Interrupt Stack Table (IST) with dedicated stacks
/// for critical interrupts like double fault.
pub fn init() {
    unsafe {
        // Calculate the top of the double fault stack
        let stack_ptr = core::ptr::addr_of!(DOUBLE_FAULT_STACK);
        let stack_start = (*stack_ptr).storage.as_ptr() as u64;
        let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE as u64;

        // Set IST entry 1 for double fault handler
        // IST indices are 1-based in hardware but 0-based in our array
        let tss_ptr = core::ptr::addr_of_mut!(TSS);
        (*tss_ptr).interrupt_stack_table[0] = stack_end;

        // Set IOMAP base to the size of TSS (no I/O permission bitmap)
        (*tss_ptr).iomap_base = mem::size_of::<TaskStateSegment>() as u16;
    }
}

/// Returns a reference to the static TSS
///
/// # Safety
///
/// This function is unsafe because it returns a reference to a static mutable
/// variable.
pub unsafe fn get_tss() -> &'static TaskStateSegment {
    &*core::ptr::addr_of!(TSS)
}
