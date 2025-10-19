pub mod address;
pub mod paging;
pub mod allocator;
pub mod heap;

pub use address::{PhysAddr, VirtAddr, Page, PhysFrame};
pub use paging::{PageTable, PageTableEntry, PageTableFlags, PageTableManager};
pub use allocator::{BumpAllocator, Locked, HeapUsage};
pub use heap::{init_heap, heap_usage, HEAP_SIZE};

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;
