# Estudo Empírico em Larga Escala: 2.625 `.def` / 1.914 Lutadores de 27 Repositórios

Síntese técnica e quantitativa obtida por engenharia reversa automatizada e leitura de código sobre um corpus de **2.625 definições `.def`** (**1.914 pastas únicas de personagens**), somando **8.078 arquivos `.cns`/`.st`**, **1.827 `.cmd`**, **1.835 `.air`**, **58 `.zss`** e **498.987 ações de animação (`[Begin Action]`)** extraídos de **27 fontes públicas** (Elecbyte oficial + 26 repositórios abertos de M.U.G.E.N / Ikemen GO no GitHub), complementada por uma auditoria de **9.130 arquivos de estado** para quantificação de armadilhas reais.

---

## 1. Demografia Técnica do Corpus Real

| Métrica Analisada | Distribuição Observada no Corpus (2.625 `.def`) | Implicação Prática de Engenharia |
| :--- | :--- | :--- |
| **`mugenversion`** | `1.0` (**988**), datas WinMUGEN `06,14,2003` / `04,14,2001` / `04,14,2002` (**929+**), `1.1` (**303**), `ikemenversion` explícito (`1.0` / `0.99.2`) | Personagens antigos têm **bug de bitmask `Floor(2**n)`** se rodarem em modo compatibilidade WinMUGEN; sempre declare `1.0` ou `1.1` em personagens novos. |
| **`localcoord`** | Omitido/padrão 320×240 (**2.086**), `428,240` widescreen 16:9 (**265**), `320,240` (**201**), `380,360` WOS (**39**), `342,240` (**12**), `320` abreviado (**10**), `1280,720` (**6**), `640,480` (**4**) | `428,240` é o padrão dominante de personagens HD/16:9 em sprites Lo-Res (PotS/CvS). `const240p()` mantém velocidades consistentes entre 4:3 e 16:9. |
| **Modularidade (`st`..`st10`)** | 1 arquivo (**484**), 2–5 arquivos (**956**), 6–8 arquivos (**1.147**), 9–11 arquivos (**137**) | Projetos profissionais convergem para **6 a 8 arquivos de estado** (`system`, `normals`, `specials`, `ex`, `supers`, `helpers`, `custom`, `-2/-3`). |
| **Integrações Modernas** | `movelist` / `movelist.dat` (**385**), `[Palette Keymap]` (**385**), variantes `unotag`/tag (**140**), arquivo `ai` separado (**38**), `.zss` (**16** defs / **58** arquivos) | `[Palette Keymap]` e `movelist.dat` padronizam UI de seleção e lista de golpes sem depender de scripts externos. |
| **Sistemas de Common (`stcommon`)** | `common1.cns` (**2.282**), `common.cns` (**90**), `n_common.st` (**32**), `cns/common1.cns` (**28**), `common1.cns.zss` (**7**) | Sobrescrever apenas os estados necessários em um arquivo `override.cns` é mais seguro do que duplicar um `common1.cns` inteiro modificado. |

---

## 2. Taxonomia dos 8 Arquétipos Reais da Comunidade

### 2.1. Padrão PotS / CvS (`mredraith/CVS-P` — 266 defs, 171 chars)
O padrão mais influente da cena competitiva moderna:
- **Arquitetura**: `localCoord = 428,240`, `st`..`st7` (`System.st`, `Normal.st`, `Specials.st`, `EX-Specials.st`, `Supers.st`, `Helpers.st`, `Custom.st`, `<Char>.st`), `movelist.dat`.
- **Tick Fix no `[Statedef -1]`**: `CtrlSet { value = 1 }` no topo do `-1` quando `stateno = 52 || stateno = 105 || stateno = 5120 && !animtime` ou `hitover`, eliminando o **1 frame morto** entre o fim de estado e a leitura de comandos do jogador.
- **Projéteis como `Helper` (não `Type = Projectile`)**: Permitem `HitOverride` + `HitBy` para colisão mútua, rastro procedural, escala por `AttackMulSet`/`fvar`, sons e HitSparks dedicados.
- **HitSparks sem sprites extras**: `[Statedef 8000]` usa 1 único sprite ampliado frame a frame com `AngleDraw { scale = 0.125 + 0.046875 * Time }` e apagado com `Trans { trans = AddAlpha, alpha = 256 - 64 * (Time - 4), 256 }`.
- **Anti-Pause Trick (`var(58) = gametime`)**: Gravado no `[Statedef -3]` com `ignorehitpause = 1` para detectar quando o lutador ficou congelado em `Pause`/`SuperPause` ou custom state.
- **Sistemas Característicos**: Parry (`700/701/702`), Dodge/Roll (`710/720`), Custom Combo (`760`, `var(20)` temporizado), Zero Counter, Corner Push (`fvar(5)`), Damage Dampener escalonado + Counter-Hit `1.2x` (`EnemyNear, MoveType = A`), Juggle Limiter manual (`var(15)/var(16)`), e Supers em dupla no modo `TeamMode = Simul`.

