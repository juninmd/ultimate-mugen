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

`AfterImage` `AfterImageTime` `AllPalFX` `AngleAdd` `AngleDraw` `AngleMul` `AngleSet` `AssertSpecial` `AttackDist` `AttackMulSet` `BGPalFX` `BindToParent` `BindToRoot` `BindToTarget` `ChangeAnim` `ChangeAnim2` `ChangeState` `CtrlSet` `DefenceMulSet` `DestroySelf` `EnvColor` `EnvShake` `Explod` `ExplodBindTime` `GameMakeAnim` `Gravity` `Helper` `HitAdd` `HitBy` `HitDef` `HitFallDamage` `HitFallSet` `HitFallVel` `HitVelSet` `LifeAdd` `LifeSet` `MakeDust` `ModifyExplod` `MoveHitReset` `NotHitBy` `PalFX` `ParentVarAdd` `ParentVarSet` `Pause` `PlayerPush` `PlaySnd` `PosAdd` `PosFreeze` `PosSet` `PowerAdd` `PowerSet` `Projectile` `RemoveExplod` `ScreenBound` `SelfState` `SprPriority` `StateTypeSet` `SuperPause` `TargetBind` `TargetDrop` `TargetFacing` `TargetLifeAdd` `TargetPowerAdd` `TargetState` `TargetVelAdd` `TargetVelSet` `Trans` `Turn` `VarAdd` `VarRandom` `VarRangeSet` `VarSet` `VelAdd` `VelMul` `VelSet` `Width`

Também suportados: `Zoom` `ReversalDef` `HitOverride` `FallEnvShake` `RemapPal` `StopSnd` `SndPan` `VictoryQuote` `ForceFeedback` `Offset` `Null` `DisplayToClipboard` `AppendToClipboard` `ClearClipboard`.

Notas sobre Explod: `ignorehitpause` vale 1 por padrão (use 0 para congelar junto com o hit pause do dono). `postype = left/right/front/back` e `space = screen` mantêm o explod fixo na tela enquanto a câmera se move. `ModifyExplod` aceita `vel`, `accel`, `pos`, `scale`, `bindtime`, `removetime`, `sprpriority`, `ontop`, `under`, `removeongethit`, `ignorehitpause`, `facing`, `vfacing`, `supermovetime`, `pausemovetime` e `trans`; `facing` é relativo ao dono. `ExplodBindTime` só altera o tempo de vínculo.

Notas de helper: um helper congela durante `Pause` e `SuperPause`, a menos que ainda tenha ticks de `pausemovetime` / `supermovetime` ou tenha nascido com `pausebool = 1`, que o faz ignorar qualquer pausa. Helpers com `keyctrl = 1` só recebem os comandos do root enquanto o round é lutado; decidido o round, não recebem nenhum. Na tela de vitória (estado de round 4) os helpers ficam congelados e ocultos.

## Triggers suportados

`Abs` `ACos` `AILevel` `Alive` `Anim` `AnimElem` `AnimElemNo` `AnimElemTime` `AnimExist` `AnimTime` `ASin` `ATan` `AuthorName` `BackEdge` `BackEdgeBodyDist` `BackEdgeDist` `BottomEdge` `CameraPos` `CameraZoom` `CanRecover` `Ceil` `Command` `Cond` `Const` `Const240p` `Const480p` `Const720p` `Cos` `Ctrl` `DrawGame` `E` `Exp` `Facing` `Floor` `FrontEdge` `FrontEdgeBodyDist` `FrontEdgeDist` `FVar` `GameHeight` `GameTime` `GameWidth` `GetHitVar` `HitCount` `HitDefAttr` `HitFall` `HitOver` `HitPauseTime` `HitShakeOver` `HitVel` `ID` `IfElse` `InGuardDist` `IsHelper` `IsHomeTeam` `LeftEdge` `Life` `LifeMax` `Ln` `Log` `Lose` `MatchNo` `MatchOver` `MoveContact` `MoveGuarded` `MoveHit` `MoveReversed` `MoveType` `Name` `NumEnemy` `NumExplod` `NumHelper` `NumPartner` `NumProj` `NumProjID` `NumTarget` `P1Name` `P2BodyDist` `P2Command` `P2Dist` `P2Life` `P2MoveType` `P2Name` `P2StateNo` `P2StateType` `P3Name` `P4Name` `P5Name` `P6Name` `P7Name` `P8Name` `PalNo` `ParentDist` `Pi` `PlayerID` `PlayerIDExist` `Pos` `Power` `PowerMax` `PrevStateNo` `ProjContact` `ProjContactTime` `ProjGuarded` `ProjGuardedTime` `ProjHit` `ProjHitTime` `Random` `RightEdge` `RootDist` `RoundNo` `RoundsExisted` `RoundState` `ScreenPos` `SelfAnimExist` `Sin` `StageVar` `StateNo` `StateType` `SysFVar` `SysVar` `Tan` `TeamMode` `TeamSide` `TicksPerSecond` `Time` `TimeMod` `TopEdge` `UniqHitCount` `Var` `Vel` `Win`

