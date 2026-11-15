# Checklist de pré-voo (lint de editor e validação empírica) para personagens

Derivado do estudo estático do Fighter Factory Studio (`docs/wiki/Authoring-Tool-Survey.md`), das **8 regras do linter do motor** ([`crates/engine/src/lint.rs`](../../../../crates/engine/src/lint.rs)) e da auditoria de **2.625 `.def` / 9.130 arquivos `.cns`/`.st`**. Rode antes de homologar qualquer lutador.

## 1. As 8 Regras Estritas de Lint do `ume-engine` (`mugen_check --lint`)

Execute `cargo run --release -p ume-engine --example mugen_check -- --strict --lint <dir_char>` para checar:
1. **`DuplicateParameter`**: parâmetro repetido dentro do mesmo `[State]` ou `[Statedef]` (exceto `triggerall`/`triggerN` e múltiplas atribuições `var(x)`/`fvar(x)` dentro de `VarSet`/`VarAdd`). *No corpus real: 16.810 ocorrências.*
2. **`InvalidValue`**: valores fora dos enums aceitos para `trans` (`default`, `none`, `add`, `add1`, `addalpha`, `sub`), `afterimage, trans` (`none`, `add`, `add1`, `sub`), `postype` (`p1`, `p2`, `front`, `back`, `left`, `right`, `none`) e `space` (`stage`, `screen`).
3. **`NegativeStatePersistent`**: parâmetro `persistent` dentro de `[Statedef -1]`, `-2` ou `-3`. *No corpus real: 12.585 ocorrências.*
4. **`StateNumberMismatch`**: `[State X, ...]` cujo número `X` difere do `[Statedef Y]` pai. *No corpus real: 1.391.086 ocorrências — erro #1 em copy-paste.*
5. **`GarbageLine`**: texto solto sem formato `chave = valor` fora de comentários `;` ou cabeçalhos `[...]`.
6. **`DuplicateAction`**: dois `[Begin Action N]` com o mesmo número `N` no `.air`. *No corpus real: 2.400 ocorrências.*
7. **`EmptyAction`**: `[Begin Action N]` sem nenhuma linha de frame no `.air`. *No corpus real: 144 ocorrências.*
8. **`DuplicateCommandKey`**: chaves repetidas dentro de `[Command]`, `[Defaults]` ou `[Remap]` no `.cmd`.

## 2. Parâmetros obrigatórios e versões

- `[Data]`: `life`, `attack`, `defence`, `fall.defence_up`, `liedown.time`, `airjuggle`.
- Todo `[Statedef]`: `type`, `movetype`, `physics`, `anim` (em helpers ocultos use `anim = 9999` ou `-2` com sprite vazio).
- `attr`, `hitflag` e `guardflag` válidos: use apenas as combinações oficiais (evite repetir atributos).
- Parâmetros de 1.1 (`xscale`, `yscale`, `angle`, interpolação em `.air`, `ModifyExplod`, `CameraZoom`, triggers `CameraPos`, `ScreenWidth`) não existem em 1.0. Declare `mugenversion = 1.0` ou `1.1` no `.def` (evite datas antigas WinMUGEN como `04,14,2001` que ativam o bug `Floor(2**n)`).
- Migre `[Data] volume` (obsoleto) para `volumescale = 100`.
- Redirecionamentos (`Parent,`, `Root,`, `Target,`, `Helper(id),`, `Partner,`, `EnemyNear,`) exigem guarda prévia (`NumTarget`, `NumHelper`, `IsHelper`, `NumPartner`, `NumEnemy`).

## 3. Portabilidade Linux / WebAssembly VFS / Android

- **Case-sensitivity exata**: *287 definições `.def` no corpus* quebram em Linux/Web porque citam `States\System.st` ou `Sprite.sff` com maiúsculas diferentes do arquivo real. Use sempre `/` (barra normal) e grafia idêntica ao disco.
- **Encoding limpo**: salve `.def`, `.cns`, `.cmd`, `.air` e `.zss` em **UTF-8 sem BOM e sem bytes NUL** (alguns arquivos legados em Latin-1/Shift-JIS quebram leitores estritos).
- **Conflitos e sintaxe**: remova marcadores de merge Git (`<<<<<<< HEAD`) e use `=` (nunca `==`) em expressões `.cns`.

