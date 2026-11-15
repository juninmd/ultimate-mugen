# Automatic updates

The desktop build updates itself from the latest GitHub Release. The release tag is the version source of truth: CI stamps `UME_RELEASE_VERSION` into the binary, publishes the assets, and the installed build replaces itself with the matching executable for its platform.

## Release assets

Every release carries, per platform (`windows-x86_64`, `linux-x86_64`, `macos-aarch64`):

| Asset | Purpose |
| --- | --- |
| `ultimate-mugen-v<version>-<platform>.zip` / `.tar.gz` | Manual install package (binary + README) |
| `ultimate-mugen-v<version>-<platform>[.exe]` | Raw executable downloaded by the auto-updater |
| `latest.json` | Update manifest read by the running build |
| `SHA256SUMS.txt` | Human-readable hashes for every asset |

`latest.json` keeps the archive under `platforms.<platform>` and adds the raw binary:

```json
{
  "version": "0.4.0",
  "notes": "Short release notes",
  "pub_date": "2026-11-05T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/juninmd/ultimate-mugen/releases/download/v0.4.0/ultimate-mugen-v0.4.0-windows-x86_64.zip",
      "sha256": "<archive sha256>",
      "size": 1234567,
      "binary": {
        "url": "https://github.com/juninmd/ultimate-mugen/releases/download/v0.4.0/ultimate-mugen-v0.4.0-windows-x86_64.exe",
        "sha256": "<binary sha256>",
        "size": 9876543
      }
    }
  }
}
```

Only the three published platforms can self-update. Other targets (web, Android, iOS, non-x86_64 Linux, Intel macOS) still read the notice-only path.

## Flow

1. At boot the game requests `releases/latest/download/latest.json` over HTTPS (8 s timeout, silently ignored when offline).
2. A newer `version` with a `binary` entry for the current platform shows an update panel on the title/menu with the installed and the available versions. Nothing is downloaded until the player accepts "Update now".
3. Accepting starts the background download (180 s timeout) to `.update-v<version>[.exe]` next to the running executable.
4. The download is rejected unless its `sha256` and byte size match the manifest and the staged executable answers `--version` with `ultimate-mugen <version>`.
5. Once verified, the staged file replaces the installed executable (on Windows the running file is first renamed to `.old`) and the game restarts with `--no-update-check`, so a bad manifest cannot cause a restart loop.
6. The previous `.old` file is removed at the next boot.

A failed download, an offline machine or a read-only install folder only changes the status line; the running build keeps working. "Not now" forgets the version until the next boot, and the footer notice stays visible.

## Command line and opt-outs

| Flag | Effect |
| --- | --- |
| `--check-update` | Prints whether a newer release exists and exits |
| `--update` | Downloads, verifies and installs the newest release, then exits |
| `--no-auto-update` / `UME_NO_AUTO_UPDATE=1` | Keeps the footer notice, never shows the prompt (so never downloads or installs) |
| `--no-update-check` / `UME_NO_UPDATE_CHECK=1` | No network request at all |

Local `-dev` builds never self-update.

## Verifying a release

`node scripts/release-manifest.mjs --verify latest.json --dir <assets> --version <version> --repo juninmd/ultimate-mugen` checks that both the archive and the raw binary exist, that their URLs point at the tagged release and that their `sha256` values match. The `Release` workflow runs exactly this check against the draft before publishing, and probes the live manifest afterwards.
