# Referências de UI nas engines Rust

Estudo das onze engines Rust listadas em "Best Rust Game Engine for Game Development" (WeeTech Solution, 2026) e o que vale aproveitar em menus, telas de loading, HUD, configurações e botões. Nada é copiado para este repositório. Só ideias e padrões de layout são registrados aqui.

Restrições de stack que valem para todas as recomendações: renderer wgpu + winit, tiny-skia, UI própria em modo imediato em `src/ui.rs` e `src/app/menu_view.rs`, tokens de layout em `src/ui_tokens.rs`. Sem macroquad, sem egui, sem Lua.

## Método e limites

- O código foi lido apenas em cinco repositórios, clonados em modo raso com acesso de leitura da sessão: `not-fl3/macroquad` (MIT OR Apache-2.0), `ggez/ggez` (MIT), `FyroxEngine/Fyrox` (MIT), `bevyengine/bevy` (MIT OR Apache-2.0) e `AmbientRun/ambient` (MIT OR Apache-2.0). As licenças foram confirmadas nos arquivos do repositório.
- As outras seis (Amethyst, Piston, Nannou, Blue Engine, Tetra, rend3) não foram abertas. As notas abaixo vêm só do artigo e estão marcadas como não verificadas.
- O artigo é texto de marketing. Suas descrições são genéricas e algumas afirmações (por exemplo, quais engines estão ativamente mantidas) não foram checadas.

## Engine por engine

| Engine | Licença | Relevância para UI | O que podemos usar |
|---|---|---|---|
| Macroquad | MIT OR Apache-2.0 | Alta. UI em modo imediato em `src/ui/` (widgets, `style.rs`, `key_repeat.rs`, `combobox`, `clipboard.rs`, `input_handler.rs`) | O modelo mais próximo da nossa UI. Repetição de tecla para setas seguradas nos menus, uma struct de estilo para cores de botões e painéis, comportamento de combobox para configurações. Pela regra de stack, não adotamos a crate. |
| Fyrox | MIT | Alta. `fyrox-ui` é um conjunto completo de widgets em modo retido: botões, dropdown list, dropdown menu, tab control, scroll viewer, progress bar, message box, popup, file browser, navegação e estilo | Referência do conjunto de widgets que uma tela de configurações precisa. `navigation.rs` para teclado e gamepad, `progress_bar` para loading, `messagebox` para diálogos de confirmação e um módulo de estilo para temas. Como o modo retido com troca de mensagens não combina com nosso código imediato, ficam só as ideias. |
| Bevy | MIT OR Apache-2.0 | Média. `bevy_ui` (flexbox), `bevy_ui_render`, `bevy_ui_widgets` (widgets sem aparência, comportamento separado do estilo) | Vale copiar conceitualmente a separação entre comportamento do widget e estilo visual. Flexbox não é necessário; nosso layout virtual de 1280x720 com tokens basta. O fluxo de telas por estados sugere como estruturar loading e transições. |
| Ambient | MIT OR Apache-2.0 | Baixa a média. Árvore de elementos de UI (`element`, `element_component`) sobre o próprio runtime | Só a ideia de compor elementos pequenos e reutilizáveis (botão, painel, rótulo) com propriedades. O runtime é grande demais para aproveitar. |
| GGEZ | MIT | Baixa. Sem camada de UI; tem caminho de texto e glifos e timers | Referência de loop mínimo e de renderização de texto com cache de glifos. Nada a copiar para o visual da UI. |
| Tetra | não verificada | Baixa (só pelo artigo) | Nada além do que já temos. |
| Amethyst | não verificada | Baixa (só pelo artigo). Vale checar o estado do projeto antes de qualquer referência | Ignorar. |
| Piston | não verificada | Baixa (só pelo artigo) | Ignorar. |
| Nannou | não verificada | Baixa (só pelo artigo; é code art criativo, não UI de jogo) | Ignorar. |
| Blue Engine | não verificada | Baixa (só pelo artigo) | Ignorar. |
| rend3 | não verificada | Baixa (só renderer, sem UI) | Ignorar. As ideias de pós-processamento já estão cobertas pelo nosso `postfx`. |

