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

- **Onda 44 - Despacho de Estado AssignKind Zero-Alloc e Máscaras de Bits Case-Insensitive**:
  - `matchs::AssignKind`: Substituição de alocações dinâmicas de `String` em atribuições de variáveis por um enum de 1 byte `AssignKind`, reduzindo pela metade a memória da tupla e eliminando operações na heap.
  - `hitdef::state_mask` / `attr_kind_mask` / `guardflag_bits`: Processamento de bytes ASCII insensível a maiúsculas in-place sem alocação de strings.
  - `triggers::const_val`: Busca em buffer de pilha com zero alocações para constantes `velocity.*.x`.

- **Onda 45 - Cache de Explods em Array e Varredura Triangular de Colisão de Projéteis**:
  - `Match::update_explods`: Pré-indexação de posições e estados de pausa dos lutadores em array compacto de 8 posições na pilha, eliminando varreduras lineares por explod.
  - `Match::update_projectiles`: Flags de congelamento em buffer de pilha para até 64 projéteis sem alocações na heap, e iteração triangular de pares `(a + 1)..n` reduzindo pela metade verificações de colisão entre projéteis.

- **Onda 46 - Caminho Rápido de Caixas de Colisão e Geometria Pré-Escalada**:
  - `Fighter::boxes`: Saída antecipada para listas vazias de caixas de colisão antes de calcular constantes ou fatores de escala (reduz drasticamente o trabalho em frames ociosos/movimento).
  - Dimensões X e Y pré-escaladas (`sx * s` e `sy * s`), reduzindo as multiplicações de ponto flutuante por caixa de 8 para 4.
  - Combinação de `flip_h != (facing < 0.0)` em uma única condição booleana XOR branchless.
  - `Fighter::c`: Busca de fatia guardada por `!self.size_overrides.is_empty()`.

- **Onda 47 - Junção de Candidatos VFS em Buffer de Pilha e Casamento de Sufixo Zero-Alloc**:
  - `vfs::resolve`: `with_joined` em buffer de pilha de 512 bytes para caminhos candidatos `base_dir/name` e `data/name`, eliminando alocações na heap no caminho direto de busca.
  - Substituição de strings dinâmicas `format!("/{base}")` por predicado de bytes in-place `is_match`.
  - Uso direto de `ini::unquote_str` sem alocar strings intermediárias.
  - Fallback de pasta sub-microssegundo em 50k arquivos (0.9 us/op) e redução no tempo de carga de lutadores (`kfm.def` caiu para 6.7 ms).

- **Onda 48 - Pipeline de HUD Zero-Alloc e Quads de Textura Diretos**:
  - `Hud::draw`: Otimização de `main_of` em varredura única de iterador sem alocar `Vec<&Fighter>` por lado a cada quadro.
  - Uso de array fixo em pilha `roots = [r0, r1]` substituindo vetores dinâmicos na heap.
  - Eliminação de alocações de parceiros em partidas 1v1 convencionais.
  - Substituição de vetores de malha/vértices/índices no desenho de retratos por chamadas diretas de `draw_texture_ex`.
