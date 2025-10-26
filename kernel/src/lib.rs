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
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(crate::testing::test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

use core::panic::PanicInfo;

pub mod boot;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod serial;
pub mod testing;
pub mod time;

use alloc::{
    boxed::Box,
    vec,
};

pub use boot::{
    MemoryRegion,
    MemoryRegionType,
    Multiboot2Info,
};
use interrupts::timer;
pub use memory::{
    Page,
    PageTable,
    PageTableEntry,
    PageTableFlags,
    PageTableManager,
    PhysAddr,
    PhysFrame,
    VirtAddr,
};

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
