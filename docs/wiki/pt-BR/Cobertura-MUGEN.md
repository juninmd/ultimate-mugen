# Cobertura de controllers e triggers MUGEN

O teste de cobertura está em `crates/engine/tests/mugen_coverage_matrix.rs`. Ele verifica que cada controller e trigger listado do MUGEN 1.1 tem implementação na engine, usando o mesmo teste de despacho que relata nomes não suportados ao carregar um personagem.

## Status

| Área | Nomes oficiais verificados | Implementados | Observações |
| --- | --- | --- | --- |
| Controllers de estado | 90 | 90 | Coberto por teste. |
| Triggers (forma de valor) | 92 | 92 | Coberto por teste. |
| Triggers (forma de comparação, ex.: `StateType = S`, `MoveType = A`, `AnimElem = 3`, `Command = "x"`, `Name = "..."`) | 10 | 10 | Tratados na camada de expressões, cobertos por `mugen11_triggers.rs`. Fora da lista do teste. |
| Redirecionamentos (`enemy`, `enemynear`, `partner`, `helper`, `parent`, `root`) | 6 | 6 | Cobertos por `enemy_selection.rs` e `helper_parity.rs`. |
| Extensões do Ikemen GO | muitos | a maioria | Listadas em `CNS-Scripts.md`. |

## Itens parciais conhecidos

- `DestroySelf`: `recursive` também destrói os helpers que ele criou, e `removeexplods` remove os explods que ele possui (semântica conforme a documentação do Ikemen GO).
- `Helper`: os parâmetros `inheritjuggle` e `size.xscale`/`sprpriority` não são lidos no caminho de criação.
- `Explod`: `shadow` e `layerno` não são lidos.
- `PlaySnd`, `Pause`, `VarSet` e `Trans`: a verificação de parâmetros pelo teste de despacho é inconclusiva; precisam de uma revisão manual contra a documentação do MUGEN antes de afirmar cobertura total de parâmetros.
- `MakeDust`: `spacing` é respeitado (poeira a cada `spacing` ticks, padrão 1).

## Como verificar um nome novo

1. Adicione o nome à lista correspondente em `mugen_coverage_matrix.rs`.
2. Rode `cargo test -p ume-engine --test mugen_coverage_matrix`.
3. Se falhar, implemente o controller ou trigger e documente em `CNS-Scripts.md`.
