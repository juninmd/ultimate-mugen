# Volta auxiliary correction prompts

Generate exactly one standalone combat sprite per call. Attach `assets/chars/sprites/volta/04_special.png` first as the sole authority for Volta identity, black/gunmetal armor, cyan visor and armor panels, yellow accent placements, proportions, palette and pixel-art finish. Attach the named current target second for pose guidance only. Preserve its pose and direction while replacing the white/yellow armor set with the canonical black/cyan armor. Do not add lightning, aura, glow, effects, scenery, text or shadows unless the target itself explicitly contains them. Keep hard-edged pixel clusters, no antialiasing, transparent background, full body, 400×400 normalized canvas. Preserve the target frame geometry in `meta.json`; do not promote until compared with the anchor and neighboring frames at native and enlarged sizes. Each candidate maps only to its named target.

## One-pose targets

| Target file | Affected animations | Current SHA-256 |
| --- | --- | --- |
| `airGuard.png` | airGuard, airGuardHit, airGuardStart | `1303d212d5390181eacec95c0da6958a2ac0b6195efcd9530df968839731a06e` |
| `airGuardHit.png` | airGuardHit | `9ffc6e2c9f2b9f359cff801c2d40314380c61deb78bad0d4dfe108072a9cad01` |
| `crouch.png` | cjab, clk, crouch, crouchStart, crouchTurn, hitCrouch, lowKick, sweep | `73c92b0ee34d105a5093f9e96eab729f4fcd8447b50c0115da9deaea39019b94` |
| `crouch0.png` | crouchEnd, crouchStart, lowKick | `4af033470775247564421e59ef44792e8fbe45302805fc7eb649672a92b16218` |
| `crouch1.png` | clk, crouch, lowKick | `c6c82ff45188a5251cb9780dfe870db68e47a9601113aefa8077a6dbd932f18e` |
| `crouchGuard.png` | crouchGuard, crouchGuardHit, crouchGuardStart | `d3666a3086196f3b1c2cb7e00025a1086f022fa6ef3498b9b1931a1cb34f6175` |
| `crouchGuardHit.png` | crouchGuardHit | `a66624192381ccbd6b8c5c07c84a4e0db5487c5f7052720deb60c6efd2d3bec3` |
| `dfKneel0.png` | defeat3 | `59ffcbb0425942f661a660fd07b61b93723006fcfc146564690b0c5722a238a4` |
| `dfKneel1.png` | defeat3 | `0f7719c4187d31f71546ff37352e8576274b9a9365f264a3c6a1bbb617cb1e19` |
| `dfLie.png` | defeat1, defeat2, defeat3, defeat4 | `7f915a4c538339eb1ee6a20442060aff48e0e35e1939ad35094a220ef4f2167e` |
| `dfProp.png` | defeat1 | `bffc3d72b3090da500ab20c107fcd64f1393956c7e17bde398a1d469bc95dffb` |
| `dfProp2.png` | defeat1 | `068c5f1201b32f8f1da77a2a39212d23334a9d1f661515601da3223ae73aa151` |
| `dfReach0.png` | defeat4 | `0a43bc88c6c2e158883aba93fd6abe46b89879b6f8c9c47c03d8b91b72a41412` |
| `dfReach1.png` | defeat4 | `8e982f5217a75808ba2943bb1148e9819e5d794091230aa25ad496d187b388bd` |
| `dfSit0.png` | defeat2 | `700a49e1b0f4dacbad16857406c0e728b5a0f3dfc8114c33b46037b3d37f1b5c` |
| `dfSit1.png` | defeat2 | `cd88680a52114349be89f4cc67899ed377fbf85e98e33c539383cc35c7a75825` |
| `getUp1.png` | getUp | `43388839d4e437c6cbb2a45f4b0904904715e6933c0e788a6177275af731401d` |
| `getUp2.png` | getUp | `51a368330474c812e7582490a0fd2aed4f3f4fce2eac0290e95e69186d31e687` |
| `getUp3.png` | getUp | `4af033470775247564421e59ef44792e8fbe45302805fc7eb649672a92b16218` |
| `guard.png` | capacitor, guard, guardEnd, guardHit, guardStart | `08d084be535c99706570b73d73318a90a6f8f8b83798ffbdfe26678f084163af` |
| `guardHit.png` | guardHit | `c3eb5e211b7949ee3cbf1aef7e052f724b64084b74dee0ccf278f6d8adc54cc2` |
| `hitCrouch.png` | hitCrouch | `7138f3cfa723a1e13f281ab0cdc6ff026d37076bc04e7ba4cf5182e243ac255f` |
| `hitL1.png` | hitLowHeavy, hitLowLight, hitLowMedium | `430606784940efc7bc7b5f979b13afd0008dc0142a730e9b12545bd41351c490` |
| `hitL2.png` | hitLowHeavy, hitLowMedium | `7a538ae6dd37569634915af0a4c97617216c15fe16dfff80697eb9eba96f0151` |
| `hitL3.png` | hitLowHeavy | `a1ba5a5a183af7d00c3557ab8c18ff4a74099073bc65627118e665aff2406e20` |
| `jumpDown.png` | airLowKick, airRecover, ajab, akick, jumpDown, tesla, teslaEx | `8d6ff915c0796d277a6b8f03b9272668a03ef1a41ea622a1761b316546a01015` |
| `land.png` | land | `c8cf8eec49c25e6014a29a75abb23c24f1a94875529392e952bee48d274db4ab` |
| `turn.png` | turn | `202d554adb03d323c9ad9e7a982a7c464518b217b0edbd2ab8f508871c370027` |

Start with one representative guarding pose as a pilot. Review it before generating any other pose; retain source output and normalized candidate separately from the tracked sprite files.
