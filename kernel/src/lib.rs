#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

pub mod boot;
pub mod memory;
pub mod interrupts;
pub mod time;
pub mod serial;
pub mod io;
pub mod testing;

pub use boot::{Multiboot2Info, MemoryRegion, MemoryRegionType};
pub use memory::{PhysAddr, VirtAddr, Page, PhysFrame, PageTable, PageTableEntry, PageTableFlags, PageTableManager};

use alloc::{vec, boxed::Box};
use interrupts::timer;

/// Kernel main entry point called from boot64.asm
#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_magic: u32, _multiboot_info: usize) -> ! {
    // Initialize serial port for logging
    serial::init();

    log_info!("YomiOS Kernel v{}", env!("CARGO_PKG_VERSION"));
    log_debug!("Debug logging enabled");

    // Initialize heap allocator
    log_info!("Initializing memory subsystem...");
    memory::init_heap();
    log_info!("Memory subsystem initialized");

    // Initialize Interrupt Descriptor Table
    log_info!("Initializing interrupt handlers...");
    interrupts::init();
    log_info!("IDT initialized");

    // Enable timer interrupts
    log_info!("Enabling timer interrupts...");
    interrupts::enable_timer_interrupts();
    log_info!("Timer interrupts enabled at {} Hz", timer::TIMER_FREQUENCY);

    // Test breakpoint exception
    // This should be caught by the breakpoint handler and return normally
    log_debug!("Testing breakpoint exception...");
    unsafe {
        core::arch::asm!("int3");
    }
    log_debug!("Breakpoint exception handled successfully");

    // Test heap allocation with Vec
    let mut vec = vec![1, 2, 3];
    vec.push(4);
    log_info!("Vec test passed: {} elements", vec.len());

    // Test heap allocation with Box
    let boxed = Box::new(42);
    log_info!("Box test passed: value = {}", *boxed);

    // Test different log levels
    log_warn!("This is a warning message");
    log_error!("This is an error message (for testing)");

    // Test printk! macro for backward compatibility
    printk!("Kernel initialization complete!");

    // Prevent optimization from removing these allocations
    core::hint::black_box(vec);
    core::hint::black_box(boxed);

    log_info!("Entering idle loop...");

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

/// Kernel initialization function
pub fn init() {
    serial::init();
    memory::init_heap();
    interrupts::init();
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_fatal!("KERNEL PANIC: {}", info);

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
