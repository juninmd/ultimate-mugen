# Netplay and Replays

> 🇧🇷 O netplay usa rollback (GGRS): cada jogador roda a luta localmente, prevê o controle do adversário e corrige quando o controle real chega. Replays guardam só os controles de cada frame.

## Playing online (desktop)

Both players use the same build and the same characters and stage files. Each one opens UDP port 7000 (player 1) or 7001 (player 2), or the port given with `--port`.

```
ultimate-mugen --netplay 1 --peer <friend-ip>:7001 --chars kfm,kfm --stage dojo
ultimate-mugen --netplay 2 --peer <friend-ip>:7000 --chars kfm,kfm --stage dojo
```

| Flag | Meaning |
|---|---|
| `--netplay 1\|2` | your side |
| `--peer ip:port` | the other player's address |
| `--port n` | local UDP port (default 7000 / 7001) |
| `--chars a,b` | character ids or names for P1 and P2 (same on both sides) |
| `--stage id` | stage id or name |
| `--delay n` | input delay in frames (default 2) |

The bottom of the screen shows the connection state and ping. Pause is disabled online. Browser and mobile netplay arrive with the WebRTC transport after the wgpu/winit migration.

## How rollback works here

- The whole fight state is `Match`. Saving a frame is `match.clone()`; character data (`CharDef`) is shared through `Arc`, so a snapshot only copies the moving parts.
- `ume_netplay::Rollback` answers the GGRS requests: save, load and advance. Only the newest frame returns presentation events (sounds, hit sparks), so rolled-back frames never replay a sound twice.
- `ume_netplay::checksum` hashes positions, states, life, power, variables, explods and projectiles. GGRS compares it between peers to detect desyncs.
- Inputs are one `u32` per player per frame (the same bits as local play).

## Rules for deterministic content

Rollback only works if the same inputs always produce the same fight.

- Use `Random` from the engine, never the system clock.
- Do not read anything outside the match state from a state controller.
- Characters, stages and the engine build must be the same on both sides.

## Tests

```
cargo test --release -p ume-netplay
UME_NETPLAY_FIGHTS=200 cargo test --release -p ume-netplay --test rollback p2p
```

- `sync_test_detects_no_desync`: GGRS sync-test, rolls back every frame and checks the checksum.
- `p2p_fights_at_100ms_stay_in_sync`: two peers over an in-memory network with 100 ms of latency play 50 fights with desync detection on.
- `replay_round_trip_reproduces_match`: a recorded replay plays back to the same final state.

## Replay files (`.umr`)

| Field | Type |
|---|---|
| magic | `UMRP` |
| version | `u32` (1) |
| players | `u32` |
| metadata count, then key/value strings | `u32` length + UTF-8 |
| frame count | `u32` |
| runs | `u32` repeat count + one `u32` input per player |

Equal consecutive frames are stored once, so held buttons cost almost nothing.

```rust
let mut rec = Replay::new(2);
rec.meta.push(("p1".into(), "kfm".into()));
rec.push(&[p1_bits, p2_bits]);
std::fs::write("fight.umr", rec.to_bytes())?;
```
