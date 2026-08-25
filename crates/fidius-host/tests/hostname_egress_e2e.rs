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

//! Hostname-carrying TCP egress E2E (FIDIUS-I-0034): the production
//! resolve-and-pin path. A real WASM guest (`tests/wasm-fixtures/tcp-echo`)
//! dials through `std::net::TcpStream`; fidius's shadowed
//! `wasi:sockets/ip-name-lookup` pins the guest's lookups, and the
//! `socket_addr_check` hands the embedder policy a
//! `TcpTarget { host, addr }` — `Some(name)` for hostname dials, `None` for
//! IP literals. `authorize_dns` gates the lookup itself.

#![cfg(feature = "wasm")]

use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use fidius_host::executor::{
    EgressDenied, EgressPolicy, TcpTarget, WasmComponentExecutor, WasmMethod,
};
use fidius_host::{PluginHandle, PluginInfo, PluginRuntimeKind};

const IFACE: &str = "fidius:tcp-echo/tcp-echo@1.0.0";

// ── fixture + mock server (mirrors tcp_egress_e2e.rs) ──────────────────────

fn tcp_echo_component() -> Option<Vec<u8>> {
    static BYTES: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    BYTES
        .get_or_init(|| {
            let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/wasm-fixtures/tcp-echo");
            let built = Command::new("cargo")
                .args(["build", "--target", "wasm32-wasip2", "--release"])
                .current_dir(&fixture)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !built {
                return None;
            }
            let art = fixture.join("target/wasm32-wasip2/release/tcp_echo_guest.wasm");
            std::fs::read(&art).ok()
        })
        .clone()
}

fn mock_tcp_echo_once(n: usize) -> (u16, std::thread::JoinHandle<()>) {
    mock_tcp_echo_once_on("127.0.0.1", n)
}

/// Like `mock_tcp_echo_once`, but bound to a chosen loopback flavor — the
/// rotation test needs two DISTINCT IPs (`127.0.0.1` and `::1`) so a rotated
/// resolution actually changes the pinned address.
fn mock_tcp_echo_once_on(host: &str, n: usize) -> (u16, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind((host, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = vec![0u8; n];
            if stream.read_exact(&mut buf).is_ok() {
                let _ = stream.write_all(&buf);
                let _ = stream.flush();
            }
        }
    });
    (port, h)
}

// ── reference policies ─────────────────────────────────────────────────────

/// What the check handed the policy, recorded for assertions: the future
/// weir-style hostname allow-list, plus a witness log.
struct NameAllowList {
    allowed: Vec<String>,
    seen: Arc<Mutex<Vec<(Option<String>, SocketAddr)>>>,
    deny_dns_for: Option<String>,
}

impl NameAllowList {
    fn new(allowed: &[&str]) -> Self {
        Self {
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
            seen: Arc::new(Mutex::new(Vec::new())),
            deny_dns_for: None,
        }
    }
}

impl EgressPolicy for NameAllowList {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("http denied by tcp-only test policy"))
    }
    // NOTE: authorize_tcp is deliberately NOT overridden — authorization goes
    // exclusively by name. If the pin were bypassed (host: None), the default
    // authorize_tcp denies, so any success below proves the pin worked.
    fn authorize_tcp_target(&self, target: &TcpTarget<'_>) -> Result<(), EgressDenied> {
        self.seen
            .lock()
            .unwrap()
            .push((target.host.map(str::to_string), target.addr));
        match target.host {
            Some(host) if self.allowed.iter().any(|a| a == host) => Ok(()),
            _ => Err(EgressDenied::new("host not in name allow-list")),
        }
    }
    fn authorize_dns(&self, name: &str) -> Result<(), EgressDenied> {
        match &self.deny_dns_for {
            Some(denied) if denied == name => Err(EgressDenied::new("dns denied")),
            _ => Ok(()),
        }
    }
}

/// The executor's `#[doc(hidden)]` resolver seam — tests inject one to model
/// multi-name/same-IP and rotation without real DNS.
type TestResolver = Arc<dyn Fn(&str) -> std::io::Result<Vec<IpAddr>> + Send + Sync>;

