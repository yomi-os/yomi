use super::address::{Page, PhysAddr, PhysFrame, VirtAddr};
use bitflags::bitflags;

bitflags! {
    /// Page table entry flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PageTableFlags: u64 {
        /// Page is present in memory
        const PRESENT =         1 << 0;
        /// Page is writable
        const WRITABLE =        1 << 1;
        /// Page is accessible from user mode
        const USER_ACCESSIBLE = 1 << 2;
        /// Write-through caching is enabled
        const WRITE_THROUGH =   1 << 3;
        /// Page cache is disabled
        const NO_CACHE =        1 << 4;
        /// Page has been accessed
        const ACCESSED =        1 << 5;
        /// Page has been written to
        const DIRTY =           1 << 6;
        /// Page is a huge page (2MB or 1GB)
        const HUGE_PAGE =       1 << 7;
        /// Page is global
        const GLOBAL =          1 << 8;
        /// Disable execution on this page
        const NO_EXECUTE =      1 << 63;
    }
}

/// Page table entry
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Create a new empty page table entry
    pub const fn new() -> Self {
        Self { entry: 0 }
    }

    /// Check if the entry is unused (all zeros)
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Get the flags of this entry
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }

    /// Get the physical frame mapped by this entry
    pub fn frame(&self) -> Option<PhysFrame> {
        if self.flags().contains(PageTableFlags::PRESENT) {
            // Extract the physical address (bits 12-51)
            let addr = self.entry & 0x000F_FFFF_FFFF_F000;
            Some(PhysFrame::containing_address(PhysAddr::new(addr)))
        } else {
            None
        }
    }

    /// Set the frame and flags for this entry
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        // Ensure the frame address is page-aligned
        debug_assert!(frame.start_address().is_aligned(4096));
        self.entry = frame.start_address().as_u64() | flags.bits();
    }

    /// Set the entry as unused
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Set flags for this entry
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        // Preserve the address bits, update only the flags
        let addr = self.entry & 0x000F_FFFF_FFFF_F000;
        self.entry = addr | flags.bits();
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("entry", &format_args!("{:#x}", self.entry))
            .field("flags", &self.flags())
            .field("frame", &self.frame())
            .finish()
    }
}

/// Page table (512 entries, 4KB aligned)
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Create a new page table with all entries set to unused
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Zero out all entries
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    /// Iterate over the entries
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    /// Iterate mutably over the entries
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

impl core::ops::Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl core::ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl core::fmt::Debug for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("PageTable")
            .field("entries_count", &512)
            .finish()
    }
}

/// Page table manager
pub struct PageTableManager {
    p4_table: &'static mut PageTable,
}

impl PageTableManager {
    /// Get the current page table from CR3
    ///
    /// # Safety
    /// This function is unsafe because it reads from CR3 and creates a mutable reference
    /// to the page table at that address.
    pub unsafe fn current() -> Self {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);

        // Extract the page table address (bits 12-51)
        let p4_table_addr = cr3 & 0x000F_FFFF_FFFF_F000;
        let p4_table = &mut *(p4_table_addr as *mut PageTable);

