// Copyright 2025 Yomi OS Development Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

// Use modules from the library crate instead of redeclaring them
// This prevents duplicate compilation of modules with separate statics/macros
use alloc::{
    boxed::Box,
    vec,
};

use interrupts::timer;
use yomi_kernel::{
    boot,
    interrupts,
    log_debug,
    log_error,
    log_fatal,
    log_info,
    log_warn,
    memory,
    printk,
    serial,
    serial_println,
    vga,
    // Import macros exported by the library
    vga_println,
};

/// Kernel entry point called from boot.asm
///
/// # Arguments
/// * `magic` - Multiboot2 magic number (should be 0x36d76289)
/// * `info_addr` - Physical address of Multiboot2 information structure
#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, info_addr: usize) -> ! {
    // Initialize VGA for early boot debugging
    // This must come first as it provides fallback output if serial fails
    unsafe {
        vga::init();
    }
    vga_println!("YomiOS Boot");
    vga::write_diagnostic("[BOOT]");

    // Initialize serial port for logging
    serial::init();

    // Test raw serial output immediately after init
    serial_println!("=== YomiOS Serial Console Test ===");
    serial_println!("Serial port initialized successfully!");

    log_info!("YomiOS Kernel v{}", env!("CARGO_PKG_VERSION"));
    log_debug!("Debug logging enabled");
    vga::write_diagnostic("[SERIAL]");

    // Validate Multiboot2 boot
    unsafe {
        if let Some(mbi) = boot::multiboot2::Multiboot2Info::from_ptr(magic, info_addr) {
            log_info!("Multiboot2 boot validated");
            // Store multiboot info for later use (e.g., memory map parsing)
            core::hint::black_box(mbi);
        } else {
            log_fatal!("Invalid Multiboot2 magic: 0x{:08x}", magic);
            panic!("Invalid Multiboot2 boot (magic mismatch)");
        }
    }

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

    // Hang - timer interrupts will continue to fire
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// Panic handler is provided by the library (yomi_kernel::panic)
