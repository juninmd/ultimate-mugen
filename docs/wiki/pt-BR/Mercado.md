# Mercado GitHub

[English](../Marketplace.md) · **Português (BR)**

Instale personagens, cenários e músicas direto de qualquer repositório público do GitHub. Abra **Mercado GitHub** no menu principal.

## Começando

1. Pressione `U` (ou clique em **+ Adicionar**) e cole um link: `https://github.com/dono/repo`, `dono/repo` ou o link de uma pasta, como `https://github.com/dono/repo/tree/main/mugen`.
2. O jogo lê a árvore do repositório e lista o que encontrar.
3. Marque os itens (`Espaço`, clique ou **Todos**) e pressione **Instalar** (`I`). Os downloads rodam em paralelo, com barra de progresso.
4. Itens instalados já ficam ativos. Use **Ativar** (`E`) e **Desativar** (`O`) em qualquer seleção, inclusive em massa, ou **Remover** (`X`) para apagar do aparelho.

Tudo fica salvo localmente: a lista de repositórios, os itens instalados e o estado ativo/inativo continuam na próxima vez.

## O que é detectado

| Tipo | Regra | Arquivos instalados |
|---|---|---|
| Personagem | pasta com `.def` e, na árvore, `.sff`, `.air` e `.cns`/`.cmd` | a pasta inteira, mais arquivos declarados em `[Files]` que estejam em outro lugar do repositório |
| Cenário | `.def` com `.sff` de mesmo nome, ou qualquer `.def` dentro de uma pasta `stages/` | os arquivos do cenário, o sprite `spr` e a música `bgmusic` |
| Música | `.ogg`, `.mp3`, `.flac`, `.wav` (wav abaixo de 32 KB é ignorado como efeito sonoro) | o arquivo de áudio |
| Pacote | `.zip` | o arquivo, aberto em memória como uma importação manual |

Arquivos de screenpack (`system.def`, `select.def`, `fight.def`, ...) nunca viram cenário ou personagem. Pastas ocultas (`.git`, `.github`) são ignoradas.

As músicas ativas tocam em rodízio como trilha da luta quando o cenário não tem música própria. Cenário M.U.G.E.N com `bgmusic` mantém a própria trilha.

## Onde ficam os dados

| Plataforma | Estado e downloads |
|---|---|
| Linux | `$XDG_DATA_HOME/ultimate-mugen/market` (padrão `~/.local/share/ultimate-mugen/market`) |
| macOS | `~/Library/Application Support/ultimate-mugen/market` |
| Windows | `%APPDATA%\ultimate-mugen\market` |
| Android | armazenamento interno do app, pasta `market/` |
| Web | IndexedDB, banco `ultimate-mugen-market` |

Defina `UME_MARKET_HOME` para mudar a pasta no desktop. Cada item instalado é um único arquivo; a lista de repositórios, os itens instalados, o ETag de cada listagem e o token opcional ficam em `state.json`.

## Limite de requisições, token e cache

A listagem usa a API do GitHub (60 requisições por hora sem token) e os arquivos vêm de `raw.githubusercontent.com`, que não tem o mesmo limite. A listagem envia o ETag salvo: se nada mudou, a resposta é `304` e vale o cache local. O repositório é atualizado ao abrir se o cache tiver mais de 10 minutos, ou sob demanda com `R`.

Para aumentar o limite da API pressione `T` e cole um token de acesso pessoal (sem permissões para repositórios públicos), ou defina `UME_GITHUB_TOKEN` / `GITHUB_TOKEN` no desktop. O token só é enviado para `api.github.com`.

## Limites de segurança

- Só links `https` para `github.com`, `api.github.com`, `raw.githubusercontent.com`, `media.githubusercontent.com` e `codeload.github.com` são acessados. Redirecionamentos para outros hosts são bloqueados.
- Caminhos vindos do repositório são validados: sem `..`, caminhos absolutos, barras invertidas, letras de unidade ou caracteres de controle. Entradas inseguras somem da listagem.
- Limites: 64 MB por arquivo, 256 MB e 4000 arquivos por item, 48 MB para a listagem. Itens acima disso aparecem como "grande demais" e não podem ser instalados.
- O conteúdo nunca é executado como código. Os personagens passam pelos mesmos leitores da importação manual.
- Respeite a licença de cada autor. Baixar de um repositório não dá direito de redistribuir.

## Observações por plataforma

| Plataforma | Adicionar repositório |
|---|---|
| Desktop | digitar no jogo (colar não existe na janela do jogo; digite `dono/repo`) |
| Web e navegador do celular | o prompt do próprio navegador, com colar |
| App Android | escolha uma sugestão; o app ainda não tem teclado no jogo |

## Veja também

- [Adicionar conteúdo M.U.G.E.N](Conteudo-MUGEN.md) para a pasta `mugen/` manual.
- [Configuração](Configuracao.md) para variáveis de ambiente.
