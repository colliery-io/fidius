# fidius-host::arena <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Thread-local arena pool for Arena-strategy plugin invocations.

Plugin methods built with `buffer = Arena` write their serialized output
into a host-provided buffer rather than allocating on each call. This
module supplies those buffers from a per-thread pool so repeated calls
reuse memory instead of reallocating.

## Functions

### `fidius-host::arena::acquire_arena`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn acquire_arena (min_capacity : usize) -> Vec < u8 >
```

Acquire an arena buffer with at least `min_capacity` bytes. Prefers to reuse a pooled buffer if one is available; otherwise allocates a fresh `Vec<u8>`. The returned buffer is filled with zero bytes up to its length (which equals its capacity after this call) — callers write into it via raw pointers and track their own output length.

<details>
<summary>Source</summary>

```rust
pub fn acquire_arena(min_capacity: usize) -> Vec<u8> {
    let target = min_capacity.max(DEFAULT_ARENA_CAPACITY);
    ARENA_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if let Some(mut buf) = pool.pop() {
            if buf.capacity() < target {
                let extra = target - buf.capacity();
                buf.reserve_exact(extra);
            }
            // Fill the available capacity with zeros so the raw mut-slice
            // view aligns with the Vec's len invariant.
            let cap = buf.capacity();
            buf.clear();
            buf.resize(cap, 0);
            buf
        } else {
            vec![0u8; target]
        }
    })
}
```

</details>



### `fidius-host::arena::release_arena`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn release_arena (buf : Vec < u8 >)
```

Return an arena buffer to the pool for future reuse. The buffer's capacity is retained; length is irrelevant (callers set length via direct writes, not Vec::extend).

<details>
<summary>Source</summary>

```rust
pub fn release_arena(buf: Vec<u8>) {
    ARENA_POOL.with(|pool| pool.borrow_mut().push(buf));
}
```

</details>



### `fidius-host::arena::grow_arena`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn grow_arena (buf : & mut Vec < u8 > , needed_capacity : usize)
```

Grow an in-flight arena buffer to hold at least `needed_capacity` bytes. Used by the retry path when a plugin returns `STATUS_BUFFER_TOO_SMALL`.

<details>
<summary>Source</summary>

```rust
pub fn grow_arena(buf: &mut Vec<u8>, needed_capacity: usize) {
    if buf.capacity() < needed_capacity {
        let extra = needed_capacity - buf.capacity();
        buf.reserve_exact(extra);
    }
    let cap = buf.capacity();
    buf.clear();
    buf.resize(cap, 0);
}
```

</details>



