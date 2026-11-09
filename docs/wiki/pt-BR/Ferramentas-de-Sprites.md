# Ferramentas de Sprites

Ferramentas locais (Rust, `charforge` binary) em `crates/charforge/` (`spritekit` module) que transformam arte gerada em assets prontos para MUGEN. Nunca tocam `assets/chars` nem cenários: a saída vai para a pasta escolhida.

```
cargo run --release -p ume-charforge -- slice folha.png saida/ --group 9000 --axis feet
cargo run --release -p ume-charforge -- slice folha.png saida/ --mode grid --grid 6 2
cargo run --release -p ume-charforge -- palettes base.act variantes/ --only red blue dark --keep 1 2
cargo run --release -p ume-charforge -- air saida/manifest.json --action 200 --ticks 4 --attack
cargo test --release -p ume-charforge
```

| Ferramenta | O que faz |
|---|---|
| `slice` | Detecta o fundo (transparência ou cor da borda), recorta por pedaços conectados ou por grade, põe o eixo nos pés (chão comum por fileira), monta uma paleta única de 256 cores (índice 0 transparente, avisa se quantizou) e grava PNGs indexados e `manifest.json` |
| `palettes` | Gera variantes `.act` (matiz, saturação, brilho) a partir de um `.act` base; o índice 0 e os de `--keep` não mudam |
| `air` | Gera uma action AIR a partir do manifesto com `Clsn2` anatômico (cabeça, tronco, pernas) e `Clsn1` opcional para membros estendidos, relativos ao eixo de cada sprite |

`collision_boxes.py` também oferece `interpolate_boxes` para quadros intermediários.

## O que não foi portado

O gerador de golpes está em [Gerador de Golpes](Gerador-de-Golpes.md). Os editores gráficos, o assistente de IA e o servidor MCP do CharFactory, o simulador tick a tick e os escritores SFF/SND se sobrepõem ao próprio motor (`crates/formats`, `crates/engine`, `mugen_check`) ou são grandes. Os candidatos futuros estão na página do Gerador de Golpes.

## Créditos

Abordagem inspirada no [CharFactory](https://github.com/eduardovbe/CharFactory) de eduardovbe, licença MIT, Copyright (c) 2026 eduardovbe. O código aqui é uma implementação nova e menor; o aviso MIT está em `REFERENCES.md`.
