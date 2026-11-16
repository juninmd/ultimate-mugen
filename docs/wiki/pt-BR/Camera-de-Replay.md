# Câmera de Replay

A câmera de replay é um diretor cinematográfico para a reprodução. Ela só escolhe para onde o enquadramento vai; o replay continua re-simulando os mesmos inputs, então o resultado da partida nunca muda.

Entrada por frame: um `Subject` por lutador (`x` e se está atacando). Saída: um `Shot` com o foco `x` e o `zoom`.

- O foco é o ponto médio dos lutadores, puxado 35% em direção ao atacante para enquadrar o golpe.
- O zoom aperta quando os lutadores estão perto, de 1.0 até 1.6, e fica em 1.0 quando estão longe.
- Pan e zoom suavizam até o alvo, então o enquadramento nunca dá salto.
- Posições não finitas são ignoradas; sem lutadores válidos, mantém o último enquadramento.

Puro e determinístico: sem relógio, sem aleatoriedade, então o mesmo replay gera o mesmo caminho de câmera em qualquer máquina.

Código: `crates/engine/src/replay_camera.rs`. Testes: `crates/engine/tests/replay_camera.rs`.