/// Method indices, in the order `load_with` declares them.
const CONNECT_AND_ECHO: usize = 0;
const CONNECT_SEQ: usize = 1;

fn load(policy: Arc<dyn EgressPolicy>) -> PluginHandle {
    load_with(policy, None)
}

fn load_with(policy: Arc<dyn EgressPolicy>, resolver: Option<TestResolver>) -> PluginHandle {
    let bytes = tcp_echo_component().expect("tcp_echo_guest.wasm present");
    let info = PluginInfo {
        name: "tcp-echo".into(),
        interface_name: "tcp-echo".into(),
        interface_hash: 0,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: fidius_core::descriptor::BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![
        WasmMethod {
            name: "connect-and-echo".into(),
            wire_raw: false,
            streaming: false,
        },
        WasmMethod {
            name: "connect-seq".into(),
            wire_raw: false,
            streaming: false,
        },
    ];
    let mut exec = WasmComponentExecutor::from_component_bytes_with_egress(
        &bytes,
        IFACE.into(),
        methods,
        vec!["tcp".into()],
        Some(policy),
        info,
    )
    .expect("load");
    if let Some(resolver) = resolver {
        exec.set_resolver(resolver);
    }
    PluginHandle::from_wasm(exec)
}

// ── the tests ──────────────────────────────────────────────────────────────

/// Production path, initiative acceptance criterion 1 (hostname half): a guest
/// dialing `localhost:<port>` reaches the policy as
/// `TcpTarget { host: Some("localhost"), addr }` and a name-keyed allow-list
/// authorizes it — bytes round-trip.
#[test]
fn hostname_dial_reaches_policy_with_name_and_echoes() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP hostname_dial_reaches_policy_with_name_and_echoes: fixture not built");
        return;
    }
    let payload = b"select 1".to_vec();
    let (port, server) = mock_tcp_echo_once(payload.len());
    let policy = Arc::new(NameAllowList::new(&["localhost"]));
    let seen = policy.seen.clone();
    let handle = load(policy);
    let echoed: Vec<u8> = handle
        .call_method(0, &(format!("localhost:{port}"), payload.clone()))
        .expect("echo");
    server.join().unwrap();
    assert_eq!(echoed, payload, "name-authorized connect must round-trip");
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, addr)| host.as_deref() == Some("localhost") && addr.port() == port),
        "policy must see TcpTarget {{ host: Some(\"localhost\") }}; saw {observed:?}"
    );
}

/// Production path, criterion 1 (IP half) + pin correctness: an IP-literal
/// dial performs no lookup, so the policy sees `host: None` and the
/// name-keyed allow-list denies — no IP fallthrough.
#[test]
fn ip_literal_dial_reaches_policy_as_none_and_is_denied() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP ip_literal_dial_reaches_policy_as_none_and_is_denied: fixture not built");
        return;
    }
    let policy = Arc::new(NameAllowList::new(&["localhost"]));
    let seen = policy.seen.clone();
    let handle = load(policy);
    let echoed: Vec<u8> = handle
        .call_method(0, &("127.0.0.1:1".to_string(), b"x".to_vec()))
        .expect("call");
    assert!(
        echoed.is_empty(),
        "IP-literal dial must be denied by a name-keyed policy"
    );
    let observed = seen.lock().unwrap();
    assert!(
        observed.iter().any(|(host, _)| host.is_none()),
        "policy must see host: None for an IP-literal dial; saw {observed:?}"
    );
}

/// `authorize_dns` denial: the guest's lookup fails before resolution — the
/// connect never happens (`authorize_tcp_target` is never consulted) and the
/// guest gets an empty result.
#[test]
fn authorize_dns_denial_fails_lookup_before_connect() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP authorize_dns_denial_fails_lookup_before_connect: fixture not built");
        return;
    }
    let mut policy = NameAllowList::new(&["localhost"]);
    policy.deny_dns_for = Some("localhost".to_string());
    let policy = Arc::new(policy);
    let seen = policy.seen.clone();
    let handle = load(policy);
    let echoed: Vec<u8> = handle
        .call_method(0, &("localhost:1".to_string(), b"x".to_vec()))
        .expect("call");
    assert!(echoed.is_empty(), "denied lookup must yield no bytes");
    assert!(
        seen.lock().unwrap().is_empty(),
        "a denied lookup must never reach authorize_tcp_target"
    );
}

