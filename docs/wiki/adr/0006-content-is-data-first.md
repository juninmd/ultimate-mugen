# ADR 0006: Content is data first, with a documented extension table

- Status: Accepted
- Date: 2026-11-15

## Context

Fighters, stages, routes, story objects, looks and props already ship as data files, but the list of them lived only in the heads of contributors. Outside contributors need to know which changes need no Rust and where each file is documented.

## Decision

New content kinds are data files under `assets/`, embedded by `build.rs` when they ship with the game, and each one gets a row in [Extension points](../Extension-Points.md) with its English and `pt-BR` documentation and a test that loads the shipped files. Code changes in `crates/engine` or `crates/formats` are reserved for behavior or formats the data cannot express.

A single content-pack manifest that lets the marketplace declare several kinds of content in one repository is a possible next step; it needs its own ADR before any loader change, because it touches the registry and the marketplace.

## Consequences

Contributors can add a fighter, stage or route without reading the engine. Every new data kind carries a documentation and test cost up front. The manifest idea stays out of the code until a concrete marketplace need justifies it.
