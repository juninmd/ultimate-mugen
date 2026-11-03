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

---

# Rodada 7 — acessibilidade (fotossensibilidade) e anúncio de round

Fontes (resumos de busca): [Xbox Accessibility Guideline 117](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/117) (tremor e blur radial desligáveis ou numa escala 0–100 %), [Guideline 118](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/118) (flash = variação de ~10 % de luminância, ~20 % da tela; testar gatilhos fotossensíveis), WCAG 2.1 e [guia da Unity](https://unity.com/blog/video-game-accessibility-basics) (no máximo **3 flashes por segundo**); [Guilty Gear — Heaven or Hell](https://guiltygear.fandom.com/wiki/Heaven_or_Hell) (anúncio ciente da contagem de rounds: "Final" no duelo decisivo); [GameDesignSkills](https://gamedesignskills.com/game-design/combat-design/) e [TV Tropes](https://tvtropes.org/pmwiki/pmwiki.php/Main/HitSpark) (faísca de defesa suave e azul, golpe pesado maior e caótico, o VFX precisa cair exatamente no hit-stop — já era o caso).

Implementado:
- **Limite de 3 flashes grandes por segundo** (`Vfx::flash_add` / `invert_add`, espaçamento mínimo de 0,34 s, com teste) — vale em todos os níveis.
- **Efeitos de impacto → "Mínimo (fotossensibilidade)"**: desliga flashes, quadro de impacto, tremor, zoom, blur radial, aberração, partículas/clima, o flash branco no sprite atingido e o flash de tela do M.U.G.E.N. (os níveis Reduzido/Normal/Intenso continuam escalando em 40/100/130 %). Config salva em `impact_lv`.
- **Anúncio "FINAL ROUND"** no round decisivo (1×1 em melhor de 3); lutas de round único (sequência de equipe) nunca dizem "final". Coberto por teste.
- Os níveis de pós-processamento já permitem desligar bloom/aberração (Vídeo → Pós-processamento); no celular o padrão é Médio.

---

# Rodada 8 — pesquisa de *features* (netcode, controles simples, seleção) e replay

Fontes (resumos de busca; sites primários bloqueados pela rede da sessão):
- **Rollback netcode** — [Push Square](https://www.pushsquare.com/news/2022/06/rejoice-street-fighter-6-has-rollback-netcode-and-cross-play), [Gematsu](https://www.gematsu.com/2022/06/street-fighter-6-supports-rollback-netcode-cross-play), [Sportskeeda (MK1)](https://sportskeeda.com/esports/mortal-kombat-1-connectivity-guide-does-rollback-netcode), [GameBrief](https://www.gamebrief.net/glossary/rollback-netcode): rollback (GGPO, 2006) virou padrão do gênero (Strive, SF6, MK1, Tekken 8). Exige **simulação determinística em passo fixo + estado serializável**.
- **Controles simples** — [Capcom (manual SF6)](https://game.capcom.com/manual/SF6/en/ps5/page/2/3), [Red Bull](https://www.redbull.com/int-en/street-fighter-6-modern-vs-classic-controls), [EventHubs](https://www.eventhubs.com/news/2023/jul/27/sf6-director-game-classic-controls/), [DashFight (GBVS Rising)](https://dashfight.com/news/control-scheme-in-granblue-fantasy-versus-rising-4481): SF6 *Modern* faz especiais com um botão e **paga com ~20 % menos dano**; GBVS permite **misturar** inputs simples e técnicos na mesma luta, com **recarga maior** nos simples e **+10 % de dano** nos técnicos.
- **Seleção de personagem** — [ResetEra](https://www.resetera.com/threads/which-relatively-recent-1-year-old-fighting-game-has-the-best-character-select-screen.809688/): a do SF6 é elogiada pelas **animações idle** que mostram personalidade antes de escolher; grandes elencos exigem layout em camadas/rolagem → já feito (idle animado na prévia, grade com rolagem, filtro, aleatório, até 5 por equipe).

## O que isso gerou
- **Prova de determinismo** (pré-requisito de replay e rollback): teste `simulation_is_deterministic_for_replays` — mesmas entradas → mesmo estado a cada 100 ticks, 1v1 e 2v2 simultâneo, com golpes realmente trocados.
- **Replay da última luta** (novo item *Replay* na tela de resultado; indicador ● REPLAY no HUD): grava as entradas de todos os lutadores por tick e re-simula. Verificado de ponta a ponta no app (carregar → gravar → reproduzir 700 ticks → mesmo estado final: `replay-check: identical=true`). Não grava duelos da sequência de equipe.

## Roadmap priorizado (não implementado)
| Prioridade | Feature | Base / observação |
|---|---|---|
| Alta | **Rollback netcode** | motor já é determinístico e de passo fixo (provado); falta serializar/restaurar o estado do `Match` e transporte UDP/WebRTC |
| Alta | **Compensação dos controles simples** (SP1–SP3) | pesquisa: ~−20 % de dano (SF6) ou recarga maior (GBVS); hoje os especiais simples não têm custo |
| Média | **Assistências estilo Kameo** | MK1: parceiro chamado por botão, recarga de ~7 s com retrato escurecido |
| Média | **Tag / troca no combo** | MvCI: troca a qualquer momento |
| Média | **Frame meter / hitboxes no treino** | SF6 tem barra de frames; já há hitboxes (F2) e histórico de inputs |
| Baixa | **Replay salvo em arquivo / compartilhável** | as entradas já são gravadas; falta serializar com versão e hash dos pacotes |

---

# Rodada 9 — motor, jogabilidade e HUD v3 em polígonos

Fontes (resumos de busca): [Dustloop — Damage Scaling](https://www.dustloop.com/w/GBVS/Damage), [Dustloop — GGAC Damage](https://www.dustloop.com/w/GGAC/Damage), [Giant Bomb — Proration](https://giantbomb.com/wiki/Concepts/Proration): a escala de dano existe para **combos não matarem de vida cheia**, cada golpe tem a sua taxa, golpes rápidos escalam mais, e **supers mantêm um piso de dano**. Sobre o empurrão no canto (*corner push-back*), as buscas não trouxeram fonte — é um padrão do gênero implementado por conhecimento geral, não verificado nesta sessão.

## Motor / jogabilidade
- **Escala de dano em combos** (`matchs::combo_scale`): 100 % nos dois primeiros golpes, depois 90 → 80 → 70 → 60 → 50 → 40 → 30 → 25 %, piso de 20 % e dano mínimo de 1; vale para lutadores nativos (M.U.G.E.N. mantém as próprias regras). O HUD mostra "DANO xx %" do próximo golpe.
- **Empurrão no canto** (`corner_push`, `Match::at_corner`): defensor encostado na borda empurra o atacante para trás (1,2 a 4,5 px conforme o knock-back) em vez de absorver tudo.
- **Squash & stretch**: aterrissagens achatam o lutador, saltos o esticam (pés plantados, decai em ~0,3 s).
- Testes: `combo_damage_scales_down_and_never_vanishes`, `native_combos_are_prorated_and_corners_push_attackers_back`.

## "Mais polígonos"
O motor é 2D (sprites rasterizados); o ganho de geometria está nas camadas desenhadas por malha, via `src/poly.rs` (preenchimentos convexos com **cor por vértice**, chanfros, paralelogramos, hexágonos, círculos/anéis com facetas adaptativas à resolução):
- **HUD v3**: moldura metálica inclinada com bisel dourado, barra de vida em gradiente vertical + brilho + borda quente + 10 marcas de segmento, rastro de dano, retratos em **moldura octogonal**, timer **hexagonal duplo**, pips de round em **losango**, medidor de super em **três células** com **hexágono** de nível, plaquetas de nome com a cor do lutador.
- **VFX**: faíscas em **lâmina afilada** (cabeça clara, cauda que some), estilhaços **irregulares de 5 pontas** com luz/sombra, brasas em losango giratório, anéis de choque e brilhos com facetas que aumentam com a resolução.

---

# Rodada 10 — história, cinemáticas, despertar, chefe final

## O que entrou
- **Face-off no início do round 1** (`Match::intro_ticks`, estados `192`/`194`): os dois lutadores posam (anim. *intro* / *taunt*), câmera avança e **balões de fala** (HUD, polígonos) trocam `intro` (lado 1) e `taunt` (lado 2) — falas por personagem no JSON (`intro`, `taunt`, `form`), genéricas para MUGEN. Qualquer botão de ataque pula (a entrada é gravada → o replay pula igual).
- **Cutscenes** (`src/app/cutscene.rs`): abertura do Arcade, confronto antes do chefe e final — letterbox, eclipse escarlate em polígonos, corvos, retratos vivos (quem fala fica iluminado), caixa de diálogo com *typewriter*; `Esc` pula.
- **Transformação / despertar** (`T`, clique do analógico direito, `Kp0`; IA desperta com < 45 % de vida): custa 1 barra, uma vez por round, **×1,3 de dano, ×1,25 de defesa, regeneração de medidor, +10 % de vida**, pausa cinematográfica (pilar de luz, anéis, relâmpagos, brasas), chamas e halo na cor do lutador até o fim do round, etiqueta no HUD com o nome da forma. Estados `193`, anim. `powerUp`.
- **Derrotas por round**: 4 sequências (`defeat1..4`, estados `5190..5193`) — tenta levantar e cai · senta derrotado · ajoelha · mão ao céu; varia a cada round, a mais longa (com câmera lenta no derrotado e cores apagadas) no KO que decide a luta.
- **Zoom-out para equipes** (`Match::zoom`, `set_zoom_for_teams`): o campo cresce de 1× (1v1) até 1,8× (5v5) conforme a distância entre os lutadores; HUD continua no layout base. Fundos procedurais escalam junto.
- **Supers maiores**: +30 % de dano (compilador), pausa de 55 ticks, 3 anéis, tempestade radial de faíscas, tremor contínuo, letterbox, nome do golpe, som em duas camadas.
- **Chefe final** *Mikazuki Oboro* (mistura Itachi × Aizen — ilusionista calmo, olhos escarlates, corvos, chama negra, espelho partido, Susanoo espectral e hipnose absoluta): oculto na seleção, fecha a escada do Arcade, 2300 de vida, ×1,28 de ataque, animações e movimento 30 % mais rápidos, 2 barras no início + recarga passiva, IA mais agressiva (ataca de mais longe, recupera mais rápido), aura constante, banner "CHEFE FINAL", nível de IA 8 e desperta com 60 % de vida (*Mangekyo Desperto*).

## Testes
`awakening_costs_a_bar_boosts_stats_and_ends_with_the_round`, `first_round_waits_for_the_face_off_and_a_button_skips_it`, `every_ko_plays_a_defeat_sequence_and_the_deciding_one_is_the_longest`, `big_teams_zoom_out_and_everyone_stays_inside_the_field`, `final_boss_is_complete_and_formidable`. Tour de capturas: `UME_TOUR=story ./target/release/ultimate-mugen --shots <dir>`.

