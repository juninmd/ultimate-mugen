# Potencial Liberado

Personagens sem transformação própria (importados do M.U.G.E.N. e UME) ganham uma transformação genérica que não exige editar os arquivos do personagem. Aperte o botão de transformação (`T`, o mesmo dos despertares nativos) com o lutador no chão, com controle e com a luta em andamento.

| Etapa | Nome (pt-BR / en) | Custo | Duração | Efeito |
|-------|-------------------|-------|---------|--------|
| 1ª | Potencial Liberado / Unleashed Potential | meia barra de poder | 12 s | ataque x1,2, dano recebido x0,9 |
| 2ª | Modo Fúria / Fury Mode | barra de poder cheia | 9 s | ataque x1,4, dano recebido x0,75 |

Cada etapa vale uma vez por round. O Modo Fúria só vem depois do Potencial Liberado. Personagens com forma própria (`forms` ou a animação 196 legada) mantêm a transformação deles e não ganham esta.

Visual: a aura e a cor do lutador mudam (azul no Potencial, vermelho na Fúria), um anel e uma etiqueta curta tocam na ativação, e o HUD mostra uma tira de tempo abaixo do medidor de poder. Nativos com formas também mostram uma tira de forma no HUD.

Código: `crates/engine/src/potential.rs` (estado e multiplicadores), `crates/engine/src/matchs/unleash.rs` (ativação), `src/vfx/awaken.rs` (efeitos), `src/hud/passive_meter.rs` (tiras). Teste: `crates/engine/tests/potential.rs`.
