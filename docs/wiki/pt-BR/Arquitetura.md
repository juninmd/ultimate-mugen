# Arquitetura

[English](../Architecture.md) · **Português (BR)**

Como o código é dividido, em que direção as dependências podem apontar e onde estender. As decisões por trás das regras ficam nos [ADRs](../adr/README.md).

## Mapa de crates

| Crate | Responsabilidade | Pode depender de |
|---|---|---|
| `ume-formats` | INI, SFF, AIR, SND, ACT, PCX, PNG, FNT, cenários, VFS (pastas e zips) | nada do workspace; compila para `wasm32` |
| `ume-engine` | VM do lutador, expressões, triggers, controllers, HitDef, regras de partida, compilador UMF | `ume-formats` |
| `ume-netplay` | Rollback com ggrs, checksum de estado, formato de replay | `ume-engine` |
| `ume-render` | Pipelines de sprites e pós-processamento em wgpu | somente wgpu |
| `ume-shell` | Adaptador, superfície, janela, entrada, API `quad`, saída de áudio, ritmo de quadros | `ume-render` |
| `ume-market` | Mercado no GitHub: listagem, pacotes, limites de segurança | `ume-formats` |
| `ume-charforge`, `ume-devtools` | Ferramentas de criação e auditoria | formats, engine |
| `ultimate-mugen` (`src/`) | Telas, HUD, cenários, VFX, mixagem de áudio, opções, registro | tudo acima, GPU só por `ume_shell::quad` |

O `scripts/crate-boundaries.mjs` falha o CI quando uma dependência de runtime quebra a tabela (por exemplo `ume-render` dependendo de `winit`, ou `ume-engine` dependendo de `wgpu`).

## Fluxo em execução

1. `ume-formats` lê arquivos de personagem e cenário, ou o JSON nativo UMF, para dados simples.
2. `ume-engine` compila CNS, ZSS e UMF para um único bytecode e executa `Match::tick` a 60 Hz fixos. Nunca lê relógio, threads ou GPU.
3. `ume-netplay` clona e calcula checksum de `Match` para rollback e replays.
4. `src/` lê o estado de `Match` a cada quadro e desenha por `ume_shell::quad`, que agrupa em lotes no `ume-render`.

O determinismo é o motivo de a simulação ser single-thread e sem alocação por tick. Carga e decodificação podem usar threads via `ume_formats::par::map`.

## Onde o código vai

| Você quer adicionar | Coloque em |
|---|---|
| Um controller ou trigger | `crates/engine` (arquivo novo, uma linha de ligação em `controllers.rs` ou `triggers.rs`) e documente em `Scripts-CNS.md` |
| Um formato de arquivo ou parser | `crates/formats` |
| Um lutador, cenário ou rota | Só dados: JSON UMF, `.def`, TOML; sem Rust |
| Uma tela ou elemento de HUD | `src/app/<tela>.rs` ou a pasta da funcionalidade em `src/` |
| Um efeito de GPU | `crates/render`, exposto por `ume_shell::quad` |
| Uma regra do mercado | `crates/market` |

## Pontos de extensão sem código no engine

- Lutadores nativos: `assets/chars/*.json` ([Personagem UMF](Personagem-UMF.md)).
- Cenários: `.def` do M.U.G.E.N ou dados nativos ([Cenários](Cenarios.md)), objetos de história em TOML ([Objetos de cenário](Objetos-de-Cenario.md)).
- Rotas de arcade e desbloqueios: TOML ([Arquivos de rota](Arquivos-de-Rota.md)).
- Passivos: dados no lutador ([Passivos](Passivos.md)).

## Manter sustentável

- Uma responsabilidade por arquivo, limite de 250 linhas, arquivos antigos não crescem ([Layout de arquivos](Layout-de-Arquivos.md)).
- Refatoração que move código é um pull request próprio e não muda comportamento.
- Crie um ADR quando uma decisão mudar uma fronteira, uma dependência ou um formato de dados.
