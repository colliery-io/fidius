---
title: "fidius-host::executor::body_tee"
description: "Tee/replay bodies for the egress response hook (FIDIUS-I-0035)."
editUrl: false
sidebar:
  order: 32
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Tee/replay bodies for the egress response hook (FIDIUS-I-0035).

To honor [`ResponseDirective::RetryOnce`](../../../../rust/fidius-host/executor/wasm/#enum.ResponseDirective),
the dispatch path must be able to send the guest's request body a second
time — but an outgoing body streams to the wire once and is gone. The
`TeeBody` wrapper solves this without changing wire behavior: frames pass
through to the inner body untouched while their bytes are copied into a
side buffer, capped at `REPLAY_CAP`. Whether a retry is possible is
decided *at response time* from the capture's terminal state (a bodiless
GET has no `Content-Length`, so a pre-dispatch size check would exclude the
most common retryable requests).
