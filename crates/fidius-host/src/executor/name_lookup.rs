// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Resolve-and-pin for `wasi:sockets` name lookups (FIDIUS-I-0034).
//!
//! fidius shadows `wasi:sockets/ip-name-lookup` in the linker (after
//! `add_to_linker_sync`, with `allow_shadowing(true)`) with this module's
//! implementation, which:
//!
//! 1. consults [`EgressPolicy::authorize_dns`] **before** resolving — a denial
//!    fails the guest's lookup with `permanent-resolver-failure` (the same
//!    error upstream returns for a lookup denied outright), resolves nothing,
//!    and pins nothing;
//! 2. resolves host-side (std `ToSocketAddrs`, matching upstream, unless a
//!    test injects a resolver); and
//! 3. records `name ↔ IPs` in a per-store [`PinTable`] **inside** the blocking
//!    resolution task — the pin is written before the future completes, so no
//!    address ever reaches the guest un-pinned.
//!
//! `socket_addr_check` then recovers the dialed name for a connect's IP from
//! the pin table and hands the policy a `TcpTarget { host: Some(name), .. }`.
//! The shadow is installed only under the same two-key condition that enables
//! name lookup at all (a `tcp`/`udp` grant AND an embedder policy); otherwise
//! upstream's implementation stands untouched.

use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use wasmtime::component::{HasData, Resource};
use wasmtime_wasi::p2::bindings::sockets::ip_name_lookup::{
    Host, HostResolveAddressStream, ResolveAddressStream,
};
use wasmtime_wasi::p2::bindings::sockets::network::{self, ErrorCode, IpAddress, Network};
use wasmtime_wasi::p2::SocketError;
use wasmtime_wasi::runtime::{poll_noop, spawn_blocking};
use wasmtime_wasi::ResourceTable;
use wasmtime_wasi_io::poll::{subscribe, DynPollable};

use super::wasm::EgressPolicy;

/// What this store's lookups pinned: both directions of `name ↔ IPs`.
///
/// Semantics (FIDIUS-I-0034): names are stored ASCII-lowercase, IPs canonical
/// (`to_canonical()`), so neither case nor v4-mapped-v6 spelling can dodge a
/// pin. Re-resolving a name replaces its entry wholesale — IPs the old
/// resolution mapped that the new one no longer does are unpinned (a stale pin
/// must not authorize). When two pinned names share an IP, the most recent
/// resolution wins the `IP → name` side.
#[derive(Default)]
pub(crate) struct PinState {
    by_name: HashMap<String, Vec<IpAddr>>,
    by_ip: HashMap<IpAddr, String>,
}

impl PinState {
    /// Record one completed resolution of `name` (already lowercased) to
    /// `ips` (already canonicalized), applying replace-on-re-resolve.
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

    /// The name this store's lookups most recently resolved to `ip`
    /// (canonicalize before calling), if any.
    pub(crate) fn host_for(&self, ip: &IpAddr) -> Option<String> {
        self.by_ip.get(ip).cloned()
    }
}

/// Shared handle to a store's pins: one clone lives in `HostState` (feeding
/// the shadowed lookup), one in the `WasiCtx`'s `socket_addr_check` closure
/// (recovering the name at connect time), and one in each in-flight blocking
/// resolution task. Lifetime = the store: per-call for unary dispatch, the
/// persistent store's lifetime for configured instances.
pub(crate) type PinTable = Arc<Mutex<PinState>>;

/// The host-side resolution function. Injectable so tests can model
/// multi-name/same-IP and rotation without real DNS (`#[doc(hidden)]` seam on
/// the executor); the default matches upstream wasmtime-wasi: std
/// `ToSocketAddrs` on `(name, 0)`.
pub(crate) type Resolver = Arc<dyn Fn(&str) -> std::io::Result<Vec<IpAddr>> + Send + Sync>;

pub(crate) fn default_resolver() -> Resolver {
    Arc::new(|name| Ok((name, 0).to_socket_addrs()?.map(|sa| sa.ip()).collect()))
}

/// `HasData` marker for the shadowed instance (the `D` in `add_to_linker`).
pub(crate) struct FidiusNameLookup;

impl HasData for FidiusNameLookup {
    type Data<'a> = NameLookupView<'a>;
}

/// Per-call view the bindgen host traits run against: the store's own
/// `ResourceTable` (shared with the rest of WASI so `network` handles and the
/// `resolve-address-stream` resource interoperate), plus the pin table, the
/// embedder policy, and the resolver.
pub(crate) struct NameLookupView<'a> {
    pub table: &'a mut ResourceTable,
    pub pins: &'a PinTable,
    pub policy: Option<&'a Arc<dyn EgressPolicy>>,
    pub resolver: &'a Resolver,
}

// `ip_name_lookup::add_to_linker` bounds the view by `network::Host` (owner of
// the trappable `error-code` conversion) and its `HostNetwork` resource
// supertrait. Glue only, mirrored from wasmtime-wasi's own impls — this does
// NOT re-register the `wasi:sockets/network` instance.
impl network::Host for NameLookupView<'_> {
    fn convert_error_code(&mut self, error: SocketError) -> wasmtime::Result<ErrorCode> {
        error.downcast()
    }

    fn network_error_code(
        &mut self,
        err: Resource<wasmtime::Error>,
    ) -> wasmtime::Result<Option<ErrorCode>> {
        let err = self.table.get(&err)?;
        if let Some(err) = err.downcast_ref::<std::io::Error>() {
            return Ok(Some(ErrorCode::from(err)));
        }
        Ok(None)
    }
}

impl network::HostNetwork for NameLookupView<'_> {
    fn drop(&mut self, this: Resource<Network>) -> wasmtime::Result<()> {
        self.table.delete(this)?;
        Ok(())
    }
}

