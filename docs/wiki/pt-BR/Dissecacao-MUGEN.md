# Dissecação de quatro lutadores M.U.G.E.N

Estudo somente leitura de quatro pacotes públicos, abertos arquivo por arquivo com `tools/mugen2umf`: Aizen TYBW (Fighter Factory Ultimate, SFF v2), Gogeta SSJ4 (Pocket_Danyell e Sarevok), Goku SSJ GT (Yamcha Z) e Goku SSJ4 (_Vision). Os pacotes são arquivos de terceiros: nada deles é commitado, só o que aprendemos. Estudos relacionados: [Estudo de código M.U.G.E.N](MUGEN-Code-Study.md) e [Estudo de lutadores](Fighter-Study.md).

## Tamanho de cada pacote

| | Aizen TYBW | Gogeta SSJ4 | Goku SSJ GT | Goku SSJ4 |
|---|---|---|---|---|
| Sprites | SFF v2.01, 1380 sprites, 696 paletas, 29 MB | SFF v1, 523 imagens | SFF v1, 116 imagens | SFF v1, 198 imagens |
| Sons | 272 (19 grupos, até 4 MB cada) | 33 | 5 | 19 |
| Ações AIR | 262 | 232 | 84 | 171 |
| Caixas Clsn1 / Clsn2 | 83 / 1538 | 301 / 1202 | 115 / 477 | 309 / 1469 |
| Comandos CMD | 67 | 65 | 62 | 65 |
| Life / ataque / defesa | 2250 / 115 / 125 | 2600 / 110 / 140 | 2100 / 120 / 110 | 2400 / 130 / 130 |

A Clsn2 supera a Clsn1 em cerca de 5 para 1 em todos: o corpo é dividido em 4 a 6 caixas pequenas por frame (cabeça, tronco, braços, pernas) e o ataque é de uma a três caixas. É a mesma regra da auditoria de hit boxes nativa.

## Layout de arquivos

| Arquivo | Papel |
|---|---|
| `.def` | `[Info]`, `[Files]`, `pal1..pal12`, `pal.defaults` |
| `.cmd` | `[Remap]`, `[Defaults]` (`command.time`, `command.buffer.time`), lista `[Command]`, `[Statedef -1]` roteando entrada para estado |
| `.cns` | Constantes `[Data]`, `[Size]`, `[Velocity]`, `[Movement]` e depois os estados |
| `st1..st3` | `IA.cns` (`-2`/`-3`), `Especials.cns`, `Supers.cns`, `-final`, `-story-intros` |
| `common1.cns` | Cópia dos estados comuns, sobrescritos ao redeclarar o número |
| `.air` | Frames `grupo,imagem,x,y,ticks[,flip,trans]`, `Clsn1` (ataque), `Clsn2` (corpo), `Clsn2Default`, `Loopstart` |
| `.sff` / `.snd` / `.act` | Sprites, sons e paletas de 256 cores |

## Constantes que valem copiar

- Andar 2,3 a 2,4, correr 4,6, pulo `-8,1` a `-10,1`, `yaccel` 0,43 a 0,44, `airjuggle` 15, largura de chão 15, altura 60, `attack.dist` 160. O elenco nativo já usa essa escala.
- O Aizen usa `airjump.num = 6` e andar 7: movimento rápido e flutuante feito só de constantes.

## Camada de comandos

- Quarto de círculo `~D, DF, F, a`, meia-lua `~B, DB, D, DF, F, a`, carga `~60$D,U,a` e `~60$B,F,a` (Goku SSJ), toque duplo `FF`, `BB`, `~D,U` para super pulo, botões `a+b`.
- `/$B,x`, `/$D,y`: direção segurada com botão, gerando golpes de comando.
- Buffers: `command.time = 10` e `buffer.time` 1 na maioria; 15 a 30 nos pacotes de entrada fácil.
- Um movimento, um botão por golpe: no Aizen, `D,DF,F` com a, b ou c é Silent Slash, Dark Waves ou Kurohitsugi. A UMF faz igual, um golpe por botão.
- Gogeta e Goku SSJ4 têm comandos `ai..ai13` (sequências de botões) que só ligam uma flag de IA no `-2`; o motor tem IA própria, então não migram.
- Cancels são triggers extras: `trigger2 = numhelper(700) > 0` (Aizen cancela em especiais enquanto o clone vive), `movehit` e `movecontact` (165 usos no Aizen, 22 no Gogeta, 14 no Goku SSJ4). Na UMF: `cancels` e `rekka`.

## Números dos golpes normais

Valores: dano, `pausetime` de atacante e defensor, `ground.hittime`, pontos de `juggle` e `poweradd`.

