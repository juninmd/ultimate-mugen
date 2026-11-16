# Cobertura de animações nativas

Esta entrega completa a arte dedicada de poses do elenco nativo: todo quadro de animação que o motor
consegue nomear resolve para o próprio arquivo PNG. Os canvases numerados compartilhados (`09_hit1`,
`13_fall2`, ...) continuam atendendo os aliases legados, e quadros de efeito puramente procedurais
(`boltHit`, feixes) ficam renderizados pelo motor por design; ambos são resoluções aprovadas, não lacunas.

## Incluído nesta etapa

- As imagens dedicadas `portrait.png` dos dezesseis lutadores seguem nos cartões do seletor e no HUD.
- Toda pose textual referenciada por uma animação tem PNG dedicado em `assets/chars/sprites/<id>/`,
  incluindo defesa, agachamento, salto, flip, turn, land, vitória, intro, taunt, o conjunto de derrotas
  `df*` e as poses de dano baixo que antes reutilizavam idle. Poses inline têm o quadro `inline_<hash>`
  renderizado uma única vez.
- Os PNGs dedicados devem ser gerados em pixel art arcade via `generate_image` usando uma âncora canônica validada do próprio personagem (a ferramenta depreciada `generate_missing_sprites` baseada no rig vetorial foi removida). Escreva quadros 400×400 RGBA com eixo (200, 392) e metadado `refHeight` 320, seguindo a convenção existente.
- Verificação: `cargo run --release -p ume-engine --example native_sprite_audit -- <out.json> --require-complete`
  deve passar para todos os lutadores, e `cargo run --release -p ume-devtools --bin ume-tools -- audit-sprites --out <report.json>` deve
  ficar limpo em geometria e bordas alfa. O `cargo run --release -p ume-devtools --bin ume-tools -- audit-moves` é a auditoria estática do elenco (inputs, cancels, throws, hit boxes).

## Ciclo de QA com sprite-gen

A skill [sprite-gen](https://github.com/aldegad/sprite-gen) segue como ferramenta oficial de QA de qualidade
artística nos loops nativos, sem exigir credenciais de geração de imagem:

- `ume-tools motion-descriptor --character assets/chars/<id>.json --sprites assets/chars/sprites/<id> --animation walk --out walk.json`
- `sprite-gen inspect-motion --source walk.json --out walk-report.json` mede âncoras, quadros duplicados,
  deriva da caixa e contato com as bordas; contato de pé exige `--contacts` e `--foot-box` explícitos.
- `ume-tools preview-motion` e `ume-tools preview-scene` renderizam GIFs e MP4s de revisão
  com os estágios do sprite-gen. Veja [Importação de Sprite Atlas](Importacao-Sprite-Atlas.md) pelo caminho
  de importação component-row quando se quiser arte gerada por IA.

## Limites conhecidos

Inventários e relatórios de auditoria não comprovam que as animações ocorrem naturalmente em partida:
gatilhos de estado, fluidez de transição e ajuste das hitboxes precisam ser validados no Treino. Recores são
aplicados no carregamento pelo sistema de paletas, então a conferência visual deve cobrir cada paleta de um
lutador, não apenas a primeira.
