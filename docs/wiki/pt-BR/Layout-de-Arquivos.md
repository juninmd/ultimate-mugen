# Layout de arquivos

[English](../File-Layout.md) · **Português (BR)**

Uma feature precisa ser revisável e mesclável sem esperar as outras. Isso só funciona quando cada arquivo tem um único motivo para mudar: duas pessoas trabalhando em features diferentes mexem em arquivos diferentes, e um PR que adiciona um modo não reescreve o arquivo que outro PR está editando.

## A regra de 250 linhas

- Nenhum arquivo de código passa de **250 linhas físicas** (`node scripts/file-size.mjs`).
- O limite não é meta. A maioria dos arquivos deve ficar entre 80 e 200 linhas: se um arquivo já tem dois motivos para mudar com 120 linhas, divida agora.
- A regra vale para Rust, JavaScript, TypeScript, CSS, HTML e WGSL - todo arquivo que um desenvolvedor lê como código.
- Não vale para conteúdo e documentação: JSON UMF, `.cns`, `.air`, `.def`, `.snd`, atlas de sprites, páginas `.md` e imagens geradas.
- Tabelas Rust geradas contam como código. Quando não couberem, coloque-as em um arquivo só de dados, em uma pasta com o nome da tabela (por exemplo `native/tables/`), para ninguém editar à mão.

## Arquivos legados (catraca)

A árvore já tem arquivos acima do limite. Eles estão em [`scripts/file-size-baseline.json`](../../scripts/file-size-baseline.json) com a contagem exata de linhas no dia em que a regra entrou.

- Um arquivo no baseline pode continuar grande, mas não pode crescer uma linha.
- Um arquivo fora do baseline nasce com menos de 250 linhas.
- Um arquivo que desce para menos de 250 linhas sai do baseline. Rode `node scripts/file-size.mjs --update` no mesmo PR que o dividiu.
- Nunca regenere o baseline para aumentar um número. `--update` só é válido junto com uma divisão.

O CI roda `node scripts/file-size.mjs`; uma execução com falha lista cada caminho, a contagem e o limite.

## Uma responsabilidade por arquivo

Um arquivo tem uma responsabilidade quando todas as respostas abaixo são iguais:

| Pergunta | Mesma responsabilidade | Responsabilidades diferentes |
|---|---|---|
| Por que ele mudaria? | duas funções mudam quando o formato muda | uma muda com o formato, a outra com a câmera |
| Que dados ele possui? | uma struct e seus auxiliares | várias structs que conversam entre si |
| Quem revisa? | o mesmo revisor com o mesmo contexto | um especialista diferente |
| Dá para testar sozinho? | sim, pela API pública da crate | só pela tela inteira |

O nome do arquivo declara a responsabilidade: `collision.rs`, `hitdef.rs`, `palette.rs`, `lifebars.rs`. Nomes que escondem a responsabilidade são recusados na revisão: `utils.rs`, `helpers.rs`, `misc.rs`, `common.rs`, `manager.rs`.

## Formato de pasta

Uma feature é uma pasta; o arquivo ao lado é a face pública dela.

```
crates/engine/src/native/
  mod.rs            fachada: declarações e reexports, só isso
  compile.rs        JSON UMF -> estados
  pack.rs           empacotamento de atlas
  rig.rs            tipos do esqueleto e pontos de entrada públicos
  rig/
    body.rs         articulações do corpo
    detail.rs       articulações de cabelo e roupa
    face.rs         articulações do rosto
```

- `feature.rs` guarda os tipos compartilhados por todas as partes e as funções que o resto da crate chama.
- `feature/<assunto>.rs` guarda um assunto cada. O Rust permite uma pasta ao lado do arquivo de mesmo nome, então não é preciso `mod.rs` dentro dela.
- `mod.rs` é fachada: declarações de módulo e `pub use`, nada mais. Mire em 60 linhas; `mod.rs` com lógica é arquivo que não foi dividido.
- Leve o tipo junto do código que o possui. Mantenha o caminho antigo funcionando com um reexport (`pub use collision::ResolvedHit;`) para a divisão não virar um rename na árvore inteira.
- A aplicação segue a mesma ideia: `src/app.rs` mantém o frame e o estado, `src/app/<tela>.rs` é dona de uma tela cada.

