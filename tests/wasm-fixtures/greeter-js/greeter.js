// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Non-Rust polyglot guest (FIDIUS-I-0025): a JavaScript implementation of the
// SAME `greeter` WIT as the Rust + Python guests, built into a WASM component
// with jco/ComponentizeJS. The fidius host loads and calls it identically.
//
// WIT type mapping (jco): s64/u64 -> BigInt, list<u8> -> Uint8Array,
// result<T, plugin-error> -> return T (throw for the error arm).
export const greeter = {
  greet(name) {
    return `Hello, ${name}!`;
  },
  add(a, b) {
    return a + b; // BigInt + BigInt -> BigInt (the result<s64> Ok arm)
  },
  echoBytes(data) {
    return new Uint8Array(Array.from(data).reverse());
  },
  probeEnv() {
    return false; // env access is a host capability; not needed for the demo
  },
  fidiusInterfaceHash() {
    return 0x0102030405060708n; // must equal the Rust/Python guests' hash
  },
};
