# Checklist de pré-voo (lint de editor) para personagens

Derivado do estudo estático do Fighter Factory Studio (ver `docs/wiki/Authoring-Tool-Survey.md`). Rode antes de homologar qualquer lutador.

## 1. Parâmetros obrigatórios

- `[Data]`: `life`, `attack`, `defence`, `fall.defence_up`, `liedown.time`, `airjuggle`.
- Todo `[Statedef]`: `type`, `movetype`, `physics`, `anim`.
- Sem parâmetro duplicado no mesmo bloco. Sem atributo ou flag de hit repetidos (`attr`, `hitflag`, `guardflag`).
- `attr` e `hitflag` válidos: use só as combinações da tabela do manual.

## 2. Parâmetros por versão

- Parâmetros de 1.1 (`xscale`, `yscale`, `angle`, interpolação em `.air`, `ModifyExplod`, `camerazoom`, triggers `CameraPos`, `ScreenWidth`) não existem em 1.0. Declare `mugenversion` no `.def` e não use o que a versão não tem.
- Parâmetros marcados como obsoletos (ex.: `volume` em `[Data]`, trocado por `volumescale`) devem ser migrados.
- Redirecionamentos (`Parent,`, `Root,`, `Target,`, `Helper(id),`) só valem onde o alvo existe; guarde com `NumTarget`, `NumHelper`, `IsHelper`.

## 3. Sprites e paletas

- Sprites exigidas existem: o `[Files] sprite`, o idle (0,0 e animação 0 a 5), retrato e ícone de seleção conforme o `.def`.
- Toda sprite tem id, eixo e paleta válidos. Sem paleta sem uso, sem paletas com cores idênticas (deduplicar).
- Paleta de 32 cores só se todas as sprites usadas couberem em 5 bits; senão 256.
- Importação em lote: nomeie como `grupo_indice` (também `grupo-indice` ou `grupo,indice`); escolha o modo de paleta (própria, forçar, trocar) e o recorte antes ou depois do eixo.
- Exporte com eixo comum para alinhar, e confira com onion skin que os pés ficam na mesma linha.

## 4. Estados, variáveis e helpers

- Liste o uso de `var`, `fvar` e `sysvar` por arquivo e por helper; um mesmo índice com dois significados é bug.
- `var` fica em 0 a 59, `fvar` em 0 a 39 e `sysvar` em 0 a 4; índices de persistência (`IntPersistIndex`, `FloatPersistIndex`) não colidem com variáveis temporárias.
- Todo `ChangeState` e `SelfState` aponta para um estado que existe, toda `ChangeAnim` para uma ação que existe, todo `AnimElem` para um elemento que existe.
- Estado de guard exige `movetype` coerente (`H` ao apanhar, `A` ao atacar); caso contrário o motor avisa "wrong move type".
- `RemapPal` em Helper e Explod aponta para grupos e índices de paleta reais.
- `Explod` com `bindtime` ou bind id inexistente não prende em nada; confira o `id`.
- `PalFX`: `time`, `add`, `mul` dentro de faixa válida.

## 5. Sintomas reportados pelo emulador e causa

| Aviso | Causa comum |
| :--- | :--- |
| changed to invalid state X (from state Y) | `value` de `ChangeState` digitado errado ou estado removido |
| Changed to invalid action X | `ChangeAnim` ou `anim` do Statedef sem `[Begin Action X]` |
| Changed to invalid animelem | `AnimElem` maior que o número de frames |
| Guard state error: wrong move type | estado de guard (120 a 155) sem `movetype = H` |
| invalid remappal | grupo ou índice de paleta inexistente |
| bindid not found for explod | `BindID` de Explod sem Helper correspondente |
| Ignoring invalid AI file | arquivo de IA fora do formato esperado |

## 6. Ferramentas do repositório que cobrem cada item

`mugen_check` (carga e parse), `sprite_audit` e `native_sprite_audit` (sprites), `hover_check` e `stuck_check` (chão e travas), `auto_clsn` e `auto_ground` (caixas e eixo), `scaffold_char` (esqueleto).
Lacunas conhecidas (viram issues): schema de obrigatório e por versão, relatório de `var/fvar/sysvar`, dedupe e uso de paletas, importação por nome `grupo_indice`.
