# Checklist de conversão MUGEN para UMF

O que o `mugen2umf.py` (skill `mugen-to-ume-migrator`) leva de um lutador M.U.G.E.N para um lutador nativo UMF, conferido com o Goku SSJ4. Cada execução grava `coverage.md` ao lado do JSON com o destino de cada `Statedef`.

## Migra hoje

| Área | Como fica no UMF | Conferido |
|---|---|---|
| Sprites | todo sprite do SFF em PNG, escala 1:1, eixo mantido | 198/198 |
| Sons | o `.snd` inteiro em WAV | 19/19 |
| Animações e caixas | `.air` com `Clsn1`, `Clsn2`, `Clsn1Default` | todas as actions |
| Comandos e golpes | rotas do `Statedef -1`, `when` stand/crouch/air/fly, `cost`, `cancels` | 86 golpes |
| Intro, vitória e provocação | estados 170-199 como `scripts.cns` verbatim, `Statedef 190` próprio no início do round | texto e som da intro iguais |
| Carga de ki | `abilities.charge` no **R** (Start) com anim e som de início, loop e fim | sons iguais |
| Voo e golpes de voo | `abilities.fly` mais golpes com `when: "fly"` | normais de voo 300-340 |
| Teleportes | `teleport` em etapas, ou `abilities.teleport` quando usa `p2dist` | deslocamento a 3 px |
| Cinemáticas longas | cadeias de até 12 estados, saídas por tempo, `landAt`, `followUp` hit/contact/hitNow | Meteoro toca os 6 sons |
| HitDef e projéteis | hits sem limite, `recover`, `missTime`, guarda, pausa, faíscas | ok |
| FX | Explod, AfterImage, EnvShake, EnvColor, SuperPause, NotHitBy | ok |
| Extras | `--extra` mescla sem apagar habilidades detectadas e soma formas (Limit Breaker), clones, passivos, falas | ok |

## Parcial ou aproximado

- Estados de agarrão próprios (Goku 616, 810, 820, 821, 1025-1028) são trocados pelo bloco `throw` do UMF.
- A velocidade da carga segue a rampa nativa (0 a 3000 em 150 ticks, o original chega a 930).
- O dano fica em 59-74 % do original porque a proração de combo nativa escala os hits.
- Gatilhos `AnimElem` em animação com loop só mapeiam a primeira volta.

## Ainda não migra

| Controlador | Efeito da falta |
|---|---|
| `VelMul` | pulos em cinemáticas longas sobem mais (Meteoro -132 vs -103) |
| `PosFreeze` | o lutador não fica preso por alguns quadros |
| `PosSet` | reposicionamento absoluto dentro do golpe |
| `PalFX` | flash de cor dentro do golpe |

Código morto (estados que nenhum comando alcança) e estados sombreados (comando já tomado por uma rota anterior, então nunca disparam nem no M.U.G.E.N) aparecem no `coverage.md` e ficam de fora de propósito.

## Evidência

Rode o original e o UMF lado a lado:

```sh
UME_CHAR_DIR=<pasta-mugen> UME_TEST_UMF_JSON=<saida>/<id>.json UME_EVIDENCE_DIR=<dir> \
  cargo test --release -p ume-engine --test umf_parity_evidence -- --nocapture
```

Os resultados do Goku SSJ4 estão em `docs/evidence/umf-converter/` (`parity.md` e uma folha por cenário: linha de cima original, de baixo UMF).
