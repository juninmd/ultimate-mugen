# Lutadores dos fullgames M.U.G.E.N mais famosos

30 personagens extraídos de três dos fullgames mais conhecidos da cena M.U.G.E.N,
instalados em `mugen/chars/` e validados no motor.

- Instalação: `C:\Users\jr_ac\AppData\Local\Programs\UltimateMUGEN\mugen\chars\`
- Instalador: `scripts/fetch-famous-chars.ps1` (baixa do Archive.org, extrai e copia)
- Volume: ~305 MB

## Mortal Kombat Project 4.9.3 (10)

| Personagem | `.def` | Autor do char | Anims / Sprites | Pasta |
|---|---|---|---|---|
| Scorpion | `scorpion umkiii.def` | Team MKP | 194 / 533 | `Scorpion MKP` |
| Sub-Zero | `sub-zero umk3.def` | SUB | 228 / 592 | `Sub-Zero MKP` |
| Liu Kang | `liu_kang.def` | Team MKP | 163 / 319 | `Liu Kang MKP` |
| Raiden | `raiden mk3.def` | PhoboSS & Borg117 | 249 / 734 | `Raiden MKP` |
| Sonya Blade | `sonya.def` | Team MKP | 170 / 435 | `Sonya Blade MKP` |
| Jax Briggs | `jax briggs.def` | Team MKP | 166 / 390 | `Jax Briggs MKP` |
| Johnny Cage | `johnny cage.def` | SUB | 191 / 353 | `Johnny Cage MKP` |
| Mileena | `mileena mkii.def` | Fly / Aice Man | 136 / 284 | `Mileena MKP` |
| Shang Tsung | `shang tsung mkii.def` | FIREBOY | 180 / 562 | `Shang Tsung MKP` |
| Shao Kahn | `shao kahn.def` | unknown & SUB | 155 / 295 | `Shao Kahn MKP` |

Origem: <https://archive.org/details/mortal-kombat-project-4.9.3>

## Super Marvel vs. Capcom: Eternity of Heroes (10)

| Personagem | `.def` | Autor do char | Anims / Sprites | Pasta |
|---|---|---|---|---|
| Ryu | `ryu_mvc.def` | Scar | 241 / 1055 | `Ryu MvC` |
| Ken | `ken_mvc.def` | Scar | 232 / 1075 | `Ken MvC` |
| Hulk | `incrediblehulk.def` | Acey | 290 / 970 | `Hulk MvC` |
| Homem-Aranha | `amazingspiderman.def` | Sludge | 255 / 915 | `Homem-Aranha MvC` |
| Capitão América | `captainamerica_mvc.def` | Acey | 357 / 1544 | `Capitao America MvC` |
| Homem de Ferro | `mvc2_ironman.def` | IronMan | 343 / 1905 | `Homem de Ferro MvC` |
| Wolverine | `xmvcwolverinex.def` | vyn | 232 / 821 | `Wolverine MvC` |
| Magneto | `mvc2_magneto.def` | Magneto | 269 / 1570 | `Magneto MvC` |
| Morrigan | `morrigane.def` | EMW | 292 / 933 | `Morrigan MvC` |
| Deadpool | `deadpool.def` | Unlimited Team / Wucash | 331 / 1283 | `Deadpool MvC` |

Origem: <https://archive.org/details/SMVCEOH>

## The King of Fighters Zillion Super Edition 2022 (10)

| Personagem | `.def` | Autor do char | Anims / Sprites | Pasta |
|---|---|---|---|---|
| Kyo Kusanagi | `kyo.def` | Misamu K Young | 949 / 2548 | `Kyo KOF` |
| Iori Yagami | `iori.def` | MugenChina | 878 / 2453 | `Iori KOF` |
| Terry Bogard | `terry.def` | Alpha Dreams | 563 / 1322 | `Terry KOF` |
| Mai Shiranui | `mai.def` | Misamu K Young | 374 / 1309 | `Mai KOF` |
| Kula Diamond | `kula.def` | DDD111 & KaKu | 372 / 1022 | `Kula KOF` |
| Rugal Bernstein | `rugal.def` | Capcom | 909 / 1717 | `Rugal KOF` |
| Ryo Sakazaki | `ryo.def` | Alpha Dreams | 606 / 1232 | `Ryo KOF` |
| Robert Garcia | `robert.def` | Alpha Dreams | 831 / 1765 | `Robert KOF` |
| Leona Heidern | `leona.def` | Proto | 752 / 1777 | `Leona KOF` |
| Athena Asamiya | `athena.def` | Misamu K Young | 1456 / 3639 | `Athena KOF` |

Origem: <https://archive.org/details/the-king-of-fighters-zillion-super-edition-mugen-2022>

### Particularidade do pacote KOF

Os chars do KOF Zillion são **dependentes do fullgame**: o `.def` original aponta
`cmd = ..\..\data\command.cmd` e `stcommon = common1.cns`, arquivos que vivem em
`data/` do jogo. Para deixá-los autocontidos, o instalador copia `command.cmd` e
`common1.cns` para dentro de cada pasta e reescreve a linha `cmd =` do `.def`.

Sem esse ajuste o lutador **anda mas nunca ataca** (os `ChangeState` de comando
ficam no `common1.cns` compartilhado). Antes: 5 estados distintos e 0 golpes em
60 s de CPU; depois: 26–49 estados e 8–47 golpes conectados.

## Correções de motor necessárias (feitas no código)

Ao importar esses fullgames apareceram três limitações reais do Ultimate MUGEN,
corrigidas em `crates/engine`:

| Sintoma | Causa | Correção |
|---|---|---|
| `panic: index out of bounds (len 1, index 1)` ao carregar Kyo | `localcoord = 400` (valor único) e o loader fazia `lc[1]` | `loader::local_size()` aceita 1 valor (`400` → `400x240`) e tem teste de regressão |
| Lutador carrega, anda, mas **nunca ataca** | `stcommon` (common1.cns) não era lido: os `ChangeState` de comando moram lá | loader passa a ler `stcommon` **apenas** de dentro da pasta do char (sem fallback global) |
| Lutadores grandes (localcoord 400) não chegavam perto o bastante para o CPU atacar | distâncias do AI fixas em 70/110 unidades | `ai.rs` escala as distâncias por `local_w / 320` |

## Validação

```powershell
# todos os personagens (dublados + fullgames) em combate CPU de 60 s
cargo run --release -p ume-engine --example mugen_check -- "$env:LOCALAPPDATA\Programs\UltimateMUGEN\mugen\chars"

# regressão do motor
cargo test --workspace --release
```

Resultado completo em `docs/PTBR_CHARS.md` (dublados) e no log de `mugen_check`
usado na entrega: **todos os 30 carregam e lutam** (`OK`, sem `FAIL`/`PANIC`).

## Licenciamento

Os arquivos **não** são redistribuídos com o projeto (`mugen/` está no `.gitignore`);
o script baixa dos links públicos do Archive.org. Personagens e sprites pertencem a
NetherRealm/WB (Mortal Kombat), Marvel/Capcom e SNK — uso pessoal.
