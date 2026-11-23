# Strings de combo

Cada lutador nativo (os 16 do elenco e os 5 NPCs do Story) tem uma string assinatura de três partes além do gatling padrão. Tudo é dado: as animações `string_*` são montadas com os frames de soco, chute e golpe forte do próprio lutador (mesmas poses e partes de hit), com velocidade, número de hits e recuperação próprios.

| Entrada | Golpe | Função |
|---|---|---|
| `B+x` | `string_a` | Abertura. Também sai do cancel do `jab` no contato |
| `B+a` | `string_b` | Elo do meio. Cancel de `string_a` no contato |
| `B+y` | `string_c` | Finalizador: lançamento, wall bounce ou ground bounce conforme o lutador; pesados ganham armadura |

`string_a` e `string_b` cancelam em `string_c` e nos dois especiais assinatura do lutador no contato. Validado por `native_combo_routes`, `native_roster_lab` e `native_hitbox_audit`.

## Mini combos com a mesma tecla

Apertar o mesmo botão de ataque de novo depois de acertar encadeia o próximo golpe, sem direção extra. Cada golpe da cadeia é um `normal` com `rekka` (`after`, `window`, `contact: true`): só dispara a partir do golpe de origem no contato, nunca do neutro. As animações reutilizam os sprites pixel art existentes do lutador em ordens e tempos novos.

| Tecla | Cadeia |
|---|---|
| `x` | `jab` > `jab2` > `jab3` (finalizador com empurrão leve, cancela nos dois especiais) |
| `a` | `lkick` > `lkick2` > `lkick3` |
| `y` | `strong` > `strong2` |
| `x` agachado | `cjab` > `cjab2` |
| `x` no ar | `ajab` > `ajab2` |

O `speed` do lutador (> 1 encurta cada frame) e a velocidade de andar/correr para frente subiram em todo o elenco; o chefe final acompanha para continuar mais rápido. A velocidade de recuo não mudou. Validado por `cargo test --release -p ume-engine --test it native_same_button_chains::`.

## Variantes de projétil

Todo lutador do elenco tem uma variante lenta e pesada e uma rápida e baixa do projétil assinatura, com os mesmos efeitos: `~D, DB, B, y` (lenta, dois hits, longo alcance) e `~D, DB, B, x` (rápida, baixa, acerta agachado). Quem já usa o input pega o primeiro livre entre `~D, DF, F, y` e `~D, DF, F, a`.
