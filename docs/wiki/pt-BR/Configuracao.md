# Configuração e controles

## Arquivo `ultimate-mugen.cfg`

Fica ao lado do executável e é gravado pelo menu **Configurações** (menu principal ou menu de pausa: Vídeo, Áudio, Controles, Jogabilidade, Idioma, Sobre). Pode ser editado à mão. Na web o mesmo texto fica no `localStorage`, no Android no armazenamento privado do app.

| Chave | Valores | Efeito |
|---|---|---|
| `fullscreen` | `0` / `1` | Tela cheia |
| `res` | `0` 1280×720, `1` 1920×1080, `2` 2560×1440, `3` 3840×2160, `4` 5120×2880 | Tamanho da janela |
| `quality` | `0` automática, `1` 3×, `2` 4,5×, `3` 6× | Densidade da arte vetorial |
| `fps` | `0` / `1` | Mostra FPS |
| `fx` | `0` a `2` | Pós-processamento (bloom, vinheta etc.) |
| `impact_lv` | `0` mínimo (fotossensibilidade), `1` reduzido, `2` normal, `3` intenso | Força dos efeitos de impacto |
| `version` | `2` | Versão do formato (arquivos antigos sem ela continuam válidos) |
| `vsync` | `0` / `1` | V-Sync (nativo) |
| `fps_cap` | `0` sem limite, `1` 30, `2` 60, `3` 120, `4` 144 | Limitador de quadros (nativo) |
| `master` `music` `sfx` | `0` a `10` | Níveis de volume |
| `mute` | `0` / `1` | Silencia tudo |
| `difficulty` | `1` a `8` | Nível da CPU |
| `round_time` | `0` 99 s, `1` 60 s, `2` 30 s, `3` infinito | Tempo do round |
| `rounds` | `0` primeiro a 1, `1` primeiro a 2, `2` primeiro a 3 | Rounds para vencer |
| `life` | `0` 50%, `1` 75%, `2` 100%, `3` 150%, `4` 200% | Multiplicador da barra de vida |
| `lang` | `en` / `pt` / `es` | Idioma de toda a interface |
| `keep_aspect` | `1` (padrão) / `0` | `1` coloca barras pretas e mantém a imagem 16:9 em qualquer janela; `0` estica a imagem até preencher a janela |
| `filter` | `nearest` (padrão) / `linear` | Amostragem dos sprites ao ampliar: pixels nítidos ou suavização bilinear |
| `window` | `LARGURAxALTURA`, `0x0` sem valor | Tamanho da janela, gravado automaticamente cerca de um segundo depois de parar de redimensionar; vale mais que `res` até você escolher um tamanho no menu Vídeo |

## Janela redimensionável

A janela pode ser arrastada para qualquer tamanho. O jogo desenha em um espaço lógico fixo 16:9 que é escalado para a janela, então a área de luta, o HUD e os menus ficam consistentes em todos os tamanhos e o tamanho da janela é independente da resolução do jogo. Com **Manter proporção** ligado, a sobra fica preta; desligado, a imagem é esticada e o mouse e o toque são mapeados de volta ao espaço esticado. A flag `--res LxA` continua definindo o tamanho inicial e tem prioridade sobre o valor salvo em `window`.

## Linha de comando

| Flag | Efeito |
|---|---|
| `--res 2560x1440` | Resolução da janela |
| `--4k` | Janela 3840×2160 |
| `--fullscreen` | Tela cheia |
| `--touch` | Força os controles de toque |
| `--mute` | Sem áudio |
| `--shots <pasta>` | Tour automático salvando capturas |
| `--shots-audio` | Ativa som durante o tour de capturas; `--mute` continua tendo prioridade |
| `--version` | Mostra a versão |
| `--check-update` | Mostra se existe release mais nova e sai |
| `--update` | Baixa, verifica e instala a release mais nova, depois sai |
| `--no-auto-update` / `--no-update-check` | Mantém só o aviso / não procura atualização |

## Variáveis de ambiente

| Variável | Efeito |
|---|---|
| `UME_MUGEN_DIR` | Pasta de conteúdo M.U.G.E.N alternativa |
| `UME_MARKET_HOME` | Pasta dos downloads e do estado do [mercado](Mercado.md) |
| `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` | Token opcional do GitHub para o mercado (aumenta o limite da API) |
| `UME_NO_UPDATE_CHECK=1` | Não procura atualizações |
| `UME_NO_AUTO_UPDATE=1` | Mantém o aviso, mas nunca baixa nem instala |
| `UME_NOFX=1` | Desliga a camada animada dos cenários (teste de desempenho) |
| `UME_TOUR=mugen` + `UME_CHARS=a,b` + `UME_STAGE=<id>` | Com `--shots`, fotografa lutadores importados |

## Idioma e localização

A interface traz três idiomas: **inglês** (padrão), **português (Brasil)** e **espanhol**. Configurações › Idioma alterna em tempo real e grava `lang=en|pt|es` em `ultimate-mugen.cfg`.

