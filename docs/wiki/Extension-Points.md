# Extension points

**English** · [Português (BR)](pt-BR/Pontos-de-Extensao.md)

Most content is data. This page maps each kind of content to the file that defines it, the page that documents it and whether a Rust change is needed. Add new content here first, in code only when the data cannot express it.

| Content | Where it lives | Documented in | Rust change |
|---|---|---|---|
| Native fighter | `assets/chars/<id>.json`, listed in `assets/chars/index.json` | [UMF character](UMF-Character.md), [Make a character](Make-a-Character.md) | No |
| Fighter bios and nationality | `assets/char_profiles.json` | [Characters](Characters.md) | No |
| M.U.G.E.N fighter or stage | `mugen/chars/`, `mugen/stages/` or the marketplace | [Adding M.U.G.E.N content](MUGEN-Content.md), [Marketplace](Marketplace.md) | No |
| Built-in stage from data | `assets/stages/data/<id>/stage.def` and its files, embedded by `build.rs` | [Make a stage](Make-a-Stage.md) | No |
| Stage look (rim light, tint, vignette) | `assets/stagelook.toml` | [Visual effects](Visual-Effects.md) | No |
| Stage props | `assets/stageprops.toml` and `assets/props/` | [Visual effects](Visual-Effects.md) | No |
| Story objects (crates, pickups, bonus rounds) | `assets/story/objects.toml` | [Stage objects](Stage-Objects.md) | No |
| Story epilogues | `assets/story/epilogues.json` | [Story mode](Story-Mode.md) | No |
| Arcade routes and unlocks | `assets/routes/unlocks.toml` | [Route files](Route-Files.md) | No |
| Post-processing | `assets/postfx.toml` | [Visual effects](Visual-Effects.md) | No |
| Passives, signature meters | fields of a UMF fighter | [Passives](Passives.md) | No |
| State controller or trigger | `crates/engine` | [Customizing the engine](Engine.md), [CNS scripts](CNS-Scripts.md) | Yes |
| New file format | `crates/formats` | [Architecture](Architecture.md) | Yes |
| New screen | `src/app/<screen>.rs` | [File layout](File-Layout.md) | Yes |

## Rules for a new kind of content

1. Define it as a data file under `assets/`, embedded through `build.rs` when it ships with the game.
2. Give it a wiki page with an English and a `pt-BR` version, and add a row to the table above.
3. Add a test that loads the shipped files, so a broken data file fails CI instead of the game.
4. Record the choice in an [ADR](adr/README.md) when it introduces a new format.
