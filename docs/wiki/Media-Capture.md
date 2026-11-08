# Media capture

Screenshots and GIFs for the README come from the real web build running in headless Chromium, so what the docs show is what the game renders.

## Steps

1. `rustup target add wasm32-unknown-unknown` and `cargo install wasm-bindgen-cli --locked --version <Cargo.lock version>`.
2. `./scripts/build-web.sh` writes `web/dist`.
3. `node scripts/web-smoke.js web/dist` must pass before any capture.
4. `CAPTURE_OUT=<dir> node scripts/web-capture.js <plan.json>` plays a key plan and saves the results.

## Plan file

```json
{
  "video": "fight.webm",
  "steps": [
    { "key": "Enter", "wait": 900, "shot": "menu" },
    { "key": "ArrowDown" },
    { "hold": "ArrowRight", "ms": 600 },
    { "wait": 3000, "shot": "fight" }
  ]
}
```

- `key` taps a key for `hold` frames (default 2) and waits `wait` ms (default 900).
- `hold` with `ms` keeps a key down for that long.
- `shot` saves `<shot>.png` after the step; `video` records the whole session as WebM.

Convert a WebM to a small GIF with `ffmpeg -i fight.webm -vf "fps=12,scale=640:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse" fight.gif`. Software rendering is slower than a GPU, so the GIFs show the game at a reduced frame rate.
