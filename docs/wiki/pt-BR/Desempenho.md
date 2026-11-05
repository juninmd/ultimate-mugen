Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Consultas de seções INI sem alocação e análise de constantes CNS em pilha

- `Section::get` e `Section::get_all` sem alocação: `ini::Section::get` e `ini::Section::get_all` comparam chaves diretamente com `eq_ignore_ascii_case` sem alocar instâncias de `String` em minúsculas a cada consulta.
- `find_section` sem alocação: `ini::find_section` utiliza `eq_ignore_ascii_case` diretamente no nome da seção sem alocar strings em minúsculas.
- Análise de constantes CNS em pilha: `cns::parse_constants` desempacota vetores numéricos separados por vírgula em um array `[f64; 4]` na pilha, eliminando alocações de `Vec<f64>` em cada linha de constantes.
- Vetor de comandos pré-alocado: `cns::parse_cmd` pré-aloca a capacidade de `cmds` para `secs.len()`.
