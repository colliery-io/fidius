// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0026 Phase 2: a Rust WASM guest that serves a *server-streaming*
// interface. `tick(count)` returns an exported `tick-stream` resource the host
// polls via `next()` — `some(v)` per item, `none` at clean end. Dropping the
// resource runs this struct's `Drop` (the cancel path / D3).
#[allow(warnings)]
mod bindings;

use std::cell::Cell;

use bindings::exports::fidius::ticker::ticker::{Guest, GuestTickStream, PluginError, TickStream};

// fnv1a("tick:u32->u64!stream") — must match the macro's interface hash for
// `fn tick(&self, count: u32) -> fidius::Stream<u64>` (the `!stream` marker is
// what distinguishes a streaming method). The host checks this at load.
const INTERFACE_HASH: u64 = 0xFD15_2C8A_A111_2FC3;

struct Component;

/// Guest-side stream state. `next` takes `&self` (WIT resource methods are
/// `&self`), so the cursor lives behind a `Cell`.
struct Ticks {
    current: Cell<u64>,
    count: u64,
}

impl GuestTickStream for Ticks {
    fn next(&self) -> Result<Option<u64>, PluginError> {
        let c = self.current.get();
        if c < self.count {
            self.current.set(c + 1);
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }
}

impl Guest for Component {
    type TickStream = Ticks;

    fn tick(count: u32) -> TickStream {
        TickStream::new(Ticks {
            current: Cell::new(0),
            count: count as u64,
        })
    }

    fn fidius_interface_hash() -> u64 {
        INTERFACE_HASH
    }
}

bindings::export!(Component with_types_in bindings);
