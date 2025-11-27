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

//! Process management module for Yomi OS.
//!
//! This module provides the foundational process structures and process table
//! for managing processes in the kernel.

mod capability;
mod context;
mod id;
mod ipc;
mod process;
mod state;
mod table;

pub use capability::CapabilitySet;
pub use context::ProcessContext;
pub use id::ProcessId;
pub use ipc::Message;
pub use process::Process;
pub use state::ProcessState;
pub use table::{
    ProcessError,
    ProcessTable,
};
