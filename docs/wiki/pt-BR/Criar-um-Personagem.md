# Criar um personagem (guia rápido)

[English](../Make-a-Character.md) · **Português (BR)**

Este é o caminho curto. Você faz um lutador funcionando em cerca de dez linhas e depois o expande. Todos os campos estão na [referência completa do UMF](Personagem-UMF.md).

## 1. Comece de um lutador que já funciona

Um arquivo UMF pode **herdar** um lutador inteiro com `extends`. O seu lutador recebe os atributos, a arte, as animações e os golpes do pai, e você escreve só o que muda.

Copie este arquivo para `umf/hello.json` (ou use o pronto em [`docs/examples/umf/hello-fighter.json`](../../examples/umf/hello-fighter.json)):

```json
{
  "format": "umf/1",
  "id": "hello",
  "name": "Hello Fighter",
  "author": "você",
  "color": "#2a9d8f",
  "extends": "kaito",
  "palettes": [{ "top": "#2a9d8f", "trim": "#16544d" }],
  "moves": [{ "id": "gale", "name": "Sky Gale", "cost": 500 }]
}
```

O que cada linha diz:

| Linha | Significado |
|---|---|
| `format` | Sempre `umf/1` |
| `id` | Único, minúsculo, sem espaços. É também o nome interno do arquivo |
| `name`, `author`, `color` | O que a tela de seleção mostra. `color` é a cor do cartão |
| `extends` | Herda tudo de `kaito` (vale qualquer lutador ou kit carregado) |
| `palettes` | Recoloração. As chaves são os nomes de cor da arte do pai (`top`, `trim`, `hair`) |
| `moves` | Altera um golpe pelo `id`. Aqui `gale` mantém os comandos e ganha outro nome e custo |

## 2. Onde colocar o arquivo

- **Ao lado do jogo:** coloque em `umf/`. Não precisa recompilar.
- **Ou** aponte uma pasta com `UME_UMF_DIR=/caminho/da/pasta`.
- **Dentro do jogo:** coloque em `assets/chars/`, adicione o `id` em `assets/chars/index.json` se quiser uma ordem fixa na seleção, e rode `cargo run --release`.

Abra o jogo e escolha **Treino** para testar. Aperte `F2` para mostrar as caixas de golpe.

## 3. Adicione um golpe novo

Os golpes são pareados pelo `id`, então você pode adicionar, alterar ou remover qualquer um:

```json
"moves": [
  { "id": "gale", "name": "Sky Gale", "cost": 500 },
  { "id": "slide", "remove": true },
  {
    "id": "crest", "name": "Crest Kick", "input": "F, F, b",
    "kind": "special", "anim": "gale",
    "hits": [{ "at": 3, "damage": 80, "level": "heavy", "launch": true }]
  }
]
```

- `input` usa a sintaxe de comandos: `D, DF, F, x` é o quarto de círculo mais `x`; `x+y` significa as duas teclas juntas.
- `kind` é `normal`, `special` ou `super`. Um normal cancela em especiais e supers ao acertar ou bloquear.
- `anim` precisa ser o nome de uma animação do pai (`gale` aqui). Animações próprias vão em `"anims"`.
- `hits[].at` é o quadro em que o golpe fica ativo. `damage` é o dano.

Mais: [golpes](Personagem-UMF.md), [hits](Personagem-UMF.md) e [sintaxe de comandos](Personagem-UMF.md).

## 4. Dê a sua cara

- **Só cores:** altere `palettes`, ou adicione `awakeningPalette` para a transformação.
- **Atributos:** `"stats": { "life": 1080, "attack": 104 }` muda um número e mantém o resto.
- **Sprites:** sua arte em pixel vai em `assets/chars/sprites/<id>/`. Veja [Importação de sprite atlas](Importacao-Sprite-Atlas.md) e [Quadros de efeito](Efeitos.md).
- **Perfil:** nacionalidade e uma biografia curta em inglês, português e espanhol. Veja [Perfil](Personagem-UMF.md).

## 5. Teste antes de jogar

```bash
cargo test --release -p ume-engine --test it wiki_examples::
cargo test --release --test it native_roster_lab:: -- --nocapture
```

O primeiro comando confere os exemplos deste guia. O segundo roda todos os ataques do elenco e mostra dano e alcance.

## Não é um lutador UMF?

- **Personagens M.U.G.E.N** (`.def`, `.sff`, `.cns`) vão em `mugen/chars/` e rodam sem mudanças. Veja [Personagens](Personagens.md) e [Scripts CNS](Scripts-CNS.md).
- **Lutador sem sprites:** não é suportado. Todo lutador é desenhado por PNGs em pixel art, como os personagens M.U.G.E.N; não existe fallback procedural.

## Próximos passos

[Referência completa do UMF](Personagem-UMF.md) · [Reuso com kits](Personagem-UMF.md) · [Criar um cenário](Criar-um-Cenario.md) · [Início da wiki](Home.md)
