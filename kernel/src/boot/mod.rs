/// Boot-related functionality
///
/// This module contains boot protocol implementations and
/// early initialization code.
pub mod multiboot2;

pub use multiboot2::{MemoryRegion, MemoryRegionType, Multiboot2Info};