## Recomendações priorizadas

Ordenadas pelo impacto visível para o jogador e pelo custo de construir sobre o código de UI atual.

1. **Tela de loading com barra de progresso e dicas.** O `progress_bar` do Fyrox e o painel simples do Macroquad mostram o padrão. Já existe `src/loading_tips.rs`; falta uma barra determinística ligada às etapas reais de carregamento (o carregamento paralelo já existe, veja [Performance](Performance.md)). Resultado esperado: o loading deixa de parecer travado.
2. **Navegação por foco com teclado e gamepad, com repetição de tecla.** `key_repeat.rs` do Macroquad e `navigation.rs` do Fyrox. Segurar cima e baixo nos menus deve andar suavemente, e o foco precisa estar visível em todas as telas.
3. **Widgets de configuração: dropdown, abas e scroll.** `dropdown_list`, `tab_control`, `scroll_viewer` do Fyrox e o combobox do Macroquad definem o comportamento esperado (abrir, fechar ao clicar fora, seleção pelo teclado). Colocar em arquivos novos em `src/ui/` ou `src/app/`, usando `ui_tokens`.
4. **Diálogos de confirmação e tooltips.** `messagebox` e `popup` do Fyrox para "sair da partida?", "apagar replay?" e similares. Deixa ações destrutivas explícitas.
5. **Struct de tema para cores e tamanhos.** `style.rs` do Macroquad e o módulo `style` do Fyrox. Já temos 11 temas em `menu_art.rs`; reunir os valores numa struct `Theme` com tokens faz de um novo tema uma mudança de dados.
6. **Transições entre telas como máquina de estados pequena.** O padrão de estados do Bevy. Fade ou slide entre título, seleção, luta e resultados, com easing igual ao da animação atual dos itens do menu (ease-out cúbico, 0,28 s).
7. **Comportamento de widget sem aparência.** A ideia do `bevy_ui_widgets`: separar teste de clique e lógica de foco do desenho, para a mesma lógica de botão servir menu, pausa e configurações.

## Situação frente ao código atual

As recomendações acima foram conferidas com o código em 2026-11-10. A maior parte da lista já existe, então o trabalho é manter isso e preencher as lacunas reais.

| Item | Situação |
|---|---|
| Loading com barra de progresso e dicas | Já existe: `Ui::loading_screen`, `progress_bar`, `src/loading_tips.rs` e rótulos reais de etapa em `src/app/loading.rs`. Manter. |
| Foco suave no menu | Já existe: `src/app/menu_focus.rs` (easing independente da taxa de quadros). Manter. |
| Diálogos de confirmação | Existe para apagar replay em `replay_browser.rs`; a pausa tem skin própria. Outras ações destrutivas podem reusar o padrão. |
| Transições entre telas | Já existe: wipe diagonal em `src/app.rs`. Manter. |
| Repetição de tecla ao segurar uma direção | Faltava: o shell descarta as repetições do sistema de propósito, então menus e sliders andavam um passo por toque. Adicionado em `src/controls/nav_repeat.rs` (atraso de 0,38 s, depois a cada 75 ms, só direções). |
| Dropdown, abas, tooltips, struct de tema | Não implementados. Candidatos a PRs pequenos depois. |

## O que não vamos fazer

- Adotar macroquad, egui ou o runtime de qualquer engine. Isso quebra a decisão de stack em [Engine](Engine.md).
- Copiar código de fontes MIT ou Apache-2.0 para o repositório. Usamos como referência de design, então não há aviso de licença a incluir.
- Mexer em sprites de personagens ou stages. Isso é tratado em outro lugar.

## Relacionados

- [Engine](Engine.md) para o stack de renderização e UI.
- [Visual Effects](Visual-Effects.md) para o pipeline de efeitos.
- Versão em inglês: [Rust-Engine-UI-References](../Rust-Engine-UI-References.md).
