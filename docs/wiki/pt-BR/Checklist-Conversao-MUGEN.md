# Checklist de conversão MUGEN para UMF

O que o `tools/mugen2umf` (skill `mugen-to-ume-migrator`) leva de um lutador M.U.G.E.N para um lutador nativo UMF, conferido com o Goku SSJ4. Cada execução grava `coverage.md` ao lado do JSON com o destino de cada `Statedef`.

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
| Movimento e paleta | `VelMul`, `VelAdd` por tick (gravidade própria, sem somar a nativa), `PosFreeze`, `PosSet`, `PalFX` como `velMul`, `velAdd`, `posFreeze`, `posSet`, `palFx` | pulo do Meteoro -103 = original |
| Extras | `--extra` mescla sem apagar habilidades detectadas e soma formas (Limit Breaker), clones, passivos, falas; golpe com `id` existente só troca os campos que lista | ok |
| Helpers | `Helper` → `spawn`; o estado do helper e o que ele chama (`ChangeState`, `HitOverride`, `HitDef p1stateno/p2stateno`) passam verbatim, renumerados a partir de 70000 | ok |
| Estados globais | `Statedef -2` / `-3` verbatim; `Parent,StateNo = N` que aponta para golpe convertido vira `@move(id)` | ok |
| Rekka / segurar / soltar | `ChangeState` com `command` dentro do golpe → golpe filho com `rekka`, ou `followUp on: "command"` / `"release"` | ok |
| Controladores chão/ar | controlador que só vale no ar (`StateType = A`) sai do golpe de chão e vice-versa | ok |
| Som próprio no acerto | `hitsound = S g,i` → `hitSound` + `ownSound: true` | ok |
| Paletas compartilhadas SFF v1 | `mugenversion = 1.x`: sprite compartilhado segue a paleta do anterior (fundos de super, efeitos); WinMUGEN mantém a regra do `.act` | ok |
| Rotas comentadas | `mugenRoutes` no extra destrava estado que o `.cmd` deixou comentado (hyper S.H.I.E.L.D. da Black Widow) | ok |

## Parcial ou aproximado

- Estados de agarrão próprios (Goku 616, 810, 820, 821, 1025-1028) são trocados pelo bloco `throw` do UMF.
- A velocidade da carga segue a rampa nativa (0 a 3000 em 150 ticks, o original chega a 930).
- O dano fica em 59-74 % do original porque a proração de combo nativa escala os hits.
- Gatilhos `AnimElem` em animação com loop só mapeiam a primeira volta (o Eat Lead da Black Widow dispara 2 rajadas em vez do loop inteiro).
- Black Widow: 32 de 43 rotas batem sons e dano do original; o resto são as cinemáticas em loop dos hypers, a cadeia de salto do Widow's Strike e o arremesso próprio.

## Ainda não migra

Controladores com condição que o conversor não consegue fixar num quadro (por exemplo `PosSet` quando `Pos Y < -175`) aparecem no `coverage.md` como descartados. O Goku SSJ4 não tem nenhum dentro dos golpes.

Código morto (estados que nenhum comando alcança) e estados sombreados (comando já tomado por uma rota anterior, então nunca disparam nem no M.U.G.E.N) aparecem no `coverage.md` e ficam de fora de propósito.

## Evidência

Rode o original e o UMF lado a lado:

```sh
UME_CHAR_DIR=<pasta-mugen> UME_TEST_UMF_JSON=<saida>/<id>.json UME_EVIDENCE_DIR=<dir> \
  cargo test --release -p ume-engine --test umf_parity_evidence -- --nocapture
```

Os resultados do Goku SSJ4 estão em `docs/evidence/umf-converter/` (`parity.md` e uma folha por cenário: linha de cima original, de baixo UMF).

Todos os golpes listados, estado original contra o golpe UMF (`routes.json` gravado pelo `convert`), mais os golpes que só existem no UMF:

```sh
UME_CHAR_DIR=<pasta-mugen> UME_TEST_UMF_JSON=<saida>/<id>.json UME_EVIDENCE_DIR=<dir> \
  UME_EVIDENCE_MOVES=qcf_x,qcb_x UME_EVIDENCE_EXTRA=golpe_novo \
  cargo test --release -p ume-engine --test umf_route_parity -- --nocapture
```

Os resultados da Black Widow estão em `docs/evidence/umf-converter/black-widow/` (`route_parity.md` e folhas).
