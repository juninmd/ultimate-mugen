# GitHub marketplace

**English** · [Português (BR)](pt-BR/Mercado.md)

Install characters, stages and music straight from any public GitHub repository. Open **Mercado GitHub** in the main menu.

## Quick start

1. Press `U` (or click **+ Adicionar**) and paste a link: `https://github.com/owner/repo`, `owner/repo`, or a folder link such as `https://github.com/owner/repo/tree/main/mugen`.
2. The game reads the repository tree and lists what it finds.
3. Mark items (`Space`, click, or **Todos**) and press **Instalar** (`I`). Downloads run in parallel with a progress bar.
4. Installed items are active right away. Use **Ativar** (`E`) and **Desativar** (`O`) on any selection, including bulk selections, or **Remover** (`X`) to delete them from the device.

Everything is stored locally, so the repository list, the installed items and their on/off state are still there next time.

## What is detected

| Kind | Rule | Installed files |
|---|---|---|
| Character | folder with a `.def` plus `.sff`, `.air` and `.cns`/`.cmd` in its tree | the whole folder, plus files declared in `[Files]` that live elsewhere in the repository |
| Stage | `.def` with a same-name `.sff`, or any `.def` under a `stages/` folder | the stage files, its `spr` sprite file and its `bgmusic` file |
| Music | `.ogg`, `.mp3`, `.flac`, `.wav` (wav under 32 KB is ignored as a sound effect) | the audio file |
| Pack | `.zip` | the archive, unpacked in memory like a manual import |

Screenpack files (`system.def`, `select.def`, `fight.def`, ...) are never treated as stages or characters. Files inside hidden folders (`.git`, `.github`) are skipped.

Active music tracks play as the fight BGM in rotation whenever the stage has no music of its own. A M.U.G.E.N stage with `bgmusic` keeps its own track.

## Where data lives

| Platform | State and downloads |
|---|---|
| Linux | `$XDG_DATA_HOME/ultimate-mugen/market` (default `~/.local/share/ultimate-mugen/market`) |
| macOS | `~/Library/Application Support/ultimate-mugen/market` |
| Windows | `%APPDATA%\ultimate-mugen\market` |
| Android | app internal storage, `market/` |
| Web | IndexedDB database `ultimate-mugen-market` |

Set `UME_MARKET_HOME` to move the folder on desktop. Each installed item is a single bundle file; the list of repositories, the installed items, the ETag of each listing and the optional token are in `state.json`. Bundle file names are percent-encoded with lowercase letters only, so items and folders whose names differ just in letter case never share a file on Windows or macOS.

## Rate limits, token and caching

The listing uses the GitHub API (60 requests per hour without a token) and file downloads use `raw.githubusercontent.com`, which is not limited the same way. Listings send the saved ETag, so an unchanged repository answers `304` and is served from the local cache. A repository is refreshed when you open it if the cache is older than 10 minutes, or on demand with `R`.

To raise the API limit press `T` and paste a personal access token (no scopes needed for public repositories), or set `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` on desktop. The token is only ever sent to `api.github.com`.

## Safety limits

- Only `https` links to `github.com`, `api.github.com`, `raw.githubusercontent.com`, `media.githubusercontent.com` and `codeload.github.com` are fetched. Redirects to other hosts are blocked.
- Paths coming from a repository are validated: no `..`, absolute paths, backslashes, drive letters or control characters. Unsafe entries are dropped from the listing.
- Limits: 64 MB per file, 256 MB and 4000 files per item, 48 MB for the listing. Items above the limits are shown as "grande demais" and cannot be installed.
- Content never runs as code. Characters are loaded by the same sandboxed parsers as manual imports.
- Respect each author's license. Installing from a repository is a download, not a redistribution right.

## Platform notes

| Platform | Adding a repository |
|---|---|
| Desktop | type in the game (paste is not available in the game window; type `owner/repo`) |
| Web and phone browser | the browser's own prompt, with paste |
| Android app | choose a suggestion; the app has no in-game keyboard yet |

## Related

- [Adding M.U.G.E.N content](MUGEN-Content.md) for the manual `mugen/` folder.
- [Configuration](Configuration.md) for environment variables.
