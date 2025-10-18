#![no_std]

use core::panic::PanicInfo;

pub mod boot;

pub use boot::{Multiboot2Info, MemoryRegion, MemoryRegionType};

/// Kernel main entry point called from boot64.asm
#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_magic: u32, _multiboot_info: usize) -> ! {
    // TODO: Validate multiboot magic number
    // TODO: Parse multiboot2 info structure
    // TODO: Initialize VGA buffer for output
    // TODO: Initialize memory management

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
