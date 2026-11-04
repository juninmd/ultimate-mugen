# Stress tests and resilience

## What is covered

| Area | Test | Command |
|---|---|---|
| Format parsers (SFF, AIR, INI/DEF/CNS/CMD, SND, ACT, PCX, PNG, zip, stage) | Deterministic mutation fuzzer seeded from the bundled kfm zip. Fails on any panic or any input slower than 1.5 s. | `UME_FUZZ_ITERS=4000 cargo test -p ume-formats --release --test fuzz` |
| Spawner spam (Explod, Projectile, Helper every tick) | Tick cost stays bounded | `cargo test -p ume-engine --release --test stress -- --nocapture` |
| Hot paths | Built-in bench for SFF decode, AIR parse, CNS compile, char load, match tick with 2, 6 and 10 fighters | `cargo run -p ume-engine --release --example hotpath_bench` |
| Rollback under latency | `crates/netplay/tests/rollback.rs` (in-memory sockets) | `cargo test -p ume-netplay` |

## Findings and fixes

- A corrupt SFF header could claim a 65535x65535 sprite: 3 to 48 s and gigabytes of RAM per file. Sprites and PNGs above 16M pixels (`MAX_SPRITE_PIXELS`) are now treated as empty.
- Explods and projectiles had no cap. With Helpers spawning every tick, 240 ticks took 72 s and the cost grew quadratically (projectile pairs). Both are now capped at 256; extra spawns are dropped, as in MUGEN. 2000 ticks with 61 fighters now take 1.2 s (about 0.6 ms per tick).

## Hot path numbers (kfm720, release, cloud CI-class CPU)

| Path | Time |
|---|---|
| SFF decode | 1.1 ms |
| AIR parse | 1.0 ms |
| CNS parse and compile | 3.2 ms |
| Char load without images | 7.6 to 10 ms |
| Match tick, 2 fighters | 5 us |
| Match tick, 6 fighters | 15 us |
| Match tick, 10 fighters | 39 us |

The simulation is far below the 16.6 ms frame budget, so loading and rendering are the places to optimize, not the tick.

## Comparison and ideas not done yet

- GGRS provides `SyncTestSession` (resimulate every frame and compare checksums) and desync detection; `bevy_ggrs` documents a desync debugging workflow. Running a sync test in CI over a scripted fight would catch non-determinism early.
- Ikemen GO added rollback netcode through GGPO-style save/load of full game state; the cost driver there is state size, so keeping fighters free of large per-tick allocations (already tested in `tick_alloc`) matters.
- Not covered yet: packet loss and jitter in the netplay harness (only latency), GPU device-lost recovery, low-memory behavior, a CNS/expression fuzzer in the engine crate, and a roster stress that loads many different characters.
