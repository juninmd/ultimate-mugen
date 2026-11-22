# Parity report: Goku Ssj4

**Goku Ssj4: IDENTICAL | data sections identical 9/9 | moves identical 61/61**

Every move is played from its real input on the original M.U.G.E.N fighter and on the UMF bundle, 150 ticks each, and every tick of both fighters (state, animation, frame, position, velocity, control, life, power, helpers, sounds) is compared.

## Data

| section | compared | different | examples |
|---|---|---|---|
| anims | 168 | 0 |  |
| commands | 65 | 0 |  |
| constants | 79 | 0 |  |
| frame | 1 | 0 |  |
| indexed | 198 | 0 |  |
| palettes | 8 | 0 |  |
| sounds | 19 | 0 |  |
| sprites | 198 | 0 |  |
| states | 114 | 0 |  |

## Moves

| command | input | when | state | reached | states visited | damage | sounds | result |
|---|---|---|---|---|---|---|---|---|
| balao11 | `/B, a` | stand | 800 | no | [20, 200, 0] | 0 | 1 | identical |
| balao22 | `/B, b` | stand | 800 | no | [20, 201, 0] | 0 | 2 | identical |
| balao33 | `/B, c` | stand | 800 | no | [20, 202, 0] | 0 | 2 | identical |
| balao1 | `/F, a` | stand | 800 | no | [20, 200, 0] | 30 | 2 | identical |
| balao2 | `/F, b` | stand | 800 | no | [20, 201, 0] | 69 | 3 | identical |
| balao3 | `/F, c` | stand | 800 | no | [20, 202, 0] | 77 | 3 | identical |
| slash1 | `F, D, x` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| slash2 | `F, D, y` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| slash3 | `F, D, z` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| slash11 | `D, x` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| slash22 | `D, y` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| slash33 | `D, z` | air | 615 | yes | [40, 50, 615, 616, 52, 0] | 137 | 2 | identical |
| punch_d | `~D, DF, F, x` | stand | 2992 | yes | [10, 12, 2992, 0] | 104 | 5 | identical |
| ssj sequence | `~F, B, a` | stand | 1050 | yes | [20, 1050] | 260 | 13 | identical |
| ssj sequence2 | `~F, B, b` | stand | 2050 | yes | [20, 2050, 2530, 1020, 0] | 143 | 9 | identical |
| ssj sequence3 | `~F, B, c` | stand | 2070 | yes | [20, 2070, 2730] | 221 | 12 | identical |
| s_soco | `D, DF, F, b` | stand | 1020 | yes | [10, 12, 1020, 0] | 0 | 1 | identical |
| xbaikamehameha | `~D, DB, B, F, c` | stand | 1900 | yes | [10, 12, 1900] | 0 | 3 | identical |
| fireball | `~D, DF, F, x` | stand | 1000 | no | [10, 12, 2992, 0] | 104 | 5 | identical |
| fireballb | `~D, DB, B, z` | stand | 1099 | yes | [10, 12, 1099, 0] | 56 | 2 | identical |
| fireball3 | `~B, F, B, z` | stand | 1390 | yes | [20, 105, 1390, 1391, 1392, 0] | 131 | 6 | identical |
| fireball3b | `~F, B, F, z` | stand | 2390 | yes | [20, 100, 2390, 2391, 2392, 0] | 131 | 6 | identical |
| airchokamehameha | `~D, DB, B, F, b` | stand | 31800 | yes | [10, 12, 31800] | 162 | 12 | identical |
| airkamehameha | `~D, DB, B, F, a` | stand | 31700 | yes | [10, 12, 31700] | 391 | 22 | identical |
| a | `a` | stand | 300 | no | [200, 0] | 0 | 1 | identical |
| b | `b` | stand | 301 | no | [201, 0] | 0 | 2 | identical |
| c | `c` | stand | 302 | no | [202, 0] | 0 | 2 | identical |
| x | `x` | stand | 310 | no | [210, 0] | 82 | 3 | identical |
| y | `y` | stand | 330 | no | [230, 0] | 72 | 3 | identical |
| z | `z` | stand | 340 | no | [240, 0] | 90 | 3 | identical |
| fly | `a+x` | air | 3001 | yes | [40, 50, 3001] | 0 | 0 | identical |
| fly | `a+x` | stand | 3115 | yes | [3115, 3001] | 0 | 1 | identical |
| m_atack | `b, b, F, b` | stand | 4010 | no | [201, 0] | 0 | 2 | identical |
| m_atack2 | `y, y, F, y` | stand | 6010 | no | [230, 0] | 72 | 3 | identical |
| supergenkidama | `~B, D, c` | stand | 47800 | yes | [20, 10, 47800, 81900] | 0 | 3 | identical |
| Dragon_Fist | `~D, F, D, F, b` | stand | 3030 | yes | [10, 12, 10, 12, 3030, 3031, 3041, 0] | 0 | 6 | identical |
| charge | `/a` | stand | 730 | no | [200, 0] | 0 | 1 | identical |
| teleport_a | `~B, D, B, x` | stand | 7500 | yes | [20, 10, 12, 7500, 0] | 0 | 1 | identical |
| airteleport_a | `~B, F, B, y` | stand | 7550 | yes | [20, 105, 7550, 3001] | 0 | 1 | identical |
| airteleport_b | `~F, B, F, y` | stand | 7535 | yes | [20, 100, 7535, 3001] | 0 | 1 | identical |
| teleport_a2 | `~D, z` | stand | 7510 | yes | [10, 7510, 0] | 0 | 1 | identical |
| teleport_b | `~F, D, F, y` | stand | 7520 | yes | [20, 10, 12, 7520, 0] | 0 | 1 | identical |
| teleport_b2 | `~D, y` | stand | 7530 | yes | [10, 7530, 0] | 0 | 1 | identical |
| kamehameha | `~D, DB, B, F, a` | stand | 1700 | no | [10, 12, 31700] | 391 | 22 | identical |
| chokamehameha | `~D, DB, B, F, b` | stand | 1800 | no | [10, 12, 31800] | 162 | 12 | identical |
| FF | `F, F` | stand | 100 | yes | [20, 0, 100, 0] | 0 | 0 | identical |
| BB | `B, B` | stand | 105 | yes | [20, 0, 105, 0] | 0 | 0 | identical |
| punch_d2 | `~D, DF, F, a` | stand | 3210 | no | [10, 12, 200, 0] | 0 | 1 | identical |
| start | `s` | stand | 195 | yes | [195, 0] | 0 | 0 | identical |
| x | `x` | crouch | 400 | yes | [10, 400, 12, 0] | 30 | 2 | identical |
| y | `y` | crouch | 410 | yes | [10, 410, 12, 0] | 48 | 3 | identical |
| z | `z` | crouch | 440 | yes | [10, 440, 12, 0] | 77 | 3 | identical |
| a | `a` | crouch | 430 | yes | [10, 430, 12, 0] | 0 | 1 | identical |
| b | `b` | crouch | 435 | yes | [10, 435, 12, 0] | 0 | 2 | identical |
| c | `c` | crouch | 437 | yes | [10, 437, 12, 0] | 0 | 2 | identical |
| a | `a` | air | 600 | yes | [40, 50, 600, 50, 52, 0] | 0 | 3 | identical |
| b | `b` | air | 601 | yes | [40, 50, 601, 50, 52, 0] | 0 | 3 | identical |
| c | `c` | air | 602 | yes | [40, 50, 602, 50, 52, 0] | 0 | 3 | identical |
| x | `x` | air | 610 | yes | [40, 50, 610, 50, 52, 0] | 0 | 3 | identical |
| y | `y` | air | 611 | yes | [40, 50, 611, 50, 52, 0] | 0 | 3 | identical |
| z | `z` | air | 612 | yes | [40, 50, 612, 50, 52, 0] | 0 | 3 | identical |