### 2.2. Padrão Team Z2 / Hyper DBZ (`BuuZ2`, `SSJ_GokuZ2`, `GohanZ2`)
- **`config.txt` acoplado como `st6` no `[Statedef 5900]`**: Usuário configura Throw Tech (`0/1/2`), transparência de FX, voz EN/JP e dampener editando apenas valores no arquivo de configuração.
- **2OS Helper (`Helper(2)`)**: Helper invisível com `pausemovetime/supermovetime = 9999999` que lê `life` para calcular exatamente o dano real sofrido (`life - helper(2),var(2)`), contornando falhas de `gethitvar(damage)`.
- **Trava `!WinKO` obrigatória em `TimerFreeze`/`RoundNotOver`**: Garante que supers que matam o oponente não bloqueiem a transição para a winpose.
- **Super Armor / Focus Attack (`1300/1301`)**: `Helper` colado via `BindToRoot` com `HitOverride` + `NotHitBy` coordenado e reset de carga ao sofrer hit ou cancelar em dash (`StateNo = [100,105]`).

### 2.3. Padrão JUS / Jump & Naruto WOS (`Meiloorun`, `BillsLasa`, `Loco1911`, `Xandrarre`)
- **`Helper(999999), Fvar(16)` (Escala Universal JUS)**: Todas as posições e escalas de `Explod` são multiplicadas por `Helper(999999), Fvar(16)`, permitindo portabilidade visual instantânea entre screenpacks diferentes.
- **Transformações por Deslocamento de Animação**: `anim = <base> + var(11)` ou `+10000` / `+15000` para alternar entre Forma Base, Bankai, HOS, KCM ou Baryon duplicando apenas o índice de ação sem reescrever toda a máquina de estados.
- **Clones (`Kage Bunshin`) e Marionetes (`Sasori`)**: `Helper` autônomo (`helpertype = normal` ou `player`) com `ScreenBound = 0`, `HitOverride` direcionando para estado de dissipação (`stateno = 15001`), e monitoramento contínuo de `parent,stateno = 170/181` e `parent,life = 0`.
- **Arquivos RPG/Modulares (WOS)**: `Weapon.cns`, `Upgrades.cns`, `Stages.cns` controlando inventário, buffs passivos e interações ambientais.

### 2.4. Padrão Marvel vs Capcom / Anime Rush (`DocDamage/MUGENX` — 1.205 defs)
- **Magic Series**: Gatlings em cadeia contínua `Light -> Medium -> Heavy -> Launcher (420)`.
- **Super Jump (`Statedef 90`)**: Impulso vertical elevado após Launcher + tracking de câmera vertical e combos aéreos (`600..650`) com `airjump.num` e `airdash`.
- **OTG (`hitflag = MAFD`)**: Presente em **12.553** `HitDef`s do corpus para continuar combos em oponentes derrubados no chão.
- **Advancing Guard / Pushblock**: `Command` durante blockstun (`[150,155]`) gerando `Helper` com `PlayerPush = 1` ou `VelSet` que empurra o atacante para longe.

### 2.5. Padrão Tag, Assist & Coop (`UnoTag`, `Add004`, `Ikemen tag.zss`)
- **`UnoTag` (**140** variantes `.def`)**: Patcher automático que injeta `tag_system.cns` no `st9` e comandos dedicados.
- **`Add004`**: Usa `sysfvar(0)` apontando para um Helper controlador (`var(0) = 90900`), enviando parceiros ociosos ao estado `190190` e coordenando assistências e switch-cancels via `sysfvar(4)`.
- **Nativo Ikemen GO / Ultimate MUGEN (`tag.zss`)**: Controladores nativos `TagIn`, `TagOut`, triggers `standby`, `teamLeader`, `teamMap(...)`, estados dedicados (`StateTagEnteringScreen`, `StateTagLeavingScreen`, `StateTagWaitingOutside`, `StateTagJumpingIn`).

### 2.6. Bonus Stages, Eventos PvE & Bosses (`SF2Car`, `SoR2`, `ctm-magus`)
- **Bonus Targets (`SF2Car`)**: Personagem sem golpes próprios cujo `[Statedef 0]` troca de `anim` conforme faixas decrescentes de `life` (`life > 1450`, etc.), gerando `Explod`s de destroços aleatórios ao levar dano e `NotHitBy` em derrota/vitória.
- **Bosses / RPG (`ctm-magus`)**: `[State -3, Prevent Underground Juggles]` corrige imediatamente alvos em queda livre que atravessam `pos y > 0` usando `TargetBind` em `ceil(-pos y)`.

