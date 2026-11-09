# Mercado GitHub

[English](../Marketplace.md) · **Português (BR)**

Instale personagens, cenários e músicas direto de qualquer repositório público do GitHub. Abra **Mercado GitHub** no menu principal.

## Como usar

1. Aperte `U` (ou clique em **+ Adicionar**) e informe o link: `https://github.com/dono/repo`, `dono/repo` ou link de subpasta como `https://github.com/dono/repo/tree/main/mugen`.
2. O jogo lê a árvore do repositório e mostra tudo o que encontrou.
3. Marque os itens desejados (`Espaço`, clique ou **Todos**) e aperte **Instalar** (`I`). Os downloads rodam em paralelo com barra de progresso.
4. O conteúdo instalado fica ativo na hora. Use **Ativar** (`E`) e **Desativar** (`O`) em qualquer seleção, inclusive em lote, ou **Remover** (`X`) para apagar do aparelho.

Tudo fica salvo localmente: a lista de repositórios, os itens instalados e o estado de ativação continuam lá na próxima abertura.

## Repositórios sugeridos

| Repositório | Personagens | Stages | Músicas | Packs |
|---|---|---|---|---|
| [juninmd/ultimate-mugen](https://github.com/juninmd/ultimate-mugen) | — | — | — | [elecbyte.zip](https://github.com/juninmd/ultimate-mugen/blob/master/assets/mugen/elecbyte.zip) |
| [Carlmundo/NooksVsCrans](https://github.com/Carlmundo/NooksVsCrans) | [chars](https://github.com/Carlmundo/NooksVsCrans/tree/master/chars) | [stages](https://github.com/Carlmundo/NooksVsCrans/tree/master/stages) | [sound](https://github.com/Carlmundo/NooksVsCrans/tree/master/sound) | [Rayman.zip](https://github.com/Carlmundo/NooksVsCrans/blob/master/chars/Rayman.zip) |

Os dois repositórios estão na lista de sugestões do jogo (`SUGGESTED` em `src/market.rs`). Para adicionar outro, digite `owner/repo` no mercado; os links acima abrem cada pasta no GitHub.

## O que é detectado

| Tipo | Regra | O que é instalado |
|---|---|---|
| Personagem | pasta com `.def` e `.sff`, `.air` e `.cns`/`.cmd` na árvore | a pasta inteira, mais arquivos declarados no `[Files]` que estejam em outras partes do repositório |
| Cenário | `.def` com `.sff` de mesmo nome, ou qualquer `.def` dentro de pasta `stages/` | os arquivos do cenário, o sprite `spr` e a música `bgmusic` |
| Música | `.ogg`, `.mp3`, `.flac`, `.wav` (wav menor que 32 KB é ignorado como efeito sonoro) | o arquivo de áudio |
| Pacote | `.zip` | o arquivo compactado, descompactado em memória como na importação manual |

Arquivos de screenpack (`system.def`, `select.def`, `fight.def`, ...) nunca viram cenários nem personagens. Arquivos dentro de pastas ocultas (`.git`, `.github`) são ignorados.

Músicas ativas tocam como trilha da luta em revezamento sempre que o cenário não tiver música própria. Cenários M.U.G.E.N com `bgmusic` continuam tocando a própria faixa.

## Onde os dados ficam salvos

| Plataforma | Estado e downloads |
|---|---|
| Linux | `$XDG_DATA_HOME/ultimate-mugen/market` (padrão `~/.local/share/ultimate-mugen/market`) |
| macOS | `~/Library/Application Support/ultimate-mugen/market` |
| Windows | `%APPDATA%\ultimate-mugen\market` |
| Android | armazenamento interno do app, pasta `market/` |
| Web | banco IndexedDB `ultimate-mugen-market` |

Defina `UME_MARKET_HOME` para trocar a pasta no computador. Cada item instalado é gravado em um pacote único; a lista de repositórios, itens instalados, o ETag de cada listagem e o token opcional ficam em `state.json`. Os nomes dos pacotes passam por percent-encoding só com minúsculas, então itens e pastas que diferem apenas por maiúsculas/minúsculas nunca dividem o mesmo arquivo no Windows ou no macOS.

## Limites da API, token e cache

A listagem usa a API do GitHub (60 requisições por hora sem token) e os downloads usam `raw.githubusercontent.com`, que não tem essa trava. As listagens guardam o ETag recebido: se o repositório não mudou, o GitHub devolve `304` e a listagem vem do cache local. Um repositório é atualizado ao abrir se o cache tiver mais de 10 minutos, ou manualmente com `R`.

Para aumentar o limite, aperte `T` e cole um personal access token (não precisa de permissões para repositórios públicos), ou defina `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` no computador. O token só é enviado para `api.github.com`.

## Limites de segurança

- Só links `https` para `github.com`, `api.github.com`, `raw.githubusercontent.com`, `media.githubusercontent.com` e `codeload.github.com` são acessados. Redirecionamentos para outros hosts são bloqueados.
- Caminhos vindos do repositório são validados: sem `..`, caminhos absolutos, barras invertidas, letras de unidade, caracteres de controle, espaços no início ou nomes de dispositivo reservados do Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`). Entradas inseguras somem da listagem.
- Limites: 64 MB por arquivo, 256 MB e 4000 arquivos por item, 48 MB para a listagem. Itens acima disso aparecem como "grande demais" e não podem ser instalados.
- O conteúdo nunca é executado como código. Os personagens passam pelos mesmos leitores da importação manual.
- Respeite a licença de cada autor. Baixar de um repositório não dá direito de redistribuir.

## Observações por plataforma

| Plataforma | Adicionar repositório |
|---|---|
| Desktop | digitar no próprio jogo (colar não funciona na janela do jogo; digite `dono/repo`) |
| Web e celular | prompt nativo do navegador, com colar |
