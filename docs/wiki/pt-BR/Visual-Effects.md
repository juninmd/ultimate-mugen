# Efeitos Visuais

Os efeitos de apresentação ficam em `src/vfx.rs` e `src/vfx/`; a simulação nunca os lê.

- **Níveis de impacto** (`src/vfx/impact.rs`): leve (<40 de dano), médio (<90), pesado (hitdef pesado ou >=90) e hyper (>=150). Cada nível define faíscas, brilho, onda de choque, flare, soco de câmera, tremor, flash e aberração cromática. Vale para qualquer personagem, pois lê o evento `Hit` do motor.
- **Rachaduras no chão** (`src/vfx/crack.rs`): golpes pesados e hyper em alvo no chão deixam rachaduras brilhantes que somem (só qualidade alta).
- **Qualidade** (`src/vfx/quality.rs`): segue a opção `fx` (0 baixa, 1 média, 2 alta). As quantidades escalam 0,35 / 0,7 / 1,0; média remove rachaduras e luzes de cenário, baixa também remove detritos e metade do flare.
- As partículas continuam no pool `MAX_PARTS`.
- `UME_SHOTS_ONLY=fx` gera imagens de hit, heavy, hyper, counter, guard e super.
- **Super e despertar** (`src/vfx/awaken.rs`): anéis em camadas (peito e chão), pilar de energia, rajada de faíscas em 360 graus, rachaduras no chão em qualidade alta.
- **Dash** (`src/vfx/dash.rs`): riscos de velocidade e anel no chão ao iniciar um dash.
- **Final do KO** (`src/vfx/ko.rs`): três ondas de choque, núcleo branco, faíscas em 360 graus, cortes radiais, detritos e rachaduras no perdedor. Custo do lado da simulação: `cargo test --release --bin ultimate-mugen cost -- --ignored --nocapture`.

## Formas com anti-aliasing

Formas vetoriais desenhadas por `ume_shell::quad` levam uma franja de alfa de um pixel montada nos vértices (`crates/render/src/canvas/shape.rs`), então barras inclinadas da UI, círculos, anéis e linhas diagonais têm bordas suaves em todos os backends, sem MSAA.

- `draw_convex` (usado por `Ui::quad`), `draw_circle`, `draw_ellipse`, `draw_poly`, `draw_circle_lines` e `draw_line` são suavizados. `draw_triangle`, `draw_mesh` e retângulos continuam duros, porque são usados como peças de malhas maiores e a franja deixaria emendas.
- Arestas alinhadas aos eixos nunca são suavizadas: já caem na grade de pixels, e faixas empilhadas continuam encostadas.
- Cantos usam mitra com limite de 3x, e formas com raio inscrito abaixo de 1 px voltam ao leque simples.
- Linhas e anéis mais finos que 1 px mantêm a largura e reduzem o alfa, então fios finos param de piscar.
- `glow` (`src/glow.rs`) é uma única malha de gradiente radial (241 vértices, uma camada de overdraw) cuja queda integra os 14 discos empilhados que substituiu (910 vértices, 14 camadas), então o visual é o mesmo, sem faixas.

Custo, medido com `UME_TOUR=perf` (Kaito vs Shion, 1280x720): os frames de luta foram de 2999 para 3921 vértices em média e a montagem do batch de 0,20 para 0,23 ms; as draw calls não mudaram (12,5).
