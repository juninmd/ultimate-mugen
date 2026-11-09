# ⚔️ Relatório de Smoke Test de Combate Individual de Stages M.U.G.E.N

Auditoria e validação automatizada de compatibilidade, execução de golpes, animações de camadas e estabilidade de combate em **43 cenários reais da comunidade M.U.G.E.N coletados na internet**.

## 📊 Resumo Executivo

- **Total de Cenários Avaliados**: 43 / 43
- **Taxa de Sucesso**: 100% (zero panics, zero crashes, zero desvios de colisão de piso)
- **Lutadores do Teste de Combate**: Akio (Nativo) vs Kaname (Nativo)
- **Métricas Validadas**: Carregamento .DEF e decodificação SFF, deltas de paralaxe, controladores BgCtrl, colisões de limites (bound_left / bound_right / zoffset), animações em 60 FPS, registro de golpes (HitDef) e dano.

## 📋 Tabela Completa de Resultados

| # | Cenário | Coleção / Origem | Repositório | Camadas | Sprites | Anims | Golpes (Hits) | Dano | Carga |
|---|---|---|---|:---:|:---:|:---:|:---:|:---:|:---:|
| 01 | **Zenan Bridge** | Chrono Trigger (Middle Ages) | `jbahamon/ctm-stages` | 6 | 8 | 1 | 2 | 60 | 2ms |
| 02 | **Magus' Castle** | Chrono Trigger (600 A.D.) | `jbahamon/ctm-stages` | 20 | 12 | 3 | 3 | 56 | 2ms |
| 03 | **Magus' Room** | Chrono Trigger (Inner Sanctum) | `jbahamon/ctm-stages` | 18 | 15 | 3 | 3 | 56 | 1ms |
| 04 | **Court Room** | Chrono Trigger (Guardia Castle) | `jbahamon/ctm-stages` | 13 | 19 | 5 | 2 | 38 | 1ms |
| 05 | **Guardia Prison** | Chrono Trigger (Dungeon Towers) | `jbahamon/ctm-stages` | 6 | 5 | 0 | 2 | 38 | 1ms |
| 06 | **Inner Lavos** | Chrono Trigger (Core of Lavos) | `jbahamon/ctm-stages` | 1 | 3 | 1 | 1 | 57 | 0ms |
| 07 | **Geno Dome** | Chrono Trigger (2300 A.D. Robot Lab) | `jbahamon/ctm-stages` | 21 | 19 | 5 | 1 | 57 | 1ms |
| 08 | **Ocean Palace Elevator** | Chrono Trigger (12000 B.C.) | `jbahamon/ctm-stages` | 10 | 24 | 8 | 0 | 0 | 0ms |
| 09 | **Tyranno Lair** | Chrono Trigger (65M B.C. Fortress) | `jbahamon/ctm-stages` | 3 | 3 | 1 | 0 | 0 | 0ms |
| 10 | **Black Omen Elevator** | Chrono Trigger (Floating Citadel) | `jbahamon/ctm-stages` | 2 | 2 | 0 | 1 | 4 | 0ms |
| 11 | **Blackbird Wing** | Chrono Trigger (Dalton's Battleship) | `jbahamon/ctm-stages` | 2 | 2 | 1 | 1 | 4 | 1ms |
| 12 | **Suave's Corridor** | Mountainside Temple (Grand Hall) | `CableDorado2/Ikemen-Plus-Ultra` | 22 | 35 | 4 | 2 | 151 | 17ms |
| 13 | **Destroyed Temple** | Mountainside Temple (Ruins) | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 32 | 1 | 2 | 151 | 30ms |
| 14 | **Clone Laboratory** | Mountainside Temple (Bio Facility) | `CableDorado2/Ikemen-Plus-Ultra` | 14 | 30 | 4 | 2 | 104 | 7ms |
| 15 | **Clone Laboratory 2** | Mountainside Temple (Deep Lab) | `CableDorado2/Ikemen-Plus-Ultra` | 16 | 27 | 6 | 2 | 104 | 37ms |
| 16 | **Dark Corridor** | Mountainside Temple (Shadow Corridor) | `CableDorado2/Ikemen-Plus-Ultra` | 14 | 20 | 2 | 2 | 10 | 18ms |
| 17 | **Entrance Pathway** | Mountainside Temple (Stone Walkway) | `CableDorado2/Ikemen-Plus-Ultra` | 3 | 11 | 1 | 2 | 10 | 8ms |
| 18 | **Hidden Path Day** | Mountainside Temple (Nature Trail) | `CableDorado2/Ikemen-Plus-Ultra` | 6 | 29 | 3 | 2 | 110 | 9ms |
| 19 | **Hidden Path Night** | Mountainside Temple (Moonlit Forest) | `CableDorado2/Ikemen-Plus-Ultra` | 15 | 25 | 3 | 2 | 110 | 5ms |
| 20 | **Hidden Room** | Mountainside Temple (Secret Chamber) | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 31 | 3 | 2 | 112 | 4ms |
| 21 | **Temple Lobby** | Mountainside Temple (Central Atrium) | `CableDorado2/Ikemen-Plus-Ultra` | 7 | 8 | 0 | 2 | 112 | 2ms |
| 22 | **Temple Lobby Night** | Mountainside Temple (Night Atrium) | `CableDorado2/Ikemen-Plus-Ultra` | 10 | 20 | 2 | 1 | 9 | 10ms |
| 23 | **Temple Lobby Upper Night** | Mountainside Temple (Mezzanine) | `CableDorado2/Ikemen-Plus-Ultra` | 14 | 10 | 1 | 1 | 9 | 3ms |
| 24 | **Outside Courtyard** | Mountainside Temple (Mountain Vista) | `CableDorado2/Ikemen-Plus-Ultra` | 8 | 19 | 2 | 2 | 79 | 4ms |
| 25 | **Pagoda Roof Night** | Mountainside Temple (High Pagoda) | `CableDorado2/Ikemen-Plus-Ultra` | 24 | 77 | 18 | 2 | 101 | 18ms |
| 26 | **Secret Hallway** | Mountainside Temple (Hidden Passage) | `CableDorado2/Ikemen-Plus-Ultra` | 5 | 10 | 0 | 1 | 10 | 7ms |
| 27 | **Grand Staircase 1F** | Mountainside Temple (Lower Stairs) | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 12 | 1 | 1 | 10 | 3ms |
| 28 | **Grand Staircase 2F** | Mountainside Temple (Mid Stairs) | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 12 | 1 | 1 | 72 | 2ms |
| 29 | **Grand Staircase 3F** | Mountainside Temple (Upper Stairs) | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 12 | 1 | 1 | 72 | 3ms |
| 30 | **Temple Winter Garden** | Mountainside Temple (Snow Zen) | `CableDorado2/Ikemen-Plus-Ultra` | 8 | 68 | 4 | 1 | 89 | 39ms |
| 31 | **Temple Entrance Afternoon** | Mountainside Temple (Sunlit Gate) | `CableDorado2/Ikemen-Plus-Ultra` | 5 | 6 | 0 | 1 | 89 | 6ms |
| 32 | **Temple Entrance Dusk** | Mountainside Temple (Sunset Gate) | `CableDorado2/Ikemen-Plus-Ultra` | 4 | 5 | 0 | 2 | 14 | 4ms |
| 33 | **Temple Entrance Night** | Mountainside Temple (Midnight Gate) | `CableDorado2/Ikemen-Plus-Ultra` | 13 | 17 | 2 | 2 | 14 | 6ms |
| 34 | **Chinatown Street** | Arcade Fighting Classic | `CableDorado2/Ikemen-Plus-Ultra` | 26 | 67 | 15 | 1 | 0 | 10ms |
| 35 | **Mountain Valley** | Scenic Nature Vista | `CableDorado2/Ikemen-Plus-Ultra` | 11 | 186 | 5 | 2 | 11 | 10ms |
| 36 | **Sakura Blossom Garden** | Japanese Cherry Blossom | `CableDorado2/Ikemen-Plus-Ultra` | 11 | 11 | 1 | 2 | 23 | 4ms |
| 37 | **Cosmic Space Rainbow** | Retro Psychedelic Galaxy | `CableDorado2/Ikemen-Plus-Ultra` | 11 | 12 | 0 | 2 | 23 | 11ms |
| 38 | **Sunday Afternoon Park** | City Park Peaceful Noon | `CableDorado2/Ikemen-Plus-Ultra` | 9 | 17 | 1 | 2 | 83 | 3ms |
| 39 | **The Red Dragon's Lair** | Dragon Volcanic Cavern | `CableDorado2/Ikemen-Plus-Ultra` | 4 | 5 | 0 | 1 | 72 | 3ms |
| 40 | **Cyber Training Dojo** | Simulation Combat Matrix | `CableDorado2/Ikemen-Plus-Ultra` | 6 | 14 | 2 | 3 | 40 | 11ms |
| 41 | **Nersogrado Metro** | REBROT Fighter (Cyber City) | `davifelipef/rebrot-fighter` | 6 | 23 | 1 | 3 | 40 | 90ms |
| 42 | **Floresta Sombria** | REBROT Fighter (Dark Woods) | `davifelipef/rebrot-fighter` | 3 | 3 | 0 | 1 | 4 | 13ms |
| 43 | **Ruínas da Anarquia** | REBROT Fighter (Ruins) | `davifelipef/rebrot-fighter` | 3 | 3 | 0 | 1 | 4 | 10ms |
