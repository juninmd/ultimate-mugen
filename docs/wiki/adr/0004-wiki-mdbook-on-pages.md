# ADR 0004: Wiki rendered with mdBook under `/wiki/` on GitHub Pages

- Status: Accepted
- Date: 2026-11-15

## Context

The wiki is Markdown in `docs/wiki` (English) and `docs/wiki/pt-BR`, readable only through the repository. Outside contributors need a browsable, searchable site, and the web game already owns the root of the Pages site.

## Decision

`scripts/wiki-book.mjs` stages `docs/wiki` and the contributing guides into one mdBook source with a generated `SUMMARY.md`. `scripts/build-wiki.sh` builds it, and `pages.yml` publishes it to `/wiki/` next to the game. The Markdown files stay the single source; no page is duplicated.

## Consequences

Sections are declared in `scripts/wiki-book.mjs`; a page not listed there still appears under Reference, so nothing is lost. Links that point outside `docs/` (for example to `scripts/`) are plain repository paths and do not resolve on the site. mdBook is a single Rust binary with no Node toolchain.
