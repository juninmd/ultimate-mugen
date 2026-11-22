# Esquema UMF

`docs/schema/umf-2.schema.json` é a definição legível por máquina dos [lutadores UMF 2](Personagem-UMF.md) e dos kits `umf-kit/1`. É um JSON Schema (2020-12) que espelha as structs serde de `crates/native/src` e **rejeita chaves desconhecidas**: `"hitz"` ou `"lyfe"` falha em vez de ser ignorado pelo carregador.

## Validar um lutador

```bash
node scripts/umf-schema.mjs                       # todos os lutadores, kits e exemplos do repositório
node scripts/umf-schema.mjs caminho/lutador.json  # um arquivo, por exemplo um pacote do mugen-assets
```

O validador fica no repositório (`scripts/umf-schema.mjs`, sem dependência). O CI roda por `scripts/umf-schema.test.mjs` no job `release-scripts`, então um lutador do jogo que quebre o esquema reprova o pull request. Os erros mostram o ponteiro JSON: `assets/chars/kaito.json: /moves/3/hits/0/level: "mid" is not one of ["light","medium","heavy"]`.

## Usar no editor

Coloque `"$schema"` no lutador (o carregador ignora) ou mapeie o arquivo nas configurações do editor. VS Code:

```json
{ "json.schemas": [{ "fileMatch": ["assets/chars/*.json", "umf/**/*.json"], "url": "./docs/schema/umf-2.schema.json" }] }
```

## O que cobre

| Área | Definição |
|---|---|
| Identidade e perfil | `id` (`^[a-z0-9][a-z0-9_-]*$`), `profile` com textos `en`, `pt`, `es`, `element`, `alignment` |
| `stats`, `passives`, `abilities` | Números tipados, `fly`, `teleport`, `superSpeed`, `charge`, `forms` |
| `anims` | `frames` como `"pose*ticks"` ou objetos com `pose`, `sprite`, `effect`, `hit`, `hurt`, `offset`, `ticks` |
| `moves` | `kind`, `when`, `hits`, `projectile` (um ou lista), `cancels`, `rekka`, `throw`, `cinematic`, `explods`, `afterimage`, `shake`, `flash`, `superPause`, `followUp`, `blink`, `vfx`, `quake`, `velMul`, `velAdd`, `posFreeze`, `posSet`, `palFx` |
| `helpers`, `bossRules`, `scripts` | Stand, clones, fases de chefe, ganchos ZSS e CNS |
| Kits e `extends` | Lutador com `extends` ou `format: "umf-kit/1"` pode omitir os campos obrigatórios; o golpe então precisa só do `id` |

`art.poses` e `art.style` são objetos abertos: pertencem ao pipeline de sprites e ao rig obsoleto. Os campos legados `stage` e `country` no topo são aceitos e ignorados.

## O que não verifica

Referências entre partes (um nome de `anim` que exista, um id de `cancels` que exista, um PNG que exista) e o piso de balanceamento. Isso continua nos testes Rust (`native_quality_bar` e os testes do elenco).

## Mudar o formato

Mudar uma struct serde em `crates/native/src` exige mudar o esquema no mesmo pull request. Rode `node --test scripts/umf-schema.test.mjs`: ele falha quando o esquema rejeita um lutador do jogo, que é como aparece um campo novo no carregador e ausente no esquema.

## Formato de armazenamento

JSON para definições e PNG para sprites foram medidos contra MessagePack, CBOR, bincode, postcard, QOI, WebP sem perda, LZ4 e atlas: veja o [ADR 0008](../adr/0008-umf-schema-and-storage-review.md).
