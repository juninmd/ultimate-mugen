# Valdric — revisão independente de famílias de sprites

- Data: 2026-11-12
- Escopo: inventário estático do manifesto atual, resolução de referências, hashes de PNGs e revisão visual das folhas de contato em tamanho nativo e nearest-neighbor 2×.
- Resultado: **73 animações, 235 referências resolvidas a arquivos de sprite e 106 PNGs examinados**. O inventário encontrou 80 stems PNG únicos resolvidos e zero referências de sprite sem resolução.
- Runtime: **não executado**. Não há afirmação de aprovação em jogo, timing, colisão ou aparência após recolor em runtime.
- Testes: não executados.

## Proveniência

- Manifesto: [`assets/chars/valdric.json`](../../../../assets/chars/valdric.json), SHA-256 `6e8ef7a74e97fe9b9248c01d3217b884c97d1ac46c815c316d7fc45909e9c39e`.
- Metadados de sprite: [`assets/chars/sprites/valdric/meta.json`](../../../../assets/chars/sprites/valdric/meta.json), SHA-256 `629a68836b89aab863b7ccffebd8a7c89cfb0a6c03db8438d31aec207a60c941`.
- Âncora canônica de identidade/acabamento: [`assets/chars/sprites/valdric/04_special.png`](../../../../assets/chars/sprites/valdric/04_special.png), SHA-256 `28968edb0eeb0b819410cc109563225de1aebf3a62d3ae37d4edfe48e97b2489`. A pose/efeitos dessa âncora não foram usados para inferir ação de outras poses.
- Resolução estática segue `crates/engine/src/native/pixel.rs`; depois `crates/engine/src/native/compile.rs` chama `low_normals::complete` para injetar as animações low/air-low quando as condições do manifesto são atendidas. Para `clk` e `akick` atuais essas condições são verdadeiras. Resolução de frames segue: `frame.sprite` precede `pose`; poses nomeadas tentam arquivo exato, stem mapeado, `01_idle` e `idle`; poses inline usam a chave `inline_<FNV-1a do JSON serializado>`; quadros `effect` são procedurais.
- O [JSON sibling](valdric-full-family-independent-review-20261112.json) contém os 106 hashes completos, sequência/ticks das 73 animações, resolução de cada referência, aliases, inventário não referenciado, moves e formas.

## Classificação visual por família

A classificação compara identidade, traje e acabamento com `04_special`. “Legado” significa que o quadro mantém o design antigo de capuz/coroa e roupa vinho; “misto” significa alternância entre esse design e o traje canônico no mesmo ciclo. A tabela distingue referências explícitas nas 73 animações JSON e imagens adicionadas pelo compilador nativo.

| Família | Classificação | Frames ativos relevantes |
|---|---|---|
| Idle | Canônica | `01_idle`, `07_idle1`, `08_idle2` |
| Locomoção | Mista | `walk0–walk3` legados, `walk4–walk5` canônicos; `turn`, saltos, flips, hop e land legados; `run` usa `sg_run_*` canônicos |
| Crouch e guarda | Legada | `crouch*`, `guard*`, `crouchGuard*`, `airGuard*` |
| Dano e recuperação | Mista | `hitCrouch`, `hitL1–hitL3`, `getUp1–getUp3` legados; high-hit, fall e lie canônicos |
| Ataques normais | Predominantemente legado/misto | `jab0–1`, `str0–1`, `lk0–1`, `hk0–1`, `clk`, `sweep0` legados; parte dos ataques aéreos/crouch e `sweep1` canônicos |
| Royal Lance | Núcleo canônico, cauda legada | `sb0`, `sb1`, `sb2` canônicos; a recuperação ainda usa `jab0` legado |
| Outros especiais transformados | Mistos | `cu0`, `cu1`, `mh1`, `mh2`, `nc0`, `nc1` e dois quadros inline legados coexistem com poses canônicas |
| Intro/vitória | Canônica | Reuso de `06_victory`; derrota/morte reutiliza `14_lie` canônico |
| Efeitos | Procedurais | `royalFx`, `royalHit`, `dominionBeam`, `dominionHit` não resolvem PNG de corpo |

