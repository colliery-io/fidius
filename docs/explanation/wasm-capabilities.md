<!--
Copyright 2026 Colliery, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# Capabilities & the WASM Sandbox

Native (cdylib) plugins run with the host's full authority — they can open files,
make network calls, and read the environment, because they are just shared
libraries. WASM component plugins are different: they run inside a wasmtime
sandbox with **no ambient authority**. A plugin can do nothing to the outside
world unless the package's `[wasm].capabilities` allow-list explicitly grants it.
This is the security reason to ship a plugin as a component.

## The model: WASI present, zero grants

A subtle but important design point (FIDIUS-T-0102): real components built by
standard toolchains *import* WASI interfaces (`wasi:cli`, `wasi:io`,
`wasi:clocks`, `wasi:filesystem`, …) even when the plugin never calls them — the
language runtime references them. An **empty** wasmtime `Linker` therefore can't
even instantiate such a component.

So fidius does **not** use an empty linker. Instead it:

1. Wires `wasmtime-wasi` into the `Linker` so any conforming component
   instantiates, **and**
2. Gives the guest a **deny-all `WasiCtx`** — no filesystem preopens, no
   environment, no inherited stdio, no sockets.

The WASI *interfaces* are present (so the component links), but they are backed
by a context that grants **nothing**. Capabilities in the manifest selectively
open specific facets of that context.

## Declaring capabilities

Capabilities are a string allow-list under `[wasm]` in `package.toml`:

```toml
[wasm]
component = "plugin.wasm"
capabilities = ["clocks", "random", "stdout"]
```

An **empty or absent** list (`capabilities = []`) is the default: a fully
deny-all sandbox. `fidius package inspect` surfaces this list prominently — it is
the single most security-relevant field a deployer reviews before trusting a
plugin.

### Recognized capabilities

| Capability          | Grants                                                        |
| ------------------- | ------------------------------------------------------------- |
| `env:VAR_NAME`      | Read **one** named host env var (e.g. `env:LOG_LEVEL`). Bare `env` is **rejected** — see below |
| `fs:ro:<path>` / `fs:rw:<path>` | Read-only / read-write access to **one** directory (e.g. `fs:ro:/etc/certs`). Bare `fs` is **rejected** — see below |
| `args`              | Read process arguments                                        |
| `stdout` / `stderr` | Write to the host's standard out / error                      |
| `stdin`             | Read the host's standard input                                |
| `network` / `sockets` | Raw outbound sockets + DNS (coarse; SSRF floor applied — see below) |
| `http`              | Brokered outbound HTTP via `wasi:http` — **only** with a host `EgressPolicy` (see below) |
| `clocks`            | Wall/monotonic clocks (always available; accepted as a no-op) |
| `random`            | Secure randomness (always available; accepted as a no-op)     |

### `env` is per-variable, never inherit-all

Bare `env` (which would grant the guest **every** host environment variable — i.e.
all your secrets: `AWS_SECRET_ACCESS_KEY`, `DATABASE_URL`, tokens) is **rejected at
load** with a clear error. Grant exactly the variables a connector needs, by name:

```toml
capabilities = ["env:STRIPE_API_BASE", "env:LOG_LEVEL"]
```

Each `env:NAME` exposes that one variable's value (skipped if it isn't set on the
host). This keeps host secrets out of an untrusted guest's reach and is what makes
host-side credential injection (below) meaningful.

Unknown names **fail closed**: a manifest listing a capability fidius doesn't
recognize (a typo, or an unsupported one) is rejected at load with a clear error,
rather than silently granting nothing. This is verified by the
`unknown_capability_rejected_at_load` test.

### Filesystem is per-directory, never whole-FS

Filesystem access is **path-scoped** (FIDIUS-A-0008): you grant exactly one
directory, read-only or read-write:

```toml
capabilities = ["fs:ro:/etc/myapp/certs", "fs:rw:/var/lib/myapp/cache"]
```

The host **preopens** that directory; WASI's capability model then scopes the guest
to it — there is no path-traversal escape and no ambient filesystem. `fs:ro:` grants
read-only (the guest physically cannot write); `fs:rw:` grants read + write. A
non-existent granted directory is skipped, so the guest's `open()` fails with a
normal error rather than crashing the host.

Bare `fs` / `filesystem` (the **whole** filesystem — every file and secret) is
**rejected at load**, exactly like bare `env`. Deny-all remains the default: with no
`fs:*` grant, the guest has no filesystem at all. The host decides which paths to
grant — fidius ships the mechanism (preopens), not a path policy.

### Raw sockets (`network`/`sockets`) are coarse — prefer `http`

`network`/`sockets` grant **raw** `wasi:sockets`: outbound TCP/UDP plus DNS, with
no per-host filtering and no place for the host to inspect or decorate a
connection. There is no broker seam, so it cannot be scoped the way HTTP can.