/// Default delegation, e2e flavor (initiative criterion 2): a pre-0034 policy
/// overriding ONLY `authorize_tcp` still authorizes a hostname dial — the
/// default `authorize_tcp_target` delegates on the resolved addr, so behavior
/// is byte-identical to before resolve-and-pin existed.
struct LegacyLoopbackOnly;
impl EgressPolicy for LegacyLoopbackOnly {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("http denied"))
    }
    fn authorize_tcp(&self, addr: &SocketAddr) -> Result<(), EgressDenied> {
        if addr.ip().is_loopback() {
            Ok(())
        } else {
            Err(EgressDenied::new("only loopback"))
        }
    }
}

#[test]
fn legacy_policy_hostname_dial_unchanged() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP legacy_policy_hostname_dial_unchanged: fixture not built");
        return;
    }
    let payload = b"legacy".to_vec();
    let (port, server) = mock_tcp_echo_once(payload.len());
    let handle = load(Arc::new(LegacyLoopbackOnly));
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("localhost:{port}"), payload.clone()),
        )
        .expect("echo");
    server.join().unwrap();
    assert_eq!(
        echoed, payload,
        "an authorize_tcp-only policy must be unaffected"
    );
}

/// A resolver mapping fixed names to fixed IPs — the injected stand-in for DNS.
fn map_resolver(entries: &[(&str, IpAddr)]) -> TestResolver {
    let map: Vec<(String, IpAddr)> = entries.iter().map(|(n, ip)| (n.to_string(), *ip)).collect();
    Arc::new(move |name| {
        map.iter()
            .filter(|(n, _)| n == name)
            .map(|(_, ip)| Ok(*ip))
            .next()
            .unwrap_or_else(|| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "unknown name",
                ))
            })
            .map(|ip| vec![ip])
    })
}

/// Pin correctness, not IP fallthrough (initiative criterion 3): two names
/// resolve to the SAME IP inside one instance; the listed name connects, the
/// unlisted one is denied — even though that exact IP was authorized moments
/// earlier under the listed name (most-recent pin wins, no residual authority).
#[test]
fn same_ip_second_name_denied_unless_listed() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP same_ip_second_name_denied_unless_listed: fixture not built");
        return;
    }
    let payload = b"pin".to_vec();
    let (p1, server) = mock_tcp_echo_once(payload.len());
    let loopback: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver = map_resolver(&[("db.internal", loopback), ("evil.internal", loopback)]);
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let seen = policy.seen.clone();
    let handle = load_with(policy, Some(resolver));
    // One guest call = one store = one pin table shared by both dials.
    let results: Vec<Vec<u8>> = handle
        .call_method(
            CONNECT_SEQ,
            &(
                vec![format!("db.internal:{p1}"), format!("evil.internal:{p1}")],
                payload.clone(),
            ),
        )
        .expect("seq");
    server.join().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], payload, "listed name must connect");
    assert!(
        results[1].is_empty(),
        "unlisted name must be denied even on an IP the listed name just used"
    );
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, _)| host.as_deref() == Some("evil.internal")),
        "the denied connect must have been attributed to the dialed name; saw {observed:?}"
    );
}

/// Unlisted name → denied (criterion 3, simple half): no allow-list entry, no
/// connect, even though the name resolves fine.
#[test]
fn unlisted_name_denied() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP unlisted_name_denied: fixture not built");
        return;
    }
    let loopback: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver = map_resolver(&[("other.internal", loopback)]);
    let handle = load_with(
        Arc::new(NameAllowList::new(&["db.internal"])),
        Some(resolver),
    );
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &("other.internal:1".to_string(), b"x".to_vec()),
        )
        .expect("call");
    assert!(
        echoed.is_empty(),
        "a name not on the allow-list must be denied"
    );
}

