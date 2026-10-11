---
title: "fidius-guest::host_call"
description: "Guest-side consumer for **host functions over WASM** (the plugin → host callback channel, wasm variant)."
sidebar:
  order: 16
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Guest-side consumer for **host functions over WASM** (the plugin → host callback channel, wasm variant).

WASM can't share a raw [`crate::host_ffi::HostFunctionTable`] across the
sandbox, so the guest **imports** a host function and dispatches through
it: `fidius:host-call/host.call(interface-name, expected-version,
expected-hash, index, args) -> (status, payload)`. The identity triple
travels with every call, so the host enforces the version/hash gate on
each call — the wasm counterpart of the dylib channel's bind-time gate,
with the same guarantee (typed loud failure, never a mis-dispatch of
positional bincode). This `generate!` composes with the macro's
interface-export `generate!` exactly like [`crate::client_stream`]'s
`fidius:stream-pull` import.

## Functions

### <span id="fn.call"></span>`fn call`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn call (interface : & 'static str , expected_version : u32 , expected_hash : u64 , index : u32 , args : & [u8] ,) -> Result < Vec < u8 > , HostCallError >
```

Invoke a host function through the `fidius:host-call` import: bincode of the argument tuple in, bincode of the result out, with the status protocol mapped to [`HostCallError`]. Used by the wasm side of every generated `<Trait>Client` method.

<details>
<summary>Source</summary>

```rust
pub fn call(
    interface: &'static str,
    expected_version: u32,
    expected_hash: u64,
    index: u32,
    args: &[u8],
) -> Result<Vec<u8>, HostCallError> {
    let (status, payload) = bindings::fidius::host_call::host::call(
        interface,
        expected_version,
        expected_hash,
        index,
        args,
    );
    decode_host_call_status(interface, index, status, payload)
}
```

</details>



### <span id="fn.probe"></span>`fn probe`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn probe (interface : & 'static str , expected_version : u32 , expected_hash : u64 ,) -> Result < () , HostCallError >
```

Run the name/version/hash gate without dispatching a function (the reserved probe index). `Ok(())` means the host bound a matching table; the error is the same typed error a real call would surface.

<details>
<summary>Source</summary>

```rust
pub fn probe(
    interface: &'static str,
    expected_version: u32,
    expected_hash: u64,
) -> Result<(), HostCallError> {
    call(
        interface,
        expected_version,
        expected_hash,
        HOST_CALL_PROBE_INDEX,
        &[],
    )
    .map(|_| ())
}
```

</details>



