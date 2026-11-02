# Pesquisa: como o M.U.G.E.N funciona hoje e como modernizar

> Relatório técnico completo (em inglês, com detalhes de formato byte a byte): [`research-mugen-en.md`](research-mugen-en.md).

## 1. O M.U.G.E.N clássico (Elecbyte, 1999–2013)

Um personagem é uma pasta com arquivos texto/binários:

| Arquivo | Função | Implementado aqui |
|---|---|---|
| `.def` | Manifesto: nome, autor, `localcoord`, lista de arquivos, paletas | `crates/engine/src/loader.rs` |
| `.sff` | Sprites. **v1**: PCX 8-bit RLE + paleta compartilhada. **v2**: RLE8, RLE5, LZ5, PNG8/24/32 + tabela de paletas | `crates/formats/src/sff.rs`, `pcx.rs`, `png.rs` |
| `.air` | Animações: quadros, tempos, flip, blend, **Clsn1** (golpe) / **Clsn2** (corpo), `Loopstart` | `crates/formats/src/air.rs` |
| `.cmd` | Comandos (`~D, DF, F, x`, `/`, `$`, `>`, `+`, cargas `~30a`) + `[Statedef -1]` | `crates/engine/src/commands.rs`, `cns.rs` |
| `.cns` | Constantes + máquina de estados: `Statedef`, `triggerN`, ~100 *state controllers* | `crates/engine/src/fighter.rs`, `controllers.rs`, `triggers.rs`, `matchs.rs` |
| `.snd` | Sons WAV indexados por grupo/número | `crates/formats/src/snd.rs` + `src/audio.rs` |
| `.act` | Paletas alternativas (ordem invertida) | `crates/formats/src/act.rs` |
| `stages/*.def` | Cenários com camadas parallax, tiles, animação | `crates/formats/src/stage.rs` |

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

## 4. Arquitetura escolhida (Rust)

```
crates/formats → decodificadores/parsers MUGEN (SFF v1/v2, AIR, CMD/CNS INI, SND, ACT, PCX, PNG, zip, stages) — Rust puro, compila para WASM
crates/engine  → VM única: expressões MUGEN (AST avaliado), Fighter, ~90 controllers, HitDef, Match (rounds, colisão,
                 projéteis, helpers, explods, câmera), common states próprios, IA, efeitos, UMF (compilador + rig tiny-skia)
src/           → app macroquad: telas, render (blend aditivo), HUD, áudio sintetizado, DualShock/DualSense (vibração + lightbar), toque
```

**Por que uma VM só?** Personagens UMF são compilados para os mesmos `StateDef`/controllers do MUGEN. Um personagem UMF luta contra um MUGEN importado com as mesmas regras de colisão, guarda, hitstop, combos e física.

**Multiplataforma**: Rust + macroquad (OpenGL/GLES/WebGL) gera binário nativo para Windows/Linux/macOS, WASM (~3 MB) para navegador/PWA e APK/IPA para Android/iOS a partir do mesmo código. **Controles**: gilrs para entrada; vibração do DualShock 4/DualSense por relatórios HID (USB 0x05/0x02, Bluetooth 0x11/0x31 com CRC32), com fallback de force-feedback do gilrs e `vibrationActuator` na web. **Determinismo**: passo fixo 60 Hz e PRNG próprio — base para replays e rollback netcode (próximo passo, via WebRTC/UDP).

## 5. Personagens de anime/jogos

Personagens de Bleach, Dragon Ball, Naruto etc. são propriedade intelectual de terceiros — **não são distribuídos** com o jogo. O motor é compatível com os personagens MUGEN feitos por fãs: o jogador importa os que já possui. A lista dos mais populares (e seus autores conhecidos) está em [`ROSTER.md`](ROSTER.md).
