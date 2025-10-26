//! Kernel Heap Initialization
//!
//! This module provides heap initialization functionality for the kernel.
//! It sets up a global allocator using the BumpAllocator implementation.

use super::allocator::{BumpAllocator, Locked};

/// Heap size (100 KB)
///
/// This is a reasonable initial heap size for the kernel.
/// It will be increased as needed in future milestones.
pub const HEAP_SIZE: usize = 100 * 1024;

/// Global allocator
///
/// This is the global allocator used by all heap allocations in the kernel.
/// It uses the BumpAllocator implementation, which is simple but sufficient
/// for initial development.
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

/// Heap area (allocated in BSS section)
///
/// This static array reserves memory for the heap in the BSS section.
/// The BSS section is automatically zeroed by the bootloader.
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Initialize the kernel heap
///
/// This function must be called early in the kernel initialization process,
/// before any heap allocations are made.
///
/// # Safety
///
/// This function is safe to call, but it must be called only once during
/// kernel initialization, before any heap allocations occur.
pub fn init_heap() {
    unsafe {
        let heap_start = core::ptr::addr_of!(HEAP) as usize;
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }

    crate::log_debug!(
        "Heap initialized: start = {:#x}, size = {} KB",
        core::ptr::addr_of!(HEAP) as usize,
        HEAP_SIZE / 1024
    );
}

/// Get current heap usage statistics
///
/// Returns information about heap usage including total size, used size,
/// and number of active allocations.
pub fn heap_usage() -> super::allocator::HeapUsage {
    ALLOCATOR.lock().usage()
}

/// OOM (Out Of Memory) handler
///
/// This function is called when a memory allocation fails.
/// It panics with information about the failed allocation.
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
