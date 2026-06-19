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

extern crate fidius_core as fidius;

use fidius_macro::plugin_interface;

// v1 is server-streaming only — `Stream<T>` is not allowed in argument position
// (client-streaming / bidirectional are deferred). FIDIUS-I-0026.
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait BadStream: Send + Sync {
    fn sink(&self, items: fidius::Stream<u32>) -> u32;
}

fn main() {}
