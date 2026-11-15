# Ajustes de personagem

Configurações → Jogabilidade → **Ajustes de personagens** muda parâmetros de cada personagem sem alterar os arquivos originais. Os valores são porcentagens (100% = sem mudança), salvos em `char_tuning.toml` ao lado do executável (web: armazenamento do navegador).

| Parâmetro | Vale para | Mecanismo |
|---|---|---|
| Vida | MUGEN e UME | `data.life` |
| Dano causado | MUGEN e UME | `data.attack` (multiplicador de ataque) |
| Dano recebido | MUGEN e UME | `tuning.defence` lido pelo multiplicador de defesa |
| Andar, correr, pular | MUGEN e UME | constantes `velocity.*` |
| Gravidade | MUGEN e UME | `movement.yaccel` |
| Barra de poder | MUGEN e UME | `data.power` |
| Custo de energia | MUGEN e UME | valores negativos de `PowerAdd` multiplicados por `tuning.power_cost` |

Faixas: 25% a 300% (barra de poder e custo de energia: 0% a 300%). Personagem com todos os valores em 100% não é gravado no arquivo. Use "Restaurar padrão" para voltar ao original.

Formato:

```toml
["native:kaito"]
life = 150
defence = 75
```

Código: `src/char_tuning/` (modelo, armazenamento, testes), `src/registry/tuning.rs` (aplicado ao armazenar o personagem), `src/app/char_tuning_screen.rs` (tela).
