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
- `src/model3d/aura.rs` desenha os efeitos só do 3D: rastro do `AfterImage` em cópias aditivas do quadro atual (Shunpo carmesim, Sonido dourado na forma Hollow) e uma barreira de reiatsu na defesa (estados 120-155) que pisca ao bloquear (150-155).

## Ferramentas

`tools/retarget` (Python, só stdlib): `retarget.py` aplica os clipes de `clips.json` num rig de `rigs.json`; `autorig.py` cria esqueleto e pesos para malha sem rig; `keyposes.py <in.glb> <out.glb> <rig>` aplica os clipes feitos à mão de `keyposes.json` (postura Bankai, sequência de espada, Getsuga com preparo, carga e disparo, Juujishou, Shunpo, Cero, defesa e pose de vitória), com direções de membros em `[azimute, elevação]` e `bladeR` resolvido por `blade_solve.py` para apontar a Zangetsu. Folha de clipes: `cargo run --release -p ume-model3d --example clip_sheet -- <modelo.glb> <saida.png> Clip1,Clip2 <quadros>`. Retrato: `cargo run -p ume-model3d --example portrait`. Vozes: `tools/voices/generate.py <id>` (mantém os outros bancos).

## Cenário 3D: Aeroporto

`airport3d` ("Aeroporto 3D — Pôr do Sol") é um fundo 3D ao vivo atrás da mesma feature `model3d`: sem a feature o cenário some, sem outra mudança.
- `assets/model3d/airport.glb` (128 KB) é gerado por `python3 tools/airport/build.py <saida.glb>` (só stdlib): aviões, terminal com fingers, torre de controle, pista, morros, nuvens e sol. Sem arte de terceiros.
- Clipe `ambient` (loop de 40 s): um jato pousa na pista ao fundo, outro taxia no pátio e um trator de bagagem cruza a vaga.
- `Model::project_scene` (`crates/model3d/src/perspective.rs`) projeta em perspectiva (corte de plano próximo, Lambert plano, neblina por vértice, céu em gradiente) no mesmo `Raster` com z-buffer dos lutadores.
- `src/model3d/airport.rs` renderiza 480x270 (320x180 no modo lite) a 30 Hz numa textura; com "fundo" desligado no lite aparece uma imagem estática da mesma cena.

## Golpes do Ichigo

Pulos: `D, U` ou `U, U` super pulo (também no ar), dois pulos aéreos, três dashes aéreos. Especiais: Getsuga Tenshou (`~D, DF, F, x`), Getsuga carregado (`~D, DB, B, y`, segure `y`; 45 ticks para a onda máxima), Shunpo (`~D, DB, B, b`, rastro de Sonido na forma Hollow), Shunpo Rush, Hollow Cero (`~D, DB, B, a`, segure `a`; 40 ticks para o raio grande), Getsuga Juujishou, Kuro Getsuga, Hollow Rampage; Hollow Total adiciona Cero Oscuras. A carga usa o campo `holdRelease` (ver [Personagem UMF](Personagem-UMF.md)).

## Créditos

Veja `assets/model3d/ATTRIBUTION.md`.
