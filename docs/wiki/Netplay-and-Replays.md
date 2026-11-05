# Netplay and Replays

**English** · [Português (BR)](pt-BR/Replays-Rede.md)

> 🇧🇷 O netplay usa rollback (GGRS): cada jogador roda a luta localmente, prevê o controle do adversário e corrige quando o controle real chega. Replays guardam só os controles de cada frame.

## Playing online (desktop)

Both players use the same build and the same characters and stage files. Each one opens UDP port 7000 (player 1) or 7001 (player 2), or the port given with `--port`.

```
ultimate-mugen --netplay 1 --peer <friend-ip>:7001 --chars kfm,kfm --stage dojo
ultimate-mugen --netplay 2 --peer <friend-ip>:7000 --chars kfm,kfm --stage dojo
```

| Flag | Meaning |
|---|---|
| `--netplay 1|2` | your side |
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

## Spectators and netplay data in replays

A third connection type observes without contributing input:

```
ultimate-mugen --spectate --peer <host-ip>:7000 [--port 7002] [--chars a,b --stage id]
```

It builds a GGRS `SpectatorSession` (`PlayerType::Spectator`), renders the confirmed inputs from the host and never
sends local input; the overlay shows `SPECTATOR · ping · frames behind`.

Replays recorded online store the session configuration (`netplay`: side, peer, port, delay, rollback window, player
count, spectator flag) next to the inputs, so a desync can be reproduced with the same settings. When GGRS reports a
desync, the game writes `replays/desync-<unix>.json` with that config, the frame, `Match::checksum()` and the last 120
input frames. A desync report is a diagnostic, not a replay: the Replays menu never lists it (neither as a fight nor as
an unreadable file), so it does not look like a corrupted replay after a desync.

## Rules for deterministic content

- Never read real time (`Instant`, `now`) or random numbers from the operating system (`rand::random`, `rand::thread_rng`) during a tick; use the match's seeded PRNG.
- Floating-point calculations must run in deterministic mode (all target platforms use IEEE 754 with standard rounding).
- All collections whose iteration affects the simulation must have deterministic ordering (use `Vec` or sorted keys, never direct iteration over a raw `HashSet`/`HashMap`).

## Binary format (`ume_netplay::Replay`)

A raw stream of recorded inputs:

| Field | Type |
|---|---|
| magic | `UMRP` |
| version | `u32` (1) |
| players | `u32` |
| metadata count, then key/value strings | `u32` length + UTF-8 |
| frame count | `u32` |
| runs | `u32` repeat count + one `u32` input per player |

Equal consecutive frames are stored once, so held buttons cost almost nothing.

The reader rejects incomplete or overlong frame runs, trailing bytes, files larger than 129 MiB, decoded input streams larger than 64 MiB and more than 16,384 metadata entries. The writer returns an error when player counts, frame data or metadata exceed these limits. The encoded size allows a worst-case run per frame while decoded frames remain capped at 64 MiB.

```rust
let mut rec = Replay::new(2);
rec.meta.push(("p1".into(), "kfm".into()));
rec.push(&[p1_bits, p2_bits]);
std::fs::write("fight.umr", rec.to_bytes().expect("valid replay size"))?;
```

## Saved replays in the game (`replays/`)

Every finished fight is archived automatically to `replays/` beside the executable as a small JSON file
(`<label>-<unix-time>.umr.json`). The menu entry **Replays** lists them newest first.

| Field | Meaning |
|---|---|
| `format` / `version` | `ultimate-mugen-replay` / `1`; a different version is refused with a clear message |
| `label`, `saved_at` | name shown in the browser and the Unix timestamp (sorted by it) |
| `mode` | `Arcade`, `Cpu`, `Versus`, `Training`, `Watch`, `TeamSeq`, `TeamSimul`, `TeamTag` |
| `chars`, `stage`, `palettes` | fighter ids, stage id and the palette of each side |
| `seed` | initial RNG seed of the match (replays restore it before the first tick) |
| `teams`, `simul`, `tag` | team line-ups by fighter id (team modes) |
| `rules` | rounds, round time, life %, single round, pushbox, shared team life, game speed |
| `inputs` | run-length encoded input stream: `slots`, then `count` + one input per slot per run |

Replay decoding bounds files to 16 MiB, 500,000 runs and 2,000,000 total frames to keep malformed or corrupt streams from allocating unbounded memory. Desync reports (`desync-*.json`) in `replays/` are preserved without being listed as corrupt replays.

Replaying feeds the recorded inputs through `Pilot::Replay`; the simulation is deterministic, so the fight is
rebuilt exactly. A round-trip test records 1200 ticks, reloads the file and compares the final state checksum
(`cargo test -p ume-engine --test replay_roundtrip`).

Browser keys:

| Key | Action |
|---|---|
| ↑ / ↓, Enter | pick and play |
| R | rename (Enter confirms, Esc cancels) |
| Delete (twice) | delete the selected file |
| S | toggle sorting between date and name |
| Esc | back to the menu |

Playback keys:

| Key | Action |
|---|---|
| Space / Start | pause and resume |
| 1 / 2 / 4 | playback speed 1x, 2x, 4x |
| `.` or → while paused | advance one frame |
| Esc | leave the replay and return to the browser |

The folder keeps the newest 64 files; older ones are pruned when a new replay is saved. Files that cannot be read
(for example, recorded by another build) are listed with the reason instead of failing silently.
