Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Consultas de seções INI sem alocação e análise de constantes CNS em pilha

- `Section::get` e `Section::get_all` sem alocação: `ini::Section::get` e `ini::Section::get_all` comparam chaves diretamente com `eq_ignore_ascii_case` sem alocar instâncias de `String` em minúsculas a cada consulta.
- `find_section` sem alocação: `ini::find_section` utiliza `eq_ignore_ascii_case` diretamente no nome da seção sem alocar strings em minúsculas.
- Análise de constantes CNS em pilha: `cns::parse_constants` desempacota vetores numéricos separados por vírgula em um array `[f64; 4]` na pilha, eliminando alocações de `Vec<f64>` em cada linha de constantes.
- Vetor de comandos pré-alocado: `cns::parse_cmd` pré-aloca a capacidade de `cmds` para `secs.len()`.
- **Onda 42 - Caminho Rápido de Efeitos de Paleta, Buffers de Quadro em Pilha e Consultas Zero-Alloc**:
  - PalParams::apply: Caminho rápido para transformações idênticas e sem rotação de matiz/inversão, ignorando trigonometria e desenrolando canais de cores.
  - PalFx::from_controller: Eliminação de alocações na heap para buscas de parâmetros de controladores usando chaves em buffer de pilha.
  - Match::tick: Remoção de alocações dinâmicas de Vec<usize> por tick para waken e 	ag_requests através de arrays em pilha, unificando loops com 
etain_mut.
- **Onda 43 - Carregador de Personagens Zero-Alloc e Unquote In-Place**:
  - `ini::unquote_str`: Empréstimo in-place de fatias sem alocar novas instâncias de `String` para strings limpas ou entre aspas.
  - `loader::load_mugen_char`: Conversão de `fget` para retornar `Option<&str>` ao invés de `Option<String>`, eliminando cerca de 20 alocações na heap por carregamento de lutador.
  - `loader::local_size`: Análise em pilha via `split_once(',')` substituindo a alocação dinâmica de `nums`.
  - `cns::parse_cmd`: `unquote_str` in-place para nome e comandos.
