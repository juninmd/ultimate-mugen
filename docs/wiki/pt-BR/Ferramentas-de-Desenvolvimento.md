# Ferramentas de desenvolvimento

[English](../Dev-Tools.md) · **Português (BR)**

As ferramentas de build e auditoria do repositório são binários Rust, não scripts. `crates/devtools` gera o `ume-tools`:

| Comando | Substitui | O que faz |
|---|---|---|
| `ume-tools pack-sprites [out] [--sprites dir]` | `scripts/pack_sprites.py` | Empacota `assets/chars/sprites/<id>/` em `<out>/assets/chars/sprites/<id>.pack` para o build web (limite de 768 px, 256 cores, âncoras do `meta.json` reescaladas, em paralelo). Chamado por `scripts/build-web.sh` |
| `ume-tools stage-thumbs [stages]` | `scripts/generate-stage-thumbnails.py` | Grava miniaturas 240x135 (Lanczos) em `<stages>/thumbs` |
| `ume-tools audit-moves [dir] [--json]` | `scripts/audit_native_moves.py` | Auditoria estática do elenco: inputs, cancels, throws, hit boxes |
| `ume-tools audit-sprites --out report.json` | `scripts/audit_native_sprites.py` | Inventário de PNGs por arquivo com limites alfa |

Rode com `cargo run --release -p ume-devtools --bin ume-tools -- <comando>`. A cobertura de resolução de poses é `native_sprite_audit --require-complete` (o `native_sprite_coverage.py` removido apenas a espelhava). As ferramentas de criação de personagem ficam em `crates/charforge` ([Ferramentas de sprites](Ferramentas-de-Sprites.md), [Gerador de golpes](Gerador-de-Golpes.md)).

## Ainda em Python

Resta apenas `tools/story_segments.py`: ele desenha as faixas dos cenários do modo história com o gerador aleatório do Python e rasterizadores do Pillow, e a arte já versionada precisa continuar idêntica, então não foi portado. O resto é `ume-tools`: `gen-story-npcs` (idêntico ao script antigo, mas sobrescreve arquivos de NPC que divergiram), `motion-descriptor`, `preview-motion`, `preview-scene`, `ability-reel` (ffmpeg; legendas com fonte bitmap embutida) e a biblioteca `umf_edit` (edição de JSON preservando ordem das chaves e fim de linha). O script avulso da imagem de showcase foi removido; a imagem continua versionada.
