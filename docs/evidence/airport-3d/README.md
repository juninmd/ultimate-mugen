# Airport 3D stage evidence

- `scene-960x540.jpg`, `scene-landing-taxi.jpg`: `cargo run --release -p ume-model3d --example airport -- assets/model3d/airport.glb out.png 960 540 2 <seconds>` at 7.5 s and 14 s (landing jet, taxiing jet).
- `fight-*.jpg`: headless Chromium (SwiftShader) running the `web` profile build: `node scripts/web-airport-smoke.js web/dist` (stage `cenario=21`, Ichigo 3D vs Kaito).
- Native cost per rendered frame (480x270, 1 sample, 5.1k triangles): pose+project 0.8 ms, raster 2.0 ms, resolve 1.0 ms; rendered every 2nd tick.
- wasm size vs master (`web` profile, after wasm-bindgen): 22,595,537 -> 22,738,828 bytes (+143,291, +0.63%); gzip 13,299,590 -> 13,345,063 (+45,473, +0.34%).