/// Pin attribution: after `db.internal` resolves to an IP, a LITERAL dial to
/// that IP (same store) arrives as `host: Some("db.internal")` — the pin says
/// what this instance was told that address is. This is the documented TOCTOU
/// narrowing: the guest may connect to any address it was actually given for a
/// name, under that name's authority.
#[test]
fn literal_dial_to_pinned_ip_is_attributed_to_the_name() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP literal_dial_to_pinned_ip_is_attributed_to_the_name: fixture not built");
        return;
    }
    let payload = b"attrib".to_vec();
    let (p1, s1) = mock_tcp_echo_once(payload.len());
    let (p2, s2) = mock_tcp_echo_once(payload.len());
    let loopback: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver = map_resolver(&[("db.internal", loopback)]);
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let handle = load_with(policy, Some(resolver));
    let results: Vec<Vec<u8>> = handle
        .call_method(
            CONNECT_SEQ,
            &(
                vec![format!("db.internal:{p1}"), format!("127.0.0.1:{p2}")],
                payload.clone(),
            ),
        )
        .expect("seq");
    s1.join().unwrap();
    s2.join().unwrap();
    assert_eq!(results[0], payload);
    assert_eq!(
        results[1], payload,
        "a literal dial to the pinned IP carries the name's authority"
    );
}

/// Rotation / resident lifetime (initiative criterion 4): within ONE store,
/// `db.internal` first resolves to 127.0.0.1, then rotates to ::1. After
/// re-resolution the new pin authorizes ::1 — and the STALE pin is gone: a
/// literal dial back to 127.0.0.1 now arrives as `host: None` and the
/// name-keyed policy denies it. (Same-store sequential dials stand in for a
/// resident configured instance — the store, and thus the pin table, is the
/// unit of lifetime either way.)
#[test]
fn rotation_replaces_pin_and_stale_ip_loses_authority() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP rotation_replaces_pin_and_stale_ip_loses_authority: fixture not built");
        return;
    }
    let payload = b"rotate".to_vec();
    let (p1, s1) = mock_tcp_echo_once_on("127.0.0.1", payload.len());
    let (p2, s2) = mock_tcp_echo_once_on("::1", payload.len());
    let v4: IpAddr = "127.0.0.1".parse().unwrap();
    let v6: IpAddr = "::1".parse().unwrap();
    // Stateful resolver: first resolution of db.internal → 127.0.0.1, every
    // later one → ::1 (the rotated endpoint).
    let calls = Arc::new(AtomicUsize::new(0));
    let resolver: TestResolver = Arc::new(move |name| {
        if name == "db.internal" {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![if n == 0 { v4 } else { v6 }])
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "unknown name",
            ))
        }
    });
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let seen = policy.seen.clone();
    let handle = load_with(policy, Some(resolver));
    let results: Vec<Vec<u8>> = handle
        .call_method(
            CONNECT_SEQ,
            &(
                vec![
                    format!("db.internal:{p1}"), // resolves → 127.0.0.1, pinned
                    format!("db.internal:{p2}"), // re-resolves → ::1, pin REPLACED
                    format!("127.0.0.1:{p1}"),   // stale IP: pin gone → host: None → denied
                ],
                payload.clone(),
            ),
        )
        .expect("seq");
    s1.join().unwrap();
    s2.join().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], payload, "pre-rotation endpoint must work");
    assert_eq!(
        results[1], payload,
        "re-resolution must authorize the rotated endpoint"
    );
    assert!(
        results[2].is_empty(),
        "the stale pin must NOT authorize the old IP after rotation"
    );
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, addr)| host.is_none() && addr.ip() == v4),
        "the post-rotation literal dial must arrive unpinned (host: None); saw {observed:?}"
    );
}

/// Case-insensitivity (DNS is case-insensitive): the guest dials
/// `DB.INTERNAL`; the pin, the policy's `TcpTarget.host`, and `authorize_dns`
/// all see the lowercased name, so a lowercase allow-list entry matches.
#[test]
fn mixed_case_dial_matches_lowercase_allow_list() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP mixed_case_dial_matches_lowercase_allow_list: fixture not built");
        return;
    }
    let payload = b"case".to_vec();
    let (port, server) = mock_tcp_echo_once(payload.len());
    let loopback: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver = map_resolver(&[("db.internal", loopback)]);
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let seen = policy.seen.clone();
    let handle = load_with(policy, Some(resolver));
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("DB.INTERNAL:{port}"), payload.clone()),
        )
        .expect("echo");
    server.join().unwrap();
    assert_eq!(echoed, payload, "case must not defeat a name allow-list");
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, _)| host.as_deref() == Some("db.internal")),
        "the policy must see the lowercased name; saw {observed:?}"
    );
}

