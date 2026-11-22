# Parity report: Black Widow

**Black Widow: IDENTICAL | data sections identical 9/9 | moves identical 43/43**

Every move is played from its real input on the original M.U.G.E.N fighter and on the UMF bundle, 150 ticks each, and every tick of both fighters (state, animation, frame, position, velocity, control, life, power, helpers, sounds) is compared.

## Data

| section | compared | different | examples |
|---|---|---|---|
| anims | 256 | 0 |  |
| commands | 58 | 0 |  |
| constants | 79 | 0 |  |
| frame | 1 | 0 |  |
| indexed | 975 | 0 |  |
| palettes | 84 | 0 |  |
| sounds | 84 | 0 |  |
| sprites | 975 | 0 |  |
| states | 221 | 0 |  |

## Moves

| command | input | when | state | reached | states visited | damage | sounds | result |
|---|---|---|---|---|---|---|---|---|
| qcf_2p | `~D, DF, F, x+y` | stand | 3000 | yes | [10, 3000] | 161 | 41 | identical |
| qcf_2k | `~D, DF, F, a+b` | stand | 3100 | yes | [10, 3100, 3101, 3102] | 156 | 15 | identical |
| qcb_2p | `~D, DB, B, x+y` | stand | 3200 | yes | [10, 3200, 3201] | 0 | 3 | identical |
| fdf_x | `~F, D, DF, x` | stand | 1000 | yes | [20, 10, 1000, 0] | 12 | 6 | identical |
| fdf_y | `~F, D, DF, y` | stand | 1001 | yes | [20, 10, 1001, 0] | 78 | 7 | identical |
| fdf_z | `~F, D, DF, z` | stand | 1002 | yes | [20, 10, 1002, 0] | 126 | 7 | identical |
| qcf_y | `~D, DF, F, y` | stand | 1020 | yes | [10, 1020, 0] | 102 | 7 | identical |
| qcf_z | `~D, DF, F, z` | stand | 1090 | yes | [10, 1090, 0] | 78 | 6 | identical |
| qcf_x | `~D, DF, F, x` | stand | 1030 | yes | [10, 1030, 0] | 54 | 6 | identical |
| qcf_z | `~D, DF, F, z` | air | 1060 | yes | [40, 50, 1060, 948, 52, 0] | 0 | 4 | identical |
| qcf_x | `~D, DF, F, x` | air | 1050 | yes | [40, 50, 1050, 948, 52, 0] | 66 | 7 | identical |
| qcf_y | `~D, DF, F, y` | air | 1040 | yes | [40, 50, 1040, 948, 52, 0] | 0 | 4 | identical |
| qcb_x | `~D, DB, B, x` | stand | 1100 | yes | [10, 1100, 0] | 60 | 4 | identical |
| qcb_y | `~D, DB, B, y` | stand | 1110 | yes | [10, 1110, 0] | 60 | 4 | identical |
| qcb_z | `~D, DB, B, z` | stand | 1120 | yes | [10, 1120, 0] | 60 | 4 | identical |
| qcb_a | `~D, DB, B, a` | stand | 1200 | yes | [10, 1200, 1201, 1202, 1203, 52, 0] | 60 | 9 | identical |
| qcb_b | `~D, DB, B, b` | stand | 1210 | yes | [10, 1210, 1211, 1212, 1213, 52, 0] | 60 | 9 | identical |
| qcb_c | `~D, DB, B, c` | stand | 1220 | yes | [10, 1220, 1221, 1222, 1223, 52, 0] | 0 | 6 | identical |
| qcf_a | `~D, DF, F, a` | stand | 1300 | yes | [10, 1300, 0] | 54 | 5 | identical |
| qcf_b | `~D, DF, F, b` | stand | 1310 | yes | [10, 1310, 1313, 12, 0] | 54 | 9 | identical |
| qcf_c | `~D, DF, F, c` | stand | 1320 | yes | [10, 1320, 1323, 1324, 0] | 54 | 10 | identical |
| z | `z` | stand | 800 | no | [220, 0] | 60 | 7 | identical |
| FF | `F, F` | stand | 100 | yes | [20, 0, 100, 0] | 0 | 0 | identical |
| BB | `B, B` | stand | 105 | yes | [20, 0, 105, 52, 0] | 0 | 1 | identical |
| DU | `D,$U` | stand | 700 | yes | [10, 700, 701, 52, 0] | 0 | 2 | identical |
| start | `s` | stand | 195 | yes | [195, 0] | 0 | 1 | identical |
| x | `x` | stand | 200 | yes | [200, 0] | 18 | 3 | identical |
| y | `y` | stand | 210 | yes | [210, 0] | 48 | 7 | identical |
| a | `a` | stand | 230 | yes | [230, 0] | 18 | 3 | identical |
| b | `b` | stand | 240 | yes | [240, 0] | 54 | 4 | identical |
| c | `c` | stand | 250 | yes | [250, 0] | 48 | 5 | identical |
| holddown | `/$D` | crouch | 400 | no | [10, 11, 12, 0] | 0 | 0 | identical |
| x | `x` | air | 600 | yes | [40, 50, 600, 948, 52, 0] | 0 | 3 | identical |
| y | `y` | air | 610 | yes | [40, 50, 610, 948, 52, 0] | 0 | 3 | identical |
| z | `z` | air | 620 | yes | [40, 50, 620, 948, 52, 0] | 0 | 3 | identical |
| a | `a` | air | 630 | yes | [40, 50, 630, 948, 52, 0] | 0 | 3 | identical |
| b | `b` | air | 640 | yes | [40, 50, 640, 948, 52, 0] | 42 | 4 | identical |
| c | `c` | air | 650 | yes | [40, 50, 650, 948, 52, 0] | 0 | 3 | identical |
| holdfwd | `/$F` | air | 7730 | no | [40, 50, 52, 0] | 0 | 1 | identical |
| holdfwd | `/$F` | stand | 832 | no | [20, 0] | 0 | 0 | identical |
| holdback | `/$B` | stand | 855 | no | [20, 0] | 0 | 0 | identical |
| FF | `F, F` | air | 880 | no | [40, 50, 110, 0] | 0 | 0 | identical |
| guardpush | `x+y` | stand | 6300 | no | [200, 0] | 18 | 3 | identical |