        Self { p4_table }
    }

    /// Create a PageTableManager from a given P4 table
    ///
    /// # Safety
    /// The caller must ensure that the P4 table is valid and properly initialized.
    pub unsafe fn from_p4_table(p4_table: &'static mut PageTable) -> Self {
        Self { p4_table }
    }

    /// Map a page to a physical frame
    pub fn map_page(
        &mut self,
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        // Get the P4 table address
        let p4_addr = self.p4_table as *mut PageTable;

        // Traverse the page table hierarchy, creating tables as needed
        // We use raw pointers to avoid multiple mutable borrows
        let p4 = unsafe { &mut *p4_addr };
        let p3_addr = Self::next_table_create_ptr(p4, page.p4_index())?;
        let p3 = unsafe { &mut *p3_addr };
        let p2_addr = Self::next_table_create_ptr(p3, page.p3_index())?;
        let p2 = unsafe { &mut *p2_addr };
        let p1_addr = Self::next_table_create_ptr(p2, page.p2_index())?;
        let p1 = unsafe { &mut *p1_addr };

        // Check if the page is already mapped
        if !p1[page.p1_index()].is_unused() {
            return Err("Page already mapped");
        }

        // Set the page table entry
        p1[page.p1_index()].set_frame(frame, flags | PageTableFlags::PRESENT);

        // Flush the TLB for this page
        Self::flush_tlb(page.start_address());

        Ok(())
    }

    /// Unmap a page
    pub fn unmap_page(&mut self, page: Page) -> Result<PhysFrame, &'static str> {
        // Traverse the page table hierarchy
        let p4 = &*self.p4_table;
        let p3 = Self::next_table_ptr(p4, page.p4_index()).ok_or("P3 table not present")?;
        let p3 = unsafe { &*p3 };
        let p2 = Self::next_table_ptr(p3, page.p3_index()).ok_or("P2 table not present")?;
        let p2 = unsafe { &*p2 };
        let p1 = Self::next_table_ptr(p2, page.p2_index()).ok_or("P1 table not present")?;
        let p1 = unsafe { &mut *(p1 as *mut PageTable) };

        // Get the entry
        let entry = &mut p1[page.p1_index()];

        if entry.is_unused() {
            return Err("Page not mapped");
        }

        // Get the frame before clearing the entry
        let frame = entry.frame().ok_or("Entry not present")?;

        // Clear the entry
        entry.set_unused();

        // Flush the TLB for this page
        Self::flush_tlb(page.start_address());

        Ok(frame)
    }

    /// Translate a virtual address to a physical address
    pub fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        // Traverse the page table hierarchy
        let p4 = &*self.p4_table;
        let p3 = Self::next_table_ptr(p4, addr.p4_index())?;
        let p3 = unsafe { &*p3 };
        let p2 = Self::next_table_ptr(p3, addr.p3_index())?;
        let p2 = unsafe { &*p2 };
        let p1 = Self::next_table_ptr(p2, addr.p2_index())?;
        let p1 = unsafe { &*p1 };

        // Get the entry
        let entry = &p1[addr.p1_index()];

        if !entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        // Get the frame and add the offset
        let frame = entry.frame()?;
        let offset = addr.page_offset() as u64;

        Some(frame.start_address() + offset)
    }

    /// Get or create the next level page table (returns raw pointer)
    fn next_table_create_ptr(
        table: &mut PageTable,
        index: usize,
    ) -> Result<*mut PageTable, &'static str> {
        // Check if the entry is already present
        if !table[index].is_unused() {
            return Self::next_table_ptr(table, index)
                .map(|ptr| ptr as *mut PageTable)
                .ok_or("Failed to get next table");
        }

        // TODO: Allocate a new frame from the frame allocator
        // For now, we return an error since frame allocation is not implemented yet
        Err("Frame allocation not implemented")
    }

    /// Get the next level page table (returns raw pointer)
    fn next_table_ptr(table: &PageTable, index: usize) -> Option<*const PageTable> {
        let entry = &table[index];

        if entry.is_unused() || !entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        let frame = entry.frame()?;
        let ptr = frame.start_address().as_u64() as *const PageTable;

        Some(ptr)
    }

    /// Flush the TLB for a single page
    fn flush_tlb(addr: VirtAddr) {
        unsafe {
            core::arch::asm!(
                "invlpg [{}]",
                in(reg) addr.as_u64(),
                options(nostack, preserves_flags)
            );
        }
    }

    /// Flush the entire TLB by reloading CR3
    pub fn flush_tlb_all() {
        unsafe {
            core::arch::asm!(
                "mov {0}, cr3",
                "mov cr3, {0}",
                out(reg) _,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_entry() {
        let mut entry = PageTableEntry::new();
        assert!(entry.is_unused());

        let frame = PhysFrame::containing_address(PhysAddr::new(0x1000));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        entry.set_frame(frame, flags);

        assert!(!entry.is_unused());
        assert!(entry.flags().contains(PageTableFlags::PRESENT));
        assert!(entry.flags().contains(PageTableFlags::WRITABLE));
        assert_eq!(entry.frame(), Some(frame));
    }

    #[test]
    fn test_page_table_indexing() {
        let mut table = PageTable::new();
        assert!(table[0].is_unused());

        let frame = PhysFrame::containing_address(PhysAddr::new(0x1000));
        table[0].set_frame(frame, PageTableFlags::PRESENT);
        assert!(!table[0].is_unused());
    }

    #[test]
    fn test_page_table_flags() {
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        assert!(flags.contains(PageTableFlags::PRESENT));
        assert!(flags.contains(PageTableFlags::WRITABLE));
        assert!(flags.contains(PageTableFlags::USER_ACCESSIBLE));
        assert!(!flags.contains(PageTableFlags::NO_EXECUTE));
    }
}
