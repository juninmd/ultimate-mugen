# Roadmap de Desenvolvimento: Ultimate MUGEN

Este documento descreve as prioridades arquiteturais, melhorias de compatibilidade e novas funcionalidades planejadas para o **Ultimate MUGEN**.

---

## 1. Netcode com Rollback (Determinismo & Multiplayer Online)

A arquitetura do motor foi concebida com um loop de simulação estritamente determinístico a 60 ticks fixos por segundo, com PRNG controlado e sem Garbage Collection, fornecendo a fundação ideal para netcode com Rollback.

- [ ] **Serialização do Estado de Partida (`MatchStateSnapshot`)**:
  - Implementar serialização compacta e rápida em memória do estado de `Match`, `Fighter`, `Projectile` e `Explod`.
  - Criar pool de estados para restore/rollback de até 10-15 frames sem alocações dinâmicas a cada tick.
- [ ] **Camada de Transporte Multiplataforma**:
  - **Web**: Conexão peer-to-peer determinística através de WebRTC DataChannels (não confiável e ordenado/não ordenado com baixa latência).
  - **Nativo (Desktop)**: Socket UDP puro ou integração com bibliotecas de rollback abertas (como GGPO/GGRS em Rust).
- [ ] **Sincronização de Entradas e Janela de Predição**:
  - Buffering de inputs locais e remotos com predição e resimulação (*rewind & replay*) instantânea em caso de descompasso.
  - Indicadores na interface para ping, jitter e contagem de frames de rollback.

---

## 2. Sistema de Replays e Tournaments

- [ ] **Gravador de Replays Determinísticos**:
  - Gravação de semente inicial (`seed`), IDs de lutadores/paletas, cenário escolhido e fluxo sequencial de entradas compactadas por bitfield.
  - Como a simulação é determinística, um arquivo de replay de vários minutos ocupará apenas alguns kilobytes.
- [ ] **Player de Replay**:
  - Reprodução em velocidade normal, acelerada (2x, 4x), pausa e avanço quadro a quadro.
  - Capacidade de assumir o controle do personagem em qualquer ponto do replay ("Takeover / Replay Take").

---

## 3. Compatibilidade Estendida M.U.G.E.N

Embora Kung Fu Man e personagens clássicos rodem com fidelidade, há funcionalidades avançadas do padrão Elecbyte e extensões do Ikemen GO a serem integradas:

- [ ] **State Controllers (sctrls) Adicionais e Raros**:
  - Suporte completo a `Zoom` / zoom dinâmico de câmera durante ataques e supers.
  - Controladores avançados de `PalFX` com curvas senoidais de interpolação e efeitos de inversão de cores.
  - `RemapPal` e troca dinâmica de paletas durante a luta.
- [ ] **Modos de Equipe (Team Modes)**:
  - Modo **Simul** (2 vs 2 simultâneo na mesma tela com suporte a 4 jogadores).
  - Modo **Tag** (troca dinâmica de lutador com assistências).
  - Modo **Turns** (estilo The King of Fighters, com vencedor continuando para o próximo oponente).
- [ ] **Compatibilidade Expandida de Áudio e Mídia**:
  - Suporte a reprodução de trilhas musicais de fundo em formato MP3/OGG para cenários (atualmente apenas sons WAV embutidos e sintetizados).
  - Suporte a vídeo de introdução e telas de vitória customizadas por personagem (`intro.storyboard` / `ending.storyboard`).

---

## 4. Ferramentas e Ecossistema UMF (Ultimate Mugen Fighter)

O formato UMF simplifica a criação de lutadores eliminando a necessidade de dezenas de ferramentas antigas (como Fighter Factory).

- [ ] **Editor Visual Baseado na Web**:
  - Editor visual em WebGL/WASM para edição de esqueletos, poses e timing de animações em tempo real no navegador.
  - Pré-visualização instantânea de hitboxes e hurtboxes geradas automaticamente.
  - Exportação e importação direta em formato `.json`.
- [ ] **Efeitos Visuais Particulares e Shaders**:
  - Criação de efeitos procedurais de partículas declarativos em JSON (raios, fogo, ondas de choque) acoplados aos membros do esqueleto.
  - Shaders customizados para auras, sombras dinâmicas e distorções de calor.

---

## 5. Novos Modos de Jogo e Experiência do Usuário

- [ ] **Modo Survival / Sobrevivência**:
  - Enfrentar ondas consecutivas de lutadores com barra de vida persistente ou recuperação parcial por round.
- [ ] **Boss Rush & Desafios de Combo (Trials)**:
  - Modo dedicado para demonstrar e treinar combos específicos com feedback em tempo real de acerto de inputs e timing.
- [ ] **Localização e Acessibilidade**:
  - Suporte a múltiplos idiomas (Português, Inglês, Espanhol, Japonês) em toda a interface e menus.
  - Mapeamento configurável de botões dentro do próprio jogo com persistência em `LocalStorage` (Web) ou arquivo de configuração local (Desktop).

---

## 6. Otimizações de Desempenho e Memória

- [ ] **Streaming e Cache Assíncrono de Sprites SFF**:
  - Carregamento assíncrono sob demanda de grupos de sprites não essenciais para reduzir o pico de memória inicial em personagens com SFFs superiores a 100 MB.
- [ ] **Aceleração por Hardware de Rasterização**:
  - Otimização do pipeline de renderização tiny-skia usando SIMD/AVX2 e conversão direta para texturas na GPU.
