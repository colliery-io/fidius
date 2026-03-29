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

use fidius::{plugin_impl, plugin_interface};
use serde::{Deserialize, Serialize};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Calculator: Send + Sync {
    fn add(&self, input: AddInput) -> AddOutput;

    #[optional(since = 2)]
    fn multiply(&self, input: MulInput) -> MulOutput;
}

#[derive(Serialize, Deserialize)]
pub struct AddInput {
    pub a: i64,
    pub b: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AddOutput {
    pub result: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MulInput {
    pub a: i64,
    pub b: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MulOutput {
    pub result: i64,
}

pub struct BasicCalculator;

#[plugin_impl(Calculator)]
impl Calculator for BasicCalculator {
    fn add(&self, input: AddInput) -> AddOutput {
        AddOutput {
            result: input.a + input.b,
        }
    }

    fn multiply(&self, input: MulInput) -> MulOutput {
        MulOutput {
            result: input.a * input.b,
        }
    }
}

fidius::fidius_plugin_registry!();
