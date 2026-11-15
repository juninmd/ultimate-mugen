# Criar um cenário (guia rápido)

[English](../Make-a-Stage.md) · **Português (BR)**

Há três tipos de trabalho com cenários. Escolha o que você precisa:

| Você quer | Precisa de código? | Vá para |
|---|---|---|
| Um fundo novo com a sua arte, no formato M.U.G.E.N | Não | [Cenário M.U.G.E.N](#1-um-cenário-mugen) |
| Mais conteúdo em um dos cinco cenários do modo história (objetos, música, céu animado) | Não | [Cenários do modo história](#2-cenários-do-modo-história) |
| Um cenário novo no modo história ou um cenário embutido novo | Sim, Rust | [O que precisa de código](#3-o-que-precisa-de-código) |

## 1. Um cenário M.U.G.E.N

1. Coloque dois arquivos em `mugen/stages/`: `meu-cenario.def` e `meu-cenario.sff` (a arte).
2. Parta de [`docs/examples/stage/my-stage.def`](../../examples/stage/my-stage.def).

O `.def` é texto simples com seções:

| Seção | O que definir |
|---|---|
| `[Info]` | `name`, `displayname`, `author` |
| `[Camera]` | `startx`, `boundleft`, `boundright` (até onde a câmera vai), `boundhigh`, `boundlow`, `verticalfollow`, `tension` |
| `[PlayerInfo]` | `p1startx`, `p2startx` (onde cada lutador começa) |
| `[StageInfo]` | `zoffset` (altura da linha do chão na tela), `localcoord` (tamanho de desenho, por exemplo `320, 240`) |
| `[BGdef]` | `spr`: o arquivo `.sff` |
| `[BG nome]` | Uma seção por camada (veja abaixo) |

Cada camada de fundo:

```ini
[BG Sky]
type = normal          ; normal (um sprite) ou anim (uma ação do arquivo .air)
spriteno = 0, 0        ; grupo, imagem do .sff
layerno = 0            ; 0 = atrás dos lutadores, 1 = na frente
start = 0, 0           ; posição em unidades locais
delta = .2, .2         ; paralaxe: 1, 1 anda junto com o chão; menor anda mais devagar
tile = 1, 0            ; repete em x, em y (0 = sem repetir)
```

Todas as chaves do formato, incluindo escala, rotação e janelas de recorte, estão na [referência de cenários](../Stages.md).

O texto depois de `;` é comentário. Os nomes das chaves são os do M.U.G.E.N, então um cenário copiado de um pacote M.U.G.E.N é um bom ponto de partida.

Limites deste caminho: o céu animado em loop, as faixas do modo história, os quebráveis e a música por cenário das próximas seções não se aplicam a cenários M.U.G.E.N.

## 2. Cenários do modo história

O modo história tem cinco cenários: `brazil`, `neon`, `bamboo`, `volcano` e `dojo`. Cada um pode ser estendido com dados:

| O que | Onde | Observações |
|---|---|---|
| Faixa de fundo | `assets/stages/story/<cenário>.png` | Um segmento por luta (quatro ondas e o chefe). Regere com `python3 tools/story_segments.py` |
| Céu animado | `assets/stages/animation/<cenário>/00.png` a `07.png` | PNGs transparentes de 1672×941, em loop a cada oito ticks. Regere com `python tools/gen_stage_loops.py` |
| Quebráveis, comida, fase bônus | `assets/story/objects.toml`, tabela `[stages.<cenário>]` | Formato completo em [Objetos de cenário](Objetos-de-Cenario.md) |
| Música da luta | `tweaks.toml`, seção `[music]`, chave `<id do cenário>` | Caminho para `.ogg`, `.mp3`, `.wav` ou `.flac`, relativo ao `tweaks.toml`. Só no desktop. Veja [Configuração](Configuracao.md) |

Exemplo de `tweaks.toml`:

```toml
[music]
fight = "music/minha-luta.ogg"   ; toda luta sem trilha própria
dojo = "music/dojo.ogg"          ; vence o fight, só no dojo
```

Exemplo de entrada em `objects.toml`:

```toml
[stages.dojo]
objects = [
  { fight = 0, kind = "crate", x = -78.0, lane = -22.0, drop = "apple" },
]
```

Confira com o tour de evidências: `UME_TOUR=storyobjects` e `UME_TOUR=storyloop` (veja [Objetos de cenário](Objetos-de-Cenario.md)).

## 3. O que precisa de código

Estas coisas não podem ser feitas só com arquivos hoje. Cada uma é uma lacuna do formato, não erro nos seus arquivos:

- **Um cenário embutido novo.** Cenários nativos são funções Rust em `src/stages.rs` (`builtin()`). Um novo precisa de código, de um PNG embutido e de uma miniatura. Veja [Criar cenários](../Stages.md).
- **Um cenário novo no modo história.** A lista de cenários é fixa em `src/stages/story_strip.rs` (`STORY_ARENAS`), e os dados do modo história em `src/story/stages.rs`.
- **Céu animado para um cenário M.U.G.E.N.** Os quadros do loop são lidos só para os ids de cenários embutidos.
- **Seção `[Music]` em um `.def`.** A música por cenário vem só do `tweaks.toml`.
- **Camada `type = parallax`.** Planejada, ainda não existe no formato.

## Teste

```bash
cargo test --release -p ume-engine --test wiki_examples
UME_SHOTS_ONLY=stages cargo run --release -- --shots out/   # capturas de todos os cenários
```

O primeiro comando confere as seções do `.def` de exemplo. O segundo salva capturas dos cenários em `out/`.

## Próximos passos

[Referência de cenários](../Stages.md) · [Objetos de cenário](Objetos-de-Cenario.md) · [Modo história](../Story-Mode.md) · [Criar um personagem](Criar-um-Personagem.md) · [Início da wiki](Home.md)
