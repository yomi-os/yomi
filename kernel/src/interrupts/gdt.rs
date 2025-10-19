//! Global Descriptor Table (GDT) implementation
//!
//! The GDT defines memory segments and system descriptors including the TSS.

use super::tss::TaskStateSegment;
use core::mem;

/// GDT entry structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    /// Creates a null GDT entry
    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    /// Creates a code segment entry
    ///
    /// Access byte: 0x9A
    /// - Present (bit 7): 1
    /// - DPL (bits 5-6): 00 (ring 0)
    /// - Descriptor type (bit 4): 1 (code/data)
    /// - Executable (bit 3): 1
    /// - Direction/Conforming (bit 2): 0
    /// - Readable (bit 1): 1
    /// - Accessed (bit 0): 0
    ///
    /// Granularity byte: 0xA0
    /// - Long mode (bit 5): 1
    /// - Default operation size (bit 6): 0 (must be 0 for 64-bit)
    const fn code_segment() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0x9A, // Present, Ring 0, Code segment, Executable, Readable
            granularity: 0xA0, // Long mode
            base_high: 0,
        }
    }

    /// Creates a data segment entry
    ///
    /// Access byte: 0x92
    /// - Present (bit 7): 1
    /// - DPL (bits 5-6): 00 (ring 0)
    /// - Descriptor type (bit 4): 1 (code/data)
    /// - Executable (bit 3): 0
    /// - Direction/Conforming (bit 2): 0
    /// - Writable (bit 1): 1
    /// - Accessed (bit 0): 0
    const fn data_segment() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0x92, // Present, Ring 0, Data segment, Writable
            granularity: 0,
            base_high: 0,
        }
    }
}

/// TSS descriptor (16 bytes in 64-bit mode)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct TssDescriptor {
    length: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

impl TssDescriptor {
    /// Creates a TSS descriptor for the given TSS
    fn new(tss: &'static TaskStateSegment) -> Self {
        let ptr = tss as *const _ as u64;
        let limit = mem::size_of::<TaskStateSegment>() - 1;

        Self {
            length: limit as u16,
            base_low: (ptr & 0xFFFF) as u16,
            base_middle: ((ptr >> 16) & 0xFF) as u8,
            access: 0x89, // Present, Available TSS
            granularity: 0,
            base_high: ((ptr >> 24) & 0xFF) as u8,
            base_upper: (ptr >> 32) as u32,
            reserved: 0,
        }
    }
}

/// Global Descriptor Table
#[repr(C, align(16))]
struct Gdt {
    null: GdtEntry,
    code: GdtEntry,
    data: GdtEntry,
    tss: TssDescriptor,
}

impl Gdt {
    /// Creates a new GDT
    const fn new() -> Self {
        Self {
            null: GdtEntry::null(),
            code: GdtEntry::code_segment(),
            data: GdtEntry::data_segment(),
            // TSS will be initialized later with actual address
            tss: TssDescriptor {
                length: 0,
                base_low: 0,
                base_middle: 0,
                access: 0,
                granularity: 0,
                base_high: 0,
                base_upper: 0,
                reserved: 0,
            },
        }
    }

    /// Sets the TSS descriptor
    fn set_tss(&mut self, tss: &'static TaskStateSegment) {
        self.tss = TssDescriptor::new(tss);
    }
}

/// Static GDT instance
static mut GDT: Gdt = Gdt::new();

/// GDT pointer structure used by the `lgdt` instruction
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

/// Initializes and loads the GDT with TSS
///
/// This function must be called before loading the IDT to ensure the TSS
/// is properly set up.
pub fn init(tss: &'static TaskStateSegment) {
    unsafe {
        // Set the TSS descriptor in the GDT
        let gdt_ptr_mut = core::ptr::addr_of_mut!(GDT);
        (*gdt_ptr_mut).set_tss(tss);

        // Create GDT pointer
        let gdt_ptr = GdtPointer {
            limit: (mem::size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };

        // Load the GDT
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );

        // Reload segment registers
        // CS (Code Segment) is reloaded using a far return
        core::arch::asm!(
            "push 0x08",           // Code segment selector
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",               // Far return to reload CS
            "2:",
            "mov ax, 0x10",        // Data segment selector
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            tmp = lateout(reg) _,
            out("ax") _,
        );

        // Load the TSS
        // TSS selector is 0x18 (offset of TSS in GDT)
        core::arch::asm!(
            "ltr ax",
            in("ax") 0x18u16,
            options(nostack, preserves_flags)
        );
    }
}
