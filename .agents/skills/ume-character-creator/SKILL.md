---
name: ume-character-creator
description: >-
  Guia definitivo, runbook e manual técnico para criação de personagens nativos UMF 2.0
  para o motor Ultimate MUGEN (UME). Cobre arquivo único JSON declarativo, renderização
  somente por sprites Pixel-Art (PNG), voo parametrizável, teletransporte, super velocidade,
  multi-transformações (parciais ou corpo todo, troca de habilidades, troca de personagem),
  exibição automática de consumo de super barra no menu de comandos e bateria de testes.
---

# Manual de Criação de Personagens UME: UMF 2.0 & Recursos Core Avançados

O formato **UMF** (*Ultimate Mugen Format*) é o padrão moderno de personagens do motor **Ultimate MUGEN (UME)**. Diferente do ecossistema clássico M.U.G.E.N (que exige 6 ou mais arquivos legados como `.def`, `.sff`, `.air`, `.cmd`, `.cns` e `.snd`), o padrão UME consolida toda a identidade visual, física, lógica de combate, inteligência artificial e habilidades especiais em um **único manifesto JSON declarativo** localizado em `assets/chars/<id>.json`.

---

## 1. Diferencial Arquitetural: UME vs MUGEN Legado

| Recurso | MUGEN Clássico (6 arquivos) | UME Nativo (UMF 2.0 JSON) |
| :--- | :--- | :--- |
| **Arquitetura de Arquivos** | Fragmentada (`.def`, `.sff`, `.air`, `.cns`, `.cmd`, `.snd`) | Arquivo único JSON (`assets/chars/<id>.json`) |
| **Renderização** | SFFv1 (PCX 8-bit) ou SFFv2 (PNG) | Sprites PNG em pixel art (sem arte procedural) |
| **Paletas & Cores** | Arquivos binários `.act` (256 cores) | Hexadecimais declarativos com shading shader em tempo real |
| **Voo (`Fly`)** | Código manual extenso em CNS (`physics = N` com VelSet) | Declarativo: `abilities.fly` (speed, cost contínuo, input) |
| **Teletransporte** | PosAdd e Turn amarrados a triggers complexos | Declarativo: `abilities.teleport` (cost, invuln, input) |
| **Super Velocidade** | Variáveis manuais e PosAdd duplicados | Declarativo: `abilities.superSpeed` (mul, cost contínuo, afterimage) |
| **Transformações** | Helpers complexos ou troca de sprites manual | Declarativo: `abilities.forms` (multi-estágios, buffs, corpo/partes, swap de golpes ou lutador) |
| **Menu de Comandos** | Texto estático ou ausente | Exibição automática do custo em barras (`1 BAR`, `X PWR`, `X/s PWR`) |
| **Hot-Reload & Build** | Reinicialização manual externa | Embutido via `build.rs` ou carregado via `add_native_json` |

---

> UMF 2 (ADR 0007): `format` é `umf/2`, quadros só de pose usam `"pose*ticks"`, e `node scripts/umf2-migrate.mjs` grava o layout canônico antes do commit. O piso de qualidade fica em `native_quality_bar`.

## 2. Visão Geral da Anatomia de um Personagem UMF 2.0

```json
{
  "format": "umf/2",
  "id": "meu_lutador",
  "name": "Meu Lutador",
  "series": "Minha Série",
  "author": "Criador",
  "description": "Texto exibido no seletor de personagens.",
  "color": "#1ca3ec",
  "winQuote": "Vitória incontestável.",
  "intro": "A batalha começa agora.",
  "taunt": "Você pode fazer melhor que isso.",
  "form": "Despertar Supremo",
  "boss": false,
  "speed": 1.0,
  "stats": {
    "life": 1000,
    "power": 3000,
    "attack": 100,
    "defense": 100,
    "walk": [2.6, 2.3],
    "run": [5.6, 4.8],
    "jump": [-8.5, -2.6, 0, 2.8]
  },
  "abilities": {
    "fly": { "speed": 4.0, "cost": 60.0, "input": "a+b" },
    "teleport": { "cost": 300.0, "input": "b+c" },
    "superSpeed": { "mul": 2.0, "cost": 120.0, "input": "a+c" },
    "forms": []
  },
  "art": { "type": "sprites", "style": {}, "poses": {} },
  "palettes": [],
  "awakeningPalette": { "hair": "#ffe600" },
  "anims": {},
  "moves": []
}
```

