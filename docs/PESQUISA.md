# Pesquisa: como o M.U.G.E.N funciona hoje e como modernizar

> Relatório técnico completo (em inglês, com detalhes de formato byte a byte): [`research-mugen-en.md`](research-mugen-en.md).

## 1. O M.U.G.E.N clássico (Elecbyte, 1999–2013)

Um personagem é uma pasta com arquivos texto/binários:

| Arquivo | Função | Implementado aqui |
|---|---|---|
| `.def` | Manifesto: nome, autor, `localcoord`, lista de arquivos, paletas | `src/mugen/loader.ts` |
| `.sff` | Sprites. **v1**: PCX 8-bit RLE + paleta compartilhada. **v2**: RLE8, RLE5, LZ5, PNG8/24/32 + tabela de paletas | `src/mugen/sff.ts`, `pcx.ts` |
| `.air` | Animações: quadros, tempos, flip, blend, **Clsn1** (golpe) / **Clsn2** (corpo), `Loopstart` | `src/mugen/air.ts` |
| `.cmd` | Comandos (`~D, DF, F, x`, `/`, `$`, `>`, `+`, cargas `~30a`) + `[Statedef -1]` | `src/engine/commands.ts`, `src/mugen/cns.ts` |
| `.cns` | Constantes + máquina de estados: `Statedef`, `triggerN`, ~100 *state controllers* | `src/engine/fighter.ts`, `controllers.ts`, `triggers.ts` |
| `.snd` | Sons WAV indexados por grupo/número | `src/mugen/snd.ts` + `src/engine/audio.ts` |
| `.act` | Paletas alternativas (ordem invertida) | `src/mugen/act.ts` |
| `stages/*.def` | Cenários com camadas parallax, tiles, animação | `loadMugenStage` |

**Loop do motor** (60 ticks/s fixos): input → buffers de comando → para cada jogador/helper roda os estados −3, −2, −1 e o estado atual (um `ChangeState` reinicia a avaliação no mesmo tick) → física (`S/C/A/N`) → animação → empurrão entre jogadores e limites de tela → **detecção de golpe Clsn1×Clsn2** (guarda, hitpause, dano, estado 5000+) → projéteis/explods → câmera → render.

**Linguagem de triggers**: expressões com int/float (divisão inteira!), intervalos `[a,b)`, redirecionamento (`p2, life`, `helper(5), var(1)`), comparações especiais (`command = "x"`, `animelem = 3, >= 2`, `hitdefattr = SC, NA`). Aqui ela é **compilada uma única vez para closures** — nada de interpretar string a cada tick.

## 2. Sucessores modernos

* **Ikemen GO** (Go, MIT): sucessor de fato. Lua para menus, ZSS, estágios 3D glTF, Vulkan, **rollback netcode** (padrão na 1.0), builds Android/iOS. Limitações: sem build web, sem UX de toque, criação de personagem ainda é editar texto + Fighter Factory.
* **Web**: só existem renderizadores de animação abandonados (roundonejs, 2021) e um experimento WASM quebrado. **Não existe motor MUGEN completo no navegador** — é exatamente a lacuna que este projeto ocupa.

## 3. Dores do MUGEN e o "ideal moderno"

| Dor | Como resolvemos |
|---|---|
| DSLs sem tipo, erros silenciosos, números mágicos (estado 5000, var(17)) | Formato **UMF** (JSON/TS tipado): golpes declarativos com nomes; o compilador gera os estados |
| Hitboxes desenhadas à mão, coordenada por coordenada | **Hitboxes automáticas** a partir do esqueleto (`hit: { part: "handR" }`) |
| Ripar sprites, alinhar eixos, paletas indexadas | Arte **procedural por esqueleto** (vetorial, nítida em qualquer resolução) ou spritesheet PNG; paletas alternativas por objeto |
| Só desktop Windows, sem toque | **Web/PWA** (desktop, celular, tablet), controles de toque com **modo simples** (especial com 1 botão), gamepad |
| Distribuição por ZIP em fórum | Importação de `.zip`/pasta direto no navegador, salvo em IndexedDB; funciona offline |
| Sem netplay | Simulação determinística em passo fixo e separada do render (base para rollback via WebRTC — próximo passo) |

## 4. Arquitetura escolhida

```
src/mugen/   → decodificadores e parsers dos formatos MUGEN (SFF v1/v2, AIR, CMD, CNS, SND, ACT, stages)
src/engine/  → VM única: expressões, Fighter (estados), controllers, HitDef, Match (rounds, colisão,
               projéteis, helpers, explods, câmera), common states próprios, render Canvas, áudio, IA
src/native/  → formato UMF + compilador UMF → mesmos StateDefs que um personagem MUGEN
src/chars/   → elenco original (10 lutadores) em UMF
src/stages/  → 6 cenários originais procedurais com parallax e partículas
src/game/    → registro de personagens, importação, sessão de luta
src/ui/      → telas (título, menu, seleção, VS, luta, pausa, resultado, importação), HUD, toque
```

**Por que uma VM só?** Personagens nativos são compilados para os mesmos `StateDef`/controllers do MUGEN. Assim um personagem UMF luta contra um personagem MUGEN importado com as mesmas regras de colisão, guarda, hitstop, combos e física — e toda melhoria no motor beneficia os dois.

**Multiplataforma**: TypeScript + Canvas 2D, sem dependências pesadas (só `fflate` para zip). Roda em qualquer navegador; instala como PWA (tela cheia, paisagem, offline). Para lojas: empacotar o `dist/` com **Capacitor** (Android/iOS) ou **Tauri** (Windows/macOS/Linux).

## 5. Personagens de anime/jogos

Personagens de Bleach, Dragon Ball, Naruto etc. são propriedade intelectual de terceiros — **não são distribuídos** com o jogo. O motor é compatível com os personagens MUGEN feitos por fãs: o jogador importa os que já possui. A lista dos mais populares (e seus autores conhecidos) está em [`ROSTER.md`](ROSTER.md).