---

## 3. Frequências Empíricas de Sctrls, Triggers, HitDef e Constantes

### Top Controladores de Estado (em 9.905 CNS/CMD)
1. `ChangeState` (**412.908**) | 2. `PlaySnd` (**315.572**) | 3. `Explod` (**230.351**) | 4. `VarSet` (**229.640**) | 5. `VelSet` (**143.810**)
6. `HitDef` (**112.110**) | 7. `ChangeAnim` (**111.861**) | 8. `Helper` (**104.972**) | 9. `AssertSpecial` (**65.130**) | 10. `DestroySelf` (**57.906**)
11. `NotHitBy` (**55.801**) | 12. `VarAdd` (**55.633**) | 13. `RemoveExplod` (**51.148**) | 14. `PosSet` (**48.930**) | 15. `VelAdd` (**47.012**)
16. `StateTypeSet` (**43.679**) | 17. `ModifyExplod` (**43.664**) | 18. `PosAdd` (**39.622**) | 19. `TargetBind` (**36.504**) | 20. `PalFX` (**31.763**)
21. `SelfState` (**29.693**) | 22. `AngleDraw` (**26.488**) | 23. `ScreenBound` (**23.140**) | 24. `PlayerPush` (**21.816**) | 25. `EnvShake` (**21.430**)
26. `Trans` (**20.730**) | 27. `Null` (**19.600**) | 28. `TargetState` (**18.923**) | 29. `Turn` (**18.733**) | 30. `Width` (**18.502**)

### Top `AssertSpecial` Flags no Corpus
`noshadow` (**30.229**), `intro` (**6.709**), `invisible` (**6.674**), `roundnotover` (**4.667**), `noautoturn` (**4.167**), `nofg` (**3.848**), `nobardisplay` (**3.671**), `nowalk` (**2.817**), `unguardable` (**2.683**), `nobg` (**2.495**), `nomusic` (**2.382**), `noairguard` (**1.450**), `nojugglecheck` (**1.437**), `timerfreeze` (**1.309**), `nokosnd` (**1.016**), `nostandguard` (**971**), `nocrouchguard` (**954**), `nokoslow` (**659**), `globalnoshadow` (**635**), `noko` (**217**).

### Catálogo Completo de `AssertSpecial` no `ume-engine` ([`flags.rs`](../../../../crates/engine/src/flags.rs))
- **59 Flags de Personagem (`asf`)**: `invisible`, `noairguard`, `noautoturn`, `nocrouchguard`, `nojugglecheck`, `noko`, `noshadow`, `nostandguard`, `nowalk`, `unguardable`, `animatehitpause`, `animfreeze`, `autoguard`, `drawunder`, `noaibuttonjam`, `noaicheat`, `noailevel`, `noairjump`, `nobrake`, `nocombodisplay`, `nocornerpush`, `nocrouch`, `nodizzypointsdamage`, `nofacedisplay`, `nofacep2`, `nofallcount`, `nofalldefenceup`, `nofallhitflag`, `nofastrecoverfromliedown`, `nogetupfromliedown`, `noguardbardisplay`, `noguarddamage`, `noguardko`, `noguardpointsdamage`, `nohardcodedkeys`, `nohitdamage`, `noinput`, `nointroreset`, `nojump`, `nokofall`, `nokovelocity`, `nolifebaraction`, `nolifebardisplay`, `nomakedust`, `nonamedisplay`, `nopowerbardisplay`, `noredlifedamage`, `noscore`, `nostand`, `nostunbardisplay`, `noturntarget`, `nowinicondisplay`, `postroundinput`, `projtypecollision`, `runfirst`, `runlast`, `sizepushonly`, `nodestroyself`, `immovable`.
- **19 Flags Globais (`gsf`)**: `globalnoko`, `globalnoshadow`, `intro`, `nobardisplay`, `nobg`, `nofg`, `nokoslow`, `nokosnd`, `nomusic`, `roundnotover`, `timerfreeze`, `camerafreeze`, `notimedisplay`, `roundfreeze`, `roundnotskip`, `skipfightdisplay`, `skipkodisplay`, `skiprounddisplay`, `skipwindisplay`.

