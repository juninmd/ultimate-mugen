---
name: ume-media-capture
description: >-
  Guia definitivo e pipeline automatizado para captura de mídia de alta fidelidade
  (screenshots 1280x720 e GIFs animados otimizados com dither Bayer) em ambientes headless Linux,
  Web/Chromium Playwright, automação de combates e evidências visuais de Pull Requests para Ultimate MUGEN.
---

# Manual de Captura de Mídia & Evidências UME: Screenshots HD e GIFs Animados

O motor **Ultimate MUGEN (UME)** exige comprovação visual em alta fidelidade para todas as alterações que afetam gameplay, HUD, balanceamento, shaders, áudio, movimentação e renderização. Em ambientes headless Linux (CI runners, containers de desenvolvimento e servidores sem GPU dedicada ou display X11 físico), a captura deve ser determinística, automatizada e sem artefatos de corte.

Este manual e skill padronizam o fluxo de geração de mídia e evidências visuais para PRs.

---

## 1. Arquitetura de Captura Headless

Em containers Linux headless, o executável nativo pode depender de extensões X11/XKB (`libxkbcommon-x11`). A via mais confiável e uniforme em qualquer SO é o pipeline WebGL2 / WebAssembly via headless Chromium com o backend de software SwiftShader / ANGLE:

- **Resolução Nativa**: `1280 × 720` (16:9 HD standard).
- **Driver de Renderização**: `--use-gl=angle --use-angle=swiftshader --enable-unsafe-swiftshader`.
- **Taxa de Amostragem**: Gravação contínua a 25/30 FPS em WebM via Playwright Context.
- **Sampling de Entrada**: Teclas devem ser sustentadas por pelo menos 2 animation frames (`requestAnimationFrame`) para garantir registro pela VM de 60 Hz.

---

## 2. Ferramenta Automatizada: `scripts/record-gameplay.js`

O repositório fornece o utilitário CLI `scripts/record-gameplay.js` para captura com um único comando:

```bash
# Captura completa com screenshots e GIF animado
node scripts/record-gameplay.js --out docs/evidence/feature-topic --gif gameplay.gif --duration 6
```

### Argumentos Suportados:
| Flag | Padrão | Descrição |
| :--- | :--- | :--- |
| `--out <dir>` | `docs/evidence/recordings` | Diretório de destino para imagens e GIFs |
| `--gif <nome>` | `gameplay.gif` | Nome do arquivo GIF gerado |
| `--duration <s>` | `6` | Segundos de combate simulado gravados |
| `--shots` | `true` | Salva milestones (`01_menu`, `02_select`, `03_stage`, `04_fight`) |
| `--dist <path>` | `web/dist` | Caminho do bundle web compilado |

---

## 3. Otimização de GIFs com FFmpeg (Bayer Dithering)

Para que os GIFs tenham qualidade arcade limpa, sem ruído granular excessivo e com tamanho abaixo de 5 MB (ideal para carregar instantaneamente em Pull Requests do GitHub):

```bash
ffmpeg -y -i input.webm \
  -vf "fps=20,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" \
  output.gif
```

### Por que esta fórmula?
1. **`fps=20`**: Fluidez arcade nítida com 33% menos quadros que 30 FPS.
2. **`scale=480:-1`**: Mantém proporção 16:9 (`480x270`), legível em mobile e desktop.
3. **`palettegen=max_colors=128`**: Limita a tabela para cores dominantes dos sprites e HUD, cortando peso.
4. **`dither=bayer:bayer_scale=3`**: Evita o aspecto granulado do Floyd-Steinberg em superfícies chapadas de pixel-art.

---

## 4. Padrão de Evidências em Pull Requests

Cada PR aberta no Ultimate MUGEN deve seguir a estrutura de evidências:

1. **Diretório de Armazenamento**: `docs/evidence/<feature-slug>/`
2. **Galeria no Corpo da PR**:
```markdown
### 📸 Galeria de Evidências Visuais

| Tela Inicial / Menu | Seleção de Lutadores | Cenário & HUD Inicial |
|:---:|:---:|:---:|
| ![Menu](https://raw.githubusercontent.com/juninmd/ultimate-mugen/<branch>/docs/evidence/<topic>/01_menu.png) | ![Select](https://raw.githubusercontent.com/juninmd/ultimate-mugen/<branch>/docs/evidence/<topic>/02_select.png) | ![Stage](https://raw.githubusercontent.com/juninmd/ultimate-mugen/<branch>/docs/evidence/<topic>/03_stage.png) |

### 🎬 Gameplay em Ação (GIF 60Hz)

![Gameplay](https://raw.githubusercontent.com/juninmd/ultimate-mugen/<branch>/docs/evidence/<topic>/gameplay.gif)
```

---

## 5. Checklist de Verificação antes de Submeter
- [ ] O bundle web está atualizado (`./scripts/build-web.sh`).
- [ ] O teste smoke web passou (`node scripts/web-smoke.js web/dist`).
- [ ] As capturas PNG possuem dimensão 1280x720 e não exibem tela preta.
- [ ] O GIF animado pesa menos de 6 MB e exibe movimentação real com HUD visível.
- [ ] As mídias estão commitadas na branch sob `docs/evidence/<topic>/`.
