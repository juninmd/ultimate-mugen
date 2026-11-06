# Adicionar conteúdo M.U.G.E.N

## Personagens

1. Coloque a pasta em `mugen/chars/<nome>/` com o `.def` dentro, ou jogue o `.zip` direto em `mugen/`.
2. Abra o jogo: o personagem aparece na seleção. `Tab` filtra entre todos, originais e M.U.G.E.N.

Na versão web, use o botão **Importar MUGEN (.zip)** ou arraste o `.zip` para a janela. A importação é idempotente: o mesmo arquivo, ou o mesmo lutador ou cenário encontrado de novo em outro caminho, nome ou caixa de letras, é registrado uma única vez (a identidade é um hash do `.def` normalizado mais o arquivo de sprites). Pacotes salvos no navegador por versões antigas são deduplicados automaticamente no próximo carregamento.

Uma importação web mostra uma tela de carregamento em tela cheia desde o momento em que os arquivos são escolhidos até o último pacote ser registrado. Os arquivos são lidos, descompactados e registrados **um por frame**, com no máximo dois pacotes brutos aguardando na memória do wasm; um lote de pacotes grandes não roda mais em um único frame congelado. A mesma tela aparece na inicialização enquanto os pacotes salvos no IndexedDB são restaurados, um de cada vez.

Centenas de personagens são registradas sem decodificar tudo; só o que está sob o cursor é carregado, e um cache LRU limita a memória.

## Cenários

Coloque `<nome>.def` e o `.sff` em `mugen/stages/`, ou dentro de um `.zip` em `mugen/`.

## O que é suportado

| Item | Situação |
|---|---|
| `.sff` v1 e v2 (PCX, RLE8, RLE5, LZ5, PNG 8/24/32) | Suportado |
| `.air`, `.cmd`, `.cns`, `.st`, `.snd` (WAV PCM 8/16 bits), `.act` | Suportado |
| `localcoord` (personagens HD 720p) | Suportado |
| ~80 state controllers e mais de 100 triggers | Ver [Scripts CNS](Scripts-CNS.md) |
| `ReversalDef`, `HitOverride`, `RemapPal`, `SndPan`, `VictoryQuote` | Planejado |
| `[BGCtrl]` em cenários, screenpacks, lifebars `fight.def`, fontes `.fnt` | Planejado |
| Extensões do Ikemen-GO (ZSS, Map, Dizzy, Tag...) | Planejado |

## Validar um personagem

```bash
cargo run --release -p ume-engine --example mugen_check -- mugen/chars/kfm
cargo run --release -p ume-engine --example sprite_audit -- mugen/chars/kfm
cargo run --release -p ume-engine --example hover_check -- mugen/chars/kfm
```

| Ferramenta | Mostra |
|---|---|
| `mugen_check` | Carrega e simula uma luta sem janela; lista erros de parse, estados inválidos e controladores, triggers e parâmetros não suportados (`--strict` falha neles) |
| `sprite_audit` | Sprites ausentes ou em branco referenciados pelo `.air` |
| `hover_check` | Lutador preso fora do chão (`TRACE=1` mostra as transições) |

Personagens com licença que proíbe redistribuição não devem ser commitados: `mugen/` é ignorada pelo git.