## 4. Sprites e paletas

- Sprites exigidas existem: o `[Files] sprite`, o idle (0,0 e animação 0 a 5), retrato `9000,0` (pequeno 25×25) e `9000,1` (grande 120×140) conforme o `.def`.
- Toda sprite tem id, eixo e paleta válidos. Sem paleta sem uso, sem paletas com cores idênticas (deduplicar).
- Paleta de 32 cores só se todas as sprites usadas couberem em 5 bits; senão 256 com índice 0 transparente.
- Importação em lote: nomeie como `grupo_indice` (também `grupo-indice` ou `grupo,indice`); escolha o modo de paleta e faça recorte justo (tight crop) com eixo comum alinhado nos pés via Onion Skin.

## 5. Estados, variáveis, helpers e prevenção dos top bugs

- Liste o uso de `var`, `fvar` e `sysvar` por arquivo e por helper; colisão de índice é bug silencioso.
- `var` fica em `0..59`, `fvar` em `0..39` e `sysvar` em `0..4`; índices acima de `IntPersistIndex`/`FloatPersistIndex` não devem ser usados como temporários. Em **ZSS**, `let` aloca `var(40..59)`.
- **Prevenção de Vazamento de `AfterImage`** (*42% dos arquivos com AfterImage no corpus esquecem*): sempre zere com `AfterImageTime { time = 0 }` ao sofrer dano (`movetype = H`) ou encerrar o super.
- **Prevenção de Vazamento de `Helper`** (*1.833 arquivos no corpus*): proteja spawns com `!NumHelper(id)` e garanta `DestroySelf` por tempo máximo, fim de animação e mudança de estado do `root`.
- **Anti-Freeze em Arremessos** (*271 arquivos com `TargetBind` sem guarda*): todo custom hit state (`movetype = H`) preso por `TargetBind` deve ter `[State X, Failsafe] type = SelfState; trigger1 = !gethitvar(isbound); value = 5050`.
- **Anti-Trava de Winpose**: nunca use `AssertSpecial { flag = roundnotover / timerfreeze }` sem `triggerall = !WinKO`.
- **Underground Juggles**: no `[Statedef -3]`, corrija alvos em queda com `target, pos y > 0` usando `TargetBind` em `ceil(-pos y)`.
- Ao defender um golpe, o motor escolhe o estado de reação pelo `statetype` do defensor: S entra no estado `150`, C no `152`, A no `154`. Defensor com outro `statetype` (ex.: `L`) dispara `"Guard state error: wrong move type"`. Garanta que `150`, `152` e `154` existam.
- `Explod` com `ontop = 1` ignora `sprpriority` e ordena diferente entre MUGEN 1.1 e Ikemen GO; prefira `ontop = 0` com `sprpriority` explícito.

## 6. Sintomas reportados pelo emulador e causa

| Aviso | Causa comum |
| :--- | :--- |
| `changed to invalid state X (from state Y)` | `value` de `ChangeState` digitado errado ou estado removido |
| `Changed to invalid action X` | `ChangeAnim` ou `anim` do Statedef sem `[Begin Action X]` |
| `Changed to invalid animelem` | `AnimElem` maior que o número de frames |
| `Guard state error: wrong move type` | defensor com `statetype` fora de `S`, `C` e `A` no momento do guard |
| `invalid remappal` | grupo ou índice de paleta inexistente |
| `bindid not found for explod` | `BindID` de Explod sem Helper correspondente |
| `Ignoring invalid AI file` | arquivo de IA fora do formato esperado |

## 7. Ferramentas do repositório que cobrem cada item

`mugen_check --strict --lint` (carga, lint de 8 regras e luta headless de 60s), `sprite_audit` e `native_sprite_audit` (sprites), `hover_check` (chão e flutuação), `auto_clsn` e `auto_ground` (caixas e eixo), `scaffold_char` (esqueleto).
