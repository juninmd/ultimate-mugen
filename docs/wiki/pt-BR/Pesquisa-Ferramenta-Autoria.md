# Pesquisa de ferramenta de autoria: Fighter Factory Studio

Resumo em português de [Authoring tool survey](../Authoring-Tool-Survey.md). Análise estática de uma build pública do Fighter Factory Studio (FFS 3). Nenhum arquivo do pacote foi copiado, nenhum binário foi executado.

- O schema `syntax.xml` descreve o formato `.def/.cns/.cmd/.air` como dados: tipos, `required`, `minver`, `deprecated`. Ex.: `StateDef` exige `type`, `movetype`, `physics`, `anim`.
- Checagens de asset: sprite obrigatória ausente, paleta duplicada ou sem uso, paleta de 32 cores em 5 bits.
- O emulador avisa estado, animação ou elemento inválido, guard com movetype errado, RemapPal inválido e bind de Explod inexistente, sempre com tick, estado e linha.
- Fluxo a igualar: importar sprites por nome `grupo_indice`, eixo comum nas exportações, onion skin, relatório de uso de `var/fvar/sysvar`.
- Disassembly com radare2 (Hopper e IDA são comerciais, Ghidra bloqueado pela rede, Rizin fora dos repositórios): o tratamento de guard ramifica pelo `statetype` do defensor, 1 entra no estado 150, 2 no 152, 3 no 154; outro valor gera "wrong move type". Os demais avisos seguem só pelas strings.
- O REA roda, mas a decompilação nativa exige Hopper, Ghidra, IDA ou Rizin, ausentes aqui.

O checklist resultante está em `.agents/skills/mugen-character-creator/references/checklist_lint_editor.md`.
