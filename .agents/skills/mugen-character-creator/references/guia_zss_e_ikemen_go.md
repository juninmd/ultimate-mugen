# Guia Prático de Ikemen GO, ZSS (Zantei State Script) e Compatibilidade Dual

Baseado nos **58 arquivos `.zss` reais** do corpus (`kfm_zss`, `kfm_zaxis`, `ikemenarmor`, `glow.zss`, `dizzy.zss`, `guardbreak.zss`, `tag.zss`) e validado diretamente contra o compilador ZSS do `ultimate-mugen` ([`crates/engine/src/zss/mod.rs`](../../../../crates/engine/src/zss/mod.rs)) e módulos Ikemen ([`crates/engine/src/ikemen/`](../../../../crates/engine/src/ikemen/mod.rs)).

---

## 1. Diferenças Fundamentais entre `.cns` e `.zss`

| Aspecto | CNS Clássico | ZSS (Ikemen GO / UME) |
| :--- | :--- | :--- |
| **Comentários** | `;` até o fim da linha | `#` até o fim da linha (NUNCA use `;` para comentar!) |
| **Parâmetros de Sctrl** | 1 por linha (`key = val`) | Dentro de chaves separados por `;` ou quebra de linha: `sctrl{k: v; k2: v2}` |
| **Atribuição em Params** | `key = value` | `key: value` (`:` em vez de `=`) |
| **Cabeçalho Statedef** | Chave por linha abaixo do `[Statedef N]` | Tudo inline no colchete: `[StateDef 200; type: S; movetype: A; physics: S; anim: 200;]` |
| **Condicionais** | `triggerall` + `trigger1..N` por bloco | Blocos estruturados `if cond { ... } else if cond2 { ... } else { ... }` e `switch/case/default` |
| **Loops & Funções** | Inexistentes (precisava de estados repetitivos) | `for i = 0; i < N; i = i + 1 { ... }`, `while cond { ... }`, `break`/`continue` e `[Function Nome(args)] { ... }` |
| **Escopo de Blocos** | Repetir `ignorehitpause = 1` em cada `[State]` | Envolver dezenas de comandos em `ignoreHitPause { ... }` ou `persistent(0) { ... }` |
| **Variáveis** | Apenas `var(0..59)` e `fvar(0..39)` globais | Variáveis nomeadas locais (`let x = 1;`, lidas como `x` no UME) + mapas `mapSet`/`mapAdd` e `map()`/`teamMap()` |

---

## 2. Recursos Avançados e Regras do Compilador ZSS

### 2.1. Variáveis Locais (`let`), Funções (`[Function]`) e Loops (`for` / `while`)
```zss
[Function ApplyScaleBoost(base, bonus)]
{
    let sc = base + (bonus * 0.25);
    mapSet{map: "last_scale"; value: sc}
}

[StateDef 200; type: S; movetype: A; physics: S; anim: 200; ctrl: 0;]
if time = 0 {
    call ApplyScaleBoost(0.5, power / 1000.0);
}
```
- **Regras no compilador do `ultimate-mugen` ([`zss/mod.rs`](../../../../crates/engine/src/zss/mod.rs))**:
  - Variáveis locais `let` são alocadas regressivamente em slots internos **`var(40..59)`** (`LOCALS_MIN = 40`, `LOCALS_TOP = 59`, máx. **20 variáveis locais** simultâneas) e referenciadas diretamente pelo nome (`x`, sem prefixo `$`). Se seu `.cns` legado usa `var(40..59)`, defina `IntPersistIndex = 40` ou migre para `mapSet`.
  - `[Function Nome(p1, p2)] { ... }` envolve o corpo em chaves `{ ... }` e é expandida inline via `call Nome(a, b);` (profundidade máx. **8**). No Ikemen GO 0.99 upstream também existe a forma de retorno `[Function Nome(a) ret]` com `$ret`.
  - Loops `for init; cond; step { ... }` e `while cond { ... }` são desenrolados estaticamente até **`LOOP_CAP = 32` iterações** por tick.

### 2.2. Mapas Semânticos (`mapSet`, `map()`, `teamMap()`)
Eliminam 100% das colisões de índice `var(N)` entre personagens e módulos universais (como armadura, glow, tag e dizzy):
```zss
# Forma portátil (Ikemen GO + ultimate-mugen)
mapSet{map: "aura_active"; value: 1}
mapAdd{map: "dizzy_timer"; value: -1}

# Leitura de mapa local e mapa compartilhado de equipe (Simul / Tag)
if map(aura_active) > 0 || teamMap(team_super_ready) > 0 { ... }
```
- Nota: o açúcar sintático `map(nome) := valor;` é específico de versões recentes do Ikemen GO; use `mapSet` / `mapAdd` para compatibilidade garantida também no `ume-engine`.
- No `ume-engine`, [`MapStore`](../../../../crates/engine/src/ikemen/mod.rs) mantém mapa determinístico por lutador compatível com rollback (GGRS).

