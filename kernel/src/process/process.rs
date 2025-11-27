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

//! Process structure and management.

use alloc::collections::VecDeque;

use super::{
    capability::CapabilitySet,
    context::ProcessContext,
    id::ProcessId,
    ipc::Message,
    state::ProcessState,
};
use crate::memory::{
    PhysAddr,
    VirtAddr,
};

/// A process in the system.
#[derive(Debug, Clone)]
pub struct Process {
    /// Unique process identifier.
    pub pid: ProcessId,
    /// Current process state.
    pub state: ProcessState,
    /// Physical address of the process's page table (CR3 value).
    pub page_table: PhysAddr,
    /// CPU context for context switching.
    pub context: ProcessContext,
    /// Top of the process's stack.
    pub stack_top: VirtAddr,
    /// Entry point address for the process.
    pub entry_point: VirtAddr,
    /// Capabilities held by this process.
    pub capabilities: CapabilitySet,
    /// Queue of pending IPC messages.
    pub ipc_queue: VecDeque<Message>,
}

impl Process {
    /// Create a new process.
    ///
    /// # Arguments
    /// * `entry_point` - Virtual address where execution should start
    /// * `stack_addr` - Virtual address of the stack top
    /// * `capabilities` - Initial capability set for the process
    ///
    /// # Note
    /// The `pid` is initially set to 0 and should be assigned by the
    /// `ProcessTable` when the process is added.
    pub fn new(entry_point: VirtAddr, stack_addr: VirtAddr, capabilities: CapabilitySet) -> Self {
        Self {
            pid: ProcessId::new(0),
            state: ProcessState::Ready,
            page_table: PhysAddr::new(0),
            context: ProcessContext::for_entry(entry_point.as_u64(), stack_addr.as_u64()),
            stack_top: stack_addr,
            entry_point,
            capabilities,
            ipc_queue: VecDeque::new(),
        }
    }

    /// Create a new process with a specific page table.
    pub fn with_page_table(
        entry_point: VirtAddr,
        stack_addr: VirtAddr,
        page_table: PhysAddr,
        capabilities: CapabilitySet,
    ) -> Self {
        let mut process = Self::new(entry_point, stack_addr, capabilities);
        process.page_table = page_table;
        process
    }

    /// Set the process ID.
    ///
    /// This should only be called by the ProcessTable when adding a process.
    #[inline(always)]
    pub(crate) fn set_pid(&mut self, pid: ProcessId) {
        self.pid = pid;
    }

    /// Set the process state.
    #[inline(always)]
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    /// Mark the process as running.
    #[inline(always)]
    pub fn mark_running(&mut self) {
        self.state = ProcessState::Running;
    }

    /// Mark the process as ready.
    #[inline(always)]
    pub fn mark_ready(&mut self) {
        self.state = ProcessState::Ready;
    }

    /// Mark the process as blocked.
    #[inline(always)]
    pub fn mark_blocked(&mut self) {
        self.state = ProcessState::Blocked;
    }

    /// Mark the process as waiting for a message.
    #[inline(always)]
    pub fn mark_waiting_for_message(&mut self) {
        self.state = ProcessState::WaitingForMessage;
    }

    /// Mark the process as terminated.
    #[inline(always)]
    pub fn mark_terminated(&mut self) {
        self.state = ProcessState::Terminated;
    }

    /// Queue an IPC message for this process.
    pub fn queue_message(&mut self, message: Message) {
        self.ipc_queue.push_back(message);

        // Wake up the process if it was waiting for a message
        if self.state == ProcessState::WaitingForMessage {
            self.state = ProcessState::Ready;
        }
    }

    /// Retrieve the next pending IPC message.
    pub fn receive_message(&mut self) -> Option<Message> {
        self.ipc_queue.pop_front()
    }

    /// Check if there are pending messages.
    #[inline(always)]
    pub fn has_pending_messages(&self) -> bool {
        !self.ipc_queue.is_empty()
    }

    /// Get the number of pending messages.
    #[inline(always)]
    pub fn pending_message_count(&self) -> usize {
        self.ipc_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_process() -> Process {
        Process::new(
            VirtAddr::new(0x1000),
            VirtAddr::new(0x8000),
            CapabilitySet::new(),
        )
    }

    #[test]
    fn test_process_creation() {
        let entry = VirtAddr::new(0x1000);
        let stack = VirtAddr::new(0x8000);
        let caps = CapabilitySet::new();

        let process = Process::new(entry, stack, caps);

        assert_eq!(process.pid.as_u64(), 0);
        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.entry_point, entry);
        assert_eq!(process.stack_top, stack);
        assert_eq!(process.context.rip, entry.as_u64());
        assert_eq!(process.context.rsp, stack.as_u64());
    }

    #[test]
    fn test_process_with_page_table() {
        let entry = VirtAddr::new(0x1000);
        let stack = VirtAddr::new(0x8000);
        let page_table = PhysAddr::new(0x10000);
        let caps = CapabilitySet::new();

        let process = Process::with_page_table(entry, stack, page_table, caps);

        assert_eq!(process.page_table, page_table);
    }

    #[test]
    fn test_state_transitions() {
        let mut process = create_test_process();

        assert_eq!(process.state, ProcessState::Ready);

        process.mark_running();
        assert_eq!(process.state, ProcessState::Running);

        process.mark_blocked();
        assert_eq!(process.state, ProcessState::Blocked);

        process.mark_ready();
        assert_eq!(process.state, ProcessState::Ready);

        process.mark_waiting_for_message();
        assert_eq!(process.state, ProcessState::WaitingForMessage);

        process.mark_terminated();
        assert_eq!(process.state, ProcessState::Terminated);
    }

    #[test]
    fn test_message_queue() {
        let mut process = create_test_process();
        let sender = ProcessId::new(2);

        assert!(!process.has_pending_messages());
        assert_eq!(process.pending_message_count(), 0);

        let msg1 = Message::new(sender, 1);
        let msg2 = Message::new(sender, 2);

        process.queue_message(msg1);
        process.queue_message(msg2);

        assert!(process.has_pending_messages());
        assert_eq!(process.pending_message_count(), 2);

        let received = process.receive_message().unwrap();
        assert_eq!(received.tag, 1);

        assert_eq!(process.pending_message_count(), 1);
    }

    #[test]
    fn test_message_wakes_waiting_process() {
        let mut process = create_test_process();
        process.mark_waiting_for_message();

        assert_eq!(process.state, ProcessState::WaitingForMessage);

        let msg = Message::new(ProcessId::new(2), 42);
        process.queue_message(msg);

        assert_eq!(process.state, ProcessState::Ready);
    }
}
