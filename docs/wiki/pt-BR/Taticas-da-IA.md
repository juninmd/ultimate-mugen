# Táticas da CPU

`crates/engine/src/ai/tactics.rs` adiciona reações à IA a partir do nível 3: anti-aéreo (rival no ar, descendo, a até ~105 unidades: comando que começa com `F, D, DF`), punição de golpe errado (rival atacando sem contato, logo fora do alcance), confirmação de acerto (encadeia comando de vários passos depois de acertar) e disciplina de medidor (`power >= cost`; EX guarda uma barra; super só com rival em hit-stun, abaixo de 40% de vida ou o lutador abaixo de 50%).

Evidência em `docs/evidence/ai-tactics/`: taxa de nocaute de 83,4% para 96,7% e rodada média de 70,9 s para 54,5 s, sem lutadores travados. Veja [AI-Tactics](../AI-Tactics.md).
