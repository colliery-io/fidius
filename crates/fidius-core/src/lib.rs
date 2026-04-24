// Copyright 2026 Colliery, Inc.
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

pub mod descriptor;
pub mod error;
pub mod hash;
pub mod package;
pub mod python_descriptor;
pub mod registry;
pub mod status;
pub mod wire;

#[cfg(feature = "async")]
pub mod async_runtime;

pub use descriptor::*;
pub use error::PluginError;
pub use status::*;

// Re-export inventory so generated code can reference it via fidius_core::inventory
pub use inventory;