## Dividir um arquivo existente

1. Liste as responsabilidades do arquivo. Se a lista não é óbvia, leia os chamadores primeiro; o arquivo ainda não foi entendido.
2. Crie `feature/` e mova uma responsabilidade por PR, um commit por extração.
3. Não mude comportamento durante a extração. Os testes passam antes e depois, e o diff é quase todo movimento de código.
4. Extraia primeiro a menor unidade coesa (a de menos chamadores). Reescrita total conflita com todo PR aberto.
5. Atualize esta página se o novo layout for diferente do descrito aqui.

## Trabalhar em paralelo

- Prefira arquivos novos. Uma feature adiciona `feat/<coisa>.rs` e uma linha de registro; não anexa um bloco a um arquivo quente.
- Pontos de registro são os únicos arquivos que duas features compartilham de forma legítima. Adicione uma linha, em ordem alfabética, e nunca reformate o arquivo: ramos de `controllers.rs` e `triggers.rs`, declarações em `mod.rs`, `assets/chars/index.json`, tabelas de rotas.
- Se a feature precisa mexer em um arquivo quente, coloque a lógica no próprio arquivo primeiro e rebaseie a linha de ligação por último, para o conflito ser de uma linha.
- Não renomeie nem reformate arquivo compartilhado dentro de um PR de feature. Faça isso sozinho, sem mudar comportamento.
- Uma página de wiki por feature, linkada na [Home](Home.md), para duas features não editarem o mesmo parágrafo.

## Arquivos quentes para dividir primeiro

Estes arquivos concentram as responsabilidades mais independentes e são a origem atual dos conflitos de merge. Dividir um deles remove a entrada do baseline.

| Arquivo | Linhas | Divisão sugerida |
|---|---|---|
| `crates/engine/src/native/rig.rs` | 2265 | `rig/skeleton.rs`, `rig/pose.rs`, `rig/palette.rs` ao lado de `body.rs`, `detail.rs`, `face.rs` |
| `crates/engine/src/matchs.rs` | 1751 | `matchs/round.rs`, `matchs/collision.rs`, `matchs/camera.rs`, `matchs/tick.rs` |
| `src/app.rs` | 1486 | `src/app/<tela>.rs`, uma tela cada, frame e estado ficam em `app.rs` |
| `src/stages.rs` | 1069 | `stages/load.rs`, `stages/layers.rs`, `stages/motion.rs` |
| `src/stagefx.rs` | 704 | `stagefx/scenes.rs` já existe; extraia `stagefx/particles.rs`, `stagefx/weather.rs` |
| `src/market.rs` | 700 | `market/install.rs`, `market/listing.rs`; a crate mantém a lógica |
| `crates/engine/src/native/rig/detail.rs` | 723 | `detail/hair.rs`, `detail/cloth.rs` |
| `crates/formats/src/screenpack.rs` | 610 | `screenpack/lifebars.rs`, `screenpack/motif.rs` |

## Checklist de revisão

- [ ] Nenhum arquivo novo acima de 250 linhas.
- [ ] Nenhum arquivo do baseline cresceu.
- [ ] Arquivo que cruzou para menos de 250 linhas saiu do baseline neste PR.
- [ ] Lógica nova mora em arquivo com o nome dela, não em `utils.rs`.
- [ ] Arquivos compartilhados mostram apenas linhas de registro, sem reformatação.
- [ ] A página de wiki da feature foi atualizada em inglês e português.

## Páginas relacionadas

- [Customizar a engine](Engine.md) - arquitetura, novo controller, novo trigger ou formato.
- [Desempenho](Desempenho.md) - onde ficam os caminhos quentes.
