---
name: ume-character-creator
description: >-
  Guia definitivo, runbook e manual técnico para criação de personagens nativos UMF 2.0
  para o motor Ultimate MUGEN (UME). Cobre arquivo único JSON declarativo, pipelines
  vetorial (Rig) e Pixel-Art (PNG), voo parametrizável, teletransporte, super velocidade,
  multi-transformações (parciais ou corpo todo, troca de habilidades, troca de personagem),
  exibição automática de consumo de super barra no menu de comandos e bateria de testes.
---

# Manual de Criação de Personagens UME: UMF 2.0 & Recursos Core Avançados

O formato **UMF** (*Ultimate Mugen Format*) é o padrão moderno de personagens do motor **Ultimate MUGEN (UME)**. Diferente do ecossistema clássico M.U.G.E.N (que exige 6 ou mais arquivos legados como `.def`, `.sff`, `.air`, `.cmd`, `.cns` e `.snd`), o padrão UME consolida toda a identidade visual, física, lógica de combate, inteligência artificial e habilidades especiais em um **único manifesto JSON declarativo** localizado em `assets/chars/<id>.json`.

---

## 1. Diferencial Arquitetural: UME vs MUGEN Legado

| Recurso | MUGEN Clássico (6 arquivos) | UME Nativo (UMF 2.0 JSON) |
| :--- | :--- | :--- |
| **Arquitetura de Arquivos** | Fragmentada (`.def`, `.sff`, `.air`, `.cns`, `.cmd`, `.snd`) | Arquivo único JSON (`assets/chars/<id>.json`) |
| **Renderização** | SFFv1 (PCX 8-bit) ou SFFv2 (PNG) | Híbrido: Rig vetorial procedural OU Sprites PNG em lote |
| **Paletas & Cores** | Arquivos binários `.act` (256 cores) | Hexadecimais declarativos com shading shader em tempo real |
| **Voo (`Fly`)** | Código manual extenso em CNS (`physics = N` com VelSet) | Declarativo: `abilities.fly` (speed, cost contínuo, input) |
| **Teletransporte** | PosAdd e Turn amarrados a triggers complexos | Declarativo: `abilities.teleport` (cost, invuln, input) |
| **Super Velocidade** | Variáveis manuais e PosAdd duplicados | Declarativo: `abilities.superSpeed` (mul, cost contínuo, afterimage) |
| **Transformações** | Helpers complexos ou troca de sprites manual | Declarativo: `abilities.forms` (multi-estágios, buffs, corpo/partes, swap de golpes ou lutador) |
| **Menu de Comandos** | Texto estático ou ausente | Exibição automática do custo em barras (`1 BAR`, `X PWR`, `X/s PWR`) |
| **Hot-Reload & Build** | Reinicialização manual externa | Embutido via `build.rs` ou carregado via `add_native_json` |

---

## 2. Visão Geral da Anatomia de um Personagem UMF 2.0

```json
{
  "format": "umf/1",
  "id": "meu_lutador",
  "name": "Meu Lutador",
  "series": "Minha Série",
  "author": "Criador",
  "description": "Texto exibido no seletor de personagens.",
  "color": "#1ca3ec",
  "winQuote": "Vitória incontestável.",
  "intro": "A batalha começa agora.",
  "taunt": "Você pode fazer melhor que isso.",
  "form": "Despertar Supremo",
  "boss": false,
  "speed": 1.0,
  "stats": {
    "life": 1000,
    "power": 3000,
    "attack": 100,
    "defense": 100,
    "walk": [2.6, 2.3],
    "run": [5.6, 4.8],
    "jump": [-8.5, -2.6, 0, 2.8]
  },
  "abilities": {
    "fly": { "speed": 4.0, "cost": 60.0, "input": "a+b" },
    "teleport": { "cost": 300.0, "input": "b+c" },
    "superSpeed": { "mul": 2.0, "cost": 120.0, "input": "a+c" },
    "forms": []
  },
  "art": { "type": "rig", "style": {}, "poses": {} },
  "palettes": [],
  "awakeningPalette": { "hair": "#ffe600" },
  "anims": {},
  "moves": []
}
```

---

## 3. Especificação das Novas Features Core

### 3.1 Voo Parametrizável (`abilities.fly`)
Permite ao personagem levitar e controlar o voo em qualquer direção no ar e no chão.
- `speed` (`f32`): Velocidade de movimentação livre direcional (padrão `3.5`).
- `cost` (`f32`): Consumo contínuo de barra de energia por segundo (calculado como `cost / 60` por tick).
- `input` (`string`): Tecla ou combinação para ligar/desligar (padrão `"a+b"`).
- `anim` (`string`, opcional): Animação customizada enquanto voa (fallback para `jumpUp`).

