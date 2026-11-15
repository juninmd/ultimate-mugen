# Combo Tiers and Energy Notch

The combo counter shows a tier badge under the hit count. Tiers are computed in `src/hud/combo_tier.rs`.

| Hits | Badge |
|---|---|
| 3-5 | GOOD |
| 6-9 | GREAT! |
| 10-14 | SUPERB! |
| 15+ | CLIMAX!! |

A `COUNTER!` badge is drawn above the tier when the opening hit was a counter hit.

The energy bar has a notch at 500 power, the cost of a [Rush Cancel](Rush-Cancel.md).

## Roster mobility and bounces

- Suzume and Jin have `stats.airDashes = 2`.
- Wall bounce: Suzume `swallow`, Jin `gustdash`, Kaito `tempest`, Garan `ascent`, Bruno `bullrush`, Shion `cleave`, Daichi `daybreak`.
- Ground bounce: Garan `brimstone`, Daichi `landslide`.
