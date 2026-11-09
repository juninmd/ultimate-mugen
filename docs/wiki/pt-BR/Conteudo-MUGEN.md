# Adicionar conteúdo M.U.G.E.N

## Personagens

1. Coloque a pasta em `mugen/chars/<nome>/` com o `.def` dentro, ou jogue o `.zip` direto em `mugen/`.
2. Abra o jogo: o personagem aparece na seleção. `Tab` filtra entre todos, originais e M.U.G.E.N.

Use Menu principal › **Importar M.U.G.E.N** (o único botão de importação; na web abre o seletor do navegador, no desktop o diálogo de arquivos do sistema) ou arraste o `.zip` para a janela da web. A importação é idempotente: o mesmo arquivo, ou o mesmo lutador ou cenário encontrado de novo em outro caminho, nome ou caixa de letras, é registrado uma única vez (a identidade é um hash do `.def` normalizado mais o arquivo de sprites). Pacotes salvos no navegador por versões antigas são deduplicados automaticamente no próximo carregamento.

Uma importação web mostra uma tela de carregamento em tela cheia desde o momento em que os arquivos são escolhidos até o último pacote ser registrado. Os arquivos são lidos, descompactados e registrados **um por frame**, com no máximo dois pacotes brutos aguardando na memória do wasm; um lote de pacotes grandes não roda mais em um único frame congelado. A mesma tela aparece na inicialização enquanto os pacotes salvos no IndexedDB são restaurados, um de cada vez.

Centenas de personagens são registradas sem decodificar tudo; só o que está sob o cursor é carregado, e um cache LRU limita a memória.

Antes de um personagem entrar na lista, o catálogo confere se cada arquivo que o `.def` cita em `[Files]` existe dentro da própria pasta do personagem (ou na raiz do pacote, ou em `data/`): `sprite`, `anim` e `cns` são obrigatórios. Quem falha na conferência vai para a **quarentena**: fica fora da seleção e o motivo é listado. Arquivos nunca são emprestados da pasta de outro personagem, então um `.def` copiado que aponta para os sprites de outro lutador não carrega esses sprites pelo nome. A conferência é estrutural (presença dos arquivos), rápida o bastante para a varredura de inicialização; um lutador cujos arquivos existem, mas cujas animações referenciam sprites faltando, continua carregando e aparece com os frames faltantes.

Abra **Personagens MUGEN** no menu principal para ver o resumo: quantos personagens carregaram, quantos estão em quarentena e os motivos mais comuns, além de uma galeria de todos os personagens carregados (retrato, nome e autor; `←/→` trocam de página) e a lista da quarentena (`TAB` alterna a visão; `ESC` volta).

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
cargo run --release -p ume-formats --example mugen_refs -- mugen/chars
cargo run --release -p ume-formats --example mugen_lint -- mugen/chars
cargo run --release -p ume-engine --example mugen_pairs -- --matrix mugen/chars
cargo run --release -p ume-formats --example mkselectdef -- mugen > mugen/select.def
```

| Ferramenta | Mostra |
|---|---|
| `mugen_check` | Carrega e simula uma luta sem janela; lista erros de parse, estados inválidos e controladores, triggers e parâmetros não suportados (`--strict` falha neles) |
| `sprite_audit` | Sprites ausentes ou em branco referenciados pelo `.air` |
| `hover_check` | Lutador preso fora do chão (`TRACE=1` mostra as transições) |

Personagens com licença que proíbe redistribuição não devem ser commitados: `mugen/` é ignorada pelo git.
