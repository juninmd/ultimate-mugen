# Wiki do Ultimate MUGEN

Guia para jogadores e criadores que querem customizar a engine: instalar, configurar, criar lutadores, cenários e scripts.

| Página | Para quem | O que ensina |
|---|---|---|
| [Instalação e execução](Instalacao.md) | Jogadores | Baixar, compilar, rodar no PC, navegador e celular |
| [Configuração e controles](Configuracao.md) | Jogadores | Arquivo `.cfg`, flags de linha de comando, variáveis de ambiente, controles |
| [Adicionar conteúdo M.U.G.E.N](Conteudo-MUGEN.md) | Jogadores | Instalar personagens e cenários `.def`/`.zip` e validar |
| [Criar personagem UMF](Personagem-UMF.md) | Criadores | Lutador completo em um único JSON |
| [Arte do esqueleto (rig)](Rig.md) | Criadores | Estilo, poses, articulações e paletas |
| [Scripts CNS](Scripts-CNS.md) | Criadores | Estados, controllers, triggers e comandos suportados |
| [Criar cenários](Cenarios.md) | Criadores | Cenário M.U.G.E.N `.def` e cenário nativo |
| [Customizar a engine](Engine.md) | Desenvolvedores | Arquitetura, novo controller, novo trigger, testes |
| [Solução de problemas](Problemas.md) | Todos | Erros comuns e ferramentas de diagnóstico |

## Formatos aceitos

| Formato | Uso | Arquivos |
|---|---|---|
| UMF | Lutador nativo, declarativo | `assets/chars/<id>.json` |
| M.U.G.E.N | Lutadores e cenários clássicos | `.def`, `.sff` v1/v2, `.air`, `.cmd`, `.cns`, `.snd`, `.act` |

Lutadores UMF e M.U.G.E.N rodam na mesma máquina de estados e podem se enfrentar.

## Roadmap

O plano de compatibilidade com o Ikemen-GO e as decisões de arquitetura estão em [`docs/ROADMAP.md`](../ROADMAP.md). Itens marcados como **planejado** nesta wiki ainda não existem na versão atual.
