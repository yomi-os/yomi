#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod boot;
pub mod memory;

pub use boot::{Multiboot2Info, MemoryRegion, MemoryRegionType};
pub use memory::{PhysAddr, VirtAddr, Page, PhysFrame, PageTable, PageTableEntry, PageTableFlags, PageTableManager};

use alloc::{vec, boxed::Box};

/// Kernel main entry point called from boot64.asm
#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_magic: u32, _multiboot_info: usize) -> ! {
    // TODO: Validate multiboot magic number
    // TODO: Parse multiboot2 info structure
    // TODO: Initialize VGA buffer for output

    // Initialize heap allocator
    memory::init_heap();

    // Test heap allocation with Vec
    let mut vec = vec![1, 2, 3];
    vec.push(4);
    // TODO: printk!("Vec test: {:?}", vec);

    // Test heap allocation with Box
    let boxed = Box::new(42);
    // TODO: printk!("Box test: {}", boxed);

    // Prevent optimization from removing these allocations
    core::hint::black_box(vec);
    core::hint::black_box(boxed);

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

/// Kernel initialization function
pub fn init() {
    // Kernel initialization code will go here
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
