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

//! Testing framework for kernel tests
//!
//! This module provides infrastructure for running tests in QEMU with
//! programmatic exit codes.

use core::panic::PanicInfo;

/// Exit codes for QEMU isa-debug-exit device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    /// Test succeeded
    Success = 0x10,
    /// Test failed
    Failed = 0x11,
}

/// Exit QEMU with the given exit code
///
/// This function writes to the isa-debug-exit device at I/O port 0xf4.
/// QEMU must be started with `-device isa-debug-exit,iobase=0xf4,iosize=0x04`
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use crate::interrupts::port::Port;

    unsafe {
        let mut port = Port::<u32>::new(0xf4);
        port.write(exit_code as u32);
    }

    // If QEMU exit fails, halt forever
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Trait for testable functions
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where T: Fn()
{
    fn run(&self) {
        crate::serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        crate::serial_println!("[OK]");
    }
}

/// Test runner that executes all tests
pub fn test_runner(tests: &[&dyn Testable]) {
    crate::serial_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    crate::serial_println!("\nTest result: OK. {} passed; 0 failed", tests.len());
    exit_qemu(QemuExitCode::Success);
}

/// Panic handler for test mode
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    crate::serial_println!("[FAILED]");
    crate::serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_trivial_assertion() {
        assert_eq!(1 + 1, 2);
    }

    #[test_case]
    fn test_exit_codes() {
        assert_eq!(QemuExitCode::Success as u32, 0x10);
        assert_eq!(QemuExitCode::Failed as u32, 0x11);
    }
}
