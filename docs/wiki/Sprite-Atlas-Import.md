# Sprite Atlas Import

The CLI limits atlas files to 64 MiB, image dimensions to 16,384 pixels per side and 8 million total pixels, and output canvases to 2,048 pixels to keep local imports bounded.

The `import_sprite_atlas` example validates and imports a transparent 3-column by 2-row atlas containing six poses. It leaves the source atlas untouched and exports one standalone PNG per requested pose name.

```powershell
cargo run -p ume-engine --example import_sprite_atlas -- --atlas .\poses.png --out assets\chars\sprites\fighter --cols 3 --rows 2 --names 01_idle,02_walk,03_attack,04_crouch,05_hit,06_fall --canvas 400 --validate-only
cargo run -p ume-engine --example import_sprite_atlas -- --atlas .\poses.png --out assets\chars\sprites\fighter --cols 3 --rows 2 --names 01_idle,02_walk,03_attack,04_crouch,05_hit,06_fall --canvas 400
```

`--canvas` defaults to 400 pixels. Every output has that square size; opaque pixels are uniformly scaled with nearest-neighbor sampling, centered horizontally by their cell, and aligned by their alpha bounds to the common floor at y=392. Transparent cell padding does not affect placement. The default standing-pose reference is 80% of the source cell height. Use `--reference-height` to provide the source alpha height of the standing pose when a character family requires explicit calibration. The same reference is applied to all six poses, so crouching and falling poses retain the standing scale. Only transformed opaque bounds must fit the canvas.

The importer requires actual RGBA alpha, six non-empty cells, and a meaningful connected alpha component in every pose. Alpha below 16 is discarded in exported frames and ignored for border and component validation. It rejects poses whose alpha reaches the cell edge and sheets with excessive isolated alpha components; inspect and repair the atlas instead of accepting clipped poses or noise. These checks are diagnostics, not visual quality approval. `--validate-only` also normalizes all six poses and checks that their opaque bounds fit before reporting success. All six PNGs are encoded before any output file is written.

`meta.json` is merged without discarding unrelated fields. Each imported `frames[name]` entry records the 320-pixel reference and normalized axes (`axisX=200`, `axisY=392` by default). The importer preserves the top-level `refHeight` exactly as found, including leaving it absent, so older frames keep their image-height fallback. Existing frame entries with the same names are replaced. Existing PNGs are protected unless `--overwrite` is supplied. `--validate-only` performs validation and collision checks without writing files.

Validation evidence: a synthetic padded 300x200 RGBA atlas passed validation and produced six 400x400 PNGs. An independent PNG decode confirmed the last opaque row was 391, meeting the floor edge at axisY=392, with no exported alpha below 16. A fixture with alpha crossing a cell border and an oversized normalization request were rejected; validate-only created no output directory. The generated Bruno walk-cycle atlas also passed validation and import with `--reference-height 468`; its output bbox was x=115..271, y=75..391 with no alpha below 16. The source atlas SHA-256 remained `56AED5E92D7F55A80C0CD288DA983E033DA17EEB6F67D8A95D099F0147ACE1D7`.

The PNG writer converts straight RGBA channels to tiny-skia's premultiplied pixel representation and encodes the result. Atlas bytes are never modified.
