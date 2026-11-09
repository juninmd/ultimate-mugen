# Customização: comparação e proposta

English: [customization-comparison.md](customization-comparison.md).

## O que outras engines oferecem

| Engine | Superfície de customização | Custo para o modder |
|---|---|---|
| M.U.G.E.N | `system.def`, `fight.def`, `mugen.cfg`, arquivos de motif, sprite sheets | Muitos arquivos, formatos de texto, edição binária de SFF, reiniciar para ver mudanças |
| Ikemen GO | Os mesmos mais Lua `main.lua`, `config.json`, módulos, `motif.ini` | Exige Lua e conhecer o layout do motif |
| OpenBOR | Scripts de texto por entidade, `models.txt`, pastas de módulo | Dialeto próprio, reconstruir paks |

## O que o Ultimate MUGEN oferecia

Configurações pelo menu e `ultimate-mugen.cfg` (vídeo, áudio, regras, idioma), remapeamento de teclas, screenpacks para rótulos, ordem da seleção e barras de vida, lutadores nativos em JSON UMF e o gerador de golpes. Faltava: mudar o visual da interface e do HUD sem editar Rust, e qualquer aviso de validação em arquivos editados à mão.

## Lacunas

1. Sem tema em dados: cores do fundo do menu e do HUD eram constantes no código.
2. Sem hot reload: toda edição pedia reinício.
3. Sem validação: erro de digitação não fazia nada, em silêncio.
4. Sem um ponto de partida único para o modder.

## Proposta e ordem

1. **`tweaks.toml`** (este PR): um arquivo ao lado do executável (localStorage na web), criado com padrões na primeira execução, seções `[theme]` e `[hud]` com cores hex, recarregado em até um segundo após salvar, dicas de typo e número da linha em um aviso na tela e no log. Sem Lua, sem recompilar.
2. Estender o `tweaks.toml` com padrões de regras, posições do HUD e rótulos de menu, reaproveitando o parser e os avisos.
3. Pasta `mods/` com arquivos de override por mod sobre o jogo base.
4. Atalho "Abrir tweaks" no jogo e galeria de temas.

O passo 1 vem primeiro porque cobre a maior lacuna (visual em dados), não exige UI nova e cria o carregador, o hot reload e a validação que os próximos passos reaproveitam.
