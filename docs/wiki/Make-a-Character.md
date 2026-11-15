# Make a character (quick start)

**English** · [Português (BR)](pt-BR/Criar-um-Personagem.md)

This is the short path. You will make a working fighter in about ten lines, then grow it. Every field is listed in the [full UMF reference](UMF-Character.md).

## 1. Start from a fighter that already works

A UMF file can **inherit** a whole fighter with `extends`. Your fighter gets the stats, art, animations and moves of the parent, and you write only what changes.

Copy this file to `umf/hello.json` (or use the ready one in [`docs/examples/umf/hello-fighter.json`](../examples/umf/hello-fighter.json)):

```json
{
  "format": "umf/1",
  "id": "hello",
  "name": "Hello Fighter",
  "author": "you",
  "color": "#2a9d8f",
  "extends": "kaito",
  "palettes": [{ "top": "#2a9d8f", "trim": "#16544d" }],
  "moves": [{ "id": "gale", "name": "Sky Gale", "cost": 500 }]
}
```

What it says:

| Line | Meaning |
|---|---|
| `format` | Always `umf/1` |
| `id` | Unique, lowercase, no spaces. It is also the file's internal name |
| `name`, `author`, `color` | What the select screen shows. `color` is the card colour |
| `extends` | Inherit everything from `kaito` (any loaded fighter or kit works) |
| `palettes` | Recolour. Keys are the colour names of the parent's art (`top`, `trim`, `hair`) |
| `moves` | Change a move by `id`. Here `gale` keeps its inputs and gets a new name and cost |

## 2. Where the file goes

- **Next to the game:** put it in `umf/`. No rebuild needed.
- **Or** point to a folder with `UME_UMF_DIR=/path/to/folder`.
- **Built into the game:** put it in `assets/chars/`, add its `id` to `assets/chars/index.json` if you want a set order, and run `cargo run --release`.

Run the game and pick **Training** to test it. Press `F2` to show hitboxes.

## 3. Add a new move

Moves are matched by `id`, so you can add, change or remove any of them:

```json
"moves": [
  { "id": "gale", "name": "Sky Gale", "cost": 500 },
  { "id": "slide", "remove": true },
  {
    "id": "crest", "name": "Crest Kick", "input": "F, F, b",
    "kind": "special", "anim": "gale",
    "hits": [{ "at": 3, "damage": 80, "level": "heavy", "launch": true }]
  }
]
```

- `input` uses the command syntax: `D, DF, F, x` is a quarter circle plus `x`; `x+y` means both at once.
- `kind` is `normal`, `special` or `super`. A normal cancels into specials and supers on hit or block.
- `anim` must be the name of an animation in the parent (`gale` here). Your own animations go in `"anims"`.
- `hits[].at` is the frame where the hit goes live. `damage` is the amount.

More: [moves](UMF-Character.md#moves), [hits](UMF-Character.md#hits), [command syntax](UMF-Character.md#command-syntax).

## 4. Give it your own look

- **Colours only:** change `palettes`, or add `awakeningPalette` for the transformation.
- **Stats:** `"stats": { "life": 1080, "attack": 104 }` changes one number and keeps the rest.
- **Sprites:** your own pixel art goes in `assets/chars/sprites/<id>/`. See [Sprite atlas import](Sprite-Atlas-Import.md) and [Rig](Rig.md).
- **Profile:** a nationality and a short bio in English, Portuguese and Spanish. See [Profile](UMF-Character.md#profile-nationality-and-biography).

## 5. Check it before you play

```bash
cargo test --release -p ume-engine --test wiki_examples
cargo test --release --test native_roster_lab -- --nocapture
```

The first command checks the examples in this guide. The second plays every attack of the roster and prints damage and reach.

## Not a UMF fighter?

- **M.U.G.E.N characters** (`.def`, `.sff`, `.cns`) go in `mugen/chars/`. They run without changes. See [Characters](Characters.md) and [CNS scripts](CNS-Scripts.md).
- **A fighter without sprites:** use the rig (the default `art.type`). See [Rig](Rig.md).

## Next

[Full UMF reference](UMF-Character.md) · [Reuse with kits](UMF-Character.md#reuse-extends-use-and-kits) · [Make a stage](Make-a-Stage.md) · [Wiki home](Home.md)
