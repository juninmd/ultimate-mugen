# Fighter study: 50+ fighters and what they teach

Reference survey of 52 well known fighters from commercial games and M.U.G.E.N. releases. It records the kit that defines each one and how that kit maps to a UMF fighter. Public move wikis (SRK, Dustloop, StrategyWiki, MKKomplete, Fighters Generation) were consulted; exact frame data differs per game version, so treat the numbers as design direction, not as copied data. No art, sprite or code from these games is used.

## Fast fighters (rushdown, mobility)

| Fighter | Game | Defining tools | UMF take-away |
|---|---|---|---|
| Ken | Street Fighter | Fast run-in kicks, quick shoryuken, kara-cancel normals | Short dash attack that cancels into specials (`lunge`) |
| Chun-Li | Street Fighter | Lightning legs multi-hit, fast jab chain, strong air normals | Many-hit light specials, `air` kicks with forward drift |
| Cammy | Street Fighter | Spiral arrow dive, cannon strike, fast footsies | Diving air kick with angled velocity (`falcondive`) |
| Akuma | Street Fighter | Air fireball, demon flip, teleport, low health | Air versions of every special, `teleport` ability |
| Sol Badguy | Guilty Gear | Gunflame, Volcanic Viper, Roman Cancel | Cancelable specials, invuln uppercut |
| Ky Kiske | Guilty Gear | Stun Edge projectile, Vapor Thrust, toolbox | Projectile + anti-air pair |
| Chipp Zanuff | Guilty Gear | Teleport, wall cling, dash strings | Fast mobility with teleport and super speed |
| May | Guilty Gear | Dolphin ride, Beach Ball, command jump | Mixed mobility specials |
| Jack-O | Guilty Gear | Servants, install | Helper style projectiles |
| Ramlethal | Guilty Gear | Swords, long reach | Long-reach normals |
| Testament | Guilty Gear | Trap setups | `trap` projectiles |
| Goku | Dragon Ball FighterZ | Auto combo, super dash, Kamehameha, Instant Transmission | Light auto string, super speed, teleport |
| Vegeta | Dragon Ball FighterZ | Galick Gun, Final Flash, fast low pokes | Beam supers, rage |
| Gogeta | Dragon Ball FighterZ | Mixed beams and rushes | Multi-hit supers with invuln |
| Cell | Dragon Ball FighterZ | Beam, regenerate, Solar Flare | Life regeneration passive |
| Frieza | Dragon Ball FighterZ | Death Beam, Death Ball | Piercing multi-hit projectile |
| Kyo Kusanagi | King of Fighters | Rekka series, jump-in, fast pokes | Special that cancels into follow-ups |
| Iori Yagami | King of Fighters | Command dash, Scumgale, rage | Rage passive, forward lunge |
| Mai Shiranui | King of Fighters | Fan projectile, flying squirrel dive | Air dive and fan projectile |
| K' | King of Fighters | Fast mid-range kicks, flame | Fast pokes + projectiles |
| Athena | King of Fighters | Psycho Ball, teleport | Teleport |
| Terry Bogard | Fatal Fury | Power Wave, Burn Knuckle, Rising Tackle | Ground wave, command rush, invuln uppercut |
| Hyde | Under Night In-Birth | Fast gap close, strong lows | Run-in strikes |
| Yuzuriha | Under Night In-Birth | Evasive dashes, lone action | Evasive dash |
| Wolverine | Marvel vs Capcom 3 | Berserker Slash, Tornado Claw, Drill Claw | Dive and rush specials, regeneration |
| Zero | Marvel vs Capcom 3 | Rekkoha, Hyper Zero Rush | Many-hit rush super |
| Dante | Marvel vs Capcom 3 | Style cancels, juggling | Cancel routes between moves |
| Fox / Sheik | Smash Bros | Fast aerials, short hops | Air-first neutral |
| Scorpion | Mortal Kombat | Spear, teleport punch | Pull projectile, `teleport` |
| Sub-Zero | Mortal Kombat | Freeze ball, slide, clone | Slide low (`slide`) + freeze projectile |
| Peacock | Skullgirls | Trap projectiles, screen filling | Many small projectiles |
| Akira Yuki | Virtua Fighter | Single-hit heavy hits, strong throws | Strong launch hits, grabs |
| Jacky Bryant | Virtua Fighter | Fast kicks, juggles | Fast kick chains |

## Strong fighters (power, armor, grabs)

