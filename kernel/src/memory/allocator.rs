//! Kernel Heap Allocator
//!
//! This module implements a simple Bump Allocator for kernel heap allocation.
//! The Bump Allocator is a linear allocator that allocates memory by bumping
//! a pointer forward. It's simple but cannot reuse freed memory.
//!
//! This will be replaced with more advanced allocators (Slab, Buddy) in the future.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::Mutex;

/// Bump Allocator (linear allocator)
///
/// Allocates memory by moving a pointer forward. Very simple but cannot reuse memory.
/// Suitable for initial implementation and will be replaced with more sophisticated
/// allocators later.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create a new BumpAllocator
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initialize the heap with a memory region
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `heap_start` points to valid, unused memory
    /// - The memory region `[heap_start, heap_start + heap_size)` is valid
    /// - This function is called only once
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    /// Get the current heap usage statistics
    pub fn usage(&self) -> HeapUsage {
        HeapUsage {
            total: self.heap_end - self.heap_start,
            used: self.next - self.heap_start,
            allocations: self.allocations,
        }
    }
}

/// Heap usage statistics
#[derive(Debug, Clone, Copy)]
pub struct HeapUsage {
    pub total: usize,
    pub used: usize,
    pub allocations: usize,
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        // Align the allocation start address
        let alloc_start = align_up(allocator.next, layout.align());

        // Check for overflow
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        // Check if we have enough space
        if alloc_end > allocator.heap_end {
            // Out of memory
            ptr::null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();
        allocator.allocations -= 1;

        // Bump Allocator essentially does nothing for deallocation
        // We can reset the entire heap when allocation count reaches 0
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}

/// Align address upward to the given alignment
///
/// # Arguments
///
/// * `addr` - The address to align
/// * `align` - The alignment (must be a power of 2)
///
/// # Returns
///
/// The smallest address >= `addr` that is aligned to `align`
fn align_up(addr: usize, align: usize) -> usize {
    // align must be a power of 2
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

/// A wrapper around a type that provides interior mutability with a Mutex
///
/// This allows the allocator to be used as a static global allocator
/// while still being able to mutate its internal state.
pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    /// Create a new Locked wrapper
    pub const fn new(inner: A) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Lock the inner value and return a guard
    pub fn lock(&self) -> spin::MutexGuard<'_, A> {
        self.inner.lock()
    }
}
