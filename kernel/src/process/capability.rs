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

//! Process capability set for capability-based security.
//!
//! This module provides a placeholder implementation for the capability system.
//! The full capability model will be implemented in a future issue.

use alloc::vec::Vec;

/// A capability token that grants specific permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    /// The type of capability.
    pub cap_type: CapabilityType,
    /// The object ID this capability refers to.
    pub object_id: u64,
    /// Permission flags for this capability.
    pub permissions: u64,
}

/// Types of capabilities in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    /// Memory region capability.
    Memory,
    /// IPC endpoint capability.
    Endpoint,
    /// Process capability.
    Process,
    /// Device capability.
    Device,
}

/// A set of capabilities held by a process.
#[derive(Debug, Clone)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }

    /// Create a capability set with initial capabilities.
    pub fn with_capabilities(caps: Vec<Capability>) -> Self {
        Self { capabilities: caps }
    }

    /// Add a capability to the set.
    pub fn add(&mut self, cap: Capability) {
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
    }

    /// Remove a capability from the set.
    pub fn remove(&mut self, cap: &Capability) -> bool {
        if let Some(pos) = self.capabilities.iter().position(|c| c == cap) {
            self.capabilities.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if the set contains a specific capability.
    pub fn contains(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Check if the set has any capability of the given type.
    pub fn has_capability_type(&self, cap_type: CapabilityType) -> bool {
        self.capabilities.iter().any(|c| c.cap_type == cap_type)
    }

    /// Get the number of capabilities in the set.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Get an iterator over the capabilities.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    /// Clear all capabilities from the set.
    pub fn clear(&mut self) {
        self.capabilities.clear();
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_capability_set() {
        let caps = CapabilitySet::new();
        assert!(caps.is_empty());
        assert_eq!(caps.len(), 0);
    }

    #[test]
    fn test_add_capability() {
        let mut caps = CapabilitySet::new();
        let cap = Capability {
            cap_type: CapabilityType::Memory,
            object_id: 1,
            permissions: 0x3,
        };

        caps.add(cap);
        assert!(!caps.is_empty());
        assert_eq!(caps.len(), 1);
        assert!(caps.contains(&cap));
    }

    #[test]
    fn test_add_duplicate_capability() {
        let mut caps = CapabilitySet::new();
        let cap = Capability {
            cap_type: CapabilityType::Memory,
            object_id: 1,
            permissions: 0x3,
        };

        caps.add(cap);
        caps.add(cap);
        assert_eq!(caps.len(), 1);
    }

    #[test]
    fn test_remove_capability() {
        let mut caps = CapabilitySet::new();
        let cap = Capability {
            cap_type: CapabilityType::Endpoint,
            object_id: 42,
            permissions: 0x1,
        };

        caps.add(cap);
        assert!(caps.remove(&cap));
        assert!(caps.is_empty());
        assert!(!caps.remove(&cap));
    }

    #[test]
    fn test_has_capability_type() {
        let mut caps = CapabilitySet::new();
        assert!(!caps.has_capability_type(CapabilityType::Device));

        caps.add(Capability {
            cap_type: CapabilityType::Device,
            object_id: 0,
            permissions: 0,
        });
        assert!(caps.has_capability_type(CapabilityType::Device));
        assert!(!caps.has_capability_type(CapabilityType::Process));
    }
}
