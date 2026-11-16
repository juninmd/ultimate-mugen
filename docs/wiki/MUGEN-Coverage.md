# MUGEN controller and trigger coverage

The coverage guard lives in `crates/engine/tests/mugen_coverage_matrix.rs`. It checks that every listed MUGEN 1.1 controller and trigger has an engine implementation, using the same dispatch probe that reports unsupported names when a character loads.

## Status

| Area | Official names checked | Implemented | Notes |
| --- | --- | --- | --- |
| State controllers | 90 | 90 | Guarded by a test. |
| Triggers (value form) | 92 | 92 | Guarded by a test. |
| Triggers (comparison form, e.g. `StateType = S`, `MoveType = A`, `AnimElem = 3`, `Command = "x"`, `Name = "..."`) | 10 | 10 | Parsed in the expression layer, covered by `mugen11_triggers.rs`. Not in the probe list. |
| Redirections (`enemy`, `enemynear`, `partner`, `helper`, `parent`, `root`) | 6 | 6 | Covered by `enemy_selection.rs` and `helper_parity.rs`. |
| Ikemen GO extensions | many | most | Listed in `CNS-Scripts.md`. |

## Known partial items

- `DestroySelf`: the `recursive` and `removeexplods` parameters are not read. The helper is always destroyed alone.
- `Helper`: `inheritjuggle` and `size.xscale`/`sprpriority` parameters are not read by the spawn path.
- `Explod`: `shadow` and `layerno` are not read.
- `PlaySnd`, `Pause`, `Varset` and `Trans`: parameter-level checks in the probe are inconclusive; these need a manual pass against the MUGEN docs before claiming full parameter coverage.
- `MakeDust`: `spacing` is honoured (dust appears every `spacing` ticks, default 1).

## How to check a new name

1. Add it to the matching list in `mugen_coverage_matrix.rs`.
2. Run `cargo test -p ume-engine --test mugen_coverage_matrix`.
3. If it fails, implement the controller or trigger and document it in `CNS-Scripts.md`.
