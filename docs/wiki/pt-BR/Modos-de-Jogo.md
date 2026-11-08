# Modos de jogo

[English](../Game-Modes.md) · **Português (BR)**

O menu principal oferece lutas individuais (Arcade, Versus CPU, 2 Jogadores, Treino, CPU vs CPU) e três modos de equipe. O tamanho da equipe é escolhido na seleção com `Q` / `E`.

| Modo | Lutadores em campo | O round acaba quando |
|---|---|---|
| Equipe · Sequência | 1 por lado, o próximo entra após um KO; o vencedor mantém a vida | um lado fica sem lutadores |
| Equipe · Paralelo | a equipe inteira ao mesmo tempo (2 a 5); `Tab` passa o controle ao próximo | todos os membros de um lado caem |
| Equipe · Tag | 1 ativo + os demais em espera (2 ou 3 por lado) | todos os membros de um lado caem |

## Arcade / Modo História

`Arcade` no menu principal executa a campanha narrativa do Torneio do Eclipse Escarlate com cutscenes completas, progressão dinâmica de vilões e desbloqueio de lutador secreto:

- **Prólogo (`Cutscene kind: 0`).** Cada um dos 15 lutadores nativos possui sua própria introdução narrativa e juramento pessoal antes de iniciar a escada do torneio.
- **Rotação Dinâmica de Vilões Principais.** O topo da escalada de 8 oponentes culmina em um confronto com cutscene pré-luta personalizada e arena temática do chefe:
  - **Tier 0 (Primeira Jornada):** O vilão supremo **Mikazuki Oboro** (`oboro`) no Portão dos Espíritos (`spirit`).
  - **Tier 1 (Após Derrotar Oboro):** **Garan, o Oni das Cinzas** (`garan`) torna-se o novo vilão principal na Cratera Vulcânica (`volcano`).
  - **Tier 2 (Após Derrotar Garan):** **Valdric Morn** (`valdric`) assume o trono como o tirano imperial final em Neo-Metropolis (`neon`).
  - **Zeradas Seguintes:** Alterna ciclicamente entre os 3 vilões principais. Se o jogador escolher o próprio chefe da rodada, a engine seleciona outro vilão para evitar luta espelho no chefe final.
- **Poderes de Chefe e Fases de Oboro.** Ao assumirem o posto de chefe final do modo história, os vilões recebem atributos de chefe (`boss = true`, 2000 HP, 125% de ataque, IA agressiva de chefe, recarga contínua de super barra, aura visual e banner `CHEFE FINAL`). Ao enfrentar Oboro, a Fase 1 traz sua forma Mangekyo Desperto (`form = 1`); derrotá-lo dispara a cutscene **Despertar: Eclipse Absoluto** (`kind: 3`) e a Fase 2 contra o **Eclipse Absoluto** (`form = 2`) com vida ampliada, regeneração contínua e golpes sombrios exclusivos.
- **Cutscene Final por Personagem (`Cutscene kind: 2`) e Desbloqueio de Oboro.** Ao derrotar o chefe final, cada personagem exibe sua própria cutscene de encerramento em 5 páginas (`src/app/cutscene/endings.rs`). Ao concluir, **Mikazuki Oboro** é desbloqueado na seleção de personagens, a rotação de vilões é avançada e o jogo retorna diretamente à tela de seleção de personagens com anúncio em banner comemorativo e o novo chefe habilitado.

## História (fases de pancadaria)

`História` no menu principal é uma campanha estilo Final Fight / Sengoku: cinco fases, cada uma com algumas **ondas de capangas lutando ao mesmo tempo** (1 contra 2 a 4, usando as regras de equipe simultânea) e fechada por um **chefe de fase**.

