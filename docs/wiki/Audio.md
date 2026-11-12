# Audio

> 🇧🇷 Sons de personagens (.snd), música de cenário com loop e posição estéreo (pan). Formatos: WAV, OGG, MP3 e FLAC.

## Formats

Character sounds come from the `.snd` file. Music files can be WAV, OGG Vorbis, MP3 or FLAC. Everything is decoded by `symphonia` (pure Rust), so the same code runs on desktop, web and Android.

## Character sounds

- Sounds play from where the fighter stands: a fighter on the left of the screen is heard on the left speaker.
- `channel` works as in M.U.G.E.N: a new sound on the same channel of the same fighter replaces the old one; `lowpriority = 1` keeps the old one if it is still playing.
- `PlaySnd` reads `pan`, `abspan`, `loop`, `freqmul`, `volumescale`, `channel` and `lowpriority`. `StopSnd` stops one channel, or every sound of the character with `channel = -1`.
- Looping sounds keep playing until the channel is stopped or the fight ends.
- `SndPan` moves sounds that are already playing on that channel (or every channel and looping sound of the character with `channel = -1`); playback continues from the same point.
- A sound that fails to decode still plays, without panning.

## Move voices

Every special, super, finisher and awakening of the native roster is announced by its fighter when the move starts, anime style (`Hellflame Orb!`). Each fighter has its own voice and the Settings > Audio row **Move voices** scales or mutes them (0% turns them off).

- Clips live in `assets/voices/voices.bank`, embedded in the binary (about 0.9 MB, 16 kHz Vorbis, one stream per fighter). A move's clip is found by the move id; `ex_` versions reuse the base move's clip and the `npc_*` fighters borrow their parent's voice.
- `MoveInfo.state` ties the move to its state, so the voice fires on the state change and follows cancels, combos and form moves.
- Clips are generated offline with [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M) (Apache-2.0, model and voices) through `kokoro-onnx` (MIT): `python3 tools/voices/generate.py [fighter...]` with `KOKORO_MODEL` / `KOKORO_VOICES` pointing at `kokoro-v1.0.onnx` / `voices-v1.0.bin` from the kokoro-onnx releases, plus `ffmpeg` with libvorbis. The cast table (voice, speed, effect per fighter) is at the top of that script. Nothing is synthesised at run time.
- To add a fighter, add it to the cast table and rerun the script; moves without a clip stay silent.

## Built-in music per stage

Every stage without a `[Music]` entry plays one of the built-in synthesized themes, picked by the stage (`Stage.music`), so the dojo, neon city, bamboo grove, volcano and the other arenas each have their own track. Themes are rendered once at startup on a background thread (`music-synth`) and loop until the fight ends.

## Hit sounds and impact ducking

- Hits come in tiers: `hitlight`, `hit` and `hitheavy`, each with three detuned variants, so repeated hits never sound identical. Counter hits, supers, transformations and the KO have their own sounds.
- Impact ducking dips the music for a moment on a hit (`hit` 0.8, `hitheavy` 0.55, `counter` 0.45, `super` 0.35, `ko` 0.2 of the current level) and recovers smoothly.
- A crowd cheer plays on the KO.
- `AssertSpecial` `nomusic` silences the music and keeps the effects.

## Stage music (M.U.G.E.N stages)

```
[Music]
bgmusic = sound/stage.ogg
bgmloopstart = 88200
bgmloopend = 1764000
```

`bgmloopstart` and `bgmloopend` are in samples. The song plays once up to `bgmloopend`, then repeats the part between the two points. With both at 0 the whole file loops. If the file is missing or cannot be decoded, the built-in music for the stage plays instead.