| Estado | Gogeta | Goku SSJ | Goku SSJ4 | Aizen |
|---|---|---|---|---|
| 200 leve em pé | 20, pausa 4,5, hit 11, juggle 4, power 22 | 23, pausa 7,8, hit 11, juggle 1, power 20 | 23, pausa 5,8, hit 7, juggle 1, power 20 | 20, pausa 9,9, hit 15, juggle 2, power 40 |
| 210 forte em pé | 55, pausa 4,5, hit 11 | 55, pausa 11,11, hit 15, juggle 4, power 60 | 63, pausa 12,12, hit 15, juggle 5, power 65 | 30, pausa 9,9, hit 15 |
| 400 leve agachado | 23, pausa 10,11, `guardflag = L` | 23, pausa 10,12, `L` | 23, pausa 5,8 | 20, pausa 0,15 |
| 410 forte agachado | 37, pausa 12,12, `M` | 62, trip, `fall = 1`, juggle 7 | 37, pausa 12,12, `M` | 30, pausa 0,60 |
| 600 leve no ar | 20, pausa 7,8, `HA` | 12, pausa 7,8, `HA` | 12, pausa 7,8 | 30, pausa 10,10 |
| 610 forte no ar | 72, pausa 12,12, hit 17 | 70, pausa 9,10 | mesmos valores do 600 | sem HitDef direto |

Regras dos pacotes: leves pausam 4 a 8 ticks e dão 15 a 25 de power, fortes pausam 11 a 12 e dão 55 a 70, `hittime` de chão 11 a 15, golpes baixos usam `guardflag = L` e `ground.type = Low`, golpes aéreos usam `HA`, e o forte agachado é a rasteira (`Trip` com `fall = 1`).

## Tipos de golpe e como são feitos

| Tipo | Exemplo | Construção |
|---|---|---|
| Bola de energia | Goku SSJ 1000 | `Projectile` em `Time = 14` com `ProjAnim`, `ProjHitAnim`, `velocity = 5.5,0`, `projpriority`, `HitDef` embutido, `AnimTime = 0` volta ao 0 |
| Bola no ar | Goku SSJ 1005 | Igual, `type = A`, `ProjHeightBound` remove no chão |
| Super de raio | Goku SSJ 1800 | `poweradd = -2000`, `SuperPause` no `AnimElem = 1`, `AfterImage`, três `Projectile` no mesmo frame: visual, longo com `projhits = 30`, `projmisstime = 5`, `damage = 19,4` |
| Ataque múltiplo | Goku SSJ 1050 | Uma ação longa, pares de `VelAdd` (+4 e -4) em `AnimElem` alternados, `EnvShake` no último |
| Agarrão | Goku SSJ4 1601 | `TargetBind` a cada elemento, `TargetLifeAdd` em `AnimTime = 0`, `TargetState` para o levantar |
| Clone | Aizen 700 | `Helper` com `anim = 9999`, `DestroySelf` em `time = 15` ou sem o root, e `numhelper(700)` como trigger de cancel |
| Faísca de hit | Aizen 995 | `Helper` em `movehit = 1 && numhelper(995) = 0`, `postype = p2` |
| Contra-ataque | Aizen Kyouka Suigetsu | Janela de `ReversalDef` ou `HitOverride` e estado que teleporta com `PosSet` |
| Teleporte | Aizen Shunpo | `PosAdd` de `p2dist x + 40` e `Turn` (o mesmo código que a UMF gera em `teleport`) |
| Forma | Gogeta Kaio Ken | Flag `var(11)`, `-2` soma power por tick, `AttackMulSet 1.2`, `DefenceMulSet 1.2`, aura `Explod` a cada 10 ticks |
| Power up | Goku SSJ 730 a 741 | Sobrescreve os estados comuns de power up com `PalFX`, `EnvColor` e som |

O Aizen é o único que constrói quase tudo com helpers: 98 `Helper`, 140 `Explod`, 128 `AssertSpecial`, 48 `AngleDraw`. O Goku SSJ é o oposto: nenhum helper nem explod, 12 `Projectile`. O Gogeta fica no meio com 32 `Projectile` e 109 `Explod`.

## Organização de sprites, sons e tempo

- Sprites: grupo 0 parado, 20 andar, 40 pulo, 100 e 105 virar e agachar, 200 a 240 ataques em pé, 400 agachado, 600 pulo, 5000 a 5120 apanhar, 9000 retrato. Especiais usam grupos livres de 1000 em diante. Goku SSJ4: 121 grupos para 198 imagens; Gogeta: 523 imagens em 109 grupos (63 vinculadas, reaproveitando pixels).
- Sons: grupo 0 golpes e vozes, 5 acertos, 6 defesa, 11 KO, 20 em diante para especiais. O Aizen tem 104 sons só no grupo 0.
- Paletas: `pal1..pal12` no `.def`; Goku SSJ4 e Gogeta trazem 5 `.act`; o Aizen embute as paletas no SFF v2.
- Tempo: um golpe normal dura de 8 a 17 ticks no total (Goku SSJ4 leve em pé 4 + 11 + 2, Aizen 2 + 2 + 4 + 4, Gogeta forte em pé 2 + 4 + 2); o frame de acerto é o do meio.

## O que mudou na UME

1. Lutadores nativos devem ler como esses pacotes: cadeias de normais, rekka, golpes de comando, versões aéreas, um projétil por botão, teleporte, contra-ataque e forma.
2. Caixas de acerto e de corpo seguem o membro: no mínimo 3 caixas de corpo.
3. Frame data mira a tabela acima: pausa de 4 a 8 nos leves, 11 a 12 nos fortes, hit time de chão 11 a 15.
4. A reconstrução do Captain Marlow Reyes é o primeiro lutador feito com este método, veja [Recriar um lutador M.U.G.E.N como UMF](Recriar-um-Lutador-MUGEN.md).
