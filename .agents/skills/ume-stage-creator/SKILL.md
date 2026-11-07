---
name: ume-stage-creator
description: >-
  Guia definitivo, runbook e manual técnico para criação e integração de cenários nativos
  em Pixel Art (16:9, 1672x941) para o motor Ultimate MUGEN (UME). Cobre prompts padronizados,
  pipeline de arte via rotas de IA e ferramentas locais (Codex CLI, Grok CLI, Forge),
  dimensões, composição de arena (deck contínuo nos 22% inferiores, linha de pés a 85%, centro desobstruído),
  miniaturas (240x135), registro modular em Rust (src/stages.rs e src/stages/*.rs respeitando o teto
  de 250 linhas por arquivo), paletas de fallback 3x3 e bateria de validação (cargo test stages::).
---

# Manual de Criação de Cenários UME: Padrão Pixel Art Arcade 16:9

O motor **Ultimate MUGEN (UME)** suporta dois caminhos para arenas: cenários legados M.U.G.E.N (`.def` + `.sff`) e cenários nativos ilustrados de alta fidelidade em **Pixel Art 16:9**.

Este manual padroniza a criação de cenários no mesmo padrão de excelência de arenas como `colosseum.png`, `texas.png`, `dojo.png` e as Maravilhas do Mundo.

---

## 1. Especificações Técnicas de Mídia

| Atributo | Padrão Nativo UME | Localização no Repositório |
| :--- | :--- | :--- |
| **Resolução Principal** | `1672 × 941` pixels (proporção exata 16:9) | `assets/stages/<id>.png` |
| **Resolução de Miniatura** | `240 × 135` pixels (RGB otimizado) | `assets/stages/thumbs/<id>.png` |
| **Formato de Imagem** | PNG TrueColor (RGB 24-bit ou RGBA 32-bit) | Sem canal alfa no fundo do céu |
| **Estética Visual** | Retro Arcade Pixel Art moderno (Neo-Geo, CPS2, Street Fighter III) | Clusters de pixel nítidos e deliberados, sem blur |
| **Paleta Fallback 3x3** | Matriz `[[u8; 3]; 3]` para ícones rápidos no seletor | Registrado no código Rust em `StageEntry` |
| **Metadados e Prompts** | Registro obrigatório de proveniência | `assets/stages/prompts.json` |

---

## 2. Regras de Composição de Arena de Luta (2D Fighting Deck)

Para garantir legibilidade visual absoluta durante os combates de 60 FPS, todo cenário deve respeitar rigidamente a composição ortográfica:

```
+---------------------------------------------------------------+ 0% (Topo)
| Céu dramático, nuvens estilizadas, iluminação cinematográfica |
| Silhuetas distantes (montanhas, arranha-céus, monumentos)     |
| Elementos temáticos centrais em profundidade (fundo)          |
+---------------------------------------------------------------+ 78% (Início Deck)
| Deck de Combate: Piso Contínuo, Nivelado e Desobstruído       |
| ------------------------------------------------------------- | 85% (Linha dos Pés)
| Base da Arena / Detalhes de Textura do Chão / Borda Inferior  |
+---------------------------------------------------------------+ 100% (Base)
```

1. **Visão Lateral Ortográfica:** Projeção plana sem distorção angular ou perspectiva isométrica.
2. **Piso de Luta nos 22% Inferiores:** O chão deve ocupar exatamente os 22% inferiores da imagem (iniciando em ~78% da altura).
3. **Linha dos Pés dos Lutadores a 85%:** A base onde os lutadores pisam e deslizam fica rigidamente alinhada em 85% da altura total da imagem.
4. **Centro Limpo e Desobstruído:** Nunca posicione caixas, pedras, colunas, escadas, personagens secundários ou obstáculos no plano onde os lutadores caminham. Elementos arquitetônicos ou naturais devem permanecer **atrás** do plano de luta.
5. **Enquadramento Lateral:** Adicione adereços decorativos (árvores, postes, lanternas, bandeiras, vegetação) nas bordas laterais (< 10% e > 90%), sem invadir a área central de combate.
6. **Full Bleed:** A imagem deve cobrir todo o retângulo sem bordas pretas, marcas d'água, lettering ou elementos de interface (UI).

---

## 3. Template Oficial de Prompt de Geração

Todo prompt para criação de cenário deve seguir esta estrutura padronizada (registrada em `assets/stages/prompts.json`):

```text
Use case: stylized-concept. Asset: production background for a side-view 2D fighting game. One single wide 16:9 landscape.
Primary request: <Descrição detalhada do tema, monumentos, atmosfera, iluminação (golden hour, sunset, midnight neon), cores predominantes e texturas>.
Exquisite modern high-quality pixel art with deliberate crisp pixel clusters, richly detailed textures, layered atmospheric depth and masterful cinematic color.
Composition: orthographic side view, broad continuous horizontal fighting floor in bottom 22 percent, fighter foot line at 85 percent image height. Keep center unobstructed and readable for fighters. No obstacles on fighting plane. Full bleed.
No people, characters, UI, readable text, logos, watermark, borders. Entirely pixel art, no blur, no painterly brushwork, no 3D render.
```

---

## 4. Pipeline de Geração e Produção de Arte

### Opção A: Geração via `route_media.py` (Codex CLI / Grok CLI / APIs)
O repositório integra a skill `generate2dmedia` que permite gerar imagens diretamente via CLI autenticada do usuário:

```bash
# 1. Escreva o prompt em arquivo temporário
# 2. Execute route_media
python C:/Users/jr_ac/.gemini/skills/generate2dmedia/scripts/route_media.py image --prompt-file prompt.txt --out-dir out_stage

# 3. Converta e enquadre para a resolução exata 1672x941
python -c "
from PIL import Image
im = Image.open('out_stage/generated.png').convert('RGB')
w, h = im.size
target_ratio = 1672.0 / 941.0
crop_w = int(h * target_ratio)
x0 = (w - crop_w) // 2
cropped = im.crop((x0, 0, x0 + crop_w, h))
final_im = cropped.resize((1672, 941), Image.Resampling.LANCZOS)
final_im.save('assets/stages/<id>.png', optimize=True)
"
```

### Opção B: Geração de Miniaturas Automática
Sempre que uma nova arena for gerada ou alterada, execute o script oficial:

```bash
python scripts/generate-stage-thumbnails.py
```
Isso gera automaticamente `assets/stages/thumbs/<id>.png` na resolução `240 × 135` com filtro Lanczos e compressão otimizada.

---

## 5. Extração da Paleta de Fallback 3x3

O seletor de cenários exige uma matriz 3x3 RGB que representa as cores médias do topo (céu), centro (monumentos/fundo) e base (piso de combate):

```bash
python -c "
from PIL import Image
im = Image.open('assets/stages/thumbs/<id>.png')
im3 = im.resize((3, 3))
print('3x3 thumb palette:', [[list(im3.getpixel((x, y))) for x in range(3)] for y in range(3)])
"
```

---

## 6. Registro no Código Rust (Regra dos 250 Linhas)

O projeto possui um linter rigoroso (`node scripts/file-size.mjs`) com teto estrito de **250 linhas por arquivo fonte**. Arquivos grandfathered não podem crescer.

1. **Novos Grupos de Cenários em Submódulo:**
   - Crie submódulos dedicados em `src/stages/<nome>.rs` (ex: `src/stages/wonders.rs`).
   - Mantenha o arquivo abaixo de 250 linhas.
   - Não adicione comentários no código Rust (nomes devem explicar a intenção).

2. **Estrutura do Submódulo:**
   - Vetor `IDS: [&str; N]`.
   - Arrays de bytes `include_bytes!` para miniaturas e imagens nativas.
   - Funções `illustration_preview(id)` e `illustration(id)` com cache `OnceLock`.
   - Função `builtin() -> Vec<StageEntry>` com as chamadas a `entry(...)`.
   - Testes unitários internos (`#[cfg(test)]`) validando decodificação e dimensões.

3. **Integração no Facade `src/stages.rs`:**
   - Declare o módulo: `mod <nome>;`
   - Encaminhe `illustration_preview`, `illustration` e `prepare_illustration`.
   - Adicione os cenários ao retorno de `builtin()`: `stages.extend(<nome>::builtin());`.

---

## 7. Bateria Obrigatória de Validação

Antes de concluir qualquer alteração de cenário, execute:

```bash
# 1. Gate de tamanho de arquivo (máximo 250 linhas)
node scripts/file-size.mjs

# 2. Testes unitários de decodificação e cache de ilustrações
cargo test --bin ultimate-mugen stages::

# 3. Teste de alocação zero no tick
cargo test -p ume-engine --release --test tick_alloc
```

Nenhum cenário deve ser considerado pronto sem que todos os testes passem com 100% de sucesso.
