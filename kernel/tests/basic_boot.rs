//! Basic boot integration test
//!
//! This test verifies that the kernel can boot successfully
//! and that basic functionality works.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yomi_kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use yomi_kernel::{serial_print, serial_println};

/// Entry point for basic boot test
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize kernel subsystems
    yomi_kernel::init();

    // Run tests
    test_main();

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Panic handler for test mode
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    yomi_kernel::testing::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    serial_print!("test_println... ");
    serial_println!("Testing serial output");
    serial_println!("Multiple lines work!");
}

#[test_case]
fn test_trivial_assertion() {
    assert_eq!(1 + 1, 2);
    assert_ne!(1, 2);
    assert!(true);
}

#[test_case]
fn test_kernel_boots() {
    // If we reach here, kernel has booted successfully
    serial_println!("Kernel booted successfully");
}
