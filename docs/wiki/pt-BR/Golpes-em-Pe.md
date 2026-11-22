# Golpes em pé (regra de queda)

Os lutadores caíam em cerca de um a cada sete golpes no chão. A regra abaixo faz golpes leves e médios apenas empurrarem o defensor, que continua em pé; golpes fortes, supers e combos longos ainda derrubam.

## Regra

Num golpe em defensor **no chão e vivo** cujo HitDef pede `fall`, a engine mantém o defensor em pé quando todas as condições valem:

- o golpe tira menos que o limiar da vida máxima do defensor (padrão 16%);
- o total do combo com este golpe fica abaixo de 3x o limiar (combo longo ainda termina em queda);
- o atacante está no escopo (padrão: só lutadores nativos);
- o ataque não é hyper (attr `H*`), não tem `down.bounce`, `wall.bounce` nem `ground.type` trip, nem `p2stateno`, nem `forcenofall`;
- o golpe não é o que mata.

Quando se aplica, o defensor usa os estados de hit em pé (`5000`/`5010`, depois `5001`/`5011`), a velocidade vertical é descartada e o empurrão horizontal cresce até 75% (quanto mais perto do limiar, maior). Golpes em defensor no ar (juggles) nunca são alterados.

Código: `crates/engine/src/matchs/knockdown.rs` (`Match::standing_push`), ligado em `Match::apply_hit`.

## Escopo e compatibilidade

| Escopo | Cobre | Motivo |
|---|---|---|
| Nativos (padrão) | golpes de lutadores UMF | o `fall` vem de `level: heavy` / `launch`, não é escrito golpe a golpe |
| Todos | também lutadores M.U.G.E.N | HitDefs M.U.G.E.N definem `fall` de propósito (rasteiras, lançadores); opcional |
| Clássico | ninguém | comportamento M.U.G.E.N exato |

O escopo segue o **atacante**, porque o HitDef é dado dele.

## Ajuste

Ajustes > Jogabilidade > *Quedas ao apanhar* (`knockdown` em `ultimate-mugen.cfg`): Clássico, Nativos 8%, Nativos 16% (padrão), Nativos 24%, Todos 16%. Limiar maior significa menos quedas. O valor é gravado nos replays (replays antigos tocam como Clássico) e o netplay usa o preset padrão.

## Medição

`ume-tools knockdown-rate --roster akio,bruno,... --frames 5400 --rule native --threshold 16` simula IA contra IA no elenco e mostra, por faixa de dano, quantos golpes em defensor no chão terminaram em queda.
