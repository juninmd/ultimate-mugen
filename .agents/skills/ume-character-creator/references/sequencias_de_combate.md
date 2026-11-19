# Sequências de combate nativas

1. Use o relatório de `native_sprite_audit` para identificar o PNG resolvido por quadro. Verifique `file`, `resolution`, `pixelFingerprint` e `uniqueBodyImages`. Uma animação pode passar no modo estrito e ainda exibir um único PNG em todas as poses. Foi o caso de `taunt0` e `taunt1`, que resolviam para `06_victory.png`.
2. Gere a primeira pose a partir da arte canônica do próprio lutador. Para a segunda, use o primeiro quadro como referência direta e peça a alteração apenas na região que se move. Use a mesma escala nos dois PNGs e registre o corpo pelo apoio dos pés e pela posição do tronco. Confira o canal alfa sobre fundo claro e escuro. A prévia bruta pode conter uma franja semitransparente em torno do desenho.
3. Inspecione a folha de contato e confira os PNGs integrados e suas entradas em `meta.json`. Audite a resolução novamente e capture os dois quadros em jogo. Para a provocação, abra o build web com `?p1=shion&p2=<id>&modo=versus&luta=1&e2e=1`. O estado `19901` no lado 2 indica a provocação da abertura. Registre mais de um quadro e confira visualmente as poses, pois a captura pode cair na mesma fase do ciclo.

## Backlog de reações a dano

Na primeira medição do elenco de 21 personagens, `hitLight`, `hitLowLight` e `hitCrouch` apresentaram pelo menos um par de quadros consecutivos com a mesma impressão digital em todos os personagens. `hitAir` teve 36 quadros resolvidos por poses compartilhadas. Use esses resultados para priorizar a inspeção e diferenciar uma pausa intencional de uma pose de recuo ausente.

- [ ] Auditar o elenco para `hitLight`, `hitMedium`, `hitHeavy`, `hitLowLight`, `hitLowMedium`, `hitLowHeavy`, `hitCrouch` e `hitAir`. Identificar por lutador quais quadros faltam, resolvem para outra pose ou repetem a mesma imagem. Verificar também a transição para queda, quique, chão e recuperação.
- [ ] Criar um piloto de reação com impacto e recuo legíveis. Variar a amplitude entre golpes leves e fortes, tratar agachamento e ar, e preservar a identidade, o eixo e os tempos do estado. Diferenciar reação a dano de reação à defesa.
- [ ] Capturar no jogo golpes reais leves, médios e fortes e as reações baixas, agachadas e aéreas. Revisar impacto, recuo e transição para o estado seguinte antes de estender o padrão ao elenco.

No piloto aéreo de Akio e Spitfire, uma execução do Whirl Launcher exibiu o segundo elemento de `hitAir` por apenas um tick antes de trocar para `fall`. A duração `-1` no último quadro permite mantê-lo até a troca de animação pelo estado. Use uma simulação tick a tick para confirmar a ordem dos elementos e a aterrissagem quando a captura web saltar um quadro curto.

No piloto de dano leve do Kaito, um jab real do Akio levou ao estado 5000 durante o hitstop. A animação 5000 avançou pelos elementos de impacto e recuo já no estado 5001. Quando o tempo de reação terminou, o motor voltou à animação 0 diretamente. Para capturar o recuo no navegador, use `?e2e=1` e acompanhe `anim` e `anim_elem` (índice a partir de zero) em `window.__ume_state`. No piloto, `anim=5000` e `anim_elem=1` identificam o recuo. Um quadro de guarda no fim da sequência pode ser substituído pelo primeiro quadro da guarda quando o estado termina.

Execute `native_sprite_audit <out.json> --require-complete --require-hit-reactions` e confira `hitReactionGaps`. Para verificar uma reação dedicada, compare os dois primeiros elementos pelo `pixelFingerprint` e confirme que ambos resolvem para os PNGs previstos. Em `hitAir`, compare também as impressões digitais com as de `fall` para detectar uma cópia com outro nome. A resolução estrita cobre a disponibilidade dos quadros e pode passar com imagens repetidas.

Quando um NPC de história usa `spriteSet`, verifique o manifesto e a resolução do NPC separadamente. Um PNG novo no pacote compartilhado não muda a sequência do NPC se seu manifesto continuar solicitando a pose antiga. Preserve os tempos originais e confira ambos em combate.
