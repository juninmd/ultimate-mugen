# Arquivos de rota e desbloqueio (TOML)

Substitui em dados os caminhos de arcade em Lua e as condições `unlock` do Ikemen GO. Veja [Scripts Lua](Scripts-Lua.md) para o raciocínio. Não há runtime de script: a rota é um arquivo TOML e as condições usam uma pequena linguagem de expressões verificada ao carregar.

## Onde ficam os arquivos

| Arquivo | Uso | Origem |
|---|---|---|
| `route_<id do lutador>.toml` | Rota de arcade daquele lutador | Pasta de dados (ao lado do executável, armazenamento do app no Android, armazenamento do navegador na web) |
| `unlocks.toml` | Regras de desbloqueio e ocultação de lutadores e cenários | Mesmo lugar; sem ele vale o `assets/routes/unlocks.toml` embutido |

Um arquivo com erro é reportado no console com o número do passo ou da regra e depois ignorado, então o jogo mantém o comportamento embutido.

## Rota de arcade

```toml
[route]
id = "kfm-arcade"

[[step]]
storyboard = "chars/kfm/intro.def"

[[step]]
repeat = 6
fight = { order = 1 }

[[step]]
when = "consecutive_wins == 6"
fight = { p2 = ["SuaveDude"] }
else = { fight = { order = 3, winscreen = true } }

[[step]]
action = "clear"
```

Cada `[[step]]` tem exatamente um entre `fight`, `storyboard` ou `action`.

| Chave | Significado |
|---|---|
| `fight` | `order` (1 pega o próximo oponente comum, 2 ou mais pega o chefe), `p2` (ids ou nomes, vale o primeiro que existir), `stage`, `winscreen` |
| `storyboard` | Lido e validado, ainda não exibido: a engine avisa e continua |
| `action` | `clear` encerra a rota |
| `repeat` | De 1 a 99 vezes; `when` é verificado a cada repetição |
| `when` | Condição; o passo só roda se for verdadeira |
| `else` | Mesmas chaves do corpo de um passo (`fight`, `storyboard`, `action`); roda quando `when` é falso |

A rota é planejada quando o arcade começa, supondo que o jogador vence todas as lutas, que é o único caminho possível de uma partida. Perder uma luta encerra a partida como sempre.

## Condições

Números, texto entre aspas, `true`, `false`, variáveis, `== != < <= > >=`, `&&`, `||`, `!` e parênteses.

| Variável | Valor |
|---|---|
| `stats.arcade.clear` | Arcades concluídos salvos |
| `consecutive_wins`, `match_no` | Lutas já planejadas nesta partida |
| `mode` | `arcade`, `story`, `survival`, `cpu`, `versus`, `training`, `watch` ou `team` |
| `netplay` | 1 em partida online, senão 0 |
| `team_size` | 1 ou 2 (só em rotas) |
| `player` | Id do lutador escolhido pelo jogador |

Variável desconhecida é erro de carregamento e lista as conhecidas.

## Desbloqueios

```toml
[[unlock]]
target = "char:oboro"
when = "stats.arcade.clear >= 1"

[[hide]]
target = "char:oboro"
when = "mode == \"bossrush\""
```

O alvo é `char:<id>` ou `stage:<id>`. Um alvo com regras `unlock` fica disponível só enquanto todas forem verdadeiras; um alvo com regras `hide` fica oculto enquanto qualquer uma for. Alvos sem regra continuam abertos. As regras de lutador valem na seleção; as de cenário são lidas mas ainda não aplicadas.

## Fora desta versão

Lógica arbitrária, resultados por round (`perfect_rounds`), rótulos e `goto`, e arcos do modo história.
