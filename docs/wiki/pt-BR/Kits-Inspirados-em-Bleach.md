# Kits inspirados em Bleach

Tres lutadores nativos copiam habilidades de personagens de Bleach. Os nomes dos personagens nao mudam: e so inspiracao. Tudo e dado em `assets/chars/*.json`; nenhum sprite, cenario ou codigo do engine mudou.

## Kaname Fuyutsuki (estilo Byakuya)

Espadachim nobre e preciso: `byakurai` (`~D, DF, F, y`, raio fino rapido), `shakkaho` (`~D, DB, B, x`, bola vermelha), `shunpo_behind` (`D, D, a`, passo relampago com corte) e o super `kageyoshi` (`~D, DF, F, D, DF, F, y`, tempestade de petalas). Regeneracao de energia sobe para 0.5.

## Suzume Kagerou (estilo Yoruichi)

`shunko` (`~D, DB, B, y`) e `flash_cat` (`D, D, b`). Ganho de velocidade por pilha sobe para 0.04.

## Shion Kurogane (estilo Zaraki)

`reckless` (`D, D, x`, armadura de dois golpes) e furia a partir de 50% de vida (ate 1.5x) somada ao enrage por tempo.

## Byakuren Sakuraba (Personagem Secreto do Modo História — Inspirado em Kuchiki Byakuya)
Desbloqueado automaticamente ao zerar o Modo História (`stats.story.clear >= 1` em `assets/routes/unlocks.toml`). Conta com pacote próprio de sprites pixel-art em branco/preto/sakura (`assets/chars/sprites/byakuren/`), 6 efeitos sonoros WAV originais (`snd_200_0..5.wav`) e explora todos os recursos do UMF 2.0:
- **Habilidades Core**: Caminhada Espiritual (`fly`), *Shunpo* (`teleport`), *Senka* (`superSpeed`), *Carga de Reiatsu* (`charge`) e clone autônomo *Utsusemi* (`helpers.clones`).
- **Kido & Shikai**: *Chire, Sakuramai* (chão e ar), *Hadō #4: Byakurai*, *Hadō #33: Sōkatsui*, *Bakudō #61: Rikujōkōrō*, *Bakudō #81: Dankū* e *Shunpo: Senka* com teleporte nas costas.
- **Transformações em 2 Estágios**: Forma 1 (*Bankai: Mil Pétalas da Sombra* — libera *Maré de Pétalas* e *Gokei: Cúpula*) e Forma 2 (*Shukei: Hakuteiken* — libera o golpe supremo de 2 barras *Juízo Branco*).
