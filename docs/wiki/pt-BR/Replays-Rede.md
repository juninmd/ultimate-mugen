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
O leitor recusa arquivos incompletos ou com trechos maiores que a contagem declarada, bytes extras no fim, arquivos acima de 129 MiB, trilhas decodificadas acima de 64 MiB e mais de 16.384 entradas de metadados. O gravador retorna erro quando jogadores, quadros ou metadados excedem esses limites. O tamanho codificado permite um trecho para cada quadro no pior caso, mantendo a trilha decodificada limitada a 64 MiB.

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

A pasta guarda os 64 arquivos mais novos; os antigos são removidos quando um novo replay é gravado. Arquivos que não
podem ser lidos (por exemplo, gravados por outra build) aparecem com o motivo em vez de falharem em silêncio.

Os detalhes do netplay por rollback continuam em [Netplay-and-Replays](../Netplay-and-Replays.md).

## Espectadores e dados de netplay nos replays

Um terceiro tipo de conexão observa sem enviar entrada:

```
ultimate-mugen --spectate --peer <ip-do-host>:7000 [--port 7002] [--chars a,b --stage id]
```
Ele monta uma `SpectatorSession` do GGRS (`PlayerType::Spectator`), renderiza as entradas confirmadas pelo host e nunca envia entrada local; o overlay mostra `ESPECTADOR · ping · quadros atrás`.

Replays gravados online guardam a configuração da sessão (`netplay`: lado, peer, porta, delay, janela de rollback, número de jogadores, flag de espectador) junto das entradas, então um desync pode ser reproduzido com os mesmos ajustes. Quando o GGRS detecta um desync, o jogo grava `replays/desync-<unix>.json` com essa configuração, o quadro, o `Match::checksum()` e os últimos 120 quadros de entrada. O relatório de desync é um diagnóstico, não um replay: o menu Replays nunca o lista (nem como luta, nem como arquivo ilegível), então ele não parece um replay corrompido depois de um desync.
