# Ultimate MUGEN

Motor de luta 2D **feito do zero**, inspirado no M.U.G.E.N, moderno e multiplataforma (navegador, desktop e **celular**), **compatível com personagens M.U.G.E.N** e com um formato novo, muito mais simples, para criar lutadores.

![Luta](screenshots/fight-native.png)

## Destaques
- **Compatibilidade MUGEN**: lê `.def`, `.sff` (v1 PCX e v2 RLE8/RLE5/LZ5/PNG), `.air`, `.cmd`, `.cns`, `.snd`, `.act` e cenários. Triggers, redirecionamentos, ~90 state controllers, HitDef completo, helpers, projéteis, explods, super pause, after images, paletas. Testado com o Kung Fu Man oficial.
- **Formato UMF** (Ultimate Mugen Format): um personagem = 1 arquivo JSON/TS com golpes declarativos. Hitboxes automáticas a partir do esqueleto, comandos na sintaxe MUGEN, botão "especial simples" no celular. Compila para a mesma VM do MUGEN — personagens UMF e MUGEN lutam entre si.
- **10 lutadores originais** e **6 cenários originais** com parallax e partículas, 100% procedurais (sem assets de terceiros).
- **Mobile de verdade**: analógico virtual, botões, modo simples (SP1/SP2/SP3/SUPER com um toque), vibração, PWA instalável em tela cheia e offline.
- Modos **Arcade, Versus CPU, 2 Jogadores, Treino (hitboxes), CPU vs CPU**; IA que funciona com qualquer personagem (inclusive MUGEN).
- **Importação**: arraste `.zip` ou pastas de personagens/cenários MUGEN; ficam salvos no navegador (IndexedDB).

## Rodando
```bash
npm install
npm run dev        # http://localhost:5173
npm test           # testes (parsers, VM, comandos, elenco, KFM headless se disponível)
npm run build      # gera dist/ (site estático / PWA)
```

### Celular, desktop e lojas
- **Web/PWA**: publique `dist/` em qualquer hospedagem HTTPS → "Adicionar à tela inicial".
- **Android/iOS**: `npx cap init && npx cap add android|ios` usando `dist/` como `webDir` (Capacitor).
- **Windows/macOS/Linux**: `npm create tauri-app` apontando para `dist/` (binário pequeno) ou Electron.

## Controles
| | Teclado | Toque |
|---|---|---|
| Mover | WASD / setas | Analógico esquerdo |
| Socos | U (fraco) · I (forte) · O | LP · HP |
| Chutes | J (fraco) · K (forte) · L | LK · HK |
| Especiais simples | 1 · 2 · 3 · (1+2 = Super) | SP1 · SP2 · SP3 · SUPER |
| Pausa / Hitboxes | Esc / F2 | ❚❚ |

Gamepads padrão (Xbox/PlayStation) funcionam automaticamente.

## Documentação
- [`docs/PESQUISA.md`](docs/PESQUISA.md) — como o MUGEN funciona, sucessores (Ikemen GO), dores e arquitetura escolhida.
- [`docs/research-mugen-en.md`](docs/research-mugen-en.md) — relatório técnico detalhado dos formatos.
- [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md) — como criar um personagem UMF.
- [`docs/ROSTER.md`](docs/ROSTER.md) — elenco original + catálogo de personagens MUGEN populares (Dragon Ball, Bleach, Naruto…) para importar.

## Aviso sobre personagens de terceiros
Personagens de anime/jogos (Goku, Ichigo, Naruto…) são propriedade de seus detentores e **não são distribuídos** aqui. O motor executa os pacotes MUGEN feitos por fãs que o próprio jogador importar.

## Estrutura
```
src/mugen   formatos MUGEN (SFF/AIR/CMD/CNS/SND/ACT/stages)
src/engine  VM de estados, expressões, hit system, match, render, áudio, IA, efeitos
src/native  formato UMF + compilador + rig procedural
src/chars   elenco original      src/stages  cenários originais
src/game    registro/importação/sessão      src/ui  telas, HUD, controles de toque
```
