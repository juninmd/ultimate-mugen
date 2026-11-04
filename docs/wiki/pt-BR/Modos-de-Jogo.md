# Modos de jogo

[English](../Game-Modes.md) · **Português (BR)**

O menu principal oferece lutas individuais (Arcade, Versus CPU, 2 Jogadores, Treino, CPU vs CPU) e três modos de equipe. O tamanho da equipe é escolhido na seleção com `Q` / `E`.

| Modo | Lutadores em campo | O round acaba quando |
|---|---|---|
| Equipe · Sequência | 1 por lado, o próximo entra após um KO; o vencedor mantém a vida | um lado fica sem lutadores |
| Equipe · Paralelo | a equipe inteira ao mesmo tempo (2 a 5); `Tab` passa o controle ao próximo | todos os membros de um lado caem |
| Equipe · Tag | 1 ativo + os demais em espera (2 ou 3 por lado) | todos os membros de um lado caem |

## Sobrevivência

`Sobrevivência` no menu principal é uma sequência sem fim contra o elenco inteiro:

- **Oponentes.** Depois de escolher seu lutador, o jogo sorteia o adversário a cada rodada (nunca o chefe oculto, nunca um pacote quebrado). O cenário gira a cada rodada.
- **Escalada.** O nível da CPU sobe a cada duas lutas, partindo da sua dificuldade e com teto 8.
- **A vida continua.** A barra com que você termina uma luta é a mesma com que começa a próxima, mais a recuperação configurada em Jogabilidade › *Vida recuperada (Sobrevivência)* (0 / 20 / 35 / 50 / 100 %). Um KO encerra a sequência.
- **Pontuação.** Cada vitória soma o `Score`/`ScoreTotal` da engine para o seu lado mais um bônus de `1000 × rodada`, então o número é determinístico e vem da própria simulação.
- **Fim de jogo.** A tela final mostra a pontuação, quantas lutas você venceu e o recorde local; o recorde fica salvo em `ultimate-mugen.cfg` (`survival_best`) e um novo recorde é destacado.
- **Replays.** A última luta é arquivada como qualquer outra e pode ser assistida pela tela de fim de jogo (botão R) ou pelo navegador de Replays; o replay guarda a vida inicial para a re-simulação bater exatamente.

Estatísticas de pontuação usadas: `Fighter::score` e `Fighter::score_total`, preenchidas por `HitDef.score` e pelos controladores `ScoreAdd`/`TargetScoreAdd` e lidas pelos triggers `Score`/`ScoreTotal`.

## Equipe Tag

Cada lado tem 2 ou 3 lutadores. No início de todo round só o primeiro está em campo; os outros esperam.

- **Troca.** Aperte o botão Tag (`R` no P1, Num Enter no P2; configurável em Controles, ação "Troca (Tag)"). Em controle e toque a troca usa o botão de despertar (R3 / SELECT), porque o despertar fica desativado neste modo. A CPU troca sozinha quando o lutador está abaixo de 30 % de vida e o parceiro está bem mais saudável.
- **Recarga.** Depois de cada troca o lado espera `tag_cooldown_max` ticks (padrão 180, três segundos). O HUD mostra uma barra e a marca `TAG` sob o retrato de cada parceiro.
- **Janela de tag cancel.** A troca só é aceita quando o lutador ativo tem controle, ou nos primeiros `tag_cancel_window` ticks (padrão 30) depois de um golpe seu tocar o oponente (acerto ou bloqueio). Dá para cancelar a recuperação de um golpe que conectou no parceiro, mas não no meio de um golpe no ar nem enquanto se apanha.
- **Lutadores em espera.** Ficam invisíveis, não podem ser acertados, empurrados nem alvejados (`enemy`, `P2`, `EnemyNear` os ignoram), não recebem entrada e mantêm a vida. Seguem a posição do lutador ativo, então entram exatamente onde a troca ocorreu, no estado 0 e com controle.
- **KO.** Quando o ativo morre, o próximo parceiro vivo entra sozinho após 50 ticks (a recarga é ignorada). O round só acaba quando todos os membros de um lado ficam sem vida; o tempo esgotado decide pela soma da vida da equipe.
- **Vida e variáveis** de quem saiu ficam intactas; todos começam o round seguinte com vida cheia.
- **Scripts.** `TeamMode` retorna `3` neste modo. `TagOut` / `TagIn` trocam os lutadores de verdade (veja [Extensões Ikemen](../Ikemen-Extensions.md)); `Standby` vale `1` para quem espera e `NumPartner` conta os parceiros.
- **Replays** gravam o botão Tag como qualquer entrada, então uma luta Tag se repete exatamente.

Chaves da engine (`Match`): `team_tag`, `tag_cooldown_max`, `tag_cancel_window`. Evento: `Event::Tag { side, fighter }`.
