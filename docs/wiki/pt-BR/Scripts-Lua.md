# Scripts Lua em conteúdo M.U.G.E.N

Alguns pacotes trazem arquivos `.lua`. Esta página diz para que servem, o que a engine faz ao encontrá-los e o que usar no lugar. A engine não tem runtime Lua por decisão (veja [Engine](Engine.md) e [Decisões de compatibilidade](Decisoes-de-Compatibilidade.md)): binário menor, WASM simples e simulação determinística para rollback.

## O que foi verificado

Nenhum dos personagens guardados no projeto (os dois pacotes Goku GT DBFZ, Gogeta SSJ4, Goku SSJ4) contém `.lua`; são CNS, CMD, AIR, SFF e SND comuns. As notas abaixo sobre Lua em outros pacotes vêm da documentação do Ikemen GO, não de um pacote deste repositório. Se um lutador específico trouxer Lua, envie o pacote para os scripts serem lidos e documentados aqui.

## Onde o Lua aparece

M.U.G.E.N 1.0 e 1.1 não têm Lua. Ele vem do Ikemen GO, que o usa em três lugares.

| Lugar | Função | Arquivos típicos |
|---|---|---|
| Camada de engine e screenpack | Menus, modos de jogo, seleção, tela de vitória, hooks | `main.lua`, `start.lua`, arquivos de motif e módulos |
| Módulos externos | Extras opcionais: modos, estatísticas, ferramentas de treino | `external/mods/*.lua` |
| Pacotes de conteúdo | Raro. Lutador ou screenpack com módulo que adiciona menu, modo ou ajuste de HUD | `*.lua` ao lado do `.def` |

A lógica do lutador não é escrita em Lua. Ela fica em CNS e ZSS (controllers, triggers, comandos, IA), que a engine já executa. Um `.lua` dentro da pasta de um lutador quase nunca muda como ele luta; ele estende menus, modos ou telas ao redor.

## O que a engine faz hoje

- O carregador de personagens não lê `.lua` e não o lista como erro. O lutador carrega e luta normalmente.
- Se o lutador dependia do Lua (entrada de menu, modo, ajuste de HUD), esse comportamento extra simplesmente não existe.
- O `mugen_check` reporta controllers, triggers e parâmetros não suportados, mas não arquivos `.lua` soltos.

## Alternativas sem Lua

| Uso do Lua | Substituto |
|---|---|
| Menus e telas | Screenpack (`system.def`, `fight.def`) e o código de menus em Rust, veja [Screenpacks](Screenpacks.md) |
| Modos de jogo | Modos em Rust e dados, veja [Modos de jogo](Modos-de-Jogo.md) |
| Comportamento e IA do lutador | CNS ou ZSS, ou UMF JSON, veja [Scripts CNS](Scripts-CNS.md) e [Criar personagem UMF](Personagem-UMF.md) |
| Regras e ajustes | Campos de dados (TOML ou JSON) lidos pelo Rust |

## Recomendação

1. Manter Lua sem suporte. Não adicionar runtime.
2. Fazer o importador detectar `.lua` e avisar uma vez por personagem: "ignorado: scripts Lua não são suportados", para o criador saber por que falta um menu ou modo.
3. Incluir a mesma linha no relatório do `mugen_check`.
4. Quando um pacote real depender do Lua, ler o script e portar o efeito para Rust ou dados caso a caso, sem interpretar Lua arbitrário.
