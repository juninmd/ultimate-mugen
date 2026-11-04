# Pesquisa de jogos de luta: oportunidades

[English](../Research-Fighting-Games.md) · **Português (BR)**

Esta página compara recursos publicamente conhecidos de lutadores 2D e 2.5D modernos com o estado atual do Ultimate MUGEN e ordena oportunidades concretas. É um documento de planejamento: **nenhum recurso listado aqui está implementado ainda**. Só recursos públicos são citados e nenhum asset de terceiros é copiado.

Escala de esforço: **P** menos de 2 dias, **M** cerca de 1 semana, **G** 2 a 4 semanas, **GG** mais de 1 mês.

## Jogos pesquisados

| Jogo | Conhecido por |
|---|---|
| Dragon Ball FighterZ | Fidelidade ao anime, 3v3 com assists, auto-combo para iniciantes, lobbies ranqueados |
| Sparking! Zero | Elenco grande, destruição de cenário, tutorial longo, regras de batalha personalizáveis |
| Melty Blood: Type Lumina | Estilos Moon, auto-combo, rollback, lobbies |
| Guilty Gear -Strive- | Rollback com lobby de avatares, tutorial e missões, treino com dados de frame |
| BlazBlue Cross Tag Battle | Layout simples de 2 botões, assists, rollback |
| Street Fighter 6 | Controles modernos, HUD do Drive, medidor de frames, replays compartilháveis, Battle Hub, tutorial |
| Mortal Kombat 1 | Kameos, torres, treino com tutorial por personagem |
| Tekken 8 | Modo prática, ghost battle, replays e dicas, guias por personagem |
| Them's Fighterz, Skullgirls | Rollback, treino muito completo, opções para daltônicos, tutorial |
| Persona 4 Arena Ultimax | Hit-stop e supers cinematográficos, apresentação de história |
| The King of Fighters XV | Seleção de time com ordem, 3v3 em sequência, rollback, prática |
| Under Night In-Birth II | Layout compacto de 6 botões, HUD do GRD, rollback, treino profundo |
| Ikemen GO e comunidade M.U.G.E.N | Screenpacks abertos, `fight.def`/`system.def`, netplay com rollback, Lua, opções comuns de treino |

## Estado atual do Ultimate MUGEN

Verificado no repositório em 2026-11-04.

