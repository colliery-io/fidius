# fidius-host::executor::name_lookup <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Resolve-and-pin for `wasi:sockets` name lookups (FIDIUS-I-0034).

fidius shadows `wasi:sockets/ip-name-lookup` in the linker (after
`add_to_linker_sync`, with `allow_shadowing(true)`) with this module's
implementation, which:
1. consults [`EgressPolicy::authorize_dns`] **before** resolving — a denial
fails the guest's lookup with `permanent-resolver-failure` (the same
error upstream returns for a lookup denied outright), resolves nothing,
and pins nothing;
2. resolves host-side (std `ToSocketAddrs`, matching upstream, unless a
test injects a resolver); and
3. records `name ↔ IPs` in a per-store [`PinTable`] **inside** the blocking
resolution task — the pin is written before the future completes, so no
address ever reaches the guest un-pinned.
`socket_addr_check` then recovers the dialed name for a connect's IP from
the pin table and hands the policy a `TcpTarget { host: Some(name), .. }`.
The shadow is installed only under the same two-key condition that enables
name lookup at all (a `tcp`/`udp` grant AND an embedder policy); otherwise
upstream's implementation stands untouched.

## Structs

### `fidius-host::executor::name_lookup::PinState`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


**Derives:** `Default`

What this store's lookups pinned: both directions of `name ↔ IPs`.

Semantics (FIDIUS-I-0034): names are stored ASCII-lowercase, IPs canonical
(`to_canonical()`), so neither case nor v4-mapped-v6 spelling can dodge a
pin. Re-resolving a name replaces its entry wholesale — IPs the old
resolution mapped that the new one no longer does are unpinned (a stale pin
must not authorize). When two pinned names share an IP, the most recent
resolution wins the `IP → name` side.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `by_name` | `HashMap < String , Vec < IpAddr > >` |  |
| `by_ip` | `HashMap < IpAddr , String >` |  |

#### Methods

##### `record` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn record (& mut self , name : & str , ips : & [IpAddr])
```

Record one completed resolution of `name` (already lowercased) to `ips` (already canonicalized), applying replace-on-re-resolve.

<details>
<summary>Source</summary>

```rust
    fn record(&mut self, name: &str, ips: &[IpAddr]) {
        if let Some(old) = self.by_name.remove(name) {
            for ip in old {
                // Unpin only if the reverse entry still points at this name —
                // a newer resolution of another name may have claimed the IP.
                if self.by_ip.get(&ip).is_some_and(|n| n == name) {
                    self.by_ip.remove(&ip);
                }
            }
        }
        self.by_name.insert(name.to_string(), ips.to_vec());
        for ip in ips {
            self.by_ip.insert(*ip, name.to_string());
        }
    }
```

</details>



##### `host_for` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn host_for (& self , ip : & IpAddr) -> Option < String >
```

The name this store's lookups most recently resolved to `ip` (canonicalize before calling), if any.

<details>
<summary>Source</summary>

```rust
    pub(crate) fn host_for(&self, ip: &IpAddr) -> Option<String> {
        self.by_ip.get(ip).cloned()
    }
```

</details>





### `fidius-host::executor::name_lookup::FidiusNameLookup`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


`HasData` marker for the shadowed instance (the `D` in `add_to_linker`).



### `fidius-host::executor::name_lookup::NameLookupView`<'a>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


Per-call view the bindgen host traits run against: the store's own `ResourceTable` (shared with the rest of WASI so `network` handles and the `resolve-address-stream` resource interoperate), plus the pin table, the embedder policy, and the resolver.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `table` | `& 'a mut ResourceTable` |  |
| `pins` | `& 'a PinTable` |  |
| `policy` | `Option < & 'a Arc < dyn EgressPolicy > >` |  |
| `resolver` | `& 'a Resolver` |  |



## Functions

### `fidius-host::executor::name_lookup::default_resolver`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn default_resolver () -> Resolver
```

<details>
<summary>Source</summary>

```rust
pub(crate) fn default_resolver() -> Resolver {
    Arc::new(|name| Ok((name, 0).to_socket_addrs()?.map(|sa| sa.ip()).collect()))
}
```

</details>



