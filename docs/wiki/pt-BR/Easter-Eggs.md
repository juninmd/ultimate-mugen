# Easter Eggs Durante as Lutas

O Ultimate MUGEN inclui um sistema de Easter Eggs puramente de apresentação (`src/easter.rs` e `src/easter/`) que reage a momentos clássicos de jogos de luta, eventos de cenário e lore dos lutadores sem tocar na simulação determinística (`crates/engine`).

## Arquitetura
- **Seguro para determinismo**: lê `&Match` apenas na camada de apresentação (`src/app/fight_screen.rs`), mantendo rollback e replays intactos.
- **Localizado**: todos os banners, falas e cameos possuem suporte a Português, Inglês e Espanhol via `crate::i18n::tr3`.
- **Sprites animados em Pixel Art**: folhas multi-frame em `assets/easter/` renderizadas com `FilterMode::Nearest`.
- **Ritmo limpo**: controle por bitsets (por round e por partida), cooldown global e limite de 3 elementos simultâneos na tela.

## Easter Eggs de Combate e Arcade (20)
1. **QUENTINHO!** — Cameo clássico do árbitro no canto inferior direito em lançamentos pesados (`dano >= 115`).
2. **EMPATE ÉPICO!** — Nocaute duplo simultâneo.
3. **PIXEL MÁGICO!** — Vencer um round ou lançar um Super com menos de 5–6% de vida.
4. **INTOCÁVEL!** — Vitória perfeita com 100% de vida e coroa dourada.
5. **CHOQUE DE TITÃS!** — Troca simultânea de golpes em até 3 ticks.
6. **PROVOCOU, LEVOU!** — Levar golpe até 90 ticks após provocar.
7. **RESPEITO MÚTUO!** — Ambos os lutadores provocam em até 120 ticks.
8. **NO MESMO RITMO!** — Ambos pulam juntos duas vezes no mesmo round.
9. **AGACHAMENTO TÁTICO!** — Sequência rápida de 4+ agachamentos à distância.
10. **SPEEDRUN K.O.!** — Vencer um round em menos de 10 segundos.
11. **NO ESTOURAR DO RELÓGIO!** — Round decidido nos últimos 3 segundos.
12. **SEM GRAVIDADE!** — Combo aéreo de 5+ acertos.
13. **MURALHA DE FERRO!** — Defender 6+ golpes seguidos sem quebrar a guarda.
14. **DEFESA EM PEDAÇOS!** — Quebra de guarda (`state_no == 5300`).
15. **COLISÃO DE SUPERS!** — Ambos os lados disparam um Super em até 90 ticks.
16. **DESPERTAR DUPLO!** — Ambos os lutadores despertam/transformam na mesma partida.
17. **CHEGA, JÁ DEU!** — Combo monstruoso de 15+ acertos com cameo do árbitro.
18. **FUGA DE MESTRE!** — Virar o jogo com golpe pesado após ficar preso no canto.
19. **DUELO DE OLHARES...** — Ambos parados se encarando por 3 segundos fazem um arbusto seco rolar pela arena.
20. **FICHA MÁXIMA — 1UP!** — Encher a super barra no máximo dispara uma ficha dourada girando pela tela.

