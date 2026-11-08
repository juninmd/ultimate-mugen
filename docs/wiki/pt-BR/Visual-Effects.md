# Efeitos Visuais

Os efeitos de apresentação ficam em `src/vfx.rs` e `src/vfx/`; a simulação nunca os lê.

- **Níveis de impacto** (`src/vfx/impact.rs`): leve (<40 de dano), médio (<90), pesado (hitdef pesado ou >=90) e hyper (>=150). Cada nível define faíscas, brilho, onda de choque, flare, soco de câmera, tremor, flash e aberração cromática. Vale para qualquer personagem, pois lê o evento `Hit` do motor.
- **Rachaduras no chão** (`src/vfx/crack.rs`): golpes pesados e hyper em alvo no chão deixam rachaduras brilhantes que somem (só qualidade alta).
- **Qualidade** (`src/vfx/quality.rs`): segue a opção `fx` (0 baixa, 1 média, 2 alta). As quantidades escalam 0,35 / 0,7 / 1,0; média remove rachaduras e luzes de cenário, baixa também remove detritos e metade do flare.
- As partículas continuam no pool `MAX_PARTS`.
- `UME_SHOTS_ONLY=fx` gera imagens de hit, heavy, hyper, counter, guard e super.
