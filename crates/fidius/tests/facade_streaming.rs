// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 5: facade tests for the `streaming` feature surface. The
// streaming runtime behaviour (pull/backpressure/drop-cancel) is covered by
// `fidius-host`'s own e2e suite; here we guard that the streaming types are
// reachable through the facade, and — by building under `--features streaming` —
// that the host+streaming feature combination compiles and its other facade tests
// (core, host) run.
#![cfg(feature = "streaming")]

#[test]
fn streaming_types_are_reexported() {
    fn assert_exists<T>() {}
    assert_exists::<fidius::ChunkStream>();

    // StreamExecutor is a trait — guard it via a bound.
    fn _accepts_executor<E: fidius::StreamExecutor>() {}

    // The `Stream<T>` author-facing marker is always re-exported.
    assert_exists::<fidius::Stream<u64>>();
}
