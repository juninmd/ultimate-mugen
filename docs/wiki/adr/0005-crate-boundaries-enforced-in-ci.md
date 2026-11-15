# ADR 0005: Crate dependency boundaries enforced in CI

- Status: Accepted
- Date: 2026-11-15

## Context

The crate table in `AGENTS.md` described which crate may depend on which, but nothing stopped a drift such as the simulation importing the GPU.

## Decision

`scripts/crate-boundaries.mjs` lists forbidden runtime and build dependencies per crate and runs in the `release-scripts` CI job. Dev-dependencies are exempt, because tests and examples may combine crates.

## Consequences

A boundary change needs an edit to the script and a new ADR, which makes it a visible decision. The check reads `Cargo.toml` only, so it does not catch an indirect path through another allowed crate.
