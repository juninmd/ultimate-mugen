Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Decodificação SND pré-alocada e filtragem de comandos sem alocação

- Tabelas hash pré-alocadas para sons SND: `snd::decode_snd_checked` limita a capacidade em `count.min(buf.len() / 16 + 1)` e pré-aloca as tabelas `out` e `seen`, eliminando redimensionamentos repetidos e rehashes durante a decodificação de áudio de personagens.
- Resolução estática de chaves de paleta no carregador: `loader::load_character` resolve substituições de paletas (`pal1`..`pal12`) usando uma tabela estática de fatias `PAL_KEYS` em vez de formatar 12 strings no heap por personagem.
- Deduplicação de sprites pré-alocada: Pré-dimensiona os vetores `seen` e `unique` para o tamanho de `sff.sprites.len()`.
- Extração de lista de golpes e comandos de IA com zero alocações: `loader::load_character` filtra comandos no local usando `eq_ignore_ascii_case` antes de clonar ou alocar strings, removendo dezenas de alocações no heap por personagem carregado.
