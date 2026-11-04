# Cobertura de animações nativas

Esta entrega é parcial. Ela atualiza artes e o carregador, mas não completa o conjunto de animações do elenco nativo.

## Incluído nesta etapa

- Imagem dedicada `portrait.png` para os dezesseis lutadores. O seletor usa essas imagens nos cartões e o HUD as prioriza nos retratos.
- 156 novos quadros de movimento instalados para Kaito e parte do elenco: idle, caminhada, corrida, agachamento, salto, defesa, dano, recuperação e ataques. A cobertura varia por lutador; essa contagem não significa que cada sequência esteja completa.
- O carregador nativo respeita a altura declarada positiva de cada lutador. Os campos opcionais `refHeight`, `axisX` e `axisY` por quadro alinham escala e apoio dos pés das novas artes.
- Efeitos procedurais deixaram de reutilizar a arte idle do lutador. A auditoria nativa aponta reutilizações e quadros ausentes.

## Lacunas conhecidas

Muitas animações ainda usam poses compartilhadas ou idle, e a quantidade de quadros varia entre os personagens. Caminhada para frente e para trás, corrida, dano, transformação, salto, defesa, recuperação, ataques e vitória ainda não têm sequências completas para todos. Algumas poses inline não possuem PNG correspondente e continuam usando o fallback de compatibilidade. A presença do retrato não comprova sua qualidade visual em todos os cartões e estados do HUD.

A sonda `native-coverage` captura uma comparação de altura com câmera fixa e quadros individuais de idle, caminhada, recuo, corrida, dano e transformação. Ela seleciona os quadros diretamente; não comprova gatilhos naturais de jogo nem fluidez. Se faltar uma animação exigida, a sonda falha informando personagem e ID.

Use o exemplo `native_sprite_audit` para comparar arquivos resolvidos e imagens distintas por pixels. Use `scripts/audit_native_sprites.py` para inventariar dimensões e transparência dos PNGs. Nenhum inventário substitui a inspeção visual ou testar as animações jogando.

A geometria por quadro é opcional. `refHeight` é a altura em pixels da origem que representa a escala nominal em pé; os eixos são âncoras nos pixels da origem. O carregador aplica a resolução de arte depois desses valores.
