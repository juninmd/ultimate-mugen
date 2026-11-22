# Container de Sprites (LZ4)

Todo bloco de pixels que o engine guarda para usar depois é comprimido com LZ4, com uma regra só para os dois tipos de lutador. Os decoders são Rust puro (`lz4_flex`, modo seguro, compila para `wasm32`), o formato não perde nada e o PNG continua como formato de criação.

## Formatos

| Magic | Contém | Gerado por | Lido por |
|---|---|---|---|
| `UML1` | um sprite: indexado (8 bits + paleta) ou RGBA | `ume-tools pack-sprites` (packs web dos lutadores) | `ume_formats::lz4_sprite`, carregador de sprites de `ume_native` |
| `UMSF` | um `.sff` inteiro: paletas, tabela de sprites, um bloco `UML1` por buffer de pixels distinto | `ume-tools cook-sff` | `ume_formats::sff::decode_sff` e `decode_sff_only` |

Os leitores reconhecem o magic, então PNG, `.sff` clássico e arquivo cozido são intercambiáveis sob o mesmo nome. Um arquivo cozido pode substituir o `.sff` original na pasta ou no zip do personagem.

## Layout `UML1` (little endian)

```text
"UML1"  tipo u8 (0 RGBA, 1 indexado)  cores u16  largura u16  altura u16
recorte x u16  y u16  largura u16  altura u16
paleta: cores * 4 bytes (RGBA), só no indexado
bloco LZ4 dos pixels recortados (ausente quando o recorte é vazio)
```

O recorte remove só a borda exatamente zero (índice 0, ou RGBA `0,0,0,0`), então a decodificação devolve todos os bytes. Cor escondida sob alfa zero no resto da imagem é mantida.

## Layout `UMSF`

Cabeçalho `"UMSF"`, versão u8 (1), versão do sff u8, 2 bytes zero; depois paletas únicas, tabela de paletas (grupo, índice, referência), blocos (`UML1` sem paleta) e a tabela de sprites (grupo, índice, eixos, largura, altura, `palette_index`, referência de paleta e de bloco; `u32::MAX` = nenhum). Sprites que compartilhavam um buffer no `.sff` decodificado (links) compartilham um bloco e voltam como um `Arc` só. Os blocos decodificam em paralelo por `ume_formats::par::map` (uma thread no `wasm32`) e `decode_sff_only` pula os blocos dos sprites não pedidos.

## Limites

Um sprite tem no máximo 8192 px por lado e 16 Mpx, e um arquivo `UMSF` pode declarar no máximo 512 Mpx de sprites pedidos. Contagens e tamanhos são checados contra o tamanho do arquivo; arquivo truncado ou corrompido devolve erro e nunca dá panic.

## Comandos

```bash
cargo run --release -p ume-devtools --bin ume-tools -- cook-sff in.sff out.sff
cargo run --release -p ume-devtools --bin ume-tools -- pack-sprites web/dist
```

`cook-sff` decodifica o original, cozinha, decodifica o resultado e falha se alguma paleta ou sprite (pixels, eixos, paleta, links) for diferente.

## Medições

62 personagens M.U.G.E.N. reais do `mugen-assets` (54.382 sprites), uma thread, blocos por sprite: LZ4 decodifica ~1,5 GB/s de pixels indexados, 28 µs por sprite contra 61 µs do decode RLE8/LZ5 original. Totais, `.sff` original contra `UMSF`: 853,6 MB contra 624,1 MB, decode de 2,30 s contra 1,42 s com o número padrão de workers. A comparação com zstd, PNG8, QOI e deflate está no [ADR 0009](../adr/0009-lz4-sprite-container.md).
