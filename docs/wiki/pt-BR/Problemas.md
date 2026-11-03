# Solução de problemas

| Sintoma | Causa provável | O que fazer |
|---|---|---|
| Personagem não aparece na seleção | `.def` fora de `mugen/chars/<nome>/` ou erro de parse | Rode `mugen_check` na pasta e leia o erro |
| Sprites invisíveis ou quadrados | Sprite ausente no `.sff` ou paleta errada | `sprite_audit` lista as referências quebradas |
| Lutador flutua ou afunda | Estado que não volta ao chão | `hover_check` com `TRACE=1` |
| Personagem HD gigante ou minúsculo | `localcoord` ausente no `.def` | Adicione `localcoord = 1280, 720` em `[Info]` |
| Golpe sem efeito | Controller ainda não implementado | Veja a lista em [Scripts CNS](Scripts-CNS.md) |
| UMF não carrega | JSON inválido ou campo obrigatório faltando | `cargo run --release` mostra o erro do `serde_json` com linha e coluna |
| Jogo lento | Resolução de arte alta | Menu Vídeo → Qualidade, ou `quality=1` no `.cfg` |
| Muitos flashes | Sensibilidade à luz | `impact_lv=0` no `.cfg` |
| Não acha a pasta `mugen/` | Executável rodando de outro diretório | Defina `UME_MUGEN_DIR` |

Para relatar um bug, abra uma issue com o personagem ou cenário, a saída do `mugen_check` e os passos para reproduzir.
