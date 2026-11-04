# Native animation coverage

This delivery is a partial art and loader update. It does not complete the native roster animation set.

## Included in this wave

- Dedicated `portrait.png` images for all sixteen native fighters. The roster uses these images for character tiles and the HUD prefers them for portraits.
- 156 newly installed motion frames across Kaito and selected fighters: idle, walk, run, crouch, jump, guard, damage, recovery and attacks. Coverage varies by fighter; these counts describe installed PNGs, not complete animation sequences.
- Native sprite loading honors each fighter’s positive declared height. Optional per-frame `refHeight`, `axisX` and `axisY` metadata keeps generated frames aligned to the standing body scale and floor.
- Procedural effects no longer fall back to a fighter’s idle art. The native sprite audit reports body-art fallback and missing-frame cases.

## Known gaps

Many animations still resolve to shared poses or idle, and frame counts are uneven across the roster. Dedicated forward/backward walk, run, damage, transformation, jump, defense, recovery, attack and victory sequences are not complete for every fighter. Some inline animation poses have no matching PNG and continue to use the compatibility fallback. Portrait presence does not by itself prove visual quality in every roster tile or HUD state.

The `native-coverage` runtime probe captures a fixed-camera height comparison and individual frames from idle, walk, backward walk, run, damage and transformation animations. It selects frames directly; it does not prove natural gameplay triggers or smooth animation. A missing required animation now fails the probe with the fighter and animation ID.

Use the `native_sprite_audit` example to compare resolved source files and unique pixel images. Use `scripts/audit_native_sprites.py` for PNG dimensions and alpha inventory. Neither inventory is a substitute for inspecting the art or playing the animations.

Per-frame geometry metadata is optional. `refHeight` is the source-pixel height representing the fighter’s nominal standing scale; axes are source-pixel anchors. The loader applies art resolution after these values.
