# Lutadores 3D

Status: entregue atrás da feature `model3d` do Cargo, ligada por padrão. Rollback: tirar `model3d` de `default` no `Cargo.toml` raiz; o jogo volta a desenhar os sprites 2D do doador e esconde a categoria 3D.

## Princípio

A simulação continua 2D. Hitboxes, física, máquina de estados e rollback nunca leem o modelo. Um lutador 3D é um lutador UMF (`assets/chars3d/<id>.json`, normalmente com `extends` de um kit nativo) cujo desenho é trocado por um modelo GLB, então um lutador 3D luta contra um 2D com as mesmas regras. O modo leve (`lite::scale() < 1`) volta para os sprites 2D.

## Pipeline

1. `Model::from_glb` lê o `.glb` (parser próprio sobre `serde_json`, texturas PNG via `png`).
2. `StateMap` (JSON) liga nomes de anim nativos (`anims`) ou faixas de estado MUGEN (`states`) a um clipe, loop e velocidade.
3. A pose é função pura de `anim_no`/`anim_elapsed` (ou `stateno`/`statetime`), skinning na CPU, seguro para rollback.
4. `Model::project` usa câmera ortográfica no plano da luta, sombreamento Lambert e triângulos de trás para frente, desenhados com `ume_shell::quad::draw_mesh` agrupados por textura.

## No jogo

- `src/model3d.rs` é a fachada (`draw`, `draw_fit`, `is_3d`, `portrait`); `src/model3d/specs.rs` lista os modelos.
- Luta: uma linha em `render.rs`. Painéis de seleção, versus e resultado: `draw_fit` dentro de `draw_char_fit`. Tiles e loading usam `portrait`.
- A seleção tem a categoria `3D`.
- A forma despertada troca para o segundo modelo (Ichigo: Hollow Total).

## Ferramentas

`tools/retarget` (Python, só stdlib): `retarget.py` aplica os clipes de `clips.json` num rig de `rigs.json`; `autorig.py` cria esqueleto e pesos para malha sem rig. Retrato: `cargo run -p ume-model3d --example portrait`. Vozes: `tools/voices/generate.py <id>` (mantém os outros bancos).

## Golpes do Ichigo

Pulos: `D, U` ou `U, U` super pulo (também no ar), dois pulos aéreos, três dashes aéreos. Especiais: Getsuga Tenshou, Shunpo, Shunpo Rush, Hollow Cero, Getsuga Juujishou, Kuro Getsuga, Hollow Rampage; Hollow Total adiciona Cero Oscuras.

## Créditos

Veja `assets/model3d/ATTRIBUTION.md`.
