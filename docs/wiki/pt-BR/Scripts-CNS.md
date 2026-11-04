# Scripts CNS

Toda lógica de luta, inclusive a dos lutadores UMF, roda como CNS: estados (`[Statedef]`), controllers (`[State]`) e triggers. Para programar um lutador à mão, use o formato M.U.G.E.N (pasta com `.def`, `.cns`, `.cmd`) e coloque em `mugen/chars/`.

Sintaxe moderna ZSS (do Ikemen-GO): **planejado**, compilando para o mesmo bytecode.

## Arquivo `.def`

```ini
[Info]
name = "Meu Lutador"
displayname = "Meu Lutador"
author = "você"
localcoord = 320, 240

[Files]
cmd = meu.cmd
cns = meu.cns
st = meu.st
stcommon = common1.cns
sprite = meu.sff
anim = meu.air
sound = meu.snd
pal1 = meu1.act
```

`localcoord = 1280, 720` marca personagem HD; a engine escala sozinha.

## Exemplo de estado

```ini
[Statedef 1000]
type = S
movetype = A
physics = S
anim = 1000
ctrl = 0
velset = 0, 0
poweradd = 30

[State 1000, golpe]
type = HitDef
trigger1 = AnimElem = 3
attr = S, SA
damage = 90, 10
animtype = Heavy
guardflag = MA
pausetime = 12, 12
sparkno = 2
ground.type = High
ground.hittime = 18
ground.velocity = -5
air.velocity = -4, -6
fall = 1

[State 1000, fim]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
```

E no `.cmd`:

```ini
[Command]
name = "hadou"
command = ~D, DF, F, x
time = 20

[Statedef -1]

[State -1, hadou]
type = ChangeState
value = 1000
triggerall = command = "hadou"
trigger1 = statetype = S && ctrl
```

Extras de comando (Ikemen GO 1.0):

| Recurso | Comportamento |
|---|---|
| Passo `N` | Direção neutra: nenhuma direção pressionada (`command = N, x`, `~D, N, x`, `N+x`). O passo neutro é um estado mantido, como `/`, e vale enquanto nenhuma direção está pressionada |
| `pausebuffer` | `[Command]` (ou `command.pausebuffer` em `[Defaults]`) mantém ativo por esse número de ticks extras um comando disparado durante hit pause, hit shake ou pause/super pause, para que possa ser lido quando a pausa terminar. Também aceita `buffer.pausetime`. Padrão 0 |
| Estados customizados | Quando um jogador roda um estado de outro personagem (`TargetState`, `p2getp1state`), `Command` usa os comandos do dono do estado contra a entrada do jogador no estado |

## State controllers suportados

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AppendToClipboard` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `ClearClipboard` `CtrlSet` `DefenceMulSet` `DestroySelf` `DisplayToClipboard` `DizzyPointsAdd` `DizzyPointsSet` `DizzySet` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `FallEnvShake` `ForceFeedback` `GameMakeAnim` `Gravity` `GuardBreakSet` `GuardPointsAdd` `GuardPointsSet` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitOverride` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `MapAdd` `MapReset` `MapSet` `ModifyExplod` `ModifyProjectile` `ModifySnd` `MoveHitReset` `NotHitBy` `Null` `Offset` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RedLifeAdd` `RedLifeSet` `RemapPal` `RemoveExplod` `RemoveText` `ReversalDef` `RootVarAdd` `RootVarSet` `ScreenBound` `SelfState` `SndPan` `SprPriority` `StateTypeSet` `StopSnd` `SuperPause` `TagIn` `TagOut` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Text` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `VictoryQuote` `Width`

Limites dos controladores mais simples: `Offset` desloca somente o sprite desenhado, nunca as caixas de colisão nem a posição; `Null` não faz nada; `VictoryQuote` guarda o seu `value` no lutador para a apresentação da frase de vitória; `ForceFeedback` vira um evento de vibração para o personagem (`self = 1`) ou para o oponente; `StopSnd` e `SndPan` agem nos canais de som iniciados pelo personagem; `DisplayToClipboard`, `AppendToClipboard` e `ClearClipboard` guardam as últimas 10 linhas no lutador (`%d`, `%i`, `%f`, `%%`, `\n` e `\t` no texto, até 5 `params`).

