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
- `replay_file_of_1200_ticks_reloads_to_the_same_checksum`: records 1200 ticks with the header, reloads the bytes and compares the final checksum.
- `replay_from_another_build_is_rejected_with_a_clear_message`: the build stamp check.

## Replays on disk (desktop and Android)

Every finished fight is saved to `replays/` next to the executable (Android: the app's data folder; override with `UME_REPLAY_DIR`). Files are named `<date>-<time>-<p1>-vs-<p2>.umr`. Online fights are not saved yet.

Open **Replays** in the main menu to browse them:

| Key | Action |
|---|---|
| Up / Down | choose a replay |
| Enter | play it |
| R | rename (type the new name, Enter confirms, Esc cancels) |
| Delete | delete (asks for confirmation) |
| Tab | change the order: newest, oldest, name |
| Esc | back |

Playback controls during a replay:

| Key | Action |
|---|---|
| 1 / 2 / 4 | speed 1x / 2x / 4x (Up / Down step through the speeds) |
| Space | pause and resume |
| `.` or Right | advance one frame (pauses first) |
| Esc or P | pause menu |

The on-screen bar has the same buttons for mouse and touch. When a replay reaches its last recorded tick, the match state is compared with the checksum stored in the file and the bar shows `VERIFICADO` or `DIVERGIU`.

A replay stores the engine build, mode, stage, the character ids of each team, the palettes, the RNG seed, the final checksum and the inputs of every tick. Replays only play on the build that recorded them: a replay from another build is listed in red with the message `This replay was recorded with build X; this is build Y.`, and it is not started. A replay that names a character or stage that is not installed shows `Personagem não instalado` or `Cenário não instalado`.

Replay metadata keys: `build`, `mode`, `stage`, `team1`, `team2` (ids separated by the unit separator `0x1f`), `palettes`, `simul`, `seed`, `created` (Unix seconds), `checksum` (32 hex digits). The web build does not save replays to disk; the last fight can still be replayed from the results screen.

## Replay file format (`.umr`)

| Field | Type |
|---|---|
| magic | `UMRP` |
| version | `u32` (1) |
| players | `u32` |
| metadata count, then key/value strings | `u32` length + UTF-8 |
| frame count | `u32` |
| runs | `u32` repeat count + one `u32` input per player |

Equal consecutive frames are stored once, so held buttons cost almost nothing.

The `ume-netplay` crate builds without GGRS for the web (`default-features = false`); the `rollback` feature (default) adds the rollback session.

```rust
let mut rec = Replay::with_header(2, &header);
rec.push(&[p1_bits, p2_bits]);
std::fs::write("fight.umr", rec.to_bytes())?;
```
