# Seleção de personagens

Resumo em português de [Character-Select](../Character-Select.md).

- Pesquisa: grade central compacta, cor por jogador (azul P1, vermelho P2), retrato grande do lutador sob o cursor, confirmação com feedback claro, slots de equipe, cenário na mesma tela, tamanho comparável entre lutadores.
- Design: painéis de vidro iguais ao menu e à HUD, retrato grande com brilho na cor do lutador, cursor suave com cantos e etiqueta P1/P2/CPU, até 5 slots de equipe, cenário no cabeçalho (`[` e `]`).
- Paletas: linha de 12 amostras numeradas sob cada painel; `,` / `.` trocam, `P` sorteia e o clique escolhe direto. Cada lado e cada membro de equipe guardam a própria paleta; a última paleta de cada jogador fica salva em `ultimate-mugen.cfg` (`pal1`, `pal2`) e nos replays.
- Navegação pela grade por coluna: `Cima` e `Baixo` mantêm a coluna e caem no quadro mais próximo da linha de destino, então uma última linha mais curta nunca trava nem pula o cursor. `PgUp`/`PgDn` andam uma página, `Home`/`End` vão aos extremos, a roda rola por linha e `R` sorteia um quadro.
- Tamanho uniforme: `draw_char_fit` encaixa a caixa envolvente da animação parada num quadro fixo, calculada uma vez por lutador. Sprites e cenários nunca são alterados.

## Força, vida e ordenação

Cada cartão tem uma barra dourada fina na borda inferior, que mostra a força. O painel lateral mostra três medidores acima da placa de nome: FOR (força), VID (vida) e VEL (velocidade de caminhada). Força = ataque × vida / 1000; vida e velocidade vêm dos dados do personagem, nativo ou M.U.G.E.N. A tecla `O` alterna a ordem: elenco, força, vida, velocidade e nome. Detalhes em [Character-Select](../Character-Select.md).