### 2.3. Estados Globais `[StateDef -4]` e `[StateDef +1]`
- **`[StateDef -4]`** (Ikemen GO & UME): Executa **mesmo durante `Pause`/`SuperPause` e inclui Helpers** (diferente do `-2`/`-3`). Ideal para detecção de Tag/Turn order, limpeza universal e sistemas globais de engine.
- **`[StateDef +1]`** (Ikemen GO): Executa **antes** do `-1` e `-2`. Muito utilizado por módulos globais (`armor.zss`, `glow.zss`). *(Para portabilidade máxima no UME e Ikemen GO, prefira `[StateDef -4]` ou `[StateDef -2]` com guardas `!isHelper`)*.

---

## 3. Sistemas Nativos Ikemen GO Suportados pelo Motor

| Sistema | Controladores (`Sctrl`) | Gatilhos (`Trigger`) | Observações Práticas |
| :--- | :--- | :--- | :--- |
| **Red Life (Vida Recuperável)** | `RedLifeSet`, `RedLifeAdd` | `redLife` | Controla vida recuperável ao defender/trocar em Tag (`tag.zss`) |
| **Dizzy / Stun** | `DizzyPointsSet`, `DizzyPointsAdd`, `DizzySet` | `dizzy`, `dizzyPoints`, `dizzyPointsMax` | Configure `[Data] dizzypoints = 1000` e ação `5300` |
| **Guard Break** | `GuardPointsSet`, `GuardPointsAdd`, `GuardBreakSet` | `guardBreak`, `guardPoints`, `guardPointsMax` | Configure `[Data] guardpoints = 1000`; zera pontos ao defender |
| **Tag / Standby** | `TagIn`, `TagOut` | `standby`, `teamLeader`, `numPartner` | Troca líder ativo e envia parceiro para fora da tela |
| **Texto na Tela** | `Text`, `RemoveText` | `numText` | Substitui hacks de fontes em `Explod` para UI de combos/tutoriais |
| **Edição Dinâmica** | `ModifyProjectile`, `ModifySnd`, `ModifyPlayer`, `ModifyHitDef` | `redirectId`, `playerIdExist(id)` | Permite teleguiar projéteis e fazer fade de áudio com `volumescale` |
| **Introspecção de Colisão** | `OverrideClsn`, `TransformClsn`, `ResizeClsn` | `clsnOverlap`, `isAsserted(flag)` | Permite clash preciso entre hitboxes sem depender de gambiarra de `Helper` |

---

## 4. Compatibilidade Dual: MUGEN 1.0/1.1 + Ikemen GO no Mesmo Personagem

1. **Truque do Slot Alto `st90=` no `.def`**:
   - O MUGEN clássico (1.0/1.1) lê apenas `st` e `st1`..`st9` (até 10 slots) e ignora `st90`. O Ikemen GO e o UME carregam slots numéricos altos:
   ```ini
   [Files]
   st   = states/system.cns
   st1  = states/normals.cns
   st90 = states/ikemen_addon.zss ; Ignorado no MUGEN clássico, carregado no Ikemen GO!
   ```
   - Dentro de `ikemen_addon.zss` no `[StateDef -4]`, você pode gerar um marcador (ex.: `Explod` ID `999900` invisível ou `map(is_ikemen) := 1`), permitindo que seus `.cns` ativem extras (como `RedLife`, `TagIn`, `Text`) somente quando `NumExplod(999900) > 0`.
2. **Ordenação de `Explod` com `ontop = 1`**:
   - Em **MUGEN 1.1**, explods com `ontop = 1` reaproveitam slots liberados anteriores. Em **Ikemen GO**, são anexados ao final da lista (`sprpriority` é ignorado quando `ontop = 1`). Nunca dependa de `ontop = 1` para sobrepor dois efeitos específicos entre si — use `ontop = 0` com `sprpriority` distintos (ex.: `5` vs `6`).
3. **`movelist.dat` e `[Palette Keymap]`**:
   - Adicionar `movelist = movelist.dat` no `[Files]` do `.def` garante que o menu de pausa do Ikemen GO exiba golpes, comandos e custo de barra automaticamente.
4. **Escala com `const240p()`, `const480p()`, `const720p()`, `const1080p()`**:
   - No `ume-engine` ([`triggers.rs`](../../../../crates/engine/src/triggers.rs)), `const240p(v)` calcula `v * local_w / 320.0`, ajustando velocidades e posições automaticamente quer o lutador use `localcoord = 320,240`, `428,240` (16:9) ou `1280,720`.