- **Estrutura.** Fase = 3 ondas + 1 chefe. Cenários: Becos da Cidade (praça), Metrô (Neo-Metrópole), Bambuzal, Área Industrial (Cratera Vulcânica) e Cobertura do Chefão (Dojo). Chefes: Bruno, Marlow, Lian, Garan, Valdric. Se você escolher um chefe como lutador, entra o substituto (Daichi, Kaname, Shion, Haruki, Kaito).
- **Chefes.** Têm 150 % de vida e 110 % de ataque; um round decide cada luta.
- **Vida acumulada.** A barra que você termina uma onda é a que começa a próxima, mais 20 % após a onda e 60 % após a fase (as "comidas" dos arcades originais).
- **Créditos.** São 3 créditos; ao cair você pode *Continuar* (retoma a mesma luta com 60 % de vida) até acabarem.
- **Pontuação.** `ScoreTotal` do motor mais bônus por onda (500 x fase, chefe x4).
- **Dificuldade.** Nível da CPU sobe por fase e +2 nos chefes (máx. 8).
- **Faixas de profundidade (estilo Final Fight).** O Story tem uma faixa de chão: dois botões extras andam para o fundo ou para a frente do cenário (P1 `Z` / `X`, P2 `Num -` / `Num +`, reconfiguráveis em Controles; os bits do motor são `LANE_UP` / `LANE_DOWN`). Só anda no chão e com controle, e cada fase limita a faixa ao chão real (`lane_far` / `lane_near` em `src/story/stages.rs`). O golpe só acerta, e os corpos só se empurram, quando os dois estão a até 14 unidades de profundidade; a CPU se alinha ao herói sozinha. Sprites, sombras, reflexos e efeitos acompanham a faixa e quem está mais perto desenha na frente. Os outros modos nunca ativam as faixas.
- **Botão de defesa.** O Story tem um botão exclusivo de defesa (P1 `C`, P2 `Num *`, reconfigurável). Enquanto segurado, quem está no chão e com controle bloqueia como se segurasse para trás, sem recuar (com Baixo, defende agachado). Fora do Story é ignorado.
- **HUD do Story.** Capangas não têm barra de vida nem ficha de equipe. O herói mantém o painel da esquerda, uma faixa inferior mostra fase, onda, pontos e créditos, e a luta de chefe ganha uma barra larga de **Vilão da Fase** (nome e vida) à direita.
- **Capangas (`assets/chars/npc_*.json`, gerados por `scripts/gen_story_npcs.py`).** Lutadores com paleta trocada, ocultos na seleção (`"npc": true`), cada um com uma habilidade própria: Brute (golpes fortes com armadura), Razor (35 % mais rápido, chutes avançam), Spitfire (jab lança orbe), Colossus (160 % de vida, armadura em todo golpe básico), Phantom (rápido e invulnerável ao chutar).

Captura de evidências: `UME_TOUR=storymode ultimate-mugen --shots <dir> --res 1280x720 --mute`.

## Sobrevivência

`Sobrevivência` no menu principal é uma sequência sem fim contra o elenco inteiro:

- **Oponentes.** Depois de escolher seu lutador, o jogo sorteia o adversário a cada rodada (nunca o chefe oculto, nunca um pacote quebrado). O cenário gira a cada rodada.
- **Escalada.** O nível da CPU sobe a cada duas lutas, partindo da sua dificuldade e com teto 8.
- **A vida continua.** A barra com que você termina uma luta é a mesma com que começa a próxima, mais a recuperação configurada em Jogabilidade › *Vida recuperada (Sobrevivência)* (0 / 20 / 35 / 50 / 100 %). Um KO encerra a sequência.
- **Pontuação.** Cada vitória soma o `Score`/`ScoreTotal` da engine para o seu lado mais um bônus de `1000 × rodada`, então o número é determinístico e vem da própria simulação.
- **Fim de jogo.** A tela final mostra a pontuação, quantas lutas você venceu e o recorde local; o recorde fica salvo em `ultimate-mugen.cfg` (`survival_best`) e um novo recorde é destacado.
- **Replays.** A última luta é arquivada como qualquer outra e pode ser assistida pela tela de fim de jogo (botão R) ou pelo navegador de Replays; o replay guarda a vida inicial para a re-simulação bater exatamente.

Estatísticas de pontuação usadas: `Fighter::score` e `Fighter::score_total`, preenchidas por `HitDef.score` e pelos controladores `ScoreAdd`/`TargetScoreAdd` e lidas pelos triggers `Score`/`ScoreTotal`.

## Cooperativo de 2 jogadores (Arcade e Sobrevivência)

Aperte `V` na tela de seleção (ou clique no botão **CO-OP 2P**) para começar Arcade ou Sobrevivência com dois
jogadores humanos **no mesmo lado**:

- P1 e P2 escolhem um lutador cada; os dois ficam em campo juntos (2 contra 1) usando o suporte a equipe simultânea
  da engine, então pushboxes, mira e câmera normais valem.
- O lado só perde o round/sequência quando **os dois** lutadores caem (regra de KO `team_simul`).
- As entradas são independentes: P1 usa as teclas/controle do jogador 1, P2 as do jogador 2 (sem precisar de `Tab`).
  O HUD marca cada lutador como `P1` / `P2` pelo slot do piloto, não pelo lado.
- O Arcade mantém os dois lutadores pela escada e troca o oponente a cada rodada; a Sobrevivência faz o mesmo a cada onda.
- Replays gravam os dois slots, então uma luta cooperativa se repete exatamente.

Chaves da engine: `Match::team_simul`. Os slots entram intercalados (`P1`, oponente, `P2`), então `roots()[0]` e
`roots()[2]` são o par humano.

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
