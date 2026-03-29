---
id: usability-and-reliability
level: initiative
title: "Usability and Reliability — Architecture Review Phase 2"
short_code: "FIDIUS-I-0008"
created_at: 2026-03-29T17:17:02.948844+00:00
updated_at: 2026-03-29T17:54:13.593133+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: usability-and-reliability
---

# Usability and Reliability — Architecture Review Phase 2

## Context

Phase 1 (FIDIUS-I-0007) fixed all Critical safety findings. This initiative addresses Major findings focused on usability and reliability: human-readable error messages, observability, API cleanup, error path testing, and code consolidation. See `review/10-recommendations.md` R-07 through R-18 (excluding R-13, R-15, R-16 already done in Phase 1).

## Goals

- R-07: Human-readable error messages for wire format/buffer strategy mismatches
- R-08: Fix CLI scaffolding dependencies
- R-09: Add tracing observability infrastructure
- R-10: Make PluginHandle::new() crate-private, hide raw pointers
- R-11: build_package returns error when cdylib not found
- R-12: Add CallError::UnknownStatus variant
- R-14: Preserve panic messages across FFI boundary
- R-17: Consolidate signing utility functions
- R-18: Add test coverage for error paths

## Implementation Plan

All independent except R-07 depends on R-05 (done in Phase 1) and R-18 depends on R-02/R-14 (done/in this phase).