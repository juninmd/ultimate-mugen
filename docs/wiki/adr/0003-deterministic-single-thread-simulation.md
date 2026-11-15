# ADR 0003: Deterministic single-thread simulation for rollback

- Status: Accepted
- Date: 2026-11-03

## Context

Rollback netplay (ggrs) and replays need the same inputs to give the same fight on every machine.

## Decision

`Match::tick` runs single-threaded at a fixed 60 Hz with no clock, no thread and no randomness outside `Match::random`, and allocates nothing per tick. `Match` stays `Clone` with a stable checksum, and new state fields join the checksum.

## Consequences

Netplay, replays and tests share one simulation. Performance work targets loading, rendering and data layout instead of parallelizing the tick. `cargo test -p ume-engine --release --test tick_alloc` guards the allocation rule.
