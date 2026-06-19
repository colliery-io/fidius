// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Non-Rust polyglot streaming guest (FIDIUS-I-0026): a JavaScript implementation
// of the SAME `ticker` WIT as the Rust guests, built into a WASM component with
// jco/ComponentizeJS. It exports a streaming `tick-stream` resource. The fidius
// host loads and streams from it identically — proving the streaming contract is
// language-neutral.
//
// WIT mapping (jco): u32 -> number, u64 -> BigInt, an exported resource ->
// a JS class, result<option<u64>, plugin-error> -> return the option (a BigInt
// for some, undefined for none) and throw for the error arm.

class TickStream {
  constructor(count) {
    this.i = 0n;
    this.count = BigInt(count);
  }
  next() {
    if (this.i < this.count) {
      const v = this.i;
      this.i += 1n;
      return v; // ok(some(v))
    }
    return undefined; // ok(none) -> clean end of stream
  }
}

export const ticker = {
  TickStream,
  tick(count) {
    return new TickStream(count);
  },
  fidiusInterfaceHash() {
    return 0xfd152c8aa1112fc3n; // fnv1a("tick:u32->u64!stream"); must match the host
  },
};
