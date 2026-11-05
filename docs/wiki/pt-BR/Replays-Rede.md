# Replays e Jogo em Rede

[English](../Netplay-and-Replays.md) · **Português (BR)**

> O netplay usa rollback (GGRS): cada jogador roda a luta localmente, prevê o controle do adversário e corrige quando o controle real chega. Replays guardam só os controles de cada frame.

## Formato binário (`ume_netplay::Replay`)

Uma trilha pura de entradas gravadas:

| Campo | Tipo |
|---|---|
| magic | `UMRP` |
| version | `u32` (1) |
| players | `u32` |
| contagem de metadados, depois strings chave/valor | tamanho `u32` + UTF-8 |
| contagem de quadros | `u32` |
| runs | repetição `u32` + uma entrada `u32` por jogador |

## Replays salvos no jogo (`replays/`)

Cada luta terminada é arquivada automaticamente em `replays/`, ao lado do executável, como um arquivo JSON pequeno
(`<rótulo>-<unix-time>.umr.json`). O menu **Replays** lista do mais novo para o mais antigo.

| Campo | Significado |
|---|---|
| `format` / `version` | `ultimate-mugen-replay` / `1`; versões diferentes são recusadas com mensagem clara |
| `label`, `saved_at` | nome exibido no navegador e carimbo Unix (usado na ordenação) |
| `mode` | `Arcade`, `Cpu`, `Versus`, `Training`, `Watch`, `TeamSeq`, `TeamSimul`, `TeamTag` |
| `chars`, `stage`, `palettes` | ids dos lutadores, id do cenário e paleta de cada lado |
| `seed` | semente inicial da luta (o replay restaura antes do primeiro tick) |
| `teams`, `simul`, `tag` | elencos por id de lutador (modos de equipe) |
| `rules` | rounds, tempo do round, % de vida, round único, pushbox, vida compartilhada, velocidade |
| `inputs` | trilha de entradas com run-length: `slots`, depois `count` + uma entrada por slot por trecho |

A decodificação de replays limita arquivos a 16 MiB, 500.000 trechos e 2.000.000 quadros no total para evitar alocações de memória ilimitadas por fluxos malformados. Relatórios de dessincronização (`desync-*.json`) em `replays/` são preservados sem serem listados como replays com erro.

O replay alimenta as entradas gravadas por `Pilot::Replay`; a simulação é determinística, então a luta é
reconstruída exatamente. Um teste de ida e volta grava 1200 ticks, recarrega o arquivo e compara o checksum final
(`cargo test -p ume-engine --test replay_roundtrip`).

Teclas do navegador:

| Tecla | Ação |
|---|---|
| ↑ / ↓, Enter | escolher e jogar |
| R | renomear (Enter confirma, Esc cancela) |
| Delete (duas vezes) | apagar o arquivo selecionado |
| S | alternar ordenação entre data e nome |
| Esc | voltar ao menu |

Teclas durante a reprodução:

| Tecla | Ação |
|---|---|
| Espaço / Start | pausar e continuar |
| 1 / 2 / 4 | velocidade 1x, 2x, 4x |
| `.` ou → pausado | avançar um quadro |
| Esc | sair do replay e voltar ao navegador |

A pasta guarda os 64 arquivos mais novos; os mais antigos são apagados quando um novo replay é salvo. Arquivos que
não puderem ser lidos (por exemplo, gravados por outra versão) aparecem com o motivo em vez de falharem em silêncio.
