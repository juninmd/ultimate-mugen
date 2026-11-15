# Pontos de extensão

[English](../Extension-Points.md) · **Português (BR)**

A maior parte do conteúdo é dado. Esta página liga cada tipo de conteúdo ao arquivo que o define, à página que o documenta e diz se precisa de mudança em Rust. Adicione conteúdo novo como dado primeiro e use código só quando o dado não alcançar.

| Conteúdo | Onde fica | Documentado em | Mudança em Rust |
|---|---|---|---|
| Lutador nativo | `assets/chars/<id>.json`, listado em `assets/chars/index.json` | [Personagem UMF](Personagem-UMF.md), [Criar um personagem](Criar-um-Personagem.md) | Não |
| Bios e nacionalidade | `assets/char_profiles.json` | [Personagens](Personagens.md) | Não |
| Lutador ou cenário M.U.G.E.N | `mugen/chars/`, `mugen/stages/` ou o mercado | [Conteúdo M.U.G.E.N](Conteudo-MUGEN.md), [Mercado](Mercado.md) | Não |
| Cenário embutido por dados | `assets/stages/data/<id>/stage.def` e seus arquivos, embutidos pelo `build.rs` | [Criar um cenário](Criar-um-Cenario.md) | Não |
| Aparência do cenário (luz de contorno, tom, vinheta) | `assets/stagelook.toml` | [Efeitos visuais](Efeitos-Visuais.md) | Não |
| Adereços de cenário | `assets/stageprops.toml` e `assets/props/` | [Efeitos visuais](Efeitos-Visuais.md) | Não |
| Objetos de história (caixotes, comida, fases bônus) | `assets/story/objects.toml` | [Objetos de cenário](Objetos-de-Cenario.md) | Não |
| Finais da história | `assets/story/epilogues.json` | [Modos de jogo](Modos-de-Jogo.md) | Não |
| Rotas de arcade e desbloqueios | `assets/routes/unlocks.toml` | [Arquivos de rota](Arquivos-de-Rota.md) | Não |
| Pós-processamento | `assets/postfx.toml` | [Efeitos visuais](Efeitos-Visuais.md) | Não |
| Passivos, medidores de assinatura | campos de um lutador UMF | [Passivos](Passivos.md) | Não |
| Controller ou trigger | `crates/engine` | [Customizar a engine](Engine.md), [Scripts CNS](Scripts-CNS.md) | Sim |
| Novo formato de arquivo | `crates/formats` | [Arquitetura](Arquitetura.md) | Sim |
| Nova tela | `src/app/<tela>.rs` | [Layout de arquivos](Layout-de-Arquivos.md) | Sim |

## Regras para um novo tipo de conteúdo

1. Defina como arquivo de dados em `assets/`, embutido pelo `build.rs` quando acompanha o jogo.
2. Crie uma página na wiki em inglês e em `pt-BR` e adicione uma linha à tabela acima.
3. Adicione um teste que carregue os arquivos entregues, para que um dado quebrado falhe no CI e não no jogo.
4. Registre a escolha em um [ADR](../adr/README.md) quando ela introduzir um formato novo.
