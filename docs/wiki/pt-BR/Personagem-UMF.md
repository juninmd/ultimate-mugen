# Criar personagem UMF

UMF (*Ultimate Mugen Format*) descreve um lutador inteiro em **um arquivo JSON**. O compilador (`crates/engine/src/native/compile.rs`) gera comandos, estados, HitDefs, projéteis, super pause, cancelamentos e hitboxes. O resultado roda na mesma máquina de estados dos personagens M.U.G.E.N.

## Passo a passo

1. Copie um lutador pronto como base: `cp assets/chars/kaito.json assets/chars/meu.json`.
2. Troque `id` (único, minúsculo, sem espaço), `name`, `series`, `author` e `color`.
3. Opcional: coloque o `id` em `assets/chars/index.json` para definir a ordem na seleção. Os ausentes entram em ordem alfabética depois.
4. Rode `cargo run --release`. O `build.rs` embute todos os `.json` da pasta no executável.
5. Teste no **Treino** com `F2` ligado para ver hitboxes e hurtboxes.

Carregar UMF de uma pasta sem recompilar: **planejado** (o registro já tem `add_native_json`, falta ligar à pasta `mugen/`).

## Esqueleto do arquivo

```json
{
  "format": "umf/1",
  "id": "meu",
  "name": "Meu Lutador",
  "series": "Minha Série",
  "author": "você",
  "description": "Texto da tela de seleção",
  "color": "#e85d2a",
  "winQuote": "Frase de vitória",
  "intro": "Fala no início do round 1",
  "taunt": "Resposta/provocação",
  "form": "Nome da forma despertada",
  "boss": false,
  "speed": 1.0,
  "stats": {},
  "art": { "type": "rig", "style": {}, "poses": {} },
  "palettes": [],
  "awakeningPalette": { "hair": "#f6c83e" },
  "anims": {},
  "moves": []
}
```

| Campo | Obrigatório | Descrição |
|---|---|---|
| `format` | sim | Sempre `umf/1` |
| `id`, `name`, `series`, `author`, `color` | sim | Identidade; `color` é a cor do cartão na seleção |
| `boss` | não | `true` esconde da seleção e coloca como chefe final do Arcade |
| `speed` | não | Multiplicador de velocidade das animações (>1 = golpes mais rápidos) |
| `stats` | não | Atributos, tabela abaixo |
| `art` | sim | Arte, ver [Rig](Rig.md) |
| `palettes` | não | Cores alternativas; cada item sobrescreve chaves de `art.style.colors` |
| `awakeningPalette` | não | Cores aplicadas sobre a paleta selecionada durante o despertar; sprites de efeitos não mudam |
| `anims` | sim | Animações por nome |
| `moves` | sim | Golpes |

## `stats`

| Campo | Padrão de exemplo | Significado |
|---|---|---|
| `life` | `1000` | Vida |
| `power` | `3000` | Barra máxima de super (1000 por nível) |
| `attack` / `defense` | `100` | Multiplicadores em % |
| `walk` | `[2.4, 2.2]` | Velocidade andando para frente / para trás |
| `run` | `[5.2, 4.6]` | Corrida para frente / recuo |
| `jump` | `[-8.4, -2.5, 0, 2.6]` | Velocidade vertical, horizontal para trás, neutra e para frente |
| `gravity` | — | Aceleração vertical |
| `airJumps` | — | Pulos extras no ar |
| `airDashes` | 1 | Air dashes por pulo (`movement.airdash.num`) |
| `height` | `100` | Altura do corpo |
| `width` | `[16, 15]` | Largura frente / trás para empurrão |

## `anims`

Cada animação é uma lista de quadros:

```json
"jab": { "frames": [
  { "pose": "jab0", "ticks": 3 },
  { "pose": "jab1", "ticks": 3, "hit": { "part": "handR", "radius": 10 } },
  { "pose": "jab0", "ticks": 5 }
], "loop": 0 }
```

