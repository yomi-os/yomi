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

//! Comprehensive kernel panic handler with debugging information
//!
//! This module provides a detailed panic handler that displays:
//! - Panic message and source location
//! - Stack trace (via RBP chain unwinding)
//! - CPU register state
//! - Control register values
//!
//! The panic handler is designed to help debug kernel issues by providing
//! as much context as possible about the system state at the time of panic.

use core::panic::PanicInfo;

use crate::{
    println,
    vga_println,
};

/// Main panic handler implementation
///
/// This function is called when a kernel panic occurs. It:
/// 1. Disables interrupts to prevent further corruption
/// 2. Prints panic information (message, location)
/// 3. Prints stack trace
/// 4. Prints CPU register state
/// 5. Halts the system
///
/// # Arguments
///
/// * `info` - Panic information containing message and location
pub fn panic_handler(info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further issues
    unsafe {
        crate::interrupts::disable();
    }

    // Print panic banner (to both VGA and serial)
    println!();
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("!!!     KERNEL PANIC             !!!");
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!();

    // Also output to VGA in case serial is not working
    vga_println!();
    vga_println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    vga_println!("!!!     KERNEL PANIC             !!!");
    vga_println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    vga_println!();

    // Print panic location (to both VGA and serial)
    if let Some(location) = info.location() {
        println!(
            "Panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        vga_println!(
            "Panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        println!("Panic at unknown location");
        vga_println!("Panic at unknown location");
    }

    // Print panic message
    let message = info.message();
    println!("Message: {}", message);
    vga_println!("Message: {}", message);

    println!();

    // Print uptime
    print_uptime();

    println!();

    // Print additional debug info
    print_stack_trace();
    print_register_dump();

    println!();
    println!("System halted.");

    // Halt the system
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}

/// Print system uptime
fn print_uptime() {
    let uptime_ms = crate::interrupts::timer::uptime_ms();
    let total_secs = uptime_ms / 1000;
    let ms = uptime_ms % 1000;

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    println!("Uptime: {}h {}m {}s {}ms", hours, minutes, seconds, ms);
}

/// Print stack trace by walking the RBP chain
///
/// This function walks the stack frame chain by following RBP pointers.
/// For each frame, it prints the frame number, return address (RIP), and
/// frame pointer (RBP).
///
/// # Safety
///
/// This function performs unsafe pointer dereferences. It validates addresses
/// before dereferencing to prevent further crashes during panic handling.
fn print_stack_trace() {
    println!("Stack trace:");

    let mut rbp: u64;
    unsafe {
        core::arch::asm!(
            "mov {}, rbp",
            out(reg) rbp,
            options(nomem, nostack, preserves_flags)
        );
    }

    let mut frame = 0;
    while frame < 10 && rbp != 0 {
        // Read return address from stack frame
        let rip = unsafe {
            let rip_ptr = (rbp + 8) as *const u64;
            if is_valid_kernel_address(rip_ptr as u64) {
                *rip_ptr
            } else {
                break;
            }
        };

        // Print frame
        println!("  #{}: RIP={:#018x} RBP={:#018x}", frame, rip, rbp);

        // Move to previous frame
        rbp = unsafe {
            let rbp_ptr = rbp as *const u64;
            if is_valid_kernel_address(rbp_ptr as u64) {
                *rbp_ptr
            } else {
                break;
            }
        };

        frame += 1;
    }

    if frame == 0 {
        println!("  (no stack trace available)");
    }
}

/// Check if an address is a valid kernel address
///
/// Kernel addresses should be in the higher half of the address space.
/// On x86_64, kernel space typically starts at 0xFFFF800000000000.
///
/// # Arguments
///
/// * `addr` - The address to validate
///
/// # Returns
///
/// `true` if the address is in kernel space, `false` otherwise
fn is_valid_kernel_address(addr: u64) -> bool {
    // Kernel space: 0xFFFF800000000000 and above
    // Also check it's not null and not too high
    (0xffff_8000_0000_0000..0xffff_ffff_ffff_ffff).contains(&addr)
}

/// Print CPU register dump
///
/// This function captures and displays the current state of:
/// - General purpose registers (RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, R8-R15)
/// - Instruction pointer (RIP)
/// - Flags register (RFLAGS)
/// - Control registers (CR0, CR2, CR3, CR4)
///
/// # Note
///
/// Register values are captured at the time of this function call,
/// not at the exact moment of panic. The values are approximate.
fn print_register_dump() {
    println!("CPU Registers:");

    let (rax, rbx, rcx, rdx): (u64, u64, u64, u64);
    let (rsi, rdi, rbp, rsp): (u64, u64, u64, u64);
    let (r8, r9, r10, r11): (u64, u64, u64, u64);
    let (r12, r13, r14, r15): (u64, u64, u64, u64);
    let (rip, rflags): (u64, u64);
    let (cr0, cr2, cr3, cr4): (u64, u64, u64, u64);

    unsafe {
        core::arch::asm!(
            "mov {}, rax",
            "mov {}, rbx",
            "mov {}, rcx",
            "mov {}, rdx",
            out(reg) rax,
            out(reg) rbx,
            out(reg) rcx,
            out(reg) rdx,
            options(nomem, nostack, preserves_flags)
        );

        core::arch::asm!(
            "mov {}, rsi",
            "mov {}, rdi",
            "mov {}, rbp",
            "mov {}, rsp",
            out(reg) rsi,
            out(reg) rdi,
            out(reg) rbp,
            out(reg) rsp,
            options(nomem, nostack, preserves_flags)
        );

        core::arch::asm!(
            "mov {}, r8",
            "mov {}, r9",
            "mov {}, r10",
            "mov {}, r11",
            out(reg) r8,
            out(reg) r9,
            out(reg) r10,
            out(reg) r11,
            options(nomem, nostack, preserves_flags)
        );

        core::arch::asm!(
            "mov {}, r12",
            "mov {}, r13",
            "mov {}, r14",
            "mov {}, r15",
            out(reg) r12,
            out(reg) r13,
            out(reg) r14,
            out(reg) r15,
            options(nomem, nostack, preserves_flags)
        );

        // Get RIP (instruction pointer) - use return address approximation
        core::arch::asm!(
            "lea {}, [rip]",
            out(reg) rip,
            options(nomem, nostack, preserves_flags)
        );

        // Get RFLAGS
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
            options(preserves_flags)
        );

        // Control registers
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
    }

    println!("  RAX: {:#018x}  RBX: {:#018x}", rax, rbx);
    println!("  RCX: {:#018x}  RDX: {:#018x}", rcx, rdx);
    println!("  RSI: {:#018x}  RDI: {:#018x}", rsi, rdi);
    println!("  RBP: {:#018x}  RSP: {:#018x}", rbp, rsp);
    println!("  R8:  {:#018x}  R9:  {:#018x}", r8, r9);
    println!("  R10: {:#018x}  R11: {:#018x}", r10, r11);
    println!("  R12: {:#018x}  R13: {:#018x}", r12, r13);
    println!("  R14: {:#018x}  R15: {:#018x}", r14, r15);
    println!("  RIP: {:#018x}  RFLAGS: {:#018x}", rip, rflags);
    println!();
    println!("  CR0: {:#018x}  CR2: {:#018x}", cr0, cr2);
    println!("  CR3: {:#018x}  CR4: {:#018x}", cr3, cr4);
}
