Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Analisador de animação e colisão AIR no local sem alocações no heap

- Eliminação de alocações `to_ascii_lowercase()` por linha: `air::parse_air` processa linhas de texto diretamente com comparação ASCII de bytes insensível a maiúsculas (`eq_ignore_ascii_case`), removendo de 5.000 a 20.000 alocações de string por arquivo `.air`.
- Analisador de caixas de colisão em pilha: `parse_box4` analisa quatro coordenadas diretamente em um array fixo `[f32; 4]` na pilha, eliminando alocações de `Vec<f32>` por caixa de colisão.
- Tokenização de parâmetros de quadros em buffer de pilha: Linhas AnimFrame desempacotam até 11 parâmetros separados por vírgula em um array fixo `[&str; 11]` sem alocar `Vec<&str>`.
- Análise de flags de flip e blend no local: Flags `fl` e `bl` são avaliadas diretamente sem alocação de strings em maiúsculas.
