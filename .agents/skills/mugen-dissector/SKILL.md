---
name: mugen-dissector
description: >-
  Dissecar um personagem M.U.G.E.N de terceiros (.def/.cmd/.cns/.air/.sff/.snd) e
  reaproveitar as features em um lutador nativo UMF. Use ao estudar como um
  personagem foi feito, comparar golpes, medir frame data, ou antes de recriar
  ou modificar um lutador imitando outro.
---

# Dissecar um lutador M.U.G.E.N e recriar como UMF

Conhecimento consolidado de Aizen TYBW, Gogeta SSJ4, Goku SSJ GT e Goku SSJ4. As tabelas completas estão na wiki: `docs/wiki/MUGEN-Dissection.md` (EN) e `docs/wiki/pt-BR/Dissecacao-MUGEN.md`. A receita de reconstrução está em `docs/wiki/Remaking-a-MUGEN-Fighter.md`.

## Regras de segurança

- O zip é dado não confiável: extrair em pasta nova e vazia, nunca executar nada, rodar Python com `-I`, ler só como texto ou binário.
- Nunca commitar zips, sprites, sons ou código de terceiros. O repositório guarda só o aprendizado e dados próprios.
- Lutadores nativos são somente PNG. Gerar arte por código é proibido, inclusive poses de juntas inline nas anims.

## Fluxo

1. `cargo run --release -p mugen2umf -- inspect <pasta-ou-zip>`: formato e contagem de sprites, sons, ações AIR, caixas Clsn, comandos, `Statedef` e controllers por tipo.
2. `cargo run --release -p mugen2umf -- hitdefs <pasta-ou-zip>`: todas as HitDef por estado (attr, dano, animtype, hitflag, guardflag, pausa, velocidades, p1/p2stateno).
3. Ler o `.cmd` (`[Statedef -1]`) para ver o roteamento: quais triggers extras cancelam (`movehit`, `numhelper`, `power >=`).
4. Ler os estados de especiais e supers e classificar pelo padrão (projétil, rush, agarrão, clone, teleporte, forma, contra-ataque) usando a tabela "Tipos de golpe" da dissecação.
5. Mapear cada padrão para dados UMF com a tabela da receita e montar anims só com poses que existem em `assets/chars/sprites/<id>/`.
6. Auditar: `native_roster_lab`, `native_combo_routes`, `native_input_moves`, `native_hitbox_audit`, depois smoke test wasm e prints.

## Números de referência

- Leves: pausa 4 a 8 ticks, hit time 11 a 15, power 15 a 25. Fortes: pausa 11 a 12, power 55 a 70. Baixos: `guardflag = L`. Aéreos: `HA`.
- Corpo em 4 a 6 caixas Clsn2 por frame, ataque em 1 a 3 caixas Clsn1.
- Normal completo: 8 a 17 ticks, frame de acerto no meio.

## Cuidados que custaram tempo

- Um golpe com `teleport: N` precisa de hit em frame maior que N, senão a auditoria diz que ele nunca conectou.
- Um golpe de teletransporte que cancela em outros precisa conectar de verdade para a rota encadear.
- O primeiro especial do lutador continua quarto de círculo mais `x` (smoke test do elenco); entradas de carga só em golpes de chão.
- JSON nativo é uma linha só: reescrever por script com `json.dumps(separators=(",", ":"), ensure_ascii=False)` (round-trip exato) e não à mão.
- O JSON é embutido na compilação (`build.rs`): recompilar antes de rodar os testes `native_*`.