---

## 3. Especificação das Novas Features Core

### 3.1 Voo Parametrizável (`abilities.fly`)
Permite ao personagem levitar e controlar o voo em qualquer direção no ar e no chão.
- `speed` (`f32`): Velocidade de movimentação livre direcional (padrão `3.5`).
- `cost` (`f32`): Consumo contínuo de barra de energia por segundo (calculado como `cost / 60` por tick).
- `input` (`string`): Tecla ou combinação para ligar/desligar (padrão `"a+b"`).
- `anim` (`string`, opcional): Animação customizada enquanto voa (fallback para `jumpUp`).

### 3.2 Habilidade de Teletransporte (`abilities.teleport`)
Blink espacial instantâneo com invulnerabilidade que posiciona o personagem nas costas do adversário (`P2Dist X + 40`).
- `cost` (`f32`): Barra de energia consumida imediatamente no disparo (ex: `300.0`).
- `input` (`string`): Comando de acionamento (padrão `"b+c"`).
- `anim` (`string`, opcional): Animação de desmaterialização (fallback para `idle`).

### 3.3 Super Velocidade (`abilities.superSpeed`)
Aceleração cinemática com rastro de AfterImage holográfico ativado por toggle.
- `mul` (`f32`): Multiplicador de velocidade de movimentação (ex: `2.2` = +120% de aceleração).
- `cost` (`f32`): Consumo contínuo de barra por segundo enquanto ativa.
- `input` (`string`): Comando de ativação (padrão `"a+c"`).

### 3.4 Multi-Transformações (`abilities.forms[]`)
O UME suporta múltiplos estágios de transformação, que podem alterar partes do corpo, o corpo todo, os golpes ou metamorfosear o personagem em outro.

#### A. Alteração de Partes vs Corpo Todo:
- **Alteração de Partes**: Defina `parts: ["hair", "eye"]` acompanhado de `palette`. Apenas os elementos listados serão repintados/alterados em tempo de execução.
- **Alteração do Corpo Todo**: Omita o campo `parts`. Todas as chaves da paleta fornecida substituirão o esquema de cores global do personagem.

#### B. Troca de Habilidades (Como 2 Personagens em 1):
- Cada forma pode definir sua própria lista de `moves: [...]`.
- `replace_moves: true`: Desativa os golpes da forma base e habilita exclusivamente os novos golpes da forma.
- `replace_moves: false`: Adiciona os novos golpes ao arsenal existente do personagem.

#### C. Transformação em Outro Personagem:
- Defina `character: "outro_personagem_id"`.
- Em tempo de transformação, o motor carrega e compila o `Arc<CharDef>` do personagem alvo, transferindo vida proporcional, ataques, animações e sprites completos para o lutador.

### 3.5 Mecânicas Exóticas e Expressividade Total (`helpers`, `throw` cinemático, `bossRules`, `scripts`)
- **Stand Estilo JoJo (`helpers.stand`)**: Helper com hitbox/hurtbox própria (`keyctrl = 1`), `input` para invocar/recolher, `autoSummon`, `drain` por segundo, `bind` ou perseguição suave, `hittable` + `lifeTransfer` (dano transferido ao dono via `ModifyPlayer`) e `moves[]` acionados por `input` independente ou sincronizados com `triggerMove`. Estados reservados `7000..7049`.
- **Clones Autônomos / *Kage Bunshin* (`helpers.clones`)**: Spawna até `maxActive` clones com IA própria (`walkSpeed`, `attackRange`, perseguição, seleção aleatória de `attacks[]`, `oneHitPoof` via `HitOverride`, `duration` e liberdade de câmera). Estados reservados `7100..7150`.
- **`throw` Cinemático**: Suporta `p2Anim` (`ChangeAnim2` com anims do atacante e `id` fixável em `anims.<nome>.id`), `cameraFree` (`ScreenBound movecamera = 0, 0`), `camera` (`zoom`, `time`, `lag`, `freeze`), sequência `steps[]` (`at`, `bind`, `p2Anim`, `p2State`, `damage`, `shake`, `facing`, `launch`) e failsafe automático `!GetHitVar(IsBound)`.
- **Chefes que Quebram a UI (`bossRules`)**: `hideHud`, `hideLifeBar`, `hidePowerBar`, `hideTimer`, `freezeTimer`, `asserts[]`, `texts[]` (renderizados direto no HUD via `Text`), `modifyPlayer[]` (dono ou `"enemy"`) e `phases[]` (`lifeBelowRatio`, `when`, `once`, `attackMul`, `defenceMul`).
- **ZSS / CNS Inline no JSON (`scripts`)**: Campos `zss`, `cns`, `commands[]` e `override`. Suporta `[StateDef -4]` (executado para todo jogador e helper inclusive durante `SuperPause` com `ignorehitpause = 1`), atribuições ZSS `map(chave) := valor` / `teamMap(chave) += valor` / `rootMap` / `parentMap` e macros `@anim(nome)` e `@move(id)`.