## Easter Eggs de Cenário e Cameos Ambientais (20)
21. **Dojo — Grou de Origami**: Um grou de papel animado cruza o pôr do sol sobre o telhado.
22. **Dojo — Telhas Trêmulas**: Impactos pesados fazem o velho telhado do dojo tremer.
23. **Neo-Metrópole 2099 — OVNI em Neo-Metrópole**: Um disco voador neon piscante cruza o horizonte cibernético.
24. **Neo-Metrópole 2099 — Sobrecarga // 2099**: Disparar um Super sobrecarrega a rede neon.
25. **Bambuzal ao Luar — Espíritos do Bambuzal**: Máscaras espirituais surgem entre os bambus em combos.
26. **Planalto Rochoso — El Viento Salado**: O galeão voador de Marlow cruza as nuvens ao fundo.
27. **Portão dos Espíritos — Aparição Kitsune**: Uma máscara kitsune brilhante flutua sobre o torii em Supers ou Despertares.
28. **Cratera Vulcânica — Erupção Sísmica**: Golpes pesados provocam um rugido sísmico da caldeira.
29. **Praça do Interior (Brasil) — Capivara de Boa**: Uma capivara em pixel art com uma tangerina na cabeça passeia tranquilamente pela praça.
30. **Praça do Interior (Brasil) — Caramelo na Área**: O guardião oficial da praça dá as caras no Round 2+.
31. **Cristo Redentor (Corcovado) — Tucano no Corcovado**: Um tucano colorido cruza os céus do Rio de Janeiro.
32. **Grande Muralha da China — Dragão de Papel**: Uma pipa sobrevoa as montanhas da Grande Muralha.
33. **Petra (Al-Khazneh) — Escaravelho de Petra**: Um escaravelho dourado brilha no cânion em golpes pesados.
34. **Texas — Meio-Dia no Texas**: Um arbusto seco rola pela rua principal ao meio-dia.
35. **Chichén Itzá — Sombra de Kukulcán**: O brilho esmeralda da serpente emplumada surge em Supers.
36. **Coliseu de Roma — Ave Gladiator!**: O Imperador aplaude combos de 6+ acertos com louros dourados.
37. **Taj Mahal — Pombas de Agra**: Pombas brancas levantam voo sobre o mármore no Round 2+.
38. **Pirâmides de Gizé — Olho de Hórus**: Encher a super barra desperta o escaravelho dourado de Gizé.
39. **Jardins Suspensos da Babilônia — Plumas da Babilônia**: Voo colorido sobre os jardins durante combos aéreos.
40. **Bênção das Maravilhas Antigas** (Estátua de Zeus, Farol de Alexandria, Templo de Ártemis, Colosso de Rodes, Mausoléu de Halicarnasso): Fenômeno mítico exclusivo para cada maravilha clássica.

## Easter Eggs de Lore, Rivalidades e Lutadores (20)
41. **Espelho (Mirror Match)**: Qualquer lutador enfrentando a si mesmo solta uma fala de "impostor".
42. **Suzume vs Oboro**: Banner de vingança do clã Kagerou com emblema do Eclipse Escarlate.
43. **Sylene vs Garan**: Gelo ártico de Borealia contra as chamas violetas do Submundo.
44. **Kaito vs Oboro**: Tempestade de Izu contra o arquiteto do Eclipse Escarlate.
45. **Bruno no Brasil / Corcovado**: A torcida da casa vibra pelo tricampeão de Minas Gerais.
46. **Volta em Neo-Metrópole 2099**: Fala de livre-arbítrio ao retornar à cidade onde foi construído.
47. **Akio Empolgado (<30% de vida)**: Otimismo do Dojo Queda Estelar com ícone de ramen fumegante.
48. **Desculpas do Haruki**: O arquivista gentil pede desculpas no meio da luta após um combo de 5+ acertos.
49. **Marlow vs Valdric**: O pirata dos céus provoca o príncipe sobre sua coroa imperial.
50. **Kaito vs Jin**: Corrida contra o vento entre os Penhascos de Izu e o Vale de Kiso.
51. **Shion vs Kaname**: Lâmina de meteorito contra a katana espiritual (`crossed_blades`).
52. **Lian na Grande Muralha**: A grã-mestra da Lanterna de Jade defende a Grande Muralha.
53. **Bruno vs Daichi**: Choque de gigantes entre o Ringue de Ferro e o Lobo de Altai.
54. **Garan na Cratera Vulcânica**: O Portão de Cinzas se abre onde os 99 monges o selaram.
55. **Valdric vs Oboro**: Duas coroas, um trono — Império Rubro contra Eclipse Escarlate.
56. **Suzume vs Jin**: Duelo de velocidade entre Iga e o Vale do Vento.
57. **Volta vs Haruki**: O androide pergunta ao arquivista o que torna um coração corajoso.
58. **Akio vs Kaito**: Rivalidade clássica — "Quem perder paga o ramen!"
59. **Papagaio do Marlow**: O imediato do *El Viento Salado* cruza a tela em Supers ou combos de 6+ acertos.
60. **Reclamação dos Capangas (`npc_*`)**: Os capangas do modo história reclamam do salário ao levar um combo de 4+ acertos.
