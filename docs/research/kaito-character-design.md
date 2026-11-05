# Kaito character research

Research date: 2026-11-05.

Elecbyte's [character tutorial](https://raw.githubusercontent.com/fanyer/mugen/master/docs/tutorial1.html) describes the roles of the character definition, command, animation, state and sound files. Its [CNS reference](https://raw.githubusercontent.com/fanyer/mugen/master/docs/cns.html) describes states and controllers, and its [AIR reference](https://raw.githubusercontent.com/fanyer/mugen/master/docs/air.html) describes animation elements and collision boxes. These references inform the design; Kaito is a native UMF fighter, not a separately distributed M.U.G.E.N character.

The UMF compiler turns moves into commands and state controllers, and animation frames into sprites and collision boxes. On contact it permits normal-to-normal, normal-to-special and special-to-super transitions. Each new attack needs matching active frames, hit definitions, recovery and an input that does not hide existing moves.

At the start of this work, awakening set the fighter's form and applied combat modifiers, but the renderer only added a tint and halo to the existing sprite. Blond hair therefore requires a distinct visual representation across the character's animations, not just a global tint. The move `voice` field existed in the schema but was not consumed by the native compiler. A Portuguese voice needs an asset and an explicit playback path, followed by audible in-game verification.

The candidate samples in `assets/chars/voices/kaito/` are synthetic Brazilian Portuguese speech generated locally with Windows Speech Synthesis, using Microsoft Daniel at rate 1. The lines are “Punho do dragão!”, “Tempestade!”, “Vento cortante!” and “O olho da tempestade despertou!”. They are initial takes: acting quality and playback in a match still require listening and verification.

On 2026-11-05, local faster-whisper-small transcription recovered all four lines, and FFmpeg measured non-silent samples with peaks between -6.3 and -4.2 dBFS. Automated transcription does not establish acting quality or prove in-game playback.

Acceptance evidence should include normal and awakened hair, the dragon emerging from the punch, a demonstrated combo, and audible Portuguese lines. Capture the actual game rather than a mockup. Check both normal and transformed play, including round reset and both facing directions.
