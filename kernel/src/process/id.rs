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

//! Process identifier type.

/// Unique identifier for a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProcessId(u64);

impl ProcessId {
    /// The kernel process ID (always 0).
    pub const KERNEL: Self = Self(0);

    /// Create a new ProcessId from a raw value.
    ///
    /// # Safety
    /// This should only be called by the process table when allocating PIDs.
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw process ID value.
    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Check if this is the kernel process.
    #[inline(always)]
    pub const fn is_kernel(self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PID({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_id_creation() {
        let pid = ProcessId::new(42);
        assert_eq!(pid.as_u64(), 42);
    }

    #[test]
    fn test_kernel_pid() {
        assert!(ProcessId::KERNEL.is_kernel());
        assert_eq!(ProcessId::KERNEL.as_u64(), 0);
    }

    #[test]
    fn test_non_kernel_pid() {
        let pid = ProcessId::new(1);
        assert!(!pid.is_kernel());
    }

    #[test]
    fn test_pid_equality() {
        let pid1 = ProcessId::new(10);
        let pid2 = ProcessId::new(10);
        let pid3 = ProcessId::new(20);
        assert_eq!(pid1, pid2);
        assert_ne!(pid1, pid3);
    }

    #[test]
    fn test_pid_ordering() {
        let pid1 = ProcessId::new(1);
        let pid2 = ProcessId::new(2);
        assert!(pid1 < pid2);
    }
}
