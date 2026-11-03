# Referências AAA → o que foi implementado

## Fontes pesquisadas
Achados de busca na web (os sites primários de Capcom/Epic/GDC ficam fora da rede liberada desta sessão, então os trechos abaixo vêm dos resumos das buscas, não das páginas completas):
- **Street Fighter 6 — coluna oficial "The Secrets of Drive Impact"** ([streetfighter.com](https://www.streetfighter.com/6/en-us/column/detail/effects02)) e entrevistas ([EventHubs](https://www.eventhubs.com/news/2022/jul/09/sf6-influence-hip-hop-graffiti/), [Rolling Stone](https://www.rollingstone.com/culture/rs-gaming/street-fighter-6-what-we-learned-1234746230/)): efeitos em **estilo grafite/tinta colorida** (o SF4 usava tinta preto e branco), o efeito do Drive Impact **nasce atrás do atacante** para a origem do golpe ser legível, e efeitos realistas foram evitados para não competir com os personagens nem sobrecarregar a tela → *splash de tinta atrás do atacante*, cor por jogador.
- **Tekken 8** ([Gameshub](https://www.gameshub.com/news/features/tekken-8-preview-gameplay-harada-interview-2611153/), [Gameranx](https://gameranx.com/updates/id/416856/article/tekken-8-details-revealed-with-new-rage-art-and-battle-heat-system/), [Fighters Generation](https://www.fightersgeneration.com/news1/tekken8-sep22-2.htm)): ao ativar Heat/Rage Art *a tela escurece, a câmera faz algo dramático* e o áudio ganha graves; a iluminação dinâmica dos efeitos de golpe **ilumina o cenário "como fogos de artifício"**; chuva e efeitos de clima escorrem pelos modelos; Harada comenta o "custo" de manter a legibilidade com efeitos vistosos → *luzes dinâmicas por impacto, vinheta/escurecimento no super, clima por cenário, opção de efeitos reduzidos*.
- **Guilty Gear -Strive-** ([entrevista Unreal Engine](https://www.unrealengine.com/en-US/developer-interviews/how-guilty-gear--strive--hits-an-ultra-combo-with-groundbreaking-visuals-and-gameplay), [Play UK](https://www.pressreader.com/australia/play-uk-1/20210504/285233080690459)): visual 2D/3D híbrido que imita animação desenhada à mão; **câmera se move mesmo com o jogador no controle** (equilíbrio difícil com a visibilidade); colaboração entre times 2D/efeitos/fundos → *zoom dinâmico por distância, quadros de impacto estilo anime*.
- **Guias de game feel** ([Wayline](https://www.wayline.io/learn/game-feel/1), [Mega Cat Juice Guide](https://megacatstudios.com/blogs/game-development/mega-cat-developer-juice-guide-v1-0-08-23-17), [post sobre trauma²](https://dev.to/saltmire/godot-4-screen-shake-and-hit-stop-in-one-script-11eh), [Uhiyama Lab](https://uhiyama-lab.com/en/notes/unity/unity-game-feel-hit-feedback/)): 5–8 micro-respostas em ~100 ms, hit-stop de 0,05–0,1 s, tremor trauma², *feedback separado da lógica* e opções de acessibilidade.

## Princípios → implementação

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
| **Tinta/grafite colorida atrás do atacante** (SF6) | `Splat`: mancha irregular + gotas com contorno escuro, cor do jogador (ciano P1 / magenta P2), nasce na posição do atacante em golpes pesados e supers |
| **Luz dinâmica de impacto** (Tekken 8 "fogos de artifício") | `Light`: halo aditivo grande e suave que ilumina cenário e lutadores a cada golpe, maior em super/KO |
| **Quadro de impacto anime** (Strive/anime) | inversão de cores por ~75 ms no início do super e no KO (`P4.invert` no shader de composição) |
| **Câmera viva mesmo com controle** (Strive) | zoom dinâmico: aproxima conforme os lutadores se aproximam (até +7 %), suavizado |
| **Linhas de corte anime** | `Slash`: lâminas finas radiais no ponto de impacto |
| **Clima por cenário** (Tekken 8: chuva) | chuva com respingos no chão (Neo-Metrópole), vaga-lumes (Bambuzal), espíritos (Portão), vento (Planalto) |
| **Super em destaque** | contorno dourado pulsante no lutador que ativa o super (sprite redesenhado aditivamente) |
| **Acessibilidade**: opção para reduzir flashes/tremor | Vídeo → *Efeitos de impacto*: Reduzido (40 %) · Normal · Intenso; *Pós-processamento* Desligado/Médio/Alto |

Tudo é **apenas apresentação**: a simulação determinística (60 Hz) não muda — só o ritmo em tempo real (câmera lenta) e a câmera visual.

## Arquivos
- `src/vfx.rs` — partículas (faísca, brilho, poeira, estilhaço, brasa), anéis, números flutuantes, banners, câmera (punch/KO/super), tremor, linhas de velocidade, parâmetros do pós-processo.
- `src/postfx.rs` — render targets, shaders (bright-pass, blur, composição) e níveis de qualidade.
- `src/render.rs` — câmera com zoom, reflexos, sombras suaves, integração com `Vfx`.
- `src/hud.rs`, `src/ui.rs`, `src/app.rs` — HUD animado, motes/feixes de luz, wipe, menus escalonados.

## Ideias futuras
Hit-stop com *freeze-frame* no zoom, pintura de tinta/aquarela estilo SF6 como sprites de efeito, lightning/vento por cenário, replay do KO, câmera de vitória por personagem, oclusão ambiente dos sprites (normal maps), trilha dinâmica por vida/round.