A **baseline SSRF floor** is always applied to this grant: connections to
loopback, link-local (including the cloud metadata IP `169.254.169.254`),
RFC-1918 private ranges, and unique-local addresses are **rejected** — checked on
the *resolved* address, so a hostname that resolves (or rebinds) to an internal IP
is caught too. This floor is a safety net, *not* a policy: it stops the worst
SSRF targets but still lets the plugin reach any *public* host. So treat
`network`/`sockets` as "this plugin may talk to the public internet" and reserve
it for trusted plugins. For the common case — a connector that fetches from a
specific REST API — use **`http`** instead: it is host-brokered and policy-gated
(next section).

## Brokered outbound HTTP (`wasi:http`)

A sandboxed connector whose job is to fetch from an API needs outbound HTTP. The
`http` capability provides it through `wasi:http`, but with a deliberately strict
shape — because an unrestricted egress grant is a sandbox-defeating footgun (it is
a textbook **SSRF** primitive: a hostile or compromised connector could point a
"fetch" at `http://169.254.169.254/…` to steal cloud credentials, or at an
internal admin service).

### fidius ships the mechanism, not the policy

fidius does **not** contain an allow-list, an SSRF denylist, or any credential
logic. What is "internal", which hosts are acceptable, and where secrets live are
all *deployment-specific* — fidius can't know them, and a partial built-in guard
would imply a safety guarantee it can't keep. Instead the host application
supplies an **`EgressPolicy`** hook, and **that hook is the security boundary**.
(This is the same "mechanism, not policy" stance as the streaming pipe — see ADR
FIDIUS-A-0004.)

### Two-key, fail-closed

Outbound HTTP is enabled only when **both** keys are present:

1. **The package declares it** — `capabilities = ["http", …]` in `package.toml`.
   This is *intent*, visible to whoever reviews/signs the connector.
2. **The host supplies an `EgressPolicy`** — give one to `PluginHost`:

   ```rust
   // Host-wide default: every load_wasm'd plugin uses this policy.
   let host = PluginHost::builder().search_path(dir).egress(my_policy).build()?;
   // (already hold an `Arc<dyn EgressPolicy>`? use `.egress_policy(arc)` instead.)
   let p = host.load_wasm("rest-connector", &Connector_WASM_DESCRIPTOR)?;

   // Or per-plugin (isolate connectors — a host-wide policy can't tell which
   // plugin is calling, only what the request is):
   let p = host.load_wasm_with_egress("rest-connector", &desc, stripe_only_policy)?;
   ```

Miss either key and the `wasi:http` imports are simply **not linked**: a component
that imports `wasi:http/outgoing-handler` then fails to instantiate (fails closed
at load). Neither an untrusted package alone nor a forgetful host alone can open
the network.

Every outbound request is a host call across the sandbox boundary, so the hook is
a **per-request** checkpoint — not a one-time gate. fidius calls
`authorize(&mut parts)` before *every* request, then dispatches (or, on `Err`,
refuses and the guest sees an HTTP error):

```rust
use fidius_host::executor::{EgressPolicy, EgressDenied};

pub trait EgressPolicy: Send + Sync + 'static {
    /// Inspect `parts.uri` / `parts.method`, mutate `parts.headers` to inject
    /// credentials, or return `Err` to deny. Called before every request.
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>;
}
```

### Your hook is the security boundary — the checklist

Because fidius enforces *nothing* about the destination, a real policy MUST:

- **Allow-list the destination host** — only let through the host(s) the connector
  is supposed to reach (e.g. `api.stripe.com`). Default-deny everything else.
- **Block SSRF / metadata targets** even for an allow-listed name — reject the
  cloud metadata IP `169.254.169.254`, loopback (`127.0.0.0/8`, `::1`),
  link-local (`169.254.0.0/16`, `fe80::/10`), private ranges
  (`10/8`, `172.16/12`, `192.168/16`) and ULA (`fc00::/7`).
- **Mind DNS rebinding** — an allow-listed *name* can resolve to an internal IP.
  Fully closing this means resolving the host yourself and pinning the connection
  to a vetted IP. At minimum, document the residual risk.
- **Inject credentials host-side** — add `Authorization`/tokens here so the guest
  never holds the secret. (Note: this only holds if the connector is *not* also
  granted `env` — a coarse `env` grant leaks every host secret regardless. See the
  warning below.)

### A worked reference policy

This is the shape an embedder writes — copy and harden it; fidius does not ship it
as an API:

