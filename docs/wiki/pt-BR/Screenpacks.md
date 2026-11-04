# Screenpacks

[English](../Screenpacks.md) · **Português (BR)**

Um screenpack é o conjunto de arquivos que desenha tudo em volta da luta: menu de título, seleção de personagens,
barras de vida, fontes e storyboards de abertura/logo.

**Estado:** o jogo agora carrega um screenpack quando ele existe (`mugen/data/system.def`,
`mugen/screenpacks/<pacote>/system.def` ou `UME_SCREENPACK`). A interface nativa continua como fallback para o que o
pacote não fornece:

- Rótulos `menu.itemname.*` do `system.def` substituem os títulos do menu principal (Arcade, Sobrevivência, equipes, Treino, CPU vs CPU, Sair).
- A ordem de `[Characters]` do `select.def` define a ordem da grade de seleção.
- As cenas de texto do `intro.storyboard` são exibidas sobre a abertura do Arcade, com o tempo de `end.time` de cada cena.
- `pos`/`range.x` da barra de vida do `fight.def` posicionam e dimensionam as barras (convertidos do `localcoord` do pacote para o espaço 426×240 do HUD); o resto do HUD continua nativo.
- Sprites, fontes e sons analisados ficam em `ume_formats::screenpack::Screenpack` para as telas restantes, mas o desenho de sprites de system/fight (fundos, barras, fontes) ainda é nativo.

O verificador abaixo diz se o pacote analisa; o jogo nunca deixa de abrir por causa de um pacote quebrado (usa a
interface nativa e registra os avisos).

## Layout

```
data/
  system.def      menu de título, seleção, música, storyboards
  select.def      grade do elenco e lista de cenários
  fight.def       barras de vida/energia, anúncios de round/KO
  system.sff      sprites usados pelo system.def
  system.snd      sons de menu
  fight.sff/.snd  sprites das barras e vozes do narrador
font/
  *.fnt           fontes bitmap (v1 binária ou v2 texto)
```

## Verificando um pacote

```
cargo run -p ume-formats --example screenpack_check -- <raiz do jogo> data/system.def
```

Ele imprime o que foi carregado e uma linha `warning:` por arquivo ausente ou quebrado.
