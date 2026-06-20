// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Regenerate wit/ + conversions from src/lib.rs (PC.2): the streaming method's
// item is a #[derive(WitType)] record, so the WIT carries a `record row` and a
// `rows-stream` resource whose `next()` yields `option<row>`.
fn main() {
    fidius_build::emit_wit();
}
