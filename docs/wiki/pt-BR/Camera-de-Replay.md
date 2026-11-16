# Câmera de Replay

A câmera de replay é um diretor só de exibição. Ela enquadra a luta durante a reprodução, mas nunca muda a simulação: a câmera é salva antes do desenho e restaurada logo depois, então o resultado do replay e os checksums continuam iguais.

Entrada por frame: um `Subject` por lutador (`x` e se um hit está ativo). Saída: o foco `x` e o `zoom` do desenho.

- O foco é o ponto médio dos lutadores, puxado 35% em direção ao atacante para enquadrar o golpe.
- O pan avança 12% do caminho até o foco a cada frame, somado à câmera da simulação.
- O zoom aperta quando os lutadores estão perto, até 1.6x o zoom da partida, e fica no zoom da partida quando estão longe.
- Sem lutadores válidos, mantém o enquadramento anterior.

Puro e determinístico: sem relógio, sem aleatoriedade.

Código: `crates/engine/src/replay_camera.rs` (enquadramento aplicado em `src/app/fight_screen.rs`, em volta do desenho). Testes: `crates/engine/tests/replay_camera.rs`.
