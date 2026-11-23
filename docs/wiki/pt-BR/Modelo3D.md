# Lutadores 3D (protótipo)

Status: protótipo isolado, não ligado ao jogo. Crate `crates/model3d` (`ume-model3d`); remover o membro do workspace e a pasta reverte tudo.

## Princípio

A simulação continua 2D. Hitboxes, física, máquina de estados e rollback nunca leem o modelo. O lutador 3D só desenha o mesmo `stateno` e `statetime` que o 2D usaria, então um 3D pode lutar contra um 2D.

## Pipeline

1. `Model::from_glb` lê `.glb` (parser próprio sobre `serde_json`, sem dependência nova): nós, skins, malhas e animações (linear e step; cubic spline é rejeitado).
2. `StateMap` (JSON) mapeia números de estado MUGEN (faixas) para clipe, loop e velocidade; estados desconhecidos usam o clipe padrão.
3. `Model::pose_for_state` amostra o clipe em `statetime / 60 * speed` e aplica skinning na CPU. É função pura das entradas, seguro para rollback.
4. `Model::project` usa câmera ortográfica no plano da luta, espelha por `Facing`, sombreia cada face com uma luz direcional e devolve `Triangle`s de trás para frente em pixels.

Os triângulos vão para `ume_shell::quad::draw_mesh` (cor por vértice), sem mudar o pipeline de GPU nem o lote único do web.

## Limites e próximos passos

- Sem texturas ainda. O fixture `Fox.glb` é CC-BY 4.0 e não é distribuído.
- Próximo: feature flag `model3d` (desligada), campo `model` no UMF, fallback 2D no modo leve, cenário 3D com paralaxe.