impl Host for NameLookupView<'_> {
    fn resolve_addresses(
        &mut self,
        network: Resource<Network>,
        name: String,
    ) -> Result<Resource<ResolveAddressStream>, SocketError> {
        // Parity with upstream: the network handle must be live. (Upstream
        // additionally consults its private allow_ip_name_lookup flag; fidius
        // gates equivalently by only installing this shadow under a tcp/udp
        // grant + policy — the same condition that sets that flag.)
        let _ = self.table.get(&network)?;

        // DNS is case-insensitive: normalize once, here, so the pin, the
        // TcpTarget the policy sees, and authorize_dns all agree.
        let name = name.to_ascii_lowercase();

        // The lookup gate (FIDIUS-I-0034): deny → the guest sees the same
        // resolver failure as a lookup denied outright. Nothing resolved,
        // nothing pinned.
        if let Some(policy) = self.policy {
            if policy.authorize_dns(&name).is_err() {
                return Err(ErrorCode::PermanentResolverFailure.into());
            }
        }

        // An IP-literal "lookup" resolves to itself and pins nothing — the
        // guest did not dial by name, so a name-keyed policy sees host: None.
        if let Ok(ip) = name.parse::<IpAddr>() {
            let addrs = vec![IpAddress::from(ip.to_canonical())];
            let resource = self
                .table
                .push(ResolveAddressStream::Done(Ok(addrs.into_iter())))?;
            return Ok(resource);
        }

        // Resolve off-thread (upstream's non-blocking shape). The pin is
        // written inside the task, after resolution succeeds and before the
        // future completes — the guest cannot observe an address that was
        // never pinned.
        let resolver = self.resolver.clone();
        let pins = self.pins.clone();
        let task = spawn_blocking(move || {
            let ips: Vec<IpAddr> = resolver(&name)
                .map_err(|_| SocketError::from(ErrorCode::NameUnresolvable))?
                .into_iter()
                .map(|ip| ip.to_canonical())
                .collect();
            pins.lock().unwrap().record(&name, &ips);
            Ok(ips.into_iter().map(IpAddress::from).collect())
        });
        let resource = self.table.push(ResolveAddressStream::Waiting(task))?;
        Ok(resource)
    }
}

// Mirrors upstream's stream methods exactly — the resource type IS upstream's
// `ResolveAddressStream`, so pollable semantics (subscribe/ready) come with it.
impl HostResolveAddressStream for NameLookupView<'_> {
    fn resolve_next_address(
        &mut self,
        resource: Resource<ResolveAddressStream>,
    ) -> Result<Option<IpAddress>, SocketError> {
        let stream: &mut ResolveAddressStream = self.table.get_mut(&resource)?;
        loop {
            match stream {
                ResolveAddressStream::Waiting(future) => match poll_noop(Pin::new(future)) {
                    Some(result) => {
                        *stream = ResolveAddressStream::Done(result.map(|v| v.into_iter()));
                    }
                    None => return Err(ErrorCode::WouldBlock.into()),
                },
                ResolveAddressStream::Done(slot @ Err(_)) => {
                    std::mem::replace(slot, Ok(Vec::new().into_iter()))?;
                    unreachable!();
                }
                ResolveAddressStream::Done(Ok(iter)) => return Ok(iter.next()),
            }
        }
    }

    fn subscribe(
        &mut self,
        resource: Resource<ResolveAddressStream>,
    ) -> wasmtime::Result<Resource<DynPollable>> {
        subscribe(self.table, resource)
    }

    fn drop(&mut self, resource: Resource<ResolveAddressStream>) -> wasmtime::Result<()> {
        self.table.delete(resource)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn record_and_lookup() {
        let mut pins = PinState::default();
        pins.record("db.internal", &[ip("10.0.0.5"), ip("10.0.0.6")]);
        assert_eq!(
            pins.host_for(&ip("10.0.0.5")).as_deref(),
            Some("db.internal")
        );
        assert_eq!(
            pins.host_for(&ip("10.0.0.6")).as_deref(),
            Some("db.internal")
        );
        assert_eq!(pins.host_for(&ip("10.0.0.7")), None);
    }

    #[test]
    fn re_resolve_replaces_wholesale() {
        // Rotation: the stale IP must no longer resolve to the name.
        let mut pins = PinState::default();
        pins.record("db.internal", &[ip("10.0.0.5")]);
        pins.record("db.internal", &[ip("10.0.0.9")]);
        assert_eq!(pins.host_for(&ip("10.0.0.5")), None);
        assert_eq!(
            pins.host_for(&ip("10.0.0.9")).as_deref(),
            Some("db.internal")
        );
    }

    #[test]
    fn ip_collision_most_recent_wins_without_clobbering_on_stale_removal() {
        let mut pins = PinState::default();
        pins.record("a.internal", &[ip("10.0.0.5")]);
        pins.record("b.internal", &[ip("10.0.0.5")]);
        // Most recent name wins the reverse mapping.
        assert_eq!(
            pins.host_for(&ip("10.0.0.5")).as_deref(),
            Some("b.internal")
        );
        // `a` re-resolving away must NOT unpin `b`'s claim on the shared IP.
        pins.record("a.internal", &[ip("10.0.0.6")]);
        assert_eq!(
            pins.host_for(&ip("10.0.0.5")).as_deref(),
            Some("b.internal")
        );
        assert_eq!(
            pins.host_for(&ip("10.0.0.6")).as_deref(),
            Some("a.internal")
        );
    }

    #[test]
    fn default_resolver_resolves_localhost() {
        let ips = default_resolver()("localhost").unwrap();
        assert!(ips.iter().all(|ip| ip.is_loopback()), "{ips:?}");
        assert!(!ips.is_empty());
    }
}
