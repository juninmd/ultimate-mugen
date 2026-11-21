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