### 3.6 FX Declarativo, Feixes de Ki, Hit-Confirm, Carga Customizável, Perfil UME e Kits Reutilizáveis
- **Carga de Ki Customizável (`abilities.charge`)**: `input` (padrão `"/s"`), `anim`, `auraColor` (`[r, g, b]`), `shake` (`bool`), `sound` (`[g, i]`) e `explod` (nome da anim de aura).
- **Voo Direcional (`abilities.fly`)**: aceita `upAnim`, `downAnim`, `fwdAnim`, `backAnim` e `sound`.
- **Teleporte Parametrizável e Multi-Etapa (`abilities.teleport` e `moves[].teleport`)**: em `abilities.teleport` aceita `at` (quadro do blink quando há anim própria), `offset`, `invuln`, `turn` e `sound`; em `moves[].teleport` aceita número simples (`3`), objeto `{ "at", "mode": "behind" | "front" | "above" | "offset", "offset": [x, y], "turn" }` ou **lista de etapas** `[step1, step2]` (ex.: subir para lançar ataque aéreo e teleportar de volta ao chão). Golpes com teleporte que terminam com `Pos Y < 0` caem naturalmente via estado `50`.
- **FX por Golpe (`crates/native/src/move_fx.rs`)**: `explods[]` (`at`, `anim`, `pos`, `postype`, `bind`, `removeTime`, `priority`, `scale`, `superMove`), `afterimage` (`at`, `time`, `length`, `framegap`, `trans`, `palBright`, `palContrast`, `palAdd`, `palMul`), `shake` (`at`, `time`, `ampl`, `freq`), `flash` (`at`, `time`, `color`, `under`, `superMove`) e `superPause` (`at`, `time`, `pos`, `anim`, `darken`).
- **Hit-Confirm (`moves[].followUp`)**: `{ "to": "<move_id>", "on": "hit" | "contact" | "end", "at": frame? }` — transita para o golpe cinemático somente quando conecta.
- **Feixes Multi-Hit (`projectile`)**: além de `hits` e `removeTime`, suporta `missTime`, `guardDamage`, `hitSound`, `pausetime`, `hitflag` (`"MAFP"`), `shake` e `accel`.
- **Perfil Rico UME (`profile`) e Falas Fiéis**: preencha `profile` com `country`, `nationality`, `origin`, `bio`, `title`, `fightingStyle`, `affiliation` (todos `LocalText` `en`/`pt`/`es`), `element` (elemento principal), `weakness` (fraqueza), `alignment` (`"hero"`, `"villain"`, `"anti-hero"` ou `"neutral"`) e `tags`, além de `intro`, `taunt`, `winQuote` e `description` fiéis ao personagem.
- **Kits Nativos Reutilizáveis (`extends`)**: herde módulos prontos via `"extends": ["core", "ki_fighter", "clone_arts", "aerial_rush"]` (`crates/native/src/kits/`) e sobrescreva apenas o que mudar, reduzindo dezenas de linhas de JSON por lutador.

---

## 4. Exibição de Consumo de Energia no Menu de Comandos

O painel de pausa do Ultimate MUGEN lê dinamicamente o atributo `cost` e `drain` de cada golpe em `def.movelist` e renderiza no canto direito do menu:
- Golpes sem custo (`cost == 0`): Não poluem a interface.
- Super Golpes (`cost == 1000`, `2000`): Exibem `1 BAR`, `2 BARS`.
- Golpes EX / Habilidades com custo fixo (`cost < 1000`): Exibem `{X} PWR`.
- Habilidades com dreno contínuo (`drain: true`): Exibem `{X}/s PWR`.

