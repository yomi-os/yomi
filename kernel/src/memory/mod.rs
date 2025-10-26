pub mod address;
pub mod allocator;
pub mod heap;
pub mod paging;

pub use address::{Page, PhysAddr, PhysFrame, VirtAddr};
pub use allocator::{BumpAllocator, HeapUsage, Locked};
pub use heap::{heap_usage, init_heap, HEAP_SIZE};
pub use paging::{PageTable, PageTableEntry, PageTableFlags, PageTableManager};

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;
