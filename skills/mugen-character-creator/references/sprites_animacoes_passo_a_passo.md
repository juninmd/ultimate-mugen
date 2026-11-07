# Sprites e Animações Perfeitas: Passo a Passo e Regras

## Parte A. Pipeline de Sprites (passo a passo)

1. **Ficha de design**: silhueta, 1 paleta-base (máx. 15 cores + transparente no índice 0), referência de proporção (altura em pé 100–140 px em 240p; 170–240 px em 480p).
2. **Escolha de resolução**: uma só escala por personagem. Nunca misture pixels de tamanhos diferentes. Redimensione só com vizinho-mais-próximo, em múltiplos inteiros.
3. **Canvas fixo por pose** (ex.: 256x256) e **pivô fixo** nos pés: eixo X no centro do tronco, eixo Y na sola. Todos os quadros usam o mesmo pivô.
4. **Paleta indexada 8 bits**: índice 0 = transparente. Mantenha a ordem de cores igual em todos os quadros e em todas as 12 paletas (`pal1..pal12`). Gere as paletas alternativas trocando só os valores RGB, nunca a ordem.
5. **Desenhe por lista de poses** (mínimo viável):
   - Idle 4–8 q; walk F/B 6–8 q; crouch 3 q; jump (sobe/topo/desce/pouso) 4 q.
   - Ataques: 3 socos, 3 chutes (de pé e agachado), aéreos, cada um com startup/active/recovery.
   - Reações: hit alto/baixo/ar, queda, levantar, KO, guard (pé/agachado/ar).
   - Especiais, super, intro, vitória.
6. **Limpeza**: sem anti-aliasing em 8 bits, sem pixels soltos, bordas com 1 px de contorno consistente, sem cor fora da paleta.
7. **Exportação**: PNG indexado por quadro (`grupo_imagem.png`), nomeado `grupo-imagem`. Grupo/imagem seguem a tabela de ações (0 idle, 20 walk, 200 stand, 400 crouch, 5000 hit, 5150 KO...).
8. **Montagem do SFF**: SFF v1 (PCX, paleta compartilhada) ou v2 (RLE8/RLE5/LZ5/PNG). Defina eixo (x,y) de cada sprite. Retratos: 9000,0 (pequeno 25x25) e 9000,1 (grande 120x140). Use `ume` Auto-Ground/Auto-Clsn quando disponível.
9. **Verificação**: rode `sprite_audit` e `native_sprite_audit` (ver AGENTS.md) para detectar sprites vazios ou ausentes.

## Parte B. Regras de ouro para animações perfeitas

### Alinhamento
- Pés sempre na mesma linha Y (offset 0,0) em animações de solo; só o corpo se move, não o pivô.
- Deslocamento por movimento vai em `Vel`/`posadd` do `.cns`, não no offset do sprite (exceto saltos de animação puramente visuais, já compensados).
- Troque de pose sem "tremor": diferença de pivô entre quadros consecutivos ≤ 2 px, exceto golpes com avanço.

### Tempo (60 ticks/s)
- Cada quadro tem duração inteira em ticks. Idle: 5–8 ticks/quadro; walk: 4–6; golpes rápidos: 1–3 ticks por quadro de startup.
- Startup (golpe leve) 3–6 ticks; médio 6–9; forte 9–14; recovery ≥ startup do mesmo golpe.
- Último quadro de animação não-loop usa duração `-1` (segura até o estado trocar). Loops usam `Loopstart`.
- Soma de ticks da animação ≥ `time` do estado, senão congela no último quadro.

### Caixas de colisão
- `Clsn2` (corpo) em todo quadro, ajustada à silhueta; mantenha largura estável (±4 px) entre quadros.
- `Clsn1` (ataque) apenas nos quadros ativos, cobrindo a ponta do membro e com ~10–20 px de ultrapassagem.
- `Clsn2Default`/`Clsn1Default` para reaproveitar entre quadros iguais; sem Clsn2 = invulnerável.
- Agachado: Clsn2 menor e a mesma largura; ar: Clsn2 compacta.

### Anticipation / Follow-through
- Todo golpe: antecipação (1–2 q), ação (1–2 q ativos), recuperação (2–4 q). Pose de impacto mantida 1–2 ticks extras (hitstop visual).
- Hit-reaction em 3 estágios (impacto, recuo, retorno) com duração ≥ `hittime`.

### Transições e sequência
- Fim de cada golpe volta para quadro compatível com idle/crouch (pose de transição) para evitar "teletransporte".
- Animações de transição: stand→crouch (11), crouch→stand (12), turn (5), pouso (52), em quadros curtos.
- Espelhamento: nunca arte espelhada que mude de mão; use flag `H` no AIR só em quadros simétricos.

### Efeitos e extras
- Flags de composição: `A` (add), `S` (sub), `A1` (add50%). Use só em brilhos/energia.
- Projéteis e explosões: animações separadas (grupo 700+/ 1000+) com pivô no centro.
- Sombra e reflexo vêm do motor; não pinte sombra no sprite.

