# Wiki do Ultimate MUGEN

[English](../Home.md) · **Português (BR)**

Guia para jogadores e criadores que querem customizar a engine: instalar, configurar, criar lutadores, cenários e scripts.

| Página | Para quem | O que ensina |
|---|---|---|
| [Instalação e execução](Instalacao.md) | Jogadores | Baixar, compilar, rodar no PC, navegador e celular |
| [Configuração e controles](Configuracao.md) | Jogadores | Arquivo `.cfg`, flags de linha de comando, variáveis de ambiente, controles |
| [Adicionar conteúdo M.U.G.E.N](Conteudo-MUGEN.md) | Jogadores | Instalar personagens e cenários `.def`/`.zip` e validar |
| [Mercado GitHub](Mercado.md) | Jogadores | Instalar personagens, cenários e músicas de qualquer repositório do GitHub |
| [Criar personagem UMF](Personagem-UMF.md) | Criadores | Lutador completo em um único JSON |
| [Arte do esqueleto (rig)](Rig.md) | Criadores | Estilo, poses, articulações e paletas |
| [Scripts CNS](Scripts-CNS.md) | Criadores | Estados, controllers, triggers e comandos suportados |
| [Flags do AssertSpecial](Flags-AssertSpecial.md) | Criadores | As 77 flags do Ikemen e o efeito de cada uma |
| [Vida vermelha, tontura e defesa](Sistemas-de-Vida.md) | Criadores | RedLife, DizzyPoints, GuardPoints, estados 5300/5310 e padrões do HitDef |
| [Criar cenários](Cenarios.md) | Criadores | Cenário M.U.G.E.N `.def` e cenário nativo |
| [Customizar a engine](Engine.md) | Desenvolvedores | Arquitetura, novo controller, novo trigger, testes |
| [Desempenho](Desempenho.md) | Desenvolvedores | Carga paralela, ritmo de quadros, benchmarks |
| [Solução de problemas](Problemas.md) | Todos | Erros comuns e ferramentas de diagnóstico |

## Formatos aceitos

| Formato | Uso | Arquivos |
|---|---|---|
| UMF | Lutador nativo, declarativo | `assets/chars/<id>.json` |
| M.U.G.E.N | Lutadores e cenários clássicos | `.def`, `.sff` v1/v2, `.air`, `.cmd`, `.cns`, `.snd`, `.act` |

Lutadores UMF e M.U.G.E.N rodam na mesma máquina de estados e podem se enfrentar.

## Roadmap

O plano de compatibilidade e as decisões de arquitetura estão em [`docs/ROADMAP.pt-BR.md`](../../ROADMAP.pt-BR.md). Itens marcados como **planejado** nesta wiki ainda não existem na versão atual.

## Agradecimentos

Obrigado à equipe do [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) e à Elecbyte, criadora do M.U.G.E.N, pela inspiração.
