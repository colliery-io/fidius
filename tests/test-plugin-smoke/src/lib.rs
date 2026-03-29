use fides_macro::{plugin_impl, plugin_interface};
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

fides_core::fides_plugin_registry!();