## Royal Lance: moves, aliases e hashes atuais

O manifesto vincula movimentos a animações. As três animações abaixo usam a mesma sequência de imagens, mas têm ticks diferentes. Os dois quadros inline e `jab0` fazem parte da sequência; não são substituídos por `sb0–sb2`.

| Move | Nome | Animação |
|---|---|---|
| `royallance` | Crimson Royal Lance | `royalLance` |
| `sovereign` | Sovereign Rise | `sovereign` |
| `thronebreaker` | Throne Breaker | `thronebreaker` |
| `dominion` | Scarlet Dominion Cannon | `super` |
| `lancewall` | Crimson Bulwark | `lanceWall` |
| `bastion` | Royal Counter | `royalCounter` |
| `seize` | Royal Seizure | `seize` |
| `barrage` | Crimson Barrage | `barrage` |
| `ex_royallance` | Royal Lance EX | `royalLanceEx` |
| `ex_sovereign` | Sovereign EX | `sovereignEx` |
| `ult_barrage` | Imperial Dominion Ultimate | `barrageUlt` |

| Forma | Move da forma | Animação |
|---|---|---|
| `valdric_awakened` | `awk_royallance` (Triple Royal Lance) | `royalLance` |
| `valdric_ascended` | `asc_ult_barrage` (Fallen Empire Dominion) | `barrage` |

Sequências declaradas:

- `royalLance`: `sb0` (8), inline de preparação (5), `sb1` (4), `sb2` (15), `jab0` (5), `idle0` (3).
- `lanceWall`: `sb0` (6), inline de preparação (5), `sb1` (4), `sb2` (12), `jab0` (5), `idle0` (3).
- `royalLanceEx`: `sb0` (6), inline de preparação (4), `sb1` (3), `sb2` (12), `jab0` (4), `idle0` (2).
- Forma `valdric_awakened`: `awk_royallance` → `royalLance`.
- Forma `valdric_ascended`: `asc_ult_barrage` → `barrage`; `barrage` e `barrageUlt` continuam incluindo `nc0/nc1` legados antes de `sb2`.

| Frame legado ainda ativo após o piloto | SHA-256 | Referências de animação |
|---|---|---|
| `jab0` | `fed043c4bdd0c874b251ccef7823ee16e099043bc769686077a75223c146971b` | jab, royalLance, lanceWall, royalLanceEx |
| `nc0` | `402f9ae425c854da5896be6cfbd823509377a170ee9d794bea14ff4e3f37eb24` | super, barrage, barrageUlt |
| `nc1` | `3a97dd845fe6e017d8424547d8dd22fa577667cc9cd024acbd44858bc699fe6a` | super, barrage, barrageUlt |

`sb0`, `sb1`, `sb2` atualmente ativos correspondem aos hashes `3058a0920cb9bd8597d04c268fa7f4e84fba1a4efc240792fee3fd7508e04c74`, `0a5c8cdfd6ff077774159b3745e31558bbbe0fbb60fa3d2ad739da5b94c202dd` e `c88eb38b15440a1ea159e3aab21065c50d46a02a6fe70774b5f72ff20a8ca3e8`. O `meta.json` atual registra eixo `[200,392]` e `refHeight 320` para os três. O registro de integração existente aponta margem direita de 10 px em `sb1`; não há clipping estático, mas a folga é estreita.

## Outros aliases de sequência

Os grupos abaixo resolvem sequências idênticas de imagens no manifesto. As durações permanecem específicas por animação e constam no JSON sibling.

