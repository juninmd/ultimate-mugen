# Formato UMF — criando um personagem

O UMF (*Ultimate Mugen Format*) troca os ~6 arquivos do MUGEN (DEF/SFF/AIR/CMD/CNS/SND) e as centenas de linhas de CNS por **um objeto** tipado (`crates/engine/src/native/types.rs`, JSON). O compilador (`crates/engine/src/native/compile.rs`) gera automaticamente comandos, `[Statedef -1]`, estados de golpe, HitDefs, projéteis, super pause, cancelamentos e hitboxes.

## Estrutura
```ts
const char: NativeChar = {
  format: 'umf/1', id: 'kaze', name: 'Kaze', series: 'Ventos do Leste', author: 'você', color: '#e85d2a',
  stats: { life: 1000, walk: [2.6, 2.2], run: [5, 4.5], jump: [-8.6, -2.6, 0, 2.6], height: 100 },
  art: { type: 'rig', style: { /* cores, cabelo, roupa, arma */ }, poses: { idle1: { /* ângulos */ } } },
  anims: {
    idle: { frames: [{ pose: 'idle1', ticks: 8 }, { pose: 'idle2', ticks: 8 }] },
    jab:  { frames: [{ pose: 'jab0', ticks: 3 }, { pose: 'jab1', ticks: 3, hit: { part: 'handR', radius: 10 } }, { pose: 'jab0', ticks: 5 }] },
    // ... animações padrão: walk, crouch, jumpUp, guard, hitLight, fall, lieDown, getUp, win ...
  },
  moves: [
    { id: 'jab', name: 'Soco Rápido', input: 'x', kind: 'normal', anim: 'jab', hits: [{ at: 2, damage: 30, level: 'light' }] },
    { id: 'sweep', name: 'Rasteira', input: 'b', when: 'crouch', kind: 'normal', anim: 'sweep',
      hits: [{ at: 3, damage: 60, level: 'heavy', guard: 'low', launch: true }] },
    { id: 'wind', name: 'Onda de Vento', input: '~D, DF, F, x', simple: 'sp1', kind: 'special', anim: 'cast',
      projectile: { at: 3, anim: 'windball', hitAnim: 'windhit', speed: [5, 0], offset: [40, -60], damage: 70 } },
    { id: 'rise', name: 'Ascensão', input: 'F, D, DF, y', simple: 'sp2', kind: 'special', anim: 'rise', invuln: [1, 3],
      moves: [{ at: 2, vel: [2, -9] }], hits: [{ at: 2, damage: 90, level: 'heavy', launch: true }] },
    { id: 'storm', name: 'Tempestade', input: '~D, DF, F, D, DF, F, x', simple: 'super', kind: 'super', cost: 1000, anim: 'storm',
      hits: [{ at: 3, damage: 60 }, { at: 5, damage: 60 }, { at: 7, damage: 120, level: 'heavy', launch: true }] },
  ],
  palettes: [{ gi: '#2a6be8' }, { gi: '#222' }],
};
```

## Regras que o compilador aplica
| Campo | Gera |
|---|---|
| `input` | `[Command]` com a sintaxe do MUGEN (`~`, `/`, `$`, `>`, `+`) |
| `simple` | botão de 1 toque (SP1/SP2/SP3/SUPER) no celular e teclas 1/2/3 |
| `kind` | prioridade de comando e regras de cancelamento: normal→especial→super em *hit/guard* |
| `when` | estado de origem (em pé, agachado, no ar) e física |
| `hits[].at` | `HitDef` ativado no quadro N; dano, hitstop, hitstun, knockback e faíscas derivados do `level` |
| `hit.part` no quadro | **Clsn1** automático na ponta do membro/arma; **Clsn2** calculado do esqueleto |
| `projectile` | `Projectile` com animação, velocidade, número de acertos, colisão com outros projéteis |
| `cost` / `kind: 'super'` | consome barra, `SuperPause` com escurecimento + after-image |
| `invuln`, `teleport`, `moves[].vel` | `NotHitBy`, reposicionamento atrás do oponente, `VelSet` por quadro |

Animações com nomes padrão (`idle`, `walk`, `hitLight`, `fall`, `lieDown`, `win`, ...) viram as ações reservadas do MUGEN (0, 20, 5000, 5050, 5110, 181...), então os **estados comuns** do motor funcionam sem nenhuma linha extra.

## Arte
- `type: 'rig'` — esqueleto 2D vetorial desenhado em tempo de carga (nítido em qualquer resolução, paletas grátis).
- `type: 'sheet'` — spritesheet PNG com retângulos e eixos (para quem desenha pixel art).

## E se eu quiser usar CNS?
Personagens MUGEN completos continuam funcionando: importe o `.zip`. A VM é a mesma.

## Loading without rebuilding / Carregar sem recompilar

Drop `<id>.json` files into a `umf/` folder (next to the executable, in the working directory, inside `mugen/umf/`, or in the folder named by `UME_UMF_DIR`). They are loaded at startup next to the built-in fighters; a file with the same `id` as a built-in one replaces it.

Coloque arquivos `<id>.json` numa pasta `umf/` (ao lado do executável, no diretório atual, em `mugen/umf/` ou na pasta indicada por `UME_UMF_DIR`). Eles carregam na inicialização junto dos lutadores embutidos; um arquivo com o mesmo `id` de um embutido o substitui.