| Fighter | Game | Defining tools | UMF take-away |
|---|---|---|---|
| Zangief | Street Fighter | Spinning Piledriver, command grabs, lariat | Command grab, lariat, armor |
| Sagat | Street Fighter | Tiger Shot, Tiger Uppercut | High/low projectile pair, invuln uppercut |
| M. Bison | Street Fighter | Psycho Crusher, Scissor Kick | Armored charge (`shoulder`) |
| Guile | Street Fighter | Sonic Boom, Flash Kick, charge | Charge motion, invuln anti-air |
| Broly | Dragon Ball FighterZ | Armor, Gigantic Meteor | `armor`, rage passive |
| Heihachi | Tekken | Electric wind god fist, launchers, power crush | Launcher normals, armored move |
| Kazuya | Tekken | Electric wind god fist, hellsweep | Fast mid launcher |
| Jin Kazama | Tekken | Mid launchers, Devil form | Forms |
| Law | Tekken | Dragon kicks, fast mids | Kick strings |
| Paul | Tekken | Deathfist, big launcher | Heavy single hit |
| Nina / Xiaoyu | Tekken | Fast pokes, stances | Stance-like follow-ups |
| Liu Kang | Mortal Kombat | Bicycle kick, flying kick | Dive kick special |
| Raiden | Mortal Kombat | Torpedo, lightning | Rush and projectile |
| Haohmaru | Samurai Shodown | Heavy slashes, rage | Few strong hits, rage |
| Nakoruru | Samurai Shodown | Hawk companion, fast attacks | Helper projectile |
| Ragna | BlazBlue | Gauntlet, Inferno Divider, life drain | Life drain passive |
| Magneto | Marvel vs Capcom 3 | Beams, flight | `flight` |
| Filia | Skullgirls | Hair projectiles | Hair rush |
| Hulk / Colossus | Marvel | Armored rushes, armor | `armor` hits |
| Ryu | Street Fighter | Fireball, uppercut, spinning kick | Canonical special set |

## Patterns turned into engine features

1. Dash attacks and armored charges (`lunge`, `shoulder`) as command normals that still cancel into specials.
2. Air dives with angled velocity and ground bounce (`falcondive`, `meteordrop`).
3. Specials that cancel into other specials through explicit `cancels` routes.
4. Invulnerable anti-airs (`invuln`) and one-hit armor on slow, heavy moves.
5. Fast fighters trade damage for mobility (low `hitstun` normals, `superSpeed`, `teleport`); strong fighters gain `armor`, `wallBounce` and heavy `level` hits.

## Applied to the native roster

| Group | Fighters | New moves |
|---|---|---|
| Fast | Jin, Akio, Lian, Suzume, Kaname, Haruki, Kaito, Volta | `lunge` (`F+y`, dash strike, 115 % of the heavy punch, cancels into specials) and `falcondive` (air `F+b`, angled dive) |
| Strong | Bruno, Garan, Daichi, Valdric | `shoulder` (`F+y`, one-hit armor, wall bounce, 120 % damage) and `meteordrop` (air `F+b`, ground bounce, 130 % damage) |

Only data and `moves` changed: no sprite, pose or stage was touched; every move reuses an existing animation.

## How these fighters are made from scratch

1. **Design the kit**: normals (light, medium, heavy, crouching, air), 3-4 specials, 1-2 supers, throw, dash.
2. **Art**: pixel art in Aseprite or GraphicsGale with one shared palette (index 0 transparent), 3D prerender with an orthographic camera, or emulator rips. Crop tight and set the axis on the feet.
3. **Package**: Fighter Factory loads PCX/PNG into an SFF; a `.air` file sets frames and Clsn boxes; `.cmd` holds inputs; `.cns` holds states.
4. **Frame data**: light normals startup 3-5 ticks, heavies 8-14; hitstop 9-13 ticks.
5. **Balance and test**: training mode with hitboxes, combo routes, AI.

See `.agents/skills/mugen-character-creator` and `.agents/skills/mugen-to-ume-migrator`.

## Sources

- [SRK wiki](https://srk.shib.live/w/Ultra_Street_Fighter_IV/Chun-Li), [Dustloop](https://dustloop.com/w/GGST/Offense), [StrategyWiki KOF XIII](https://strategywiki.org/wiki/The_King_of_Fighters_XIII/Moves), [MKKomplete](https://icemantraveler.github.io/mkksg/mk4/default.htm), [Fighters Generation](https://fightersgeneration.com/games/tekken8.html), [MUGEN Guild](https://mugenguild.com/forum/topics/how-make-jus2d-character-pc-199356.0.html).
