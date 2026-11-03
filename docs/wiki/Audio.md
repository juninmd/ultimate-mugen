# Audio

> 🇧🇷 Sons de personagens (.snd), música de cenário com loop e posição estéreo (pan). Formatos: WAV, OGG, MP3 e FLAC.

## Formats

Character sounds come from the `.snd` file. Music files can be WAV, OGG Vorbis, MP3 or FLAC. Everything is decoded by `symphonia` (pure Rust), so the same code runs on desktop, web and Android.

## Character sounds

- Sounds play from where the fighter stands: a fighter on the left of the screen is heard on the left speaker.
- `channel` works as in M.U.G.E.N: a new sound on the same channel of the same fighter replaces the old one; `lowpriority = 1` keeps the old one if it is still playing.
- `PlaySnd` reads `pan`, `abspan`, `loop`, `freqmul`, `volumescale` and `channel`. `StopSnd` stops one channel, or every sound of the character with `channel = -1`.
- Looping sounds keep playing until the channel is stopped or the fight ends.
- `SndPan` is accepted but does not move a sound that is already playing yet; the next sound uses the new position.
- A sound that fails to decode still plays, without panning.

## Stage music

```
[Music]
bgmusic = sound/stage.ogg
bgmloopstart = 88200
bgmloopend = 1764000
```

`bgmloopstart` and `bgmloopend` are in samples. The song plays once up to `bgmloopend`, then repeats the part between the two points. With both at 0 the whole file loops. If the file is missing or cannot be decoded, the built-in music for the stage plays instead.
