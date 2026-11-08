# Guia de Sprites, Rig Vetorial e Arte Pixel para UME

O Ultimate MUGEN oferece dois pipelines de renderização gráfica independentes para personagens UMF:
1. **Pipeline Procedural / Vetorial (Rig)**: O personagem é descrito por ossos, membros angulares e preenchimentos vetoriais em `art.style` e `art.poses`.
2. **Pipeline de Arte Pixel (PNG / Sprite Pack)**: Sprites pré-renderizados em arquivos PNG transparentes alinhados na pasta `assets/chars/sprites/<id>/`.

---

## 1. O Pipeline Vetorial (Rig)

No pipeline vetorial, nenhuma textura bitmap externa é necessária. O motor desenha o lutador em tempo real através do rasterizador tiny-skia / wgpu em 60 quadros por segundo.

### Anatomia de `art.style`
Define a compleição física, roupas e esquema de cores:
- `build`: Escala global da massa corporal (ex: `1.04`).
- `top`: Modelo da vestimenta superior (`"gi"`, `"shirt"`, `"jacket"`, `"armor"`, `"none"`).
- `pants`: Modelo da vestimenta inferior (`"gi"`, `"pants"`, `"shorts"`).
- `gloves`: `"wrap"` (faixas), `"gloves"`, `"barefoot"`.
- `hair`: Objeto com estilo de cabelo, quantidade de mechas pontiagudas (`spikes`), curvatura (`sweep`) e brilho.
- `colors`: Dicionário com `skin`, `hair`, `eye`, `top`, `top2`, `trim`, `pants`, `belt`, `wrap`.

### Poses Angulares (`art.poses`)
Cada pose define os ângulos em graus de cada junta:
- `shoulderF` / `elbowF`: Braço frontal (ombro e cotovelo).
- `shoulderB` / `elbowB`: Braço traseiro.
- `thighF` / `kneeF`: Perna frontal (coxa e joelho).
- `thighB` / `kneeB`: Perna traseira.
- `lean`: Inclinação do tronco em graus.
- `head`: Rotação da cabeça.
- `phase`: Fase da animação de respiração / ondulação de faixas e cabelo.

---

## 2. O Pipeline de Arte Pixel (Sprites PNG)

Quando a pasta `assets/chars/sprites/<id>/` existe no disco, o motor automaticamente substitui o rig pelos sprites em pixel art.

### Nomenclatura Padrão de Poses
| Arquivo | Uso no Motor |
| :--- | :--- |
| `01_idle.png` | Base de repouso, caminhada, corrida, agachamento e salto caso não haja pose específica. |
| `02_attack1.png` a `06_victory.png` | Ataques normais, especiais, projéteis e vitória. |
| `07_idle1.png`, `08_idle2.png` | Frames de respiração da animação idle. |
| `09_hit1.png` a `11_hit3.png` | Frames de reação a golpes recebidos (`hitLight`, `hitMedium`, `hitHeavy`). |
| `12_fall1.png`, `13_fall2.png` | Queda e arremesso pelo ar. |
| `14_lie.png` | Nocaute, chão e derrotas. |
| `15_power0.png` a `17_power2.png` | Animação de transformação / despertar. |

### `meta.json` e Escala Canônica
O arquivo `assets/chars/sprites/<id>/meta.json` padroniza o tamanho do lutador:
```json
{
  "refHeight": 112
}
```
`refHeight` indica a altura em pixels do corpo em pé no `01_idle.png`. O motor normaliza o lutador para 100 unidades de altura do UME, garantindo proporções exatas contra lutadores vetoriais ou clássicos.

### Regiões de Despertar de Cabelo (`awakeningHair`)
Para transformar apenas parte do corpo (ex: cabelo Super Saiyajin ou olhos brilhantes) mantendo a sprite de pixel art original intacta:
```json
{
  "frames": {
    "01_idle": {
      "awakeningHair": [35, 12, 75, 48]
    }
  }
}
```
As coordenadas `[left, top, right, bottom]` delimitam onde a nova paleta de cor será aplicada. O restante do corpo preserva integralmente suas cores originais.

---

## 3. Geração e integração de Pixel Art com referência

Use a ferramenta de geração de imagens para criar ou reparar PNGs quando o personagem já usa sprites bitmap. Preserve os demais fluxos UMF deste guia: gerar arte não substitui configurar animações, metadados, poderes ou validar o jogo.