Notas sobre Explod: `ignorehitpause` vale 1 por padrão (use 0 para congelar junto com o hit pause do dono). `postype = left/right/front/back` e `space = screen` mantêm o explod fixo na tela enquanto a câmera se move. `ModifyExplod` aceita `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` e `trans`; `facing` é relativo ao dono. `ExplodBindTime` só altera o tempo de vínculo.

## Triggers suportados

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimElemVar` `AnimExist` `AnimLength` `AnimPlayerNo` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Clamp` `ClsnOverlap` `ClsnVar` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DisplayName` `Dizzy` `DizzyPoints` `DizzyPointsMax` `DrawGame` `E` `Exp` `ExplodVar` `facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `GuardBreak` `GuardPoints` `GuardPointsMax` `HelperIndexExist` `HelperVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `ID` `IfElse` `IkemenVersion` `Index` `InGuardDist` `InputTime` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `LoseKO` `LoseTime` `MajorVersion` `Map` `MatchNo` `MatchOver` `Max` `Min` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `MugenVersion` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `NumText` `P1Dist` `P1Name` `P2BodyDist` `P2Command` `P2Ctrl` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2Power` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalFXVar` `PalNo` `ParentDist` `Physics` `Pi` `PlayerIDExist` `pos` `Power` `PowerMax` `PrevStateNo` `ProjCancelTime` `ProjClsnOverlap` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `ProjVar` `Random` `RandomRange` `RedLife` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `RunOrder` `ScreenPos` `SelfAnimExist` `SelfCommand` `Sign` `Sin` `SpritePlayerNo` `SpriteVar` `Sqrt` `Standby` `StateNo` `StateTime` `StateType` `SysFVar` `SysVar` `Tan` `TeamMap` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `vel` `Win` `WinKO` `WinPerfect` `WinTime`

Faltam (planejado): `HitVel` `PlayerID` `StageVar`.

`P3Name` a `P8Name` comparam com o jogador daquele slot (ordem de slot da partida, não de lado); um slot sem jogador nunca coincide. `P2Command` lê o buffer de comandos do oponente mais próximo, como `Command` faz para o próprio personagem.

Funções extras: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

`HitCount` soma os `numhits` de cada golpe que o atacante acerta. `UniqHitCount` conta cada par distinto `(alvo, id do HitDef)` uma única vez, então três golpes de um mesmo HitDef dão `HitCount = 3` e `UniqHitCount = 1`. Ambos são zerados juntos nas trocas de estado, a menos que `HitCountPersist = 1`.

`GetHitVar(<nome>)` lê os dados do último golpe sofrido (após `HitDef`, defesa ou `HitOverride`). Além dos nomes do MUGEN 1.1 (`type`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.*`, `kill`, `id`, `playerno`), estes nomes do Ikemen retornam valores reais: `recovertime` (ticks restantes até `fall.recovertime` permitir a recuperação, 0 fora de queda), `down.recover` (o `down.recover` do HitDef, padrão 1), `priority`, `facing` (direção do atacante, 1 ou -1), `power` (poder recebido pelo alvo), `isbound` (ao vivo, 1 enquanto o alvo está preso), `attr` (máscara de bits: S=1 C=2 A=4 L=8, NA=64 NT=128 NP=256 SA=512 ST=1024 SP=2048 HA=4096 HT=8192 HP=16384), `guardflag` (máscara de bits: H=1 L=2 A=4), `hitdamage`, `guarddamage`, `hitpower`, `guardpower`, `dizzypoints` (`dizzypoints` do HitDef, padrão o dano do golpe), `guardpoints` (`guardpoints` do HitDef, padrão o dano na defesa), `redlife` (`redlife` do HitDef, valor de golpe ou defesa), `score` (`score` do HitDef, valor de golpe ou defesa), `teamside` (lado do atacante, 1 ou 2), `projid` (-1 se não foi projétil), `playerid` (ID do atacante, igual a `id`), `p1stateno` e `p2stateno` (-1 quando o HitDef não define). Qualquer outro nome registra um aviso no stderr ao carregar a expressão e vale 0.

## Redirecionamentos

`parent` `root` `helper` `target` `partner` `enemy` `enemynear` `playerid` `p2` `stateowner`

`helper(id)` e `target(id)` aceitam um ID opcional; `partner(n)` é o n-ésimo outro jogador não-helper do mesmo lado (padrão 0; um helper redireciona pelo seu root; NaN quando não existe); `enemy(n)` e `enemynear(n)` escolhem o n-ésimo oponente. Exemplo: `trigger1 = enemynear, life < 200`.

