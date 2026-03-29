---
id: pre-commit-hook-for-license-header
level: task
title: "Pre-commit hook for license header enforcement"
short_code: "FIDIUS-T-0028"
created_at: 2026-03-29T12:20:41.037500+00:00
updated_at: 2026-03-29T12:45:16.752266+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Pre-commit hook for license header enforcement

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Configure the `pre-commit` framework (`.pre-commit-config.yaml`) to enforce Colliery Apache 2.0 license headers on all `.rs` files. Use the existing `angreal license-header --check` task as the hook command.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `.pre-commit-config.yaml` at repo root with a local hook running `angreal license-header --check`
- [ ] `pre-commit run --all-files` passes on a clean repo
- [ ] `pre-commit run --all-files` fails if a header is missing
- [ ] Contributors can run `angreal license-header` to fix before committing

## Implementation Notes

### Technical Approach

Use `pre-commit` (https://pre-commit.com) with a `local` hook type that runs `angreal license-header --check`. No custom pre-commit repo needed — just wraps the existing angreal task.

## Status Updates

- **2026-03-29**: `.pre-commit-config.yaml` created with local hook wrapping `angreal license-header --check`. `pre-commit run --all-files` passes.