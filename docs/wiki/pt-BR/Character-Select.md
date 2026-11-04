# Seleção de personagens

Resumo em português de [Character-Select](../Character-Select.md).

- Pesquisa: grade central compacta, cor por jogador (azul P1, vermelho P2), retrato grande do lutador sob o cursor, confirmação com feedback claro, slots de equipe, cenário na mesma tela, tamanho comparável entre lutadores.
- Design: painéis de vidro iguais ao menu e à HUD, retrato grande com brilho na cor do lutador, cursor suave com cantos e etiqueta P1/P2/CPU, até 5 slots de equipe, cenário no cabeçalho (`[` e `]`).
- Tamanho uniforme: `draw_char_fit` encaixa a caixa envolvente da animação parada num quadro fixo, calculada uma vez por lutador. Sprites e cenários nunca são alterados.
