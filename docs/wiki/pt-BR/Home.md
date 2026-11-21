# Wiki do Ultimate MUGEN

[English](../Home.md) · **Português (BR)**

Guia para jogadores e criadores que querem customizar a engine: instalar, configurar, criar lutadores, cenários e scripts.

| Página | Para quem | O que ensina |
|---|---|---|
| [Instalação e execução](Instalacao.md) | Jogadores | Baixar, compilar, rodar no PC, navegador e celular |
| [Atualização automática](Atualizacao-Automatica.md) | Jogadores | Auto-atualização pelas releases do GitHub, como desligar e flags |
| [Configuração e controles](Configuracao.md) | Jogadores | Arquivo `.cfg`, flags de linha de comando, variáveis de ambiente, controles |
| [Adicionar conteúdo M.U.G.E.N](Conteudo-MUGEN.md) | Jogadores | Instalar personagens e cenários `.def`/`.zip` e validar |
| [Mercado GitHub](Mercado.md) | Jogadores | Instalar personagens, cenários e músicas de qualquer repositório do GitHub |
| [Seleção de personagem](Character-Select.md) | Jogadores | Tela de seleção, retratos, equipes e atalhos |
| [Modos de jogo](Modos-de-Jogo.md) | Jogadores | Arcade, equipes (Sequência, Paralelo, Tag) e suas regras |
| [Objetos de cenário](Objetos-de-Cenario.md) | Jogadores, criadores | Caixotes, barris e carros quebráveis, comida, fases bônus e seus dados em TOML |
| [Universo e Lore](Lore.md) | Jogadores e Criadores | Cosmologia, facções, sistemas de energia e a Grande Convergência |
| [Dossiê dos Personagens](Personagens.md) | Jogadores e Criadores | Fichas biográficas, retratos, golpes, rivalidades e cenários associados |
| [Replays](Replays-Rede.md) | Jogadores | Lutas gravadas, navegador de replays, velocidade de reprodução |
| [Decisões de compatibilidade](Decisoes-de-Compatibilidade.md) | Criadores | Recursos rejeitados por decisão e o motivo |
| [Scripts Lua](Scripts-Lua.md) | Criadores | Para que serve Lua em pacotes M.U.G.E.N e as alternativas sem Lua |
| [Arquivos de rota](Arquivos-de-Rota.md) | Criadores | Rotas de arcade e regras de desbloqueio em TOML |
| [Criar um personagem (guia rápido)](Criar-um-Personagem.md) | Criadores | Um lutador funcionando em dez linhas, herdado de outro |
| [Criar personagem UMF](Personagem-UMF.md) | Criadores | Referência completa: todos os campos de um lutador em um único JSON |
| [Passivos](Passivos.md) | Criadores | Fúria, acúmulos de habilidade e regeneração por dados |
| [Potencial Liberado](Potencial-Liberado.md) | Jogadores | Transformação genérica para personagens importados: Potencial, depois Fúria |
| [Câmera de Replay](Camera-de-Replay.md) | Jogadores | Enquadramento cinematográfico do replay: aproxima em lutas próximas e acompanha o ataque |
| [Mutadores](Mutadores.md) | Criadores | Mudanças de regra por partida; gravidade é o primeiro mutador |
| [Rush Cancel](Rush-Cancel.md) | Jogadores e Criadores | Gaste energia para cancelar um golpe conectado, Overdrive e a apresentação |
| [Níveis de Combo](Niveis-de-Combo.md) | Jogadores e Criadores | Selos de combo, marca de energia, air dashes e rebotes |
| [Kits inspirados em Bleach](Kits-Inspirados-em-Bleach.md) | Jogadores | Ideias de Byakuya, Yoruichi e Zaraki em tres nativos |
| [Quadros de efeito](Efeitos.md) | Criadores | Efeitos de projétil e faíscas |
| [Scripts CNS](Scripts-CNS.md) | Criadores | Estados, controllers, triggers e comandos suportados |
| [Flags do AssertSpecial](Flags-AssertSpecial.md) | Criadores | As 77 flags do Ikemen e o efeito de cada uma |
| [Vida vermelha, tontura e defesa](Sistemas-de-Vida.md) | Criadores | RedLife, DizzyPoints, GuardPoints, estados 5300/5400 e padrões do HitDef |
| [Screenpacks](Screenpacks.md) | Criadores | Suporte a screenpacks de menu e HUD |
| [Matriz de formatos](Matriz-Formatos.md) | Criadores | Status de SFF, SND, AIR, ACT, DEF e screenpacks |
| [Estudo de screenpacks](Estudo-Screenpacks.md) | Criadores | Diferenciais de screenpacks e o que o motor adotou |
| [Dissecação de quatro lutadores](Dissecacao-MUGEN.md) | Desenvolvedores | Aizen, Gogeta, Goku SSJ e SSJ4: arquivos, comandos, normais e padrões de golpe |
| [Recriar um lutador M.U.G.E.N](Recriar-um-Lutador-MUGEN.md) | Criadores | Receita e mapeamento para UMF, com a reconstrução do Captain Marlow Reyes |
| [Importar sprite atlas](Importacao-Sprite-Atlas.md) | Criadores | Transformar uma folha de sprites em lutador |
| [Ferramentas de desenvolvimento](Ferramentas-de-Desenvolvimento.md) | Contribuidores | Ferramentas Rust de build e auditoria (`ume-tools`) |
| [Ferramentas de sprites](Ferramentas-de-Sprites.md) | Criadores | Fatiar folhas, variantes de paleta, caixas de colisão automáticas |
| [Gerador de golpes](Gerador-de-Golpes.md) | Criadores | TOML para AIR, CNS, CMD e Statedef -1 |
| [Criar um cenário (guia rápido)](Criar-um-Cenario.md) | Criadores | Cenário a partir de um `.def`, dados do modo história e o que ainda precisa de código |
| [Criar cenários](Cenarios.md) | Criadores | Referência completa: cenário M.U.G.E.N `.def` e cenário nativo |
| [Arquitetura](Arquitetura.md) | Desenvolvedores | Mapa de crates, direção das dependências, onde o código novo entra |
| [Contribuindo](https://github.com/juninmd/ultimate-mugen/blob/master/CONTRIBUTING.pt-BR.md) | Contribuidores | Compilar, verificações, regras e fluxo de pull request |
| [Pontos de extensão](Pontos-de-Extensao.md) | Criadores, Contribuidores | Qual arquivo define cada tipo de conteúdo e se precisa de Rust |
| [Decisões de arquitetura](../adr/README.md) | Contribuidores | ADRs: por que o engine é feito assim |
| [Customizar a engine](Engine.md) | Desenvolvedores | Arquitetura, novo controller, novo trigger, testes |
| [Smoke da partida web](Smoke-Partida-Web.md) | Desenvolvedores | Teste de CI que joga partidas completas no navegador e pega travamentos |
| [Layout de arquivos](Layout-de-Arquivos.md) | Desenvolvedores | A regra de 250 linhas, uma responsabilidade por arquivo, dividir arquivos, PRs paralelos |
| [Desempenho](Desempenho.md) | Desenvolvedores | Carga paralela, ritmo de quadros, benchmarks |
| [Engenharia reversa do port para navegador](Engenharia-Reversa-Ikemen-Web.md) | Desenvolvedores | Como uma build de M.U.G.E.N com 78 lutadores roda no navegador, e o que copiamos dela |
| [Pesquisa de jogos de luta](Pesquisa-Jogos-Luta.md) | Desenvolvedores | Pesquisa de outros lutadores e lista priorizada de melhorias |
| [Auditoria de animações nativas](Auditoria-Animacoes-Nativas.md) | Desenvolvedores | Cobertura de poses do elenco nativo |
| [Smoke test (modo debug)](Smoke-Test.md) | Desenvolvedores | Um comando valida as features principais no desktop e na web, com prints e relatório |
| [Estresse e resiliência](Estresse-e-Resiliencia.md) | Desenvolvedores | Fuzzing, limites de spawners e números do caminho quente |
| [Solução de problemas](Problemas.md) | Todos | Erros comuns e ferramentas de diagnóstico |

## Formatos aceitos

| Formato | Uso | Arquivos |
|---|---|---|
| UMF | Lutador nativo, declarativo | `assets/chars/<id>.json` |
| M.U.G.E.N | Lutadores e cenários clássicos | `.def`, `.sff` v1/v2, `.air`, `.cmd`, `.cns`, `.snd`, `.act` |

Lutadores UMF e M.U.G.E.N rodam na mesma máquina de estados e podem se enfrentar.

## Roadmap

O plano de compatibilidade e as decisões de arquitetura estão em [`docs/ROADMAP.pt-BR.md`](../../ROADMAP.pt-BR.md). Itens marcados como **planejado** nesta wiki ainda não existem na versão atual.

## Status de compatibilidade

O progresso rumo à paridade com o Ikemen GO v1.0.0 é acompanhado na [issue #135](https://github.com/juninmd/ultimate-mugen/issues/135), com uma issue para cada controlador, trigger ou recurso faltante. As listas de controladores, triggers e redirecionamentos suportados em [Scripts CNS](Scripts-CNS.md) são conferidas com a engine por `cargo test -p ume-engine --test wiki_sync`: um nome listado que a engine não trata, um nome implementado que não está listado ou uma diferença entre as páginas em inglês e português quebra o teste. Cada carregamento de personagem também aponta os controladores, triggers e parâmetros usados que a engine ignora (veja "Diagnóstico no carregamento" na mesma página).

## Agradecimentos

Obrigado à equipe do [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) e à Elecbyte, criadora do M.U.G.E.N, pela inspiração.