## Expressões

- Inteiro vs. float como no M.U.G.E.N: `7 / 2 = 3`, `7.0 / 2 = 3.5`.
- Intervalos: `stateno = [200, 299]`, `(200, 299]`.
- Operadores: `+ - * / % ** ! && || ^^ & | ^ ~ = != < <= > >=` e `:=` em `var(n) := valor`.

## Regras de pausa e congelamento

| Entidade | Hit pause (atacante) / hit shake (defensor) | `Pause` | `SuperPause` |
|---|---|---|---|
| Lutador ou helper | Animação, `Time`, física e timers param; só rodam controllers com `ignorehitpause = 1` | Congelado, exceto quem pausou durante `movetime` e helpers com ticks de `pausemovetime` restantes | Congelado, exceto quem pausou durante `movetime` e helpers com ticks de `supermovetime` restantes |
| Explod | Continua com `ignorehitpause = 1` (padrão); com `0` para enquanto o dono está em hit pause ou hit shake, inclusive o vínculo de posição | Congelado, salvo ticks de `pausemovetime` restantes | Congelado, salvo ticks de `supermovetime` restantes |
| Projétil | Não é afetado pelo hit pause do dono | Congelado, salvo ticks de `pausemovetime` restantes | Congelado, salvo ticks de `supermovetime` restantes |

- O orçamento de `pausemovetime`/`supermovetime` gasta um tick por frame congelado e só pelo seu próprio tipo de pausa.
- `AnimTime`, `AnimElemTime`, `AnimElemNo` e `Time` mantêm o valor durante todo o hit pause ou shake e voltam no primeiro tick livre.
- `HitPauseTime` é o hit pause restante do atacante; `GetHitVar(hitshaketime)` e `HitShakeOver` descrevem o shake do defensor. Os demais campos de `GetHitVar` são uma captura feita no momento do golpe e não mudam durante o shake; o relógio de recuperação (`HitOver`) também espera o fim do shake.
- Testes de regressão: `crates/engine/tests/freeze_rules.rs`.

## Depurar

- `F2` no treino mostra hitboxes.
- `cargo run --release -p ume-engine --example mugen_check -- <pasta>` lista estados e expressões com erro.

## Diagnóstico no carregamento

Cada carregamento de personagem confere os controladores, triggers e parâmetros lidos com o que a engine realmente implementa. Não há lista fixa: cada nome desconhecido é executado uma vez numa partida descartável e marcado quando o despacho não o trata, então o relatório acompanha a engine conforme controladores e triggers são adicionados.

O relatório indica, por arquivo e estado (`.cns`, `.cmd`, `.st`, `.zss`):

| Achado | Significado |
|---|---|
| controlador não suportado | o `type` não tem implementação |
| trigger desconhecido | o nome do trigger não é avaliado pela engine |
| redirecionamento / trigger de texto não suportado | um tipo de redirecionamento (`partner`, ...) ou comparação de texto (`p3name = ...`) que a engine não resolve |
| parâmetro desconhecido | nenhum caminho do controlador lê esse parâmetro |
| flag de AssertSpecial desconhecida | a flag não é reconhecida |

Cada carga imprime um resumo no stderr (`n controladores não suportados, m triggers desconhecidos, ...`), sem repetição por nome; `TRACE=1` acrescenta os nomes com os arquivos e estados que os usam. Defina `UME_QUIET_DIAGNOSTICS=1` para silenciar.

O `mugen_check` imprime o resumo de cada personagem e, com `--strict`, imprime os detalhes e termina com código diferente de zero se algum personagem tiver achados:

```bash
cargo run --release -p ume-engine --example mugen_check -- --strict mugen/chars/kfm
```

O CI roda com `--strict` sobre `crates/engine/tests/data/chars`; um recurso usado pelos personagens de teste que a engine não implementa quebra o build. No código, `ume_engine::diagnostics::audit_cns(arquivo, texto)` devolve os achados de um trecho e `CharDef::diagnostics()` os de um personagem carregado.

## Triggers de introspecção

Triggers que leem de volta o que o personagem criou ou o que ele está desenhando. O último argumento é o nome da variável escrito sem aspas, como no Ikemen GO; um eixo é uma segunda palavra (`pos x`). Uma consulta que não encontra nada (explod, projétil ou caixa inexistente) é indefinida (NaN): qualquer comparação com ela continua indefinida e conta como falsa, então teste a existência com `NumExplod`, `NumProj` ou `HelperIndexExist` quando a diferença importar. Posições e velocidades usam as coordenadas do próprio personagem; as posições seguem a convenção de `Pos` (`x` relativo à câmera, `y` relativo ao chão).

