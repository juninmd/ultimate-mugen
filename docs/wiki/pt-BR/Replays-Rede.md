# Replays gravados no jogo (`replays/`)

[English](../Netplay-and-Replays.md) · **Português (BR)**

Cada luta terminada é arquivada automaticamente em `replays/`, ao lado do executável, como um pequeno arquivo JSON
(`<nome>-<hora-unix>.umr.json`). A entrada **Replays** do menu lista os arquivos, do mais novo para o mais antigo.

| Campo | Significado |
|---|---|
| `format` / `version` | `ultimate-mugen-replay` / `1`; outra versão é recusada com mensagem clara |
| `label`, `saved_at` | nome mostrado no navegador e o carimbo de tempo Unix (usado na ordenação) |
| `mode` | `Arcade`, `Cpu`, `Versus`, `Training`, `Watch`, `TeamSeq`, `TeamSimul`, `TeamTag` |
| `chars`, `stage`, `palettes` | ids dos lutadores, id do cenário e a paleta de cada lado |
| `seed` | semente inicial do RNG da luta (o replay a restaura antes do primeiro tick) |
| `teams`, `simul`, `tag` | elencos por id de lutador (modos de equipe) |
| `rules` | rounds, tempo do round, % de vida, round único, pushbox, vida compartilhada, velocidade |
| `inputs` | trilha de entradas com run-length: `slots`, depois `count` + uma entrada por slot por trecho |

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
| `.` ou → com o replay pausado | avançar um quadro |
| Esc | sair do replay e voltar ao navegador |

A pasta guarda os 64 arquivos mais novos; os antigos são removidos quando um novo replay é gravado. Arquivos que não
podem ser lidos (por exemplo, gravados por outra build) aparecem com o motivo em vez de falharem em silêncio.

Os detalhes do netplay por rollback continuam em [Netplay-and-Replays](../Netplay-and-Replays.md).
