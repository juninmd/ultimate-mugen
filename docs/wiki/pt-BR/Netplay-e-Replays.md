# Netplay e Replays

[English](../Netplay-and-Replays.md) · **Português (BR)**

O netplay usa rollback (GGRS): cada jogador roda a luta localmente, prevê o controle do adversário e corrige quando o controle real chega. Replays guardam só os controles de cada frame. A versão em inglês tem todos os detalhes de netplay; esta página resume os replays em disco.

## Replays em disco (desktop e Android)

Toda luta terminada é salva em `replays/`, ao lado do executável (Android: pasta de dados do app; use `UME_REPLAY_DIR` para mudar). Os arquivos se chamam `<data>-<hora>-<p1>-vs-<p2>.umr`. Lutas online ainda não são salvas.

Abra **Replays** no menu principal:

| Tecla | Ação |
|---|---|
| Cima / Baixo | escolher o replay |
| Enter | assistir |
| R | renomear (digite o nome, Enter confirma, Esc cancela) |
| Delete | apagar (pede confirmação) |
| Tab | mudar a ordem: mais recentes, mais antigos, nome |
| Esc | voltar |

Durante o replay:

| Tecla | Ação |
|---|---|
| 1 / 2 / 4 | velocidade 1x / 2x / 4x (Cima / Baixo percorrem as velocidades) |
| Espaço | pausar e continuar |
| `.` ou Direita | avançar um quadro (pausa antes) |
| Esc ou P | menu de pausa |

A barra na tela tem os mesmos botões para mouse e toque. Ao chegar no último tick gravado, o estado da luta é comparado com o checksum guardado no arquivo e a barra mostra `VERIFICADO` ou `DIVERGIU`.

O replay guarda a versão da engine, modo, cenário, ids dos personagens de cada time, paletas, semente do RNG, checksum final e os controles de cada tick. Replays só rodam na versão que os gravou: um replay de outra versão aparece em vermelho com a mensagem `This replay was recorded with build X; this is build Y.` e não é iniciado. Se faltar um personagem ou cenário, aparece `Personagem não instalado` ou `Cenário não instalado`.

A versão web não salva replays em disco; a última luta continua disponível na tela de resultados.
