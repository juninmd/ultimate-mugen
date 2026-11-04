# Audio

> 🇧🇷 Sons de personagens (.snd) com loop (`loopstart`/`loopend`/`loopcount`, também em `ModifySnd`), `stopongethit`/`stoponchangestate`, música de cenário com loop e posição estéreo (pan). Formatos: WAV, OGG, MP3 e FLAC; músicas tracker (xm/it/mod/s3m) e MIDI não são suportadas e caem na música padrão do cenário com um aviso.

## Formats

Character sounds come from the `.snd` file. Music files can be WAV, OGG Vorbis, MP3 or FLAC. Everything is decoded by `symphonia` (pure Rust), so the same code runs on desktop, web and Android.

## Character sounds

- Sounds play from where the fighter stands: a fighter on the left of the screen is heard on the left speaker.
- `channel` works as in M.U.G.E.N: a new sound on the same channel of the same fighter replaces the old one; `lowpriority = 1` keeps the old one if it is still playing.
- `PlaySnd` reads `pan`, `abspan`, `loop`, `loopstart`, `loopend`, `loopcount`, `freqmul`, `volumescale`, `channel`, `lowpriority`, `stopongethit` and `stoponchangestate`. `StopSnd` stops one channel, or every sound of the character with `channel = -1`.
- Looping sounds keep playing until the channel is stopped or the fight ends.
- Loop points are in samples of the sound. `loop = 1` loops the whole sound forever. With `loopstart` / `loopend` the sound plays up to `loopend`, then restarts at `loopstart`; `loopend = 0` (or past the end) means the end of the sound and `loopstart >= loopend` disables the loop. `loopcount = n` repeats the loop section `n` times and then plays the rest of the sound; `-1` repeats forever and `0` does not loop. A `loopcount` overrides `loop`.
- `ModifySnd` accepts the same `loopstart`, `loopend`, `loopcount` and `loop` for a live channel (`channel = -1` for every channel of the character). Playback continues from the current position under the new loop settings.
- `SndPan` moves sounds that are already playing on that channel (or every channel and looping sound of the character with `channel = -1`); playback continues from the same point.
- A sound that fails to decode still plays, without panning.

## Stage music

```
[Music]
bgmusic = sound/stage.ogg
bgmloopstart = 88200
bgmloopend = 1764000
```

`bgmloopstart` and `bgmloopend` are in samples. The song plays once up to `bgmloopend`, then repeats the part between the two points. With both at 0 the whole file loops. If the file is missing or cannot be decoded, a warning is printed and the built-in music for the stage plays instead, so an unsupported file never mutes the fight.

### Tracker and MIDI music

`xm`, `it`, `mod`, `s3m` and MIDI / SoundFont music are not supported: the decoder (`symphonia`) has no tracker or synthesiser support, and adding one would grow the web build for a rare format. Such files go through the same decode-failure path as any broken file (warning, then the built-in stage music). Convert them to OGG, MP3, FLAC or WAV.