Faltam (planejado): `WinKO` `WinTime` `WinPerfect` `LoseKO` `LoseTime` `ProjCancelTime` `MugenVersion`.

`P3Name` a `P8Name` comparam com o jogador daquele slot (ordem de slot da partida, não de lado); um slot sem jogador nunca coincide. `P2Command` lê o buffer de comandos do oponente mais próximo, como `Command` faz para o próprio personagem.

`TeamMode` retorna 0 (single), 1 (simul, modo Paralelo), 2 (turns, modo Sequência) ou 3 (tag, reservado). Aceita também `TeamMode = Simul`, `TeamMode = turns` e `TeamMode = 1`; no `Text`, `%s` com `TeamMode` puro imprime o nome do modo.

Triggers de equipe e slot: `TeamSize` (membros da equipe; o elenco inteiro no modo Sequência), `TeamLeader` (`PlayerNo` do líder), `NumPlayer` (jogadores da partida, ambos os elencos), `PlayerNo` (slot 1-based, intercalado por lado; helper usa o slot do root), `MemberNo` (posição 1-based na equipe), `PlayerIndexExist(n)` (índice 0-based na lista de jogadores, helpers incluídos), `LastPlayerID` (ID do último jogador ou helper criado), `IntroState` (1 antes da introdução, 2 durante, 0 depois) e `OutroState` (1 com o round decidido, 2 nas poses de vitória, 0 caso contrário).

Funções extras: `Min`, `Max`, `Clamp`, `Sign`, `Sqrt`, `RandomRange`.

`HitCount` soma os `numhits` de cada golpe que o atacante acerta. `UniqHitCount` conta cada par distinto `(alvo, id do HitDef)` uma única vez, então três golpes de um mesmo HitDef dão `HitCount = 3` e `UniqHitCount = 1`. Ambos são zerados juntos nas trocas de estado, a menos que `HitCountPersist = 1`.

`GetHitVar(<nome>)` lê os dados do último golpe sofrido (após `HitDef`, defesa ou `HitOverride`). Além dos nomes do MUGEN 1.1 (`type`, `animtype`, `airtype`, `groundtype`, `damage`, `hitcount`, `guardcount`, `hitshaketime`, `hittime`, `slidetime`, `ctrltime`, `xvel`, `yvel`, `yaccel`, `chainid`, `guarded`, `fall`, `fall.*`, `kill`, `id`, `playerno`), estes nomes do Ikemen retornam valores reais: `recovertime` (ticks restantes até `fall.recovertime` permitir a recuperação, 0 fora de queda), `down.recover` (o `down.recover` do HitDef, padrão 1), `priority`, `facing` (direção do atacante, 1 ou -1), `power` (poder recebido pelo alvo), `isbound` (ao vivo, 1 enquanto o alvo está preso), `attr` (máscara de bits: S=1 C=2 A=4 L=8, NA=64 NT=128 NP=256 SA=512 ST=1024 SP=2048 HA=4096 HT=8192 HP=16384), `guardflag` (máscara de bits: H=1 L=2 A=4), `hitdamage`, `guarddamage`, `hitpower`, `guardpower`, `dizzypoints` (`dizzypoints` do HitDef, padrão o dano do golpe), `guardpoints` (`guardpoints` do HitDef, padrão o dano na defesa), `redlife` (`redlife` do HitDef, valor de golpe ou defesa), `score` (`score` do HitDef, valor de golpe ou defesa), `teamside` (lado do atacante, 1 ou 2), `projid` (-1 se não foi projétil), `playerid` (ID do atacante, igual a `id`), `p1stateno` e `p2stateno` (-1 quando o HitDef não define). Qualquer outro nome registra um aviso no stderr ao carregar a expressão e vale 0.

Triggers de combate e estado (escalados pelo localcoord quando retornam posições ou velocidades):