| Área | Hoje |
|---|---|
| Modos | Arcade, Versus CPU, 2 jogadores, Treino, CPU vs CPU, Equipe sequência, Equipe paralelo (`Mode` em `src/app.rs`) |
| Treino | Boneco (`Pilot::Dummy`), barra infinita, hitboxes no menu de pausa |
| Menu de pausa | Continuar, Hitboxes, Reiniciar, Trocar personagens, Configurações, Menu principal |
| HUD | Painéis de vidro, retratos grandes, combo, pulso de vida baixa, medidor MAX, timer urgente (PR #156) |
| Feedback de impacto | Partículas, tremor, hit-stop, câmera lenta no KO, corte de super, aberração cromática, intensidade configurável |
| Configurações | Abas Vídeo, Áudio, Controles, Jogabilidade, Idioma, Sobre |
| Netplay | Rollback GGRS via UDP só por flags (`--netplay`, `--peer`); replays `.umr`; sem lobby no jogo, sem transporte para web e celular |
| Replays | Re-simulação da última luta a partir dos controles gravados; formato de arquivo existe |
| Conteúdo | Mercado via GitHub, importação `.def`/`.zip`, lutadores UMF |
| Tutorial e desafios | Nenhum. Combo trials, sobrevivência e boss rush estão no roadmap (seção 5) |

## Oportunidades priorizadas

Prioridade: **P0** primeiro, **P1** depois, **P2** mais tarde, **P3** desejável.

### P0: maiores lacunas

| # | Oportunidade | Motivo | Esforço | Toca em |
|---|---|---|---|---|
| 1 | **Treino v2**: comportamentos do boneco (em pé, agachado, pulo, bloquear tudo, bloquear após o 1º golpe), gravar e repetir o boneco, tecla de reset com posições salvas, modos de recarga de vida e barra | SF6, Tekken 8, Strive e Skullgirls têm gravação do boneco e resets. Sem isso o treino é só ver hitbox | M | `Pilot::Dummy` em `src/session.rs`, pausa em `src/app.rs`, overlay em `src/hud.rs` |
| 2 | **Histórico de comandos e dados de frame**: entradas rolando com contagem de frames, e startup, ativo, recuperação, vantagem e dano do último golpe | SF6 e Strive mostram isso no treino; é como o jogador aprende matchups. A simulação já é determinística a 60 Hz | M | `src/hud.rs`, `crates/engine` (hooks só leitura), `src/session.rs` |
| 3 | **Tela online no jogo**: criar ou entrar por código, IP direto, atraso e ping antes da luta, revanche | O rollback só existe via flags. Todo lutador de console tem lobby e revanche. Começar com salas por código usando o `matchbox` já decidido | G | `src/netlink.rs`, `crates/netplay`, novo `Screen::Online` |
| 4 | **UI de qualidade de conexão**: gráfico de ping, jitter e frames revertidos, aviso de conexão ruim | Strive e SF6 mostram qualidade da conexão; consta na seção 1 do ROADMAP | P | `src/hud.rs`, `crates/netplay` |
| 5 | **Tutorial guiado**: básico (mover, bloquear, especial, super), depois lista de golpes por personagem com entrada ao vivo | SF6, Tekken 8, Strive e Skullgirls têm tutorial; motores estilo MUGEN normalmente não, então o iniciante fica sem apoio | G | Novo `Screen::Tutorial`, reuso de `src/app/cutscene.rs` |

### P1: alto valor, custo moderado

| # | Oportunidade | Motivo | Esforço | Toca em |
|---|---|---|---|---|
| 6 | **Combo trials** a partir de lista JSON por personagem, com próximo comando na tela | Guilty Gear, SF6, DBFZ e Melty Blood têm; já está no ROADMAP. Criadores UMF podem entregar trials junto do lutador | M | Runner sobre `Session`, esquema em `docs/CHARACTER_FORMAT.md` |
| 7 | **Controle simples (moderno)**: especiais com um botão e auto-combo opcionais por jogador | SF6 Modern, DBFZ, Melty Blood e BlazBlue CTB têm; muito importante para toque | M | `src/controls.rs`, `src/keybinds.rs`, camada de comandos em `crates/engine` |
| 8 | **Passe de acessibilidade**: paletas para daltônicos, reduzir flash e tremor separadamente, texto maior, cores de hitbox com alto contraste, alternar em vez de segurar | Vários jogos têm modo daltônico e flash reduzido. Hoje há só um controle de intensidade | M | `src/settings.rs`, `src/app/settings_screen.rs`, `src/vfx.rs`, `src/hud.rs` |
| 9 | **Visualizador de replays**: lista, velocidade, pausa, frame a frame, câmera, assumir um personagem, salvar e compartilhar `.umr` | SF6, Tekken 8 e Strive permitem navegar e compartilhar replays; formato e re-simulação já são determinísticos | M | `src/app.rs` (`Replay`), `crates/netplay`, novo `Screen::Replays` |
| 10 | **Ranked e fila de partidas** (precisa de servidor): rating, fila anônima, revanche, sequência de vitórias | Padrão em lutadores de console. Exige serviço pequeno; vem depois dos lobbies | GG | Novo serviço e `Screen::Online` |
| 11 | **Seleção de personagem**: ficha com golpes e dificuldade, prévia de stage e de paleta, animação de aleatório, favoritos, filtros por tag e origem | KOF XV e DBFZ mostram paletas e ordem do time; screenpacks do Ikemen GO têm slots e random. Parte pronta em `crates/formats/src/screenpack.rs` | M | `src/app.rs`, `src/menu_art.rs` |
| 12 | **Pausa e resultados**: lista de golpes e comandos, resumo de teclas, revanche, estatísticas (hits, dano, combo máximo, uso de barra) | Lista de golpes na pausa é norma em Tekken, SF6 e MK. A tela de resultados já existe | P | `src/app.rs` pausa e `Screen::Results` |

### P2: profundidade e acabamento

| # | Oportunidade | Motivo | Esforço | Toca em |
|---|---|---|---|---|
| 13 | **Página de opções do treino**: vida infinita ou regenerativa, modos de barra, dano, hit-stun, escala de combo | Existe em praticamente todo modo treino | P | Pausa, `src/hud.rs` |
| 14 | **Sobrevivência, boss rush e torres** com recompensas | MK1 (torres), Sparking! Zero e Tekken; seção 5 do ROADMAP | M | Novos `Mode`, reuso de `Team` |
| 15 | **Arcade com rival, introduções e finais por personagem** via storyboard | Padrão em Tekken, MK e KOF; `Cutscene` já suporta cenas | M | `src/app/cutscene.rs` |
| 16 | **Modo espectador** de partidas com rollback | Muito usado em torneios no Strive e SF6 | G | `crates/netplay`, `Screen::Online` |
| 17 | **Regras por partida**: rounds, tempo, escala de dano, gravidade, handicap | Regras do Sparking! Zero, modificadores do MK, configs da comunidade MUGEN | P | `Rules` (já existe em `Replay`) |
| 18 | **Medidor de frames e legenda de hitboxes** em tempo real | Medidor do SF6 e modos de hitbox do Skullgirls | M | `src/render.rs` |
| 19 | **UX de controle**: teste de latência, teste de botões, perfis por dispositivo, zona morta e SOCD | Necessário para fliperama e controle; teste de latência é comum | P | `src/controls.rs`, `src/gamepad.rs` |
| 20 | **Sensação do menu**: som e animação em cada cursor, repetição ao segurar, paridade mouse e toque, botão voltar no celular | Acabamento visto em todos os jogos | P | `src/ui.rs`, `src/app.rs` |

### P3: ecossistema e longo prazo

| # | Oportunidade | Motivo | Esforço | Toca em |
|---|---|---|---|---|
| 21 | **Suporte a screenpacks** (`fight.def`, `system.def`) desenhados em wgpu | Maior reaproveitamento do trabalho da comunidade; loaders existem e a migração para wgpu já terminou | GG | `crates/formats/src/screenpack.rs`, `src/hud.rs`, `src/menu_art.rs` |
| 22 | **Descoberta no mercado**: notas, tags, capturas, aviso de atualização | Jogos com mods dependem de descoberta; o mercado já existe | M | `crates/market`, `src/app/market.rs` |
| 23 | **Dicas e guias por personagem** escritos pelo criador no UMF | Mesma ideia dos guias do SF6 e Tekken 8 | P | Esquema UMF, tutorial |
| 24 | **Ferramentas de torneio**: chaveamento a partir de `.umr`, gravador, overlay para streamers | Seção 2 do ROADMAP | G | Novo crate |
| 25 | **Mais idiomas** (espanhol, japonês) e rótulos para leitores de tela | Seção 5 do ROADMAP | M | `src/i18n.rs` |

## Sequência sugerida

1. Treino (1, 2, 13), pois facilita validar o resto.
2. UI de rede e tela online (4, 3), depois replays (9).
3. Tutorial e trials (5, 6), depois acessibilidade e controle moderno (8, 7).
4. Seleção e resultados (11, 12), depois modos (14, 15).
5. Ranked e espectador (10, 16) com lobbies estáveis.
6. Screenpacks (21) em trilha separada.

## Observações

- O Ikemen GO é citado apenas como inspiração, como na página inicial.
- As listas refletem informações públicas na data da pesquisa e podem mudar com patches. São referências, não requisitos.
- Ferramentas de histórico e dados de frame (2) só podem ler o estado da luta, para manter o determinismo do rollback.
- Todo campo novo em `Match` deve entrar em `ume_netplay::checksum`.
