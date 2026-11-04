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

## Linha de comando

| Flag | Efeito |
|---|---|
| `--res 2560x1440` | Resolução da janela |
| `--4k` | Janela 3840×2160 |
| `--fullscreen` | Tela cheia |
| `--touch` | Força os controles de toque |
| `--mute` | Sem áudio |
| `--shots <pasta>` | Tour automático salvando capturas |
| `--version` | Mostra a versão |
| `--check-update` / `--no-update-check` | Controla a busca de atualização |

## Variáveis de ambiente

| Variável | Efeito |
|---|---|
| `UME_MUGEN_DIR` | Pasta de conteúdo M.U.G.E.N alternativa |
| `UME_MARKET_HOME` | Pasta dos downloads e do estado do [mercado](Mercado.md) |
| `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` | Token opcional do GitHub para o mercado (aumenta o limite da API) |
| `UME_NO_UPDATE_CHECK=1` | Não procura atualizações |
| `UME_NOFX=1` | Desliga a camada animada dos cenários (teste de desempenho) |
| `UME_TOUR=mugen` + `UME_CHARS=a,b` + `UME_STAGE=<id>` | Com `--shots`, fotografa lutadores importados |

## Controles

| Ação | Teclado P1 | Teclado P2 | Gamepad | Toque |
|---|---|---|---|---|
| Mover | setas | `I` `K` `J` `L` | D-pad / analógico | analógico virtual |
| Socos `x` `y` `z` | `Q` `W` `E` | Num `4` `5` `6` | □ △ R1 | LP HP |
| Chutes `a` `b` `c` | `A` `S` `D` | Num `1` `2` `3` | ✕ ○ R2 | LK HK |
| Especiais rápidos | `1` `2` `3` | Num `7` `8` `9` | L1, L2 | ESPECIAL, SUPER |
| Super | `1`+`2` | Num `7`+`8` | L1+L2 | SUPER |
| Despertar | `T` | Num `0` | R3 / SELECT | — |
| Pausa / hitboxes | `Esc` / `F2` | `Esc` / `F2` | START | ❚❚ |
| Tela cheia / FPS | `F11` / `F3` | — | — | — |

Os nomes `x y z a b c` são os mesmos usados nos comandos (`input`) dos personagens.

## Botões na tela

A lista de golpes da pausa e a tela de Controles mostram a tecla ou o botão real do último dispositivo usado: teclado, Xbox (A/B/X/Y, LB/LT/RB/RT) ou PlayStation (✕ ○ □ △, L1/L2/R1/R2). Em telas de toque aparecem os nomes dos botões virtuais. Controles Xbox são detectados pelo nome ou pelo vendor ID `045E`, PlayStation pelo `054C`. A vibração funciona nos dois: Xbox via force feedback do gilrs (e Gamepad API na web), DualShock 4 / DualSense via relatórios HID diretos.