/// Unresolvable name: the resolver has no entry → the guest's lookup fails,
/// nothing is pinned, and the policy is never consulted (there is no connect).
#[test]
fn unresolvable_name_fails_lookup_without_reaching_policy() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP unresolvable_name_fails_lookup_without_reaching_policy: fixture not built");
        return;
    }
    let loopback: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver = map_resolver(&[("db.internal", loopback)]);
    let policy = Arc::new(NameAllowList::new(&["db.internal", "nope.internal"]));
    let seen = policy.seen.clone();
    let handle = load_with(policy, Some(resolver));
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &("nope.internal:1".to_string(), b"x".to_vec()),
        )
        .expect("call");
    assert!(
        echoed.is_empty(),
        "an unresolvable name must yield no bytes"
    );
    assert!(
        seen.lock().unwrap().is_empty(),
        "a failed lookup must never reach authorize_tcp_target"
    );
}

/// Multi-address resolution: a name resolving to several IPs pins ALL of
/// them, so the guest's normal connect fallback (first address unreachable →
/// try the next) proceeds under the name's authority at every step.
#[test]
fn multi_ip_resolution_pins_all_candidates() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP multi_ip_resolution_pins_all_candidates: fixture not built");
        return;
    }
    let payload = b"multi".to_vec();
    // Listener only on the SECOND candidate — the guest must fall through.
    let (port, server) = mock_tcp_echo_once_on("127.0.0.1", payload.len());
    let v6: IpAddr = "::1".parse().unwrap();
    let v4: IpAddr = "127.0.0.1".parse().unwrap();
    let resolver: TestResolver = Arc::new(move |name| {
        if name == "db.internal" {
            Ok(vec![v6, v4])
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "unknown name",
            ))
        }
    });
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let seen = policy.seen.clone();
    let handle = load_with(policy, Some(resolver));
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("db.internal:{port}"), payload.clone()),
        )
        .expect("echo");
    server.join().unwrap();
    assert_eq!(
        echoed, payload,
        "fallback across resolved candidates must work"
    );
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, addr)| host.as_deref() == Some("db.internal") && addr.ip() == v4),
        "the fallback candidate must carry the name's authority too; saw {observed:?}"
    );
}

