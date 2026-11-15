# Seleção de personagens

Resumo em português de [Character-Select](../Character-Select.md).

- Pesquisa: grade central compacta, cor por jogador (azul P1, vermelho P2), retrato grande do lutador sob o cursor, confirmação com feedback claro, slots de equipe, cenário na mesma tela, tamanho comparável entre lutadores.
- Design: painéis de vidro iguais ao menu e à HUD, retrato grande com brilho na cor do lutador, cursor suave com cantos e etiqueta P1/P2/CPU, até 5 slots de equipe, cenário no cabeçalho (`[` e `]`).
- Paletas: linha de 12 amostras numeradas sob cada painel; `,` / `.` trocam, `P` sorteia e o clique escolhe direto. Cada lado e cada membro de equipe guardam a própria paleta; a última paleta de cada jogador fica salva em `ultimate-mugen.cfg` (`pal1`, `pal2`) e nos replays.
- Navegação pela grade por coluna: `Cima` e `Baixo` mantêm a coluna e caem no quadro mais próximo da linha de destino, então uma última linha mais curta nunca trava nem pula o cursor. `PgUp`/`PgDn` andam uma página, `Home`/`End` vão aos extremos, a roda rola por linha e `R` sorteia um quadro.
- Tamanho uniforme: `draw_char_fit` encaixa a caixa envolvente da animação parada num quadro fixo, calculada uma vez por lutador. Sprites e cenários nunca são alterados.

## Força, vida e ordenação

Cada cartão tem uma barra dourada fina na borda inferior, que mostra a força. O painel lateral mostra três medidores acima da placa de nome: FOR (força), VID (vida) e VEL (velocidade de caminhada). Força = ataque × vida / 1000; vida e velocidade vêm dos dados do personagem, nativo ou M.U.G.E.N. A tecla `O` alterna a ordem: elenco, força, vida, velocidade e nome. Detalhes em [Character-Select](../Character-Select.md).

## Elencos grandes (1000+ lutadores e cenários)

- A lista visível (filtro, busca e ordenação) é calculada uma vez e reaproveitada até mudar o filtro, a ordenação, o texto da busca ou o conjunto jogável (`src/app/select_view.rs`). Com 1017 lutadores, ordenar por nome custa cerca de 0,9 ms ao recalcular e 0,08 ms ao reaproveitar.
- A grade usa só as linhas que cabem acima do rodapé, ou acima dos espaços do time nos modos de equipe, então os espaços nunca cobrem a última linha (`src/app/select_grid.rs`).
- O selo do cursor (P1, P2, CPU) fica dentro do tile e não cobre mais o nome do tile de cima. Os nomes são cortados na largura do tile com reticências (`App::fit_label`).
- Uma pílula de busca abaixo do título mostra a busca atual e o número de resultados; `/` ou um clique abre a página de busca, `Esc` ou um clique limpa. A busca procura nome, id, autor e série. Na página de busca, as setas e `PgUp`/`PgDn` escolhem um resultado e `Enter` leva o cursor até ele.
- O rodapé mostra a página (`pág 3/15`) ao lado da posição.

## Seleção de cenário

- Chips de filtro (Todos, Originais, M.U.G.E.N), alternados com `Tab`, e caixa de busca aberta com `/` ou um clique; `Esc` limpa a busca.
- Nomes cortados na largura do tile com reticências; `Home`/`End` vão para as pontas; o rodapé mostra posição e página.
- `Aleatório` sorteia entre os cenários que passam no filtro e na busca atuais.
- As prévias de cenários M.U.G.E.N só começam depois que o cursor para por 0,12 s, uma por quadro, e prévias a mais de seis linhas de distância são liberadas, então rolar por centenas de cenários M.U.G.E.N não mantém todos decodificados na memória.

## Modo de estresse

`UME_STRESS=<n>` (desktop) ou `?stress=<n>` (web) adiciona `n` lutadores e `n` cenários gerados (até 5000), reaproveitando o conteúdo embutido com nomes, autores e séries gerados. É só um gancho de teste. `UME_STRESS=1000 UME_SHOTS_ONLY=roster ultimate-mugen --shots <dir> --res 1280x720 --mute` imprime linhas `roster-perf:` (custo da lista por ordenação, tempo de CPU por quadro andando na grade e na lista de cenários, memória) e salva screenshots da grade, da busca e dos filtros de cenário.
