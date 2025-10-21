#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

mod boot;
mod memory;
mod interrupts;
mod time;
mod serial;

use alloc::{vec, boxed::Box};

/// Kernel entry point called from boot.asm
///
/// # Arguments
/// * `magic` - Multiboot2 magic number (should be 0x36d76289)
/// * `info_addr` - Physical address of Multiboot2 information structure
#[no_mangle]
pub extern "C" fn kernel_main(_magic: u32, _info_addr: usize) -> ! {
    // Initialize multiboot2 information
    // let _multiboot_info = unsafe { Multiboot2Info::from_ptr(magic, info_addr) };

    // Initialize heap allocator
    memory::init_heap();

    // Initialize Interrupt Descriptor Table
    interrupts::init();

    // Enable timer interrupts
    interrupts::enable_timer_interrupts();

    // Test breakpoint exception
    // This should be caught by the breakpoint handler and return normally
    unsafe {
        core::arch::asm!("int3");
    }

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

    // Hang - timer interrupts will continue to fire
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
