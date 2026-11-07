# Pipeline de Arte e Composição de Cenários UME

Este documento detalha as regras visuais, composição geométrica, prompt engineering e técnicas de pós-processamento para arenas de combate 2D no motor Ultimate MUGEN.

---

## 1. Grade de Composição 16:9 (1672x941)

Para evitar que lutadores pareçam flutuar ou colidir com elementos do fundo, todo cenário obedece às seguintes divisões verticais:

```
Y = 0px    (0%)   ------------------------------------------------------------
                  Céu / Atmosfera / Nuvens / Iluminação de borda
Y = 300px  (32%)  ------------------------------------------------------------
                  Monumento / Arquitetura Principal (plano de fundo intermediário)
Y = 550px  (58%)  ------------------------------------------------------------
                  Linha do horizonte / Silhuetas distantes / Vegetação secundária
Y = 734px  (78%)  ------------------------------------------------------------
                  Início do Deck de Luta (textura do chão começa aqui)
Y = 800px  (85%)  ============================================================
                  LINHA DE CONTATO DOS PÉS DOS LUTADORES (Fighting Plane Floor)
Y = 941px  (100%) ------------------------------------------------------------
```

### Regras Críticas de Composição:
1. **Horizontalidade Estrita do Chão:** O deck de combate (entre Y=734 e Y=941) deve ser uma superfície horizontal contínua de ponta a ponta. Não utilize rampas acentuadas, degraus profundos no centro da tela ou declives.
2. **Desobstrução Central:** O retângulo central de combate (X: 15% a 85%, Y: 60% a 95%) não pode conter caixas, pedras, fontes, colunas em primeiro plano, corpos de água navegáveis ou NPCs. Os lutadores precisam de contraste limpo contra o cenário.
3. **Profundidade em Camadas (Layering):**
   - **Fundo distante:** Céu, montanhas, estrelas, sol ou lua.
   - **Plano médio:** O monumento ou arquitetura ícone (Coliseu, Pirâmide, Fachada do Texas Saloon).
   - **Primeiro plano suave (apenas nas bordas):** Tochas, lanternas, árvores ou bandeiras nas margens laterais (< 10% ou > 90%) aumentam a sensação de imersão sem interferir na leitura do hitbox.

---

## 2. Anatomia de Prompts Padronizados

Os prompts devem ser redigidos em inglês no arquivo `assets/stages/prompts.json` com a estrutura:

```json
{
  "id": "nome_do_cenario",
  "name": "Nome de Exibição",
  "category": "wonders | world | fictional",
  "aspect_ratio": "16:9",
  "target_dimensions": "1672x941",
  "prompt": "Use case: stylized-concept. Asset: production background for a side-view 2D fighting game. One single wide 16:9 landscape. Primary request: <LOCAL / TEMA / CLIMA / HORA DO DIA / CORES>. Exquisite modern high-quality pixel art with deliberate crisp pixel clusters, richly detailed textures, layered atmospheric depth and masterful cinematic color. Composition: orthographic side view, broad continuous horizontal fighting floor in bottom 22 percent, fighter foot line at 85 percent image height. Keep center unobstructed and readable for fighters. No obstacles on fighting plane. Full bleed. No people, characters, UI, readable text, logos, watermark, borders. Entirely pixel art, no blur, no painterly brushwork, no 3D render."
}
```

---

## 3. Rota de Geração Recomendada

A geração padrão do projeto utiliza a ferramenta integrada `route_media.py`:

```bash
# Execução via Codex CLI local (sem consumo de cota cloud desnecessária)
python C:/Users/jr_ac/.gemini/skills/generate2dmedia/scripts/route_media.py image \
  --prompt "..." \
  --out-dir assets/stages/temp_gen
```

---

## 4. Otimização e Conversão PNG

Ao salvar a imagem final:
1. Corte para proporção 16:9 centrada: `crop_w = int(h * 1672.0 / 941.0)`.
2. Redimensione para exatamente `1672x941` utilizando o algoritmo Lanczos.
3. Gere a miniatura `240x135` em `assets/stages/thumbs/<id>.png`.
4. Extraia a matriz 3x3 de cores para fallback do HUD seletor.
