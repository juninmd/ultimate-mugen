# Desafios de Combo

O modo treino mostra uma escada de sete metas de combo acima da arena. Funciona com qualquer lutador, nativo ou M.U.G.E.N, porque só lê os contadores de combo que o motor já mantém.

| # | Meta |
|---|---|
| 1 | 3 acertos |
| 2 | 5 acertos |
| 3 | 8 acertos |
| 4 | 25% da vida do rival em um combo |
| 5 | 6 ou mais acertos terminando com super (ao menos 1000 de energia gasta) |
| 6 | 10 ou mais acertos sem gastar barra |
| 7 | 45% da vida do rival em um combo |

O combo é avaliado quando termina. As metas são cumpridas em ordem, um aviso mostra DESAFIO CUMPRIDO e o progresso é salvo por lutador em `trials.json` (ao lado da configuração, ou no armazenamento do navegador na versão web). Ao terminar a escada aparece TODOS OS DESAFIOS CONCLUÍDOS.

## Como funciona

- `src/trials.rs`: `Tracker` puro, alimentado a cada quadro com `combo_count`, `combo_damage` e `power` do jogador e `life_max` do rival. Guarda a energia no início e o mínimo durante o combo para saber se houve gasto de barra.
- `src/app/trials_hud.rs`: lê a partida após cada tick (sem alterá-la) e desenha o painel; some na pausa, em replays e fora do treino.
- `Tracker::demo` substitui a amostra real e existe só para o tour de prints: `UME_SHOTS_ONLY=trials`.
- Custo: uma atualização de struct por quadro só no treino, sem alocação.
