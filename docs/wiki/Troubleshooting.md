# Troubleshooting

| Symptom | Likely cause | What to do |
|---|---|---|
| Character missing from select | `.def` not in `mugen/chars/<name>/` or a parse error | Run `mugen_check` on the folder and read the error |
| Invisible or square sprites | Sprite missing from the `.sff` or wrong palette | `sprite_audit` lists the broken references |
| Fighter floats or sinks | A state that never returns to the ground | `hover_check` with `TRACE=1` |
| HD character too big or small | `localcoord` missing in the `.def` | Add `localcoord = 1280, 720` to `[Info]` |
| Move has no effect | Controller not implemented yet | See the list in [CNS scripts](CNS-Scripts.md) |
| UMF does not load | Invalid JSON or a missing required field | `cargo run --release` prints the `serde_json` error with line and column |
| Game is slow | High art resolution | Video menu → Quality, or `quality=1` in the `.cfg` |
| Too many flashes | Light sensitivity | `impact_lv=0` in the `.cfg` |
| `mugen/` folder not found | Executable run from another directory | Set `UME_MUGEN_DIR` |

To report a bug, open an issue with the character or stage, the `mugen_check` output and the steps to reproduce.
