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

//! Process state definitions.

/// The current state of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is ready to run and waiting for CPU time.
    Ready,
    /// Process is currently executing on a CPU.
    Running,
    /// Process is blocked waiting for a resource or event.
    Blocked,
    /// Process is waiting to receive an IPC message.
    WaitingForMessage,
    /// Process has terminated and is waiting for cleanup.
    Terminated,
}

impl ProcessState {
    /// Check if the process can be scheduled for execution.
    #[inline(always)]
    pub const fn is_schedulable(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Check if the process is currently running.
    #[inline(always)]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the process is in a blocked state (any kind).
    #[inline(always)]
    pub const fn is_blocked(self) -> bool {
        matches!(self, Self::Blocked | Self::WaitingForMessage)
    }

    /// Check if the process has terminated.
    #[inline(always)]
    pub const fn is_terminated(self) -> bool {
        matches!(self, Self::Terminated)
    }
}

impl core::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Running => write!(f, "Running"),
            Self::Blocked => write!(f, "Blocked"),
            Self::WaitingForMessage => write!(f, "WaitingForMessage"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ready_state() {
        let state = ProcessState::Ready;
        assert!(state.is_schedulable());
        assert!(!state.is_running());
        assert!(!state.is_blocked());
        assert!(!state.is_terminated());
    }

    #[test]
    fn test_running_state() {
        let state = ProcessState::Running;
        assert!(!state.is_schedulable());
        assert!(state.is_running());
        assert!(!state.is_blocked());
        assert!(!state.is_terminated());
    }

    #[test]
    fn test_blocked_state() {
        let state = ProcessState::Blocked;
        assert!(!state.is_schedulable());
        assert!(!state.is_running());
        assert!(state.is_blocked());
        assert!(!state.is_terminated());
    }

    #[test]
    fn test_waiting_for_message_state() {
        let state = ProcessState::WaitingForMessage;
        assert!(!state.is_schedulable());
        assert!(!state.is_running());
        assert!(state.is_blocked());
        assert!(!state.is_terminated());
    }

    #[test]
    fn test_terminated_state() {
        let state = ProcessState::Terminated;
        assert!(!state.is_schedulable());
        assert!(!state.is_running());
        assert!(!state.is_blocked());
        assert!(state.is_terminated());
    }
}