### 3.2 Habilidade de Teletransporte (`abilities.teleport`)
Blink espacial instantâneo com invulnerabilidade que posiciona o personagem nas costas do adversário (`P2Dist X + 40`).
- `cost` (`f32`): Barra de energia consumida imediatamente no disparo (ex: `300.0`).
- `input` (`string`): Comando de acionamento (padrão `"b+c"`).
- `anim` (`string`, opcional): Animação de desmaterialização (fallback para `idle`).

### 3.3 Super Velocidade (`abilities.superSpeed`)
Aceleração cinemática com rastro de AfterImage holográfico ativado por toggle.
- `mul` (`f32`): Multiplicador de velocidade de movimentação (ex: `2.2` = +120% de aceleração).
- `cost` (`f32`): Consumo contínuo de barra por segundo enquanto ativa.
- `input` (`string`): Comando de ativação (padrão `"a+c"`).

### 3.4 Multi-Transformações (`abilities.forms[]`)
O UME suporta múltiplos estágios de transformação, que podem alterar partes do corpo, o corpo todo, os golpes ou metamorfosear o personagem em outro.

#### A. Alteração de Partes vs Corpo Todo:
- **Alteração de Partes**: Defina `parts: ["hair", "eye"]` acompanhado de `palette`. Apenas os elementos listados serão repintados/alterados em tempo de execução.
- **Alteração do Corpo Todo**: Omita o campo `parts`. Todas as chaves da paleta fornecida substituirão o esquema de cores global do personagem.

#### B. Troca de Habilidades (Como 2 Personagens em 1):
- Cada forma pode definir sua própria lista de `moves: [...]`.
- `replace_moves: true`: Desativa os golpes da forma base e habilita exclusivamente os novos golpes da forma.
- `replace_moves: false`: Adiciona os novos golpes ao arsenal existente do personagem.

#### C. Transformação em Outro Personagem:
- Defina `character: "outro_personagem_id"`.
- Em tempo de transformação, o motor carrega e compila o `Arc<CharDef>` do personagem alvo, transferindo vida proporcional, ataques, animações e sprites completos para o lutador.

---

## 4. Exibição de Consumo de Energia no Menu de Comandos

O painel de pausa do Ultimate MUGEN lê dinamicamente o atributo `cost` e `drain` de cada golpe em `def.movelist` e renderiza no canto direito do menu:
- Golpes sem custo (`cost == 0`): Não poluem a interface.
- Super Golpes (`cost == 1000`, `2000`): Exibem `1 BAR`, `2 BARS`.
- Golpes EX / Habilidades com custo fixo (`cost < 1000`): Exibem `{X} PWR`.
- Habilidades com dreno contínuo (`drain: true`): Exibem `{X}/s PWR`.

---

## 5. Fluxo de Criação Passo a Passo de um Lutador UME

1. Crie o arquivo `assets/chars/<seu_id>.json` copiando o modelo de `examples/exemplo_lutador_completo.json`.
2. Configure a identidade (`id`, `name`, `series`, `author`, `color`).
3. Adicione as habilidades core desejadas dentro do bloco `"abilities"`.
4. Defina os golpes em `"moves"`, especificando `"cost"` para golpes que consomem barra.
5. Registre o `<seu_id>` em `assets/chars/index.json` caso queira determinar a posição exata na tela de seleção.
6. Valide a compilação e execute a suíte de testes do motor:
   ```bash
   cargo test -p ume-engine --test native_abilities_and_forms
   ```
7. Inicie o jogo para teste prático em modo treino (`F2` ativa hurtboxes e hitboxes visíveis).

---

## 6. Documentos de Referência e Exemplos

Consulte os arquivos complementares incluídos nesta skill:
- [Exemplo Completo com Todas as Features](examples/exemplo_lutador_completo.json): Arquivo UMF 2.0 funcional com voo, teleporte, super velocidade, multi-transformação e troca de personagem.
- [Manual de Especificação UMF 2.0](references/manual_umf_especificacao_completa.md): Referência técnica exaustiva de cada propriedade do manifesto JSON.
- [Guia de Sprites Vetoriais e Pixel Art](references/guia_sprites_rig_e_pixel.md): Runbook detalhado para rigs vetoriais e integração de spritesheets PNG.

Para gerar ou corrigir sprites pixel art, primeiro inspecione todas as sprites existentes do próprio personagem e use uma pose aprovada como referência explícita de identidade e acabamento; não presuma que `walk0.png` seja canônica nem comece de um canvas genérico quando já houver pixel art válida. Para Garan, `04_special.png` é a âncora canônica; use `walk0.png` apenas como referência secundária depois de confirmar que retrata o mesmo oni. Revise cada arquivo e sequência em folhas de contato, separando defeitos visuais da cobertura estrutural. Retratos `portrait.png` são uma exceção: mantenha o acabamento detalhado de alta resolução e use `assets/chars/sprites/kaito/portrait.png` como referência de qualidade, conforme o procedimento específico no guia.
