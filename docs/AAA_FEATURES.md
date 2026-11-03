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

---

# Rodada 3 — modos de equipe, áudio e HUD

## Referências (vídeos)
Não consigo assistir vídeos nesta sessão; busquei descrições e artigos que descrevem os vídeos/gameplay. Padrões extraídos:
- **King of Fighters XV** ([Time Extension](https://timeextension.com/games/ps5/king_of_fighters_xv), [Xbox Addict](https://xboxaddict.com/Staff-Review/Xbox-One/14638/KING-OF-FIGHTERS-XV,-THE.html)): 3v3 em que **a ordem da equipe é escolhida antes da luta**, cada lutador tem a própria barra de vida e **a equipe compartilha uma barra de super**; o time recupera um pouco de vida ao vencer → *Equipe · Sequência*: ordem de seleção = ordem de entrada, vida do vencedor carregada (+12 %), medidor de super compartilhado, roster com retratos (atual destacado, KO riscado).
- **Mortal Kombat 1 — Kameos** ([SI](https://www.si.com/esports/fgc/everything-we-know-about-kameo-fighters-in-mortal-kombat-1), [SRK wiki](https://srk.shib.live/w/Mortal_Kombat_1/Kameo)): parceiro simultâneo que você controla *junto* do lutador principal, com HUD de medidor único e retrato escurecido em recarga → *Equipe · Paralelo*: o parceiro luta ao mesmo tempo (IA), barra compacta sob a do líder; ao cair fica no chão e deixa de ser alvo.
- **Marvel vs. Capcom: Infinite** ([Event Hubs](https://www.eventhubs.com/news/2017/apr/27/four-button-layout-tagging-anytime-counter-switching-while-getting-hit-and-more-heres-breakdown-marvel-vs-capcom-infinites-mechanics)): equipes simultâneas com barras por membro; o round termina quando a **equipe inteira** cai → regra do modo paralelo.

## Modos de equipe
| Modo | Regras |
|---|---|
| **Equipe · Sequência (3v3)** | monte 3 lutadores (ordem = ordem de entrada); o adversário é sorteado. Um de cada vez, um round por duelo; o vencedor segue com a vida restante (+12 %) e o medidor de super; o derrotado é substituído. Vence quem derrubar toda a equipe. |
| **Equipe · Paralelo (2v2)** | 2 de cada lado ao mesmo tempo; o parceiro é controlado pela IA. Quem cai fica no chão; o inimigo vivo mais próximo é o alvo; o round acaba quando uma equipe inteira cai (tempo esgotado: soma das vidas). |

Motor: `Match::team_simul`, jogadores adicionados intercalados (P1, P2, P3, P4) para `roots()[0..2]` serem sempre os líderes; companheiros se alinham atrás do líder; `can_be_hit` ignora quem já caiu; `enemy()` prioriza vivos. Cobertos por testes (`simul_team_round_ends_only_when_whole_team_is_down`, `single_fights_still_end_on_first_ko`).

## Áudio
- Síntese em camadas (transiente de ruído + corpo com queda de pitch + "crunch" em banda + cauda), filtro de estado variável (passa-baixa/banda/alta), reverb Schroeder de 3 combs, *soft clip* tanh.
- **3 variações por golpe/defesa/whoosh** em rodízio (evita a fadiga de sons idênticos); 32 kHz.
- Novos sons: counter, whooshes (leve/pesado), dash, pulo, aterrissagem, riser + boom do super, KO com cauda, stingers de ROUND / FIGHT / vitória / PERFECT, ping de medidor, **pings de combo com altura crescente (combo1…6)**, UI (tick, voltar, confirmar).
- **Ducking da música** (−72 %) durante o super e a câmera lenta do KO.
- Não pude auditionar os sons (sem saída de áudio no ambiente): validados por testes de energia, finitude e limites.

## HUD
- Retratos reais (recorte da cabeça do sprite idle; retrato 9000,x em M.U.G.E.N.) com moldura que fica vermelha em vida baixa.
- Barras compactas do parceiro, roster de equipe (sequência), alarme de vida baixa, medidor "MAX", timer pulsante, combo com *pop*.

## Ideias futuras
Assistências estilo Kameo (botão de chamar parceiro com recarga), troca de personagem (tag) no meio do combo, seleção da ordem da equipe por arrastar, modo espectador de torneio.

---

# Rodada 4 — equipes de até 5v5 e HUD v2

- **Equipe · Paralelo** agora abre em **5v5** (2 a 5 com Q/E na seleção): 10 lutadores na tela, formação compactada para todos começarem dentro do cenário, câmera e zoom seguem todos os vivos, VFX por lutador (poeira, counter, aura de super de qualquer um). Simulação de 600 ticks com 10 lutadores em ~40 ms.
- **Troca de controle**: `TAB` passa o controle ao próximo companheiro vivo; se o seu cair, o controle passa sozinho. A IA que dirigia o lutador é reiniciada na troca.
- **HUD v2**: barra principal acompanha o lutador que você controla (lado inimigo: o da linha de frente); *tiles* de retrato com barra de vida para cada companheiro (o seu em dourado, caídos escurecidos com ✕); contadores vivos/total ao lado do timer; dica `TAB · TROCAR DE LUTADOR`.
- **Na arena** (mais de 2 lutadores): barra de vida flutuante sobre cada lutador e marcador ▼ saltitante sobre o seu.
- Fontes com filtro linear (texto escalado deixou de ficar serrilhado) e *slam* de banner mais suave.

---

# Rodada 5 — pesquisa complementar (câmera de super e hit-stop)

Fontes (resumos de busca; as páginas primárias de Capcom/Epic/GDC não resolvem na rede da sessão):
- **Dragon Ball FighterZ** — [Famitsu via DBZGames](https://www.dbzgames.org/news/dragon-ball-fighterz-arcsys-director-and-game-producer-talk-about-technical-aspects/), [PC Gamer](https://pcgamer.com/uk/dragon-ball-fighterz-review), [ComicBook.com](https://comicbook.com/gaming/2018/02/12/dragon-ball-fighterz-video-without-cinematic-effects), [Kotaku sobre a palestra GDC](https://kotaku.com/breaking-down-the-animation-of-guilty-gear-and-dragonba-1836849279): o diretor Motomura explica que a **câmera se move durante certos supers** para o 3D "ficar bonito"; os supers têm **build-up e follow-through** e a *transição* entre a câmera de jogo e o close é o que vende o efeito (um vídeo sem os cinematics mostrou isso). → **câmera de super cinematográfica**: zoom que entra devagar até o lutador que ativa (até +20 %), segura durante a pausa e sai ainda mais devagar (`Vfx::sz`, ease-in 5/s, ease-out 2,2/s).
- **Hit-stop** — [Sonic Hurricane](https://sonichurricane.com/?p=1043), [GameFAQs (SF V)](https://gamefaqs.gamespot.com/boards/895976-street-fighter-v/74812570?page=2), [Shoryuken](http://shoryuken.com/2016/06/07/hitstop-in-street-fighter-v-kens-not-so-little-secret/), [SmashWiki](https://www.ssbwiki.com/Hitlag): SF V trava **8/12/15 frames** (leve/médio/pesado); média entre jogos **≈ 9/11/13**; no Smash o hitlag cresce com o dano. O motor já usa **9/11/13** nos golpes nativos — agora travado por teste (`hitstop_scales_with_attack_strength`).
- **Game feel**: manter feedback separado da lógica e oferecer opção de reduzir efeitos (já em *Vídeo → Efeitos de impacto*).

Correção visual desta rodada: o glifo "✕" não existe na fonte de exibição; caídos agora são marcados com uma cruz desenhada.

---

# Rodada 6 — pesquisa de HUD, som e modo treino

Fontes (resumos de busca; sites primários bloqueados pela política de rede da sessão):
- **HUD do SF6** — [SuperCombo Wiki](https://wiki.supercombo.gg/w/Street_Fighter_6/HUD), [manual oficial](https://game.capcom.com/manual/SF6/en/ps5/page/3/3), [guia de HUD](https://www.argentics.io/game-hud-design): avisos pop-up para **first attack, counter hit**, throw escape e reversal; estados sinalizados por **cor** (o Drive Gauge fica cinza vazio); a leitura do HUD precisa **sobreviver a blur, tremor, pós-processamento e escalas menores** → nosso HUD é desenhado *depois* do pós-processo (nunca recebe blur/aberração) e ganhou o banner **FIRST ATTACK** (uma vez por round, fora do treino).
- **Modo treino moderno** — [EventHubs](https://www.eventhubs.com/news/2022/sep/16/sf6-training-visual-frame-data/), [Fighters Generation](https://www.fightersgeneration.com/news1/sf6-sep22-6.htm): **histórico de inputs**, dados do ataque (dano do combo), HUD configurável → no **Treino**: histórico dos últimos 14 inputs (seta de direção, botões, frames segurados) e linha "ÚLTIMO COMBO · hits · dano".
- **Design de som** — [guia Pixflow](https://pixflow.net/blog/punch-impact-sound-effects-for-fight-scenes/), [SoundBridge](https://www.soundbridge.io/game-audio-soundbridge-fighting-game-scene): soco = **transiente curto + corpo grave + (voz)**, 2–4 camadas **separadas por faixa de frequência**, variações e **ducking da música** nos impactos → já implementado (transiente em passa-alta, corpo em queda de pitch, crunch em banda, 3 variações em rodízio, ducking em super/KO).
- Tekken 8: nada confiável sobre HUD/treino nas buscas; tratado como não verificado.
