---
id: 001-capability-gated-path-scoped
level: adr
title: "Capability-gated, path-scoped filesystem for WASM guests (fs:ro / fs:rw preopens)"
number: 1
short_code: "FIDIUS-A-0008"
created_at: 2026-06-20T14:51:59.461303+00:00
updated_at: 2026-06-20T15:06:04.543844+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Capability-gated, path-scoped filesystem for WASM guests (fs:ro / fs:rw preopens)

## Context **[REQUIRED]**

The WASM sandbox started **deny-all** with filesystem explicitly *never* grantable
in v1 (no preopens, ever — old `build_wasi_ctx` comment + `KNOWN_CAPABILITIES`).
Real connectors need local files: read a config/cert, write an export/cache, drain a
spool directory. The original stance forced those workloads off WASM entirely. We are
**reversing the "never" decision** and adding filesystem as a first-class, *gated*
capability — consistent with how egress (`http`, FIDIUS-I-0027) and scoped env
(`env:VAR`, FIDIUS-T-0142) already work: deny-by-default, grant narrowly.

## Decision **[REQUIRED]**

Filesystem is grantable, **path-scoped**, via two capability forms:

- `fs:ro:<path>` — preopen `<path>` **read-only** (`DirPerms::READ`, `FilePerms::READ`).
- `fs:rw:<path>` — preopen `<path>` **read-write** (`DirPerms::all`, `FilePerms::all`).

Mechanics (`crates/fidius-host/src/executor/wasm.rs`):
1. The host preopens the granted host directory at the **same guest path**
   (`WasiCtxBuilder::preopened_dir`). WASI's capability model scopes the guest to the
   preopen — no path-traversal escape, no ambient FS.
2. **Bare `fs` / `filesystem` (whole-FS) is rejected at load**, exactly like bare `env`
   — a coarse grant is a footgun.
3. `fs:ro:` / `fs:rw:` with an **empty path** is rejected at load.
4. **Deny-all default is unchanged** — no capability ⇒ no FS.
5. A **non-existent** granted dir is skipped silently; the guest's `open()` then fails
   with a normal WASI error (no host crash).
6. fidius ships **mechanism, not policy**: the host decides *which* paths to grant
   (just as it supplies the `EgressPolicy` for http). There is no built-in allow-list.

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|--------|------|------|------|------|
| **Path-scoped `fs:ro`/`fs:rw` preopens (chosen)** | WASI-native (preopen = capability); ro/rw explicit; mirrors `env:`/`http` gating; no traversal escape | Two new capability forms to document; host must choose paths | Low | S |
| **Keep filesystem ungrantable (status quo)** | Smallest sandbox surface | Pushes any file-touching connector off WASM | n/a | none |
| **Single `fs:<path>` (implicit rw)** | Fewer forms | Silent write grant; read-only intent inexpressible | Med | S |
| **Whole-FS `fs` (inherit host root)** | Trivial | Ambient authority — every secret/file; defeats the sandbox | High | none |

## Rationale **[REQUIRED]**

Preopens *are* WASI's filesystem capability primitive, so a path-scoped grant is the
idiomatic, escape-proof unit — the guest sees exactly the granted dir and nothing
above it. Splitting `ro`/`rw` makes least-privilege the default expression (a reader
connector grants `fs:ro:` and physically cannot write). Rejecting bare `fs` keeps the
dangerous coarse grant from being a one-word typo. This is the same shape as the
egress and scoped-env decisions, so the sandbox story stays uniform: **deny-all, grant
narrowly, host owns policy.**

## Consequences **[REQUIRED]**

### Positive
- File-touching connectors (config/cert readers, exporters, spoolers) can run sandboxed.
- Least-privilege by construction: ro vs rw is explicit; scope is one directory.
- Uniform with `http`/`env:` — one mental model for the whole capability allow-list.

### Negative
- Reverses a documented "never" stance — docs/explanations referencing "no filesystem"
  must be updated (wasm-capabilities.md).
- The host now has a path-granting responsibility (mechanism only; choosing safe paths
  is the embedder's job, like egress).

### Neutral
- Symlinks/hardlinks inside a preopen follow WASI/`cap-std` semantics; no extra fidius
  policy layered on top.
- `clocks`/`random`/deny-all baseline unchanged.

## Review Schedule

### Review Triggers
- Demand for finer-grained grants (single-file preopen, glob scopes, per-extension).
- A WASI filesystem semantics change (preview3 / `wasi:filesystem` major bump).