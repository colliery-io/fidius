// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-T-0177): every author-facing identifier that lands
// in the generated WIT is a reserved WIT keyword — record field names (`record`,
// `stream`, `from`, `type`), variant + case names (`Stream`, `Record`, `List`),
// and the method/param names. `build.rs` (fidius_build::emit_wit) must escape
// them (`%record`, …) or the generated wit/ fails to parse and the component
// never builds. The successful wasm32-wasip2 build IS the test.

use fidius_macro::{plugin_impl, plugin_interface, WitType};

// Keyword-named record fields. `record`/`stream`/`from` are valid Rust idents;
// `type` is a Rust keyword so it must be written as the raw ident `r#type` — the
// generator strips the `r#` before kebab-casing, then escapes to `%type`.
#[derive(WitType, Clone)]
pub struct DeadLetter {
    pub record: String,
    pub stream: u64,
    pub from: bool,
    pub r#type: u8,
}

// Keyword variant name (`Outcome` is fine, but the cases collide): `Stream` and
// `Record` are keywords, and the struct-variant `List` synthesizes a record whose
// `from` field is a keyword too.
#[derive(WitType, Clone)]
pub enum Outcome {
    Stream,
    Record(u32),
    List { from: u8 },
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Sink: Send + Sync {
    // Keyword method name (`record`), keyword param names (`list`, `option`), and
    // keyword-named user type references (`dead-letter`, `outcome`).
    fn record(&self, list: DeadLetter, option: Outcome) -> DeadLetter;
}

pub struct MySink;

#[plugin_impl(Sink, crate = "fidius_guest")]
impl Sink for MySink {
    fn record(&self, list: DeadLetter, option: Outcome) -> DeadLetter {
        let bump = match option {
            Outcome::Stream => 0,
            Outcome::Record(n) => n as u64,
            Outcome::List { from } => from as u64,
        };
        DeadLetter {
            stream: list.stream + bump,
            ..list
        }
    }
}
