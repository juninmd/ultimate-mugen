# Efeitos visuais

## Pós-processamento cinematográfico (`assets/postfx.toml`)

`src/postfx/look.rs` lê `postfx.toml` (ao lado do executável ou no `localStorage` na web) e usa o `assets/postfx.toml` embutido como padrão. Seções: `bloom`, `grade` (tonemap ACES, exposição, split tone), `distortion` (shockwave e heat), `lens` e `quality` (nível mínimo de `fx` para cada efeito). Chaves ausentes ficam neutras e valores fora da faixa são limitados.
