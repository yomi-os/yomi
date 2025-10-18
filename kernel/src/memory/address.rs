/// Physical address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Create a new physical address
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Get the address as u64
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Check if the address is aligned to the given alignment
    pub fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Align down to the given alignment
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    /// Align up to the given alignment
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }
}

impl core::ops::Add<u64> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::Sub<u64> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

/// Virtual address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Create a new virtual address
    /// Ensures the address is in canonical form (48-bit with sign extension)
    pub const fn new(addr: u64) -> Self {
        // 48-bit canonical form (sign-extend upper 16 bits)
        let canonical = ((addr << 16) as i64 >> 16) as u64;
        Self(canonical)
    }

    /// Get the address as u64
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// P4 table index (bits 39-47)
    pub const fn p4_index(self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }

    /// P3 table index (bits 30-38)
    pub const fn p3_index(self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }

    /// P2 table index (bits 21-29)
    pub const fn p2_index(self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }

    /// P1 table index (bits 12-20)
    pub const fn p1_index(self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }

    /// Page offset (bits 0-11)
    pub const fn page_offset(self) -> usize {
        (self.0 & 0xFFF) as usize
    }

    /// Check if the address is aligned to the given alignment
    pub fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Align down to the given alignment
    pub const fn align_down(self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }

    /// Align up to the given alignment
    pub const fn align_up(self, align: u64) -> Self {
        Self::new((self.0 + align - 1) & !(align - 1))
    }
}

impl core::ops::Add<u64> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl core::ops::Sub<u64> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

/// Page (4KB virtual page)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    start_address: VirtAddr,
}

impl Page {
    /// Page size in bytes (4KB)
    pub const SIZE: u64 = 4096;

    /// Create a page containing the given address
    pub const fn containing_address(addr: VirtAddr) -> Self {
        Self {
            start_address: VirtAddr::new(addr.as_u64() & !0xFFF),
        }
    }

    /// Create a page from a start address
    /// The address must be page-aligned
    pub const fn from_start_address(addr: VirtAddr) -> Self {
        Self {
            start_address: addr,
        }
    }

    /// Get the start address of the page
    pub const fn start_address(self) -> VirtAddr {
        self.start_address
    }

    /// Get the P4 index for this page
    pub const fn p4_index(self) -> usize {
        self.start_address.p4_index()
    }

    /// Get the P3 index for this page
    pub const fn p3_index(self) -> usize {
        self.start_address.p3_index()
    }

    /// Get the P2 index for this page
    pub const fn p2_index(self) -> usize {
        self.start_address.p2_index()
    }

    /// Get the P1 index for this page
    pub const fn p1_index(self) -> usize {
        self.start_address.p1_index()
    }
}

impl core::ops::Add<u64> for Page {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self {
            start_address: VirtAddr::new(self.start_address.as_u64() + rhs * Self::SIZE),
        }
    }
}

/// Physical frame (4KB physical page)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    start_address: PhysAddr,
}

impl PhysFrame {
    /// Frame size in bytes (4KB)
    pub const SIZE: u64 = 4096;

    /// Create a frame containing the given address
    pub const fn containing_address(addr: PhysAddr) -> Self {
        Self {
            start_address: PhysAddr::new(addr.as_u64() & !0xFFF),
        }
    }

    /// Create a frame from a start address
    /// The address must be frame-aligned
    pub const fn from_start_address(addr: PhysAddr) -> Self {
        Self {
            start_address: addr,
        }
    }

    /// Get the start address of the frame
    pub const fn start_address(self) -> PhysAddr {
        self.start_address
    }
}

impl core::ops::Add<u64> for PhysFrame {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self {
            start_address: PhysAddr::new(self.start_address.as_u64() + rhs * Self::SIZE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_addr_indices() {
        let addr = VirtAddr::new(0xFFFF_FFFF_8000_1234);
        assert_eq!(addr.p4_index(), 511);
        assert_eq!(addr.p3_index(), 510);
        assert_eq!(addr.p2_index(), 0);
        assert_eq!(addr.p1_index(), 1);
        assert_eq!(addr.page_offset(), 0x234);
    }

    #[test]
    fn test_page_containing_address() {
        let addr = VirtAddr::new(0x1234);
        let page = Page::containing_address(addr);
        assert_eq!(page.start_address().as_u64(), 0x1000);
    }

    #[test]
    fn test_frame_containing_address() {
        let addr = PhysAddr::new(0x5678);
        let frame = PhysFrame::containing_address(addr);
        assert_eq!(frame.start_address().as_u64(), 0x5000);
    }

    #[test]
    fn test_address_alignment() {
        let addr = PhysAddr::new(0x1234);
        assert!(!addr.is_aligned(4096));
        assert_eq!(addr.align_down(4096).as_u64(), 0x1000);
        assert_eq!(addr.align_up(4096).as_u64(), 0x2000);
    }
}
