Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Saída antecipada de caixas de colisão, fatiamento de alvos sem alocação e despacho de IA indexado

- Fast-path em caixas de colisão: Em `fighter::boxes()`, quadros sem caixas e sem modificadores ativos retornam imediatamente `vec![]` sem consultar tabelas de escala ou calcular fatores. Fatores de escala pré-multiplicados e inversão horizontal com XOR único (`flip = fr.flip_h ^ (self.facing < 0.0)`) removem penalidades de ramificação em cada vértice.
- Fatiamento de alvo sem alocação em `p2clsnrequire`: Em `matchs::step_hits`, alvos específicos emprestam a fatia `&d_boxes[req..req + 1]` diretamente em vez de alocar um `Vec` no heap.
- Despacho de comandos de IA indexado: Em `ai::Ai`, comandos são indexados como `Vec<usize>` em `fighter.def.commands` em vez de clonar árvores `CommandDef`. Ataques próximos e à distância são separados na inicialização, tornando a seleção em `O(1)` sem varreduras lineares.
- Conversão vetorizada de pixmap para straight alpha: `raster::pixmap_to_image` evita divisões inteiras em pixels opacos (`alpha == 255`) e transparentes (`alpha == 0`) utilizando `as_chunks_mut::<4>()`.