| Trigger | Variáveis |
|---|---|
| `ExplodVar(id, índice, var)` | `id` `time` `removetime` `bindtime` `bindid` `anim` `animelem` `animelemtime` `animtime` `animplayerno` `spriteplayerno` `facing` `ignorehitpause` `layerno` `pausemovetime` `supermovetime` `sprpriority` `pos x/y/z` `vel x/y/z` `accel x/y/z` `scale x/y`. Lê o `índice`-ésimo explod do chamador com esse `id` (`id` abaixo de 0 vale qualquer um) |
| `ProjVar(id, índice, var)` | `projid` `time` `projremovetime` `projhits` `projmisstime` `projpriority` `projsprpriority` `projedgebound` `projstagebound` `lowbound` `highbound` `projhitanim` `projremanim` `projcancelanim` `projremove` `anim` `animelem` `facing` `teamside` `pausemovetime` `supermovetime` `pos x/y/z` `vel x/y/z` `accel x/y/z` `velmul x/y` `remvelocity x/y/z` `scale x/y`. Lê o `índice`-ésimo projétil vivo do chamador com esse `id` |
| `PalFXVar(var)` | `time` (ticks restantes) `add.r/g/b` `mul.r/g/b` `color` `invertall`; prefixo `bg.` ou `all.` para os efeitos de fundo e global. Sem efeito ativo vale a identidade (`add` 0, `mul` 256, `color` 256) |
| `ClsnVar(tipo, índice, borda)` | `tipo` é `clsn1`, `clsn2`, `size` ou `dummy`; `borda` é `back`, `front`, `top` ou `bottom`, relativa ao personagem e à sua direção. `size` é a caixa de corpo de `size.ground.back/front` e `size.height`; `dummy` não tem caixas |
| `ClsnOverlap(tipo, playerid, tipo)` | agora aceita `size` (nos dois lados) e `dummy` (no primeiro, nunca sobrepõe) além de `clsn1` e `clsn2` |
| `ProjClsnOverlap(índice, playerid, tipo)` | a caixa de ataque do `índice`-ésimo projétil do chamador contra uma caixa daquele jogador |
| `HelperVar(var)` | num helper: `id` `helpertype` `keyctrl` `ownpal` `preserve`; indefinido fora de um helper |
| `SpriteVar(var)` | o sprite atual: `group` `image` `width` `height` `xoffset` `yoffset` |
| `AnimElemVar(var)` | o elemento atual da animação: `group` `image` `time` `hflip` `vflip` `xoffset` `yoffset` `xscale` `yscale` `angle` `alphasource` `alphadest` `numclsn1` `numclsn2` |
| `InputTime(tecla)` | ticks em que a tecla está pressionada: `U` `D` `L` `R`, relativas `F` `B`, `N` (nada pressionado) e os botões `a` `b` `c` `x` `y` `z` `s` (maiúscula importa: `B` é trás, `b` é o botão) |
| `AnimLength` `AnimPlayerNo` `SpritePlayerNo` | total de ticks da animação (-1 se nunca termina); número do jogador dono da animação e dos sprites |
| `Index` `RunOrder` `HelperIndexExist(n)` | posição do personagem entre o root e seus helpers (o root é 0); ordem de atualização na partida, começando em 1; se o root tem o `n`-ésimo helper (base 0) |
| `IkemenVersion(major)` `IkemenVersion(minor)` | versão do comportamento do Ikemen GO seguida: 1 e 0 |
| `DisplayName = "x"` `SelfCommand = "x"` | comparações de texto com o nome de exibição e com o buffer de comandos do próprio personagem |

Ainda não implementados, e apontados pelo diagnóstico de carregamento: `BGMVar`, `SoundVar` (o núcleo da engine não guarda estado de áudio), `AnimLoopCount`, `AnimPrefix`, `HelperName`, as variáveis `angle`, `friction`, `xshear`, `drawpal.*` e `shader` de `ExplodVar`, e `attr`, `guardflag`, `hitflag`, `shadow` de `ProjVar`. Um nome de variável desconhecido é apontado uma vez por carregamento como `explodvar(nome)` e vale 0.
