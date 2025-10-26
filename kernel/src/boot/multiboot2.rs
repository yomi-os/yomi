//! Multiboot2 information handling
//!
//! This module provides types and functions for extracting information
//! passed by the bootloader (GRUB2) via Multiboot2 protocol.

#![allow(dead_code)]

/// Multiboot2 magic number (passed in EAX by bootloader)
pub const MULTIBOOT2_MAGIC: u32 = 0x36d76289;

/// Multiboot2 information structure
pub struct Multiboot2Info {
    /// Address of boot information structure passed by bootloader
    #[allow(dead_code)]
    info_addr: usize,
}

impl Multiboot2Info {
    /// Initialize from magic number and address
    ///
    /// # Safety
    /// The caller must ensure that `info_addr` points to a valid
    /// Multiboot2 information structure in memory.
    pub unsafe fn from_ptr(magic: u32, info_addr: usize) -> Option<Self> {
        if magic == MULTIBOOT2_MAGIC {
            Some(Self { info_addr })
        } else {
            None
        }
    }

    /// Get memory map iterator
    pub fn memory_map(&self) -> impl Iterator<Item = MemoryRegion> {
        // TODO: Parse Multiboot2 information to extract memory map
        core::iter::empty()
    }

    /// Get framebuffer information
    pub fn framebuffer_info(&self) -> Option<FramebufferInfo> {
        // TODO: Parse Multiboot2 information to extract framebuffer info
        None
    }

    /// Get total memory size
    pub fn total_memory(&self) -> Option<usize> {
        // TODO: Parse Multiboot2 information to extract memory info
        None
    }
}

/// Memory region descriptor
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Base physical address of the region
    pub base_addr: u64,
    /// Length of the region in bytes
    pub length: u64,
    /// Type of memory region
    pub region_type: MemoryRegionType,
}

/// Memory region type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryRegionType {
    /// Usable RAM
    Usable = 1,
    /// Reserved by hardware/BIOS
    Reserved = 2,
    /// ACPI reclaimable memory
    AcpiReclaimable = 3,
    /// ACPI NVS memory
    AcpiNvs = 4,
    /// Bad memory
    BadMemory = 5,
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Physical address of framebuffer
    pub addr: u64,
    /// Pitch (bytes per line)
    pub pitch: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Bits per pixel
    pub bpp: u8,
    /// Framebuffer type
    pub fb_type: FramebufferType,
}

/// Framebuffer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FramebufferType {
    /// Indexed color
    Indexed = 0,
    /// RGB color
    Rgb = 1,
    /// EGA text mode
    EgaText = 2,
}
