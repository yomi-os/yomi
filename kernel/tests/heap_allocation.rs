//! Heap allocation integration test
//!
//! This test verifies that heap allocation works correctly,
//! including Vec and Box allocations.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yomi_kernel::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{
    boxed::Box,
    vec,
    vec::Vec,
};
use core::panic::PanicInfo;

/// Entry point for heap allocation test
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize kernel subsystems (including heap)
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
fn test_vec_allocation() {
    let mut v = Vec::new();
    for i in 0..100 {
        v.push(i);
    }
    assert_eq!(v.len(), 100);
    assert_eq!(v[50], 50);
}

#[test_case]
fn test_vec_large_allocation() {
    let mut v = Vec::new();
    for i in 0..1000 {
        v.push(i);
    }
    assert_eq!(v.len(), 1000);

    // Verify some values
    assert_eq!(v[0], 0);
    assert_eq!(v[500], 500);
    assert_eq!(v[999], 999);
}

#[test_case]
fn test_box_allocation() {
    let b = Box::new(42);
    assert_eq!(*b, 42);
}

#[test_case]
fn test_box_large_value() {
    // Allocate directly on heap to avoid stack pressure
    let large = vec![0u8; 4096].into_boxed_slice();
    assert_eq!(large.len(), 4096);
    assert_eq!(large[0], 0);
}

#[test_case]
fn test_multiple_allocations() {
    let b1 = Box::new(1);
    let b2 = Box::new(2);
    let b3 = Box::new(3);

    assert_eq!(*b1, 1);
    assert_eq!(*b2, 2);
    assert_eq!(*b3, 3);
}

#[test_case]
fn test_vec_with_strings() {
    use alloc::string::String;

    let mut v = Vec::new();
    v.push(String::from("Hello"));
    v.push(String::from("World"));

    assert_eq!(v.len(), 2);
    assert_eq!(v[0], "Hello");
    assert_eq!(v[1], "World");
}
