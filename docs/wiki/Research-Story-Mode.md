# Story mode research notes

What was found (fan wikis and reviews, not an official guide) and how it maps to the engine.

- **Final Fight (arcade, 1989).** Six rounds plus two bonus rounds, a boss at the end of each round: Damnd (Slums), Sodom (Subway/Park), Edi. E (West Side; one source lists Andore and Edi. E together), Rolento (Industrial Area), Abigail (Bay Area) and Belger (Uptown). Thugs come in waves, weapons and food are picked up along the way.
- **Capcom Arcade Stadium.** Rewind, save states, slow-down/turbo and adjustable difficulty; credits are effectively unlimited.
- **Sengoku 2.** Sword is the default weapon with high/low attacks and a parry; bosses are larger and mark story points. Stage-by-stage boss lists were not found.

Not verified: the exact enemy roster of each Final Fight stage beyond the first three, Sengoku mid-bosses, and Arcade Stadium specifics beyond the shared features above.

Depth (beat 'em up convention, from community dev guides, not Final Fight's own code): characters walk a floor band, attacks test a depth tolerance first and then the usual box overlap, and a thin tolerance makes hits miss unless fighters are nearly aligned, so the tolerance is kept generous (14 units) and the CPU aligns itself to the target.

Mapping: five stages (not six) of three thug waves plus a boss, carried life with recovery between waves and stages (food), three credits (continues). Weapons, bonus rounds and rewind are not implemented.
