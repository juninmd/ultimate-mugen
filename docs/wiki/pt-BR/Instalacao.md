# Instalação e execução

## Baixar pronto

Baixe o arquivo da sua plataforma na página de [Releases](https://github.com/juninmd/ultimate-mugen/releases), descompacte e rode `ultimate-mugen` (Windows: `ultimate-mugen.exe` ou `run.bat`). A engine procura atualizações sozinha; para desligar use `--no-update-check` ou `UME_NO_UPDATE_CHECK=1`.

## Compilar

Pré-requisito: Rust estável ([rustup.rs](https://rustup.rs)).

Linux (Debian/Ubuntu):

```bash
sudo apt-get install -y pkg-config libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev
```

| Alvo | Comando |
|---|---|
| Desktop | `cargo run --release` |
| Testes | `cargo test --workspace --release` |
| Web (PWA) | `rustup target add wasm32-unknown-unknown` e `./scripts/build-web.sh`, depois `python3 -m http.server -d web/dist 8080` |
| Android | `cargo install cargo-apk --version 0.9.6 --locked` e `cargo apk build -p ultimate-mugen --lib --release --target aarch64-linux-android` |
| iOS | alvo `aarch64-apple-ios`, guia em [macroquad.rs/articles/ios](https://macroquad.rs/articles/ios/) |

## Estrutura de pastas ao lado do executável

```
ultimate-mugen(.exe)
ultimate-mugen.cfg        configurações salvas pelo menu
mugen/
  chars/<nome>/<nome>.def personagens M.U.G.E.N
  stages/<nome>.def       cenários M.U.G.E.N
  <qualquer>.zip          pacotes zipados (personagens ou cenários)
```

A pasta `mugen/` é procurada, nesta ordem: em `UME_MUGEN_DIR`, no diretório atual, ao lado do executável e dois níveis acima dele.
