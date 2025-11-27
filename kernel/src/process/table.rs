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

//! Process table for managing all processes in the system.

use alloc::collections::BTreeMap;

use super::{
    id::ProcessId,
    process::Process,
    state::ProcessState,
};

/// Maximum number of processes the system can handle.
pub const MAX_PROCESSES: usize = 65536;

/// Errors that can occur during process table operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    /// The process table is full.
    TableFull,
    /// The specified process was not found.
    ProcessNotFound,
    /// Invalid PID provided.
    InvalidPid,
    /// The process is in an invalid state for the requested operation.
    InvalidState,
}

impl core::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TableFull => write!(f, "process table is full"),
            Self::ProcessNotFound => write!(f, "process not found"),
            Self::InvalidPid => write!(f, "invalid process ID"),
            Self::InvalidState => write!(f, "invalid process state"),
        }
    }
}

/// Table managing all processes in the system.
#[derive(Debug)]
pub struct ProcessTable {
    /// Map of process ID to process.
    processes: BTreeMap<ProcessId, Process>,
    /// Next PID to allocate.
    next_pid: u64,
}

impl ProcessTable {
    /// Create a new empty process table.
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            // Start at 1, since 0 is reserved for the kernel
            next_pid: 1,
        }
    }

    /// Allocate the next available process ID.
    fn allocate_pid(&mut self) -> Result<ProcessId, ProcessError> {
        if self.processes.len() >= MAX_PROCESSES {
            return Err(ProcessError::TableFull);
        }

        // Find the next available PID
        let start_pid = self.next_pid;
        loop {
            let pid = ProcessId::new(self.next_pid);

            // Wrap around at MAX_PROCESSES, but skip 0 (kernel)
            self.next_pid = if self.next_pid >= MAX_PROCESSES as u64 - 1 {
                1
            } else {
                self.next_pid + 1
            };

            // Check if this PID is available
            if !self.processes.contains_key(&pid) {
                return Ok(pid);
            }

            // If we've wrapped around completely, the table is full
            if self.next_pid == start_pid {
                return Err(ProcessError::TableFull);
            }
        }
    }

    /// Add a process to the table and assign it a PID.
    ///
    /// Returns the assigned process ID on success.
    pub fn add_process(&mut self, mut process: Process) -> Result<ProcessId, ProcessError> {
        let pid = self.allocate_pid()?;
        process.set_pid(pid);
        self.processes.insert(pid, process);
        Ok(pid)
    }

    /// Remove a process from the table.
    ///
    /// Returns the removed process on success.
    pub fn remove_process(&mut self, pid: ProcessId) -> Result<Process, ProcessError> {
        self.processes
            .remove(&pid)
            .ok_or(ProcessError::ProcessNotFound)
    }

    /// Get an immutable reference to a process.
    #[inline(always)]
    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.get(&pid)
    }

    /// Get a mutable reference to a process.
    #[inline(always)]
    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    /// Check if a process exists in the table.
    #[inline(always)]
    pub fn contains(&self, pid: ProcessId) -> bool {
        self.processes.contains_key(&pid)
    }

    /// Get the number of processes in the table.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.processes.len()
    }

    /// Check if the table is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    /// Iterate over all processes.
    pub fn iter(&self) -> impl Iterator<Item = (&ProcessId, &Process)> {
        self.processes.iter()
    }

    /// Iterate over all processes mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ProcessId, &mut Process)> {
        self.processes.iter_mut()
    }

    /// Get all processes in a specific state.
    pub fn processes_in_state(&self, state: ProcessState) -> impl Iterator<Item = &Process> {
        self.processes.values().filter(move |p| p.state == state)
    }

    /// Get all ready processes.
    pub fn ready_processes(&self) -> impl Iterator<Item = &Process> {
        self.processes_in_state(ProcessState::Ready)
    }

    /// Mark a process as terminated and prepare it for removal.
    pub fn terminate_process(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        let process = self
            .processes
            .get_mut(&pid)
            .ok_or(ProcessError::ProcessNotFound)?;

        if process.state == ProcessState::Terminated {
            return Err(ProcessError::InvalidState);
        }

        process.mark_terminated();
        Ok(())
    }

    /// Remove all terminated processes from the table.
    ///
    /// Returns the number of processes removed.
    pub fn cleanup_terminated(&mut self) -> usize {
        let terminated_pids: alloc::vec::Vec<_> = self
            .processes
            .iter()
            .filter(|(_, p)| p.state == ProcessState::Terminated)
            .map(|(pid, _)| *pid)
            .collect();

        let count = terminated_pids.len();
        for pid in terminated_pids {
            self.processes.remove(&pid);
        }
        count
    }
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        memory::VirtAddr,
        process::CapabilitySet,
    };

    fn create_test_process() -> Process {
        Process::new(
            VirtAddr::new(0x1000),
            VirtAddr::new(0x8000),
            CapabilitySet::new(),
        )
    }

    #[test]
    fn test_new_process_table() {
        let table = ProcessTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_add_process() {
        let mut table = ProcessTable::new();
        let process = create_test_process();

        let pid = table.add_process(process).unwrap();
        assert_eq!(pid.as_u64(), 1);
        assert!(!table.is_empty());
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_get_process() {
        let mut table = ProcessTable::new();
        let process = create_test_process();

        let pid = table.add_process(process).unwrap();

        assert!(table.get_process(pid).is_some());
        assert_eq!(table.get_process(pid).unwrap().pid, pid);

        let non_existent = ProcessId::new(999);
        assert!(table.get_process(non_existent).is_none());
    }

    #[test]
    fn test_remove_process() {
        let mut table = ProcessTable::new();
        let process = create_test_process();

        let pid = table.add_process(process).unwrap();
        assert!(table.contains(pid));

        let removed = table.remove_process(pid).unwrap();
        assert_eq!(removed.pid, pid);
        assert!(!table.contains(pid));

        // Removing again should fail
        assert_eq!(
            table.remove_process(pid),
            Err(ProcessError::ProcessNotFound)
        );
    }

    #[test]
    fn test_multiple_processes() {
        let mut table = ProcessTable::new();

        let pid1 = table.add_process(create_test_process()).unwrap();
        let pid2 = table.add_process(create_test_process()).unwrap();
        let pid3 = table.add_process(create_test_process()).unwrap();

        assert_eq!(pid1.as_u64(), 1);
        assert_eq!(pid2.as_u64(), 2);
        assert_eq!(pid3.as_u64(), 3);
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn test_pid_reuse() {
        let mut table = ProcessTable::new();

        let pid1 = table.add_process(create_test_process()).unwrap();
        let pid2 = table.add_process(create_test_process()).unwrap();

        table.remove_process(pid1).unwrap();

        // Next PID should continue incrementing, not reuse immediately
        let pid3 = table.add_process(create_test_process()).unwrap();
        assert_eq!(pid3.as_u64(), 3);
    }

    #[test]
    fn test_terminate_process() {
        let mut table = ProcessTable::new();
        let pid = table.add_process(create_test_process()).unwrap();

        table.terminate_process(pid).unwrap();

        let process = table.get_process(pid).unwrap();
        assert_eq!(process.state, ProcessState::Terminated);
    }

    #[test]
    fn test_terminate_already_terminated() {
        let mut table = ProcessTable::new();
        let pid = table.add_process(create_test_process()).unwrap();

        table.terminate_process(pid).unwrap();
        assert_eq!(
            table.terminate_process(pid),
            Err(ProcessError::InvalidState)
        );
    }

    #[test]
    fn test_cleanup_terminated() {
        let mut table = ProcessTable::new();

        let pid1 = table.add_process(create_test_process()).unwrap();
        let pid2 = table.add_process(create_test_process()).unwrap();
        let pid3 = table.add_process(create_test_process()).unwrap();

        table.terminate_process(pid1).unwrap();
        table.terminate_process(pid3).unwrap();

        let removed = table.cleanup_terminated();
        assert_eq!(removed, 2);
        assert_eq!(table.len(), 1);
        assert!(table.contains(pid2));
    }

    #[test]
    fn test_ready_processes() {
        let mut table = ProcessTable::new();

        let pid1 = table.add_process(create_test_process()).unwrap();
        let pid2 = table.add_process(create_test_process()).unwrap();
        let pid3 = table.add_process(create_test_process()).unwrap();

        table.get_process_mut(pid2).unwrap().mark_running();

        let ready: alloc::vec::Vec<_> = table.ready_processes().collect();
        assert_eq!(ready.len(), 2);
        assert!(ready.iter().any(|p| p.pid == pid1));
        assert!(ready.iter().any(|p| p.pid == pid3));
    }

    #[test]
    fn test_iter() {
        let mut table = ProcessTable::new();

        table.add_process(create_test_process()).unwrap();
        table.add_process(create_test_process()).unwrap();

        let count = table.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_success_criteria() {
        // This test matches the success criteria from the issue
        let mut process_table = ProcessTable::new();

        let entry_point = VirtAddr::new(0x1000);
        let stack_addr = VirtAddr::new(0x8000);
        let initial_caps = CapabilitySet::new();

        let process = Process::new(entry_point, stack_addr, initial_caps);

        let pid = process_table.add_process(process).unwrap();
        assert!(process_table.get_process(pid).is_some());
    }
}
