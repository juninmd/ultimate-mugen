# Contributing to Ultimate MUGEN

**English** · [Português (BR)](CONTRIBUTING.pt-BR.md)

Thanks for helping. This page gets you from a fresh clone to a merged pull request. The full documentation is the wiki at https://juninmd.github.io/ultimate-mugen/wiki/.

## Where to start

- Pick an issue labelled `good first issue`, or open one describing what you want to change before writing large code.
- Read the [architecture overview](https://juninmd.github.io/ultimate-mugen/wiki/Architecture.html) and the [file layout rules](https://juninmd.github.io/ultimate-mugen/wiki/File-Layout.html) once; they explain where code belongs.
- Characters, stages and data changes need no Rust: see [Make a character](https://juninmd.github.io/ultimate-mugen/wiki/Make-a-Character.html) and [Make a stage](https://juninmd.github.io/ultimate-mugen/wiki/Make-a-Stage.html).

## Build and run

```bash
git clone https://github.com/juninmd/ultimate-mugen
cd ultimate-mugen
cargo run --release
```

On Linux install `libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev pkg-config` first. The browser build is `./scripts/build-web.sh`, served from `web/dist`.

## Checks to run before a pull request

| Check | Command |
|---|---|
| Rust tests | `cargo test --workspace --release` |
| File size ceiling (250 lines) | `node scripts/file-size.mjs` |
| Crate boundaries | `node scripts/crate-boundaries.mjs` |
| Script tests | `node --test scripts/*.test.mjs` |
| Wiki book | `./scripts/build-wiki.sh` (needs `mdbook`) |
| Browser smoke | `./scripts/build-web.sh` then `node scripts/web-smoke.js web/dist` (Playwright) |

Do not run `cargo fmt --all`: the tree is not rustfmt-clean and it rewrites unrelated files. Format only files you created with `rustfmt --edition 2021 <file>`.

## Rules

- No comments in code; names must explain intent. Explanations go to the wiki.
- English is the default language for code, commits, issues and docs. Portuguese lives in `*.pt-BR.md` files and `docs/wiki/pt-BR/`; keep both in sync.
- A source file stays under 250 lines. Files already over the ceiling may not grow; see [File layout](https://juninmd.github.io/ultimate-mugen/wiki/File-Layout.html).
- `crates/engine` is deterministic: no clocks, threads, randomness or allocation inside `Match::tick`.
- Respect the crate boundaries in [Architecture](https://juninmd.github.io/ultimate-mugen/wiki/Architecture.html); `scripts/crate-boundaries.mjs` enforces them in CI.
- Update `docs/wiki/` whenever a format, controller, trigger, flag or folder layout changes.
- Significant design changes get an ADR in `docs/wiki/adr/`.

## Pull requests

- Title must be `type(scope)?: description` with type in `feat|fix|perf|refactor|docs|test|build|ci|chore|style|revert`; `commit-lint` rejects anything else.
- Keep a PR to one concern. Put new logic in a new file and wire it with one line in shared files, so parallel PRs do not collide.
- Include screenshots or GIFs (before and after) for anything visible, and a browser smoke result for gameplay, rendering, loading or input changes.
- Releases are automatic from `master` commit types; `feat` is minor, `fix` and `perf` are patch.

## Acknowledgments

Ultimate MUGEN is inspired by Elecbyte's M.U.G.E.N and Ikemen GO. Please keep that credit respectful in docs and discussions.