```rust
use std::collections::HashSet;
use std::net::IpAddr;
use fidius_host::executor::{EgressPolicy, EgressDenied};

struct ApiEgress {
    allowed_hosts: HashSet<String>,        // e.g. {"api.stripe.com"}
    bearer: Option<String>,                // host-side secret, never in the .wasm
    allow_loopback: bool,                  // true ONLY for local tests
}

impl EgressPolicy for ApiEgress {
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        let authority = parts.uri.authority().ok_or_else(|| EgressDenied::new("no authority"))?;
        let host = authority.host();

        // 1. allow-list
        if !self.allowed_hosts.contains(host) {
            return Err(EgressDenied::new(format!("host not allowed: {host}")));
        }
        // 2. SSRF guard on literal IPs (rebinding of a name is a documented residual)
        if let Ok(ip) = host.parse::<IpAddr>() {
            let internal = ip.is_loopback() || is_link_local(&ip) || is_private(&ip);
            if internal && !(self.allow_loopback && ip.is_loopback()) {
                return Err(EgressDenied::new(format!("internal address blocked: {ip}")));
            }
        }
        // 3. credential injection — guest never sees the secret
        if let Some(token) = &self.bearer {
            parts.headers.insert(
                http::header::AUTHORIZATION,
                format!("Bearer {token}").parse().unwrap(),
            );
        }
        Ok(())
    }
}
```

(`is_link_local`/`is_private` are small helpers over `IpAddr`; the metadata IP
`169.254.169.254` is link-local, so it's covered. The egress E2E test in
`crates/fidius-host/tests/wasm_egress_e2e.rs` exercises allow/deny/fail-closed
against a real `wasi:http` guest with policies like this.)

### The guest side: `fidius_guest::http`

The policy is half the loop; the connector is the other half. A connector written
with the fidius macros makes its outbound call through **`fidius_guest::http`** —
no hand-written WIT, no raw `wasi:http` bindings:

```rust
#[plugin_impl(Source, crate = "fidius_guest")]
impl Source for MySource {
    fn read(&self, cfg: Config) -> fidius_guest::Stream<Record> {
        // Brokered by the host EgressPolicy. Err == denied / blocked / unreachable.
        let resp = fidius_guest::http::get(&cfg.url).expect("fetch");
        // … parse resp.text() / resp.body into records …
    }
}
```

Declaring `capabilities = ["http"]` in `package.toml` is all the extra wiring:
`#[plugin_impl]` auto-exports the component and `fidius_guest::http` contributes
the `wasi:http` import — the two `wit_bindgen::generate!` compose into one
component automatically, with no macro change. The module is `wasm32-wasip2`-only;
a native/cdylib connector uses a normal HTTP client directly (it already has the
host's authority). *Unify the contract, not the capabilities* — the trust tiers
legitimately differ.

### Version compatibility (the contract)

`fidius_guest::http` pins **one** `wasi:http` version for the whole ecosystem
(vendored at `crates/fidius-guest/wit`, matched to the host's
`wasmtime-wasi-http`; see ADR *WASM guest wasi:http — pinned client contract*).
The rule a distributed connector can rely on:

> A wasm plugin built against **fidius-guest X** runs on any **fidius-host ≥ X**
> within the same `wasi:http` line. Upgrading the host (newer wasmtime) does not
> break existing connectors — WASI 0.2 is a stable line, so a newer host satisfies
> an older import. A `wasi:http` **major** bump (0.2→0.3) is a breaking change
> shipped only in a fidius **major** release.

fidius treats the guest's `wasi:http` version as a *published ABI*: it moves
rarely and deliberately, never to chase a wasmtime patch. The macro-authored
connector E2E (`crates/fidius-host/tests/macro_egress_e2e.rs`) is the drift guard
— if the guest's pinned version ever diverges from the host's, that test fails to
instantiate (red in CI) before it can ship.

### A note on `env` vs credential injection

Injecting a secret in the hook only keeps it from the guest **if that guest can't
read it another way**. This is why `env` is **per-variable** (`env:VAR_NAME`) and
bare `env` (inherit-all) is rejected: a connector you broker HTTP for can't reach
your other secrets through the environment. The one rule that remains yours: don't
grant a connector `env:THE_SECRET_VAR` for the very token you're injecting on its
behalf — broker it in the hook *or* expose it via `env`, not both.

## How a deployer reasons about it

Because the package is **signed** (see [Signing Plugins](../tutorials/signing-plugins.md))
and the signature covers the whole package including `package.toml`, the
capability list cannot be altered after signing without invalidating the
signature. So the trust workflow is:

1. `fidius package inspect` the package and read the `Capabilities` line.
2. Decide whether those grants are acceptable for this plugin's job.
3. Verify the signature against a trusted key (`require_signature(true)` +
   `trusted_keys`), which also guarantees the capability list is the one the
   signer approved.

A plugin asking for `network` when it claims to be a pure data transform is a red
flag the allow-list makes visible.

## Relation to the interface hash

Capabilities are about *authority*; the `fidius-interface-hash` is about
*contract integrity* (the plugin implements the interface the host expects). They
are independent: the hash check rejects a plugin built against the wrong
interface; the capability list bounds what a correctly-typed plugin may do; and
the signature is the security boundary over both. See the
[WASM Component ABI](wasm-component-abi.md) for the hash, and
[Your First WASM Plugin](../tutorials/your-first-wasm-plugin.md) for the end-to-end
flow.
