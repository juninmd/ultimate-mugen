# Referências AAA → o que foi implementado

Pesquisa sobre o "game feel" dos jogos de luta atuais (Street Fighter 6, Tekken 8, Guilty Gear -Strive-, Mortal Kombat, Smash) e guias de *juice*/*hit-stop*. Princípios aplicados:

| Princípio (fonte/prática) | Implementação |
|---|---|
| Um golpe satisfatório são **5–8 micro-respostas em ~100 ms**: som, partículas, hit-stop, flash, empurrão, tremor, número | `vfx::Vfx::event` dispara juntos: rajada de faíscas direcionais, brilho aditivo, anel de choque, poeira, tremor, *camera punch*, aberração cromática e número de dano |
| **Hit-stop** dá peso (já é do motor) + congelar com zoom (Smash, MK) | *punch-in* de câmera com mola a cada golpe (maior em golpes pesados) |
| **Tremor "trauma²"**: golpes leves quase não mexem, pesados chutam forte | `Vfx::shake` (trauma elevado ao quadrado, decai em ~0,6 s) |
| **KO cinematográfico**: câmera lenta + zoom no perdedor (SF6/Tekken 8) | tempo a 25 % por ~1 s e retorno suave, zoom até +28 % no perdedor, blur radial, dessaturação, vinheta vermelha, anéis e explosão de faíscas |
| **Super / Rage Art**: corte dramático | zoom, linhas de velocidade, aura de partículas subindo, anéis pulsantes, flash, banner "SUPER ARTS" |
| **Pós-processamento** (Strive/Tekken 8): bloom, vinheta, correção de cor, aberração cromática | `postfx.rs`: cena → render target MSAA → bright-pass → blur gaussiano separável ¼ res (2 passes) → composição (bloom, CA radial, blur radial, grade de cor, vinheta, grão) |
| **Reflexo e sombra de contato** (cenários polidos) | reflexo espelhado com fade por altura nos cenários procedurais; sombra com 3 camadas suaves |
| **Feedback de combate legível**: contra-ataque, guarda, perfeito | banners "COUNTER!" (acertar quem estava atacando), "PERFECT!", rótulo "GUARD" azul |
| **HUD vivo**: alarme de vida baixa, medidor MAX, timer urgente, combo com *pop* | barras com pulso vermelho < 25 %, medidor com brilho/“MAX”, timer vermelho e pulsante nos últimos 10 s, contador de hits que “estoura” e esquenta de cor |
| **UI com transições** | wipe diagonal entre telas, itens do menu entram escalonados, seleção pulsante, partículas e feixes de luz no fundo, linhas de velocidade na tela de VS, brasas na vitória |
| **Acessibilidade**: opção para reduzir flashes/tremor | Vídeo → *Efeitos de impacto*: Reduzido (40 %) · Normal · Intenso; *Pós-processamento* Desligado/Médio/Alto |

Tudo é **apenas apresentação**: a simulação determinística (60 Hz) não muda — só o ritmo em tempo real (câmera lenta) e a câmera visual.

## Arquivos
- `src/vfx.rs` — partículas (faísca, brilho, poeira, estilhaço, brasa), anéis, números flutuantes, banners, câmera (punch/KO/super), tremor, linhas de velocidade, parâmetros do pós-processo.
- `src/postfx.rs` — render targets, shaders (bright-pass, blur, composição) e níveis de qualidade.
- `src/render.rs` — câmera com zoom, reflexos, sombras suaves, integração com `Vfx`.
- `src/hud.rs`, `src/ui.rs`, `src/app.rs` — HUD animado, motes/feixes de luz, wipe, menus escalonados.

## Ideias futuras
Hit-stop com *freeze-frame* no zoom, pintura de tinta/aquarela estilo SF6 como sprites de efeito, lightning/vento por cenário, replay do KO, câmera de vitória por personagem, oclusão ambiente dos sprites (normal maps), trilha dinâmica por vida/round.
