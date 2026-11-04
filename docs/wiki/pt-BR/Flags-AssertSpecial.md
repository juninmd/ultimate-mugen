# Flags do AssertSpecial

`AssertSpecial` aceita `flag`, `flag2`, `flag3`, `flag4` e `flag5` (uma lista separada por vírgulas em um único parâmetro também funciona) e um `enabled` opcional (padrão 1). Os nomes não diferenciam maiúsculas de minúsculas. O conjunto completo do Ikemen GO v1.0.0 é entendido: 58 flags por personagem e 19 flags globais, mais a extensão `Immovable` da engine.

Uma flag vale para o tick em que é afirmada. Durante o hit pause as flags afirmadas antes da pausa continuam ligadas (comportamento do Ikemen); `AnimateHitPause` precisa ser afirmada de novo a cada tick. `IsAsserted(<flag>)` lê a mesma tabela, então `IsAsserted(NoWalk)` e `IsAsserted(TimerFreeze)` funcionam.

Uma flag fora da tabela é ignorada. O carregador imprime uma linha `AssertSpecial flags not recognised and ignored` por personagem e o `mugen_check` imprime uma linha `UNKNOWN-FLAGS`.

## Flags globais (19)

| Flag | Efeito |
|---|---|
| `Intro` | Segura o round na introdução (limite de 10 s) |
| `RoundNotOver` | Segura o estado de fim de round (limite de 15 s) |
| `TimerFreeze` | O cronômetro do round para |
| `RoundFreeze` | O cronômetro e o relógio do estado do round param |
| `RoundNotSkip` | O atalho de pular o face-off do primeiro round com um botão de ataque fica desligado |
| `CameraFreeze` | A câmera não se move |
| `GlobalNoKO` | Ninguém leva KO; a vida para em 0 e o round continua |
| `GlobalNoShadow` | Sem sombras nem reflexos |
| `NoKOSlow` | Sem câmera lenta no KO |
| `NoKOSnd` | Sem som de KO |
| `NoMusic` | A música é silenciada |
| `NoBG` / `NoFG` | As camadas de trás / da frente do cenário não são desenhadas |
| `NoBarDisplay` | Todo o HUD da luta (barras, cronômetro, quadros dos parceiros, contador de combo) some |
| `NoTimeDisplay` | O cronômetro some |
| `SkipFightDisplay` / `SkipRoundDisplay` / `SkipKODisplay` / `SkipWinDisplay` | O anúncio "FIGHT!", "ROUND n", "K.O." ou de vitória não aparece |

## Flags por personagem (58 + Immovable)

| Flag | Efeito |
|---|---|
| `Invisible`, `NoShadow` | O sprite ou a sombra não é desenhado |
| `Unguardable` | Os ataques do jogador não podem ser defendidos |
| `NoStandGuard`, `NoCrouchGuard`, `NoAirGuard` | O jogador não defende nesse tipo de estado e não entra no estado de defesa |
| `NoJuggleCheck` | Os pontos de juggle não são cobrados dos golpes do jogador |
| `NoAutoTurn`, `NoWalk`, `NoJump`, `NoAirJump`, `NoCrouch`, `NoStand` | Desligam as transições embutidas de virar, andar, pular, pulo duplo, agachar e levantar |
| `NoHardcodedKeys` | Desliga todas as transições embutidas por tecla (andar, pular, agachar, defender) |
| `AutoGuard` | Defende como se "trás" estivesse pressionado |
| `NoKO` | O jogador não leva KO ao chegar a 0 de vida |
| `NoHitDamage`, `NoGuardDamage` | Golpes / golpes defendidos não tiram vida do jogador |
| `NoGuardKO` | Um golpe defendido não mata o jogador |
| `NoKOFall` | O golpe que dá KO não força a queda |
| `NoKOVelocity` | O golpe que dá KO não soma a velocidade extra de KO |
| `NoFallHitFlag` | O jogador ignora o hitflag `F` enquanto cai |
| `NoCornerPush` | O jogador não é empurrado para trás após acertar um oponente no canto |
| `Immovable` | Extensão da engine: a pushbox do oponente não consegue mover o jogador |
| `AnimateHitPause` | A animação continua durante o hit pause |
| `AnimFreeze` | A animação para |
| `NoInput` | Botões e direções são zerados |
| `PostRoundInput` | Os comandos são lidos depois do fim do round |
| `RunFirst`, `RunLast` | Os estados do jogador rodam antes / depois dos demais no próximo tick |
| `NoDestroySelf` | `DestroySelf` não faz nada |
| `NoMakeDust` | O efeito de poeira embutido não é criado |
| `NoAILevel` | `AILevel` retorna 0 para o jogador |
| `DrawUnder` | O jogador é desenhado atrás dos outros lutadores |
| `NoLifeBarDisplay`, `NoPowerBarDisplay`, `NoFaceDisplay`, `NoNameDisplay`, `NoWinIconDisplay`, `NoComboDisplay` | A parte correspondente do HUD some |

Registradas e legíveis por `IsAsserted`, mas a engine ainda não tem a mecânica correspondente: `NoAIButtonJam`, `NoAICheat`, `NoBrake`, `NoDizzyPointsDamage`, `NoGuardPointsDamage`, `NoRedLifeDamage`, `NoScore`, `NoFaceP2`, `NoFallCount`, `NoFallDefenceUp`, `NoFastRecoverFromLieDown`, `NoGetUpFromLieDown`, `NoIntroReset`, `NoLifeBarAction`, `NoGuardBarDisplay`, `NoStunBarDisplay`, `NoTurnTarget`, `ProjTypeCollision`, `SizePushOnly`.