- `src/i18n/catalog.rs` guarda a tabela por chave (inglês e português) e `src/i18n/es_catalog.rs` a coluna em espanhol; `i18n::t(chave)` procura idioma atual → inglês → nome da chave.
- Textos avulsos usam `i18n::tr3(pt, en, es)`; o antigo `i18n::tr(pt, en)` acha o espanhol em `src/i18n/es_phrases.rs`, indexado pelo texto em inglês. Marcadores usam `i18n::fill("{nome}", &[("nome", valor)])`.
- Uma chave ausente é impressa uma única vez no log (`i18n: missing key '<key>'`).
- Ainda só em português: cutscenes e diálogos do modo história, biografias dos lutadores e nomes dos cenários (dados de conteúdo).
- No primeiro uso (sem configuração salva) o idioma do sistema é escolhido: `navigator.language` na web, `LC_ALL`/`LC_MESSAGES`/`LANG` no desktop; inglês quando não houver suporte.

## Importar conteúdo M.U.G.E.N

Menu principal › **Importar M.U.G.E.N** é o único ponto de entrada. Na web o botão abre o seletor de arquivos do navegador (a página sobrepõe um `<label>` invisível exatamente sobre o botão, para o clique ser um gesto real do usuário) e também dá para soltar `.zip` na janela. No desktop o botão abre o diálogo de arquivos do sistema (PowerShell no Windows, `osascript` no macOS, `zenity` ou `kdialog` no Linux); pastas em `chars/` e `stages/` continuam sendo lidas ao iniciar.

## Controles

| Ação | Teclado P1 | Teclado P2 | Gamepad | Toque |
|---|---|---|---|---|
| Mover | setas | `I` `K` `J` `L` | D-pad / analógico | analógico virtual |
| Socos `x` `y` `z` | `Q` `W` `E` | Num `4` `5` `6` | □ △ | LP HP |
| Chutes `a` `b` `c` | `A` `S` `D` | Num `1` `2` `3` | ✕ ○ | LK HK |
| Especiais rápidos | `V` `B` `N` | Num `7` `8` `9` | — | ESPECIAL, SUPER |
| Atalhos de golpe (slots 1-4) | `1` `2` `3` `4` | — | L1, L2, R1, R2 | — |
| Despertar (transformar) | `T` | Num `0` | L3 + R3 / SELECT | — |
| Troca (modo Tag) | `F` | Num Enter | L3 + R3 / SELECT | botão de despertar |
| Carregar ki | `R` | Num Decimal | R3 (sozinho) / START | — |
| Pausa / hitboxes | `Esc` / `F2` | `Esc` / `F2` | START | ❚❚ |
| Tela cheia / FPS | `F11` / `F3` | — | — | — |

Os nomes `x y z a b c` são os mesmos usados nos comandos (`input`) dos personagens.

No pulo, `x` `y` `a` `b` lançam um golpe aéreo cada, e baixo + `b` é o chute para baixo (gerado para todo lutador nativo por `native/air_normals.rs`, reaproveitando as animações aéreas do próprio lutador).

## Atalhos de golpe

No menu de pausa, escolha um golpe na lista e aperte `1`-`4` (ou L1, L2, R1, R2 no gamepad) para ligá-lo àquele slot. Apertar o mesmo slot de novo limpa o atalho. Depois disso, o golpe sai com um único toque, com o movimento e os botões certos executados por você. As ligações ficam por lutador em `ultimate-mugen.shortcuts`, ao lado do executável.

## Botões na tela

A lista de golpes da pausa e a tela de Controles mostram a tecla ou o botão real do último dispositivo usado: teclado, Xbox (A/B/X/Y, LB/LT/RB/RT) ou PlayStation (✕ ○ □ △, L1/L2/R1/R2). Em telas de toque aparecem os nomes dos botões virtuais. Controles Xbox são detectados pelo nome ou pelo vendor ID `045E`, PlayStation pelo `054C`. A vibração funciona nos dois: Xbox via force feedback do gilrs (e Gamepad API na web), DualShock 4 / DualSense via relatórios HID diretos.

## Tweaks (`tweaks.toml`)

Criado ao lado do executável na primeira execução (localStorage na web) com os padrões. Edite e salve: o jogo recarrega em cerca de um segundo e mostra um aviso. Erros nunca param o jogo; o valor afetado mantém o padrão e o aviso e o log citam a chave, com dica "você quis dizer" e a linha em erros de sintaxe.

| Seção | Chave | Efeito |
|---|---|---|
| `[theme]` | `background` | Cor base do fundo do menu |
| `[theme]` | `glow_bottom`, `glow_left`, `glow_right` | Brilhos de luz do menu |
| `[hud]` | `p1_tint`, `p2_tint` | Cor de destaque das barras por lado |
| `[hud]` | `bar_offset_y` | Desce as barras de vida (-4 a 40) |
| `[rules]` | `rounds_to_win` (1-9), `round_seconds` (0 = sem limite, até 999), `life_percent` (10-500), `game_speed` (0.5-2.0) | Sobrescrevem os valores do menu Settings em toda luta; remova a linha para voltar ao valor do menu |
| `[menu]` | `arcade`, `story`, `survival`, `cpu`, `versus`, `training`, `watch`, `team_seq`, `team_simul`, `team_tag`, `mugen`, `market`, `controls`, `settings`, `replays`, `quit` | Texto dessa entrada do menu principal |

Cores são `"#rrggbb"` ou `"#rgb"`. Plano e comparação com outras engines: `docs/research/customization-comparison.pt-BR.md`.
