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
| Fundo animado | `assets/stages/animation/<cenário>/` | Oito quadros-chave opacos de 1672×941 (`00.png` a `07.png`) e oito quadros intermediários `-mid.png`. A animação avança a cada quatro ticks e completa o ciclo em 64 ticks. Mantenha a câmera e o piso alinhados durante todo o loop. |
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

### Loops contínuos, tamanho e interatividade dos cenários embutidos

- **Loop contínuo a 60 FPS:** os cenários animados embutidos (`assets/stages/animation/<id>/00.png..07-mid.png`) são decodificados em paralelo (`ume_formats::par::map`) e fazem cross-fade suave entre quadros consecutivos em um ciclo fechado de 96 ticks (`loop_sample`), eliminando saltos no retorno do último quadro para o quadro `0`.
- **Arenas ampliadas:** os cenários embutidos agora cobrem de `bound_left = -680` a `bound_right = 680` (`bound_high = -140`, `zoom_out = 0.62`, `zoom_in = 1.05`) com paralaxe panorâmica `0.21` em `illustration_layout`.
- **Piso reativo e objetos interativos:** todos os 21 cenários ilustrados (incluindo `brazil` e as 14 Maravilhas do Mundo via `StageKind::Arena`) renderizam rachaduras, marcas de impacto, ondas e destroços do `StageReact`. Em lutas Versus/Arcade, `src/stages/interactive.rs` posiciona caixas destrutíveis e barris explosivos (`ObjectEvent::Blast`) nas alas externas (`|x| >= 220`, mantendo o centro da arena limpo); lutadores arremessados contra objetos os destroem e ambos os lados podem coletar itens (`any_collector = true`). Use `UME_STAGE_PROPS=0` para desativar os objetos de arena.

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
