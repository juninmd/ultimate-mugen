# Modo gráfico leve: medições (1280x720, Kaito vs Shion, renderização por software)

| Preset | Desktop (lavapipe) frame médio | Web (SwiftShader) fps |
|---|---|---|
| Alta (= comportamento anterior) | 28.7 ms | 10.0 a 10.5 |
| Média | 28.9 ms | 11.5 |
| Baixa | 19.5 ms | 18.7 |
| Mínima | 15.8 ms | 19.8 a 21.8 |
| Alta + ajuste automático (web, após 30 s) | | 20.0 (caiu sozinho para Mínima e salvou) |

Comandos: `UME_PRESET=N UME_TOUR=perf UME_PERF_SECONDS=8 xvfb-run -a target/release/ultimate-mugen --shots DIR --res 1280x720 --mute`; web: partida `?e2e=1&modo=watch` em Chromium, contagem de rAF por 15 s.
