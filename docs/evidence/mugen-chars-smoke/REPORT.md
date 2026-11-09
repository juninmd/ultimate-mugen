# Smoke test: zinclee123/mugen_chars

Repository: https://github.com/zinclee123/mugen_chars (commit ab1920e, master, shallow clone, LFS skipped; no LFS stubs found).
Engine commit: e58b010. Tools: `mugen_check --strict --lint` (release) and headless Chromium on `web/dist` (wasm, wasm-bindgen 0.2.130).

## Headless engine (`mugen_check`)

| Definition | Name | Sprites | Load | 60 s CPU fight | Result |
|---|---|---|---|---|---|
| 98-kim/98-Kim.def | 98-Kim | 0 | 58 ms | 0 hits | Patch, no sprites (expected) |
| Hinako/Hinako.def | Hinako | 869 | 40 ms | 38 hits | Loads, fights; unknown triggers/params |
| K'/K'.def | K' | 2208 | 49 ms | 57 hits | Loads, fights; unknown triggers/params |
| Lin/Lin.def | Lin | 1453 | 35 ms | 2 hits | Loads, fights; unknown triggers/params |
| MasterRyu/{Ryu,NormalRyu,EvilRyu,MasterRyu}.def | Ryu | 1510 | ~130-150 ms | 31-34 hits | Loads, fights; unknown triggers/params |
| Rugal2nd/Rugal2nd.def | Rugal2nd | 2062 | 88 ms | 28 hits | Loads, fights; unknown triggers/params |
| kusanagi-XIII/kusanagi-XIII.def | Kusanagi XIII | 2700 | 291 ms | 89 hits | Loads, fights; unknown triggers/params |
| yujiro/yujiro-lee.def | yujiro | 0 | 27 ms | 33 hits | Patch, no sprites (expected); watchdog recovered a stuck state |

- Unsupported controllers: 0 for every character.
- Unknown triggers and parameters: present in all 11 definitions, so `--strict` exits 1 (see `mugen_check-strict.txt`). Examples: `s7202`, `s7212`, `animelem` (98-kim), `pausemovetime` (yujiro).
- Lint: 98-kim reports 42 duplicate parameters, 2250 state-number mismatches and 5 garbage lines.

## Browser (web build)

- `web-smoke.js` (title, menu, select, stage, fight): `WEB SMOKE OK` (`web-smoke-select.png`).
- Import through `#ume-file`: all 8 zips in one batch did not appear within 90 s; imported one zip at a time they all appear. Roster reads **1/26** (`roster-after-import.png`): 15 native + 2 bundled Kung Fu Man + Evil Ryu, Hinako, K', Kusanagi, Lin, Master Ryu, Rugal and two more Ryu entries.
- Not imported as characters: 98-kim and yujiro (no sprites).
- Fights in the browser were not run per character; the per-character fights are the headless 60 s runs above.

## Not checked

- `backup secrets.rar` (20 MB) in the repository was not extracted; it is not a character.
- No stages, music or packs exist in the repository.