## Parte C. Formato AIR (exemplo validado)

```
[Begin Action 200]
Clsn2Default: 2
 Clsn2[0] = -14, 0, 14, -80
 Clsn2[1] = -10, -80, 10, -96
0, 0, 0, 0, 4
0, 1, 0, 0, 3
Clsn1: 1
 Clsn1[0] = 14, -70, 56, -58
0, 2, 0, 0, 3
0, 3, 0, 0, 6
0, 4, 0, 0, -1
```

## Parte D. Checklist de homologação (todos devem passar)

- [ ] Todos os sprites referenciados no `.air` existem no SFF (grupo, imagem).
- [ ] Pivô consistente (pés = Y 0) em idle/walk/crouch/ataques.
- [ ] Todo estado listado na tabela (0, 5, 11, 12, 20, 40–52, 100, 120–155, 200–460, 5000–5150, 5110, 5150, 5300) tem animação.
- [ ] Último quadro de golpes com tempo `-1` ou loop correto.
- [ ] Clsn2 em todos os quadros; Clsn1 só nos ativos.
- [ ] Sem cores fora da paleta; índice 0 transparente; 12 paletas coerentes.
- [ ] Retratos 9000,0 e 9000,1 presentes.
- [ ] `sprite_audit`, `hover_check` e `mugen_check` sem erros (executar uma vez ao final).
- [ ] Teste visual: `UME_TOUR=mugen UME_CHARS=<id> ultimate-mugen --shots <dir>`.

## Parte E. Imagem de referência e prompts de geração de sprites pixel art

### E.1 Regra: referência de corpo inteiro obrigatória
Todo personagem tem **1 imagem de corpo inteiro** (character sheet base), gerada ANTES de qualquer sprite e salva em `assets/chars/sprites/<id>/reference.png`.
- Pose neutra de frente/3-4 lateral (voltado à direita), corpo inteiro visível, pés no chão, fundo liso (magenta `#FF00FF` ou transparente), sem texto nem sombra.
- Resolução alvo já em pixel art, canvas fixo (ex.: 256x256), paleta de até 16 cores.
- Aprovada pelo usuário antes de gerar poses. Toda geração seguinte anexa `reference.png` como imagem de entrada (image-to-image / reference) e nunca é feita só por texto.
- Registrar em `reference.md` ao lado: nome, altura em px, paleta hex, traços fixos (cabelo, roupa, cicatrizes, acessórios), lado voltado.

### E.2 Prompt: imagem de referência (corpo inteiro)
```
Pixel art character reference sheet, full body, single character, standing neutral idle pose facing right, 3/4 side view, feet fully visible on the ground line.
Character: {DESCRIÇÃO: idade, corpo, cabelo, roupa, cores, acessórios, arma}.
Style: 2D fighting game sprite (Street Fighter III / King of Fights XIII), hard 1px dark outline, flat cel shading with 2 shade levels, no anti-aliasing, no gradients, no dithering noise, limited palette of max 16 colors including outline.
Canvas {256x256}, character height {120} px, centered, plain solid #FF00FF background, no text, no shadow, no watermark.
```

### E.3 Prompt: pose/quadro (a partir da referência)
```
Using the attached reference image as the exact character design, draw the same character in pose: "{POSE: ex. crouching light punch, frame 2 of 4, fist extended at impact}".
Keep identical: face, hair, outfit, colors, proportions, outline thickness, pixel scale, palette and facing direction (right).
Same canvas {256x256}, feet on the same ground line, same pivot (center of torso x=128, soles y=240), solid #FF00FF background.
Pixel art, no anti-aliasing, no new colors outside the reference palette, no motion blur, no text.
```
Gerar poses-chave primeiro (extremos), depois intermediárias, citando o número do quadro e o quadro anterior como segunda referência.

### E.4 Prompt negativo (sempre anexar)
```
anti-aliasing, blur, gradient, soft shading, 3D render, realistic, photo, extra limbs, extra fingers, different outfit, different hair color, different proportions, cropped body, text, watermark, shadow, background scenery, inconsistent outline, mixed pixel sizes
```

### E.5 Pós-processamento obrigatório (fidelidade)
1. Reduzir/quantizar para a paleta da referência (sem novas cores); índice 0 = fundo.
2. Remover o fundo magenta, alinhar pivô (pés = linha fixa) e recortar com canvas constante.
3. Comparar com `reference.png`: altura ±2 px, cores idênticas, contorno 1 px. Descartar quadros que divergem e regerar.
4. Rodar `sprite_audit` e `native_sprite_coverage.py` ao final.

### E.6 Checklist de fidelidade
- [ ] `reference.png` + `reference.md` existem e foram aprovados.
- [ ] Todo quadro foi gerado com a referência anexada.
- [ ] Mesma paleta, escala de pixel e altura em todos os quadros.
- [ ] Rosto, roupa e acessórios idênticos entre poses.
