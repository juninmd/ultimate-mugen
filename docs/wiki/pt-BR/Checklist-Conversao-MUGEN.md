# Checklist de conversão MUGEN para UMF

`tools/mugen2umf` (crate Rust, skill `mugen-to-ume-migrator`) migra um lutador M.U.G.E.N para um pacote UMF com comportamento idêntico. Ele não reescreve mais os estados como golpes UMF nativos: o pacote leva os dados originais e o motor carrega pelo mesmo código de um personagem M.U.G.E.N (`ume_engine::loader::assemble_mugen_char`). O formato está em [Personagem UMF](Personagem-UMF.md#paridade-com-mugen-migração-exata).

## O que vai junto

| Área | No pacote |
|---|---|
| Estados e controllers | `mugen.cns`, `mugen.st`, `mugen.stcommon` sem alteração (CNS e ZSS), incluindo helpers, `Statedef -2/-3`, IA e agarrões próprios |
| Comandos | `mugen.cmd` sem alteração: `[Defaults]`, todos os `[Command]` e as rotas do `Statedef -1` |
| Constantes | lidas de `[Data]`, `[Size]`, `[Velocity]`, `[Movement]` originais, com a mesma escala de `localcoord` |
| Animações | `mugen.anims`: cada action com offset, ticks, flip, blend e alpha, escala, ângulo, todas as caixas Clsn1/Clsn2, loopstart |
| Sprites | um PNG indexado por sprite do SFF (mesmos índices e paleta), true color como RGBA, ordem e duplicados do SFF mantidos para a escolha de paleta |
| Paletas | paletas do SFF (`sffpal_<k>.png`) e `pal1.act`..`pal12.act`: troca de paleta e `RemapPal`/`PalFX` veem as mesmas cores |
| Sons | cada entrada do `.snd` como `snd_g_i.wav` |
| Campos só do UMF | `profile` (país, nacionalidade, origem, bio, título, estilo de luta, afiliação em en/pt/es), `series`, falas, `rivals`: `profile_facts` junta os fatos, a skill escreve, `set_profile` valida |
| Melhorias | `--extra`: nomes dos golpes (`mugen.names`), `mugenRoutes` para estados que o `.cmd` deixou sem comando, golpes novos em `scripts` (CNS/ZSS) ou como `moves` UMF nativos com `anim` igual ao número de uma action M.U.G.E.N |

## Prova

`mugen2umf parity <original> <pacote>` (também rodado pelo `convert`) compara:

- dados: estados, comandos, constantes, animações, sprites (pixels e eixos), sprites indexados e paletas, sons, tamanhos;
- comportamento: cada comando do `Statedef -1` é jogado pelo input real no original e no pacote, 150 ticks na mesma luta, e cada tick de cada lutador (estado, animação, quadro, posição, velocidade, controle, vida, poder, helpers, sons) precisa bater.

O veredito é `IDENTICAL` ou o primeiro tick em que eles se separam. `cargo test --release -p mugen2umf` converte o Kung Fu Man de `assets/mugen/elecbyte.zip` e exige `IDENTICAL`.

## Limites

O que o motor não suporta do M.U.G.E.N aparece no `report.md` do pacote (diagnóstico do motor). Vale igual para o original e para o pacote: corrija no motor, não no conversor.
