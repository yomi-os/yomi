#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod boot;
use boot::multiboot2::Multiboot2Info;

/// Kernel entry point called from boot.asm
///
/// # Arguments
/// * `magic` - Multiboot2 magic number (should be 0x36d76289)
/// * `info_addr` - Physical address of Multiboot2 information structure
#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, info_addr: usize) -> ! {
    // Initialize multiboot2 information
    let _multiboot_info = unsafe { Multiboot2Info::from_ptr(magic, info_addr) };

    // TODO: Initialize kernel subsystems
    // init();

    // Hang
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
