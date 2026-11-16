## What and why

## Checks

- [ ] Title is `type(scope)?: description` (`feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`)
- [ ] `node scripts/file-size.mjs` and `node scripts/crate-boundaries.mjs` pass
- [ ] `cargo test --workspace --release` passes
- [ ] Wiki updated (and its `pt-BR` mirror) when a format, controller, trigger, flag or folder layout changed
- [ ] ADR added in `docs/wiki/adr/` when a boundary, dependency or data format changed
- [ ] Browser smoke test run for gameplay, rendering, loading or input changes

## Evidence

Screenshots or GIFs, before and after, for visible changes.
