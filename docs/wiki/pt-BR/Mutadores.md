# Mutadores de Partida

Mutadores mudam as regras de uma partida sem alterar os dados dos lutadores. O primeiro mutador é a **gravidade**.

| Campo | Padrão | Faixa | Efeito |
|---|---|---|---|
| `gravity` | `1.0` | `0.25` a `4.0` | Escala a queda de lutadores no ar e a queda de lutadores no chão que ficaram suspensos |

Valores fora da faixa, ou não finitos, são recusados por `Mutators::with_gravity`, que retorna `None`. Assim um valor ruim nunca chega à simulação.

Escopo nesta versão: a escala vale para o `movement.yaccel` do próprio lutador. A gravidade de hit (`yaccel` do HitDef) ainda não é escalada.

Os dois peers de uma partida com rollback precisam usar os mesmos mutadores. Eles são definidos em `Match` antes do primeiro tick (`m.mutators = ...`). O padrão `1.0` não muda nada, então partidas existentes não mudam.

Código: `crates/engine/src/mutators.rs`, aplicado em `physics_step` (`crates/engine/src/matchs.rs`). Testes: `crates/engine/tests/mutators.rs`.
