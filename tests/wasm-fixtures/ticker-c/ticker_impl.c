// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Non-Rust polyglot *streaming* guest (FIDIUS-I-0026): a C implementation of the
// SAME `ticker` WIT as the Rust/JS/Python guests. Bindings by wit-bindgen (C);
// compiled to a component with the wasi-sdk clang. It exports a streaming
// `tick-stream` resource the fidius host polls identically — no runtime embedded,
// so the component is tiny.
#include "ticker_plugin.h"
#include <stdlib.h>

// The resource representation (the WIT `resource tick-stream`). wit-bindgen
// forward-declares this type; we define its contents — our per-stream state.
struct exports_fidius_ticker_ticker_tick_stream_t {
    uint64_t i;
    uint64_t count;
};

// tick(count) -> tick-stream : construct the resource, hand back an owned handle.
exports_fidius_ticker_ticker_own_tick_stream_t
exports_fidius_ticker_ticker_tick(uint32_t count) {
    exports_fidius_ticker_ticker_tick_stream_t *rep = malloc(sizeof(*rep));
    rep->i = 0;
    rep->count = (uint64_t)count;
    return exports_fidius_ticker_ticker_tick_stream_new(rep);
}

// [method]tick-stream.next() -> result<option<u64>, plugin-error>
//   true  + ret->is_some = true  → ok(some(v))
//   true  + ret->is_some = false → ok(none) = clean end of stream
//   false                        → err (unused here)
bool exports_fidius_ticker_ticker_method_tick_stream_next(
    exports_fidius_ticker_ticker_borrow_tick_stream_t self,
    ticker_plugin_option_u64_t *ret,
    exports_fidius_ticker_ticker_plugin_error_t *err) {
    (void)err;
    if (self->i < self->count) {
        ret->is_some = true;
        ret->val = self->i;
        self->i += 1;
    } else {
        ret->is_some = false;
    }
    return true;
}

// Resource destructor: drop = guest dtor = cancel.
void exports_fidius_ticker_ticker_tick_stream_destructor(
    exports_fidius_ticker_ticker_tick_stream_t *rep) {
    free(rep);
}

uint64_t exports_fidius_ticker_ticker_fidius_interface_hash(void) {
    return 0xFD152C8AA1112FC3ULL; // fnv1a("tick:u32->u64!stream")
}