| Campo do quadro | Descrição |
|---|---|
| `pose` | Nome de uma pose em `art.poses` ou a pose inline (objeto) |
| `ticks` | Duração em ticks (60 por segundo); `-1` = infinito |
| `hit` | Hitbox ativa: `part` (membro ou arma) + `radius`, ou `box` `[x1, y1, x2, y2]` |
| `hurt` | `"none"` remove a hurtbox (invencível no quadro) |
| `effect` | Desenha um efeito em vez do corpo (projéteis, faíscas), ver [Rig](Rig.md#efeitos) |
| `offset` | Deslocamento `[x, y]` do desenho |
| `loop` | Índice do quadro onde o loop recomeça |

#### Como um frame de ataque vira hitbox

Frames desenhados com sprites em pixel usam como caixa de golpe a geometria do rig da `pose` e do `hit` do frame (`part` + `radius`), ou o `box` explícito quando existe, então cada golpe mantém o próprio alcance. O compilador então remodela a caixa para as proporções medidas em 605 personagens M.U.G.E.N (`native/pixel_hit.rs`, `H` = altura em pé, 96 unidades para `height` 100):

- como um `Clsn1` do M.U.G.E.N, a caixa vai do corpo até a ponta: começa no máximo `0.10 H` à frente da origem e alcança pelo menos `0.60 H` (a mediana do M.U.G.E.N é `0.72 H` nos socos e `0.80 H` nos chutes);
- pelo menos `0.36 H` de altura, nunca acima de `0.95 H` nem abaixo do chão (socos do M.U.G.E.N ficam entre `-0.75 H` e `-0.33 H`, chutes descem até `-0.05 H`);
- a hurtbox em pé tem `0.46 H` de largura (mediana do M.U.G.E.N `0.46 H`) e a agachada `0.67 H` de altura (mediana `0.67 H`);
- um `box` explícito é usado como escrito, então mantenha-o dentro dessas proporções; a auditoria falha caso contrário.

`cargo test --release -p ume-engine --test native_hitbox_audit` verifica todos os frames de golpe do elenco (altura, alcance, início, largura, acima da cabeça, abaixo do chão) e `native_normal_reach` imprime e compara alcance e dano dos quatro golpes normais em pé de cada lutador.

### Nomes padrão

Animações com estes nomes viram as ações reservadas do M.U.G.E.N, então os estados comuns funcionam sem código.

| Nome | Ação | Nome | Ação |
|---|---|---|---|
| `idle` | 0 | `guardHit` | 150 |
| `turn` | 5 | `crouchGuardHit` | 152 |
| `crouchTurn` | 6 | `airGuardHit` | 154 |
| `crouchStart` / `crouch` / `crouchEnd` | 10 / 11 / 12 | `lose` / `draw` / `win` | 170 / 175 / 181 |
| `walk` / `walkBack` | 20 / 21 | `intro` | 190 |
| `jumpStart` / `jumpUp` / `jumpFwd` / `jumpBack` / `jumpDown` | 40–44 | `taunt` / `powerUp` | 195 / 196 |
| `land` | 47 | `defeat1`…`defeat4` | 197–200 |
| `run` / `hop` | 100 / 105 | `hitLight` / `hitMedium` / `hitHeavy` | 5000–5002 |
| `guardStart` / `crouchGuardStart` / `airGuardStart` | 120–122 | `hitLowLight` / `hitLowMedium` / `hitLowHeavy` | 5010–5012 |
| `guard` / `crouchGuard` / `airGuard` | 130–132 | `hitCrouch` / `hitAir` | 5020 / 5030 |
| `guardEnd` | 140 | `airRecover` / `fall` / `trip` | 5040 / 5050 / 5070 |
| | | `bounce` / `bounceAir` / `lieDown` / `getUp` / `dead` | 5100 / 5101 / 5110 / 5120 / 5150 |

`taunt`, `powerUp` e `defeat*` têm versão padrão se você não definir. Outros nomes recebem números livres na ordem do arquivo.

## `moves`

```json
{ "id": "gale", "name": "Onda de Vento", "input": "~D, DF, F, x", "simple": "sp1",
  "kind": "special", "anim": "gale", "invuln": [1, 3],
  "moves": [{ "at": 2, "vel": [2, -9] }],
  "hits": [{ "at": 2, "damage": 90, "level": "heavy", "launch": true }],
  "projectile": { "at": 3, "anim": "galeOrb", "hitAnim": "galeHit", "speed": [5, 0], "offset": [40, -60], "damage": 70 } }
```

| Campo | Valores | Efeito |
|---|---|---|
| `id`, `name` | texto | Identificador e nome na lista de golpes |
| `input` | sintaxe de comando, abaixo | Comando |
| `simple` | `sp1` `sp2` `sp3` `super` | Atalho de um botão (teclas 1/2/3, toque) |
| `kind` | `normal` `special` `super` | Prioridade e cancelamento: normal → especial → super em acerto ou defesa |
| `when` | `stand` (padrão) `crouch` `air` | Estado de onde o golpe sai |
| `anim` | nome em `anims` | Animação do golpe |
| `hits` | lista | Um HitDef por item |
| `moves` | `[{ "at": quadro, "vel": [x, y] }]` | `VelSet` no quadro; use `null` para manter um eixo |
| `projectile` | objeto ou lista | Projéteis |
| `cost` | número | Barra consumida; `super` usa 1000 por padrão |
| `invuln` | `[início, fim]` | Quadros invencíveis |
| `teleport` | quadro | Reaparece atrás do oponente |
| `fx` | `aura` | Efeito visual extra |
| `voice` | `1000, 0` | Grupo e índice do som do personagem tocado no início do golpe, quando o áudio estiver incluído |
| `airPhysics` | `true` | Mantém a física aérea |
| `cancels` | lista de ids | Este golpe pode ser cancelado **para** esses golpes em contato (acerto, defesa ou projétil que encosta). Declarar substitui o gatling automático a partir deste golpe |
| `throw` | `{ "damage", "bind", "launch", "at" }` | Agarrão: veja [Traços de combate](#traços-de-combate) |
| `armor` | `{ "hits", "threshold" }` | Absorve golpes enquanto o movimento está ativo |
| `parry` | `{ "window": [de, até], "spark" }` | Devolve ataques na janela de elementos da animação |
| `timestop` | ticks | Congela todos os outros |
| `burst` | `true` | Quebra de combo usável ao apanhar |
| `requires` | `{ "life": {"max": 0.5}, "enemyLife": {"min": 0.1, "max": 0.5} }` | Condição de vida: o golpe só inicia enquanto a vida do lutador (`life`) e/ou do rival (`enemyLife`) estiver entre `min` e `max`, como fração da vida máxima (0 a 1). Qualquer limite pode ser omitido. Compila para triggers `life <= lifemax * max` e `enemy,life <= enemy,lifemax * max` |
| `flight` | `true` | Pairar em 8 direções enquanto o golpe dura |

**Finisher.** Um golpe com `"kind": "finisher"` (use `throw` ou `hits` para o efeito) só fica disponível após um nocaute (`RoundState = 3` e `WinKO`). Quando o round final termina em nocaute e o vencedor tem um finisher, o round espera até 6 segundos em "FINISH HIM" para o vencedor executá-lo; executar (ou o tempo acabar) encerra a espera. Elencos sem finisher não mudam.

**Ultimate.** Um golpe cujo `id` começa com `ult_` é a ultimate do lutador, e todo lutador nativo declara uma (o comando é `~D, DF, F, D, DF, F, x+y`). O compilador gasta o medidor inteiro nela: `cost` passa a valer `3000` por padrão (as três barras do HUD), o golpe recebe 70 % de bônus de dano onde um `super` comum recebe 30 %, e o congelamento de super dura 75 ticks em vez de 55. `cargo test --release -p ume-engine --test native_ultimates` falha quando um lutador não tem ultimate, quando o custo não é de três barras, quando a ultimate não inicia pelo comando ou não drena o medidor, ou quando o dano medido fica abaixo de 380 ou não supera todos os supers do lutador.

### Traços de combate

- **Gatling automático.** Um golpe sem `cancels` segue o gênero: um normal cancela em qualquer normal *mais forte* (`light` < `medium` < `heavy` pelo maior nível de acerto, empate pela ordem de declaração, então sequências nunca entram em loop), normais em pé ligam nos agachados, normais aéreos encadeiam com aéreos, e todo normal cancela em todo especial e super no contato. Especiais só cancelam em outros especiais por `cancels` explícito; supers cancelam a partir de qualquer golpe.
- **Rotas explícitas.** `"cancels": ["strong", "rush"]` no `jab` faz `strong` e `rush` aceitarem o `jab` no contato. Um especial de projétil pode ser cancelado quando o projétil toca o oponente. Um especial `air` pode começar de um lançador no chão que o liste (uppercut, depois mergulho).
- **`throw`** (`damage`, `bind` `[x, y]` à frente de quem agarra, `launch` `[vx, vy]`, `at` = frame do agarrão, padrão 1). O HitDef do agarrão dispara no elemento `at` (dê uma caixa `hit` a esse frame), a vítima fica presa em `at + 1`, leva o dano em `at + 2` e é lançada, então a animação precisa de pelo menos `at + 2` frames. Agarrões ignoram defesa e falham fora do alcance.
- **`armor`** (`hits` absorvidos, `threshold` opcional = dano acima do qual a armadura quebra). Do início até o último frame de acerto o golpe recebe 35 % de dano reduzido e não é interrompido; gastos os `hits`, o próximo golpe derruba normalmente.
- **Cola de multi-hit.** Golpes com três ou mais `hits` empurram o alvo só um pouco a cada acerto exceto o último (a sequência inteira conecta) e projéteis perfurantes (4+ acertos) não empurram o alvo para fora do feixe. Defina `knockback` para desativar.

Valide com `cargo test --release --test native_roster_lab -- --nocapture` (todo estado de ataque precisa conectar; imprime dano e alcance), `--test native_combo_routes` (todo cancel declarado encadeia de ponta a ponta) e `python scripts/audit_native_moves.py`.

### `hits[]`

| Campo | Padrão | Descrição |
|---|---|---|
| `at` | — | Quadro (1 = primeiro) em que o HitDef é ativado |
| `damage` | — | Dano |
| `guardDamage` | `damage / 8` | Dano na defesa |
| `level` | `medium` | `light` (hitstop 9, hitstun 12), `medium` (11, 16), `heavy` (13, 20) |
| `guard` | `mid` | `high`, `low`, `mid` ou `unblockable` |
| `knockback` / `airKnockback` | pelo nível | Velocidade `[x, y]` aplicada ao oponente |
| `launch` | `false` | Derruba |
| `hitstop`, `hitstun`, `spark`, `shake` | pelo nível | Ajuste fino |

### `projectile`

| Campo | Descrição |
|---|---|
| `at` | Quadro em que sai |
| `anim` / `hitAnim` | Animação em voo / ao acertar |
| `speed` / `accel` | Velocidade e aceleração `[x, y]` |
| `offset` | Posição de saída `[x, y]` relativa ao lutador |
| `damage`, `level`, `launch`, `knockback` | Como em `hits` |
| `hits` | Número de acertos |
| `removeTime` | Ticks até sumir |
| `priority` | Prioridade contra outros projéteis |

### Elementos e atordoamento

`hits[]` e `projectile` aceitam `element` (`fire`, `ice`, `lightning`, `earth`, `wind`, `water`) e `stun` (ticks extras de hit-stun). O compilador soma o hit-stun e o tremor do elemento e tinge a vítima por um instante (`PalFX`): `ice` congela por mais tempo (+16), `lightning` eletrocuta (+22 e tremor de 8 ticks), `fire` queima em vermelho, `earth` treme a tela, `wind` e `water` aplicam um tom curto. `stun` soma por cima e adiciona tremor de 5 ticks quando não há outro. Elementos desconhecidos são ignorados.

### Sintaxe de comando

| Símbolo | Significado | Exemplo |
|---|---|---|
| `F B U D` / `DF DB UF UB` | Direções (relativas ao lado) | `D, DF, F, x` |
| `x y z a b c` | Botões (`s` = start) | `x` |
| `,` | Em sequência | `F, F` |
| `+` | Ao mesmo tempo | `x+y` |
| `~` | Soltar | `~D, DF, F, x` |
| `/` | Segurar | `/B, F, x` |
| `$` | Direção inclusiva (`$D` aceita DB, D, DF) | `$D, x` |
| `>` | Sem nenhuma outra entrada entre os passos | `F, >F` |
| `N` | Direção neutra (nenhuma direção pressionada) | `N, x` |

## Normais de poder (Soco 3 e Chute 3)

Todo lutador nativo que não declara um golpe em `z` ou `c` ganha quatro normais extras na compilação (`native/power_normals.rs`): `power_punch` e `crouch_power_punch` em `z` (tecla padrão **E**, exibido como *Soco 3*), `power_kick` e `crouch_power_kick` em `c` (tecla padrão **D**, exibido como *Chute 3*). Cada um reaproveita animação, tempo, altura de defesa e `cancels` da normal de chão mais forte da família (`y` e depois `x` nos socos, `b` e depois `a` nos chutes; as versões agachadas usam a normal agachada), com o nome `<nome base> Max`. Os golpes causam 130 % de dano, viram `heavy`, ganham 4 de hit-stun e 2 de hit-stop quando definidos e empurram 25 % mais. Golpes que cancelam na normal base também cancelam na versão de poder, e os novos golpes aparecem na lista de golpes da pausa. Declare um golpe próprio em `z` ou `c` para não usar o automático. Versões aéreas não são geradas aqui.

Todos os lutadores nativos também recebem dois chutes baixos normais na compilação (`native/low_normals.rs`): `lowkick` em `D+a` agachado e `airlowkick` em `D+a` no ar. Eles reutilizam as animações de chute baixo agachado e de chute aéreo do próprio personagem, preservando os quadros de pixel art, e acertam a guarda baixa. A versão aérea avança e desce para alcançar o adversário em distâncias curtas. Se o lutador já declarar um golpe com o mesmo id ou a mesma combinação `D+a` e estado, o golpe declarado é preservado.

## Carregar energia (todo lutador)

Todo lutador nativo pode carregar o medidor: segure **R** (a entrada `s` / Start; para pausar a luta use **Esc** ou **P**). O lutador entra no estado `3920`, repete a animação `powerUp` com um brilho azul e ganha 4 de energia por tick (um super de 1000 em cerca de quatro segundos) até soltar o botão (depois de pelo menos 6 ticks) ou encher o medidor. O estado é `movetype = I`, então um golpe o interrompe, e um lutador com formas ainda se transforma com `T` quando o medidor chega ao custo da forma. O comando é `charge_cmd` (`/s`) e aparece como `Charge Energy` na lista de golpes. A IA embutida só carrega no nível 3 ou mais, com menos de 700 de energia, quando o rival está a mais de 190 unidades e não está atacando.

## Habilidades no ar (todo especial e super)

Todo `special` e `super` com `when` ausente ou `"stand"` ganha também uma versão aérea (estado `20000 + estado de chão`, rótulo `State -1, air <nome>`), acionada pelo mesmo comando com o lutador no ar. O lutador flutua no começo do golpe (mantém 40 % do deslize), a gravidade volta a 45 % depois de 12 ticks, o golpe termina no estado de queda do salto (`50`) com controle, e tocar o chão o interrompe com o estado de pouso (`52`). As versões aéreas causam 70 % do dano de chão, os projéteis saem inclinados para baixo (inclinação 0,85) para alcançar um rival no chão, e cada salto permite um especial aéreo (`var(57)`, zerada ao pousar). Agarrões, golpes de voo e golpes com `when: "crouch"` ou `"air"` mantêm suas próprias regras.

## Sprites pixel art (opcional)

Se existir `assets/chars/sprites/<id>/`, o lutador é desenhado com arquivos PNG em vez do rig. Use PNGs transparentes voltados para a direita, com os pés na borda inferior e o mesmo tamanho de tela em todos os arquivos de um lutador. Uma pose sem PNG usa `01_idle.png`; um lutador sem a pasta continua com o rig. Depois de estender o rig, execute `cargo run --release -p ume-engine --example generate_missing_sprites` para renderizar um PNG dedicado para cada pose que o carregador buscaria no canvas idle.

| Arquivo | Poses que o usam |
|---|---|
| `01_idle` | `idle0`, andar, correr, agachar, defender, pular e toda pose sem arte própria |
| `02_attack1` … `06_victory` | poses de soco, chute, especial, projétil e vitória / provocação |
| `07_idle1`, `08_idle2` | os outros quadros de respiração do `idle` (`idle3` reaproveita `07_idle1`) |
| `09_hit1`, `10_hit2`, `11_hit3` | `hitH1`, `hitH2`, `hitH3` (dano) |
| `12_fall1`, `13_fall2` | `hitAir1`/`fall1`/`trip` e `hitAir2`/`fall2`/`bounce` (lançado ao ar) |
| `14_lie` | `lie`, `dead`, `lose` e toda pose `df*` (nocauteado, com hurtbox baixa e larga) |
| `15_power0`, `16_power1`, `17_power2` | `powerup0`, `powerup1`, `powerup2` (transformação, tecla `T`) |

`meta.json` guarda `{"refHeight": N}`: a altura em pixels do corpo em pé em `01_idle.png`. Toda pose do lutador é escalada para esse corpo medir 100 unidades, então um sprite deitado ou agachado mantém o mesmo tamanho de pixel. Sem ele, cada PNG é escalado a 100 unidades pela própria altura.

Quando `awakeningPalette` altera `hair`, cada sprite define `frames.<nome>.awakeningHair` como `[esquerda, topo, direita, baixo]` em pixels da imagem original. A troca de cabelo fica limitada a esse retângulo; sprites sem retângulo mantêm o cabelo original. O empacotamento web redimensiona essas coordenadas junto com a imagem.

As entradas de `palettes` também valem para esses PNGs: cada pixel é casado com a linha de sombreamento de uma cor declarada em `art.style.colors` e a cor da paleta substitui no mesmo tom. Uma chave de paleta sem entrada correspondente em `art.style.colors`, ou uma cor que nada na arte lembra, fica intocada.

O build web não lê essa pasta do disco. `scripts/build-web.sh` roda `ume-tools pack-sprites` (`crates/devtools`), que grava um `assets/chars/sprites/<id>.pack` por lutador (imagens limitadas a 768 px, 256 cores, âncoras do `meta.json` reescaladas) em `web/dist`; o navegador baixa o pack quando o lutador ganha foco na seleção ou é carregado para a luta, e o engine lê pelo mesmo diretório virtual. Se o download falhar, o rig continua.

## Validar

```bash
cargo test --workspace --release
cargo run --release -p ume-engine --example rig_sheet
```

`rig_sheet` gera uma folha com todas as poses para revisar a arte sem abrir o jogo.

Os pacotes de sprites do navegador carregam de forma assíncrona. Um pacote novo ou substituído invalida as prévias e todas as paletas em cache do lutador afetado; os demais mantêm seus caches. O carregamento da luta aguarda os pacotes dos lutadores nativos escolhidos e informa falhas de download, em vez de iniciar com uma aparência procedural antiga. Definições nativas importadas sem pacote integrado continuam compatíveis.