### Modas Empíricas de Constantes, `CMD` e `AIR`
- **Constantes (`[Data]`/`[Movement]`/`[Velocity]`)**: `life = 1000` (1.518x), `power = 3000` (848x; `5000` em 97x), `airjuggle = 15` (1.486x), `IntPersistIndex = 60` (750x) ou `0` (533x), `FloatPersistIndex = 40` (1.037x), `yaccel = .44` (703x em 240p; `.5` em 171x), `walk.fwd = 2.4` (349x), `jump.neu = 0,-8.4` (398x). Nota: `data.volumescale = 100` substitui o legado `data.volume`.
- **`.cmd`**: `command.time = 15` (652x), `command.buffer.time = 1` (680x). Modificadores mais usados: `~` release (**71.225**), `/` hold (**34.801**), `$` 4-way (**32.390**), `+` chord (**30.494**), `buffer.time` por comando (**3.146**), `[Remap]` (**1.091**), `>` estrito (**47**).
- **`.air` (1.835 arquivos / 498.987 ações)**: `Clsn2` blocos (**978.362**), `Clsn1` blocos (**245.089**), `Clsn2Default` (**133.307**), `Clsn1Default` (**9.422**), transparência `A`/`AS` (**881.325**) vs `S` (**31.098**), flips `H` (**127.563**) e `V` (**81.309**), `Loopstart` (**45.346**), parâmetros MUGEN 1.1 de escala/ângulo (**10.124**) e `Interpolate` (**2.817**).

---

## 4. As 10 Armadilhas Mais Comuns Encontradas em Personagens Reais

| # | Bug / Armadilha Real | Ocorrência Medida no Corpus | Sintoma / Falha | Correção Definitiva |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **`StateNumberMismatch`** | **1.391.086** blocos `[State X]` sob `[Statedef Y]` | Copiar/colar `[State 0, ...]` sem renomear quebra leitura, debug e falha em `--lint` | Sempre alinhe o número de `[State N]` com `[Statedef N]` |
| 2 | **`AfterImage` sem `AfterImageTime`** | **1.472 / 3.500** arquivos com `AfterImage` (42%) | Rastro fantasma congela na tela quando golpe/super é interrompido | No `[Statedef -2]` ou ao sair do estado com `MoveType = H`, emita `AfterImageTime { time = 0 }` |
| 3 | **Vazamento de `Helper`** | **1.833 / 6.073** arquivos têm `Helper` sem `DestroySelf` no mesmo arquivo | Atinge o limite de helpers (~56) no meio da luta e para de spawnar projéteis/FX | Proteja criação com `!NumHelper(id)` e garanta `DestroySelf` por `time > max`, fim de anim ou troca de `root,stateno` |
| 4 | **Quebra de Case-Sensitivity em Linux/WASM** | **287** referências no `.def` divergem em maiúsculas/minúsculas | Funciona no Windows (NTFS) mas falha ao carregar no Linux, Web/WASM e Android | Use sempre `/` (nunca `\`) e exata correspondência de maiúsculas/minúsculas nos nomes de arquivos |
| 5 | **Parâmetros Duplicados e Ações Duplicadas/Vazias** | **16.810** params duplicados; **2.400** ações duplicadas; **144** vazias no `.air` | Motor sobrescreve silenciosamente o primeiro valor ou falha no `lint --strict` | Remova chaves repetidas dentro do mesmo `[State]` e ações duplicadas/vazias no `.air` |
| 6 | **`persistent` em Estados Negativos** | **12.585** usos em `[Statedef -1 / -2 / -3]` | `persistent != 1` em estados negativos é ignorado ou dispara erro `NegativeStatePersistent` | Controle execução única em `-2`/`-3` com `var(...)` ou `gametime`, nunca `persistent = 0` |
| 7 | **`TargetBind` sem `!GetHitVar(isbound)`** | **271** arquivos com `TargetBind` sem proteção de soltura | Se P1 for interrompido por um helper ou terceiro (Simul), P2 congela no ar eternamente | Todo estado custom `movetype = H` de agarrão precisa de `SelfState { value = 5050 }` quando `!gethitvar(isbound)` |
| 8 | **Underground Juggles (`pos y > 0`)** | Comum em quedas e golpes multi-hit aéreos | Oponente atingido em `hitfall` afunda abaixo do chão | Adicione correção no `[Statedef -3]` com `TargetBind { pos = ceil(p2dist x), ceil(-pos y) }` quando `target, pos y > 0` |
| 9 | **`RoundNotOver` / `TimerFreeze` sem `!WinKO`** | Causa recorrente em supers cinematográficos | Ao vencer com Super, o jogo fica preso impedindo a pose de vitória | Exija `triggerall = !WinKO` ao usar `flag = roundnotover` / `timerfreeze` no `[Statedef -2]` |
| 10 | **Encoding não-UTF8, `==` em CNS e Marcadores Git** | Vários arquivos em Latin-1/Shift-JIS, bytes NUL e conflitos `<<<<<<< HEAD` | Erros de parse em ferramentas modernas e operadores `==` rejeitados pelo parser CNS | Salve scripts em UTF-8 sem BOM/NUL, use `=` (não `==`) em `.cns` e rode `mugen_check --lint` |
