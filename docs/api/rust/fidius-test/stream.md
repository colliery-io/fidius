# fidius-test::stream <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Composition harness for streaming plugins — **for tests, not production**.

`stream_of` / `collect` / `pump` make it trivial to test a streaming plugin
in isolation or to wire a producer to a consumer. They are the reference `|`
for "pipes of plugins."

## Structs

### `fidius-test::stream::CollectSink`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Default`

A [`StreamSink`] that records everything it accepts — for asserting on the far end of a `pump`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `items` | `Mutex < Vec < Value > >` |  |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new () -> Self
```

A fresh, empty sink.

<details>
<summary>Source</summary>

```rust
    pub fn new() -> Self {
        Self::default()
    }
```

</details>



##### `take` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn take (& self) -> Vec < Value >
```

Snapshot of everything accepted so far.

<details>
<summary>Source</summary>

```rust
    pub fn take(&self) -> Vec<Value> {
        self.items.lock().unwrap().clone()
    }
```

</details>





## Functions

### `fidius-test::stream::stream_of`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn stream_of (items : Vec < Value >) -> ChunkStream
```

An in-memory source over a fixed item sequence. **Test-only by nature** — you would never construct a stream from a `Vec` in production; that it is useful only in tests is exactly why the whole module is test-tier.

Yields each value as `Ok`, then a clean end of stream.

<details>
<summary>Source</summary>

```rust
pub fn stream_of(items: Vec<Value>) -> ChunkStream {
    ChunkStream::new(futures::stream::iter(
        items.into_iter().map(Ok::<Value, CallError>),
    ))
}
```

</details>



### `fidius-test::stream::collect`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
async fn collect (mut s : ChunkStream) -> Result < Vec < Value > , CallError >
```

Drain a stream to a `Vec`, stopping at — and returning — the first error. The single-plugin unit-test idiom: `collect(plugin.process(stream_of(rows)))`.

<details>
<summary>Source</summary>

```rust
pub async fn collect(mut s: ChunkStream) -> Result<Vec<Value>, CallError> {
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        out.push(item?);
    }
    Ok(out)
}
```

</details>



### `fidius-test::stream::pump`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
async fn pump < S > (mut out : ChunkStream , into : & S) -> Result < () , CallError > where S : StreamSink + ? Sized ,
```

The reference pull-loop wiring a producer stream to a [`StreamSink`].

Pull-paced: exactly one item is awaited at a time, so a slow sink naturally
backpressures the producer (the next item is not pulled until the sink has
accepted the current one). Stops at the first error from either side —
producer error or sink rejection — and returns it; on a clean end of stream
returns `Ok(())`. This is the ~10 lines you would copy into production and
then grow your own retries/observability around.

<details>
<summary>Source</summary>

```rust
pub async fn pump<S>(mut out: ChunkStream, into: &S) -> Result<(), CallError>
where
    S: StreamSink + ?Sized,
{
    while let Some(item) = out.next().await {
        into.accept(item?).await?;
    }
    Ok(())
}
```

</details>



