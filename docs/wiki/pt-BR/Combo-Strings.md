# Strings de combo

Cada lutador nativo (os 16 do elenco e os 5 NPCs do Story) tem uma string assinatura de três partes além do gatling padrão. Tudo é dado: as animações `string_*` são montadas com os frames de soco, chute e golpe forte do próprio lutador (mesmas poses e partes de hit), com velocidade, número de hits e recuperação próprios.

| Entrada | Golpe | Função |
|---|---|---|
| `B+x` | `string_a` | Abertura. Também sai do cancel do `jab` no contato |
| `B+a` | `string_b` | Elo do meio. Cancel de `string_a` no contato |
| `B+y` | `string_c` | Finalizador: lançamento, wall bounce ou ground bounce conforme o lutador; pesados ganham armadura |

`string_a` e `string_b` cancelam em `string_c` e nos dois especiais assinatura do lutador no contato. Validado por `native_combo_routes`, `native_roster_lab` e `native_hitbox_audit`.
