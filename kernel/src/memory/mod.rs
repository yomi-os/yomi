pub mod address;
pub mod paging;

pub use address::{PhysAddr, VirtAddr, Page, PhysFrame};
pub use paging::{PageTable, PageTableEntry, PageTableFlags, PageTableManager};

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;
