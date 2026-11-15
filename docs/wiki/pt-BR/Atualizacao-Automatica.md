# Atualização automática

A versão desktop se atualiza a partir da última Release do GitHub. A tag da release é a fonte da verdade: a CI grava `UME_RELEASE_VERSION` no binário, publica os artefatos e a build instalada se substitui pelo executável correspondente à sua plataforma.

## Artefatos da release

Cada release traz, por plataforma (`windows-x86_64`, `linux-x86_64`, `macos-aarch64`):

| Artefato | Função |
| --- | --- |
| `ultimate-mugen-v<versão>-<plataforma>.zip` / `.tar.gz` | Pacote para instalação manual (binário + README) |
| `ultimate-mugen-v<versão>-<plataforma>[.exe]` | Executável puro baixado pelo atualizador |
| `latest.json` | Manifesto de atualização lido pela build em execução |
| `SHA256SUMS.txt` | Hashes legíveis de todos os artefatos |

O `latest.json` mantém o pacote em `platforms.<plataforma>` e adiciona o binário puro:

```json
{
  "version": "0.4.0",
  "notes": "Notas curtas da release",
  "pub_date": "2026-11-05T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "url": "https://github.com/juninmd/ultimate-mugen/releases/download/v0.4.0/ultimate-mugen-v0.4.0-windows-x86_64.zip",
      "sha256": "<sha256 do pacote>",
      "size": 1234567,
      "binary": {
        "url": "https://github.com/juninmd/ultimate-mugen/releases/download/v0.4.0/ultimate-mugen-v0.4.0-windows-x86_64.exe",
        "sha256": "<sha256 do binário>",
        "size": 9876543
      }
    }
  }
}
```

Só as três plataformas publicadas se atualizam sozinhas. Web, Android, iOS, Linux que não seja x86_64 e macOS Intel seguem apenas com o aviso na tela.

## Fluxo

1. Na inicialização o jogo busca `releases/latest/download/latest.json` por HTTPS (timeout de 8 s, ignorado em silêncio quando offline).
2. Uma `version` mais nova com entrada `binary` para a plataforma atual mostra um painel no título/menu com a versão instalada e a disponível. Nada é baixado até a pessoa aceitar "Atualizar agora".
3. Ao aceitar, o download roda em segundo plano (timeout de 180 s) para `.update-v<versão>[.exe]` ao lado do executável em execução.
4. O download só é aceito se o `sha256` e o tamanho em bytes baterem com o manifesto e se o executável baixado responder `--version` com `ultimate-mugen <versão>`.
5. Verificado o pacote, ele substitui o executável instalado (no Windows o executável em uso é renomeado para `.old` antes) e o jogo reinicia com `--no-update-check`, para que um manifesto ruim não cause laço de reinícios.
6. O `.old` anterior é apagado na inicialização seguinte.

Falha de download, máquina offline ou pasta de instalação somente leitura mudam apenas a linha de status; a build em execução continua funcionando. "Agora não" esquece a versão até o próximo boot, e o aviso do rodapé continua visível.

## Linha de comando e como desligar

| Flag | Efeito |
| --- | --- |
| `--check-update` | Mostra se existe release mais nova e sai |
| `--update` | Baixa, verifica e instala a release mais nova, depois sai |
| `--no-auto-update` / `UME_NO_AUTO_UPDATE=1` | Mantém o aviso no rodapé e nunca mostra a pergunta (então nunca baixa nem instala) |
| `--no-update-check` / `UME_NO_UPDATE_CHECK=1` | Nenhuma requisição de rede |

Builds locais `-dev` nunca se atualizam.

## Conferir uma release

`node scripts/release-manifest.mjs --verify latest.json --dir <artefatos> --version <versão> --repo juninmd/ultimate-mugen` confere que o pacote e o binário puro existem, que as URLs apontam para a release da tag e que os `sha256` batem. O workflow `Release` roda exatamente essa checagem no rascunho antes de publicar e valida o manifesto público depois.
