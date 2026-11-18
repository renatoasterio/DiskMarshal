# 0001 — Layered workspace, one crate per layer

Status: accepted (2026-11-17)

## Context

DiskMarshal deletes files and talks to several macOS APIs. The rules that
decide what may be deleted must be testable without touching a disk, and the
interface must not be able to reach the operating system by accident.

## Decision

Use Clean Architecture with four library crates (`domain`, `application`,
`infrastructure`, `ui`) and thin binaries under `apps/` that act as
composition roots. Dependencies point inward only.

## Consequences

- A wrong dependency fails to compile instead of surviving until review.
- Domain and application code is tested with fakes, in milliseconds.
- More files and some indirection (a port and an adapter per capability).
- `unsafe` can be confined to one crate and forbidden in the others.
