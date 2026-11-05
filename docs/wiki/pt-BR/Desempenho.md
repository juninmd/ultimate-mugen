Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Parâmetros de shake no HitDef sem alocação e correspondência de tipos no local

- Resolução de parâmetros de shake sem alocação: `hitdef::build` usa consultas diretas de strings estáticas (`parse_shake`) em vez de formatar 10 chaves dinâmicas no heap (`{prefix}.freq`, `{prefix}.time`, `{prefix}.ampl`, etc.) a cada instanciação de `HitDef`.
- Análise de `animtype` e `groundtype` no local: `hitdef::animtype` e `hitdef::groundtype` comparam tokens padrão de animação/chão do MUGEN usando `eq_ignore_ascii_case` diretamente em fatias `&str` sem alocar strings em minúsculas.