---

## 5. Fluxo de Criação Passo a Passo de um Lutador UME

1. Crie o arquivo `assets/chars/<seu_id>.json` copiando o modelo de `examples/exemplo_lutador_completo.json`.
2. Configure a identidade (`id`, `name`, `series`, `author`, `color`).
3. Adicione as habilidades core desejadas dentro do bloco `"abilities"`.
4. Defina os golpes em `"moves"`, especificando `"cost"` para golpes que consomem barra.
5. Registre o `<seu_id>` em `assets/chars/index.json` caso queira determinar a posição exata na tela de seleção.
6. Valide a compilação e execute a suíte de testes do motor:
   ```bash
   cargo test -p ume-engine --test native_abilities_and_forms
   ```
7. Inicie o jogo para teste prático em modo treino (`F2` ativa hurtboxes e hitboxes visíveis).

---

## 6. Documentos de Referência e Exemplos

Consulte os arquivos complementares incluídos nesta skill:
- [Exemplo Completo com Todas as Features](examples/exemplo_lutador_completo.json): Arquivo UMF 2.0 funcional com voo, teleporte, super velocidade, multi-transformação e troca de personagem.
- [Manual de Especificação UMF 2.0](references/manual_umf_especificacao_completa.md): Referência técnica exaustiva de cada propriedade do manifesto JSON.
- [Guia de Sprites e Pixel Art](references/guia_sprites_e_pixel.md): Runbook detalhado de sprites PNG e integração de spritesheets.

Para gerar ou corrigir sprites pixel art, primeiro inspecione todas as sprites existentes do próprio personagem e use uma pose aprovada como referência explícita de identidade e acabamento; não presuma que `walk0.png` seja canônica nem comece de um canvas genérico quando já houver pixel art válida. Para sprites de combate nativas, preserve o canvas transparente 400 × 400 RGBA, escala e linha-base do pacote, gere uma pose por chamada e mantenha o mapeamento entre saída e arquivo. Use a pose-alvo antiga só para movimento/enquadramento se ela divergir da identidade aprovada; não carregue efeitos que não existam naquela pose. Siga o prompt-base e os critérios de aprovação em `references/guia_sprites_e_pixel.md`. Para Garan, `04_special.png` é a âncora canônica; use `walk0.png` apenas como referência secundária depois de confirmar que retrata o mesmo oni. Revise cada arquivo e sequência em folhas de contato, separando defeitos visuais da cobertura estrutural. Retratos `portrait.png` são uma exceção: mantenha o acabamento detalhado de alta resolução e use `assets/chars/sprites/kaito/portrait.png` como referência de qualidade, conforme o procedimento específico no guia.

- [Validação de sequências e reações a dano](references/sequencias_de_combate.md)

## Recriar um lutador M.U.G.E.N existente

Para copiar features de um pacote M.U.G.E.N em um nativo, siga `docs/wiki/pt-BR/Recriar-um-Lutador-MUGEN.md` (mapeamento M.U.G.E.N para UMF, passos e o caso do Captain Marlow Reyes) e a skill `mugen-dissector`. Anims só com poses que têm PNG; poses de juntas inline não são permitidas.

## Paridade com um lutador M.U.G.E.N

Quando o UMF vem de um pacote MUGEN (via `mugen2umf.py`, skill `mugen-to-ume-migrator`):

- `stats.height` igual a `meta.json.refHeight` mantém o tamanho e a nitidez do original (1 px = 1 unidade).
- `art.style.boxes: "exact"` usa `hit.box` e `hurt` como escritos (Clsn1/Clsn2), sem recorte automático.
- `moves[].sounds: [{at, snd:[g,i], fight?}]` toca o som no quadro `at`; os wav ficam em `sprites/snd_{g}_{i}.wav` e são listados em `meta.json.sounds`. `fight: true` usa o fight.snd do motor.
- Cada lutador UMF compartilhável é uma pasta: `<id>.json` + `sprites/` (PNGs, `meta.json`, wavs, `portrait.png`). O jogo monta o pacote sozinho ao achar o JSON ao lado de `sprites/`.
- Compare lado a lado com o original no navegador antes de entregar: tamanho, golpes, caixas (debug de caixas em Training) e sons.

