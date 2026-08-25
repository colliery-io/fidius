# fidius-host::client_stream <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Host-side **producer** for client-streaming (FIDIUS-I-0030 / ADR-0007).

The inverse of the server-streaming `StreamState`: here the **host** produces
and the **guest** consumes. [`host_producer_handle`] builds a
[`FidiusStreamHandle`] from an iterator of bincode-encoded items; the guest's
`HostStream<T>` pulls them by calling `next`. Reusing the same handle struct
keeps both stream directions on one ABI.

## Structs

### `fidius-host::client_stream::ProducerState`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


Boxed producer state: a lazy item source plus a held-back `pending` item, so a `BUFFER_TOO_SMALL` retry re-delivers the same item instead of dropping it (mirrors `StreamState`). The source is pulled — and, for the typed path, **encoded** — only when the guest asks for the next item, so an unbounded input stays bounded in memory (FIDIUS-T-0172).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `next_encoded` | `NextEncoded` |  |
| `pending` | `Option < Vec < u8 > >` |  |



## Functions

### `fidius-host::client_stream::producer_next`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
unsafe extern "C" fn producer_next (h : * mut FidiusStreamHandle , buf : * mut u8 , cap : u32 , out_len : * mut u32 ,) -> i32
```

The `next` callback the guest invokes: deliver one item into the guest buffer.

<details>
<summary>Source</summary>

```rust
unsafe extern "C" fn producer_next(
    h: *mut FidiusStreamHandle,
    buf: *mut u8,
    cap: u32,
    out_len: *mut u32,
) -> i32 {
    let st = &mut *((*h).state as *mut ProducerState);
    if st.pending.is_none() {
        match (st.next_encoded)() {
            Some(Ok(bytes)) => st.pending = Some(bytes),
            Some(Err(())) => return STATUS_SERIALIZATION_ERROR,
            None => return STATUS_STREAM_END,
        }
    }
    let bytes = st.pending.as_ref().unwrap();
    if bytes.len() > cap as usize {
        *out_len = bytes.len() as u32;
        return STATUS_BUFFER_TOO_SMALL;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
    *out_len = bytes.len() as u32;
    st.pending = None;
    STATUS_OK
}
```

</details>



### `fidius-host::client_stream::producer_drop`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
unsafe extern "C" fn producer_drop (h : * mut FidiusStreamHandle)
```

Finish/cancel: free the producer state + the handle box. Called once by the guest consumer's `Drop`.

<details>
<summary>Source</summary>

```rust
unsafe extern "C" fn producer_drop(h: *mut FidiusStreamHandle) {
    drop(Box::from_raw((*h).state as *mut ProducerState));
    drop(Box::from_raw(h));
}
```

</details>



### `fidius-host::client_stream::build_handle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_handle (next_encoded : NextEncoded) -> * mut FidiusStreamHandle
```

<details>
<summary>Source</summary>

```rust
fn build_handle(next_encoded: NextEncoded) -> *mut FidiusStreamHandle {
    let st = Box::into_raw(Box::new(ProducerState {
        next_encoded,
        pending: None,
    }));
    Box::into_raw(Box::new(FidiusStreamHandle {
        next: producer_next,
        drop_fn: producer_drop,
        state: st as *mut c_void,
    }))
}
```

</details>



### `fidius-host::client_stream::host_producer_handle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn host_producer_handle (items : impl Iterator < Item = Vec < u8 > > + Send + 'static ,) -> * mut FidiusStreamHandle
```

Build a `FidiusStreamHandle` the guest can pull, from an iterator of **pre-encoded** items (raw path). The returned handle is owned by the guest consumer, which frees it via `drop_fn`.

<details>
<summary>Source</summary>

```rust
pub fn host_producer_handle(
    items: impl Iterator<Item = Vec<u8>> + Send + 'static,
) -> *mut FidiusStreamHandle {
    let mut items = items;
    build_handle(Box::new(move || items.next().map(Ok)))
}
```

</details>



### `fidius-host::client_stream::host_producer_handle_typed`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn host_producer_handle_typed < I : serde :: Serialize + 'static > (items : impl Iterator < Item = I > + Send + 'static ,) -> * mut FidiusStreamHandle
```

Like [`host_producer_handle`] but takes a **typed** item iterator and bincode-encodes each item **lazily** — only when the guest pulls it (FIDIUS-T-0172). This is the path the typed `PluginHandle::call_client_streaming` / `call_bidi_streaming` use, so an unbounded input iterator flows with bounded host memory. An item that fails to encode surfaces as a serialization error to the guest call (no panic across the FFI).

<details>
<summary>Source</summary>

```rust
pub fn host_producer_handle_typed<I: serde::Serialize + 'static>(
    items: impl Iterator<Item = I> + Send + 'static,
) -> *mut FidiusStreamHandle {
    let mut items = items;
    build_handle(Box::new(move || {
        items
            .next()
            .map(|i| fidius_core::wire::serialize(&i).map_err(|_| ()))
    }))
}
```

</details>



