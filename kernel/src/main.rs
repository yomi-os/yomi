#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod serial;
mod vga_buffer;

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("🦀 Yomi Kernel Starting...");
    vga_println!("Welcome to Yomi!");

    #[cfg(test)]
    test_main();

    serial_println!("Kernel initialization complete!");

    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
