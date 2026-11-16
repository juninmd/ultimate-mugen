# Entradas de Kit

[English](../Kit-Inputs.md)

Os lutadores nativos não compartilham mais o mesmo conjunto de entradas. Cada arquétipo troca alguns movimentos de meia-lua por entradas que combinam com seu estilo. A lista de golpes do menu de pausa mostra a entrada real de cada golpe.

- **Valdric** (Soberano) e **Volta** (Tesla): carga, segurar trás por 40 ticks e depois frente + botão.
- **Bruno** (Lariat) e **Daichi** (Aura Rush): movimento de 360° + botão, estilo grappler.

Golpes com variante aérea mantêm entradas que podem ser feitas no ar.

## Mobilidade aérea

`stats.airDashes` define quantos air dashes o lutador tem antes de pousar (padrão 1). Rushdown e aéreos ganham mais, grapplers menos: Suzume, Jin, Akio, Lian e Kaito têm 2, Bruno tem 0 e os demais têm 1.

## Continuações rekka

Um golpe só começa a partir de outro declarando `rekka` no JSON do personagem:

```json
{"id":"gust_cross","name":"Gust Crossup","input":"b","kind":"special","anim":"mirage","rekka":{"after":["gustdash"],"window":[6,20]}}
```

`after` lista os ids dos golpes de origem, `window` é o intervalo de tempo de estado (ticks dentro do golpe de origem, a pausa de acerto não conta) em que o comando é aceito, e `contact: true` exige também que a origem acerte ou seja defendida. Sem isso a continuação funciona também no erro. A continuação nunca começa do neutro, então pode reaproveitar um botão simples já usado por um normal. Código: `crates/engine/src/native/rekka.rs`.

Em uso: Lian (Flowing Silk Kicks, depois `y`), Kaito (Storm Rush, depois `x`) e Jin (Gust Dash, depois `b`).

## Requisitos de distância e estado

`requires` também aceita `distance` (pixels entre os corpos, `min` e/ou `max`) e `enemyState` (`air`, `ground`, `crouch`, `down`, `hit` ou `attack`). Dois golpes podem dividir o mesmo comando e a engine escolhe pela situação, desde que as condições não se sobreponham. A variante deve vir antes do golpe que ela substitui na lista `moves`, porque vale o primeiro que servir.

Em uso: Daichi (Landslide Point Blank a até 55 pixels, senão Landslide Heel) e Bruno (Bull Catch contra inimigo no ar, senão Bulldozer Rush).
