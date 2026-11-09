# Gerador de Golpes

`crates/charforge/` (`movekit` module) transforma uma descrição TOML dos golpes em texto MUGEN: animações com caixas Clsn automáticas, statedefs, entradas de comando e um bloco `[Statedef -1]`. Edite o spec e gere de novo em vez de editar a saída à mão.

```
cargo run --release -p ume-charforge -- moves golpes.toml saida/
cargo test --release -p ume-charforge
```

Grava `moves.air`, `moves.cns`, `moves.cmd` e `statedef-1.cns`. Um exemplo completo está em `crates/charforge/examples/moves.toml`; ele espera sprites fatiados pelas [Ferramentas de sprites](Ferramentas-de-Sprites.md) em `sprites/manifest.json`.

## Spec

| Chave | Significado |
|---|---|
| `[defaults]` | `sprite_manifests`, `clsn1_height` (altura em pé, aumenta a caixa de ataque), `hitsound`, `guardsound`, `super_sound`, `voice_channel` |
| `kind` | `normal`, `crouch`, `air`, `special`, `air_special`, `super`, `projectile`, `throw` (define dano, pausa, poder e atributo padrão) |
| `state`, `anim` | Número do estado e da animação (padrão: o estado) |
| `command`, `command_name`, `command_time` | Notação de entrada MUGEN; nome padrão `spec_<estado>` |
| `frames` | `[grupo, item, ticks, "hit"?, {x, y, flip, trans, clsn1, clsn2}?]`; `"hit"` põe Clsn1 automática |
| `hit` | Lista de HitDef; `elem` ou `elems` escolhem os elementos (padrão: primeiro quadro com Clsn1); `extra` adiciona parâmetros brutos |
| `sounds`, `vel`, `vfx`, `helpers`, `cancel`, `extra`, `voice` | PlaySnd, velocidade, Explod, Helper, cancelamentos e controladores brutos |
| `power`, `superpause` | Custo de poder e SuperPause para supers |
| `phases` | Estados encadeados com `name`, `next`, `like`; o golpe é o primeiro estado |

As entradas geradas no `[Statedef -1]` ficam na ordem super, especial, agarrão, normal e usam `roundstate = 2`, tipo de estado e poder.

## Validação

`cargo run --release -p ume-charforge -- check <pasta_do_char> --strict` carrega o personagem com o loader do motor e mostra os mesmos diagnósticos do `mugen_check`; `--strict` falha em controladores, triggers ou parâmetros sem suporte.

## Próximos candidatos do CharFactory

Simulador de golpes tick a tick para frame data, ajuste de combos, ferramentas de faixas de paleta e renderizador de contact sheet. Ainda não portados.

## Créditos

Abordagem inspirada no [CharFactory](https://github.com/eduardovbe/CharFactory) (MIT, Copyright (c) 2026 eduardovbe). Implementação nova; veja `REFERENCES.md`.
