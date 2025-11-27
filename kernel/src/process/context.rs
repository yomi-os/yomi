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

//! Process CPU context for context switching.

/// CPU context saved during context switches.
///
/// This structure holds all general-purpose registers, instruction pointer,
/// and flags that need to be preserved when switching between processes.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcessContext {
    /// RAX - Accumulator register
    pub rax: u64,
    /// RBX - Base register
    pub rbx: u64,
    /// RCX - Counter register
    pub rcx: u64,
    /// RDX - Data register
    pub rdx: u64,
    /// RSI - Source index register
    pub rsi: u64,
    /// RDI - Destination index register
    pub rdi: u64,
    /// RBP - Base pointer register
    pub rbp: u64,
    /// RSP - Stack pointer register
    pub rsp: u64,
    /// R8 - General purpose register
    pub r8: u64,
    /// R9 - General purpose register
    pub r9: u64,
    /// R10 - General purpose register
    pub r10: u64,
    /// R11 - General purpose register
    pub r11: u64,
    /// R12 - General purpose register
    pub r12: u64,
    /// R13 - General purpose register
    pub r13: u64,
    /// R14 - General purpose register
    pub r14: u64,
    /// R15 - General purpose register
    pub r15: u64,
    /// RIP - Instruction pointer
    pub rip: u64,
    /// RFLAGS - CPU flags register
    pub rflags: u64,
}

impl ProcessContext {
    /// RFLAGS default value with interrupts enabled (IF=1).
    const DEFAULT_RFLAGS: u64 = 0x200;

    /// Create a new process context with zeroed registers.
    pub const fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: Self::DEFAULT_RFLAGS,
        }
    }

    /// Create a new context for a process entry point.
    ///
    /// # Arguments
    /// * `entry_point` - The address where execution should start
    /// * `stack_top` - The top of the process's stack
    #[inline(always)]
    pub const fn for_entry(entry_point: u64, stack_top: u64) -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: stack_top,
            rsp: stack_top,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry_point,
            rflags: Self::DEFAULT_RFLAGS,
        }
    }

    /// Set the stack pointer.
    #[inline(always)]
    pub fn set_stack_pointer(&mut self, rsp: u64) {
        self.rsp = rsp;
        self.rbp = rsp;
    }

    /// Set the instruction pointer.
    #[inline(always)]
    pub fn set_instruction_pointer(&mut self, rip: u64) {
        self.rip = rip;
    }
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let ctx = ProcessContext::new();
        assert_eq!(ctx.rax, 0);
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rflags, 0x200); // IF=1
    }

    #[test]
    fn test_for_entry() {
        let entry = 0x1000;
        let stack = 0x8000;
        let ctx = ProcessContext::for_entry(entry, stack);

        assert_eq!(ctx.rip, entry);
        assert_eq!(ctx.rsp, stack);
        assert_eq!(ctx.rbp, stack);
        assert_eq!(ctx.rflags, 0x200);
    }

    #[test]
    fn test_set_stack_pointer() {
        let mut ctx = ProcessContext::new();
        ctx.set_stack_pointer(0x5000);
        assert_eq!(ctx.rsp, 0x5000);
        assert_eq!(ctx.rbp, 0x5000);
    }

    #[test]
    fn test_set_instruction_pointer() {
        let mut ctx = ProcessContext::new();
        ctx.set_instruction_pointer(0x2000);
        assert_eq!(ctx.rip, 0x2000);
    }

    #[test]
    fn test_context_size() {
        // Ensure the context is the expected size (18 * 8 = 144 bytes)
        assert_eq!(core::mem::size_of::<ProcessContext>(), 144);
    }
}