| Trigger | Resultado |
|---|---|
| `HitDefVar(<nome>)` | Lê o último `HitDef` do personagem: `p1stateno`, `p2stateno`, `priority`, `sparkx`, `sparky`, `sparkno`, `guard.sparkno`, `pausetime.p1`, `pausetime.p2`, `guard.pausetime.p1/.p2`, `ground.velocity.x/.y`, `air.velocity.x/.y`, `down.velocity.x/.y`, `guard.velocity`, `attr` e `guardflag` (máscaras de bits como no `GetHitVar`), `id`, `chainid`, `damage`, `guard.damage`, `hittime`, `numhits`, `fall`, `yaccel`. `pausetime` e `*.velocity` sem sufixo valem o primeiro componente. Nomes desconhecidos retornam 0 |
| `MoveHitVar(<nome>)` | Lê o último golpe que este personagem acertou: `id` e `playerno` do alvo, `sparkx`, `sparky` (ponto de contato relativo ao atacante), `cornerpush` (recuo aplicado ao atacante no canto), `overridden` (o golpe foi absorvido por um `HitOverride`), `frame` (1 no tick seguinte ao golpe) |
| `MoveCountered` | Como `MoveHit`, mas só quando o alvo estava em ataque (`movetype = A`) ao ser atingido. Zerado junto com `MoveHit` |
| `HitByAttr = SCA, NA, SA` | 1 quando o último golpe sofrido combina com a lista de atributos (mesma sintaxe do `HitDefAttr`) |
| `HitOverridden` | 1 no tick seguinte a um `HitOverride` absorver um golpe |
| `PrevStateType = S`, `PrevMoveType = A` | Tipo de estado e de movimento do estado anterior ao atual |
| `PrevAnim` | Número da animação antes da última troca de animação |
| `SelfStateNoExist(n)` | 1 quando os arquivos do próprio personagem definem o estado `n` |
| `InRunState` | 1 nos estados de corrida padrão (100, 105) |
| `InCustomState` | 1 enquanto executa um estado de outro personagem (`p2getp1state`) |
| `InCustomAnim` | 1 enquanto toca uma animação de outro personagem (`ChangeAnim2`) |
| `IsAsserted(<flag>)` | 1 quando a flag foi declarada neste tick com `AssertSpecial`. Compartilha a tabela de flags do `AssertSpecial` (flags do personagem e `TimerFreeze`, `Intro`, `RoundNotOver`) |
| `JugglePoints` | Pontos de juggle restantes: o `data.airjuggle` do atacante (o próprio quando não está apanhando) menos os pontos já gastos |
| `HitVel X` / `HitVel Y` | Velocidade definida pelo último golpe sofrido |
| `ParentExist` | 1 enquanto o pai deste helper ainda existe |
| `AirJumpCount` | Saltos aéreos feitos desde o pouso |
## Triggers de borda da tela

`LeftEdge` e `RightEdge` são absolutos: sempre devolvem a borda esquerda e a direita da tela. `FrontEdge` e `BackEdge` dependem de `Facing`: quem olha para a direita lê `FrontEdge = RightEdge` e `BackEdge = LeftEdge`, e quem olha para a esquerda lê o contrário. Os quatro usam as unidades do `localcoord` do personagem e as mesmas bordas de `FrontEdgeDist` e `BackEdgeDist`, então `(FrontEdge - X) * Facing` é igual a `FrontEdgeDist` (com `X` a posição do personagem no mundo).

## Distâncias entre localcoords diferentes

Todo trigger de posição e distância converte o resultado do espaço do mundo para o localcoord do personagem que o avalia (`P2Dist`, `P2BodyDist`, `RootDist`, `ParentDist`, `ScreenPos`, `LeftEdge`/`RightEdge`/`FrontEdge`/`BackEdge` e a família `...EdgeDist`). Numa luta 1280x720 contra 320x240, o mesmo vão aparece 4 vezes maior para o personagem HD. `P2BodyDist` subtrai as larguras frontais dos dois depois de convertê-las para o mundo, então vale 0 quando as caixas se tocam, seja qual for o localcoord de cada personagem. `P2Dist` e `P2BodyDist` dependem apenas das duas posições: a câmera nunca interfere, inclusive nos estados customizados de um arremesso.
## Zoom da câmera

`Zoom` muda a escala da câmera (1 = normal, acima de 1 = mais perto). A câmera é determinística: avança um passo por tick, sem relógio.

| Parâmetro | Padrão | Significado |
| --- | --- | --- |
| `scale` | 1 | Escala pedida |
| `time` | 1 | Ticks em que o pedido continua ativo; dispare a cada tick para mantê-lo |
| `delta` | 0 | Somado à escala pedida a cada tick do pedido (uma rampa) |
| `pos` | 0, 0 | Deslocamento do foco da câmera no localcoord do jogador |
| `lag` | 0 | 0 vai à escala pedida em um tick; valores próximos de 1 suavizam (`1 - lag` da distância restante por tick) |
| `zoomindelay` | 0 | Ticks de espera antes de um zoom-in (`scale > 1`) começar |
| `bound` | 1 | 1 mantém a escala dentro dos limites do cenário (`zoomin` como maior escala, `1 / zoomout` como menor, ampliada pelo zoom-out de equipes) e o deslocamento dentro dos limites do cenário; 0 só limita a escala a 0.25..4 e ignora os limites do cenário |

Quando o pedido termina, a câmera volta à escala normal usando `zoominspeed` / `zoomoutspeed` do cenário.

`CameraZoom` devolve a escala atual da câmera. `ZoomVar(<nome>)` expõe o estado do zoom: `scale` (atual), `target` (escala pedida), `active`, `time` (ticks restantes), `posx`, `posy`, `lag`, `delta`, `delay`, `bound`, `min` e `max` (limites de escala permitidos pelo cenário agora). Todos os nomes valem 0 (ou a escala atual em `target`) quando nenhum `Zoom` está ativo.

## Redirecionamentos

`parent`, `root`, `helper(id)`, `target(id)`, `partner(n)` (o n-ésimo outro jogador não-helper do mesmo lado, padrão 0; um helper redireciona pelo seu root; NaN quando não existe), `enemy(n)`, `enemynear(n)`, `playerid(id)`, `p2`. Exemplo: `trigger1 = enemynear, life < 200`.

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