### Preparação da referência
- Abra o sprite de melhor qualidade do próprio personagem e use-o como referência visual explícita. Prefira uma pose nítida, corpo inteiro, de frente lateral ou 3/4 coerente com o movimento alvo.
- Registre dimensões, caixa alfa, linha do chão, eixo/pivô, altura de referência e orientação da pose de referência antes de gerar. Uma imagem que parece semelhante pode ter escala ou enquadramento diferentes.
- Faça inventário das animações e sprites já existentes; identifique lacunas ou defeitos concretos. Não substitua sprites bons sem necessidade.

### Exceção: retratos de interface
Retratos como `portrait.png` podem usar uma pixel art de alta resolução, com clusters menores, shading e detalhes faciais mais finos que os sprites de combate. Preserve essa diferença de acabamento: o retrato continua sendo pixel art, mas não deve ser simplificado, reduzido nem alinhado ao pivô/tela das poses de jogo.

- Use o retrato detalhado aprovado do próprio personagem como referência visual. Se ele foi removido ou substituído por engano, restaure a versão original do controle de versão em vez de tentar recriá-la; ela é a referência mais fiel.
- Antes de criar ou restaurar, confirme o `portrait.png` realmente usado pelo personagem e veja o retrato Kaito aprovado em `assets/chars/sprites/kaito/portrait.png`. Ele define o padrão de acabamento: busto quadrado de alta resolução, clusters de pixel visíveis, contorno deliberado, shading rico e detalhes finos no rosto, cabelo e roupa. Use também uma sprite do próprio personagem para manter identidade, roupa e paleta; Kaito é referência de qualidade, não de design do personagem.
- Para criar um retrato ausente sem original versionado, peça um busto/close-up quadrado, centralizado e legível no slot de UI, com cabeça, cabelo, acessórios e ombros inteiros, expressão clara e silhueta limpa. Preserve a densidade de clusters e o contraste do retrato Kaito; faça os detalhes por agrupamentos de pixels intencionais, sem acabamento de pintura suave, antialiasing, blur, gradientes ou aparência vetorial. Não converta uma pose de combate pequena em retrato por simples ampliação.
- Prompt-base: “Crie um retrato de personagem em pixel art de alta resolução para interface, busto quadrado com fundo transparente. Use a imagem de referência do retrato Kaito apenas como referência de qualidade e acabamento em pixel art; use a referência do personagem solicitado para identidade, rosto, cabelo, roupa, acessórios e paleta. Mantenha contorno nítido, clusters de pixels deliberados, sombreamento rico e detalhes faciais legíveis. Centralize cabeça e ombros com margem segura, sem cortes, texto, objetos extras, blur, antialiasing ou fundo.” Ajuste as características específicas sem remover essas restrições.
- Gere em alta resolução adequada ao slot de UI, com transparência real e canal RGBA; enquadre cabeça, faixa/acessórios e ombros sem cortes acidentais. O retrato Kaito aprovado mede 1254 × 1254 px e serve como referência concreta de resolução e acabamento, não como tamanho obrigatório para todo projeto. Não aplique chroma key, recorte de sprite sheet nem redimensionamento suave quando já houver transparência.
- Confira o PNG ampliado e no tamanho real do menu, sobre fundos claros e escuros; verifique halo, bordas semitransparentes indesejadas, leitura do rosto e fidelidade à identidade do personagem. Mantenha o retrato fora das poses auditadas como animação e não altere os demais retratos ao corrigir um único arquivo.

### Prompt de geração
- Descreva o personagem e a ação da pose sem ambiguidade. Trave roupa, paleta, proporções, penteado, faixa, silhueta, contorno, direção e tamanho relativo à referência.
- Peça pixel art com pixels duros e clusters limpos, transparência real, sem antialiasing, blur, glow, gradientes, texto, fundo ou objetos extras. Para efeitos, solicite-os explicitamente apenas quando fizerem parte da pose.
- Para uma animação, prefira gerar e revisar poses individuais a partir da mesma referência canônica. Se gerar uma tira, exija células iguais e separação clara; confirme a contagem e os limites das células antes de cortar. Modelos podem produzir dimensões, escalas, direção ou personagem inconsistentes entre chamadas, mesmo com prompts parecidos.
- Não trate a imagem gerada como pronta. Compare cada pose lado a lado com a referência e com as poses vizinhas; rejeite dedos/membros duplicados, acessórios flutuantes, recortes, mudança de roupa, inversão de lado, halo colorido ou bordas semitransparentes indesejadas.