| Animações | Fontes resolvidas |
|---|---|
| `jumpUp, jumpBack` | `jumpUp`, `jumpBack` |
| `guardStart, guard, guardEnd` | `guardStart`, `guard`, `guardEnd` |
| `crouchGuardStart, crouchGuard` | `crouchGuardStart`, `crouchGuard` |
| `airGuardStart, airGuard` | `airGuardStart`, `airGuard` |
| `hitAir, fall` | `hitAir`, `fall` |
| `bounce, bounceAir` | `bounce`, `bounceAir` |
| `lieDown, dead, lose` | `lieDown`, `dead`, `lose` |
| `hkick, stringC` | `hkick`, `stringC` |
| `royalLance, lanceWall, royalLanceEx` | `royalLance`, `lanceWall`, `royalLanceEx` |
| `royalHit, dominionHit` | `royalHit`, `dominionHit` |
| `sovereign, sovereignEx` | `sovereign`, `sovereignEx` |
| `barrage, barrageUlt` | `barrage`, `barrageUlt` |

## Frames legados ativos

| Família | Frames visualmente legados ou ciclos mistos |
|---|---|
| Locomoção e guarda | `turn`, `walk0–walk3`, `crouch`, `crouch0`, `crouch1`, `jumpStart`, `jumpUp`, `jumpTop`, `jumpDown`, `flip0–flip3`, `hop`, `land`, `guard`, `guardHit`, `crouchGuard`, `crouchGuardHit`, `airGuard`, `airGuardHit`. |
| Reações de dano | `hitCrouch`, `hitL1–hitL3`, `getUp1–getUp3`; quadros de high hit/fall/lie são canônicos. |
| Ataques normais | `clk`, `hk0–hk1`, `jab0–jab1`, `lk0–lk1`, `str0–str1`, `sweep0`; `cjab`, `sweep1`, `ajab` e `akick` têm quadros canônicos, mas alguns ciclos também usam quadros antigos de crouch/jump. |
| Transformações e especiais | `cu0`, `cu1`, `mh1`, `mh2`, `nc0`, `nc1`, `inline_cd8c2a8f4e49d4c8`, `inline_f0ca448d673ece1e`. |

O inventário completo em JSON dá o SHA-256 de cada PNG e a lista das animações JSON que o referenciam; ele separa as duas referências injetadas no compile path. Destaque adicional: `jab0` é reutilizado na recuperação de Royal Lance; `nc0`/`nc1` são usados por `super`, `barrage` e `barrageUlt` mesmo após a correção de `sb0–sb2`.

## Arquivos sem referência direta no manifesto

`lowKick.png` (`bc298477b7de49558e550cdd4e166223a124fc4c81930d55e84ec00a0b8ca70e`) e `airLowKick.png` (`84659e7d5d36dd011664a6508a0668a8736fa926295d22bcf65e1db866f35d3e`) não são referências explícitas em `assets/chars/valdric.json`, mas não são órfãos: `native::low_normals::complete` injeta os ataques e seus frames porque o manifesto atual contém `clk` com `when=crouch` e `akick` com `when=air`; o código substitui o frame de strike por `lowKick`/`airLowKick`. As duas imagens foram examinadas e correspondem à identidade canônica. `lkick` separadamente resolve `lk0/lk1`; `akick` e `divekick` resolvem `akick`. Nove sprites `df*` antigos não têm referência direta nas animações JSON atuais. O JSON lista os hashes de todos esses arquivos, templates/fallbacks e inline PNGs. A injeção foi verificada estaticamente no código e nas precondições, sem executar o runtime.

## Paletas e limite da auditoria

O manifesto declara duas paletas selecionáveis e uma substituição de `aura`/`energy` na forma `valdric_ascended`. A inspeção comparou os PNGs base e não renderizou essas variações. A revisão visual das folhas nativa e 2× foi feita durante a auditoria, mas as folhas temporárias não fazem parte deste par de relatórios. A conferência é estática e visual sobre PNGs/manifesto; não substitui execução em jogo.

**Nenhum sprite, manifesto, metadata, código ou teste foi alterado para produzir este relatório.**
