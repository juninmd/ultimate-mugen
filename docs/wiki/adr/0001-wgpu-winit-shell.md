# ADR 0001: Own wgpu renderer and winit shell, no game framework

- Status: Accepted
- Date: 2026-11-03

## Context

M.U.G.E.N sprites are palette-indexed and need palette swaps, PalFx and large atlases at low cost. The engine must run on Windows, Linux, macOS, the browser (WebGL2), Android and iOS from one codebase.

## Decision

Rendering is `ume-render` on wgpu with the palette in the fragment shader. The window, input and surface live in `ume-shell` on winit. Game code draws only through `ume_shell::quad`, never through wgpu directly. macroquad and Bevy are not used.

## Consequences

One pipeline serves every target and indexed sprites upload one byte per pixel. We own the batching and atlas budget, so those need benchmarks when changed. The `quad` name is a leftover from an earlier API shape and may be renamed in a separate refactor.
