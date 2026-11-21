# Homologação

O `ume-tools roster-audit` carrega todos os personagens e cenários de uma coleção M.U.G.E.N instalada, joga lutas reais
de CPU contra CPU entre eles e grava um relatório legível por máquina de tudo que está quebrado: carregamento,
visual (sprites ausentes ou invisíveis), som (chamadas sem amostra), mecânica (estados travados, lutador abaixo do
chão, ataques sem fim, resgates do watchdog), as checagens específicas da importação no navegador e a memória que
cada lutador precisa.

É o substituto reproduzível do "abrir o jogo e testar alguns personagens na mão".

## Como rodar

```bash
cargo build --release -p ume-devtools
target/release/ume-tools roster-audit                     # mugen/chars + mugen/stages -> docs/evidence/homologation
target/release/ume-tools roster-audit --only "goku"        # uma família de personagens (os cenários continuam todos)
target/release/ume-tools roster-audit --limit 20 --no-fights   # passada estática rápida
target/release/ume-tools roster-audit --opponent "vegeta"  # força todas as lutas contra um oponente
target/release/ume-tools roster-audit --fail-over critical # sai com 2 em qualquer achado crítico (gate de CI)
```

| Flag | Significado |
|---|---|
| `--chars <dir>` / `--stages <dir>` | raízes varridas (padrão `mugen/chars`, `mugen/stages`) |
| `--out <dir>` | onde `report.json`, `report.md` e `issues.csv` são escritos |
| `--limit N` / `--stage-limit N` | audita só os N primeiros personagens / cenários |
| `--fights N` | lutas por personagem (cada uma contra outro oponente e cenário, padrão 2) |
| `--stage-fights N` | passo de rotação de oponentes quando o cenário é o alvo |
| `--only`, `--only-stage`, `--opponent` | filtros por substring para personagens, cenários e oponente forçado |
| `--no-fights`, `--quiet` | só a passada estática, e sem progresso por item no stderr |
| `--fail-over <severidade>` | código de saída 2 quando existe achado naquela severidade ou pior |

A execução é paralela (um carregamento por personagem por worker, nunca a coleção inteira na memória) e cada luta
roda dentro de `catch_unwind`, então um pacote quebrado não aborta a auditoria.

## O que cada luta verifica

Cada luta roda 30 segundos de CPU contra CPU em um cenário real, com o fundo do cenário ativo, e só julga controle e
animação durante o round (`roundstate = 2`), para não confundir intro ou vitória com travamento:

| Código | Severidade | Significado |
|---|---|---|
| `fight-panic` | crítica | a partida entrou em pânico durante o tick (a mensagem fica no relatório) |
| `fight-non-finite` | crítica | posição, velocidade, vida ou poder virou NaN/inf |
| `fight-life-range`, `fight-power-range` | maior | vida ou poder saiu dos próprios limites |
| `fight-below-floor-p1/p2` | maior | o lutador ficou mais de 12 px abaixo do chão por mais de 20 ticks |
| `fight-stuck-p1/p2` | maior | 420 ticks sem controle fora dos estados deitados |
| `fight-endless-attack-p1/p2` | maior | 240 ticks preso dentro de um mesmo estado de ataque |
| `fight-no-contact` | maior | o par não acertou nenhum golpe em 30 segundos |
| `fight-off-camera-p1/p2` | maior | o lutador ficou fora da câmera por mais de 3 ticks |
| `fight-watchdog-rescue` | menor | a rede de segurança do motor interveio (o script não devolve o controle) |
| `fight-camera-nan` | crítica | posição ou zoom da câmera virou não-finito |
| `fight-engine-warning` | menor | aviso que o motor levantou durante a luta (anim ou estado ausente) |

## O que a passada estática verifica

| Código | Severidade | Significado |
|---|---|---|
| `char-load-fail`, `char-load-panic` | crítica | o pacote não carrega de jeito nenhum |
| `char-no-anims`, `char-no-states`, `char-no-commands` | crítica | `.air`/`.cns`/`.cmd` não produziu nada |
| `char-web-quarantine` | crítica | a importação do navegador recusa o pacote (`ume_formats::health`), então ele nunca aparece na web |
| `char-unsupported-controller`, `char-unknown-trigger` | maior | controller ou trigger que o motor não executa |
| `char-sprite-missing` | maior | um quadro de animação aponta para um sprite inexistente no `.sff` |
| `char-sprite-transparent` | menor | o quadro existe mas todos os pixels são transparentes |
| `char-sound-missing` | maior | `PlaySnd` pede uma amostra que o `.snd` não tem |
| `char-unknown-parameter` | menor | parâmetro de controller que o motor nunca lê |
| `char-wasm-atlas-overflow`, `char-wasm-atlas-pressure` | maior/menor | os pixels de sprite não cabem no atlas 2048 do navegador (4,2 M pixels), então ele precisa expulsar e reenviar |
| `char-load-slow` | menor | carregamento acima de 2,5 s |

Códigos de cenário: `stage-load-fail`, `stage-load-panic`, `stage-no-layers`, `stage-no-sprites`,
`stage-layer-sprite-missing`, `stage-layer-anim-missing`, `stage-bounds-inverted`, `stage-bounds-tight`,
`stage-spawn-overlap`, `stage-music-outside-vfs`, `stage-no-shadow`.

## Lendo o relatório

- `report.md` ranqueia os personagens e cenários com os piores achados, e lista os pacotes mais lentos para carregar e
  os ticks de simulação mais caros.
- `report.json` traz o resumo, todos os achados e todas as lutas (golpes, dano, estados distintos, custo por tick).
- `summary.json` é a forma compacta (contagem por código e os primeiros exemplos), pequena o bastante para versionar.
- `issues.csv` é a mesma lista achatada para planilha ou diff entre duas execuções.

Um achado é evidência para reproduzir, não veredito: `--only <nome> --opponent <nome>` replica o par exato e
`--no-fights` isola os problemas estáticos de um pacote.

## Validação na web

O navegador é o alvo mais crítico: ele recusa pacotes que o carregador do desktop tolera e tem um orçamento de
textura muito menor. Os achados de atlas saem da soma dos pixels de todos os sprites do pacote (indexados e RGBA)
comparada com o atlas 2048 usado pelo build wasm (4,2 M pixels). Para reproduzir uma importação de verdade:

```bash
node scripts/web-mugen-smoke.js web/dist mugen/incoming/meu-char.zip
```

Ele importa o pacote pelo input de arquivo real, entra direto numa luta com o personagem importado e falha com
pânico, relógio congelado, renderizador travado ou heap acima do limite. Com um pacote que já está no elenco (o
`elecbyte.zip` embutido), defina `MUGEN_SMOKE_CHAR` para o personagem e o teste valida a deduplicação por chave de
conteúdo em vez de exigir uma entrada nova.
