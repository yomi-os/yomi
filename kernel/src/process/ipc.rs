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

//! Inter-process communication (IPC) message types.
//!
//! This module provides a placeholder implementation for the IPC message
//! system. The full IPC system will be implemented in a future issue.

use super::ProcessId;

/// Maximum size of inline message data in bytes.
pub const MAX_INLINE_DATA: usize = 56;

/// An IPC message that can be sent between processes.
#[derive(Debug, Clone)]
pub struct Message {
    /// The sender's process ID.
    pub sender: ProcessId,
    /// The message tag identifying the message type.
    pub tag: u64,
    /// Inline message data.
    pub data: [u8; MAX_INLINE_DATA],
    /// Length of valid data in the data array.
    pub data_len: usize,
}

impl Message {
    /// Create a new empty message.
    pub const fn new(sender: ProcessId, tag: u64) -> Self {
        Self {
            sender,
            tag,
            data: [0; MAX_INLINE_DATA],
            data_len: 0,
        }
    }

    /// Create a message with data.
    pub fn with_data(sender: ProcessId, tag: u64, data: &[u8]) -> Option<Self> {
        if data.len() > MAX_INLINE_DATA {
            return None;
        }

        let mut msg = Self::new(sender, tag);
        msg.data[..data.len()].copy_from_slice(data);
        msg.data_len = data.len();
        Some(msg)
    }

    /// Get the message data as a slice.
    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }

    /// Check if the message has any data.
    pub fn has_data(&self) -> bool {
        self.data_len > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message() {
        let sender = ProcessId::new(1);
        let msg = Message::new(sender, 42);

        assert_eq!(msg.sender, sender);
        assert_eq!(msg.tag, 42);
        assert_eq!(msg.data_len, 0);
        assert!(!msg.has_data());
    }

    #[test]
    fn test_message_with_data() {
        let sender = ProcessId::new(1);
        let data = b"Hello";
        let msg = Message::with_data(sender, 1, data).unwrap();

        assert_eq!(msg.sender, sender);
        assert_eq!(msg.tag, 1);
        assert_eq!(msg.data(), data.as_slice());
        assert!(msg.has_data());
    }

    #[test]
    fn test_message_data_too_large() {
        let sender = ProcessId::new(1);
        let data = [0u8; MAX_INLINE_DATA + 1];
        let msg = Message::with_data(sender, 1, &data);

        assert!(msg.is_none());
    }

    #[test]
    fn test_max_inline_data() {
        let sender = ProcessId::new(1);
        let data = [0xaa; MAX_INLINE_DATA];
        let msg = Message::with_data(sender, 1, &data).unwrap();

        assert_eq!(msg.data_len, MAX_INLINE_DATA);
        assert_eq!(msg.data(), &data);
    }
}
