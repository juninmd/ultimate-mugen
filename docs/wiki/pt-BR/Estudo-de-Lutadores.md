# Estudo de lutadores: 50+ lutadores e o que ensinam

Resumo em português de [Fighter Study](../Fighter-Study.md): mais de 50 lutadores de jogos comerciais e do M.U.G.E.N foram estudados (SRK, Dustloop, StrategyWiki, MKKomplete). A tabela completa está no arquivo em inglês.

## Padrões transformados em recursos

1. Golpes de avanço e investidas com armadura (`lunge`, `shoulder`), que ainda cancelam em especiais.
2. Mergulhos aéreos com velocidade angular e quique no chão (`falcondive`, `meteordrop`).
3. Especiais que cancelam em outros por `cancels` explícito.
4. Anti-aéreos com invulnerabilidade e armadura de um golpe em golpes lentos e pesados.
5. Rápidos trocam dano por mobilidade (`superSpeed`, `teleport`); fortes ganham `armor`, `wallBounce` e golpes `heavy`.

## Como são feitos do zero

1. Projetar o kit. 2. Arte em pixel art com paleta única, prerender 3D ortográfico ou rips. 3. Empacotar em SFF, AIR, CMD, CNS. 4. Frame data: leves 3-5 ticks de startup, pesados 8-14. 5. Balancear no modo treino.

Veja as skills `mugen-character-creator` e `mugen-to-ume-migrator`.
