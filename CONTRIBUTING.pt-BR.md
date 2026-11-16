# Contribuindo com o Ultimate MUGEN

[English](CONTRIBUTING.md) · **Português (BR)**

Obrigado por ajudar. Esta página leva você de um clone novo até um pull request aceito. A documentação completa é a wiki em https://juninmd.github.io/ultimate-mugen/wiki/.

## Por onde começar

- Escolha uma issue com o rótulo `good first issue` ou abra uma descrevendo a mudança antes de escrever código grande.
- Leia uma vez a [visão de arquitetura](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Arquitetura.html) e as [regras de layout de arquivos](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Layout-de-Arquivos.html); elas dizem onde cada código deve ficar.
- Personagens, cenários e dados não precisam de Rust: veja [Criar um personagem](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Criar-um-Personagem.html) e [Criar um cenário](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Criar-um-Cenario.html).

## Compilar e rodar

```bash
git clone https://github.com/juninmd/ultimate-mugen
cd ultimate-mugen
cargo run --release
```

No Linux instale antes `libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev pkg-config`. A versão para navegador é `./scripts/build-web.sh`, servida a partir de `web/dist`.

## Verificações antes do pull request

| Verificação | Comando |
|---|---|
| Testes Rust | `cargo test --workspace --release` |
| Limite de 250 linhas por arquivo | `node scripts/file-size.mjs` |
| Fronteiras entre crates | `node scripts/crate-boundaries.mjs` |
| Testes dos scripts | `node --test scripts/*.test.mjs` |
| Livro da wiki | `./scripts/build-wiki.sh` (precisa de `mdbook`) |
| Smoke no navegador | `./scripts/build-web.sh` e depois `node scripts/web-smoke.js web/dist` (Playwright) |

Não rode `cargo fmt --all`: a árvore não está formatada e o comando reescreve arquivos sem relação. Formate só arquivos novos com `rustfmt --edition 2021 <arquivo>`.

## Regras

- Sem comentários no código; os nomes explicam a intenção. Explicações vão para a wiki.
- Inglês é o idioma padrão de código, commits, issues e docs. O português fica em `*.pt-BR.md` e `docs/wiki/pt-BR/`; mantenha os dois em sincronia.
- Um arquivo de código fica abaixo de 250 linhas. Arquivos que já passam do limite não podem crescer; veja [Layout de arquivos](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Layout-de-Arquivos.html).
- `crates/engine` é determinístico: sem relógio, threads, aleatoriedade ou alocação dentro de `Match::tick`.
- Respeite as fronteiras entre crates em [Arquitetura](https://juninmd.github.io/ultimate-mugen/wiki/pt-BR/Arquitetura.html); o CI aplica `scripts/crate-boundaries.mjs`.
- Atualize `docs/wiki/` quando mudar formato, controller, trigger, flag ou estrutura de pastas.
- Mudanças de design relevantes ganham um ADR em `docs/wiki/adr/`.

## Pull requests

- O título deve ser `tipo(escopo)?: descrição` com tipo em `feat|fix|perf|refactor|docs|test|build|ci|chore|style|revert`; o `commit-lint` rejeita o resto.
- Um PR trata de um assunto. Coloque lógica nova em arquivo novo e ligue com uma linha nos arquivos compartilhados, para PRs paralelos não colidirem.
- Inclua prints ou GIFs (antes e depois) para qualquer mudança visível e o resultado do smoke no navegador para mudanças de jogo, renderização, carregamento ou entrada.
- Os releases são automáticos a partir dos tipos de commit no `master`; `feat` é minor, `fix` e `perf` são patch.

## Agradecimentos

O Ultimate MUGEN é inspirado no M.U.G.E.N da Elecbyte e no Ikemen GO. Mantenha esse crédito respeitoso em docs e discussões.
