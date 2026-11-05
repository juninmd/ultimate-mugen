Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Pipeline de avaliação de controladores e gatilhos com zero alocações

- Indexação de alvos sem alocação no heap: `controllers::target_indices` retorna buffer em pilha `([usize; 8], usize)` em vez de `Vec<usize>`, eliminando alocações em controladores de alvo (`targetstate`, `targetlifeadd`, `targetpoweradd`, `targetfacing`, `targetbind`, etc.).
- Resolução de flags AssertSpecial sem alocação: `flags::names_in` e `flags::lookup` operam diretamente em fatias `&str` com comparação ASCII insensível a maiúsculas/minúsculas (`eq_ignore_ascii_case`), removendo alocações de strings a cada quadro.
- Constantes de velocidade em buffer na pilha: `triggers::Ctx::const_val` formata a chave em um buffer de 64 bytes para buscar sufixos `.x` sem alocação no heap via `format!("{k}.x")`.
- Atribuições de variáveis tipadas: `Match::assigns` armazena o enum `AssignKind` em vez de alocar uma `String` na atribuição de variáveis de gatilhos.
