Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Descompressão RLE em bloco e eliminação de escrita redundante de zeros no SFF v2

- Eliminação de escrita de zeros em `decode_rle8`: Como `vec![0u8; size]` já aloca memória inicializada em zero, repetições de pixels transparentes (`val == 0`) e literais zero (`d == 0`) pulam escritas na memória e apenas avançam o índice `j`. Isso elimina esperas nos buffers de escrita e sujeira nas linhas de cache (geralmente 50-80% de transparência).
- Preenchimento em bloco no RLE5: Em `decode_rle5`, loops byte a byte foram substituídos por `.fill(c)` em fatias (ou avanço direto quando `c == 0`), transformando iterações individuais em memsets vetorizados.
- Cópia direta de fatias para paletas SFF v2: Em `decode_sff_v2`, paletas com limites válidos utilizam cópia em lote ou direta via `copy_from_slice`, eliminando checagens de limites e adições saturadas por cor.
