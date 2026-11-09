# Táticas da CPU

`crates/engine/src/ai/tactics.rs` adiciona reações à IA a partir do nível 3: anti-aéreo (rival no ar, descendo, a até ~105 unidades: comando que começa com `F, D, DF`), punição de golpe errado (rival atacando sem contato, logo fora do alcance), confirmação de acerto (encadeia comando de vários passos depois de acertar) e disciplina de medidor (`power >= cost`; EX guarda uma barra; super só com rival em hit-stun, abaixo de 40% de vida ou o lutador abaixo de 50%).

Evidência em `docs/evidence/ai-tactics/`: taxa de nocaute de 83,4% para 96,7% e rodada média de 70,9 s para 54,5 s, sem lutadores travados. Veja [AI-Tactics](../AI-Tactics.md).

## Leitura do rival (`ai/reading.rs`)

| Habilidade | Comportamento |
|---|---|
| Atraso de reação | A CPU só enxerga o ataque após `(8 - nível) * 2` ticks: níveis baixos bloqueiam tarde e o nível 8 lê na hora |
| Altura da guarda | Bloqueia baixo ou alto conforme o `guardflag` do ataque, sem chutar |
| Punição de bloqueio | Golpe do rival foi defendido e ele ainda se recupera: responde com um golpe de um só botão (mais vezes em níveis altos) |
| Perfil do rival | Médias móveis de quanto o rival pula e defende: quem pula muito aumenta a chance de anti-aéreo, quem fica na guarda aumenta a de atacar |

Evidência em `docs/evidence/ai-reading/`: `cargo run --release -p ume-engine --example ai_ladder` joga espelhos entre níveis (nível 8 vence o 3 em 66,7 % → 86,5 % dos rounds, nível 6 vence o 3 em 54,8 % → 81,0 %), e `native_duel` mostra nocaute de 96,7 % → 99,4 % com o round médio de 54,5 s → 45,6 s.
