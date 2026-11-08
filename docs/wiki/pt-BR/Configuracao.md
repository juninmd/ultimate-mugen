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
| `lang` | `pt` / `en` | Idioma das telas de configurações e versus |
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

A interface traz dois idiomas numa tabela de strings por chave (`src/i18n.rs`): **inglês** (padrão) e **português (Brasil)**.
Configurações › Idioma troca em tempo real e grava `lang=en|pt` em `ultimate-mugen.cfg`.

- A busca é idioma atual → inglês → nome da chave, então um catálogo incompleto nunca mostra rótulo vazio.
- Uma chave ausente é impressa uma única vez no log (`i18n: missing key '<key>'`).
- Os catálogos são compilados no binário, então a troca não faz I/O e funciona no web/mobile.
- Um teste mantém os dois catálogos em sincronia e um teste de grep falha se as telas migradas (menu, seleção, resultados, pausa, replays, sobrevivência, controles) voltarem a ter português fixo.

## Controles

| Ação | Teclado P1 | Teclado P2 | Gamepad | Toque |
|---|---|---|---|---|
| Mover | setas | `I` `K` `J` `L` | D-pad / analógico | analógico virtual |
| Socos `x` `y` `z` | `Q` `W` `E` | Num `4` `5` `6` | □ △ R1 | LP HP |
| Chutes `a` `b` `c` | `A` `S` `D` | Num `1` `2` `3` | ✕ ○ R2 | LK HK |
| Especiais rápidos | `1` `2` `3` | Num `7` `8` `9` | L1, L2 | ESPECIAL, SUPER |
| Super | `1`+`2` | Num `7`+`8` | L1+L2 | SUPER |
| Despertar | `T` | Num `0` | R3 / SELECT | — |
| Troca (modo Tag) | `F` | Num Enter | R3 / SELECT | botão de despertar |
| Carregar ki (Start) | `R` | Num Decimal | — | — |
| Pausa / hitboxes | `Esc` / `F2` | `Esc` / `F2` | START | ❚❚ |
| Tela cheia / FPS | `F11` / `F3` | — | — | — |

Os nomes `x y z a b c` são os mesmos usados nos comandos (`input`) dos personagens.

No pulo, `x` `y` `a` `b` lançam um golpe aéreo cada, e baixo + `b` é o chute para baixo (gerado para todo lutador nativo por `native/air_normals.rs`, reaproveitando as animações aéreas do próprio lutador).

## Botões na tela

A lista de golpes da pausa e a tela de Controles mostram a tecla ou o botão real do último dispositivo usado: teclado, Xbox (A/B/X/Y, LB/LT/RB/RT) ou PlayStation (✕ ○ □ △, L1/L2/R1/R2). Em telas de toque aparecem os nomes dos botões virtuais. Controles Xbox são detectados pelo nome ou pelo vendor ID `045E`, PlayStation pelo `054C`. A vibração funciona nos dois: Xbox via force feedback do gilrs (e Gamepad API na web), DualShock 4 / DualSense via relatórios HID diretos.
