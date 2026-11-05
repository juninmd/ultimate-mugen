Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Avanço de animação em consulta única e atalho na limpeza de alvos

- Desempacotamento de quadro de animação em consulta única em `advance_anim`: Desempacota `(total, loop_start, len, ft)` em uma única consulta a `self.anim()`, eliminando buscas repetidas em HashMap por lutador e explod animado a cada tick.
- Atalho para substituições de tamanho vazias: `fighter::c` consulta diretamente constantes do personagem quando `self.size_overrides.is_empty()`, pulando varreduras lineares na fatia.
- Limpeza de alvos obsoletos com atalho: `matchs::step` pula movimentações de vetores e chamadas de retenção sempre que `self.fighters[k].targets.is_empty()`.
