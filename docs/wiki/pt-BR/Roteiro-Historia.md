# Roteiro do Modo História (TOML)

O Modo História do Ultimate MUGEN é controlado por um arquivo declarativo `.toml`. Você pode personalizar cada fase, onda de inimigos, chefe, falas/diálogos, recuperação de vida e créditos — ou desativar completamente o Modo História.

## Ordem de Carregamento

1. Variável de ambiente `UME_STORY_SCRIPT=<caminho>` (se definida e legível).
2. `story/campaign.toml` no diretório atual ou ao lado do executável do jogo.
3. Campanha padrão embutida (`assets/story/default.toml`).

Se um arquivo externo tiver erro de sintaxe ou for inválido, o motor volta automaticamente para a campanha padrão sem travar.

## Desativando o Modo História

Você pode desativar o Modo História de duas formas:

- **No jogo:** **Configurações -> Jogabilidade -> Modo História (TOML) -> DESLIGADO**.
- **Via TOML:** Crie `story/campaign.toml` com:

```toml
[campaign]
id = "desativado"
enabled = false
```

## Referência do Esquema

```toml
[campaign]
id = "minha_campanha"
enabled = true
credits = 3
wave_recovery = 0.2
stage_recovery = 0.6
boss_life_factor = 1.5
boss_attack_factor = 1.1

[campaign.title]
en = "CYBER REBELLION"
pt = "REBELIÃO CIBER"
es = "REBELIÓN CIBER"

[[stages]]
arena = "neon"
accent = [0.4, 0.9, 1.0]
lane_far = -22.0
lane_near = 8.0
boss = "volta"
alt_boss = "akio"
waves = [
  ["npc_razor", "npc_spitfire"],
  ["npc_colossus"],
]

[stages.title]
en = "SECTOR ZERO"
pt = "SETOR ZERO"
es = "SECTOR CERO"

[stages.beats]
en = ["The grid is locked.", "Break the firewall."]
pt = ["A rede está trancada.", "Quebre o firewall."]
es = ["La red está bloqueada.", "Rompe el cortafuegos."]
```

- `title` e `beats` aceitam tanto texto simples / lista simples (usados em todos os idiomas) quanto tabela `{ en, pt, es }`.
- `alt_boss` entra automaticamente caso o jogador escolha o `boss` principal da fase.

## Editor da História (dentro do jogo)

Ajustes → Jogabilidade → **Editor da História** edita a campanha ativa sem sair do jogo: créditos, vida recuperada por onda e por fase, vida e ataque do chefe e as quatro falas de cada fase no idioma atual. Cada mudança é validada, salva como TOML (`story-campaign.toml` no armazenamento do app, localStorage na web) e aplicada na hora. **Restaurar roteiro original** apaga a cópia salva.

Ordem de busca da campanha: `UME_STORY_SCRIPT` → salvo pelo Editor → `story/campaign.toml`.
