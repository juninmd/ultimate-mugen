# Importação de Sprite Atlas

O CLI limita os arquivos de atlas a 64 MiB, as dimensões a 16.384 pixels por lado e 8 milhões de pixels no total, e o canvas de saída a 2.048 pixels para limitar o uso de memória.

O exemplo `import_sprite_atlas` valida e importa um atlas transparente em grade de 3 colunas por 2 linhas com seis poses. O atlas de origem permanece intacto e o importador exporta um PNG independente para cada nome solicitado.

```powershell
cargo run -p ume-engine --example import_sprite_atlas -- --atlas .\poses.png --out assets\chars\sprites\lutador --cols 3 --rows 2 --names 01_idle,02_walk,03_attack,04_crouch,05_hit,06_fall --canvas 400 --validate-only
cargo run -p ume-engine --example import_sprite_atlas -- --atlas .\poses.png --out assets\chars\sprites\lutador --cols 3 --rows 2 --names 01_idle,02_walk,03_attack,04_crouch,05_hit,06_fall --canvas 400
```

`--canvas` usa 400 pixels por padrão. Cada saída tem esse tamanho quadrado; os pixels opacos são redimensionados uniformemente com amostragem pelo vizinho mais próximo, centralizados horizontalmente pela célula e alinhados pelos limites alfa ao chão comum em y=392. O padding transparente da célula não afeta o posicionamento. A referência padrão para a pose em pé é 80% da altura da célula de origem. Use `--reference-height` para informar a altura alfa da pose em pé quando a família do personagem exigir calibração explícita. A mesma referência é aplicada às seis poses, mantendo a escala das poses agachadas e caídas. Apenas os limites opacos transformados precisam caber no canvas.

O importador exige alfa RGBA real, seis células não vazias e um componente alfa conectado relevante em cada pose. Alfa abaixo de 16 é removido dos frames exportados e ignorado na validação de borda e componentes. Poses cuja alfa alcança a borda da célula e atlas com excesso de componentes alfa isolados são rejeitados; inspecione e corrija o atlas em vez de aceitar poses cortadas ou ruído. Essas verificações são diagnósticos, não aprovação de qualidade visual. `--validate-only` também normaliza as seis poses e verifica se os limites opacos cabem antes de aprovar. Os seis PNGs são codificados antes de qualquer arquivo de saída ser gravado.

`meta.json` é mesclado sem descartar campos não relacionados. Cada entrada `frames[name]` importada registra a referência de 320 pixels e os eixos normalizados (`axisX=200`, `axisY=392` por padrão). O importador preserva o `refHeight` de nível superior como estava, inclusive deixando-o ausente, para manter o fallback pela altura da imagem nos frames antigos. Entradas de frame existentes com os mesmos nomes são substituídas. PNGs existentes são protegidos, a menos que `--overwrite` seja informado. `--validate-only` valida e verifica colisões sem gravar arquivos.

Evidência de validação: um atlas RGBA sintético de 300x200 com padding passou pela validação e produziu seis PNGs de 400x400. A decodificação independente confirmou que a última linha opaca era 391, alinhada à borda do chão em axisY=392, sem alfa exportado abaixo de 16. Um fixture com alfa atravessando a borda da célula e uma normalização grande demais foram rejeitados; o modo de validação não criou a pasta de saída. O atlas gerado do ciclo de caminhada do Bruno também passou pela validação e importação com `--reference-height 468`; a saída teve bbox x=115..271, y=75..391 sem alfa abaixo de 16. O SHA-256 do atlas de origem permaneceu `56AED5E92D7F55A80C0CD288DA983E033DA17EEB6F67D8A95D099F0147ACE1D7`.

O gravador PNG converte canais RGBA não pré-multiplicados para a representação pré-multiplicada do tiny-skia antes de codificar. Os bytes do atlas nunca são modificados.
