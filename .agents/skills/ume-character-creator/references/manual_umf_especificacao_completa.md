# Manual de Especificação Técnica UMF 2.0 (Ultimate Mugen Format)

O formato **UMF** (*Ultimate Mugen Format*) consolida toda a arquitetura de um lutador de combate 2D em um manifesto declarativo unificado (`JSON`). O compilador nativo do Ultimate MUGEN (`crates/engine/src/native/compile.rs`) traduz o documento em tempo de compilação ou carregamento para a máquina de estados determinística de 60 Hz compatível com M.U.G.E.N / IKEMEN.

---

## 1. Estrutura Raiz do Objeto (`NativeChar`)

| Campo | Tipo | Obrigatório | Descrição |
| :--- | :--- | :---: | :--- |
| `format` | `string` | Sim | Versão do formato. Sempre `"umf/1"`. |
| `id` | `string` | Sim | Identificador único em minúsculas (ex: `"kaito"`, `"zephyr"`). |
| `name` | `string` | Sim | Nome exibido na interface e HUD. |
| `series` | `string` | Sim | Nome da franquia/série de origem. |
| `author` | `string` | Sim | Nome do autor/criador. |
| `description` | `string` | Não | Texto descritivo exibido na tela de seleção. |
| `color` | `string` | Sim | Cor hexadecimal (ex: `"#2f8f9d"`) usada nos cartões e temas da UI. |
| `winQuote` | `string` | Não | Frase de vitória exibida após o nocaute. |
| `intro` | `string` | Não | Linha dita no início do round 1. |
| `taunt` | `string` | Não | Provocação padrão. |
| `form` | `string` | Não | Nome da transformação legado (`form_name`). |
| `boss` | `bool` | Não | Se `true`, oculta da seleção padrão e fecha a escada Arcade. |
| `speed` | `f32` | Não | Multiplicador de velocidade de animação (> 1 encurta frames). |
| `stats` | `object` | Não | Atributos e constantes de movimentação física. |
| `abilities` | `object` | Não | Habilidades especiais core (Voo, Teleporte, Super Velocidade, Multi-Transformações). |
| `passives` | `object` | Não | Regeneração passiva de vida ou barra (`meterRegen`, `lifeRegen`). |
| `art` | `object` | Sim | Estilo vetorial (rig) e biblioteca de poses anatômicas. |
| `palettes` | `array` | Não | Paletas alternativas selecionáveis com botões de soco/chute. |
| `awakeningPalette`| `object` | Não | Substituições de cor ativadas durante transformações. |
| `anims` | `object` | Sim | Mapeamento de animações por nome. |
| `moves` | `array` | Sim | Lista de golpes, projéteis, supers e cancelamentos. |

---

## 2. Atributos Físicos (`stats`)

```json
"stats": {
  "life": 1000,
  "power": 3000,
  "attack": 100,
  "defense": 100,
  "walk": [2.6, 2.3],
  "run": [5.6, 4.8],
  "jump": [-8.5, -2.6, 0, 2.8],
  "gravity": 0.48,
  "airJumps": 1,
  "height": 100,
  "width": [16, 15]
}
```

- `life`: Quantidade máxima de pontos de vida (padrão 1000).
- `power`: Capacidade máxima da barra de super meter (1000 por barra / nível).
- `attack` / `defense`: Multiplicadores percentuais base de dano causado e recebido.
- `walk`: Velocidade horizontal para frente e para trás `[fwd, back]`.
- `run`: Velocidade de corrida frontal e pulo recuado (back-hop).
- `jump`: Vetor de 4 velocidades de salto: `[y_vel, back_x, neutral_x, fwd_x]`.
- `gravity`: Aceleração gravitacional vertical (`movement.yaccel`).
- `airJumps`: Número de pulos duplos permitidos no ar.
- `height` / `width`: Dimensões da caixa de colisão física e empurrão entre lutadores.

---

## 3. Habilidades Especiais Core (`abilities`)

### 3.1 Voo Parametrizável (`abilities.fly`)
Permite ao lutador flutuar livremente nos eixos $X$ e $Y$, cancelando a gravidade padrão (`physics = N`).
- `speed` (`f32`): Velocidade de movimentação direcional (padrão 3.5).
- `cost` (`f32`): Custo contínuo em barra por segundo de voo (dreno por frame: `cost / 60`).
- `input` (`string`): Comando de ativação/desativação (padrão `"a+b"`).
- `anim` (`string`): Nome da animação customizada enquanto voa (fallback para `jumpUp`).

### 3.2 Teletransporte Instantâneo (`abilities.teleport`)
Blink espacial com evasão frame-1 que reposiciona o lutador imediatamente nas costas do oponente.
- `cost` (`f32`): Quantidade de super meter consumida na ativação (ex: `300.0`).
- `input` (`string`): Comando para disparo (padrão `"b+c"`).
- `anim` (`string`): Animação do sumiço (fallback para `idle`).

### 3.3 Super Velocidade (`abilities.superSpeed`)
Aceleração cinemática de corrida e golpes com AfterImage residual.
- `mul` (`f32`): Multiplicador de velocidade de deslocamento (ex: `2.0` = 200%).
- `cost` (`f32`): Dreno contínuo de meter por segundo durante o estado ativo.
- `input` (`string`): Comando de toggle (padrão `"a+c"`).

### 3.4 Multi-Transformações (`abilities.forms[]`)
Permite múltiplas transformações sequenciais ao longo dos rounds:
- `id` (`string`): Identificador único da forma.
- `name` (`string`): Nome exibido na interface e anúncios de despertar.
- `cost` (`f32`): Barra necessária para ativação (ex: `1000.0` = 1 Barra).
- `attack` (`f32`): Multiplicador acumulativo de ataque (ex: `1.4`).
- `defense` (`f32`): Multiplicador acumulativo de defesa (ex: `1.2`).
- `palette` (`object`): Substituição de cores para partes ou corpo todo.
- `parts` (`string[]`): Lista opcional de membros a alterar (ex: `["hair", "eye"]`). Quando omitido, a paleta inteira é atualizada.
- `character` (`string`): ID de outro personagem nativo registrado para metamorfose completa (troca do `Arc<CharDef>`).
- `moves` (`NativeMove[]`): Golpes especiais ou supers exclusivos desta forma.
- `replace_moves` (`bool`): Se `true`, substitui completamente os golpes da forma base; se `false`, adiciona novas habilidades ao arsenal.

---

## 4. Golpes, Projéteis e Consumo de Energia (`moves`)

```json
{
  "id": "corte_celestial",
  "name": "Corte Celestial",
  "input": "~D, DF, F, y",
  "simple": "sp1",
  "kind": "special",
  "anim": "strong",
  "cost": 500.0,
  "invuln": [1, 3],
  "moves": [{"at": 2, "vel": [4.0, -3.0]}],
  "hits": [{"at": 3, "damage": 85, "level": "heavy", "launch": true}]
}
```

### Exibição de Consumo no Menu de Comandos
Todo golpe com `cost > 0.0` tem seu gasto de super meter automaticamente calculado e exibido no painel de pausa:
- `cost == 1000.0`: Exibe `1 BAR`.
- `cost == 2000.0`: Exibe `2 BARS`.
- `cost < 1000.0`: Exibe `{valor} PWR`.
- Habilidades com dreno contínuo (`drain: true`): Exibe `{valor}/s PWR`.
