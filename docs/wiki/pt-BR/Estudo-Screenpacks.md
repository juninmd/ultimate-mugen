# Estudo de screenpacks

O que screenpacks (motifs) de M.U.G.E.N e Ikemen GO fazem de diferente de um menu simples e o que o Ultimate MUGEN aproveitou. Só as ideias são usadas; nenhuma arte, layout ou som de terceiros é copiado.

## Fontes

Mesmas fontes da versão em inglês ([Screenpack-Study](../Screenpack-Study.md)): wiki do Ikemen GO (Screenpack features, Updating to 1.0), DeepWiki do menu, issue 329 e threads do MUGEN Guild. A busca ficou limitada a fóruns e espelhos de wiki, então os nomes de parâmetros vêm de fontes secundárias.

## Diferenciais

| Área | Diferencial | Ultimate MUGEN |
|---|---|---|
| Estrutura | `system.def` aponta para `select.def`, `fight.def`, sprites, sons e fontes, com fallback para o motif padrão | Igual, veja Screenpacks |
| Seleção | Grid e cursor por parâmetros, retratos animados, stage por personagem | Já nativo, veja Character-Select |
| Versus | Order select na tela de versus, retratos animados | Pendente (order select) |
| Vitória | Retrato, nome e frase do vencedor | Pose e nome existiam; frase pendente |
| Resultados | Tempo, combo e dano, melhores tempos no time attack | Novo painel de estatísticas |
| Resolução | Packs 4:3 e 16:9 via `localcoord` | `localcoord` convertido para 426x240 |

## Implementado

A tela de resultados mostra quatro chips de vidro: tempo de luta, maior combo de cada lado, dano total causado por cada lado e vida restante em porcentagem. `src/match_stats.rs` observa a partida a cada tick, fora do motor determinístico, então rollback e replays não mudam. Textos em inglês, português e espanhol.

## Próximos

1. Order select na tela de versus em modos de time.
2. Frase de vitória sob o nome do vencedor.
3. Nota (S a D) por tempo, vida restante e combo.
4. Melhores tempos por personagem no time attack.