### Normalização e integração
- Recorte ou componha cada pose em uma tela de dimensões consistentes para aquela família de animação. Mantenha os pés na mesma linha de chão quando aplicável, alinhe o eixo em `meta.json` e mantenha `refHeight` compatível. Não copie eixos de uma imagem de tamanho diferente sem recalcular.
- Preserve transparência RGBA. Inspecione o canal alfa e os pixels de borda em mais de uma cor de fundo para detectar matte/franja; uma porcentagem alta de alfa parcial, isoladamente, não prova defeito. Não aplique limiar global de alfa, propagação de cor ou troca de matiz no sprite inteiro: esses filtros podem apagar sombreado e destruir a arte. Teste a correção numa cópia, amplie as bordas e compare também o desenho central antes de integrar.
- Não aumente ou reduza sprites com interpolação suave. Se a escala precisar mudar, use vizinho mais próximo e confira visualmente a silhueta e os clusters.
- Atualize o manifesto de animação para referenciar todos os quadros em ordem, com ticks coerentes com a velocidade. Confirme que nenhum nome de pose foi omitido ou duplicado acidentalmente.
- Faça uma folha de contato com nomes e dimensões, confira todas as poses em tamanho ampliado e no tamanho de jogo. Depois execute a auditoria nativa de sprites e capture o estado relevante em runtime; uma folha de contato ou teste estrutural não prova a leitura visual no jogo.

### Evidência e encerramento
- Antes de encerrar, compare os PNGs do pacote com os arquivos realmente resolvidos pela auditoria nativa. Apague apenas aliases sem referência nem papel de fallback; preserve retratos e stems base como `01_idle` e `04_special` quando o carregador os usa como fallback.
- Informe os arquivos substituídos/criados e mantenha backup dos originais antes de mudanças em lote.
- Diferencie auditoria de cobertura, testes de lógica, inspeção de PNG e validação visual em runtime. Só afirme uma verificação quando ela realmente foi executada.
- Para animações de locomoção, valide a sequência inteira em movimento: identidade, direção, altura, eixo dos pés, continuidade e ausência de saltos entre quadros.

- Audite cada quadro efetivamente resolvido, não apenas os nomes dos movimentos: uma pose de fallback pode servir a várias animações e uma imagem nomeada pode continuar ativa sem estar óbvia no manifesto. Substitua o fallback compartilhado quando estiver cortado e forneça PNGs nomeados para as poses específicas que precisam de silhuetas diferentes.
- Antes de apagar um PNG, confira a chave que o loader realmente resolve: `frame.sprite` sobrescreve `frame.pose`; uma pose vetorial declarada em `art.poses` pode continuar sendo usada para a geometria enquanto outra imagem nomeada fornece o pixel art. Trate os fallbacks implementados em `native/pixel.rs` como dependências e preserve os retratos da UI.
- Se uma pose declarativa for um objeto inline, mantenha esse objeto ao trocar a arte. O motor deriva a chave `inline_<hash>` do conteúdo; um PNG com essa chave troca só a imagem. Substituir a pose inline por uma pose nomeada diferente pode mudar hitboxes e índices de sprite e deve ser tratado como alteração de comportamento.
- Em folhas de células, confira os limites de cada célula e o resultado recortado; remova o fundo conectado às bordas, preserve a resolução por vizinho mais próximo e rejeite membros cortados, pixels soltos e efeitos que cruzem para outra célula. Verifique novamente cada PNG em folha de contato no tamanho nativo.
- Mantenha poses distintas ao longo de sequências de ataque: quadros de carga, impacto e recuperação não devem repetir o mesmo desenho só para preencher nomes. Se a geração produzir uma pose inadequada, gere outra ou use a pose intacta mais próxima apenas quando a silhueta e a função continuarem legíveis.
- Em spritesheets multi-quadro, não confie apenas na contagem ou na grade regular: compare pose, silhueta, roupa e acessórios de cada quadro com o `walk0` e a referência de estilo. Um modelo pode repetir quadros não adjacentes (por exemplo, 1≈3 e 2≈4) mesmo quando todas as células estão presentes. Rejeite repetições não intencionais ou desvios de identidade; regenere o quadro defeituoso individualmente e componha a folha novamente.

## 4. Gerador Automático de Sprites Faltantes
Para gerar imagens PNG dedicadas a partir do modelo vetorial:
```bash
cargo run --release -p ume-engine --example generate_missing_sprites
```
O comando analisa `assets/chars/*.json` e renderiza os quadros transparentes pré-alinhados no pivô dos pés.
