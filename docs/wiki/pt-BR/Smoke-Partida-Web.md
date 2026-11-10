# Smoke da partida na web

`node scripts/web-match-smoke.js web/dist` joga partidas completas no Chromium headless (job `web-smoke` do CI, depois do build web) e falha se a partida travar.

Cada cenário abre um deep link com `?e2e=1`, que publica `window.__ume_state` a cada frame (tela, `clock.tick`, round, vitórias, lutadores). As configurações entram via `localStorage` (rounds de 30 s, velocidade 1.15x, mudo).

| Cenário | O que cobre |
|---|---|
| `cpu-bot-best-of-3` | Um jogador roteirizado se aproxima e aperta ataques contra a CPU por pelo menos dois rounds |
| `watch-cpu-vs-cpu` | Sem input: a partida precisa chegar sozinha à tela de resultados |

Falha por: erro de página, panic de Rust, `RuntimeError` ou `console.error` inesperado; tick da simulação parado por `MATCH_SMOKE_STALL_MS` (padrão 20000 ms) fora de pausa e fim de luta; nenhum frame de animação ou thread principal sem resposta pelo mesmo tempo; imagem renderizada nunca mudar durante a luta; resultados não alcançados em `MATCH_SMOKE_LIMIT_MS` (padrão 360000 ms) ou impossíveis de fechar com Enter.

`MATCH_SMOKE_SCENARIOS=<nome,...>` roda um subconjunto e `WEB_SMOKE_SHOTS=<dir>` guarda capturas (o CI envia em caso de falha).