/// The REAL resident-instance path (initiative criterion 4, production
/// flavor): `configure()` parks the instance on a persistent store, so the
/// pin table lives across SEPARATE `call_method` invocations. Rotation
/// between calls replaces the pin; the old IP loses the name's authority for
/// later calls.
#[test]
fn configured_instance_pins_persist_and_rotate_across_calls() {
    if tcp_echo_component().is_none() {
        eprintln!(
            "SKIP configured_instance_pins_persist_and_rotate_across_calls: fixture not built"
        );
        return;
    }
    let payload = b"resident".to_vec();
    let (p1, s1) = mock_tcp_echo_once_on("127.0.0.1", payload.len());
    let (p2, s2) = mock_tcp_echo_once_on("::1", payload.len());
    let v4: IpAddr = "127.0.0.1".parse().unwrap();
    let v6: IpAddr = "::1".parse().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let resolver: TestResolver = Arc::new(move |name| {
        if name == "db.internal" {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![if n == 0 { v4 } else { v6 }])
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "unknown name",
            ))
        }
    });
    let policy = Arc::new(NameAllowList::new(&["db.internal"]));
    let seen = policy.seen.clone();

    // Build by hand so we can set the resolver AND configure() — the
    // persistent store is created here, resolver and pins ride in it.
    let bytes = tcp_echo_component().expect("fixture");
    let info = PluginInfo {
        name: "tcp-echo".into(),
        interface_name: "tcp-echo".into(),
        interface_hash: 0,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: fidius_core::descriptor::BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![
        WasmMethod {
            name: "connect-and-echo".into(),
            wire_raw: false,
            streaming: false,
        },
        WasmMethod {
            name: "connect-seq".into(),
            wire_raw: false,
            streaming: false,
        },
    ];
    let mut exec = WasmComponentExecutor::from_component_bytes_with_egress(
        &bytes,
        IFACE.into(),
        methods,
        vec!["tcp".into()],
        Some(policy),
        info,
    )
    .expect("load");
    exec.set_resolver(resolver);
    exec.configure(&[])
        .expect("configure onto persistent store");
    let handle = PluginHandle::from_wasm(exec);

    // Call 1: resolves → v4, pinned, allowed.
    let e1: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("db.internal:{p1}"), payload.clone()),
        )
        .expect("call 1");
    s1.join().unwrap();
    assert_eq!(e1, payload, "pre-rotation call must echo");

    // Call 2 (SEPARATE call, same store): re-resolves → v6, pin replaced.
    let e2: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("db.internal:{p2}"), payload.clone()),
        )
        .expect("call 2");
    s2.join().unwrap();
    assert_eq!(
        e2, payload,
        "post-rotation call must echo via the new endpoint"
    );

    // Call 3: the OLD IP, dialed as a literal, in yet another call. If the
    // stale pin survived across calls it would still carry db.internal's
    // authority — it must not.
    let e3: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("127.0.0.1:{p1}"), payload.clone()),
        )
        .expect("call 3");
    assert!(
        e3.is_empty(),
        "the rotated-away IP must be unpinned across calls on the resident store"
    );
    let observed = seen.lock().unwrap();
    assert!(
        observed
            .iter()
            .any(|(host, addr)| host.is_none() && addr.ip() == v4 && addr.port() == p1),
        "call 3 must arrive unpinned (host: None); saw {observed:?}"
    );
}

// ── high-level (builder) path parity ───────────────────────────────────────

/// The ergonomic embedder path — `PluginHost::builder().egress(..)` +
/// `load_wasm` — exercises the same shadow: a hostname dial is authorized by
/// name. (No resolver injection on this path; a real `localhost` resolution
/// serves as the no-injection smoke case, mirroring `tcp_egress_e2e`'s
/// builder tests.)
#[test]
fn builder_path_hostname_dial_authorized_by_name() {
    use fidius_core::wasm_descriptor::{WasmInterfaceDescriptor, WasmMethodDesc};
    use fidius_host::PluginHost;

    if tcp_echo_component().is_none() {
        eprintln!("SKIP builder_path_hostname_dial_authorized_by_name: fixture not built");
        return;
    }
    static TCP_METHODS: [WasmMethodDesc; 1] = [WasmMethodDesc {
        name: "connect-and-echo",
        wire_raw: false,
        streaming: false,
    }];
    static TCP_ECHO: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
        interface_name: "tcp-echo",
        interface_export: IFACE,
        interface_hash: 0x7CCB_0033_0000_0001,
        methods: &TCP_METHODS,
    };

    let payload = b"builder".to_vec();
    let (port, server) = mock_tcp_echo_once(payload.len());

    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("tcp-echo-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tcp_echo_guest.wasm"),
        tcp_echo_component().unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"tcp-echo-pkg\"\nversion = \"0.1.0\"\ninterface = \"tcp-echo\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"tcp_echo_guest.wasm\"\ncapabilities = [\"tcp\"]\n",
    )
    .unwrap();

    let policy = Arc::new(NameAllowList::new(&["localhost"]));
    let seen = policy.seen.clone();
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .egress_policy(policy)
        .build()
        .unwrap();
    let handle = host
        .load_wasm("tcp-echo-pkg", &TCP_ECHO)
        .expect("load_wasm");
    let echoed: Vec<u8> = handle
        .call_method(
            CONNECT_AND_ECHO,
            &(format!("localhost:{port}"), payload.clone()),
        )
        .expect("echo");
    server.join().unwrap();
    assert_eq!(
        echoed, payload,
        "builder-path hostname dial must round-trip"
    );
    assert!(
        seen.lock()
            .unwrap()
            .iter()
            .any(|(host, _)| host.as_deref() == Some("localhost")),
        "the builder path must exercise the shadow (pinned name seen)"
    );
}
