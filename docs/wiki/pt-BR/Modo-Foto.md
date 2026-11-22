# Modo Foto

O modo foto congela a luta e entrega câmera livre, filtros de cor e um quadro limpo para salvar em PNG. Funciona em luta normal, treino, sobrevivência e replays: pause o replay no golpe perfeito e tire a foto.

## Abrir e fechar

1. Pause a luta (`P`, `Esc`, `Enter` ou o botão de pausa). No replay, pause com `Espaço`.
2. Aperte `C`. O menu de pausa some e a câmera é sua.
3. `Esc`, `P` ou `C` sai do modo foto e volta ao menu de pausa.

O modo foto não existe em partidas online, onde a simulação nunca pode parar.

## Controles

| Tecla | Ação |
|---|---|
| `W A S D` / setas | Move a câmera (mais devagar quanto mais perto) |
| `+` / `-`, `PageUp` / `PageDown`, roda do mouse | Zoom entre 0,6x e 2,5x |
| `F` | Troca o filtro: Natural, Cinema, Noir, Vívido, Retrô |
| `H` | Mostra ou esconde o HUD (escondido por padrão) |
| `.` | Avança um quadro para achar o momento exato |
| `R` | Recentraliza a câmera |
| `Enter` / `Espaço` | Tira a foto (só desktop) |

A sobreposição (linha de estado, ajuda de teclas e guias de terços) não entra na imagem salva.

## Onde ficam as fotos

No desktop, `photos/ultimate-mugen-<milissegundos>.png` ao lado do executável, na resolução da janela. Web e mobile têm câmera e filtros, mas não salvam, porque a captura do navegador ou do sistema já resolve.

## Como funciona

- Câmera: `crates/engine/src/photo_camera.rs` só altera a exibição, como a [Câmera de Replay](Camera-de-Replay.md). Troca posição da câmera, tamanho da vista e altura do chão, desenha e restaura todos os campos, então simulação, resultado do replay e checksums não mudam. O movimento respeita os limites do cenário, o vertical vai até 120 unidades e o zoom de 0,6x a 2,5x.
- Filtros: cada filtro é um conjunto de valores do `Look` de pós-processamento (`saturation`, vinheta, grão, grade, aberração, bloom). O look é trocado só durante o desenho e restaurado em seguida; o nível de pós-processamento sobe para pelo menos 2, então os filtros aparecem mesmo com efeitos no mínimo.
- HUD: `Hud::hidden` pula HUD, barras de cinema e banners.
- Custo: sem passe de desenho extra, só o pós-processamento que o jogo já executa.

## Verificação

`UME_SHOTS_ONLY=photo xvfb-run -a target/release/ultimate-mugen --shots <dir> --res 1280x720 --mute` grava a pausa, o quadro limpo, o quadro com HUD, os quatro filtros e um plano aberto. Testes: `cargo test -p ume-engine --test photo_camera` e os testes `photo` do crate raiz.
