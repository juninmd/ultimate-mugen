# Pipeline de QA web

Todo pull request joga o build wasm de verdade no Chromium headless, em shards paralelos, e falha em qualquer erro de WebAssembly. Uma falha deixa prints e logs como artefatos do CI.

## O que roda

`scripts/web-qa.json` lista os cenários; `scripts/web-qa.js` roda um shard deles. Job `web-qa` do CI (matriz, `fail-fast: false`):

| Shard | Cenários |
|---|---|
| `ui` | fluxo título até a luta, clique em Arcade, intro da história, epílogo |
| `mobile` | menu por toque, controles de toque na luta, smoke da engine em viewport Pixel 7 |
| `engine` | smoke da engine (`?smoke=1`): boot, menus, seleção, treino, 5v5, história; mais o autoteste do guard |
| `loading` | import M.U.G.E.N (kfm), lutadores nativos com despertares, arena de poder, desbloqueio de áudio, tela de erro fatal (`unreachable`) |
| `match-1`, `match-2` | partidas completas em duas metades ([Smoke da partida web](Smoke-Partida-Web.md)) |

## O guard

Todo cenário roda com `scripts/web-guard.js` pré-carregado. Ele observa cada página e falha a execução, mesmo que o script esqueça, quando houver:

- `panicked`, `PanicHookInfo`, `unreachable executed` ou `renderbufferStorageMultisample` no console;
- `gpu error:` (handler de erros do wgpu em `ume-shell`);
- `pageerror` não tratado ou crash do Chromium.

Nos primeiros problemas salva `problem-N.png` na hora, e um `final-N.png` antes de fechar o navegador. Também grava `console.log` (cada linha do console com tempo e página) e `network.log` (requisições que falharam e HTTP 400+). O `fatal-trap` usa `expectFatal` porque provoca um trap de propósito; ali o guard só registra.

## Artefatos

Cada shard envia `web-qa-<shard>` (sempre, guardado 7 dias): `summary.md`, `summary.json` e uma pasta por cenário com `output.log`, `console.log`, `network.log`, `guard.json`, `shots/` e os `.png`. O resumo também vai para a página do job, com os problemas e as últimas linhas de saída.

## Rodar localmente

```
./scripts/build-web.sh
npm install --no-save playwright@1.56.1 && npx playwright install chromium
node scripts/web-qa.js --dist web/dist --out web-qa-report --shard ui
node scripts/web-qa.js --only fatal-trap,engine-smoke --jobs 2
```

GL por software é lento: mantenha `--jobs` em 1 ou 2 localmente. Para uma página avulsa use `?debug=1` e leia o console.

## Adicionar um cenário

1. Crie `scripts/web-<nome>-smoke.js` que receba a pasta dist como primeiro argumento e saia com código diferente de zero em falha.
2. Inclua em `scripts/web-qa.json` com um shard, ou liste em `excluded` com o motivo (rede, ferramenta manual).

`scripts/web-qa.test.mjs` falha quando um `web-*-smoke.js` não está coberto nem excluído, ou quando os shards do manifesto e do `ci.yml` divergem.

## Tour visual

`node scripts/web-tour.js web/dist <saida>` percorre título, menu, todas as telas utilitárias, seleção, cenários, VS, luta e pausa com teclas reais e salva um print numerado por passo (`TOUR_MOBILE=1` usa um celular na horizontal, `TOUR_LANG=pt-BR` define o idioma do navegador). Falha em panic, erro de navegação ou do navegador, ou se a luta não começar; os prints servem para revisão e evidência de PR.
